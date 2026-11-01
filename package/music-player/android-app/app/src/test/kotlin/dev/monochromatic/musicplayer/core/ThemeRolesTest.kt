// What:     `package dev.monochromatic.musicplayer.core` places this test beside the role derivation.
// Why:      The test reaches the internal pure functions in the same package as the code under test.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

import kotlin.math.abs
import kotlin.math.sign
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Ports the Rust `theme_roles_tests.rs` guarantees: AA for any root, fixed roles, gray roots, the cap, layers. */
class ThemeRolesTest {
    /**
     * Roots that look gray but have unequal channels (D118: they are chromatic and must still tint every gray), plus
     * every fifth gray level with each channel raised by one step in turn and the equal-channel gray itself.
     */
    private fun nearGrayRoots(): List<Srgb8> {
        val roots = mutableListOf<Srgb8>()
        for (value in listOf(0x808088, 0x7F8080, 0x8A8A8B, 0x8A8B8A, 0x808081, 0xFEFFFF, 0x010100, 0x7A7F9A)) {
            roots += Srgb8.fromRgbInt(value)
        }
        for (level in 0..CHANNEL_TOP step LEVEL_STEP) {
            val raised: Int = minOf(level + 1, CHANNEL_TOP)
            roots += Srgb8(level, level, level)
            roots += Srgb8(raised, level, level)
            roots += Srgb8(level, raised, level)
            roots += Srgb8(level, level, raised)
        }
        return roots
    }

    /** True when the three 8-bit channels of the root are equal. */
    private fun isCompletelyNeutral(root: Srgb8): Boolean = root.red == root.green && root.green == root.blue

    /** The spread between the largest and the smallest channel, 0 for a true gray. */
    private fun channelSpread(color: Srgb8): Int =
        maxOf(color.red, color.green, color.blue) - minOf(color.red, color.green, color.blue)

    /** Names of the gray roles of both schemes (D118); the canvases are pure black or white and are not grays. */
    private fun grayRoleNames(): List<String> = listOf(
        "deck", "currentRow", "surfaceHigh", "onSurface", "onSurfaceVariant", "outline", "outlineVariant",
    )

    /** Names of the accent roles of one scheme; the light on-primary is pure white and cannot carry chroma. */
    private fun accentRoleNames(scheme: ThemeScheme): List<String> {
        val names: List<String> = listOf(
            "primary", "secondaryContainer", "secondaryContainerHovered", "secondaryContainerPressed",
            "onSecondaryContainer", "matchFill",
        )
        return if (scheme == ThemeScheme.DARK) names + "onPrimary" else names
    }

    /** The gray roles of one scheme with their finished colors. */
    private fun grayColors(set: RoleSet): List<Pair<String, Srgb8>> {
        val names: List<String> = grayRoleNames()
        return set.toNamedList().filter { pair -> pair.first in names }
    }

    /** The accent roles of one scheme with their finished colors. */
    private fun accentColors(set: RoleSet, scheme: ThemeScheme): List<Pair<String, Srgb8>> {
        val names: List<String> = accentRoleNames(scheme)
        return set.toNamedList().filter { pair -> pair.first in names }
    }

    /** Names the gray roles whose measured chroma is at or below the floor, with the chroma. */
    private fun grayRolesBelowFloor(set: RoleSet): List<String> =
        grayColors(set).mapNotNull { (name, color) ->
            val chroma: Double = srgbToOklch(color).chroma
            if (chroma > GRAY_CHROMA_FLOOR && channelSpread(color) >= 2) null else "$name ${color.toHex()} ($chroma)"
        }

    /** Every test root: hues every 10 degrees by lightness and chroma spreads, plus grays and the extremes. */
    private fun sweptRoots(): List<Srgb8> {
        val roots = mutableListOf<Srgb8>()
        for (hue in 0 until FULL_TURN step HUE_STEP) {
            for (lightness in listOf(0.3, 0.55, 0.8, 0.95)) {
                for (chroma in listOf(0.0, 0.01, 0.03, 0.06, 0.1, 0.15, 0.2, 0.3)) {
                    roots += oklchToSrgb8(Oklch(lightness, chroma, hue.toDouble()))
                }
            }
        }
        val extremes: List<Int> =
            listOf(0xFF0000, 0xFFD000, 0xFFFF00, 0x00FF00, 0x0000FF, 0x6060C0, 0, 0xFFFFFF, 0x808080, 0x7F7F80)
        for (value in extremes) {
            roots += Srgb8.fromRgbInt(value)
        }
        roots += nearGrayRoots()
        return roots
    }

