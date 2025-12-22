use std::time::Duration;

use chrono::Utc;
use tokio::time::sleep;

use crate::get_config;

#[derive(Debug, Clone, Copy)]
pub struct Snowflake {
    machine_id: u128,
    sequence: u128,
    last_timestamp: u128,
}

impl Snowflake {
    pub fn new() -> Self {
        Self {
            machine_id: get_config().machine_id as u128,
            sequence: 0,
            last_timestamp: Utc::now().timestamp_millis() as u128,
        }
    }

    pub async fn next_id(&mut self) -> u128 {
        let mut ts = Utc::now().timestamp_millis() as u128;

        if ts.eq(&self.last_timestamp) {
            self.sequence = self.sequence.wrapping_add(1);
            let seq_mask = (1_u128 << 32) - 1;
            if (self.sequence & seq_mask) == 0 {
                sleep(Duration::from_millis(1)).await;
                ts = Utc::now().timestamp_millis() as u128;
                self.sequence = 0;
            }
        } else {
            self.sequence = 0;
        }

        self.last_timestamp = ts;

        let ts_128 = ts & ((1_u128 << 64) - 1);
        let mid_128 = self.machine_id & ((1_u128 << 32) - 1);
        let sequence_128 = self.sequence & ((1_u128 << 32) - 1);

        (ts_128 << 64) | (mid_128 << 32) | sequence_128
    }
}
