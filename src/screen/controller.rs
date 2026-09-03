use crate::utils::delay;
use esp_hal::Blocking;
use esp_hal::gpio::{Output, OutputPin as GpioPin};
use esp_hal::peripherals::SPI2;
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;

pub struct ScreenController {
    spi: Spi<'static, Blocking>,
    dc: Output<'static>,
    cs: Output<'static>,
    rst: Output<'static>,
    backlight: Output<'static>,
}

impl ScreenController {
    pub fn new(
        spi_preph: SPI2<'static>,
        backlight: Output<'static>,
        dc: Output<'static>,
        cs: Output<'static>,
        rst: Output<'static>,
        sck: impl GpioPin + 'static,
        mosi: impl GpioPin + 'static,
    ) -> Self {
        Self {
            spi: Self::setup_spi(spi_preph, sck, mosi),
            backlight,
            dc,
            cs,
            rst,
        }
    }

    fn setup_spi(
        spi_preph: SPI2,
        sck: impl GpioPin + 'static,
        mosi: impl GpioPin + 'static,
    ) -> Spi<Blocking> {
        Spi::new(
            spi_preph,
            SpiConfig::default()
                .with_frequency(Rate::from_mhz(4))
                .with_mode(Mode::_0),
        )
        .expect("failed to setup screen spi")
        .with_sck(sck)
        .with_mosi(mosi)
    }

    pub fn toggle_backlight(&mut self) {
        self.backlight.toggle();
    }

    pub(super) fn send_command(&mut self, command: u8) {
        self.dc.set_low();
        self.cs.set_low();

        self.spi
            .write(&[command])
            .expect("failed to write command ScreenSPI");
        self.cs.set_high();
    }

    pub(super) fn send_data(&mut self, data: &[u8]) {
        self.dc.set_high();
        self.cs.set_low();

        self.spi
            .write(data)
            .expect("failed to write data ScreenSPI");
        self.cs.set_high();
    }

    pub(super) fn reset(&mut self) {
        self.rst.set_low();
        delay(10);

        self.rst.set_high();
        delay(10);
    }
}
