// What:     `package dev.monochromatic.musicplayer.core` places this test beside the pane split rule.
// Why:      The test reaches the internal pure function in the same package as the code under test.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each pane width and clearance is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

// What:     `class SettingsPaneSplitTest` groups the tests of the wide Settings pane split.
// Why:      Both the creased and the crease-free case can be verified on the host JVM, without a device.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("settingsPaneSplit", () => { ... });
// ```
/** Verifies the pane widths and the crease clearance of the wide Settings form. */
class SettingsPaneSplitTest {
    // What:     `creasedWindowSplitsInExactHalves` checks the Fold inner display at the default size.
    // Why:      D110 gives a creased device exactly half of the 2076 px window to each pane.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("creased 2076 px window: 1038 + 1038", () => { ... });
    // ```
    /** Confirms a creased 2076 px window gives 1038 px to each pane. */
    @Test
    fun creasedWindowSplitsInExactHalves() {
        /** Split of the Fold inner display at density 2.4375 (390 dpi). */
        val split: SettingsPaneSplit = settingsPaneSplit(
            creasePresent = true,
            windowWidthPx = 2076,
            creaseWidthPx = 0f,
            density = 2.4375f,
        )
        assertEquals(1038, split.listWidthPx)
        assertEquals(1038, split.editorWidthPx)
    }

    // What:     `creasedOddWindowKeepsAllPixels` checks an odd width.
    // Why:      An odd window cannot split exactly, but the two panes must still cover every pixel.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("odd width: widths sum to the window", () => { ... });
    // ```
    /** Confirms an odd creased window gives the extra pixel to the editor and loses none. */
    @Test
    fun creasedOddWindowKeepsAllPixels() {
        /** Split of a 2075 px window. */
        val split: SettingsPaneSplit = settingsPaneSplit(
            creasePresent = true,
            windowWidthPx = 2075,
            creaseWidthPx = 0f,
            density = 2.4375f,
        )
        assertEquals(1037, split.listWidthPx)
        assertEquals(2075, split.listWidthPx + split.editorWidthPx)
    }

    // What:     `creasedClearanceIsHalfOfTheFloorInDp` checks the extra text inset at density 2.4375.
    // Why:      The 7.5 mm floor is about 110.3 px, so each pane keeps about 55.15 px, or 22.6 dp, clear of the center.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("half floor at 390 dpi", () => { ... });
    // ```
    /** Confirms the list text gains the clearance beyond 32 dp (none) and the editor text beyond 16 dp. */
    @Test
    fun creasedClearanceIsHalfOfTheFloorInDp() {
        /** Split of the Fold inner display at the default size. */
        val split: SettingsPaneSplit = settingsPaneSplit(
            creasePresent = true,
            windowWidthPx = 2076,
            creaseWidthPx = 0f,
            density = 2.4375f,
        )
        /** Half the 7.5 mm floor in dp at this density. */
        val halfDp: Float = CREASE_FLOOR_WIDTH_PX / 2f / 2.4375f
        assertEquals(22.6f, halfDp, 0.05f)
        assertEquals(0f, split.listTextEndExtraDp, 0f)
        assertEquals(halfDp - 16f, split.editorTextStartExtraDp, 0.001f)
    }

    // What:     `creasedClearanceGrowsAtSmallDensity` checks a display size with a low density.
    // Why:      The floor is physical, so a smaller dp density converts it to more dp, and the list needs extra too.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("half floor at density 1", () => { ... });
    // ```
    /** Confirms both panes add clearance when the density is 1. */
    @Test
    fun creasedClearanceGrowsAtSmallDensity() {
        /** Split of the same window at density 1. */
        val split: SettingsPaneSplit = settingsPaneSplit(
            creasePresent = true,
            windowWidthPx = 2076,
            creaseWidthPx = 0f,
            density = 1f,
        )
        /** Half the 7.5 mm floor in dp at density 1. */
        val halfDp: Float = CREASE_FLOOR_WIDTH_PX / 2f
        assertEquals(halfDp - 32f, split.listTextEndExtraDp, 0.001f)
        assertEquals(halfDp - 16f, split.editorTextStartExtraDp, 0.001f)
    }

