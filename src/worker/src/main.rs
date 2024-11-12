use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{Commands, RedisError, RedisResult};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
// use std::time::Duration;

fn main() -> Result<(), RedisError> {
    // Shared data structure between threads: a vector protected by a Mutex
    println!("Step 1");
    let data = Arc::new((Mutex::new(Vec::new()), Condvar::new()));
    println!("Step 2");

    // Redis connection
    let client = redis::Client::open("redis://redis-streams:6379/")?;
    println!("Step 3");
    let mut con = client.get_connection()?;
    println!("Created connection");
    let stream_key = "s1";
    let mut latest_id: String = "0".to_string();

    let options = StreamReadOptions::default().block(5000);
    let mut iter: i32 = 0;

    let data_producer = Arc::clone(&data);
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
                            for (field, value) in entry.map {
                                let (lock, cvar) = &*data_producer;
                                let mut vec = lock.lock().unwrap();
                                vec.push(value.clone()); // Push the value to the shared vector

                                cvar.notify_one();
                                println!("ID: {}, Field: {}, value: {:?}", entry.id, field, value)
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Error reading from Redis: {}", e);
                }
            }

            // thread::sleep(Duration::new(1, 0));
        }
    });

    // Clone the Arc to pass into the consumer thread
    let data_consumer = Arc::clone(&data);
    let consumer = thread::spawn(move || {
        loop {
            let (lock, cvar) = &*data_consumer;

            // Lock the mutex and wait for a notification
            let mut vec = lock.lock().unwrap();
            while vec.is_empty() {
                // Wait until the condition variable is notified
                vec = cvar.wait(vec).unwrap();
            }

            // Consume the item
            if let Some(item) = vec.pop() {
                println!("Consumer consumed: {:?}", item);
            }
        }
    });

    // Wait for both threads to finish (they won't in this case since they're infinite loops)
    producer.join().unwrap();
    consumer.join().unwrap();

    Ok(())
}
