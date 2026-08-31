use esp_hal::gpio::DriveMode;
use esp_hal::ledc::timer::{LSClockSource, config::Duty};
use esp_hal::time::Rate;

pub struct PwmTimerConfig {
    pub duty: Duty,
    pub clock_source: LSClockSource,
    pub frequency: Rate,
}

pub struct PwmChannelConfig {
    pub duty_percent: u8,
    pub drive_mode: DriveMode,
}

impl PwmTimerConfig {
    pub fn default(frequency_hz: u32) -> Self {
        Self {
            duty: Duty::Duty8Bit,
            clock_source: LSClockSource::APBClk,
            frequency: Rate::from_hz(frequency_hz),
        }
    }
}

impl PwmChannelConfig {
    pub fn default() -> Self {
        Self {
            duty_percent: 0,
            drive_mode: DriveMode::PushPull,
        }
    }
}
