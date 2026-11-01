//! OKLCH color math: sRGB conversion, gamut mapping, WCAG contrast and mixing.
//!
//! What: every color the theme derives (D108) is moved in OKLCH (lightness, chroma, hue) and converted back to 8-bit
//! sRGB here, so no role is ever computed by mixing sRGB channels, HSV or HSL.
//! Why: the theme module (`theme_roles.rs`) needs one pure, unit-testable home for the color space, so the window
//! only ever receives finished sRGB values.
//! Gamut handling: a requested lightness, chroma and hue that sRGB cannot show keeps its lightness and hue, and the
//! chroma is reduced by binary search until it fits (`gamut_map`). Per-channel clipping is never used, because it
//! shifts the hue.

/// What:
///      `const GAMUT_TOLERANCE: f64 = 1e-6;`. `f64` is the 64-bit float, TypeScript's `number`; the siblings are `f32`
///  (32-bit, too coarse for the 30-step chroma search) and the integer types.
/// Why:
///       Linear channel values a hair outside `0..=1` from float rounding still count as inside the gamut, so
///           achromatic colors do not fail the test by a rounding error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GAMUT_TOLERANCE = 1e-6;
/// ```
pub const GAMUT_TOLERANCE: f64 = 1e-6;

/// What:
///      `const CHROMA_SEARCH_STEPS: u32 = 32;`. `u32` is an unsigned 32-bit integer; siblings are `usize`, `u8`, `i32`.
/// Why:
///       Each step halves the chroma interval, so 32 steps pin the in-gamut chroma far below one 8-bit step;
///           `u32` because the count is small and only a `for` range bound uses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CHROMA_SEARCH_STEPS = 32;
/// ```
pub const CHROMA_SEARCH_STEPS: u32 = 32;

/// What:
///      `pub struct Srgb8 { red, green, blue }`: one opaque display color as three `u8` channels. `u8` is an unsigned
///  8-bit integer (0 to 255); siblings are `u16`, `f32`.
/// Why:
///       The window and every contrast check use exactly these quantized values, so a ratio that holds here holds on
///           screen; `Copy` lets the value be passed by value like a TypeScript number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Srgb8 = { red: number; green: number; blue: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Srgb8 {
    /// Red channel, 0 to 255.
    pub red: u8,
    /// Green channel, 0 to 255.
    pub green: u8,
    /// Blue channel, 0 to 255.
    pub blue: u8,
}

/// What:
///      `pub struct Oklch { lightness, chroma, hue }`: a color in cylindrical OKLab. `lightness` is 0 (black) to 1
///  (white), `chroma` is the distance from gray (0 to about 0.4), and `hue` is the angle in degrees, 0 up to 360.
/// Why:
///       Moving one of the three leaves the other two alone, which is what D108 asks every derivation to do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Oklch = { lightness: number; chroma: number; hue: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oklch {
    /// Perceptual lightness, 0 to 1.
    pub lightness: f64,
    /// Distance from the gray axis, never negative.
    pub chroma: f64,
    /// Hue angle in degrees, 0 up to 360.
    pub hue: f64,
}

/// What:
///      `fn srgb_decode(channel: u8) -> f64`: the sRGB transfer curve from an encoded 8-bit channel to linear light.
/// Why:
///       OKLab is defined on linear light, so every conversion from a stored color starts here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const c = channel / 255;
/// return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
/// ```
fn srgb_decode(channel: u8) -> f64 {
    let encoded: f64 = f64::from(channel) / 255.0;
    if encoded <= 0.04045 {
        return encoded / 12.92;
    }
    return ((encoded + 0.055) / 1.055).powf(2.4);
}

/// What:
///      `fn srgb_encode(linear: f64) -> u8`: the inverse transfer curve, clamped to the gamut and rounded to the
///  nearest 8-bit step.
/// Why:
///       The window shows 8-bit colors; rounding (not truncating) keeps the error within half a step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const c = Math.min(1, Math.max(0, linear));
/// const e = c <= 0.0031308 ? c * 12.92 : 1.055 * c ** (1 / 2.4) - 0.055;
/// return Math.round(e * 255);
/// ```
fn srgb_encode(linear: f64) -> u8 {
    let clamped: f64 = linear.clamp(0.0, 1.0);
    let encoded: f64 = if clamped <= 0.003_130_8 {
        clamped * 12.92
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    };
    // `as u8` truncates, and the value is already within 0.0 to 255.0 after the clamp and the rounding.
    return (encoded * 255.0).round() as u8;
}