    /** One drawn foreground on one drawn background, with the ratio it must reach. */
    private data class TextState(
        /** Name of the pair for the failure message. */
        val label: String,
        /** Text or outline color. */
        val foreground: Srgb8,
        /** Color it is drawn on, after any state layer. */
        val background: Srgb8,
        /** Ratio the pair must reach. */
        val minimum: Double,
    )

    /** Text colors drawn on the four surfaces, with their names. */
    private fun surfaceTexts(set: RoleSet): List<Pair<String, Srgb8>> =
        listOf("on-surface" to set.onSurface, "on-surface-variant" to set.onSurfaceVariant, "primary" to set.primary)

    /** The four surfaces text is drawn on, with their names. */
    private fun namedSurfaces(set: RoleSet): List<Pair<String, Srgb8>> =
        listOf(
            "canvas" to set.canvas,
            "deck" to set.deck,
            "current-row" to set.currentRow,
            "surface-high" to set.surfaceHigh,
        )

    /** Text pairs on the surfaces at rest and under the on-surface hover and pressed (also focus) layers. */
    private fun surfaceStates(set: RoleSet): List<TextState> {
        val states = mutableListOf<TextState>()
        for ((surfaceName, surface) in namedSurfaces(set)) {
            val hovered: Srgb8 = compositeOver(set.onSurface, HOVER_ALPHA, surface)
            val pressed: Srgb8 = compositeOver(set.onSurface, PRESSED_ALPHA, surface)
            for ((textName, text) in surfaceTexts(set)) {
                states += TextState("$textName on the $surfaceName", text, surface, AA_TEXT)
                states += TextState("$textName on the $surfaceName, hovered", text, hovered, AA_TEXT)
                states += TextState("$textName on the $surfaceName, pressed", text, pressed, AA_TEXT)
            }
        }
        return states
    }

    /** Pairs on the selected fill, on primary and on the match fill, and the outline and error pairs. */
    private fun fillStates(set: RoleSet): List<TextState> {
        val states = mutableListOf<TextState>()
        val fill: Srgb8 = set.secondaryContainer
        val hoverLayer: Srgb8 = compositeOver(set.onSecondaryContainer, HOVER_ALPHA, fill)
        val pressedLayer: Srgb8 = compositeOver(set.onSecondaryContainer, PRESSED_ALPHA, fill)
        val contentFills: List<Pair<String, Srgb8>> = listOf(
            "selected fill" to fill,
            "selected fill, hovered (OKLCH)" to set.secondaryContainerHovered,
            "selected fill, pressed (OKLCH)" to set.secondaryContainerPressed,
            "selected fill, hover layer" to hoverLayer,
            "selected fill, pressed layer" to pressedLayer,
        )
        for ((name, background) in contentFills) {
            states += TextState("on-secondary-container on the $name", set.onSecondaryContainer, background, AA_TEXT)
        }
        // Primary is drawn only on the selected fill and the layers over it (the rail letter), not on the OKLCH fills.
        for ((name, background) in listOf(contentFills[0], contentFills[3], contentFills[4])) {
            states += TextState("primary on the $name", set.primary, background, AA_TEXT)
        }
        states += TextState("on-primary on primary", set.onPrimary, set.primary, AA_TEXT)
        for ((name, alpha) in listOf("hovered" to HOVER_ALPHA, "pressed" to PRESSED_ALPHA)) {
            val layered: Srgb8 = compositeOver(set.onPrimary, alpha, set.primary)
            states += TextState("on-primary on primary, $name", set.onPrimary, layered, AA_TEXT)
        }
        states += TextState("on-surface on the match fill", set.onSurface, set.matchFill, AA_TEXT)
        states += TextState("on-surface-variant on the match fill", set.onSurfaceVariant, set.matchFill, AA_TEXT)
        states += TextState("outline on the canvas", set.outline, set.canvas, AA_NON_TEXT)
        states += TextState("outline on the deck", set.outline, set.deck, AA_NON_TEXT)
        states += TextState("on-error-container on the container", set.onErrorContainer, set.errorContainer, AA_TEXT)
        states += TextState("error on the error container", set.error, set.errorContainer, AA_NON_TEXT)
        states += TextState("error text on the canvas", set.error, set.canvas, AA_TEXT)
        return states
    }

