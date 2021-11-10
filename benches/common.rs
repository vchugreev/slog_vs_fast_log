use std::time::Duration;

use log::info;

pub const SAMPLE_SIZE: usize = 100;
pub const MEASUREMENT_TIME: Duration = Duration::from_secs(10);
pub const CHANNEL_CAPACITY: usize = 1000;

#[inline]
pub fn logging() {
    info!("===");
}
