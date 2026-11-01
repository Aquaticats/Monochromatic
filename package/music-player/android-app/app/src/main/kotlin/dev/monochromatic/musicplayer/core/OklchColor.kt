// The one OKLCH implementation of the app (D108): sRGB and OKLCH conversion, gamut mapping by chroma reduction, WCAG
// contrast and OKLCH mixing, all in Double and with no Compose or Android import, so it runs on the JVM unit tests.
// It is a line-for-line port of the desktop's `desktop-app/src/oklch.rs`, so the same root gives the same sRGB roles in
// Kotlin and Rust (the shared test vector `theme-roles-vector.txt` proves that). `ThemeRoles.kt` is its only caller.
// Gamut handling: a requested lightness, chroma and hue that sRGB cannot show keeps its lightness and hue, and the
// chroma is reduced by binary search until it fits. Per-channel clipping is never used, because it shifts the hue.

// What:     `package dev.monochromatic.musicplayer.core` places the color math beside the other pure helpers.
// Why:      The math takes plain numbers only, so it needs no import and is unit-tested on the JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import kotlin.math.*` brings in the trigonometric and rounding functions the conversions use.
// Why:      OKLCH is polar (hue is an angle) and the 8-bit exit rounds, so the standard math functions are needed.
//
// In TS you'd write (pseudocode):
// ```ts
// // Math.atan2, Math.cbrt, Math.hypot, Math.pow are global.
// ```
import kotlin.math.atan2
import kotlin.math.cbrt
import kotlin.math.cos
import kotlin.math.hypot
import kotlin.math.pow
import kotlin.math.roundToInt
import kotlin.math.sin
import kotlin.math.sqrt

// What:     `internal const val GAMUT_TOLERANCE: Double = 1e-6` is the slack of the in-gamut test.
// Why:      Linear channel values a hair outside 0 to 1 from float rounding still count as inside the gamut, so
//           achromatic colors do not fail the test by a rounding error.
//
// In TS you'd write (pseudocode):
// ```ts
// const GAMUT_TOLERANCE = 1e-6;
// ```
/** Slack in linear light below 0 and above 1 that still counts as inside the sRGB gamut. */
internal const val GAMUT_TOLERANCE: Double = 1e-6

// What:     `internal const val CHROMA_SEARCH_STEPS: Int = 32` is the number of halvings of the chroma interval.
// Why:      32 halvings pin the in-gamut chroma far below one 8-bit step.
//
// In TS you'd write (pseudocode):
// ```ts
// const CHROMA_SEARCH_STEPS = 32;
// ```
/** Number of binary-search steps that find the largest chroma sRGB can show. */
internal const val CHROMA_SEARCH_STEPS: Int = 32

// What:     `private const val CHANNEL_MAX: Double = 255.0` is the largest 8-bit channel value.
// Why:      The conversions scale between the unit interval and the 8-bit steps with it.
//
// In TS you'd write (pseudocode):
// ```ts
// const CHANNEL_MAX = 255;
// ```
/** Largest 8-bit channel value as a Double. */
private const val CHANNEL_MAX: Double = 255.0

// What:     `private const val HUE_TURN: Double = 360.0` is a full turn of the hue circle in degrees.
// Why:      Hues are wrapped into 0 up to 360 after every shift.
//
// In TS you'd write (pseudocode):
// ```ts
// const HUE_TURN = 360;
// ```
/** One full turn of the hue circle in degrees. */
private const val HUE_TURN: Double = 360.0

// What:     `private const val HUE_HALF_TURN: Double = 180.0` is half a turn of the hue circle in degrees.
// Why:      The short way round the circle is the difference folded into minus 180 up to 180.
//
// In TS you'd write (pseudocode):
// ```ts
// const HUE_HALF_TURN = 180;
// ```
/** Half a turn of the hue circle in degrees. */
private const val HUE_HALF_TURN: Double = 180.0

// What:     `private const val HUE_FOLD_OFFSET: Double = 540.0` shifts a hue difference to be positive.
// Why:      The short way round the circle is the difference plus 540 wrapped, minus 180.
//
// In TS you'd write (pseudocode):
// ```ts
// const HUE_FOLD_OFFSET = 540;
// ```
/** Offset that makes a hue difference positive before it is folded into minus 180 up to 180. */
private const val HUE_FOLD_OFFSET: Double = 540.0