    /** The whole contrast guarantee of one scheme: every pair the window draws, with its state layers. */
    private fun textStates(set: RoleSet): List<TextState> = surfaceStates(set) + fillStates(set)

    /** Names every pair of [textStates] below its minimum, with the measured ratio. */
    private fun failingStates(set: RoleSet): List<String> = textStates(set).mapNotNull { state ->
        val ratio: Double = contrastRatio(state.foreground, state.background)
        if (ratio < state.minimum) "${state.label} ($ratio below ${state.minimum})" else null
    }

    /** Asserts one contrast pair with a message naming it. */
    private fun assertRatio(label: String, foreground: Srgb8, background: Srgb8, minimum: Double) {
        val ratio: Double = contrastRatio(foreground, background)
        assertTrue("$label: ${foreground.toHex()} on ${background.toHex()} is $ratio, below $minimum", ratio >= minimum)
    }

    /** Asserts that no pair of one scheme falls below its minimum. */
    private fun checkSchemeContrast(label: String, set: RoleSet) {
        val failing: List<String> = failingStates(set)
        assertTrue("$label: $failing", failing.isEmpty())
    }

    /** The light scheme as the player held it before the B4 retune: primary and on-surface-variant of the captures. */
    private fun oldLightRoles(): RoleSet = deriveRoles(null).light.copy(
        primary = Srgb8.fromRgbInt(0x4E5E8B),
        onSurfaceVariant = Srgb8.fromRgbInt(0x5C5E6A),
        matchFill = Srgb8.fromRgbInt(0xCFD9F0),
        secondaryContainerHovered = Srgb8.fromRgbInt(0xCBD0E7),
        secondaryContainerPressed = Srgb8.fromRgbInt(0xC2C7DE),
    )

    /** Confirms the baseline passes every drawn state in both schemes. */
    @Test
    fun theBaselinePassesEveryDrawnStateInBothSchemes() {
        val palette: ThemePalette = deriveRoles(null)
        checkSchemeContrast("baseline dark", palette.dark)
        checkSchemeContrast("baseline light", palette.light)
    }

    /** Positive control: the list reports exactly the states the old light baseline fell below AA in. */
    @Test
    fun theContrastListReportsTheStatesTheOldLightBaselineFailed() {
        val failing: List<String> = failingStates(oldLightRoles())
        println("old light baseline falls below AA in: $failing")
        for (expected in listOf(
            "on-surface-variant on the surface-high, pressed",
            "primary on the surface-high, pressed",
            "primary on the selected fill, hover layer",
            "primary on the selected fill, pressed layer",
        )) {
            assertTrue("the list missed $expected: $failing", failing.any { entry -> entry.startsWith(expected) })
        }
        assertEquals("unexpected failing states: $failing", 4, failing.size)
    }

    /** Positive control: a lightened secondary text color on the new baseline is reported. */
    @Test
    fun theContrastListReportsALightenedSecondaryTextColor() {
        val lightened: RoleSet = deriveRoles(null).light.copy(onSurfaceVariant = Srgb8.fromRgbInt(0x80828D))
        assertTrue(failingStates(lightened).isNotEmpty())
    }

    /** Confirms AA text contrast on every surface and state layer for more than a thousand roots in both schemes. */
    @Test
    fun everySweptRootKeepsAaContrastInBothSchemes() {
        val roots: List<Srgb8> = sweptRoots()
        assertTrue("the sweep is too small: ${roots.size}", roots.size > MINIMUM_SWEEP)
        for (root in roots) {
            val palette: ThemePalette = deriveRoles(root)
            checkSchemeContrast("dark root ${root.toHex()}", palette.dark)
            checkSchemeContrast("light root ${root.toHex()}", palette.light)
        }
    }

