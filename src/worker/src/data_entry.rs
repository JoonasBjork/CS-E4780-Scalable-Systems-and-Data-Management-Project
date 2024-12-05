use chrono::NaiveDateTime;
use redis::Value;
use std::collections::HashMap;
use std::fmt;

fn string_from_value(ov: Option<&Value>) -> Option<String> {
    match ov {
        Some(v) => match v {
            Value::BulkString(s) => {
                if s.is_empty() {
                    None
                } else {
                    Some(String::from_utf8_lossy(s).to_string())
                }
            }
            _ => None,
        },
        None => None,
    }
}

fn float_from_value(ov: Option<&Value>) -> Option<f64> {
    match ov {
        Some(v) => match v {
            Value::BulkString(f) => {
                // In case that the input is given as a string and not as a float
                match std::str::from_utf8(f) {
                    Ok(s) => match s.parse::<f64>() {
                        Ok(float_value) => {
                            if !float_value.is_finite() {
                                // println!("GOT NAN/INF FLOAT VALUE");
                                return None;
                            }
                            Some(float_value)
                        }
                        Err(_) => None,
                    },
                    Err(_) => None,
                }
            }
            // Usually the value should be a f64
            Value::Double(f) => Some(f.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn timestamp_from_value(otime: Option<&Value>, odate: Option<&Value>) -> Option<NaiveDateTime> {
    let stime = string_from_value(otime);
    let sdate = string_from_value(odate);

    // println!("stime: {:?}, sdate: {:?}", stime, sdate);

    if let (Some(time), Some(date)) = (stime, sdate) {
        let timestamp_str = format!("{} {}", date, time);
        let format = "%d-%m-%Y %H:%M:%S.%f";

        match NaiveDateTime::parse_from_str(&timestamp_str, format) {
            Ok(datetime) => Some(datetime),
            Err(_) => None,
        }
    } else {
        return None;
    }
}

#[derive(Debug)]
pub struct DataEntry {
    pub id: Option<String>,
    pub sectype: Option<String>,
    pub last: Option<f64>,
    pub timestamp: Option<NaiveDateTime>,
}

impl DataEntry {
    pub fn from_redis_map(data: &HashMap<String, Value>) -> Self {
        let id = string_from_value(data.get("id"));
        let sectype = string_from_value(data.get("sectype"));
        let last = float_from_value(data.get("last"));
        let timestamp = timestamp_from_value(data.get("time"), data.get("date"));

        DataEntry {
            id,
            sectype,
            last,
            timestamp,
        }
    }

    pub fn is_ok(&self) -> bool {
        // println!("self.id.is_some() {}, self.sectype.is_some() {}, self.last.is_some() {}, self.timestamp.is_some() {}", self.id.is_some(), self.sectype.is_some(), self.last.is_some(), self.timestamp.is_some());
        self.id.is_some()
            && self.sectype.is_some()
            && self.last.is_some()
            && self.timestamp.is_some()
    }
}

impl fmt::Display for DataEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use match to check if each field is Some or None, and format accordingly
        write!(
            f,
            "ID: {:?}, Sector Type: {:?}, Last: {:?}, Date: {:?}",
            self.id, self.sectype, self.last, self.timestamp
        )
    }
}