// What:     `private const val NO_HUE_CHROMA: Double = 1e-4` is the chroma below which a color has no usable hue.
// Why:      A gray has no hue, so a mix takes the other color's hue instead of an arbitrary angle.
//
// In TS you'd write (pseudocode):
// ```ts
// const NO_HUE_CHROMA = 1e-4;
// ```
/** Chroma below which a color counts as gray and borrows the hue of the color it is mixed with. */
private const val NO_HUE_CHROMA: Double = 1e-4

// What:     `internal data class Srgb8` is one opaque display color as three 8-bit channels.
// Why:      The window and every contrast check use exactly these quantized values, so a ratio that holds here holds
//           on screen.
//
// In TS you'd write (pseudocode):
// ```ts
// type Srgb8 = { red: number; green: number; blue: number };
// ```
/** One opaque sRGB color with each channel in 0 to 255. */
internal data class Srgb8(
    /** Red channel, 0 to 255. */
    val red: Int,
    /** Green channel, 0 to 255. */
    val green: Int,
    /** Blue channel, 0 to 255. */
    val blue: Int,
) {
    /** Packs the channels as opaque ARGB, the form Compose's `Color(Int)` takes. */
    fun toArgb(): Int = (OPAQUE_ALPHA shl ALPHA_SHIFT) or (red shl RED_SHIFT) or (green shl GREEN_SHIFT) or blue

    /** Formats the channels as six upper-case hexadecimal digits, the form the shared test vector stores. */
    fun toHex(): String = "%02X%02X%02X".format(red, green, blue)

    /** Constants and the packed-integer reader of an 8-bit color. */
    internal companion object {
        /** Alpha byte of an opaque color. */
        private const val OPAQUE_ALPHA: Int = 0xFF

        /** Bit position of the alpha byte in packed ARGB. */
        private const val ALPHA_SHIFT: Int = 24

        /** Bit position of the red byte in packed ARGB. */
        private const val RED_SHIFT: Int = 16

        /** Bit position of the green byte in packed ARGB. */
        private const val GREEN_SHIFT: Int = 8

        /** Low 24 bits of packed ARGB, which hold the three color channels. */
        private const val RGB_MASK: Int = 0xFFFFFF

        /** Reads the three color channels of a packed ARGB or RGB integer and ignores its alpha byte. */
        fun fromRgbInt(packed: Int): Srgb8 {
            /** The three color bytes of the packed integer. */
            val rgb: Int = packed and RGB_MASK
            return Srgb8(red = (rgb shr RED_SHIFT) and OPAQUE_ALPHA, green = (rgb shr GREEN_SHIFT) and OPAQUE_ALPHA,
                blue = rgb and OPAQUE_ALPHA)
        }
    }
}

// What:     `internal data class SrgbUnit` is an sRGB color as three encoded channels in the unit interval.
// Why:      Compose stores float channels; keeping the unquantized value lets the adapters avoid an 8-bit round trip.
//
// In TS you'd write (pseudocode):
// ```ts
// type SrgbUnit = { red: number; green: number; blue: number }; // each 0..1
// ```
/** One sRGB color with each encoded channel in 0 to 1 and no quantization. */
internal data class SrgbUnit(
    /** Red channel, 0 to 1. */
    val red: Double,
    /** Green channel, 0 to 1. */
    val green: Double,
    /** Blue channel, 0 to 1. */
    val blue: Double,
)

// What:     `internal data class Oklch` is a color in cylindrical OKLab.
// Why:      Moving one of the three leaves the other two alone, which is what D108 asks every derivation to do.
//
// In TS you'd write (pseudocode):
// ```ts
// type Oklch = { lightness: number; chroma: number; hue: number };
// ```
/** A color in OKLCH: lightness 0 to 1, chroma 0 to about 0.4, and hue in degrees from 0 up to 360. */
internal data class Oklch(
    /** Perceptual lightness, 0 (black) to 1 (white). */
    val lightness: Double,
    /** Distance from the gray axis, never negative. */
    val chroma: Double,
    /** Hue angle in degrees, 0 up to 360. */
    val hue: Double,
)

