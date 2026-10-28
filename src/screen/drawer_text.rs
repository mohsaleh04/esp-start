use crate::screen::fmt::ScreenFmtWriter;
use crate::screen::font::{ASCII_FONT, ASCII_FONT_FIRST_INDEX, VAZIR_FONT};
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
 */
impl<SPI: ScreenSpi> ScreenController<SPI> {
    pub fn draw_char(
        &mut self,
        anchor: (i16, i16),
        c: char,
        inverse: bool,
    ) {
        if !c.is_ascii() {
            return;
        }

        let index = c as usize;

        if index < ASCII_FONT_FIRST_INDEX {
            return;
        }

        let Some(columns) =
            ASCII_FONT.get(index - ASCII_FONT_FIRST_INDEX)
        else {
            return;
        };

        for (x, &bits) in columns.iter().enumerate() {
            for y in 0..8 {
                if bits & (1u8 << y) != 0 {
                    self.draw_px(
                        anchor.0 + x as i16,
                        anchor.1 + y as i16,
                        inverse,
                    );
                }
            }
        }
    }
    // pub fn draw_char(
    //     &mut self,
    //     anchor: (i16, i16),
    //     c: char,
    //     inverse: bool,
    // ) {
    //     let codepoint = c as u32;
    //
    //     if codepoint < VAZIR_FONT.first as u32
    //         || codepoint > VAZIR_FONT.last as u32
    //     {
    //         return;
    //     }
    //
    //     let index = (codepoint - VAZIR_FONT.first as u32) as usize;
    //
    //     let Some(glyph) = VAZIR_FONT.glyphs.get(index) else {
    //         return;
    //     };
    //
    //     let start = glyph.bitmap_offset as usize;
    //     let end = start + glyph.width as usize;
    //
    //     let Some(columns) = VAZIR_FONT.bitmap.get(start..end) else {
    //         return;
    //     };
    //
    //     for (col, &bits) in columns.iter().enumerate() {
    //         for row in 0..glyph.height {
    //             if bits & (1u8 << row) == 0 {
    //                 continue;
    //             }
    //
    //             let x = anchor.0
    //                 + glyph.x_offset as i16
    //                 + col as i16;
    //
    //             let y = anchor.1
    //                 + glyph.y_offset as i16
    //                 + row as i16;
    //
    //             self.draw_px(x, y, inverse);
    //         }
    //     }
    // }

    pub fn draw_text(
        &mut self,
        anchor: (i16, i16),
        text: &str,
        soft_wrap: bool,
        inverse: bool,
    ) {
        self.draw_fmt(
            anchor,
            format_args!("{}", text),
            soft_wrap,
            inverse,
        );
    }

    pub(super) fn char_advance(c: char) -> i16 {
        let codepoint = c as u32;
        if codepoint < VAZIR_FONT.first as u32 || codepoint > VAZIR_FONT.last as u32 {
            return 0;
        }

        let index = (codepoint - VAZIR_FONT.first as u32) as usize;
        VAZIR_FONT
            .glyphs
            .get(index)
            .map_or(0, |glyph| glyph.advance as i16)
    }
}
