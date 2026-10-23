// Source font : Vazirmatn-Regular.ttf
// Pixel height: 8
// DPI         : 141
// Glyph range : U+0020..U+007E
// Glyphs      : 95
// Bitmap      : 320 bytes

use crate::screen::font::{BitmapFont, Glyph};

pub static VAZIR_FONT: BitmapFont = BitmapFont {
    glyphs: &[
        Glyph { width: 1, height: 1, advance: 2, x_offset: 0, y_offset: 0, bitmap_offset: 0 }, // U+0020 ' '
        Glyph { width: 1, height: 5, advance: 2, x_offset: 1, y_offset: -4, bitmap_offset: 1 }, // U+0021 '!'
        Glyph { width: 1, height: 2, advance: 3, x_offset: 1, y_offset: -5, bitmap_offset: 2 }, // U+0022 '"'
        Glyph { width: 5, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 3 }, // U+0023 '#'
        Glyph { width: 4, height: 8, advance: 5, x_offset: 0, y_offset: -6, bitmap_offset: 8 }, // U+0024 '$'
        Glyph { width: 6, height: 6, advance: 6, x_offset: 0, y_offset: -5, bitmap_offset: 12 }, // U+0025 '%'
        Glyph { width: 5, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 18 }, // U+0026 '&'
        Glyph { width: 1, height: 2, advance: 1, x_offset: 0, y_offset: -5, bitmap_offset: 23 }, // U+0027 "'"
        Glyph { width: 2, height: 8, advance: 3, x_offset: 1, y_offset: -5, bitmap_offset: 24 }, // U+0028 '('
        Glyph { width: 2, height: 8, advance: 3, x_offset: 0, y_offset: -5, bitmap_offset: 26 }, // U+0029 ')'
        Glyph { width: 3, height: 4, advance: 3, x_offset: 0, y_offset: -5, bitmap_offset: 28 }, // U+002A '*'
        Glyph { width: 3, height: 4, advance: 4, x_offset: 1, y_offset: -3, bitmap_offset: 31 }, // U+002B '+'
        Glyph { width: 1, height: 2, advance: 2, x_offset: 0, y_offset: 0, bitmap_offset: 34 }, // U+002C ','
        Glyph { width: 2, height: 1, advance: 4, x_offset: 1, y_offset: -2, bitmap_offset: 35 }, // U+002D '-'
        Glyph { width: 1, height: 1, advance: 2, x_offset: 1, y_offset: 0, bitmap_offset: 37 }, // U+002E '.'
        Glyph { width: 3, height: 6, advance: 3, x_offset: 0, y_offset: -5, bitmap_offset: 38 }, // U+002F '/'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 41 }, // U+0030 '0'
        Glyph { width: 2, height: 6, advance: 5, x_offset: 1, y_offset: -5, bitmap_offset: 45 }, // U+0031 '1'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 47 }, // U+0032 '2'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 51 }, // U+0033 '3'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 55 }, // U+0034 '4'
        Glyph { width: 3, height: 6, advance: 5, x_offset: 1, y_offset: -5, bitmap_offset: 59 }, // U+0035 '5'
        Glyph { width: 3, height: 6, advance: 5, x_offset: 1, y_offset: -5, bitmap_offset: 62 }, // U+0036 '6'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 65 }, // U+0037 '7'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 69 }, // U+0038 '8'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 73 }, // U+0039 '9'
        Glyph { width: 1, height: 3, advance: 2, x_offset: 1, y_offset: -2, bitmap_offset: 77 }, // U+003A ':'
        Glyph { width: 1, height: 5, advance: 2, x_offset: 0, y_offset: -3, bitmap_offset: 78 }, // U+003B ';'
        Glyph { width: 3, height: 3, advance: 4, x_offset: 0, y_offset: -3, bitmap_offset: 79 }, // U+003C '<'
        Glyph { width: 3, height: 2, advance: 4, x_offset: 1, y_offset: -2, bitmap_offset: 82 }, // U+003D '='
        Glyph { width: 3, height: 3, advance: 4, x_offset: 1, y_offset: -3, bitmap_offset: 85 }, // U+003E '>'
        Glyph { width: 3, height: 6, advance: 4, x_offset: 0, y_offset: -5, bitmap_offset: 88 }, // U+003F '?'
        Glyph { width: 7, height: 8, advance: 7, x_offset: 0, y_offset: -5, bitmap_offset: 91 }, // U+0040 '@'
        Glyph { width: 5, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 98 }, // U+0041 'A'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 1, y_offset: -5, bitmap_offset: 103 }, // U+0042 'B'
        Glyph { width: 5, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 107 }, // U+0043 'C'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 1, y_offset: -5, bitmap_offset: 112 }, // U+0044 'D'
        Glyph { width: 3, height: 6, advance: 5, x_offset: 1, y_offset: -5, bitmap_offset: 116 }, // U+0045 'E'
        Glyph { width: 3, height: 6, advance: 4, x_offset: 1, y_offset: -5, bitmap_offset: 119 }, // U+0046 'F'
        Glyph { width: 5, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 122 }, // U+0047 'G'
        Glyph { width: 4, height: 6, advance: 6, x_offset: 1, y_offset: -5, bitmap_offset: 127 }, // U+0048 'H'
        Glyph { width: 1, height: 6, advance: 2, x_offset: 1, y_offset: -5, bitmap_offset: 131 }, // U+0049 'I'
        Glyph { width: 4, height: 6, advance: 4, x_offset: 0, y_offset: -5, bitmap_offset: 132 }, // U+004A 'J'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 1, y_offset: -5, bitmap_offset: 136 }, // U+004B 'K'
        Glyph { width: 3, height: 6, advance: 4, x_offset: 1, y_offset: -5, bitmap_offset: 140 }, // U+004C 'L'
        Glyph { width: 5, height: 6, advance: 7, x_offset: 1, y_offset: -5, bitmap_offset: 143 }, // U+004D 'M'
        Glyph { width: 4, height: 6, advance: 6, x_offset: 1, y_offset: -5, bitmap_offset: 148 }, // U+004E 'N'
        Glyph { width: 5, height: 6, advance: 6, x_offset: 0, y_offset: -5, bitmap_offset: 152 }, // U+004F 'O'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 1, y_offset: -5, bitmap_offset: 157 }, // U+0050 'P'
        Glyph { width: 5, height: 7, advance: 6, x_offset: 0, y_offset: -5, bitmap_offset: 161 }, // U+0051 'Q'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 1, y_offset: -5, bitmap_offset: 166 }, // U+0052 'R'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 170 }, // U+0053 'S'
        Glyph { width: 5, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 174 }, // U+0054 'T'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 1, y_offset: -5, bitmap_offset: 179 }, // U+0055 'U'
        Glyph { width: 5, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 183 }, // U+0056 'V'
        Glyph { width: 7, height: 6, advance: 7, x_offset: 0, y_offset: -5, bitmap_offset: 188 }, // U+0057 'W'
        Glyph { width: 5, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 195 }, // U+0058 'X'
        Glyph { width: 5, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 200 }, // U+0059 'Y'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 205 }, // U+005A 'Z'
        Glyph { width: 1, height: 8, advance: 2, x_offset: 1, y_offset: -6, bitmap_offset: 209 }, // U+005B '['
        Glyph { width: 3, height: 6, advance: 3, x_offset: 0, y_offset: -5, bitmap_offset: 210 }, // U+005C '\\'
        Glyph { width: 1, height: 8, advance: 2, x_offset: 0, y_offset: -6, bitmap_offset: 213 }, // U+005D ']'
        Glyph { width: 3, height: 3, advance: 3, x_offset: 0, y_offset: -5, bitmap_offset: 214 }, // U+005E '^'
        Glyph { width: 4, height: 1, advance: 4, x_offset: 0, y_offset: 1, bitmap_offset: 217 }, // U+005F '_'
        Glyph { width: 2, height: 1, advance: 2, x_offset: 0, y_offset: -5, bitmap_offset: 221 }, // U+0060 '`'
        Glyph { width: 4, height: 4, advance: 4, x_offset: 0, y_offset: -3, bitmap_offset: 223 }, // U+0061 'a'
        Glyph { width: 3, height: 6, advance: 5, x_offset: 1, y_offset: -5, bitmap_offset: 227 }, // U+0062 'b'
        Glyph { width: 4, height: 4, advance: 4, x_offset: 0, y_offset: -3, bitmap_offset: 230 }, // U+0063 'c'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 0, y_offset: -5, bitmap_offset: 234 }, // U+0064 'd'
        Glyph { width: 4, height: 4, advance: 4, x_offset: 0, y_offset: -3, bitmap_offset: 238 }, // U+0065 'e'
        Glyph { width: 3, height: 6, advance: 3, x_offset: 0, y_offset: -5, bitmap_offset: 242 }, // U+0066 'f'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 0, y_offset: -3, bitmap_offset: 245 }, // U+0067 'g'
        Glyph { width: 3, height: 6, advance: 4, x_offset: 1, y_offset: -5, bitmap_offset: 249 }, // U+0068 'h'
        Glyph { width: 1, height: 6, advance: 2, x_offset: 0, y_offset: -5, bitmap_offset: 252 }, // U+0069 'i'
        Glyph { width: 1, height: 8, advance: 2, x_offset: 0, y_offset: -5, bitmap_offset: 253 }, // U+006A 'j'
        Glyph { width: 3, height: 6, advance: 4, x_offset: 1, y_offset: -5, bitmap_offset: 254 }, // U+006B 'k'
        Glyph { width: 1, height: 6, advance: 2, x_offset: 0, y_offset: -5, bitmap_offset: 257 }, // U+006C 'l'
        Glyph { width: 5, height: 4, advance: 7, x_offset: 1, y_offset: -3, bitmap_offset: 258 }, // U+006D 'm'
        Glyph { width: 3, height: 4, advance: 4, x_offset: 1, y_offset: -3, bitmap_offset: 263 }, // U+006E 'n'
        Glyph { width: 4, height: 4, advance: 5, x_offset: 0, y_offset: -3, bitmap_offset: 266 }, // U+006F 'o'
        Glyph { width: 3, height: 6, advance: 5, x_offset: 1, y_offset: -3, bitmap_offset: 270 }, // U+0070 'p'
        Glyph { width: 4, height: 6, advance: 5, x_offset: 0, y_offset: -3, bitmap_offset: 273 }, // U+0071 'q'
        Glyph { width: 2, height: 4, advance: 3, x_offset: 1, y_offset: -3, bitmap_offset: 277 }, // U+0072 'r'
        Glyph { width: 4, height: 4, advance: 4, x_offset: 0, y_offset: -3, bitmap_offset: 279 }, // U+0073 's'
        Glyph { width: 2, height: 5, advance: 3, x_offset: 0, y_offset: -4, bitmap_offset: 283 }, // U+0074 't'
        Glyph { width: 3, height: 4, advance: 4, x_offset: 1, y_offset: -3, bitmap_offset: 285 }, // U+0075 'u'
        Glyph { width: 4, height: 4, advance: 4, x_offset: 0, y_offset: -3, bitmap_offset: 288 }, // U+0076 'v'
        Glyph { width: 6, height: 4, advance: 6, x_offset: 0, y_offset: -3, bitmap_offset: 292 }, // U+0077 'w'
        Glyph { width: 4, height: 4, advance: 4, x_offset: 0, y_offset: -3, bitmap_offset: 298 }, // U+0078 'x'
        Glyph { width: 4, height: 6, advance: 4, x_offset: 0, y_offset: -3, bitmap_offset: 302 }, // U+0079 'y'
        Glyph { width: 4, height: 4, advance: 4, x_offset: 0, y_offset: -3, bitmap_offset: 306 }, // U+007A 'z'
        Glyph { width: 3, height: 7, advance: 3, x_offset: 0, y_offset: -5, bitmap_offset: 310 }, // U+007B '{'
        Glyph { width: 1, height: 7, advance: 2, x_offset: 0, y_offset: -5, bitmap_offset: 313 }, // U+007C '|'
        Glyph { width: 2, height: 7, advance: 3, x_offset: 0, y_offset: -5, bitmap_offset: 314 }, // U+007D '}'
        Glyph { width: 4, height: 1, advance: 5, x_offset: 1, y_offset: -2, bitmap_offset: 316 }, // U+007E '~'
    ],

    bitmap: &[
        0x00, 0x17, 0x03, 0x10, 0x3C, 0x17, 0x3E, 0x15, 0x20, 0x4E, 0xD3, 0x74, 0x02, 0x25, 0x1A, 0x34,
        0x2A, 0x30, 0x10, 0x2F, 0x2D, 0x32, 0x28, 0x03, 0x7F, 0x81, 0x81, 0x7E, 0x0A, 0x07, 0x0A, 0x02,
        0x0F, 0x02, 0x03, 0x01, 0x01, 0x01, 0x20, 0x1C, 0x03, 0x1E, 0x21, 0x21, 0x3E, 0x02, 0x3F, 0x22,
        0x31, 0x29, 0x26, 0x12, 0x21, 0x29, 0x3E, 0x10, 0x1C, 0x12, 0x3F, 0x37, 0x25, 0x39, 0x3E, 0x25,
        0x38, 0x01, 0x21, 0x1D, 0x03, 0x12, 0x2D, 0x29, 0x3E, 0x06, 0x29, 0x29, 0x1E, 0x05, 0x19, 0x02,
        0x02, 0x05, 0x03, 0x03, 0x03, 0x05, 0x05, 0x02, 0x02, 0x29, 0x07, 0x38, 0x46, 0xB9, 0xA5, 0xBD,
        0x22, 0x1C, 0x20, 0x1C, 0x13, 0x1C, 0x30, 0x3F, 0x29, 0x2F, 0x10, 0x1C, 0x22, 0x21, 0x21, 0x12,
        0x3F, 0x21, 0x21, 0x1E, 0x3F, 0x29, 0x29, 0x3F, 0x09, 0x09, 0x0C, 0x32, 0x21, 0x29, 0x3A, 0x3F,
        0x08, 0x08, 0x3F, 0x3F, 0x10, 0x20, 0x20, 0x1F, 0x3F, 0x0C, 0x12, 0x21, 0x3F, 0x20, 0x20, 0x3F,
        0x1C, 0x20, 0x1C, 0x3F, 0x3F, 0x04, 0x18, 0x3F, 0x0C, 0x32, 0x21, 0x21, 0x1E, 0x3F, 0x09, 0x09,
        0x06, 0x1C, 0x22, 0x21, 0x21, 0x5E, 0x3F, 0x09, 0x19, 0x26, 0x12, 0x25, 0x29, 0x3B, 0x01, 0x01,
        0x3F, 0x01, 0x01, 0x3F, 0x20, 0x20, 0x1F, 0x03, 0x0C, 0x30, 0x0C, 0x03, 0x03, 0x3C, 0x1C, 0x03,
        0x1C, 0x38, 0x07, 0x21, 0x12, 0x0C, 0x12, 0x21, 0x01, 0x06, 0x38, 0x06, 0x01, 0x21, 0x31, 0x2D,
        0x23, 0xFF, 0x03, 0x0C, 0x30, 0xFF, 0x04, 0x03, 0x04, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x04,
        0x0B, 0x0B, 0x0F, 0x3F, 0x24, 0x3C, 0x06, 0x09, 0x09, 0x09, 0x18, 0x24, 0x24, 0x3F, 0x06, 0x0B,
        0x0B, 0x0B, 0x04, 0x3F, 0x05, 0x16, 0x29, 0x29, 0x1F, 0x3F, 0x04, 0x3C, 0x3D, 0xFD, 0x3F, 0x14,
        0x22, 0x3F, 0x0F, 0x01, 0x0F, 0x01, 0x0F, 0x0F, 0x01, 0x0F, 0x06, 0x09, 0x09, 0x0F, 0x3F, 0x09,
        0x0F, 0x06, 0x09, 0x09, 0x3F, 0x0F, 0x01, 0x04, 0x0B, 0x0B, 0x0D, 0x02, 0x1F, 0x0F, 0x08, 0x0F,
        0x01, 0x0E, 0x06, 0x01, 0x01, 0x0E, 0x03, 0x03, 0x0E, 0x01, 0x09, 0x07, 0x07, 0x09, 0x21, 0x1E,
        0x06, 0x01, 0x09, 0x0D, 0x0B, 0x09, 0x08, 0x77, 0x41, 0x7F, 0x41, 0x7E, 0x01, 0x01, 0x01, 0x01,
    ],
    first: 0x20,
    last: 0x7E,
    y_advance: 13,
};
