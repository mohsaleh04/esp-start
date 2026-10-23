#!.venv/bin/python
"""
Generate a compact monochrome bitmap font for an embedded Rust renderer.

The rasterization pipeline intentionally follows the approach used by
Adafruit_GFX's fontconvert utility:

    TTF/OTF
       |
       v
    FreeType
       |
       +-- monochrome rendering
       +-- glyph metrics
       +-- bitmap
       |
       v
    Rust BitmapFont

The generated firmware data uses column-major bitmaps:

    bitmap[offset + x]
        bit 0 -> top pixel
        bit 1 -> next pixel
        ...
        bit 7 -> bottom pixel

Therefore the generated glyph height must be <= 8 pixels.

This is an OFFLINE build tool. The source font never reaches the ESP32.
"""

from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

try:
    import freetype
except ImportError as exc:
    raise SystemExit(
        "freetype-py is required.\n"
        "Install it with:\n"
        "    python -m pip install freetype-py"
    ) from exc


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

DEFAULT_FIRST = 0x20
DEFAULT_LAST = 0x7E

DEFAULT_DPI = 141

MAX_GLYPH_HEIGHT = 8
MAX_BITMAP_OFFSET = 0xFFFF

# FreeType flags.
LOAD_FLAGS = (
        freetype.FT_LOAD_TARGET_MONO
        | freetype.FT_LOAD_RENDER
)

RENDER_MODE = freetype.FT_RENDER_MODE_MONO


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------

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


# ---------------------------------------------------------------------------
# Validation helpers
# ---------------------------------------------------------------------------

def validate_u8(name: str, value: int) -> None:
    if not 0 <= value <= 0xFF:
        raise ValueError(f"{name}={value} does not fit in u8")


def validate_i8(name: str, value: int) -> None:
    if not -128 <= value <= 127:
        raise ValueError(f"{name}={value} does not fit in i8")


def validate_u16(name: str, value: int) -> None:
    if not 0 <= value <= 0xFFFF:
        raise ValueError(f"{name}={value} does not fit in u16")


def validate_range(first: int, last: int) -> None:
    if not 0 <= first <= 0xFF:
        raise ValueError(f"first codepoint must fit in u8: {first:#x}")

    if not 0 <= last <= 0xFF:
        raise ValueError(f"last codepoint must fit in u8: {last:#x}")

    if first > last:
        raise ValueError(
            f"first codepoint ({first:#x}) must not be greater "
            f"than last codepoint ({last:#x})"
        )


# ---------------------------------------------------------------------------
# FreeType helpers
# ---------------------------------------------------------------------------

def load_face(font_path: Path) -> freetype.Face:
    """Load a FreeType face from a font file."""
    try:
        return freetype.Face(str(font_path))
    except Exception as exc:
        raise RuntimeError(
            f"failed to load font: {font_path}"
        ) from exc


def configure_face(
        face: freetype.Face,
        *,
        pixel_height: int,
        dpi: int,
) -> None:
    """
    Configure the FreeType face.

    Pixel sizing is used instead of the point-size API because our target
    device is a pixel-addressed 84x48 monochrome LCD.

    DPI is still supplied so FreeType has deterministic horizontal/vertical
    metrics.
    """
    if pixel_height <= 0:
        raise ValueError("pixel height must be > 0")

    if dpi <= 0:
        raise ValueError("DPI must be > 0")

    face.set_pixel_sizes(0, pixel_height)

    # Keep the API explicit: the current pixel-size path does not directly
    # use DPI, but we store it in FontData for reproducibility.
    _ = dpi


def glyph_exists(face: freetype.Face, codepoint: int) -> bool:
    """Return whether the font contains a glyph for this codepoint."""
    return face.get_char_index(codepoint) != 0


# ---------------------------------------------------------------------------
# Bitmap conversion
# ---------------------------------------------------------------------------

def bitmap_pixel(bitmap: freetype.Bitmap, x: int, y: int) -> bool:
    """
    Read one pixel from a FreeType MONO bitmap.

    FreeType MONO bitmaps are row-major and pack 8 horizontal pixels
    into each byte, MSB first.
    """
    byte_index = x // 8
    bit_mask = 0x80 >> (x & 7)

    # pitch may be negative for some bitmap configurations.
    row = y

    if bitmap.pitch >= 0:
        offset = row * bitmap.pitch
    else:
        offset = (bitmap.rows - 1 - row) * (-bitmap.pitch)

    return bool(bitmap.buffer[offset + byte_index] & bit_mask)


