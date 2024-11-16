mod data_entry;
mod quantitative_indicators;

use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Commands, RedisError, RedisResult};
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;

use data_entry::DataEntry;
use quantitative_indicators::{QuantitativeIndicator, TimestampedValue};

use std::time::Duration;

fn main() -> Result<(), RedisError> {
    // Shared data structure between threads for alerts: a vector protected by a Mutex
    let alert_data = Arc::new((Mutex::new(Vec::new()), Condvar::new()));

    let ema_data = Arc::new(RwLock::new(
        HashMap::<String, Mutex<QuantitativeIndicator>>::new(),
    ));

    // Redis connection
    let client = redis::Client::open("redis://redis-streams:6379/")?;
    let mut con = client.get_connection()?;
    println!("Created redis connection");
    let stream_key = "s1";
    let mut latest_id: String = "0".to_string();

    let options = StreamReadOptions::default().block(5000);
    let mut iter: i32 = 0;

    let alert_producer_data = Arc::clone(&alert_data);
    let ema_producer_data = Arc::clone(&ema_data);

    let producer = thread::spawn(move || {
        loop {
            // Retrieve messages from the Redis stream starting from the last ID
            println!(
                "Reading values from stream {}, iter {}, latest_id {}",
                stream_key, iter, latest_id
            );
            iter += 1;
            let result: RedisResult<StreamReadReply> =
                con.xread_options(&[stream_key], &[latest_id.clone()], &options); // Use latest_id

            match result {
                Ok(messages) => {
                    println!("Received messages: {:?}", messages);
                    for stream in messages.keys {
                        for entry in stream.ids {
                            latest_id = entry.id.clone();

                            let record_object = DataEntry::from_redis_map(&entry.map);

                            if record_object.is_ok() {
                                // All of the fields have been defined
                                println!("Got ok DataRecord: {}", record_object);

                                println!("Producer taking read_lock");
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

                                    println!("Producer taking write_lock");
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
                                println!("Got partial DataRecord: {}", record_object);

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

    // The alert consumer consumes error events as they get added to that
    let alert_consumer_data = Arc::clone(&alert_data);
    let alert_consumer = thread::spawn(move || {
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
                println!("alert_consumer consumed: {:?}", item);
            }
        }
    });

    let ema_consumer_data = Arc::clone(&ema_data);
    let ema_consumer: thread::JoinHandle<_> = thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(10));
        println!("Ema consumer woke up");

        println!("Ema consumer taking read_lock");
        let read_lock = ema_consumer_data.read().unwrap();
        for (id, mutex) in read_lock.iter() {
            println!("Ema consumer");
            let indicator = mutex.lock().unwrap();

            println!("EMA CONSUMER id: {:?}, indicator: {}", id, indicator);
        }
    });

    // Wait for both threads to finish (they won't in this case since they're infinite loops)
    producer.join().unwrap();
    alert_consumer.join().unwrap();
    ema_consumer.join().unwrap();

    Ok(())
}
