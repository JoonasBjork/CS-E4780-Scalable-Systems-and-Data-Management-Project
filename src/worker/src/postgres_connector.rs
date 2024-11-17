use crate::data_entry::DataEntry;
use crate::quantitative_indicators::QuantitativeIndicator;

use postgres::{Client, Error, NoTls};
use std::thread;
use std::time::Duration;

pub fn create_postgres_client(
    connection_string: String,
    retries: u64,
    wait_time_in_seconds: u64,
) -> Result<Client, Error> {
    let mut already_tried: u64 = 0;
    let mut client_connection: Result<Client, Error>;

    loop {
        // println!("Trying to connect to psql");
        client_connection = Client::connect(&connection_string, NoTls);

        if already_tried >= retries || client_connection.is_ok() {
            break;
        }
        println!("Retrying to connect to psql in {}", wait_time_in_seconds);
        already_tried += 1;
        thread::sleep(Duration::from_secs(wait_time_in_seconds));
    }

    return client_connection;
}

pub fn insert_alert(client: &mut Client, dataentry: DataEntry) -> Result<u64, Error> {
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

pub fn insert_indicator(
    client: &mut Client,
    id: &String,
    quantitative_indicator: &QuantitativeIndicator,
) -> Result<u64, Error> {
    let query =
        "INSERT INTO indicators (symbol, ema_38, ema_100, bullish, bearish, last_trade_timestamp) VALUES ($1, $2, $3, $4, $5, $6)";
    client.execute(
        query,
        &[
            id,
            &quantitative_indicator.ema_38,
            &quantitative_indicator.ema_100,
            &quantitative_indicator.bullish,
            &quantitative_indicator.bearish,
            &quantitative_indicator
                .most_recent_value
                .as_ref()
                .map(|timestamped_value| timestamped_value.timestamp),
        ],
    )
}