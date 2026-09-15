use embedded_sdmmc::{TimeSource, Timestamp};

pub struct FixedTimeSource;

impl TimeSource for FixedTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 56, // 2026
            zero_indexed_month: 8,
            zero_indexed_day: 10,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}
