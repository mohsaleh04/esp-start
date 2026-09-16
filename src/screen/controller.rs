use crate::screen::commands::{AddressingCommand, PositioningCommand, ScreenCommand, config};
use crate::screen::framebuffer::Framebuffer;
use crate::screen::spi::ScreenSpi;
use crate::screen::{SCREEN_HEIGHT, SCREEN_WIDTH, ScreenDriver};

pub struct ScreenController<SPI: ScreenSpi> {
    driver: ScreenDriver<SPI>,
    framebuffer: Framebuffer,
    backlight_enabled: bool,
}

impl<SPI: ScreenSpi> ScreenController<SPI> {
    pub fn init(contrast: u8, screen_driver: ScreenDriver<SPI>) -> Result<Self, SPI::Error> {
        let mut controller = Self::new(screen_driver);
        controller.driver.reset();
        controller
            .driver
            .send_command(ScreenCommand::ExtendedFunctionSet as u8)?;
        controller
            .driver
            .send_command(ScreenCommand::SetTempCoeff as u8)?;
        controller
            .driver
            .send_command((ScreenCommand::SetBias as u8) | config::DEFAULT_BIAS)?;
        controller
            .driver
            .send_command((ScreenCommand::SetContrast as u8) | (contrast & config::MAX_CONTRAST))?;
        controller
            .driver
            .send_command(AddressingCommand::Horizontal as u8)?;
        controller
            .driver
            .send_command(ScreenCommand::NormalDisplayMode as u8)?;

        Ok(controller)
    }

    pub fn clear(&mut self) {
        self.framebuffer.clear();
    }

    /// Sends the complete in-memory framebuffer to the LCD.
    pub fn flush(&mut self) -> Result<(), SPI::Error> {
        self.set_display_address(0, 0)?;
        self.driver.send_data(self.framebuffer.as_bytes())
    }

    pub fn toggle_backlight(&mut self) {
        self.backlight_enabled = !self.backlight_enabled;
        self.driver.set_backlight(self.backlight_enabled);
    }

    fn new(driver: ScreenDriver<SPI>) -> Self {
        Self {
            driver,
            framebuffer: Framebuffer::new(),
            backlight_enabled: false,
        }
    }

    pub(super) fn draw_px(&mut self, x: i16, y: i16, inverse: bool) -> bool {
        if x < 0 || y < 0 || x >= SCREEN_WIDTH as i16 || y >= SCREEN_HEIGHT as i16 {
            return false;
        }

        self.framebuffer.set_pixel(x as usize, y as usize, !inverse);
        true
    }

    fn set_display_address(&mut self, x: u8, bank: u8) -> Result<(), SPI::Error> {
        self.driver
            .send_command((PositioningCommand::SetX as u8) | x)?;
        self.driver
            .send_command((PositioningCommand::SetBank as u8) | bank)
    }
}