    /** Confirms the Settings page pairs hold AA: Settings is drawn on the canvas alone (D109). */
    @Test
    fun settingsRowPairsHoldAaOnTheCanvasUnderEveryRoot() {
        for (root in sweptRoots()) {
            val palette: ThemePalette = deriveRoles(root)
            for (set in listOf(palette.dark, palette.light)) {
                val hovered: Srgb8 = compositeOver(set.onSurface, HOVER_ALPHA, set.canvas)
                val pressed: Srgb8 = compositeOver(set.onSurface, PRESSED_ALPHA, set.canvas)
                assertRatio("title ${root.toHex()}", set.onSurface, set.canvas, AA_TEXT)
                assertRatio("title pressed ${root.toHex()}", set.onSurface, pressed, AA_TEXT)
                assertRatio("line ${root.toHex()}", set.onSurfaceVariant, set.canvas, AA_TEXT)
                assertRatio("line hovered ${root.toHex()}", set.onSurfaceVariant, hovered, AA_TEXT)
            }
        }
    }

    /**
     * Confirms the pure black and white roles, white on-primary and the error roles are fixed. The dark deck is not
     * among them any more (D118): it is a gray and takes the root's tint.
     */
    @Test
    fun fixedRolesDoNotFollowTheAccent() {
        for (root in sweptRoots()) {
            val palette: ThemePalette = deriveRoles(root)
            assertEquals(Srgb8(0x00, 0x00, 0x00), palette.dark.canvas)
            assertEquals(Srgb8(0xFF, 0xFF, 0xFF), palette.light.canvas)
            assertEquals(Srgb8(0xFF, 0xFF, 0xFF), palette.light.onPrimary)
            assertEquals(Srgb8(0xFF, 0xB4, 0xAB), palette.dark.error)
            assertEquals(Srgb8(0xF9, 0xDE, 0xDC), palette.light.errorContainer)
        }
    }

    /** Confirms the baseline root reproduces the values the player held before D108, within a just-noticeable step. */
    @Test
    fun theBaselineStaysCloseToTheOldFixedValues() {
        val palette: ThemePalette = deriveRoles(null)
        val old: List<Triple<String, Int, Int>> = listOf(
            Triple("canvas", 0x000000, 0xFFFFFF), Triple("deck", 0x0A0A0D, 0xF4F3FA),
            Triple("currentRow", 0x121318, 0xF4F3FA),
            Triple("onSurface", 0xE4E5F0, 0x30323B), Triple("onSurfaceVariant", 0xC5C6D0, 0x5C5E6A),
            Triple("outline", 0x8E9099, 0x757780), Triple("outlineVariant", 0x44464F, 0xC5C6D0),
            Triple("primary", 0xBAC5EE, 0x4E5E8B), Triple("onPrimary", 0x1F2D52, 0xFFFFFF),
            Triple("secondaryContainer", 0x353B4D, 0xDDE2F9), Triple("onSecondaryContainer", 0xDDE2F9, 0x161B2C),
            Triple("surfaceHigh", 0x1E1F25, 0xE8E7EE), Triple("error", 0xFFB4AB, 0xBA1A1A),
            Triple("errorContainer", 0x8C1D18, 0xF9DEDC), Triple("onErrorContainer", 0xF9DEDC, 0x410E0B),
        )
        var largest = 0.0
        for ((name, dark, light) in old) {
            val darkRole: Srgb8 = palette.dark.toNamedList().first { pair -> pair.first == name }.second
            val lightRole: Srgb8 = palette.light.toNamedList().first { pair -> pair.first == name }.second
            val darkDelta: Double = deltaEOk(darkRole, Srgb8.fromRgbInt(dark))
            val lightDelta: Double = deltaEOk(lightRole, Srgb8.fromRgbInt(light))
            val bound: Double? = RETUNED_LIGHT_BOUNDS[name]
            if (bound == null) {
                largest = maxOf(largest, darkDelta, lightDelta)
            } else {
                println("retuned light $name: ${Srgb8.fromRgbInt(light).toHex()} to ${lightRole.toHex()}, $lightDelta")
                assertTrue("$name moved $lightDelta, over its bound $bound", lightDelta in BASELINE_DRIFT_LIMIT..bound)
                largest = maxOf(largest, darkDelta)
            }
        }
        println("largest baseline delta-E vs the old fixed values, retuned roles aside: $largest")
        assertTrue("baseline drifted: $largest", largest < BASELINE_DRIFT_LIMIT)
    }

    /** Confirms the dark deck is a table gray that reproduces the old literal `#0A0A0D` at the baseline exactly. */
    @Test
    fun theDarkDeckReproducesTheOldLiteralAtTheBaseline() {
        assertEquals(Srgb8(0x0A, 0x0A, 0x0D), deriveRoles(null).dark.deck)
    }

