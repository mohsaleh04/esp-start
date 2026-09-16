use crate::screen::font::{
    ASCII_FONT_CHAR_HEIGHT, ASCII_FONT_CHAR_SPACING, ASCII_FONT_CHAR_WIDTH, ASCII_FONT_LINE_SPACING,
};
use crate::screen::spi::ScreenSpi;
use crate::screen::{SCREEN_WIDTH, ScreenController};
use core::fmt::{self, Write};

pub(super) struct ScreenFmtWriter<'a, SPI: ScreenSpi> {
    screen: &'a mut ScreenController<SPI>,
    cursor: (i16, i16),
    line_start_x: i16,
    soft_wrap: bool,
    inverse: bool,
}

impl<'a, SPI: ScreenSpi> ScreenFmtWriter<'a, SPI> {
    pub(super) fn new(
        screen: &'a mut ScreenController<SPI>,
        anchor: (i16, i16),
        soft_wrap: bool,
        inverse: bool,
    ) -> Self {
        Self {
            screen,
            cursor: anchor,
            line_start_x: anchor.0,
            soft_wrap,
            inverse,
        }
    }
}
impl<'a, SPI: ScreenSpi> Write for ScreenFmtWriter<'a, SPI> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            if ch == '\n' {
                self.cursor.0 = self.line_start_x;
                self.cursor.1 += ASCII_FONT_CHAR_HEIGHT as i16 + ASCII_FONT_LINE_SPACING as i16;
                continue;
            }

            self.screen.draw_char(self.cursor, ch, self.inverse);
            self.cursor.0 += ASCII_FONT_CHAR_WIDTH as i16 + ASCII_FONT_CHAR_SPACING as i16;

            if self.soft_wrap && self.cursor.0 + ASCII_FONT_CHAR_WIDTH as i16 >= SCREEN_WIDTH as i16
            {
                self.cursor.0 = self.line_start_x;
                self.cursor.1 += ASCII_FONT_CHAR_HEIGHT as i16 + ASCII_FONT_LINE_SPACING as i16;
            }
        }

        Ok(())
    }
}
