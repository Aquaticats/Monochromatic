// What:     `package dev.monochromatic.musicplayer.core` places this test beside the OKLCH math.
// Why:      The test reaches the internal pure functions in the same package as the code under test.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies the OKLCH conversions, the gamut mapping, the contrast ratio and the mixing of `OklchColor.kt`. */
class OklchMathTest {
    /** Confirms every 8-bit gray, primary and a spread of colors survive sRGB to OKLCH and back unchanged. */
    @Test
    fun eightBitColorsRoundTripExactly() {
        for (red in listOf(0, 1, 37, 128, 200, 255)) {
            for (green in listOf(0, 3, 90, 177, 255)) {
                for (blue in listOf(0, 9, 64, 129, 254, 255)) {
                    val color = Srgb8(red, green, blue)
                    assertEquals(color, oklchToSrgb8(srgbToOklch(color)))
                }
            }
        }
    }

    /** Confirms black and white have zero chroma and the expected lightness. */
    @Test
    fun blackAndWhiteAreAchromaticEnds() {
        assertEquals(0.0, srgbToOklch(Srgb8(0, 0, 0)).lightness, TOLERANCE)
        assertEquals(1.0, srgbToOklch(Srgb8(255, 255, 255)).lightness, TOLERANCE)
        assertTrue(srgbToOklch(Srgb8(255, 255, 255)).chroma < TOLERANCE)
    }

    /** Confirms every equal-channel color reads exactly zero chroma while a one-step offset does not (D118). */
    @Test
    fun anEqualChannelColorReadsAsExactlyAchromatic() {
        for (level in 0..255) {
            assertEquals("gray $level", 0.0, srgbToOklch(Srgb8(level, level, level)).chroma, 0.0)
        }
        assertTrue(srgbToOklch(Srgb8(129, 128, 128)).chroma > 1.0e-3)
    }

    /** Confirms zero chroma always encodes to equal channels, even within 1e-12 of an 8-bit rounding boundary. */
    @Test
    fun zeroChromaAlwaysEncodesToEqualChannelsEvenOnARoundingBoundary() {
        for (step in 0..20_000) {
            val color: Srgb8 = oklchToSrgb8(Oklch(step / 20_000.0, 0.0, 123.0))
            assertTrue("step $step: $color", color.red == color.green && color.green == color.blue)
        }
        for (level in 0 until 255) {
            val encoded: Double = (level + 0.5) / 255.0
            val linear: Double =
                if (encoded <= 0.04045) encoded / 12.92 else Math.pow((encoded + 0.055) / 1.055, 2.4)
            val boundary: Double = Math.cbrt(linear)
            for (nudge in listOf(-1.0e-9, -1.0e-12, 0.0, 1.0e-12, 1.0e-9)) {
                val color: Srgb8 = oklchToSrgb8(Oklch(boundary + nudge, 0.0, 0.0))
                assertTrue("level $level nudge $nudge: $color", color.red == color.green && color.green == color.blue)
            }
        }
    }

    /** Confirms a color outside the gamut keeps its lightness and hue and loses only chroma, and lands inside. */
    @Test
    fun gamutMappingReducesChromaAndKeepsLightnessAndHue() {
        val wanted = Oklch(lightness = 0.9, chroma = 0.35, hue = 250.0)
        val mapped: Oklch = gamutMap(wanted)
        assertTrue(!isInGamut(wanted))
        assertTrue(isInGamut(mapped))
        assertEquals(wanted.lightness, mapped.lightness, 0.0)
        assertEquals(wanted.hue, mapped.hue, 0.0)
        assertTrue(mapped.chroma < wanted.chroma)
        assertTrue(mapped.chroma > 0.0)
    }

    /** Confirms lightness at the ends maps to pure white and black. */
    @Test
    fun lightnessAtTheEndsIsPureWhiteAndBlack() {
        assertEquals(Srgb8(255, 255, 255), oklchToSrgb8(Oklch(1.2, 0.2, 30.0)))
        assertEquals(Srgb8(0, 0, 0), oklchToSrgb8(Oklch(-0.1, 0.2, 30.0)))
    }

    /** Confirms the WCAG ratio of black on white is 21 and is symmetric. */
    @Test
    fun contrastOfBlackOnWhiteIsTwentyOne() {
        val black = Srgb8(0, 0, 0)
        val white = Srgb8(255, 255, 255)
        assertEquals(21.0, contrastRatio(black, white), 1e-9)
        assertEquals(contrastRatio(black, white), contrastRatio(white, black), 0.0)
    }

    /** Confirms hue wrapping matches Rust's `rem_euclid` for negative and large angles. */
    @Test
    fun hueWrappingIsEuclidean() {
        assertEquals(350.0, wrapHue(-10.0), 1e-12)
        assertEquals(10.0, wrapHue(370.0), 1e-12)
        assertEquals(0.0, wrapHue(720.0), 1e-12)
    }

    /** Confirms mixing takes the short way round the hue circle and borrows the hue of a gray end. */
    @Test
    fun mixingTakesTheShortWayAndGrayBorrowsTheHue() {
        val mixed: Oklch = mixOklch(Oklch(0.5, 0.1, 350.0), Oklch(0.7, 0.1, 10.0), 0.5)
        assertEquals(0.0, mixed.hue, 1e-9)
        assertEquals(0.6, mixed.lightness, 1e-12)
        val fromGray: Oklch = mixOklch(Oklch(0.5, 0.0, 0.0), Oklch(0.7, 0.1, 120.0), 0.5)
        assertEquals(120.0, fromGray.hue, 1e-9)
    }

    /** Confirms compositing a layer at 0 and 1 coverage returns the surface and the layer. */
    @Test
    fun compositeEndsAreTheSurfaceAndTheLayer() {
        val layer = Srgb8(200, 10, 90)
        val surface = Srgb8(20, 220, 130)
        assertEquals(surface, compositeOver(layer, 0.0, surface))
        assertEquals(layer, compositeOver(layer, 1.0, surface))
    }

    /** Confirms the packed form and the hexadecimal form of a color agree with the channels. */
    @Test
    fun packedAndHexFormsAgree() {
        val color = Srgb8(0x4E, 0x5E, 0x8B)
        assertEquals(0xFF4E5E8B.toInt(), color.toArgb())
        assertEquals("4E5E8B", color.toHex())
        assertEquals(color, Srgb8.fromRgbInt(0xFF4E5E8B.toInt()))
    }

    /** Confirms the OKLab distance is zero for equal colors and about 1 for adjacent gray steps near mid gray. */
    @Test
    fun deltaEIsZeroForEqualColorsAndSmallForNeighbours() {
        assertEquals(0.0, deltaEOk(Srgb8(10, 20, 30), Srgb8(10, 20, 30)), 0.0)
        assertTrue(deltaEOk(Srgb8(128, 128, 128), Srgb8(129, 129, 129)) < 1.0)
    }

    private companion object {
        /** Tolerance for floating-point comparisons. */
        const val TOLERANCE: Double = 1e-6
    }
}
