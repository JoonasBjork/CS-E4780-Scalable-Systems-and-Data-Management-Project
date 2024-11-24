use chrono::{Local, NaiveDateTime};
use std::fmt;

// #[derive(Debug, Clone)]
// pub struct TimestampedValue {
//     pub value: f64,
//     pub timestamp: NaiveDateTime,
// }

// impl TimestampedValue {
//     pub fn new(value: f64, timestamp: NaiveDateTime) -> Self {
//         TimestampedValue {
//             value: value,
//             timestamp: timestamp,
//         }
//     }
// }

// impl fmt::Display for TimestampedValue {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // Use match to check if each field is Some or None, and format accordingly
//         write!(
//             f,
//             "value: {:?}, timestamp: {:?}",
//             self.value, self.timestamp
//         )
//     }
// }

#[derive(Clone, Debug)]
pub struct QuantitativeIndicator {
    // Stock information
    pub ema_38: f64,
    pub prev_ema_38: f64,
    pub ema_100: f64,
    pub prev_ema_100: f64,
    pub bullish: bool,
    pub bearish: bool,
    // Analytics
    pub count_in_this_window: i32,
    pub total_latency_in_this_window_ms: i32,
    pub previous_count: i32,
    pub previous_average_latency_ms: Option<i32>,
    // Possibly interesting information about the last value
    pub most_recent_value: Option<f64>,
    pub most_recent_value_timestamp: Option<NaiveDateTime>,
}

impl QuantitativeIndicator {
    pub fn new(most_recent_value: f64, most_recent_value_timestamp: NaiveDateTime) -> Self {
        let latency: i32 = Local::now()
            .naive_local()
            .signed_duration_since(most_recent_value_timestamp)
            .num_milliseconds()
            .try_into()
            .unwrap_or_else(|_| {
                eprintln!("ERROR: Duration exceeds i32 range; setting to maximum/minimum value");
                i32::MAX
            });
        QuantitativeIndicator {
            ema_38: 0.0,
            prev_ema_38: 0.0,
            ema_100: 0.0,
            prev_ema_100: 0.0,
            bullish: false,
            bearish: false,
            count_in_this_window: 1,
            total_latency_in_this_window_ms: latency,
            previous_count: 0,
            previous_average_latency_ms: None,
            most_recent_value: Some(most_recent_value),
            most_recent_value_timestamp: Some(most_recent_value_timestamp),
        }
    }

    pub fn update_most_recent_value(
        &mut self,
        most_recent_value: f64,
        most_recent_value_timestamp: NaiveDateTime,
    ) -> () {
        self.most_recent_value = Some(most_recent_value);
        self.most_recent_value_timestamp = Some(most_recent_value_timestamp)
    }

    pub fn calculate_new_ema_38(&mut self) -> f64 {
        // Calculates the new ema_38, updates the prev and new value in the struct, and returns the calculated value
        let j = 38.0;
        let close = if let Some(value) = self.most_recent_value {
            value
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
        let close = if let Some(value) = self.most_recent_value {
            value
        } else {
            0.0
        };

        let prev_ema = self.ema_100;

        let new_ema = close * (2.0 / (1.0 + j)) + prev_ema * (1.0 - (2.0 / (1.0 + j)));

        self.prev_ema_100 = prev_ema;
        self.ema_100 = new_ema;

        return new_ema;
    }

    pub fn calculate_both_emas(&mut self) -> () {
        // println!(
        //     "Old emas: {}, {}, {}, {}",
        //     self.ema_38, self.ema_100, self.prev_ema_38, self.prev_ema_100
        // );
        self.calculate_new_ema_38();
        self.calculate_new_ema_100();
        // println!(
        //     "New emas: {}, {}, {}, {}",
        //     self.ema_38, self.ema_100, self.prev_ema_38, self.prev_ema_100
        // );

        // If the values have not been initialized yet (no previous stocks seen), don't set the bullish/bearish
        if self.prev_ema_38 == 0.0 || self.prev_ema_100 == 0.0 {
            self.bullish = false;
            self.bearish = false;
            // println!("OLD EMAS ARE ZEROS");
        } else {
            self.bullish = self.ema_38 > self.ema_100 && self.prev_ema_38 <= self.prev_ema_100;
            self.bearish = self.ema_38 < self.ema_100 && self.prev_ema_38 >= self.prev_ema_100;
            // println!(
            //     "FOUND NEW BULLISH/BEARISH: {}, {}",
            //     self.bullish, self.bearish
            // );
        }
    }

    pub fn clear_most_recent_value(&mut self) -> () {
        self.most_recent_value = None;
        self.most_recent_value_timestamp = None;
    }

    pub fn calculate_average_latency(&mut self) -> () {
        if self.count_in_this_window > 0 {
            self.previous_average_latency_ms =
                Some(self.total_latency_in_this_window_ms / self.count_in_this_window);
        } else {
            self.previous_average_latency_ms = None;
        }

        self.previous_count = self.count_in_this_window;
        self.total_latency_in_this_window_ms = 0;
        self.count_in_this_window = 0;
    }
}

impl fmt::Display for QuantitativeIndicator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use match to check if each field is Some or None, and format accordingly
        write!(
            f,
            "ema_38: {:?}, prev_ema_38: {:?}, ema_100: {:?}, prev_ema_100: {:?}, most_recent_value: {:?}, most_recent_value_timestamp {:?}",
            self.ema_38, self.prev_ema_38, self.ema_100, self.prev_ema_100, self.most_recent_value, self.most_recent_value_timestamp
        )
    }
}
