use crate::screen::commands::{config, AddressingCommand, PositioningCommand, ScreenCommand};
use crate::screen::{SCREEN_HEIGHT, SCREEN_WIDTH, ScreenDriver, validate_bounds};
use crate::screen::spi::ScreenSpi;

const SCREEN_BANKS: usize = SCREEN_HEIGHT / 8;
const SCREEN_BUFFER_LEN: usize = SCREEN_BANKS * SCREEN_WIDTH;

struct ScreenCursor { x: u8, y: u8 }

pub struct ScreenController<SPI: ScreenSpi> {
    driver: ScreenDriver<SPI>,
    framebuffer: [u8; SCREEN_BUFFER_LEN],
    cursor: ScreenCursor,
    backlight_enabled: bool
}

impl<SPI: ScreenSpi> ScreenController<SPI> {
    pub fn init(contrast: u8, screen_driver: ScreenDriver<SPI>) -> Self {
        let mut controller = Self::new(screen_driver);

        controller.driver.reset();
        controller.driver
            .send_command(ScreenCommand::ExtendedFunctionSet as u8);
        controller.driver
            .send_command(ScreenCommand::SetTempCoeff as u8);
        controller.driver
            .send_command((ScreenCommand::SetBias as u8) | config::DEFAULT_BIAS);
        controller.driver
            .send_command((ScreenCommand::SetContrast as u8) | (contrast & config::MAX_CONTRAST));

        controller.driver
            .send_command(AddressingCommand::Horizontal as u8);
        controller.driver
            .send_command(ScreenCommand::NormalDisplayMode as u8);

        controller
    }

    pub fn clear(&mut self) {
        self.reset_cursor();
        self.framebuffer = [0; SCREEN_BUFFER_LEN];
        self.driver.send_data(&self.framebuffer);
        self.reset_cursor();
    }

    pub fn toggle_backlight(&mut self) {
        self.backlight_enabled = !self.backlight_enabled;
        self.driver.set_backlight(self.backlight_enabled);
    }
}

// -----------

impl<SPI: ScreenSpi> ScreenController<SPI> {
    fn new(driver: ScreenDriver<SPI>) -> Self {
        Self {
            driver,
            framebuffer: [0; SCREEN_BUFFER_LEN],
            cursor: ScreenCursor { x: 0, y: 0 },
            backlight_enabled: false
        }
    }

    pub(super) fn draw_px(&mut self, x: i16, y: i16, inverse: bool) {
        if x < 0 || y < 0
            || x >= SCREEN_WIDTH as i16 || y >= SCREEN_HEIGHT as i16 {
            return;
        }
        let cursor_ok = self.set_cursor(x as u8, y as u8);
        if cursor_ok {
            self.set_pixel(!inverse);
        }
    }

    pub(super) fn set_cursor(&mut self, x: u8, y: u8) -> bool {
        if !validate_bounds(x, y) { return false; }
        self.cursor.x = x;
        self.cursor.y = y;
        true
    }

    // ###############

    fn set_pixel(&mut self, on: bool) {
        let x = self.cursor.x;
        let y = self.cursor.y;
        let bank = y / 8;
        let index = x as usize + bank as usize * SCREEN_WIDTH;

        if on {
            self.framebuffer[index] |= 1 << (y % 8);
        } else {
            self.framebuffer[index] &= !(1 << (y % 8));
        }

        self.set_cursor_bank(x, bank);
        self.driver.send_data(&[self.framebuffer[index]]);
    }

    fn set_cursor_bank(&mut self, x: u8, bank: u8) {
        if x >= SCREEN_WIDTH as u8 || bank >= SCREEN_BANKS as u8 {
            return;
        }
        self.driver
            .send_command((PositioningCommand::SetX as u8) | x);
        self.driver
            .send_command((PositioningCommand::SetBank as u8) | bank);
    }

    fn reset_cursor(&mut self) {
        self.set_cursor(0, 0);
        self.set_cursor_bank(0, 0);
    }
}