def bitmap_to_columns(
        bitmap: freetype.Bitmap,
) -> list[int]:
    """
    Convert FreeType's row-major MONO bitmap into our column-major format.

    Each output byte represents one vertical column:

        bit 0 = top pixel
        bit 1 = next pixel
        ...
        bit 7 = bottom pixel
    """
    width = int(bitmap.width)
    height = int(bitmap.rows)

    if height > MAX_GLYPH_HEIGHT:
        raise ValueError(
            f"glyph bitmap height {height} exceeds "
            f"maximum supported height {MAX_GLYPH_HEIGHT}"
        )

    if width == 0 or height == 0:
        return []

    columns: list[int] = []

    for x in range(width):
        value = 0

        for y in range(height):
            if bitmap_pixel(bitmap, x, y):
                value |= 1 << y

        columns.append(value)

    return columns


# ---------------------------------------------------------------------------
# Glyph extraction
# ---------------------------------------------------------------------------

def extract_glyph(
        face: freetype.Face,
        codepoint: int,
        bitmap_offset: int,
) -> tuple[GlyphData, list[int]]:
    """
    Load, render and extract one glyph.

    Metrics intentionally follow Adafruit_GFX's fontconvert conventions:

        width
        height
        xAdvance
        xOffset
        yOffset

    In particular:

        xOffset = bitmap_left
        yOffset = 1 - bitmap_top

    This means the Rust renderer can use the same baseline semantics.
    """
    if not glyph_exists(face, codepoint):
        raise ValueError(
            f"font does not contain U+{codepoint:04X}"
        )

    try:
        face.load_char(codepoint, LOAD_FLAGS)

        # Explicitly request MONO rendering as Adafruit does.
        face.glyph.render(RENDER_MODE)

    except Exception as exc:
        raise RuntimeError(
            f"failed to rasterize U+{codepoint:04X}"
        ) from exc

    slot = face.glyph
    bitmap = slot.bitmap

    width = int(bitmap.width)
    height = int(bitmap.rows)

    # FreeType's advance is 26.6 fixed point.
    advance = int(slot.advance.x) >> 6

    # Position of bitmap relative to the baseline/cursor.
    x_offset = int(slot.bitmap_left)

    # Same convention used by Adafruit_GFX fontconvert.
    y_offset = 1 - int(slot.bitmap_top)

    validate_u8("glyph.width", width)
    validate_u8("glyph.height", height)
    validate_u8("glyph.advance", advance)
    validate_i8("glyph.x_offset", x_offset)
    validate_i8("glyph.y_offset", y_offset)
    validate_u16("glyph.bitmap_offset", bitmap_offset)

    columns = bitmap_to_columns(bitmap)

    # The bitmap width should agree with the generated column count.
    if len(columns) != width:
        raise RuntimeError(
            f"internal bitmap width mismatch for U+{codepoint:04X}: "
            f"FreeType={width}, columns={len(columns)}"
        )

    # Empty glyphs such as space legitimately have no pixels.
    # Their advance is still meaningful.
    if width > 0 and advance < width + max(0, x_offset):
        # Do not silently "fix" FreeType's metrics.
        # This is only a sanity warning-worthy condition.
        pass

    glyph = GlyphData(
        width=width,
        height=height,
        advance=advance,
        x_offset=x_offset,
        y_offset=y_offset,
        bitmap_offset=bitmap_offset,
    )

    return glyph, columns


# ---------------------------------------------------------------------------
# Font generation
# ---------------------------------------------------------------------------

