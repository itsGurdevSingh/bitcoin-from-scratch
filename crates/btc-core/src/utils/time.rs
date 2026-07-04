use std::time::{SystemTime, UNIX_EPOCH};

pub struct Time;

impl Time {
    pub fn unix_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_secs()
    }
}