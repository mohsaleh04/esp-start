#!.venv/bin/python
"""Generate compact 1-bit Rust fonts for the PCD8544 84x48 LCD.

Output format is unchanged: BitmapFont, Glyph, and column-major one-byte-per-column
bitmaps (bit 0 is the top pixel). Every glyph must be at most 8 pixels high.
The text renderer must treat its Y anchor as a baseline, with the traditional
Adafruit-GFX-compatible offset of 1 - bitmap_top.

Requires freetype-py. Optional Pillow enables a full-resolution LCD preview.
"""
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

try:
    import freetype
except ImportError as exc:
    raise SystemExit('Install freetype-py: python -m pip install freetype-py') from exc


@dataclass(frozen=True)
class GlyphData:
    width: int
    height: int
    advance: int
    x_offset: int
    y_offset: int
    bitmap_offset: int


@dataclass(frozen=True)
class FontData:
    glyphs: list[GlyphData]
    bitmap: list[int]
    first: int
    last: int
    y_advance: int
    pixel_height: int
    dpi: int


def integer(text: str) -> int:
    return int(text, 0)


def check_bits(name: str, value: int, lo: int, hi: int) -> None:
    if not lo <= value <= hi:
        raise ValueError(f'{name}={value} outside [{lo}, {hi}]')


