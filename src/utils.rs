use esp_hal::time::{Duration, Instant};

pub fn delay(delay_ms: u64) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(delay_ms) {}
}