def generate(
        font_path: Path,
        *,
        pixel_height: int,
        first: int = DEFAULT_FIRST,
        last: int = DEFAULT_LAST,
        dpi: int = DEFAULT_DPI,
        allow_missing: bool = False,
) -> FontData:
    """
    Generate the complete bitmap font.
    """
    validate_range(first, last)

    if not 1 <= pixel_height <= MAX_GLYPH_HEIGHT:
        raise ValueError(
            f"pixel height must be between 1 and "
            f"{MAX_GLYPH_HEIGHT}"
        )

    face = load_face(font_path)
    configure_face(
        face,
        pixel_height=pixel_height,
        dpi=dpi,
    )

    glyphs: list[GlyphData] = []
    bitmap: list[int] = []

    for codepoint in range(first, last + 1):
        if not glyph_exists(face, codepoint):
            if allow_missing:
                glyphs.append(
                    GlyphData(
                        width=0,
                        height=0,
                        advance=0,
                        x_offset=0,
                        y_offset=0,
                        bitmap_offset=len(bitmap),
                    )
                )
                continue

            raise ValueError(
                f"font is missing U+{codepoint:04X} "
                f"({chr(codepoint)!r})"
            )

        glyph, columns = extract_glyph(
            face,
            codepoint,
            bitmap_offset=len(bitmap),
        )

        bitmap.extend(columns)
        glyphs.append(glyph)

        if len(bitmap) > MAX_BITMAP_OFFSET:
            raise ValueError(
                f"bitmap is too large: {len(bitmap)} bytes; "
                f"bitmap_offset is u16"
            )

    # FreeType's recommended vertical distance between baselines.
    #
    # With set_pixel_sizes(), face.size.metrics.height is expressed
    # in 26.6 pixel units.
    y_advance = int(face.size.height) >> 6

    # Be conservative if the font reports zero.
    if y_advance <= 0:
        y_advance = pixel_height

    validate_u8("y_advance", y_advance)

    return FontData(
        glyphs=glyphs,
        bitmap=bitmap,
        first=first,
        last=last,
        y_advance=y_advance,
        pixel_height=pixel_height,
        dpi=dpi,
    )


# ---------------------------------------------------------------------------
# Rust output
# ---------------------------------------------------------------------------

def rust_ident_from_path(path: Path) -> str:
    """
    Generate a safe-ish Rust identifier from the font filename.

    Example:

        Vazirmatn-Regular.ttf
        -> VAZIRMATN_REGULAR_FONT
    """
    raw = path.stem.upper()

    chars = []
    for char in raw:
        if char.isalnum():
            chars.append(char)
        else:
            chars.append("_")

    name = "".join(chars).strip("_")

    if not name:
        name = "BITMAP"

    if name[0].isdigit():
        name = "_" + name

    return f"{name}_FONT"


def render_rust(
        font: FontData,
        *,
        source_font: Path,
        font_name: str,
        bytes_per_line: int = 16,
) -> str:
    """
    Render FontData as a Rust source file.
    """
    lines: list[str] = []

    lines.append("// @generated by tools/fontgen.py; DO NOT EDIT.")
    lines.append("//")
    lines.append(f"// Source font : {source_font.name}")
    lines.append(f"// Pixel height: {font.pixel_height}")
    lines.append(f"// DPI         : {font.dpi}")
    lines.append(
        f"// Glyph range : U+{font.first:04X}..U+{font.last:04X}"
    )
    lines.append(f"// Glyphs      : {len(font.glyphs)}")
    lines.append(f"// Bitmap      : {len(font.bitmap)} bytes")
    lines.append("")
    lines.append("use crate::screen::font::{BitmapFont, Glyph};")
    lines.append("")
    lines.append(f"pub(super) static {font_name}: BitmapFont = BitmapFont {{")
    lines.append("    glyphs: &[")

    for index, glyph in enumerate(font.glyphs):
        codepoint = font.first + index

        if 0x20 <= codepoint <= 0x7E:
            comment = f" // U+{codepoint:04X} {chr(codepoint)!r}"
        else:
            comment = f" // U+{codepoint:04X}"

        lines.append(
            "        Glyph { "
            f"width: {glyph.width}, "
            f"height: {glyph.height}, "
            f"advance: {glyph.advance}, "
            f"x_offset: {glyph.x_offset}, "
            f"y_offset: {glyph.y_offset}, "
            f"bitmap_offset: {glyph.bitmap_offset} "
            f"}},{comment}"
        )

    lines.append("    ],")
    lines.append("")
    lines.append("    bitmap: &[")

    for start in range(0, len(font.bitmap), bytes_per_line):
        chunk = font.bitmap[start:start + bytes_per_line]

        values = ", ".join(
            f"0x{value:02X}"
            for value in chunk
        )

        lines.append(f"        {values},")

    lines.append("    ],")
    lines.append(f"    first: 0x{font.first:02X},")
    lines.append(f"    last: 0x{font.last:02X},")
    lines.append(f"    y_advance: {font.y_advance},")
    lines.append("};")
    lines.append("")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Preview / diagnostics
# ---------------------------------------------------------------------------

