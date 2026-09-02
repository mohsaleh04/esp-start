use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus;
use esp_hal::Blocking;
use esp_hal::gpio::OutputPin as GpioPin;
use esp_hal::peripherals::SPI3;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::spi::Mode;
use esp_hal::time::Rate;
use crate::utils::delay;

#[repr(u8)]
pub enum ScreenCommand {
    BasicCommands = 0x20,
    ExtendedCommands = 0x21,
    SetContrast = 0x80,
    SetTempCoeff = 0x04,
    SetBias = 0x14,
    NormalDisplayMode = 0x0C,
    PixelTestMode = 0x09
}

pub fn setup_spi(spi: SPI3, screen_sck: impl GpioPin + 'static, screen_mosi: impl GpioPin + 'static) -> Spi<Blocking> {
    Spi::new(
        spi,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(1))
            .with_mode(Mode::_0),
    ).expect("failed to setup screen spi")
        .with_sck(screen_sck)
        .with_mosi(screen_mosi)
}

pub fn send_command(spi: &mut impl SpiBus, screen_dc: &mut impl OutputPin, screen_cs: &mut impl OutputPin, command: ScreenCommand) -> bool {
    send_command_raw(spi, screen_dc, screen_cs, command as u8)
}

pub fn send_command_raw(spi: &mut impl SpiBus, screen_dc: &mut impl OutputPin, screen_cs: &mut impl OutputPin, command: u8) -> bool {
    if screen_dc.set_low().is_err() { return false }
    if screen_cs.set_low().is_err() { return false }

    if spi.write(&[command]).is_err() { return false }
    if screen_cs.set_high().is_err() { return false }
    true
}

pub fn send_data(spi: &mut impl SpiBus, screen_dc: &mut impl OutputPin, screen_cs: &mut impl OutputPin, data: &[u8]) -> bool {
    if screen_dc.set_low().is_err() { return false }
    if screen_cs.set_low().is_err() { return false }

    if spi.write(data).is_err() { return false }
    if screen_dc.set_high().is_err() { return false }
    if screen_cs.set_high().is_err() { return false }
    true
}

pub fn send_reset(rst: &mut impl OutputPin) {
    rst.set_low().ok();
    delay(10);
    rst.set_high().ok();
    delay(10);
}