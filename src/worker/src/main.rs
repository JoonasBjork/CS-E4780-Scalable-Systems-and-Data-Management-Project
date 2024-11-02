use redis::{Commands, RedisError};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

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
    let mut latest_id_res: Result<Vec<(String, Vec<(String, String)>)>, RedisError>;
    let mut latest_id: String;

    loop {
        latest_id_res = con.xrevrange_count(stream_key, "+", "-", 1);

        match latest_id_res {
            Ok(vec) if !vec.is_empty() => {
                println!("Received a non-empty result.");
                latest_id = vec[0].0.clone();
                println!("Retrieved latest_id from Redis: {}", latest_id);
                break;
            }
            Ok(_) => {
                println!(
                    "The latest ID result is empty for stream {}. Waiting for new messages...",
                    stream_key
                );
            }
            Err(err) => {
                println!("Error occurred in fecthing latest id: {}", err);
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let data_producer = Arc::clone(&data);
    let producer = thread::spawn(move || {
        loop {
            println!("Starting a new read iteration");
            // Retrieve messages from the Redis stream starting from the last ID
            let result: Result<Vec<(String, Vec<(String, String)>)>, RedisError> =
                con.xread(&[stream_key], &[latest_id.clone()]); // Use latest_id

            match result {
                Ok(messages) => {
                    println!("Received messages: {:?}", messages);
                    for message in messages {
                        let (id, fields) = message; // Each message has an ID and fields

                        // Iterate over all key-value pairs in fields
                        for (key, value) in fields {
                            let (lock, cvar) = &*data_producer;
                            let mut vec = lock.lock().unwrap();
                            vec.push(value.clone()); // Push the value to the shared vector

                            // Notify the consumer that an item has been added
                            cvar.notify_one();
                            println!("Producer added an item: Key: {}, Value: {}", key, value);
                        }

                        // Update latest_id to the ID of the processed message
                        latest_id = id; // Update the latest ID after processing
                    }
                }
                Err(e) => {
                    println!("Error reading from Redis: {}", e);
                }
            }

            // Sleep for a while to avoid busy waiting
            thread::sleep(Duration::new(1, 0));
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
                println!("Consumer consumed: {}", item);
            }
        }
    });

    // Wait for both threads to finish (they won't in this case since they're infinite loops)
    producer.join().unwrap();
    consumer.join().unwrap();

    Ok(())
}