/// What:
///      `pub fn srgb_to_oklch(color: Srgb8) -> Oklch`: reads a stored color as OKLCH. A color whose three channels
///  are equal has chroma exactly 0 (and hue 0, which carries no meaning without chroma).
/// Why:
///       The system accent arrives as sRGB and every derivation starts from its lightness, chroma and hue. The
///           matrix constants are rounded to ten digits, so an equal-channel color would otherwise read a chroma of
///           about 4e-8 with an arbitrary hue; D118 treats "all channels equal" as "no chroma at all", and a derived
///           color built from such a read must stay exactly gray.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [r, g, b] = [color.red, color.green, color.blue].map(srgbDecode);
/// const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b); // and m, s likewise
/// // then L, a, b from l, m, s, and chroma = hypot(a, b), hue = atan2(b, a) in degrees
/// // equal channels: chroma = 0, hue = 0
/// ```
pub fn srgb_to_oklch(color: Srgb8) -> Oklch {
    let red: f64 = srgb_decode(color.red);
    let green: f64 = srgb_decode(color.green);
    let blue: f64 = srgb_decode(color.blue);
    let cone_l: f64 = (0.412_221_470_8 * red + 0.536_332_536_3 * green + 0.051_445_992_9 * blue).cbrt();
    let cone_m: f64 = (0.211_903_498_2 * red + 0.680_699_545_1 * green + 0.107_396_956_6 * blue).cbrt();
    let cone_s: f64 = (0.088_302_461_9 * red + 0.281_718_837_6 * green + 0.629_978_700_5 * blue).cbrt();
    let lightness: f64 = 0.210_454_255_3 * cone_l + 0.793_617_785_0 * cone_m - 0.004_072_046_8 * cone_s;
    let axis_a: f64 = 1.977_998_495_1 * cone_l - 2.428_592_205_0 * cone_m + 0.450_593_709_9 * cone_s;
    let axis_b: f64 = 0.025_904_037_1 * cone_l + 0.782_771_766_2 * cone_m - 0.808_675_766_0 * cone_s;
    if color.red == color.green && color.green == color.blue {
        return Oklch { lightness, chroma: 0.0, hue: 0.0 };
    }
    let chroma: f64 = axis_a.hypot(axis_b);
    let hue: f64 = axis_b.atan2(axis_a).to_degrees().rem_euclid(360.0);
    return Oklch { lightness, chroma, hue };
}

/// What:
///      `pub fn oklch_to_linear(color: Oklch) -> [f64; 3]`: converts to linear-light red, green and blue WITHOUT
///  clamping; a channel below 0 or above 1 means the color is outside sRGB. `[f64; 3]` is a fixed-length array.
/// Why:
///       The gamut test and the chroma search read the unclamped channels.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const a = chroma * Math.cos(hueRadians), b = chroma * Math.sin(hueRadians);
/// const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3; // and m, s likewise
/// return [4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s, /* green, blue */];
/// ```
pub fn oklch_to_linear(color: Oklch) -> [f64; 3] {
    let hue_radians: f64 = color.hue.to_radians();
    let axis_a: f64 = color.chroma * hue_radians.cos();
    let axis_b: f64 = color.chroma * hue_radians.sin();
    let cone_l: f64 = (color.lightness + 0.396_337_777_4 * axis_a + 0.215_803_757_3 * axis_b).powi(3);
    let cone_m: f64 = (color.lightness - 0.105_561_345_8 * axis_a - 0.063_854_172_8 * axis_b).powi(3);
    let cone_s: f64 = (color.lightness - 0.089_484_177_5 * axis_a - 1.291_485_548_0 * axis_b).powi(3);
    return [
        4.076_741_662_1 * cone_l - 3.307_711_591_3 * cone_m + 0.230_969_929_2 * cone_s,
        -1.268_438_004_6 * cone_l + 2.609_757_401_1 * cone_m - 0.341_319_396_5 * cone_s,
        -0.004_196_086_3 * cone_l - 0.703_418_614_7 * cone_m + 1.707_614_701_0 * cone_s,
    ];
}

