//! Selected-text ink chosen from the actual selection background,
//!  not from the color scheme.
//!
//! The toolkit's fluent palette keeps one selection background (`#0078D4`) in both schemes
//! but flips its selection ink to black in the dark scheme.
//! Black on that blue has a WCAG 2 ratio of 4.64 and white 4.53,
//!  so the ratio alone does not separate them;
//! the light ink is the readable one on a saturated mid-tone,
//!  and it is what the light scheme already shows.

/// What:
///  `const` names a compile-time value;
///  `[u8; 4]` is a fixed array of four bytes
/// (red,
///  green,
///  blue,
///  alpha).
///  Siblings:
///  `Vec<u8>` (growable) and `&[u8]` (borrowed,
///  any length).
/// Why:
///  A color always has exactly four channels,
///  so the fixed array needs no allocation
/// and cannot have the wrong length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WHITE: readonly [number, number, number, number] = [255, 255, 255, 255];
/// ```
const WHITE: [u8; 4] = [255, 255, 255, 255];

/// Opaque black,
///  used only on light selection backgrounds.
const BLACK: [u8; 4] = [0, 0, 0, 255];

/// What:
///  `f32` is a 32-bit float (sibling `f64` has twice the precision).
/// Why:
///  White ink is kept while it reaches this WCAG 2 ratio,
///  the minimum for large text and graphics;
/// above that background lightness black takes over and then has at least 7:1.
/// `f32` matches the other color arithmetic in this crate and is precise enough for a ratio.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WHITE_MINIMUM = 3;
/// ```
pub const WHITE_MINIMUM: f32 = 3.0;

/// What:
///  Convert one stored sRGB byte into linear light between 0 and 1;
///  `u8` is one byte
/// (siblings `u16`,
///  `u32`),
///  `f32::from` widens it without loss.
/// Why:
///  Luminance is a weighted sum of linear light,
///  not of stored bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linear(channel: number): number {
///   const value = channel / 255;
///   return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
/// }
/// ```
fn linear(channel: u8) -> f32 {
    let value = f32::from(channel) / 255.0;
    if value <= 0.04045 {
        return value / 12.92;
    }
    return ((value + 0.055) / 1.055).powf(2.4);
}

/// WCAG 2 relative luminance of an opaque color,
///  from 0 (black) to 1 (white).
///  Alpha is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function luminance([red, green, blue]: Color): number {
///   return 0.2126 * linear(red) + 0.7152 * linear(green) + 0.0722 * linear(blue);
/// }
/// ```
pub fn luminance(color: [u8; 4]) -> f32 {
    return 0.2126 * linear(color[0]) + 0.7152 * linear(color[1]) + 0.0722 * linear(color[2]);
}

/// WCAG 2 contrast ratio between two opaque colors,
///  from 1 (equal) to 21 (black on white).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function contrast(first: Color, second: Color): number {
///   const [low, high] = [luminance(first), luminance(second)].sort((a, b) => a - b);
///   return (high + 0.05) / (low + 0.05);
/// }
/// ```
pub fn contrast(first: [u8; 4], second: [u8; 4]) -> f32 {
    let one = luminance(first);
    let other = luminance(second);
    return (one.max(other) + 0.05) / (one.min(other) + 0.05);
}

/// Choose the ink for selected source text on `background`.
///
/// `palette` is the toolkit's own selection ink.
///  It is used only when the background is translucent,
/// because its lightness then depends on what is behind it,
///  which this function does not know.
/// For an opaque background the result is white while white reaches [`WHITE_MINIMUM`],
///  otherwise black.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function legibleInk(background: Color, palette: Color): Color {
///   if (background[3] !== 255) return palette;
///   return contrast(WHITE, background) >= WHITE_MINIMUM ? WHITE : BLACK;
/// }
/// ```
pub fn legible_ink(background: [u8; 4], palette: [u8; 4]) -> [u8; 4] {
    if background[3] != 255 {
        return palette;
    }
    if contrast(WHITE, background) >= WHITE_MINIMUM {
        return WHITE;
    }
    return BLACK;
}
