use crate::data_entry::DataEntry;

use postgres::{Client, Error, NoTls};
use std::thread;
use std::time::Duration;

pub fn create_postgres_client(
    connection_string: String,
    retries: u64,
    wait_time_in_seconds: u64,
) -> Result<Client, Error> {
    //host: String, user: String
    // let connection_string = format!("host={} user={}", host, user);
    let mut already_tried: u64 = 0;
    let mut client_connection: Result<Client, Error>;

    loop {
        client_connection = Client::connect(&connection_string, NoTls);

        if already_tried >= retries || client_connection.is_ok() {
            break;
        }
        already_tried += 1;
        thread::sleep(Duration::from_secs(wait_time_in_seconds));
    }

    return client_connection;
}

pub fn add_alert(client: &mut Client, dataentry: DataEntry) -> Result<u64, Error> {
    let query =
        "INSERT INTO alerts (symbol, sectype, last, trading_timestamp) VALUES ($1, $2, $3, $4)";
    client.execute(
        query,
        &[
            &dataentry.id,
            &dataentry.sectype,
            &dataentry.last,
            &dataentry.timestamp,
        ],
    )
}
