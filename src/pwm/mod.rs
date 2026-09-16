mod config;
mod controller;

pub use config::PwmChannelConfig;
pub use config::PwmTimerConfig;
pub use controller::PwmController;

use esp_hal::ledc::timer::Timer;
use esp_hal::{
    gpio::interconnect::PeripheralOutput,
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{ChannelIFace, Number as ChannelNumber, config::Config as ChannelConfig},
        timer::{TimerIFace, config::Config as TimerConfig},
    },
};

pub fn setup_timer(timer: &mut Timer<'static, LowSpeed>, config: PwmTimerConfig) {
    timer
        .configure(TimerConfig {
            duty: config.duty,
            clock_source: config.clock_source,
            frequency: config.frequency,
        })
        .unwrap();
}

pub fn setup_channel<'d>(
    ledc: &mut Ledc<'d>,
    output_pin: impl PeripheralOutput<'d>,
    timer: &'d Timer<'static, LowSpeed>,
    channel_number: ChannelNumber,
    config: PwmChannelConfig,
) -> PwmController<'d> {
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut channel = ledc.channel::<LowSpeed>(channel_number, output_pin);
    channel
        .configure(ChannelConfig {
            timer,
            duty_pct: config.duty_percent,
            drive_mode: config.drive_mode,
        })
        .unwrap();

    PwmController::new(channel)
}