def print_report(font: FontData) -> None:
    print()
    print("Font generation successful")
    print("--------------------------")
    print(f"Glyph range : U+{font.first:04X}..U+{font.last:04X}")
    print(f"Glyph count : {len(font.glyphs)}")
    print(f"Pixel height: {font.pixel_height}")
    print(f"Y advance   : {font.y_advance}")
    print(f"Bitmap size : {len(font.bitmap)} bytes")

    if font.bitmap:
        print(
            f"Flash data  : "
            f"{len(font.bitmap)} bitmap + "
            f"{len(font.glyphs) * 8} glyph metadata bytes"
        )

    print()


def print_glyph_preview(
        font: FontData,
        codepoints: Iterable[int],
) -> None:
    """
    Print small ASCII previews of selected glyphs.

    This is deliberately terminal-only and has no dependency on Pillow.
    """
    glyphs = font.glyphs

    print("Glyph preview")
    print("-------------")

    for codepoint in codepoints:
        if not font.first <= codepoint <= font.last:
            continue

        glyph = glyphs[codepoint - font.first]

        print(
            f"U+{codepoint:04X} "
            f"{chr(codepoint)!r} "
            f"size={glyph.width}x{glyph.height} "
            f"advance={glyph.advance} "
            f"offset=({glyph.x_offset},{glyph.y_offset})"
        )

        if glyph.width == 0 or glyph.height == 0:
            print("  <empty>")
            print()
            continue

        start = glyph.bitmap_offset
        end = start + glyph.width
        columns = font.bitmap[start:end]

        for y in range(glyph.height):
            row = []

            for column in columns:
                row.append(
                    "█" if (column & (1 << y)) else " "
                )

            print("  " + "".join(row))

        print()


def default_preview_codepoints(
        first: int,
        last: int,
) -> list[int]:
    wanted = "AaBb0123@#$%"

    result = []

    for char in wanted:
        codepoint = ord(char)

        if first <= codepoint <= last:
            result.append(codepoint)

    return result


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def parse_int(value: str) -> int:
    """
    Parse decimal or 0x-prefixed integer.
    """
    try:
        return int(value, 0)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(
            f"invalid integer: {value!r}"
        ) from exc


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument(
        "font",
        type=Path,
        help="Path to a .ttf/.otf font file",
    )

    parser.add_argument(
        "--pixel-height",
        type=int,
        default=8,
        metavar="N",
        help="Target pixel size passed to FreeType (default: 8)",
    )

    parser.add_argument(
        "--dpi",
        type=int,
        default=DEFAULT_DPI,
        help=f"Metric DPI (default: {DEFAULT_DPI})",
    )

    parser.add_argument(
        "--first",
        type=parse_int,
        default=DEFAULT_FIRST,
        metavar="CODEPOINT",
        help="First Unicode codepoint (default: 0x20)",
    )

    parser.add_argument(
        "--last",
        type=parse_int,
        default=DEFAULT_LAST,
        metavar="CODEPOINT",
        help="Last Unicode codepoint (default: 0x7E)",
    )

    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="Output Rust source file",
    )

    parser.add_argument(
        "--name",
        help=(
            "Rust static name "
            "(default: derived from font filename)"
        ),
    )

    parser.add_argument(
        "--allow-missing",
        action="store_true",
        help="Allow missing glyphs instead of failing",
    )

    parser.add_argument(
        "--preview",
        action="store_true",
        help="Print selected glyphs as terminal pixel art",
    )

    return parser.parse_args()


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> int:
    args = parse_args()

    if not args.font.is_file():
        print(
            f"error: font file not found: {args.font}",
            file=sys.stderr,
        )
        return 1

    try:
        font = generate(
            args.font,
            pixel_height=args.pixel_height,
            first=args.first,
            last=args.last,
            dpi=args.dpi,
            allow_missing=args.allow_missing,
        )

        font_name = args.name or rust_ident_from_path(args.font)

        rust = render_rust(
            font,
            source_font=args.font,
            font_name=font_name,
        )

        args.output.parent.mkdir(
            parents=True,
            exist_ok=True,
        )

        args.output.write_text(
            rust,
            encoding="utf-8",
            newline="\n",
        )

        print_report(font)

        if args.preview:
            print_glyph_preview(
                font,
                default_preview_codepoints(
                    args.first,
                    args.last,
                ),
            )

        print(f"Rust output: {args.output}")
        print(f"Font symbol: {font_name}")

        return 0

    except (ValueError, RuntimeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
