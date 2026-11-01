// What:     `package dev.monochromatic.musicplayer.core` places this test beside the theme roles it measures.
// Why:      The test reads the derived roles and the contrast rule, which are internal to the core package.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each ring contrast must reach the 3 to 1 minimum for user interface parts.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `private const val RING_MINIMUM_RATIO: Double = 3.0` is the minimum contrast of a user interface part.
// Why:      The review asked every focus indicator for 3 to 1 or better against the surface it sits on.
//
// In TS you'd write (pseudocode):
// ```ts
// const RING_MINIMUM_RATIO = 3;
// ```
/** Least contrast of the focus ring against any surface it can sit on. */
private const val RING_MINIMUM_RATIO: Double = 3.0

// What:     `class FocusRingContrastTest` measures the ring color against every surface in both appearances.
// Why:      The ring is drawn in the primary role, so the derived primary must stay 3 to 1 away from the canvas,
//           the deck, the playing row, the menu surface and the selected fill, for the baseline root and for a sweep
//           of accent roots. Focus adds no fill (D117), so the surfaces are measured as they are.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("focus ring contrast", () => { /* cases */ });
// ```
/** Checks that the primary role, which the focus ring uses, reaches 3 to 1 on every surface. */
class FocusRingContrastTest {
    /** Accent roots: the baseline, six hues at medium lightness and chroma, and the grays and extremes. */
    private fun roots(): List<Srgb8?> {
        /** Roots collected for the sweep, starting with the null root that stands for the baseline. */
        val roots = mutableListOf<Srgb8?>(null, BASELINE_ROOT)
        for (hue in 0 until FULL_CIRCLE step HUE_STEP) {
            roots += oklchToSrgb8(Oklch(0.55, 0.15, hue.toDouble()))
            roots += oklchToSrgb8(Oklch(0.80, 0.10, hue.toDouble()))
            roots += oklchToSrgb8(Oklch(0.30, 0.10, hue.toDouble()))
        }
        for (value in listOf(0xFF0000, 0xFFFF00, 0x00FF00, 0x0000FF, 0x000000, 0xFFFFFF, 0x808080)) {
            roots += Srgb8.fromRgbInt(value)
        }
        return roots
    }

    /** Surfaces of one scheme that a focused control or row can sit on, with their names. */
    private fun surfaces(set: RoleSet): List<Pair<String, Srgb8>> = listOf(
        "canvas" to set.canvas,
        "deck" to set.deck,
        "current-row" to set.currentRow,
        "surface-high" to set.surfaceHigh,
        "selected-fill" to set.secondaryContainer,
    )

    /** Checks the near ring on every surface of one scheme, and the outer ring against the near ring. */
    private fun assertRingContrast(label: String, set: RoleSet) {
        for ((name, surface) in surfaces(set)) {
            /** Contrast of the near ring, the primary role, against the surface. */
            val ratio: Double = contrastRatio(set.primary, surface)
            assertTrue("$label near ring on $name: $ratio", ratio >= RING_MINIMUM_RATIO)
        }
        /** Contrast of the outer ring, the on-primary role, against the near ring it surrounds. */
        val outerRatio: Double = contrastRatio(set.onPrimary, set.primary)
        assertTrue("$label outer ring on the near ring: $outerRatio", outerRatio >= RING_MINIMUM_RATIO)
    }

    /** The ring reaches 3 to 1 on every surface for the baseline root, in light and in dark. */
    @Test
    fun ringReachesThreeToOneForTheBaselineRoot() {
        /** Palette derived from the baseline root. */
        val palette: ThemePalette = deriveRoles(null)
        assertRingContrast("baseline light", palette.light)
        assertRingContrast("baseline dark", palette.dark)
    }

    /** The ring reaches 3 to 1 on every surface for every swept accent root, in light and in dark. */
    @Test
    fun ringReachesThreeToOneForSweptAccentRoots() {
        for (root in roots()) {
            /** Palette derived from this root. */
            val palette: ThemePalette = deriveRoles(root)
            assertRingContrast("root ${root?.toHex()} light", palette.light)
            assertRingContrast("root ${root?.toHex()} dark", palette.dark)
        }
    }

    /** Degrees in a full turn of the hue circle. */
    private companion object {
        /** Full turn in degrees. */
        const val FULL_CIRCLE: Int = 360

        /** Hue spacing of the sweep in degrees. */
        const val HUE_STEP: Int = 45
    }
}
