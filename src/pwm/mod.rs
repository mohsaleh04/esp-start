mod config;
mod controller;

pub use config::PwmConfig;
pub use controller::PwmController;

use esp_hal::ledc::timer::Timer;
use esp_hal::{
    gpio::interconnect::PeripheralOutput,
    ledc::{
        channel::{config::Config as ChannelConfig, ChannelIFace, Number as ChannelNumber}, timer::{config::Config as TimerConfig, Number as TimerNumber, TimerIFace}, LSGlobalClkSource,
        Ledc,
        LowSpeed,
    },
    peripherals::LEDC,
};
use static_cell::StaticCell;

static PWM_TIMER: StaticCell<Timer<'static, LowSpeed>> = StaticCell::new();

pub fn setup(
    ledc: LEDC<'static>,
    output_pin: impl PeripheralOutput<'static>,
    timer_number: TimerNumber,
    channel_number: ChannelNumber,
    config: PwmConfig,
) -> PwmController<'static> {
    let mut ledc = Ledc::new(ledc);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let timer = PWM_TIMER.init(ledc.timer::<LowSpeed>(timer_number));
    timer.configure(TimerConfig {
        duty: config.duty,
        clock_source: config.clock_source,
        frequency: config.frequency,
    }).unwrap();

    let mut channel = ledc.channel::<LowSpeed>(channel_number, output_pin);
    channel.configure(ChannelConfig {
        timer,
        duty_pct: config.duty_percent,
        drive_mode: config.drive_mode,
    }).unwrap();

    PwmController::new(channel)
}
