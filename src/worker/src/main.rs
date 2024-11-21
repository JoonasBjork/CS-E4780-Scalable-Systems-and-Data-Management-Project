mod data_entry;
mod postgres_connector;
mod quantitative_indicators;

use postgres_connector::{
    create_postgres_client, insert_alerts, insert_indicators, wait_for_migration,
};
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Commands, RedisError, RedisResult};
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::time::Duration;
use std::{env, process, thread};

use data_entry::DataEntry;
use quantitative_indicators::QuantitativeIndicator;

fn main() -> Result<(), RedisError> {
    // ********** Setting up the chosen stream **********

    let replica_index: i32 = match env::var("REPLICA_INDEX") {
        Ok(value) => match value.parse::<i32>() {
            Ok(int) => int,
            Err(_) => {
                eprintln!("Error: REPLICA_INDEX must be a valid integer.");
                process::exit(1);
            }
        },
        Err(_) => {
            eprintln!("Error: REPLICA_INDEX environment variable is not set.");
            process::exit(1);
        }
    };

    let stream_key = format!("s{}", replica_index);

    // ********** Opening a Redis Client connection **********

    let redis_client = redis::Client::open("redis://redis-streams:6379/")?;
    let mut redis_con = redis_client.get_connection()?;
    let redis_options = StreamReadOptions::default().block(5000).count(10); // In general, the data comes in slow enough that only one message is retrieved at a time.

    let mut latest_id: String = "0".to_string();

    // ********** Checking that migration has been applied to db **********

    let initial_postgres_client_res = create_postgres_client(
        "postgres://username:password@database:5432/database".to_string(),
        3,
        10,
    );

    let mut initial_postgres_client = match initial_postgres_client_res {
        Ok(c) => c,
        Err(e) => {
            println!("Could not connect to postgres client '{}', exiting...", e);
            process::exit(1);
        }
    };

    let migration_succesful = wait_for_migration(
        &mut initial_postgres_client,
        &["alerts", "indicators"],
        60,
        1,
    );

    match migration_succesful {
        Ok(_) => println!("Migration has been succesfully applied to psql"),
        Err(_) => println!("Migration has not been applied to psql"),
    }

    drop(initial_postgres_client);

    // ********** Setting up the shared data structures between the threads **********

    // Shared data structure between threads for alerts: a vector protected by a Mutex
    let alert_data = Arc::new((Mutex::new(Vec::new()), Condvar::new()));

    let ema_data = Arc::new(RwLock::new(
        HashMap::<String, Mutex<QuantitativeIndicator>>::new(),
    ));

    // ********** Producer thread **********

    let alert_producer_data = Arc::clone(&alert_data);
    let ema_producer_data = Arc::clone(&ema_data);

    let producer = thread::spawn(move || {
        let mut iter: i32 = 0;
        loop {
            // Retrieve messages from the Redis stream starting from the last ID

            let read_result: RedisResult<StreamReadReply> =
                redis_con.xread_options(&[&stream_key], &[latest_id.clone()], &redis_options); // Use latest_id

            match read_result {
                Ok(messages) => {
                    // println!("Received messages: {:?}", messages);
                    for stream in messages.keys {
                        for entry in stream.ids {
                            latest_id = entry.id.clone();

                            if iter % 100 == 0 {
                                println!(
                                    "Reading values from stream {}, iter {}, latest_id {}",
                                    &stream_key, iter, latest_id
                                );
                            }

                            iter += 1;

                            let del_result: RedisResult<i32> =
                                redis_con.xdel(&[&stream_key], &[latest_id.clone()]);

                            match del_result {
                                Ok(_) => {}
                                Err(e) => println!("ERROR IN del_result: {}", e),
                            }

                            let record_object = DataEntry::from_redis_map(&entry.map);

                            if record_object.is_ok() {
                                // println!("Data is ok");
                                // All of the fields have been defined
                                // println!("Got ok DataRecord: {}", record_object);

                                // println!("Producer taking read_lock");
                                // println!("DEBUG WORKER PRODUCER TAKING READ LOCK");
                                let read_lock = ema_producer_data.read().unwrap();

                                let quant_indicator_lock_option =
                                    read_lock.get(record_object.id.as_ref().unwrap());

                                if let Some(quant_indicator_lock) = quant_indicator_lock_option {
                                    // Update the quantitative_indicator's most_recent_value
                                    let mut quantitative_indicator =
                                        quant_indicator_lock.lock().unwrap();

                                    quantitative_indicator.most_recent_value = Some((
                                        record_object.last.unwrap(),
                                        record_object.timestamp.unwrap(),
                                    ));
                                    // println!("DEBUG WORKER PRODUCER DROPPING READ LOCK");
                                } else {
                                    // Create the quantitativeIndicator Object and add it to the ema_producer_data

                                    // The read lock must be dropped here, or otherwise the thread will be deadlocked, as the thread will acquire a write_lock next.
                                    // println!("DEBUG WORKER PRODUCER DROPPING READ LOCK");
                                    drop(read_lock);

                                    // println!("DEBUG WORKER PRODUCER TAKING WRITE LOCK");
                                    let mut write_lock = ema_producer_data.write().unwrap();

                                    write_lock.insert(
                                        record_object.id.as_ref().unwrap().clone(),
                                        Mutex::new(QuantitativeIndicator::new(Some((
                                            record_object.last.unwrap(),
                                            record_object.timestamp.unwrap(),
                                        )))),
                                    );
                                    // println!("DEBUG WORKER PRODUCER DROPPING WRITE LOCK")
                                }
                            } else {
                                // println!("Data not ok");
                                // All of the fields have not been defined and there is an error. Send the data to the alerts
                                // println!("Got partial DataRecord: {}", record_object);

                                let (lock, cvar) = &*alert_producer_data;
                                // println!("DEBUG WORKER ALERT PRODUCER TAKING LOCK");
                                let mut vec = lock.lock().unwrap();

                                vec.push(record_object); // Push the value to the shared vector

                                cvar.notify_one();
                                // println!("DEBUG WORKER ALERT PRODUCER DROPPING LOCK");
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Error reading from Redis: {}", e);
                }
            }
        }
    });

    // ********** Alert consumer thread **********

    // The alert consumer consumes error events as they get added to that
    let alert_consumer_data = Arc::clone(&alert_data);
    let alert_consumer = thread::spawn(move || {
        let mut postgres_client = create_postgres_client(
            "postgres://username:password@database:5432/database".to_string(),
            3,
            10,
        )
        .expect("Alert consumer was not able to connect to Postgres");

        loop {
            thread::sleep(Duration::new(1, 0));
            let (lock, cvar) = &*alert_consumer_data;

            // Lock the mutex and wait for a notification
            // println!("DEBUG WORKER ALERT_CONSUMER TAKING LOCK");
            let mut vec = lock.lock().unwrap();
            while vec.is_empty() {
                // Wait until the condition variable is notified
                vec = cvar.wait(vec).unwrap();
            }

            let entry_vec: Vec<DataEntry> = vec.drain(..).collect();

            // Release the lock
            // println!("DEBUG WORKER ALERT_CONSUMER DROPPING LOCK");
            drop(vec);

            let alert_res = insert_alerts(&mut postgres_client, &entry_vec);
            match alert_res {
                Ok(changed_lines) => {
                    println!(
                        "Alert_consumer added {} lines to postgre alerts",
                        changed_lines
                    )
                }
                Err(e) => {
                    println!("Alert_consumer got error while adding to postgres: {}", e)
                }
            }
        }
    });

    // ********** Ema consumer thread **********

    let ema_consumer_data = Arc::clone(&ema_data);
    let ema_consumer: thread::JoinHandle<_> = thread::spawn(move || {
        let mut postgres_client = create_postgres_client(
            "postgres://username:password@database:5432/database".to_string(),
            3,
            10,
        )
        .expect("Ema consumer was not able to connect to Postgres");

        loop {
            thread::sleep(Duration::new(10, 0));
            // println!("DEBUG WORKER EMA_CONSUMER TAKING READ LOCK");
            let mut quant_indicators: Vec<(String, QuantitativeIndicator)> = Vec::new();

            let read_lock = ema_consumer_data.read().unwrap();

            for (id, mutex) in read_lock.iter() {
                let mut indicator_ref = mutex.lock().unwrap();
                indicator_ref.calculate_both_emas();
                // let indicator: &QuantitativeIndicator = &*indicator_ref;
                quant_indicators.push((id.clone(), (*indicator_ref).clone()));
            }

            // println!("DEBUG WORKER EMA_CONSUMER DROPPING READ LOCK");
            drop(read_lock);

            let insert_indicators_res = insert_indicators(&mut postgres_client, &quant_indicators);
            match insert_indicators_res {
                Ok(changed_lines) => {
                    println!(
                        "EMA consumer added {} lines to postgre indicators",
                        changed_lines
                    )
                }
                Err(e) => println!("Ema consumer got error while adding to postgres: {}", e),
            }
        }
    });

    // Wait for both threads to finish (they won't in this case since they're infinite loops)
    producer.join().unwrap();
    alert_consumer.join().unwrap();
    ema_consumer.join().unwrap();

    Ok(())
}