/// What:
///      `pub fn is_in_gamut(color: Oklch) -> bool`: true when sRGB can show the color without clamping.
/// Why:
///       The chroma search stops at the largest chroma for which this holds, and tests use it to prove the result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return oklchToLinear(color).every(c => c >= -TOL && c <= 1 + TOL);
/// ```
pub fn is_in_gamut(color: Oklch) -> bool {
    let channels: [f64; 3] = oklch_to_linear(color);
    for channel in channels {
        if channel < -GAMUT_TOLERANCE || channel > 1.0 + GAMUT_TOLERANCE {
            return false;
        }
    }
    return true;
}

/// What:
///      `pub fn gamut_map(color: Oklch) -> Oklch`: returns the same lightness and hue with the largest chroma, up to
///  the requested one, that sRGB can show.
/// Why:
///       Chroma reduction at constant lightness and hue keeps the role's brightness and tint direction; clipping each
///           channel would shift the hue. Lightness at or past the ends is pure white or black (chroma 0).
/// Gotcha:
///       Binary search assumes the in-gamut chroma set is an interval from 0, which holds for sRGB at a fixed
///           lightness and hue because the gamut is convex.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (L >= 1) return { L: 1, C: 0, H }; if (L <= 0) return { L: 0, C: 0, H };
/// if (isInGamut(color)) return color;
/// let lo = 0, hi = chroma;
/// for (let i = 0; i < 32; i++) { const mid = (lo + hi) / 2; isInGamut({ ...color, chroma: mid }) ? (lo = mid) : (hi = mid); }
/// return { ...color, chroma: lo };
/// ```
pub fn gamut_map(color: Oklch) -> Oklch {
    if color.lightness >= 1.0 {
        return Oklch { lightness: 1.0, chroma: 0.0, hue: color.hue };
    }
    if color.lightness <= 0.0 {
        return Oklch { lightness: 0.0, chroma: 0.0, hue: color.hue };
    }
    let wanted: Oklch = Oklch { chroma: color.chroma.max(0.0), ..color };
    if is_in_gamut(wanted) {
        return wanted;
    }
    let mut low: f64 = 0.0;
    let mut high: f64 = wanted.chroma;
    for _step in 0..CHROMA_SEARCH_STEPS {
        let middle: f64 = (low + high) / 2.0;
        if is_in_gamut(Oklch { chroma: middle, ..wanted }) {
            low = middle;
        } else {
            high = middle;
        }
    }
    tracing::trace!(
        lightness = wanted.lightness,
        hue = wanted.hue,
        requested_chroma = wanted.chroma,
        mapped_chroma = low,
        "reduced chroma to fit the sRGB gamut"
    );
    return Oklch { chroma: low, ..wanted };
}

/// What:
///      `pub fn oklch_to_srgb8(color: Oklch) -> Srgb8`: gamut-maps then encodes to 8-bit sRGB. A color with chroma
///  exactly 0 gets one value on all three channels.
/// Why:
///       The single exit from OKLCH to a displayable color for every derived role. The conversion matrix rows sum to
///           1 only to ten digits, so three channels of one gray can sit about 1e-10 apart; a contrast fit lands
///           within 1e-12 of an 8-bit rounding boundary, where that gap splits a gray into channels one step apart.
///           For a completely neutral root (D118) every gray must have equal channels, so zero chroma encodes once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const m = gamutMap(color);
/// if (m.chroma === 0) { const v = srgbEncode(m.lightness ** 3); return { red: v, green: v, blue: v }; }
/// const [r, g, b] = oklchToLinear(m).map(srgbEncode);
/// return { red: r, green: g, blue: b };
/// ```
pub fn oklch_to_srgb8(color: Oklch) -> Srgb8 {
    let mapped: Oklch = gamut_map(color);
    if mapped.chroma == 0.0 {
        let level: u8 = srgb_encode(mapped.lightness.powi(3));
        return Srgb8 { red: level, green: level, blue: level };
    }
    let channels: [f64; 3] = oklch_to_linear(mapped);
    return Srgb8 { red: srgb_encode(channels[0]), green: srgb_encode(channels[1]), blue: srgb_encode(channels[2]) };
}

/// What:
///      `pub fn relative_luminance(color: Srgb8) -> f64`: the WCAG 2 relative luminance (0 black to 1 white).
/// Why:
///       Contrast ratios are defined on it, and the guarantee in `theme_roles.rs` is checked with these exact values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return 0.2126 * lin(red) + 0.7152 * lin(green) + 0.0722 * lin(blue);
/// ```
pub fn relative_luminance(color: Srgb8) -> f64 {
    return 0.2126 * srgb_decode(color.red) + 0.7152 * srgb_decode(color.green) + 0.0722 * srgb_decode(color.blue);
}

