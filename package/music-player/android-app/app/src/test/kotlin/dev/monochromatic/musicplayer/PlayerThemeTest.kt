// What:     `package dev.monochromatic.musicplayer` places this test beside the theme it covers.
// Why:      The test reaches the module-internal builder without exposing it publicly.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer

import androidx.compose.material3.ColorScheme
import androidx.compose.ui.graphics.Color
import dev.monochromatic.musicplayer.core.RoleSet
import dev.monochromatic.musicplayer.core.Srgb8
import dev.monochromatic.musicplayer.core.ThemePalette
import dev.monochromatic.musicplayer.core.deriveRoles
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies that the Material color scheme is built only from derived roles, with the dark canvas pure black. */
class PlayerThemeTest {
    /** Roots covering a vivid hue, the baseline and a gray. */
    private val roots: List<Srgb8?> = listOf(
        null, Srgb8.fromRgbInt(0xFF0000), Srgb8.fromRgbInt(0x6060C0), Srgb8.fromRgbInt(0x808080),
        Srgb8.fromRgbInt(0x00B050),
    )

    /** Every slot of a scheme as a list, so a test can check them all without naming each one twice. */
    private fun slotsOf(scheme: ColorScheme): List<Color> = listOf(
        scheme.primary, scheme.onPrimary, scheme.primaryContainer, scheme.onPrimaryContainer, scheme.inversePrimary,
        scheme.secondary, scheme.onSecondary, scheme.secondaryContainer, scheme.onSecondaryContainer,
        scheme.tertiary, scheme.onTertiary, scheme.tertiaryContainer, scheme.onTertiaryContainer,
        scheme.background, scheme.onBackground, scheme.surface, scheme.onSurface, scheme.surfaceVariant,
        scheme.onSurfaceVariant, scheme.surfaceTint, scheme.inverseSurface, scheme.inverseOnSurface,
        scheme.error, scheme.onError, scheme.errorContainer, scheme.onErrorContainer, scheme.outline,
        scheme.outlineVariant, scheme.scrim, scheme.surfaceBright, scheme.surfaceDim, scheme.surfaceContainer,
        scheme.surfaceContainerHigh, scheme.surfaceContainerHighest, scheme.surfaceContainerLow,
        scheme.surfaceContainerLowest, scheme.primaryFixed, scheme.primaryFixedDim, scheme.onPrimaryFixed,
        scheme.onPrimaryFixedVariant, scheme.secondaryFixed, scheme.secondaryFixedDim, scheme.onSecondaryFixed,
        scheme.onSecondaryFixedVariant, scheme.tertiaryFixed, scheme.tertiaryFixedDim, scheme.onTertiaryFixed,
        scheme.onTertiaryFixedVariant,
    )

    /** Confirms all 48 slots are set from derived roles or the fixed semantic constants, never a library default. */
    @Test
    fun everySlotComesFromADerivedRole() {
        for (root in roots) {
            val palette: ThemePalette = deriveRoles(root)
            val allowed: Set<Color> = buildSet {
                for (set in listOf(palette.dark, palette.light)) {
                    set.toNamedList().forEach { pair -> add(pair.second.toComposeColor()) }
                }
                add(Color.Black)
                add(Color(0xFF690005))
                add(Color(0xFFFFFFFF))
            }
            for (dark in listOf(true, false)) {
                val slots: List<Color> = slotsOf(playerColorScheme(palette, dark))
                assertEquals(48, slots.size)
                for (slot in slots) {
                    assertTrue("root ${root?.toHex()} dark=$dark slot $slot is no derived role", slot in allowed)
                }
            }
        }
    }

    /** Confirms the dark scheme paints background and surface pure black (D41) and the light one pure white. */
    @Test
    fun canvasSlotsArePureBlackInDarkAndWhiteInLight() {
        for (root in roots) {
            val palette: ThemePalette = deriveRoles(root)
            val dark: ColorScheme = playerColorScheme(palette, dark = true)
            val light: ColorScheme = playerColorScheme(palette, dark = false)
            for (slot in listOf(dark.background, dark.surface, dark.surfaceContainerLowest)) {
                assertEquals(Color(0xFF000000), slot)
            }
            for (slot in listOf(light.background, light.surface, light.surfaceContainerLowest)) {
                assertEquals(Color(0xFFFFFFFF), slot)
            }
        }
    }

    /** Confirms the slots the screens read hold the derived values of the scheme in use. */
    @Test
    fun theReadSlotsFollowTheRoles() {
        val palette: ThemePalette = deriveRoles(Srgb8.fromRgbInt(0x6060C0))
        for (dark in listOf(true, false)) {
            val set: RoleSet = if (dark) palette.dark else palette.light
            val scheme: ColorScheme = playerColorScheme(palette, dark)
            assertEquals(set.primary.toComposeColor(), scheme.primary)
            assertEquals(set.onSurface.toComposeColor(), scheme.onSurface)
            assertEquals(set.onSurfaceVariant.toComposeColor(), scheme.onSurfaceVariant)
            assertEquals(set.secondaryContainer.toComposeColor(), scheme.secondaryContainer)
            assertEquals(set.onSecondaryContainer.toComposeColor(), scheme.onSecondaryContainer)
            assertEquals(set.outline.toComposeColor(), scheme.outline)
            assertEquals(set.outlineVariant.toComposeColor(), scheme.outlineVariant)
            assertEquals(set.currentRow.toComposeColor(), scheme.surfaceContainerLow)
            assertEquals(set.surfaceHigh.toComposeColor(), scheme.surfaceContainerHigh)
        }
    }

    /** Confirms the chosen-row icon circle pair (secondary on the selected fill) holds 3:1 for a UI part. */
    @Test
    fun theSecondaryPairHoldsContrastOnTheSelectedFill() {
        for (root in roots) {
            val palette: ThemePalette = deriveRoles(root)
            for (set in listOf(palette.dark, palette.light)) {
                assertTrue(
                    dev.monochromatic.musicplayer.core.contrastRatio(set.primary, set.secondaryContainer) >= 4.5,
                )
            }
        }
    }

    /** D119: the context menu is filled with the canvas color, pure white in light and pure black in dark. */
    @Test
    fun theMenuSitsOnThePureCanvasColor() {
        /** Palette derived from the baseline root. */
        val palette = deriveRoles(null)
        /** Light scheme of that palette. */
        val light = playerColorScheme(palette, dark = false)
        /** Dark scheme of that palette. */
        val dark = playerColorScheme(palette, dark = true)
        assertEquals(Color.White, trackMenuContainerColor(light))
        assertEquals(Color.Black, trackMenuContainerColor(dark))
    }
}