    /** Confirms only three equal channels make a root completely neutral, and no weaker root counts. */
    @Test
    fun completelyNeutralMeansThreeEqualChannelsAndNoWeakerRootCounts() {
        var largestGrayChroma = 0.0
        for (level in 0..CHANNEL_TOP) {
            val gray = Srgb8(level, level, level)
            val terms: RootTerms = rootTerms(gray)
            assertTrue("gray $level must be completely neutral", terms.completelyNeutral)
            assertEquals(0.0, terms.scale, 0.0)
            largestGrayChroma = maxOf(largestGrayChroma, srgbToOklch(gray).chroma)
            for (offset in ONE_STEP_OFFSETS) {
                val red: Int = level + offset.first
                val green: Int = level + offset.second
                val blue: Int = level + offset.third
                if (listOf(red, green, blue).any { channel -> channel !in 0..CHANNEL_TOP }) {
                    continue
                }
                val off = Srgb8(red, green, blue)
                val offTerms: RootTerms = rootTerms(off)
                assertTrue("${off.toHex()} has chroma and must not be neutral", !offTerms.completelyNeutral)
                assertTrue("${off.toHex()} must keep a positive scale", offTerms.scale > 0.0)
                assertTrue("${off.toHex()} must measure above 1e-3", srgbToOklch(off).chroma > 1.0e-3)
                assertEquals("${off.toHex()} must keep its own hue", srgbToOklch(off).hue, offTerms.hue, 0.0)
            }
        }
        assertEquals("equal-channel roots read exactly zero chroma", 0.0, largestGrayChroma, 0.0)
    }

    /** Confirms a completely neutral root gives equal channels for every gray role and every non-error role. */
    @Test
    fun aCompletelyNeutralRootGivesEqualChannelsForEveryGrayRole() {
        for (level in 0..CHANNEL_TOP) {
            val palette: ThemePalette = deriveRoles(Srgb8(level, level, level))
            for ((scheme, set) in listOf(ThemeScheme.DARK to palette.dark, ThemeScheme.LIGHT to palette.light)) {
                for ((name, color) in grayColors(set)) {
                    assertEquals("$scheme $name ${color.toHex()} under gray $level", 0, channelSpread(color))
                }
                for ((name, color) in set.toNamedList()) {
                    if (name.startsWith("error") || name == "onErrorContainer") {
                        continue
                    }
                    assertEquals("$scheme $name ${color.toHex()} tinted under gray $level", 0, channelSpread(color))
                }
            }
        }
    }

    /** Confirms every gray role of every weakly chromatic root keeps chroma above the floor and two 8-bit steps. */
    @Test
    fun everyGrayRoleOfAWeaklyChromaticRootKeepsAVisibleTint() {
        var checked = 0
        for (root in sweptRoots()) {
            if (isCompletelyNeutral(root) || srgbToOklch(root).chroma >= WEAK_ROOT_CHROMA) {
                continue
            }
            val palette: ThemePalette = deriveRoles(root)
            for ((scheme, set) in listOf(ThemeScheme.DARK to palette.dark, ThemeScheme.LIGHT to palette.light)) {
                assertEquals("$scheme under ${root.toHex()}", emptyList<String>(), grayRolesBelowFloor(set))
                checked += grayRoleNames().size
            }
        }
        assertTrue("the tint check ran on only $checked roles", checked > MINIMUM_TINT_CHECKS)
    }

    /** Positive control: the floor check does report the true grays that a completely neutral root gives. */
    @Test
    fun theFloorCheckReportsTrueGraysAPositiveControl() {
        val palette: ThemePalette = deriveRoles(Srgb8(0x80, 0x80, 0x80))
        assertEquals(grayRoleNames().size, grayRolesBelowFloor(palette.dark).size)
        assertEquals(grayRoleNames().size, grayRolesBelowFloor(palette.light).size)
    }

