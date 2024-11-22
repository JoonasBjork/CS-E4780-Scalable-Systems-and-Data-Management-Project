use crate::data_entry::DataEntry;
use crate::quantitative_indicators::QuantitativeIndicator;

use postgres::types::ToSql;
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

pub fn insert_alerts(client: &mut Client, dataentries: &[DataEntry]) -> Result<u64, Error> {
    let mut query =
        String::from("INSERT INTO alerts (symbol, sectype, last, trading_timestamp) VALUES ");

    let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();

    let mut query_idx = 1;
    for (i, dataentry) in dataentries.iter().enumerate() {
        if i > 0 {
            query.push_str(", ");
        }

        query.push_str(&format!(
            "(${}, ${}, ${}, ${})",
            query_idx,
            query_idx + 1,
            query_idx + 2,
            query_idx + 3
        ));
        query_idx += 4;

        params.push(&dataentry.id);
        params.push(&dataentry.sectype);
        params.push(&dataentry.last);
        params.push(&dataentry.timestamp);
    }

    client.execute(&query, &params)
}

pub fn insert_indicators(
    client: &mut Client,
    indicators: &[(String, QuantitativeIndicator)],
) -> Result<u64, Error> {
    let mut query = String::from(
        "INSERT INTO indicators (symbol, ema_38, ema_100, bullish, bearish, last_trade_timestamp) VALUES "
    );

    let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();

    let mut query_idx = 1;
    for (i, (id, quantitative_indicator)) in indicators.iter().enumerate() {
        if i > 0 {
            query.push_str(", ");
        }
        query.push_str(&format!(
            "(${}, ${}, ${}, ${}, ${}, ${})",
            query_idx,
            query_idx + 1,
            query_idx + 2,
            query_idx + 3,
            query_idx + 4,
            query_idx + 5
        ));
        query_idx += 6;

        params.push(id);
        params.push(&quantitative_indicator.ema_38);
        params.push(&quantitative_indicator.ema_100);
        params.push(&quantitative_indicator.bullish);
        params.push(&quantitative_indicator.bearish);
        params.push(&quantitative_indicator.most_recent_value_timestamp);
    }

    let params_refs: Vec<&(dyn ToSql + Sync)> = params.iter().map(|b| &**b).collect();

    client.execute(&query, &params_refs).map_err(|e| e.into())
}

pub fn wait_for_migration(
    client: &mut Client,
    table_names: &[&str],
    retries: u64,
    wait_time_in_seconds: u64,
) -> Result<(), ()> {
    let mut already_tried: u64 = 0;

    let query = "SELECT tablename
                 FROM pg_catalog.pg_tables
                 WHERE schemaname = 'public'
                 AND tablename = ANY($1);";

    loop {
        let res: Result<Vec<postgres::Row>, Error> = client.query(query, &[&table_names]);

        match res {
            Ok(rows) => {
                let db_table_names: Vec<String> = rows
                    .iter()
                    .map(|row| row.get::<_, String>("tablename"))
                    .collect();

                let all_tables_present = table_names
                    .iter()
                    .all(|&table_name| db_table_names.contains(&table_name.to_string()));

                if all_tables_present {
                    println!("All tables are present!");
                    return Ok(());
                } else {
                    println!(
                            "Postgre missing tables: {:?}",
                            table_names
                                .iter()
                                .filter(|&&table_name| !db_table_names.contains(&table_name.to_string()))
                                .collect::<Vec<&&str>>()
                        );
                }
            }
            Err(_) => {
                println!("Got error when querying psql tables");
            }
        }

        if already_tried >= retries {
            return Err(());
        }
        println!(
            "Retrying to check if psql migration has been succesful {}",
            wait_time_in_seconds
        );
        already_tried += 1;
        thread::sleep(Duration::from_secs(wait_time_in_seconds));
    }
}
