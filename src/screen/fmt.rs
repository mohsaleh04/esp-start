use crate::screen::font::VAZIR_FONT;
use crate::screen::spi::ScreenSpi;
use crate::screen::{SCREEN_HEIGHT, SCREEN_WIDTH, ScreenController};
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

    fn next_line(&mut self) {
        self.cursor.0 = self.line_start_x;
        self.cursor.1 += VAZIR_FONT.y_advance as i16;
    }
}

impl<SPI: ScreenSpi> Write for ScreenFmtWriter<'_, SPI> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for ch in text.chars() {
            if ch == '\n' {
                self.next_line();
                continue;
            }

            let advance = ScreenController::<SPI>::char_advance(ch);

            if advance == 0 {
                continue;
            }

            if self.soft_wrap
                && self.cursor.0 + advance > SCREEN_WIDTH as i16
                && self.cursor.0 > self.line_start_x
            {
                self.next_line();
            }

            if self.cursor.1 >= SCREEN_HEIGHT as i16 {
                break;
            }

            self.screen.draw_char(
                self.cursor,
                ch,
                self.inverse,
            );

            self.cursor.0 += advance;
        }

        Ok(())
    }
}
