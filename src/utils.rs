use esp_hal::time::{Duration, Instant};

pub fn delay(delay_ms: u64) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(delay_ms) {}
}

pub fn generate_seed() -> u64 {
    let rng = esp_hal::rng::Rng::new();
    ((rng.random() as u64) << 32) | rng.random() as u64
}
