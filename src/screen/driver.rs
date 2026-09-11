use crate::io::ScreenPins;
use crate::screen::spi::ScreenSpi;
use crate::utils::delay;
use esp_hal::gpio::Output;

pub struct ScreenDriver<SPI: ScreenSpi> {
    spi: SPI,
    dc: Output<'static>,
    rst: Output<'static>,
    backlight: Output<'static>,
}

impl<SPI: ScreenSpi> ScreenDriver<SPI> {
    pub fn new(spi: SPI, pins: ScreenPins) -> Self {
        Self {
            spi,
            dc: pins.dc,
            rst: pins.rst,
            backlight: pins.backlight,
        }
    }

    pub(super) fn set_backlight(&mut self, on: bool) {
        if on {
            self.backlight.set_high();
        } else {
            self.backlight.set_low();
        }
    }

    pub(super) fn send_command(&mut self, command: u8) -> Result<(), SPI::Error> {
        self.dc.set_low();
        self.spi.write(&[command])
    }

    pub(super) fn send_data(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        self.dc.set_high();
        self.spi.write(data)
    }

    pub(super) fn reset(&mut self) {
        self.rst.set_low();
        delay(10);
        self.rst.set_high();
        delay(10);
    }
}