// What:     `internal data class LinearRgb` is linear-light red, green and blue that may lie outside 0 to 1.
// Why:      The gamut test and the chroma search read the unclamped channels; a value outside 0 to 1 means the color is
//           outside sRGB.
//
// In TS you'd write (pseudocode):
// ```ts
// type LinearRgb = [number, number, number];
// ```
/** Linear-light channels of a color, unclamped. */
internal data class LinearRgb(
    /** Linear red. */
    val red: Double,
    /** Linear green. */
    val green: Double,
    /** Linear blue. */
    val blue: Double,
)

// What:     `private fun wrapHue(degrees)` folds an angle into 0 up to 360.
// Why:      Matches Rust's `rem_euclid(360.0)`, because Kotlin's `%` keeps the sign of the dividend.
//
// In TS you'd write (pseudocode):
// ```ts
// const wrapHue = (d: number) => { const r = d % 360; return r < 0 ? r + 360 : r; };
// ```
/** Wraps an angle in degrees into 0 up to 360. */
internal fun wrapHue(degrees: Double): Double {
    /** Remainder of the division, which keeps the dividend's sign. */
    val remainder: Double = degrees % HUE_TURN
    return if (remainder < 0.0) remainder + HUE_TURN else remainder
}

// What:     `private fun srgbDecode(encoded)` is the sRGB transfer curve from an encoded channel to linear light.
// Why:      OKLab is defined on linear light, so every conversion from a stored color starts here.
//
// In TS you'd write (pseudocode):
// ```ts
// const decode = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
// ```
/** Converts one encoded sRGB channel in 0 to 1 to linear light. */
private fun srgbDecode(encoded: Double): Double {
    if (encoded <= SRGB_DECODE_KNEE) {
        return encoded / SRGB_LINEAR_SLOPE
    }
    return ((encoded + SRGB_OFFSET) / SRGB_DIVISOR).pow(SRGB_GAMMA)
}

/** Encoded value up to which the sRGB curve is the linear segment. */
private const val SRGB_DECODE_KNEE: Double = 0.04045

/** Slope of the sRGB linear segment. */
private const val SRGB_LINEAR_SLOPE: Double = 12.92

/** Offset of the sRGB power segment. */
private const val SRGB_OFFSET: Double = 0.055

/** Divisor and scale of the sRGB power segment (one plus the offset, written out to match the Rust literal). */
private const val SRGB_DIVISOR: Double = 1.055

/** Exponent of the sRGB power segment. */
private const val SRGB_GAMMA: Double = 2.4

/** Linear value up to which the inverse sRGB curve is the linear segment. */
private const val SRGB_ENCODE_KNEE: Double = 0.0031308

// What:     `private fun srgbEncode(linear)` is the inverse transfer curve, clamped to the gamut.
// Why:      The exit to 8 bits rounds this value, and the Compose adapter uses it unrounded.
//
// In TS you'd write (pseudocode):
// ```ts
// const encode = (l: number) => {
//   const c = Math.min(1, Math.max(0, l));
//   return c <= 0.0031308 ? c * 12.92 : 1.055 * c ** (1 / 2.4) - 0.055;
// };
// ```
/** Converts linear light to one encoded sRGB channel, clamped to 0 to 1. */
private fun srgbEncode(linear: Double): Double {
    /** Linear light limited to 0 to 1. */
    val clamped: Double = linear.coerceIn(0.0, 1.0)
    if (clamped <= SRGB_ENCODE_KNEE) {
        return clamped * SRGB_LINEAR_SLOPE
    }
    return SRGB_DIVISOR * clamped.pow(1.0 / SRGB_GAMMA) - SRGB_OFFSET
}

