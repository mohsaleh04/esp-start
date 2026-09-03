use crate::screen::commands::{PositionCommand, ScreenCommand};
use crate::screen::{SCREEN_HEIGHT, SCREEN_WIDTH, ScreenController};

const SCREEN_BANKS: usize = SCREEN_HEIGHT / 8;
const SCREEN_BUFFER_LEN: usize = SCREEN_BANKS * SCREEN_WIDTH;

pub struct Screen {
    controller: ScreenController,
    framebuffer: [u8; SCREEN_BUFFER_LEN],
}

impl Screen {
    pub fn new(controller: ScreenController) -> Self {
        Self {
            controller,
            framebuffer: [0; SCREEN_BUFFER_LEN],
        }
    }

    pub fn init(&mut self, contrast: u8) {
        self.controller.reset();
        self.controller
            .send_command((ScreenCommand::FunctionSet as u8) | 0x01);
        self.controller
            .send_command(ScreenCommand::SetTempCoeff as u8);
        self.controller
            .send_command((ScreenCommand::SetBias as u8) | 0x04);
        self.controller
            .send_command((ScreenCommand::SetContrast as u8) | (contrast & 0x7F));

        self.controller
            .send_command((ScreenCommand::FunctionSet as u8) | 0x02);
            // set addressing orientation (0 => horizontal | 2 => vertical)
        self.controller
            .send_command(ScreenCommand::NormalDisplayMode as u8);
    }

    pub fn toggle_backlight(&mut self) {
        self.controller.toggle_backlight();
    }

    pub fn clear(&mut self) {
        self.set_cursor(0, 0);
        self.framebuffer = [0; SCREEN_BUFFER_LEN];
        self.controller.send_data(&self.framebuffer);
        self.set_cursor(0, 0);
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, on: bool) {
        if x >= SCREEN_WIDTH as u8 || y >= SCREEN_HEIGHT as u8 {
            return;
        }

        let bank = y / 8;
        let index = x as usize + bank as usize * SCREEN_WIDTH;
        if on {
            self.framebuffer[index] |= 1 << (y % 8);
        } else {
            self.framebuffer[index] &= !(1 << (y % 8));
        }

        self.set_cursor(x, bank);
        self.controller.send_data(&[self.framebuffer[index]]);
    }

    // ##############

    fn set_cursor(&mut self, x: u8, bank: u8) {
        if x >= SCREEN_WIDTH as u8 || bank >= SCREEN_BANKS as u8 {
            return;
        }
        self.controller
            .send_command((PositionCommand::SetX as u8) | x);
        self.controller
            .send_command((PositionCommand::SetBankY as u8) | bank);
    }
}