    /** Confirms a nearly gray root gives the gray roles its own hue, not the baseline hue. */
    @Test
    fun aWeaklyChromaticRootGivesTheGrayRolesTheRootHueNotTheBaselineHue() {
        val weak = Srgb8(0x7F, 0x80, 0x80)
        val rootHue: Double = srgbToOklch(weak).hue
        val baselineHue: Double = srgbToOklch(BASELINE_ROOT).hue
        assertTrue("the control root must be far from the baseline hue", abs(rootHue - baselineHue) > 60.0)
        assertEquals(rootHue, rootTerms(weak).hue, 0.0)
        val palette: ThemePalette = deriveRoles(weak)
        for ((scheme, set) in listOf(ThemeScheme.DARK to palette.dark, ThemeScheme.LIGHT to palette.light)) {
            for (name in listOf("onSurface", "onSurfaceVariant", "outline", "outlineVariant")) {
                val color: Oklch = srgbToOklch(set.toNamedList().first { pair -> pair.first == name }.second)
                val fromRoot: Double = wrapHue(color.hue - rootHue + 540.0) - 180.0
                val fromBaseline: Double = wrapHue(color.hue - baselineHue + 540.0) - 180.0
                assertTrue("$scheme $name hue ${color.hue} vs root hue $rootHue", abs(fromRoot) < ROLE_HUE_SPREAD)
                assertTrue("$scheme $name hue ${color.hue} near the baseline", abs(fromBaseline) > ROLE_HUE_SPREAD)
            }
        }
    }

    /** Confirms a weak root's accent roles are never grayer than the grays, measured from the final 8-bit colors. */
    @Test
    fun anAccentRoleOfAWeaklyChromaticRootIsNeverGrayerThanTheGrays() {
        var checked = 0
        for (root in sweptRoots()) {
            if (isCompletelyNeutral(root) || srgbToOklch(root).chroma >= WEAK_ROOT_CHROMA) {
                continue
            }
            val palette: ThemePalette = deriveRoles(root)
            for ((scheme, set) in listOf(ThemeScheme.DARK to palette.dark, ThemeScheme.LIGHT to palette.light)) {
                for ((name, color) in accentColors(set, scheme)) {
                    val chroma: Double = srgbToOklch(color).chroma
                    assertTrue(
                        "$scheme $name ${color.toHex()} has chroma $chroma under ${root.toHex()}",
                        chroma > ACCENT_CHROMA_FLOOR,
                    )
                    checked += 1
                }
            }
        }
        assertTrue("the accent check ran on only $checked roles", checked > MINIMUM_TINT_CHECKS)
    }

    /** Confirms a completely neutral root keeps every accent role at chroma zero, so each is a true gray. */
    @Test
    fun aCompletelyNeutralRootKeepsEveryAccentRoleAtChromaZero() {
        for (level in listOf(0x00, 0x40, 0x80, 0xC0, 0xFF)) {
            val palette: ThemePalette = deriveRoles(Srgb8(level, level, level))
            for ((scheme, set) in listOf(ThemeScheme.DARK to palette.dark, ThemeScheme.LIGHT to palette.light)) {
                for ((name, color) in accentColors(set, scheme)) {
                    assertEquals("$scheme $name ${color.toHex()} under gray $level", 0, channelSpread(color))
                }
            }
        }
    }

    /** Confirms the weak root gives the accent the gray floor and the vivid root keeps its scaled chroma. */
    @Test
    fun theAccentFloorRaisesAWeakRootsAccentButNotAVividRootsOrTheBaselines() {
        assertEquals(Srgb8(0xBA, 0xC5, 0xEE), deriveRoles(null).dark.primary)
        assertEquals(Srgb8(0xFF, 0xFF, 0xFF), deriveRoles(null).light.onPrimary)
        val weak: Double = srgbToOklch(deriveRoles(Srgb8(0x7F, 0x80, 0x80)).dark.primary).chroma
        val vivid: Double = srgbToOklch(deriveRoles(Srgb8(0xFF, 0x00, 0x00)).dark.primary).chroma
        assertTrue("weak primary chroma $weak", weak > ACCENT_CHROMA_FLOOR)
        assertTrue("vivid primary chroma $vivid is not above the baseline's 0.0589", vivid > 0.0589)
    }

    /** Confirms accent roles still scale with a weak root's chroma while the gray roles keep their own tint. */
    @Test
    fun accentRolesStillScaleWithTheRootChromaWhileGraysDoNot() {
        val weak: ThemePalette = deriveRoles(Srgb8(0x80, 0x80, 0x88))
        val baseline: ThemePalette = deriveRoles(null)
        val weakPrimary: Double = srgbToOklch(weak.dark.primary).chroma
        val baselinePrimary: Double = srgbToOklch(baseline.dark.primary).chroma
        assertTrue("primary $weakPrimary vs $baselinePrimary", weakPrimary < baselinePrimary * ACCENT_SHRINK_LIMIT)
        assertTrue(rootTerms(Srgb8(0x80, 0x80, 0x88)).scale in 0.0..ACCENT_SHRINK_LIMIT)
        assertEquals(1.0, rootTerms(null).scale, 0.0)
        assertTrue(srgbToOklch(weak.dark.onSurface).chroma > GRAY_CHROMA_FLOOR)
    }

