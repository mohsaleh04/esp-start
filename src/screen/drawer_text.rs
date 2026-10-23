use crate::screen::fmt::ScreenFmtWriter;
use crate::screen::font::VAZIR_FONT;
use crate::screen::spi::ScreenSpi;
use crate::screen::{SCREEN_HEIGHT, SCREEN_WIDTH, ScreenController};
use core::fmt;
use core::fmt::Write;

/**
 *   Extended Screen Drawer Functions
 */ impl<SPI: ScreenSpi> ScreenController<SPI> {
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

    // pub fn draw_filled_text(&mut self, anchor: (i16, i16), text: &str) {
    //     self.draw_round_rect(
    //         anchor,
    //         text.chars().count() as u16
    //             * (ASCII_FONT_CHAR_WIDTH as u16 + ASCII_FONT_CHAR_SPACING as u16)
    //             + 9,
    //         15,
    //         3,
    //         true,
    //     );
    //     self.draw_text((anchor.0 + 5, anchor.1 + 4), text, false, true);
    // }
}

/**
 *   Basic Screen Drawer Functions
 */ impl<SPI: ScreenSpi> ScreenController<SPI> {
    pub fn draw_char(&mut self, anchor: (i16, i16), c: char, inverse: bool) {
        if !c.is_ascii() {
            return;
        }

        let codepoint = c as usize;
        if codepoint < VAZIR_FONT.first as usize || codepoint > VAZIR_FONT.last as usize
        {
            return;
        }

        let glyph_index = codepoint - VAZIR_FONT.first as usize;
        let Some(glyph) = VAZIR_FONT.glyphs.get(glyph_index) else {
            return;
        };

        let bitmap_start = glyph.bitmap_offset as usize;
        let bitmap_len = glyph.width as usize;

        let Some(bitmap) = VAZIR_FONT.bitmap.get(bitmap_start..bitmap_start + bitmap_len)
        else {
            return;
        };

        for (col, &column) in bitmap.iter().enumerate() {
            for row in 0..glyph.height {
                if column & (1 << row) == 0 {
                    continue;
                }

                let x = anchor.0 + glyph.x_offset as i16 + col as i16;
                let y = anchor.1 + glyph.y_offset as i16 + row as i16;
                self.draw_px(x, y, inverse);
            }
        }
    }

    pub fn draw_text(
        &mut self,
        anchor: (i16, i16),
        text: &str,
        soft_wrap: bool,
        inverse: bool,
    ) {
        let line_height = VAZIR_FONT.y_advance as i16;

        let mut x = anchor.0;
        let mut y = anchor.1;

        for ch in text.chars() {
            if y + line_height > SCREEN_HEIGHT as i16 {
                break;
            }

            if ch == '\n' {
                x = anchor.0;
                y += line_height;
                continue;
            }

            if !ch.is_ascii() {
                continue;
            }

            let codepoint = ch as usize;
            if codepoint < VAZIR_FONT.first as usize || codepoint > VAZIR_FONT.last as usize
            {
                continue;
            }

            let glyph_index = codepoint - VAZIR_FONT.first as usize;
            let Some(glyph) = VAZIR_FONT.glyphs.get(glyph_index) else {
                continue;
            };

            let advance = glyph.advance as i16;
            if soft_wrap && x + advance > SCREEN_WIDTH as i16 {
                x = anchor.0;
                y += line_height;

                if y + line_height > SCREEN_HEIGHT as i16 {
                    break;
                }
            }

            self.draw_char((x, y), ch, inverse);
            x += advance;
        }
    }
}