// What:     `internal fun srgbUnitToOklch(color)` reads an unquantized sRGB color as OKLCH.
// Why:      The adapters hand it Compose's float channels; `srgbToOklch` hands it 8-bit channels over 255.
//
// In TS you'd write (pseudocode):
// ```ts
// const [r, g, b] = [color.red, color.green, color.blue].map(decode);
// const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b); // and m, s likewise
// // then L, a, b from l, m, s, and chroma = hypot(a, b), hue = atan2(b, a) in degrees
// ```
/** Converts an sRGB color with channels in 0 to 1 to OKLCH. */
internal fun srgbUnitToOklch(color: SrgbUnit): Oklch {
    /** Linear red. */
    val red: Double = srgbDecode(color.red)
    /** Linear green. */
    val green: Double = srgbDecode(color.green)
    /** Linear blue. */
    val blue: Double = srgbDecode(color.blue)
    /** Long-wavelength cone response. */
    val coneL: Double = cbrt(0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue)
    /** Medium-wavelength cone response. */
    val coneM: Double = cbrt(0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue)
    /** Short-wavelength cone response. */
    val coneS: Double = cbrt(0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue)
    /** OKLab lightness. */
    val lightness: Double = 0.2104542553 * coneL + 0.7936177850 * coneM - 0.0040720468 * coneS
    /** OKLab green-red axis. */
    val axisA: Double = 1.9779984951 * coneL - 2.4285922050 * coneM + 0.4505937099 * coneS
    /** OKLab blue-yellow axis. */
    val axisB: Double = 0.0259040371 * coneL + 0.7827717662 * coneM - 0.8086757660 * coneS
    return Oklch(
        lightness = lightness,
        chroma = hypot(axisA, axisB),
        hue = wrapHue(Math.toDegrees(atan2(axisB, axisA))),
    )
}

// What:     `internal fun srgbToOklch(color)` reads a stored 8-bit color as OKLCH. A color whose three channels are
//           equal has chroma exactly 0 (and hue 0, which carries no meaning without chroma).
// Why:      The system accent arrives as 8-bit sRGB and every derivation starts from its lightness, chroma and hue.
//           The matrix constants are rounded to ten digits, so an equal-channel color would otherwise read a chroma of
//           about 4e-8 with an arbitrary hue; D118 treats "all channels equal" as "no chroma at all", and a derived
//           color built from such a read must stay exactly gray.
//
// In TS you'd write (pseudocode):
// ```ts
// const lch = srgbUnitToOklch({ red: color.red / 255, green: color.green / 255, blue: color.blue / 255 });
// return color.red === color.green && color.green === color.blue ? { ...lch, chroma: 0, hue: 0 } : lch;
// ```
/** Converts an 8-bit sRGB color to OKLCH, with exactly zero chroma for equal channels. */
internal fun srgbToOklch(color: Srgb8): Oklch {
    /** The plain conversion, whose chroma for a gray is float noise. */
    val converted: Oklch = srgbUnitToOklch(
        SrgbUnit(color.red / CHANNEL_MAX, color.green / CHANNEL_MAX, color.blue / CHANNEL_MAX),
    )
    if (color.red == color.green && color.green == color.blue) {
        return Oklch(lightness = converted.lightness, chroma = 0.0, hue = 0.0)
    }
    return converted
}

// What:     `internal fun oklchToLinear(color)` converts to linear-light red, green and blue WITHOUT clamping.
// Why:      The gamut test and the chroma search read the unclamped channels.
//
// In TS you'd write (pseudocode):
// ```ts
// const a = chroma * Math.cos(hueRadians), b = chroma * Math.sin(hueRadians);
// const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3; // and m, s likewise
// return [4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s, /* green, blue */];
// ```
/** Converts OKLCH to linear sRGB without clamping; a channel outside 0 to 1 means the color is outside sRGB. */
internal fun oklchToLinear(color: Oklch): LinearRgb {
    /** Hue converted to radians. */
    val hueRadians: Double = Math.toRadians(color.hue)
    /** OKLab green-red axis. */
    val axisA: Double = color.chroma * cos(hueRadians)
    /** OKLab blue-yellow axis. */
    val axisB: Double = color.chroma * sin(hueRadians)
    /** Long-wavelength cone response. */
    val coneL: Double = cube(color.lightness + 0.3963377774 * axisA + 0.2158037573 * axisB)
    /** Medium-wavelength cone response. */
    val coneM: Double = cube(color.lightness - 0.1055613458 * axisA - 0.0638541728 * axisB)
    /** Short-wavelength cone response. */
    val coneS: Double = cube(color.lightness - 0.0894841775 * axisA - 1.2914855480 * axisB)
    return LinearRgb(
        red = 4.0767416621 * coneL - 3.3077115913 * coneM + 0.2309699292 * coneS,
        green = -1.2684380046 * coneL + 2.6097574011 * coneM - 0.3413193965 * coneS,
        blue = -0.0041960863 * coneL - 0.7034186147 * coneM + 1.7076147010 * coneS,
    )
}

