pub struct Glyph {
    pub width: u8,
    pub height: u8,
    pub advance: u8,
    pub x_offset: i8,
    pub y_offset: i8,
    pub bitmap_offset: u16,
}

pub struct BitmapFont {
    pub glyphs: &'static [Glyph],
    pub bitmap: &'static [u8],
    pub first: u8,
    pub last: u8,
    pub y_advance: u8,
}
