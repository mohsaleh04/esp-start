use core::cell::RefCell;
use esp_hal::Blocking;
use esp_hal::gpio::{InputPin as GpioInPin, OutputPin as GpioOutPin};
use esp_hal::peripherals::SPI2;
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;

pub fn setup(
    spi_preph: SPI2<'static>,
    spi_sclk: impl GpioOutPin + 'static,
    spi_mosi: impl GpioOutPin + 'static,
    spi_miso: impl GpioInPin + 'static,
) -> RefCell<Spi<'static, Blocking>> {
    let spi = Spi::new(
        spi_preph,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(1))
            .with_mode(Mode::_0),
    )
    .expect("failed to setup sd")
    .with_sck(spi_sclk)
    .with_mosi(spi_mosi)
    .with_miso(spi_miso);
    RefCell::new(spi)
}