/** Cubes a value as two multiplications, the way Rust's `powi(3)` compiles. */
private fun cube(value: Double): Double = value * value * value

// What:     `internal fun isInGamut(color)` is true when sRGB can show the color without clamping.
// Why:      The chroma search stops at the largest chroma for which this holds, and tests use it to prove the result.
//
// In TS you'd write (pseudocode):
// ```ts
// return oklchToLinear(color).every(c => c >= -TOL && c <= 1 + TOL);
// ```
/** True when all three linear channels lie within 0 to 1 plus the tolerance. */
internal fun isInGamut(color: Oklch): Boolean {
    /** Linear-light channels of the color. */
    val linear: LinearRgb = oklchToLinear(color)
    return listOf(linear.red, linear.green, linear.blue).all { channel ->
        channel >= -GAMUT_TOLERANCE && channel <= 1.0 + GAMUT_TOLERANCE
    }
}

// What:     `internal fun gamutMap(color)` returns the same lightness and hue with the largest chroma, up to the
//           requested one, that sRGB can show.
// Why:      Chroma reduction at constant lightness and hue keeps the role's brightness and tint direction; clipping
//           each channel would shift the hue. Lightness at or past the ends is pure white or black (chroma 0).
// Gotcha:   Binary search assumes the in-gamut chroma set is an interval from 0, which holds for sRGB at a fixed
//           lightness and hue because the gamut is convex.
//
// In TS you'd write (pseudocode):
// ```ts
// if (L >= 1) return { L: 1, C: 0, H }; if (L <= 0) return { L: 0, C: 0, H };
// if (isInGamut(color)) return color;
// let lo = 0, hi = chroma;
// for (let i = 0; i < 32; i++) {
//   const mid = (lo + hi) / 2;
//   isInGamut({ ...color, chroma: mid }) ? (lo = mid) : (hi = mid);
// }
// return { ...color, chroma: lo };
// ```
/** Reduces chroma at constant lightness and hue until the color fits in sRGB. */
internal fun gamutMap(color: Oklch): Oklch {
    if (color.lightness >= 1.0) {
        return Oklch(lightness = 1.0, chroma = 0.0, hue = color.hue)
    }
    if (color.lightness <= 0.0) {
        return Oklch(lightness = 0.0, chroma = 0.0, hue = color.hue)
    }
    /** Requested color with a non-negative chroma. */
    val wanted: Oklch = color.copy(chroma = maxOf(color.chroma, 0.0))
    if (isInGamut(wanted)) {
        return wanted
    }
    /** Largest chroma known to fit. */
    var low = 0.0
    /** Smallest chroma known not to fit. */
    var high: Double = wanted.chroma
    repeat(CHROMA_SEARCH_STEPS) {
        /** Chroma halfway between the bounds. */
        val middle: Double = (low + high) / 2.0
        if (isInGamut(wanted.copy(chroma = middle))) {
            low = middle
        } else {
            high = middle
        }
    }
    return wanted.copy(chroma = low)
}