    // What:     `widerReportedCreaseGovernsTheClearance` checks a crease wider than the floor.
    // Why:      E2 takes the larger of the 7.5 mm floor and the physical crease width.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("wide crease governs", () => { ... });
    // ```
    /** Confirms a 300 px reported crease sets the clearance instead of the 110 px floor. */
    @Test
    fun widerReportedCreaseGovernsTheClearance() {
        /** Split with a 300 px crease at density 2. */
        val split: SettingsPaneSplit = settingsPaneSplit(
            creasePresent = true,
            windowWidthPx = 2076,
            creaseWidthPx = 300f,
            density = 2f,
        )
        assertEquals(75f - 16f, split.editorTextStartExtraDp, 0.001f)
        assertEquals(75f - 32f, split.listTextEndExtraDp, 0.001f)
    }

    // What:     `narrowerReportedCreaseKeepsTheFloor` checks a crease thinner than the floor.
    // Why:      A zero-width hinge still gets the 7.5 mm floor, so the smaller number never shrinks the clearance.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("floor governs a thin crease", () => { ... });
    // ```
    /** Confirms a 10 px reported crease gives the same clearance as a zero-width one. */
    @Test
    fun narrowerReportedCreaseKeepsTheFloor() {
        /** Split with a 10 px crease. */
        val thin: SettingsPaneSplit = settingsPaneSplit(true, 2076, 10f, 2.4375f)
        /** Split with no reported width. */
        val none: SettingsPaneSplit = settingsPaneSplit(true, 2076, 0f, 2.4375f)
        assertEquals(none, thin)
    }

    // What:     `windowWithoutCreaseFollowsTheStockRatio` checks the crease-free split of a tablet width.
    // Why:      D110 gives the list the stock ratio 0.3636 of the window and the editor the rest.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no crease: 0.3636 of 1280", () => { ... });
    // ```
    /** Confirms a 1280 px window gives 465 px to the list (0.3636 * 1280 = 465.4) and the rest to the editor. */
    @Test
    fun windowWithoutCreaseFollowsTheStockRatio() {
        /** Split of a 1280 px window without a crease. */
        val split: SettingsPaneSplit = settingsPaneSplit(
            creasePresent = false,
            windowWidthPx = 1280,
            creaseWidthPx = 0f,
            density = 1f,
        )
        assertEquals(465, split.listWidthPx)
        assertEquals(815, split.editorWidthPx)
    }

    // What:     `windowWithoutCreaseAddsNoClearance` checks that a crease-free device keeps ordinary insets.
    // Why:      Without a crease nothing needs dodging, so both panes keep their ordinary 16 dp insets.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no crease: zero extra", () => { ... });
    // ```
    /** Confirms no extra inset even when a crease width is passed, because no crease is present. */
    @Test
    fun windowWithoutCreaseAddsNoClearance() {
        /** Split of a 2076 px window without a crease, with a stray width that must be ignored. */
        val split: SettingsPaneSplit = settingsPaneSplit(
            creasePresent = false,
            windowWidthPx = 2076,
            creaseWidthPx = 500f,
            density = 1f,
        )
        assertEquals(0f, split.listTextEndExtraDp, 0f)
        assertEquals(0f, split.editorTextStartExtraDp, 0f)
        assertEquals(2076, split.listWidthPx + split.editorWidthPx)
    }

    // What:     `invalidInputsThrow` checks the two refused inputs.
    // Why:      A negative width or a non-positive density has no physical meaning, so the rule must not guess.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("bad inputs throw", () => { ... });
    // ```
    /** Confirms a negative width and a zero density each throw. */
    @Test
    fun invalidInputsThrow() {
        assertThrows(SettingsPaneException::class.java) {
            settingsPaneSplit(creasePresent = true, windowWidthPx = -1, creaseWidthPx = 0f, density = 1f)
        }
        assertThrows(SettingsPaneException::class.java) {
            settingsPaneSplit(creasePresent = false, windowWidthPx = 100, creaseWidthPx = 0f, density = 0f)
        }
    }

    // What:     `floorMatchesTheE2Band` checks the physical floor in pixels against E2's band.
    // Why:      E2 gives about 110 px centered at x 1038, from 983 to 1093, on the 2076 px panel.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("floor is 110 px", () => { ... });
    // ```
    /** Confirms the floor is about 110.3 px, so the band around 1038 spans about 983 to 1093. */
    @Test
    fun floorMatchesTheE2Band() {
        assertEquals(110.3f, CREASE_FLOOR_WIDTH_PX, 0.1f)
        assertEquals(983f, 1038f - CREASE_FLOOR_WIDTH_PX / 2f, 0.5f)
        assertEquals(1093f, 1038f + CREASE_FLOOR_WIDTH_PX / 2f, 0.5f)
    }
}
