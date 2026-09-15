use crate::screen::fmt::ScreenFmtWriter;
use crate::screen::font::{
    ASCII_FONT, ASCII_FONT_CHAR_HEIGHT, ASCII_FONT_CHAR_SPACING, ASCII_FONT_CHAR_WIDTH,
    ASCII_FONT_FIRST_INDEX, ASCII_FONT_LINE_SPACING,
};
use crate::screen::spi::ScreenSpi;
use crate::screen::{SCREEN_HEIGHT, SCREEN_WIDTH, ScreenController};
use core::fmt;
use core::fmt::Write;

/**
 *   Extended Screen Drawer Functions
 */
impl<SPI: ScreenSpi> ScreenController<SPI> {
    pub fn draw_fmt(
        &mut self,
        anchor: (i16, i16),
        args: fmt::Arguments<'_>,
        soft_wrap: bool,
        inverse: bool,
    ) {
        let mut writer = ScreenFmtWriter::new(self, anchor, soft_wrap, inverse);

        writer.write_fmt(args).unwrap();
    }

    pub fn draw_filled_text(&mut self, anchor: (i16, i16), text: &str) {
        self.draw_round_rect(
            anchor,
            text.chars().count() as u16
                * (ASCII_FONT_CHAR_WIDTH as u16 + ASCII_FONT_CHAR_SPACING as u16)
                + 9,
            15,
            3,
            true,
        );
        self.draw_text((anchor.0 + 5, anchor.1 + 4), text, false, true);
    }
}

/**
 *   Basic Screen Drawer Functions
 */
impl<SPI: ScreenSpi> ScreenController<SPI> {
    pub fn draw_char(&mut self, anchor: (i16, i16), c: char, inverse: bool) {
        if !c.is_ascii() {
            return;
        }

        let ascii = c as usize;
        if ascii < ASCII_FONT_FIRST_INDEX {
            return;
        }

        let Some(char_pixel_arr) = ASCII_FONT.get(ascii - ASCII_FONT_FIRST_INDEX) else {
            return;
        };

        for (col_i, column) in char_pixel_arr.iter().enumerate() {
            for row_i in 0..ASCII_FONT_CHAR_HEIGHT {
                if column & (1 << row_i) == 0 {
                    continue;
                }

                self.draw_px(anchor.0 + col_i as i16, anchor.1 + row_i as i16, inverse);
            }
        }
    }

    pub fn draw_text(&mut self, anchor: (i16, i16), text: &str, soft_wrap: bool, inverse: bool) {
        let char_width = ASCII_FONT_CHAR_WIDTH as i16 + ASCII_FONT_CHAR_SPACING as i16;
        let line_height = ASCII_FONT_CHAR_HEIGHT as i16 + ASCII_FONT_LINE_SPACING as i16;

        let mut x = anchor.0;
        let mut y = anchor.1;

        for ch in text.chars() {
            if y >= SCREEN_HEIGHT as i16 {
                break;
            }

            if (soft_wrap && x + ASCII_FONT_CHAR_WIDTH as i16 > SCREEN_WIDTH as i16) || ch == '\n' {
                x = anchor.0;
                y += line_height;

                if ch == '\n' {
                    continue;
                }
            }

            self.draw_char((x, y), ch, inverse);
            x += char_width;
        }
    }
}