// What:     `internal fun oklchToSrgbUnit(color)` gamut-maps then encodes to unquantized sRGB. A color with chroma
//           exactly 0 gets one value on all three channels.
// Why:      The Compose adapters keep float precision; the 8-bit exit rounds this value. The conversion matrix rows
//           sum to 1 only to ten digits, so three channels of one gray can sit about 1e-10 apart; a contrast fit lands
//           within 1e-12 of an 8-bit rounding boundary, where that gap splits a gray into channels one step apart.
//           For a completely neutral root (D118) every gray must have equal channels, so zero chroma encodes once.
//
// In TS you'd write (pseudocode):
// ```ts
// const m = gamutMap(color);
// if (m.chroma === 0) { const v = encode(m.lightness ** 3); return { red: v, green: v, blue: v }; }
// const [r, g, b] = oklchToLinear(m); return { red: encode(r), green: encode(g), blue: encode(b) };
// ```
/** Converts OKLCH to sRGB channels in 0 to 1 after chroma-reduction gamut mapping. */
internal fun oklchToSrgbUnit(color: Oklch): SrgbUnit {
    /** The color reduced to what sRGB can show. */
    val mapped: Oklch = gamutMap(color)
    if (mapped.chroma == 0.0) {
        /** The one encoded value of an achromatic color. */
        val level: Double = srgbEncode(cube(mapped.lightness))
        return SrgbUnit(level, level, level)
    }
    /** Linear-light channels of the mapped color. */
    val linear: LinearRgb = oklchToLinear(mapped)
    return SrgbUnit(srgbEncode(linear.red), srgbEncode(linear.green), srgbEncode(linear.blue))
}

// What:     `internal fun oklchToSrgb8(color)` gamut-maps then encodes to 8-bit sRGB.
// Why:      The single exit from OKLCH to a displayable color for every derived role.
//
// In TS you'd write (pseudocode):
// ```ts
// const u = oklchToSrgbUnit(color); return { red: Math.round(u.red * 255), green: ..., blue: ... };
// ```
/** Converts OKLCH to 8-bit sRGB, rounding each channel to the nearest step. */
internal fun oklchToSrgb8(color: Oklch): Srgb8 {
    /** Gamut-mapped channels in 0 to 1. */
    val unit: SrgbUnit = oklchToSrgbUnit(color)
    return Srgb8(
        red = (unit.red * CHANNEL_MAX).roundToInt(),
        green = (unit.green * CHANNEL_MAX).roundToInt(),
        blue = (unit.blue * CHANNEL_MAX).roundToInt(),
    )
}

// What:     `internal fun relativeLuminance(color)` is the WCAG 2 relative luminance (0 black to 1 white).
// Why:      Contrast ratios are defined on it, and the guarantee in `ThemeRoles.kt` is checked with these exact values.
//
// In TS you'd write (pseudocode):
// ```ts
// return 0.2126 * lin(red) + 0.7152 * lin(green) + 0.0722 * lin(blue);
// ```
/** Relative luminance of an 8-bit sRGB color. */
internal fun relativeLuminance(color: Srgb8): Double =
    LUMINANCE_RED * srgbDecode(color.red / CHANNEL_MAX) +
        LUMINANCE_GREEN * srgbDecode(color.green / CHANNEL_MAX) +
        LUMINANCE_BLUE * srgbDecode(color.blue / CHANNEL_MAX)

/** Weight of linear red in WCAG relative luminance. */
private const val LUMINANCE_RED: Double = 0.2126

/** Weight of linear green in WCAG relative luminance. */
private const val LUMINANCE_GREEN: Double = 0.7152

/** Weight of linear blue in WCAG relative luminance. */
private const val LUMINANCE_BLUE: Double = 0.0722

// What:     `internal fun contrastRatio(first, second)` is the WCAG 2 ratio, 1 to 21, order-independent.
// Why:      AA asks 4.5 for text and 3 for UI parts.
//
// In TS you'd write (pseudocode):
// ```ts
// const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x); return (hi + 0.05) / (lo + 0.05);
// ```
/** WCAG 2 contrast ratio of two opaque colors. */
internal fun contrastRatio(first: Srgb8, second: Srgb8): Double {
    /** Relative luminance of the first color. */
    val firstLuminance: Double = relativeLuminance(first)
    /** Relative luminance of the second color. */
    val secondLuminance: Double = relativeLuminance(second)
    return (maxOf(firstLuminance, secondLuminance) + CONTRAST_OFFSET) /
        (minOf(firstLuminance, secondLuminance) + CONTRAST_OFFSET)
}

/** Luminance offset of the WCAG 2 contrast ratio. */
private const val CONTRAST_OFFSET: Double = 0.05

