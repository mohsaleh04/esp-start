use esp_hal::gpio::DriveMode;
use esp_hal::ledc::timer::{config::Duty, LSClockSource};
use esp_hal::time::Rate;

pub struct PwmConfig {
    pub duty: Duty,
    pub clock_source: LSClockSource,
    pub frequency: Rate,
    pub duty_percent: u8,
    pub drive_mode: DriveMode,
}

impl PwmConfig {
    pub fn led_default() -> Self {
        Self {
            duty: Duty::Duty8Bit,
            clock_source: LSClockSource::APBClk,
            frequency: Rate::from_hz(50),
            duty_percent: 50,
            drive_mode: DriveMode::PushPull,
        }
    }
}