    /** Confirms a more vivid accent gives a more vivid primary up to the cap, and that past the cap nothing changes. */
    @Test
    fun aMoreVividAccentGivesAMoreVividPrimaryUpToTheCap() {
        val muted: Double = srgbToOklch(deriveRoles(Srgb8.fromRgbInt(0x7A7F9A)).dark.primary).chroma
        val vivid: Double = srgbToOklch(deriveRoles(Srgb8.fromRgbInt(0x6060C0)).dark.primary).chroma
        assertTrue("$vivid vs $muted", vivid > muted)
        val capScale: Double = ROOT_CHROMA_CAP / srgbToOklch(BASELINE_ROOT).chroma
        assertEquals(capScale, rootTerms(Srgb8.fromRgbInt(0x4000FF)).scale, 1e-12)
        assertEquals(capScale, rootTerms(Srgb8.fromRgbInt(0xFF0000)).scale, 1e-12)
    }

    /** Confirms a vivid root does not tint the neutral surfaces beyond the baseline. */
    @Test
    fun aVividRootDoesNotTintTheNeutralSurfacesBeyondTheBaseline() {
        val baseline: Double = srgbToOklch(deriveRoles(null).dark.surfaceHigh).chroma
        val vivid: Double = srgbToOklch(deriveRoles(Srgb8.fromRgbInt(0xFF0000)).dark.surfaceHigh).chroma
        assertTrue("$vivid vs $baseline", abs(vivid - baseline) < 0.004)
    }

    /** Confirms the selected-fill layers step toward the content color, hover first and pressed further. */
    @Test
    fun theSelectedFillLayersStepTowardTheContentColor() {
        for (root in listOf(null, Srgb8.fromRgbInt(0x6060C0))) {
            val palette: ThemePalette = deriveRoles(root)
            for (set in listOf(palette.dark, palette.light)) {
                val base: Oklch = srgbToOklch(set.secondaryContainer)
                val content: Oklch = srgbToOklch(set.onSecondaryContainer)
                val hovered: Oklch = srgbToOklch(set.secondaryContainerHovered)
                val pressed: Oklch = srgbToOklch(set.secondaryContainerPressed)
                val toward: Double = sign(content.lightness - base.lightness)
                assertTrue((hovered.lightness - base.lightness) * toward > 0.0)
                assertTrue((pressed.lightness - hovered.lightness) * toward > 0.0)
            }
        }
    }

    /** Confirms the match fill is darker than primary in dark and lighter in light (D59). */
    @Test
    fun theMatchFillIsDarkerInDarkAndLighterInLight() {
        val palette: ThemePalette = deriveRoles(Srgb8.fromRgbInt(0x6060C0))
        assertTrue(srgbToOklch(palette.dark.matchFill).lightness < srgbToOklch(palette.dark.primary).lightness)
        assertTrue(srgbToOklch(palette.light.matchFill).lightness > srgbToOklch(palette.light.primary).lightness)
    }

    /** Confirms accent roles keep root hue plus their offset instead of clipping, wherever they keep enough chroma. */
    @Test
    fun accentRolesKeepTheRootHueInsteadOfClipping() {
        var checked = 0
        for (hue in 0 until FULL_TURN step HUE_STEP) {
            val root: Srgb8 = oklchToSrgb8(Oklch(0.6, 0.15, hue.toDouble()))
            val terms: RootTerms = rootTerms(root)
            val palette: ThemePalette = deriveRoles(root)
            for ((set, offset) in listOf(palette.dark to DARK_PRIMARY_OFFSET, palette.light to LIGHT_PRIMARY_OFFSET)) {
                val lch: Oklch = srgbToOklch(set.primary)
                if (lch.chroma < MIN_CHECKED_CHROMA) {
                    continue
                }
                val difference: Double = wrapHue(lch.hue - (terms.hue + offset) + 540.0) - 180.0
                assertTrue("primary hue ${lch.hue} for root hue $hue", abs(difference) < HUE_TOLERANCE)
                checked += 1
            }
        }
        assertTrue("the hue check ran on only $checked roles", checked > MINIMUM_HUE_CHECKS)
    }