/// What:
///      `pub fn contrast_ratio(first: Srgb8, second: Srgb8) -> f64`: the WCAG 2 ratio, 1 to 21, order-independent.
/// Why:
///       AA asks 4.5 for text and 3 for UI parts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x);
/// return (hi + 0.05) / (lo + 0.05);
/// ```
pub fn contrast_ratio(first: Srgb8, second: Srgb8) -> f64 {
    let first_luminance: f64 = relative_luminance(first);
    let second_luminance: f64 = relative_luminance(second);
    let lighter: f64 = first_luminance.max(second_luminance);
    let darker: f64 = first_luminance.min(second_luminance);
    return (lighter + 0.05) / (darker + 0.05);
}

/// What:
///      `pub fn composite_over(layer: Srgb8, alpha: f64, surface: Srgb8) -> Srgb8`: the color a layer of `alpha`
///  coverage shows over an opaque surface, the same blend the renderer performs for a translucent fill.
/// Why:
///       Hover and pressed state layers are alpha-only (D108 allows that); contrast checks need the resulting
///           background. This models rendering and creates no theme color.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mix = (l, s) => Math.round(l * alpha + s * (1 - alpha));
/// ```
pub fn composite_over(layer: Srgb8, alpha: f64, surface: Srgb8) -> Srgb8 {
    let blend = |layer_channel: u8, surface_channel: u8| -> u8 {
        let value: f64 = f64::from(layer_channel) * alpha + f64::from(surface_channel) * (1.0 - alpha);
        return value.round().clamp(0.0, 255.0) as u8;
    };
    return Srgb8 {
        red: blend(layer.red, surface.red),
        green: blend(layer.green, surface.green),
        blue: blend(layer.blue, surface.blue),
    };
}

/// What:
///      `pub fn mix_oklch(from: Oklch, to: Oklch, fraction: f64) -> Oklch`: interpolates lightness, chroma and hue
///  (the short way round the hue circle). A color with no chroma has no hue, so it takes the other color's hue.
/// Why:
///       D108 replaces sRGB mixes with this: hover and pressed container fills move a role toward its content color
///           in OKLCH.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const diff = ((to.hue - from.hue + 540) % 360) - 180;
/// return { lightness: lerp(from.L, to.L, t), chroma: lerp(from.C, to.C, t), hue: (from.hue + diff * t + 360) % 360 };
/// ```
pub fn mix_oklch(from: Oklch, to: Oklch, fraction: f64) -> Oklch {
    let lightness: f64 = from.lightness + (to.lightness - from.lightness) * fraction;
    let chroma: f64 = from.chroma + (to.chroma - from.chroma) * fraction;
    let hue_from: f64 = if from.chroma < 1e-4 { to.hue } else { from.hue };
    let hue_to: f64 = if to.chroma < 1e-4 { from.hue } else { to.hue };
    let difference: f64 = (hue_to - hue_from + 540.0).rem_euclid(360.0) - 180.0;
    let hue: f64 = (hue_from + difference * fraction).rem_euclid(360.0);
    return Oklch { lightness, chroma, hue };
}

/// What:
///      `pub fn delta_e_ok(first: Srgb8, second: Srgb8) -> f64`: the distance between two colors in OKLab, times 100
///  (so about 1 is a just-noticeable difference).
/// Why:
///       Reports how far a derived role moved from the captured fixed value, and bounds the baseline drift in tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return 100 * Math.hypot(a.L - b.L, a.a - b.a, a.b - b.b);
/// ```
pub fn delta_e_ok(first: Srgb8, second: Srgb8) -> f64 {
    let first_lch: Oklch = srgb_to_oklch(first);
    let second_lch: Oklch = srgb_to_oklch(second);
    let first_a: f64 = first_lch.chroma * first_lch.hue.to_radians().cos();
    let first_b: f64 = first_lch.chroma * first_lch.hue.to_radians().sin();
    let second_a: f64 = second_lch.chroma * second_lch.hue.to_radians().cos();
    let second_b: f64 = second_lch.chroma * second_lch.hue.to_radians().sin();
    let delta_l: f64 = first_lch.lightness - second_lch.lightness;
    return 100.0 * (delta_l * delta_l + (first_a - second_a).powi(2) + (first_b - second_b).powi(2)).sqrt();
}

#[cfg(test)]
#[path = "oklch_tests.rs"]
mod tests;
