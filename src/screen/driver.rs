use crate::utils::delay;
use esp_hal::gpio::Output;
use crate::screen::spi::ScreenSpi;

pub struct ScreenDriver<SPI: ScreenSpi> {
    spi: SPI,
    dc: Output<'static>,
    rst: Output<'static>,
    backlight: Output<'static>,
}

impl<SPI: ScreenSpi> ScreenDriver<SPI> {
    pub fn new(
        spi: SPI,
        dc: Output<'static>,
        rst: Output<'static>,
        backlight: Output<'static>,
    ) -> Self {
        Self {
            spi,
            dc,
            rst,
            backlight,
        }
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

        self.spi
            .write(&[command])
            .expect("failed to write command ScreenSPI");
    }

    pub(super) fn send_data(&mut self, data: &[u8]) {
        self.dc.set_high();

        self.spi
            .write(data)
            .expect("failed to write data ScreenSPI");
    }

    pub(super) fn reset(&mut self) {
        self.rst.set_low();
        delay(10);

        self.rst.set_high();
        delay(10);
    }
}