    /** Positive control: a start that fails 4.6 on white comes back darker and passing, so the fit can act. */
    @Test
    fun theFitMovesLightnessWhenATargetFails() {
        val white = Srgb8(255, 255, 255)
        val start = Oklch(0.85, 0.05, 250.0)
        val fitted: Oklch = fitLightness(start, 0.0) { candidate -> contrastRatio(candidate, white) >= 4.6 }
        assertTrue(fitted.lightness < start.lightness)
        assertEquals(start.hue, fitted.hue, 0.0)
        assertTrue(contrastRatio(oklchToSrgb8(fitted), white) >= 4.6)
    }

    /** Confirms the fit returns the endpoint when no lightness can pass. */
    @Test
    fun theFitReturnsTheEndpointWhenNothingCanPass() {
        val fitted: Oklch = fitLightness(Oklch(0.5, 0.0, 0.0), 0.0) { false }
        assertEquals(0.0, fitted.lightness, 0.0)
    }

    private companion object {
        /** AA contrast for text. */
        const val AA_TEXT: Double = 4.5

        /** AA contrast for UI parts. */
        const val AA_NON_TEXT: Double = 3.0

        /** Degrees in a turn. */
        const val FULL_TURN: Int = 360

        /** Degrees between swept hues. */
        const val HUE_STEP: Int = 10

        /** Fewest roots the sweep must contain. */
        const val MINIMUM_SWEEP: Int = 1000

        /** Largest OKLab distance (times 100) the baseline may drift from the pre-D108 values. */
        const val BASELINE_DRIFT_LIMIT: Double = 0.5

        /** Light roles retuned for B4 (lightness only) with the largest OKLab distance each may have moved. */
        val RETUNED_LIGHT_BOUNDS: Map<String, Double> = mapOf("primary" to 3.6, "onSurfaceVariant" to 1.6)

        /** Primary hue offsets of the dark and light tables. */
        const val DARK_PRIMARY_OFFSET: Double = 4.7
        const val LIGHT_PRIMARY_OFFSET: Double = 0.0

        /** Chroma below which 8-bit rounding limits hue accuracy and the hue check is skipped. */
        const val MIN_CHECKED_CHROMA: Double = 0.03

        /** Accepted hue difference in degrees. */
        const val HUE_TOLERANCE: Double = 4.0

        /** Fewest hue checks that must run. */
        const val MINIMUM_HUE_CHECKS: Int = 20

        /** Largest 8-bit channel value. */
        const val CHANNEL_TOP: Int = 255

        /** The six one-step moves of one channel away from a gray, as red, green and blue offsets. */
        val ONE_STEP_OFFSETS: List<Triple<Int, Int, Int>> = listOf(
            Triple(1, 0, 0), Triple(0, 1, 0), Triple(0, 0, 1), Triple(-1, 0, 0), Triple(0, -1, 0), Triple(0, 0, -1),
        )

        /** Distance between the gray levels of the near-gray roots. */
        const val LEVEL_STEP: Int = 5

        /** Root chroma below which a root counts as weak in the tint checks. */
        const val WEAK_ROOT_CHROMA: Double = 0.02

        /**
         * Smallest chroma, measured in OKLCH from the final 8-bit colors, that every gray role keeps under a weakly
         * chromatic root. The lowest value measured over roots stepping one degree of hue is 0.00455 (the dark deck).
         */
        const val GRAY_CHROMA_FLOOR: Double = 0.004

        /**
         * Smallest chroma, measured from the final 8-bit colors, that every accent role keeps under a weakly chromatic
         * root. The lowest value measured over the weak roots is 0.01313 (the dark pressed selected fill).
         */
        const val ACCENT_CHROMA_FLOOR: Double = 0.012

        /** Fewest gray-role checks that must run. */
        const val MINIMUM_TINT_CHECKS: Int = 1000

        /** Largest distance in degrees between a gray role's hue and the root hue (role offsets reach 17 degrees). */
        const val ROLE_HUE_SPREAD: Double = 25.0

        /** Share of the baseline primary chroma a weak root's primary must stay under. */
        const val ACCENT_SHRINK_LIMIT: Double = 0.3
    }
}
