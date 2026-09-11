use embedded_hal::spi::SpiBus;
use crate::utils::delay;
use esp_hal::Blocking;
use esp_hal::gpio::{Output, OutputPin as GpioPin};
use esp_hal::peripherals::SPI2;
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;

pub struct ScreenDriver {
    spi: Spi<'static, Blocking>,
    dc: Output<'static>,
    cs: Output<'static>,
    rst: Output<'static>,
    backlight: Output<'static>,
}

impl ScreenDriver {
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

    pub(super) fn set_backlight(&mut self, on: bool) {
        if on {
            self.backlight.set_high();
        } else {
            self.backlight.set_low();
        }
    }

    pub(super) fn send_command(&mut self, command: u8) {
        self.dc.set_low();
        self.cs.set_low();

        self.spi
            .write(&[command])
            .expect("failed to write command ScreenSPI");
        self.spi.flush().expect("failed to flush data into ScreenSPI");
        self.cs.set_high();
    }

    pub(super) fn send_data(&mut self, data: &[u8]) {
        self.dc.set_high();
        self.cs.set_low();

        self.spi
            .write(data)
            .expect("failed to write data ScreenSPI");
        self.spi.flush().expect("failed to flush data into ScreenSPI");
        self.cs.set_high();
    }

    pub(super) fn reset(&mut self) {
        self.rst.set_low();
        delay(10);

        self.rst.set_high();
        delay(10);
    }
}