def read_pixel(bitmap, x: int, y: int, mode: str) -> int:
    pitch = abs(bitmap.pitch)
    actual_y = y if bitmap.pitch >= 0 else bitmap.rows - 1 - y
    offset = actual_y * pitch
    if mode == 'mono':
        return 255 if (bitmap.buffer[offset + x // 8] & (0x80 >> (x % 8))) else 0
    return bitmap.buffer[offset + x]


def pack_columns(pixels: list[list[bool]]) -> list[int]:
    if not pixels:
        return []
    h, w = len(pixels), len(pixels[0])
    return [sum((1 << y) for y in range(h) if pixels[y][x]) for x in range(w)]


def render_glyph(face, cp: int, *, mode: str, threshold: int, tracking: int, offset: int):
    if face.get_char_index(cp) == 0:
        raise ValueError(f'Font lacks glyph U+{cp:04X} ({chr(cp)!r})')
    flags = freetype.FT_LOAD_DEFAULT
    if mode == 'mono':
        flags |= freetype.FT_LOAD_TARGET_MONO
    else:
        flags |= freetype.FT_LOAD_TARGET_NORMAL
    face.load_char(cp, flags)
    face.glyph.render(freetype.FT_RENDER_MODE_MONO if mode == 'mono' else freetype.FT_RENDER_MODE_NORMAL)
    slot = face.glyph
    bm = slot.bitmap
    width, height = int(bm.width), int(bm.rows)
    if height > 8:
        raise ValueError(f'U+{cp:04X} has {height}px height; format supports 8px max. Reduce --pixel-height.')
    advance = max(1, (int(slot.advance.x) + 32) // 64 + tracking)
    # Empty glyph: space has no bitmap. Preserve the meaningful advance.
    if width == 0 or height == 0:
        return GlyphData(0, 0, advance, 0, 0, offset), []
    pixels = [[read_pixel(bm, x, y, mode) >= threshold for x in range(width)] for y in range(height)]
    # Grayscale threshold can completely erase small dots; keep a single strongest
    # pixel rather than silently turning visible punctuation into whitespace.
    if not any(map(any, pixels)) and mode == 'gray':
        strongest = max(((read_pixel(bm, x, y, mode), x, y) for y in range(height) for x in range(width)), default=None)
        if strongest and strongest[0] > 0:
            pixels[strongest[2]][strongest[1]] = True
    # Crop transparent edges, updating the original FreeType metrics accordingly.
    xs = [x for y in range(height) for x in range(width) if pixels[y][x]]
    ys = [y for y in range(height) for x in range(width) if pixels[y][x]]
    if not xs:
        return GlyphData(0, 0, advance, 0, 0, offset), []
    left, right, top, bottom = min(xs), max(xs), min(ys), max(ys)
    cropped = [row[left:right + 1] for row in pixels[top:bottom + 1]]
    width, height = right - left + 1, bottom - top + 1
    x_offset = int(slot.bitmap_left) + left
    y_offset = 1 - int(slot.bitmap_top) + top
    for name, value, lo, hi in [('width', width, 0, 255), ('height', height, 0, 8), ('advance', advance, 0, 255),
                                ('x_offset', x_offset, -128, 127), ('y_offset', y_offset, -128, 127),
                                ('bitmap_offset', offset, 0, 65535)]:
        check_bits(f'U+{cp:04X}.{name}', value, lo, hi)
    return GlyphData(width, height, advance, x_offset, y_offset, offset), pack_columns(cropped)


def generate(path: Path, *, pixel_height: int, first: int, last: int,
             mode: str, threshold: int, tracking: int, line_height: int | None,
             allow_missing: bool, dpi: int) -> FontData:
    check_bits('first', first, 0, 255)
    check_bits('last', last, first, 255)
    check_bits('pixel_height', pixel_height, 1, 32)
    check_bits('threshold', threshold, 1, 255)
    check_bits('dpi', dpi, 1, 10000)
    if not path.is_file():
        raise ValueError(f'Font not found: {path}')
    face = freetype.Face(str(path))
    face.set_pixel_sizes(0, pixel_height)  # DPI intentionally does not apply to pixel sizing.
    glyphs, bitmap = [], []
    for cp in range(first, last + 1):
        if face.get_char_index(cp) == 0 and allow_missing:
            glyphs.append(GlyphData(0, 0, 0, 0, 0, len(bitmap)))
            continue
        glyph, packed = render_glyph(face, cp, mode=mode, threshold=threshold,
                                     tracking=tracking, offset=len(bitmap))
        glyphs.append(glyph)
        bitmap.extend(packed)
        if len(bitmap) > 65535:
            raise ValueError('Bitmap exceeds maximum u16 addressable size')
    # Derive minimum safe baseline distance from real ink extents.
    visible = [g for g in glyphs if g.width and g.height]
    min_top = min((g.y_offset for g in visible), default=0)
    max_bottom = max((g.y_offset + g.height - 1 for g in visible), default=0)
    min_line_height = max_bottom - min_top + 2  # 1px separation
    y_advance = line_height if line_height is not None else min_line_height
    if y_advance < min_line_height:
        print(f'warning: line-height {y_advance} can overlap glyphs (recommended >= {min_line_height})',
              file=sys.stderr)
    check_bits('y_advance', y_advance, 1, 255)
    return FontData(glyphs, bitmap, first, last, y_advance, pixel_height, dpi)


def rust_identifier(path: Path) -> str:
    result = re.sub(r'[^A-Za-z0-9_]', '_', path.stem).upper().strip('_') or 'BITMAP'
    return ('_' if result[0].isdigit() else '') + result + '_FONT'


def render_rust(font: FontData, source_font: Path, name: str, mode: str, threshold: int) -> str:
    if not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', name):
        raise ValueError(f'Invalid Rust identifier: {name!r}')
    lines = [
        '// @generated by tools/fontgen/fontgen.py; DO NOT EDIT.',
        f'// Source font : {source_font.name}',
        f'// Pixel height: {font.pixel_height}',
        f'// DPI metadata: {font.dpi} (not applied to pixel sizing)',
        f'// Raster mode : {mode}; threshold: {threshold}',
        f'// Glyph range : U+{font.first:04X}..U+{font.last:04X}',
        f'// Glyphs      : {len(font.glyphs)}',
        f'// Bitmap      : {len(font.bitmap)} bytes',
        '// Format: one byte per column, bit0 = top, glyph height <= 8.',
        'use crate::screen::font::{BitmapFont, Glyph};',
        f'pub static {name}: BitmapFont = BitmapFont {{', '    glyphs: &[',
    ]
    for i, g in enumerate(font.glyphs):
        cp = font.first + i
        vals = ', '.join(f'{key}: {getattr(g, key)}' for key in
                         ('width', 'height', 'advance', 'x_offset', 'y_offset', 'bitmap_offset'))
        lines.append(f'        Glyph {{ {vals} }}, // U+{cp:04X} {chr(cp)!r}')
    lines.extend(['    ],', '    bitmap: &['])
    for start in range(0, len(font.bitmap), 16):
        lines.append('        ' + ', '.join(f'0x{b:02X}' for b in font.bitmap[start:start + 16]) + ',')
    lines.extend(['    ],', f'    first: 0x{font.first:02X},', f'    last: 0x{font.last:02X},',
                  f'    y_advance: {font.y_advance},', '};', ''])
    return '\n'.join(lines)


def draw_text(font: FontData, canvas: list[list[int]], x: int, baseline: int, text: str):
    width, height = len(canvas[0]), len(canvas)
    start_x = x
    for ch in text:
        if ch == '\n':
            x, baseline = start_x, baseline + font.y_advance
            continue
        cp = ord(ch)
        if not font.first <= cp <= font.last:
            continue
        g = font.glyphs[cp - font.first]
        if x + g.advance > width and x > start_x:
            x, baseline = start_x, baseline + font.y_advance
        for col in range(g.width):
            bits = font.bitmap[g.bitmap_offset + col]
            for row in range(g.height):
                px, py = x + g.x_offset + col, baseline + g.y_offset + row
                if (bits >> row) & 1 and 0 <= px < width and 0 <= py < height:
                    canvas[py][px] = 1
        x += g.advance


def preview_png(font: FontData, path: Path, lines: list[str], scale: int):
    try:
        from PIL import Image
    except ImportError as exc:
        raise ValueError('For --preview-png install Pillow: python -m pip install Pillow') from exc
    canvas = [[0] * 84 for _ in range(48)]
    top = min((g.y_offset for g in font.glyphs if g.width and g.height), default=0)
    draw_text(font, canvas, 0, max(0, -top), '\n'.join(lines))
    image = Image.new('RGB', (84, 48), (190, 203, 173))
    pix = image.load()
    for y, row in enumerate(canvas):
        for x, set_bit in enumerate(row):
            if set_bit:
                pix[x, y] = (35, 52, 39)
    image = image.resize((84 * scale, 48 * scale), Image.Resampling.NEAREST)
    path.parent.mkdir(parents=True, exist_ok=True)
    image.save(path)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('font', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--name', default=None)
    parser.add_argument('--pixel-height', type=int, default=8)
    parser.add_argument('--first', type=integer, default=0x20)
    parser.add_argument('--last', type=integer, default=0x7e)
    parser.add_argument('--dpi', type=int, default=141, help='Metadata only; does not affect pixel rasterization')
    parser.add_argument('--mode', choices=('mono', 'gray'), default='gray')
    parser.add_argument('--threshold', type=int, default=110, help='Grayscale cutoff 1..255')
    parser.add_argument('--tracking', type=int, default=1, help='Extra horizontal pixels per character')
    parser.add_argument('--line-height', type=int, default=None)
    parser.add_argument('--allow-missing', action='store_true')
    parser.add_argument('--preview', action='store_true')
    parser.add_argument('--preview-png', type=Path, default=None)
    parser.add_argument('--preview-line', action='append', default=None,
                        help='Repeat to provide multiple preview lines')
    parser.add_argument('--preview-scale', type=int, default=5)
    args = parser.parse_args()
    try:
        font = generate(args.font, pixel_height=args.pixel_height, first=args.first, last=args.last,
                        mode=args.mode, threshold=args.threshold, tracking=args.tracking,
                        line_height=args.line_height, allow_missing=args.allow_missing, dpi=args.dpi)
        source = render_rust(font, args.font, args.name or rust_identifier(args.font), args.mode, args.threshold)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(source, encoding='utf-8')
        print(
            f'{len(font.glyphs)} glyphs | {len(font.bitmap)} bitmap bytes | line-height {font.y_advance}px | {args.output}')
        if args.preview:
            for char in 'AaWgyp0!? ':
                cp = ord(char)
                if not font.first <= cp <= font.last:
                    continue
                g = font.glyphs[cp - font.first]
                print(f'{char!r}: {g.width}x{g.height}, advance={g.advance}, offset=({g.x_offset},{g.y_offset})')
                for row in range(g.height):
                    print('   ' + ''.join(
                        '#' if font.bitmap[g.bitmap_offset + col] & (1 << row) else '.' for col in range(g.width)))
        if args.preview_png:
            preview_png(font, args.preview_png, args.preview_line or ['Hello ESP32!', '0123456789', 'gjpqy ABC!'],
                        args.preview_scale)
            print(f'Preview: {args.preview_png}')
    except (ValueError, RuntimeError, OSError) as exc:
        parser.error(str(exc))


if __name__ == '__main__':
    main()
