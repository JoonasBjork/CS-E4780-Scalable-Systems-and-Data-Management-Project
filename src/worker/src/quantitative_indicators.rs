use chrono::NaiveDateTime;
use std::fmt;

#[derive(Debug)]
pub struct TimestampedValue {
    pub value: f64,
    timestamp: NaiveDateTime,
}

impl TimestampedValue {
    pub fn new(value: f64, timestamp: NaiveDateTime) -> Self {
        TimestampedValue {
            value: value,
            timestamp: timestamp,
        }
    }
}

impl fmt::Display for TimestampedValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use match to check if each field is Some or None, and format accordingly
        write!(
            f,
            "value: {:?}, timestamp: {:?}",
            self.value, self.timestamp
        )
    }
}

pub struct QuantitativeIndicator {
    ema_38: f64,
    prev_ema_38: f64,
    ema_100: f64,
    prev_ema_100: f64,
    pub most_recent_value: Option<TimestampedValue>,
}

impl QuantitativeIndicator {
    pub fn new(most_recent_value: Option<TimestampedValue>) -> Self {
        QuantitativeIndicator {
            ema_38: 0.0,
            prev_ema_38: 0.0,
            ema_100: 0.0,
            prev_ema_100: 0.0,
            most_recent_value: most_recent_value,
        }
    }

    pub fn calculate_new_ema_38(&mut self) -> f64 {
        // Calculates the new ema_38, updates the prev and new value in the struct, and returns the calculated value
        let j = 38.0;
        let close = if let Some(ref last) = self.most_recent_value {
            last.value
        } else {
            0.0
        };

        let prev_ema = self.ema_38;

        let new_ema = close * (2.0 / (1.0 + j)) + prev_ema * (1.0 - (2.0 / (1.0 + j)));

        self.prev_ema_38 = prev_ema;
        self.ema_38 = new_ema;

        return new_ema;
    }

    pub fn calculate_new_ema_100(&mut self) -> f64 {
        // Calculates the new ema_100, updates the prev and new value in the struct, and returns the calculated value
        let j = 100.0;
        let close = if let Some(ref last) = self.most_recent_value {
            last.value
        } else {
            0.0
        };

        let prev_ema = self.ema_100;

        let new_ema = close * (2.0 / (1.0 + j)) + prev_ema * (1.0 - (2.0 / (1.0 + j)));

        self.prev_ema_100 = prev_ema;
        self.ema_100 = new_ema;

        return new_ema;
    }

    pub fn calculate_both_emas(&mut self) -> (f64, f64) {
        let ema_38 = self.calculate_new_ema_38();
        let ema_100 = self.calculate_new_ema_100();
        self.most_recent_value = None;

        return (ema_38, ema_100);
    }
}

impl fmt::Display for QuantitativeIndicator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use match to check if each field is Some or None, and format accordingly
        write!(
            f,
            "ema_38: {:?}, prev_ema_38: {:?}, ema_100: {:?}, prev_ema_100: {:?}, most_recent_value: {:?}",
            self.ema_38, self.prev_ema_38, self.ema_100, self.prev_ema_100, self.most_recent_value
        )
    }
}