// What:     `internal fun compositeOver(layer, alpha, surface)` is the color a layer of `alpha` coverage shows over an
//           opaque surface, the same blend the renderer performs for a translucent fill.
// Why:      Hover and pressed state layers are alpha-only (D108 allows that); contrast checks need the resulting
//           background. This models rendering and creates no theme color.
//
// In TS you'd write (pseudocode):
// ```ts
// const mix = (l, s) => Math.round(l * alpha + s * (1 - alpha));
// ```
/** Models the 8-bit result of an alpha layer drawn over an opaque surface, for contrast checks. */
internal fun compositeOver(layer: Srgb8, alpha: Double, surface: Srgb8): Srgb8 {
    /** Blends one channel of the layer over the surface and rounds to a step. */
    fun blend(layerChannel: Int, surfaceChannel: Int): Int =
        (layerChannel * alpha + surfaceChannel * (1.0 - alpha)).roundToInt().coerceIn(0, CHANNEL_MAX.toInt())
    return Srgb8(
        red = blend(layer.red, surface.red),
        green = blend(layer.green, surface.green),
        blue = blend(layer.blue, surface.blue),
    )
}

// What:     `internal fun mixOklch(from, to, fraction)` interpolates lightness, chroma and hue (the short way round the
//           hue circle). A color with no chroma has no hue, so it takes the other color's hue.
// Why:      D108 replaces sRGB mixes with this: hover and pressed container fills move a role toward its content
//           color in OKLCH.
//
// In TS you'd write (pseudocode):
// ```ts
// const diff = ((to.hue - from.hue + 540) % 360) - 180;
// return { lightness: lerp(from.L, to.L, t), chroma: lerp(from.C, to.C, t), hue: (from.hue + diff * t + 360) % 360 };
// ```
/** Mixes two colors in OKLCH by [fraction], taking the short way round the hue circle. */
internal fun mixOklch(from: Oklch, to: Oklch, fraction: Double): Oklch {
    /** Start hue, borrowed from the end when the start is gray. */
    val hueFrom: Double = if (from.chroma < NO_HUE_CHROMA) to.hue else from.hue
    /** End hue, borrowed from the start when the end is gray. */
    val hueTo: Double = if (to.chroma < NO_HUE_CHROMA) from.hue else to.hue
    /** Signed short-way hue difference in degrees. */
    val difference: Double = wrapHue(hueTo - hueFrom + HUE_FOLD_OFFSET) - HUE_HALF_TURN
    return Oklch(
        lightness = from.lightness + (to.lightness - from.lightness) * fraction,
        chroma = from.chroma + (to.chroma - from.chroma) * fraction,
        hue = wrapHue(hueFrom + difference * fraction),
    )
}

// What:     `internal fun deltaEOk(first, second)` is the distance between two colors in OKLab, times 100.
// Why:      About 1 is a just-noticeable difference; tests bound how far a derived role drifts from a reference.
//
// In TS you'd write (pseudocode):
// ```ts
// return 100 * Math.hypot(a.L - b.L, a.a - b.a, a.b - b.b);
// ```
/** Distance between two colors in OKLab, scaled by 100. */
internal fun deltaEOk(first: Srgb8, second: Srgb8): Double {
    /** First color in OKLCH. */
    val firstLch: Oklch = srgbToOklch(first)
    /** Second color in OKLCH. */
    val secondLch: Oklch = srgbToOklch(second)
    /** First color's green-red axis. */
    val firstA: Double = firstLch.chroma * cos(Math.toRadians(firstLch.hue))
    /** First color's blue-yellow axis. */
    val firstB: Double = firstLch.chroma * sin(Math.toRadians(firstLch.hue))
    /** Second color's green-red axis. */
    val secondA: Double = secondLch.chroma * cos(Math.toRadians(secondLch.hue))
    /** Second color's blue-yellow axis. */
    val secondB: Double = secondLch.chroma * sin(Math.toRadians(secondLch.hue))
    /** Lightness difference. */
    val deltaL: Double = firstLch.lightness - secondLch.lightness
    return DELTA_E_SCALE * sqrt(deltaL * deltaL + (firstA - secondA).pow(2) + (firstB - secondB).pow(2))
}

/** Scale that makes one unit of OKLab distance a hundred, so about 1 is a just-noticeable difference. */
private const val DELTA_E_SCALE: Double = 100.0
