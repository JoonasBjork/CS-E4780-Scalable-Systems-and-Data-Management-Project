mod data_entry;
mod postgres_connector;
mod quantitative_indicators;

use postgres_connector::{create_postgres_client, insert_alert, insert_indicator};
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Commands, RedisError, RedisResult};
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use std::{env, process};

use data_entry::DataEntry;
use quantitative_indicators::{QuantitativeIndicator, TimestampedValue};

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
    let redis_options = StreamReadOptions::default().block(5000);

    let mut latest_id: String = "0".to_string();

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
            println!(
                "Reading values from stream {}, iter {}, latest_id {}",
                &stream_key, iter, latest_id
            );
            iter += 1;
            let result: RedisResult<StreamReadReply> =
                redis_con.xread_options(&[&stream_key], &[latest_id.clone()], &redis_options); // Use latest_id

            match result {
                Ok(messages) => {
                    // println!("Received messages: {:?}", messages);
                    for stream in messages.keys {
                        for entry in stream.ids {
                            latest_id = entry.id.clone();

                            let record_object = DataEntry::from_redis_map(&entry.map);

                            if record_object.is_ok() {
                                // All of the fields have been defined
                                // println!("Got ok DataRecord: {}", record_object);

                                // println!("Producer taking read_lock");
                                let read_lock = ema_producer_data.read().unwrap();

                                let opt_lock = read_lock.get(record_object.id.as_ref().unwrap());

                                // let opt_lock =
                                // ema_producer_data.get();

                                if let Some(lock) = opt_lock {
                                    // Update the quantitative_indicator's most_recent_value
                                    let mut quantitative_indicator = lock.lock().unwrap();

                                    quantitative_indicator.most_recent_value =
                                        Some(TimestampedValue::new(
                                            record_object.last.unwrap(),
                                            record_object.timestamp.unwrap(),
                                        ));
                                } else {
                                    // Create the quantitativeIndicator Object and add it to the ema_producer_data

                                    // The read lock must be dropped here, or otherwise the thread will be deadlocked, as the thread will acquire a write_lock next.
                                    drop(read_lock);

                                    // println!("Producer taking write_lock");
                                    let mut write_lock = ema_producer_data.write().unwrap();

                                    write_lock.insert(
                                        record_object.id.as_ref().unwrap().clone(),
                                        Mutex::new(QuantitativeIndicator::new(Some(
                                            TimestampedValue::new(
                                                record_object.last.unwrap(),
                                                record_object.timestamp.unwrap(),
                                            ),
                                        ))),
                                    );
                                }
                            } else {
                                // All of the fields have not been defined and there is an error. Send the data to the alerts
                                // println!("Got partial DataRecord: {}", record_object);

                                let (lock, cvar) = &*alert_producer_data;
                                let mut vec = lock.lock().unwrap();

                                vec.push(record_object); // Push the value to the shared vector

                                cvar.notify_one();
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
        thread::sleep(Duration::from_secs(10));

        let mut postgres_client = create_postgres_client(
            "postgres://username:password@database:5432/database".to_string(),
            3,
            10,
        )
        .expect("Alert consumer was not able to connect to Postgres");

        loop {
            let (lock, cvar) = &*alert_consumer_data;

            // Lock the mutex and wait for a notification
            let mut vec = lock.lock().unwrap();
            while vec.is_empty() {
                // Wait until the condition variable is notified
                vec = cvar.wait(vec).unwrap();
            }

            // Consume the item
            if let Some(item) = vec.pop() {
                // println!("alert_consumer consumed: {:?}", item);
                let alert_res = insert_alert(&mut postgres_client, item);
                match alert_res {
                    Ok(changed_lines) => {
                        // println!("Alert_consumer added {} lines in postgres", changed_lines)
                    }
                    Err(e) => println!("Alert_consumer got error while adding to postgres: {}", e),
                }
            }
        }
    });

    // ********** Ema consumer thread **********

    let ema_consumer_data = Arc::clone(&ema_data);
    let ema_consumer: thread::JoinHandle<_> = thread::spawn(move || loop {
        let mut postgres_client = create_postgres_client(
            "postgres://username:password@database:5432/database".to_string(),
            3,
            10,
        )
        .expect("Ema consumer was not able to connect to Postgres");

        thread::sleep(Duration::from_secs(10));
        // println!("Ema consumer taking read_lock");
        let read_lock = ema_consumer_data.read().unwrap();
        for (id, mutex) in read_lock.iter() {
            let mut indicator = mutex.lock().unwrap();
            indicator.calculate_both_emas();
            let indicator_ref = &*indicator;

            let insert_indicator_res = insert_indicator(&mut postgres_client, id, indicator_ref);

            match insert_indicator_res {
                Ok(changed_lines) => {
                    // println!("EMA consumer added {} lines in postgres", changed_lines)
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
