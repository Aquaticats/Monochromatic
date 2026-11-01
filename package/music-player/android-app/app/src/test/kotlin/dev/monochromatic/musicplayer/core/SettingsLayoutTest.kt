// What:     `package dev.monochromatic.musicplayer.core` places this test beside the Settings layout rules.
// Why:      The test reaches the public pure function in the same package as the code under test.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertion and the test annotation.
// Why:      Each row's radii are compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class SettingsLayoutTest` groups the pure corner tests for the Settings template rows.
// Why:      Which corners round for a row at each position can be verified on the host JVM, without Compose.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("settingsRowRadii", () => { ... });
// ```
/** Verifies which corners of each Settings row take the group radius and which take the joint radius. */
class SettingsLayoutTest {
    // What:     `singleRowRoundsAllCorners` checks that a group of one row takes the group radius on top and bottom.
    // Why:      A lone row is both the first and the last row, so all four of its corners are group ends.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("single row takes group radius on top and bottom", () => { ... });
    // ```
    /** Confirms a one-row group rounds both its top and its bottom with the group radius. */
    @Test
    fun singleRowRoundsAllCorners() {
        assertEquals(
            SettingsRowRadii(topDp = SETTINGS_GROUP_RADIUS_DP, bottomDp = SETTINGS_GROUP_RADIUS_DP),
            settingsRowRadii(index = 0, count = 1),
        )
    }

    // What:     `firstOfTwoRowsRoundsTopOnly` checks that the first of two rows rounds only its top.
    // Why:      The first row's top is the group's outer top edge, and its bottom meets the second row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("first of two rows: group top, joint bottom", () => { ... });
    // ```
    /** Confirms the first of two rows takes the group radius on top and the joint radius on bottom. */
    @Test
    fun firstOfTwoRowsRoundsTopOnly() {
        assertEquals(
            SettingsRowRadii(topDp = SETTINGS_GROUP_RADIUS_DP, bottomDp = SETTINGS_JOINT_RADIUS_DP),
            settingsRowRadii(index = 0, count = 2),
        )
    }

    // What:     `lastOfTwoRowsRoundsBottomOnly` checks that the last of two rows rounds only its bottom.
    // Why:      The last row's bottom is the group's outer bottom edge, and its top meets the first row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("last of two rows: joint top, group bottom", () => { ... });
    // ```
    /** Confirms the last of two rows takes the joint radius on top and the group radius on bottom. */
    @Test
    fun lastOfTwoRowsRoundsBottomOnly() {
        assertEquals(
            SettingsRowRadii(topDp = SETTINGS_JOINT_RADIUS_DP, bottomDp = SETTINGS_GROUP_RADIUS_DP),
            settingsRowRadii(index = 1, count = 2),
        )
    }

    // What:     `firstOfThreeRowsRoundsTopOnly` checks that the first of three rows rounds only its top.
    // Why:      Only the group's first row touches the group's top edge, so the other rows keep joint corners on top.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("first of three rows: group top, joint bottom", () => { ... });
    // ```
    /** Confirms the first of three rows takes the group radius on top and the joint radius on bottom. */
    @Test
    fun firstOfThreeRowsRoundsTopOnly() {
        assertEquals(
            SettingsRowRadii(topDp = SETTINGS_GROUP_RADIUS_DP, bottomDp = SETTINGS_JOINT_RADIUS_DP),
            settingsRowRadii(index = 0, count = 3),
        )
    }

    // What:     `middleOfThreeRowsRoundsJointsOnly` checks that a row between two others takes joint radii only.
    // Why:      A middle row touches no group edge, so all four of its corners stay at the small joint radius.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("middle of three rows: joint on all corners", () => { ... });
    // ```
    /** Confirms a row with neighbours on both sides takes the joint radius on top and bottom. */
    @Test
    fun middleOfThreeRowsRoundsJointsOnly() {
        assertEquals(
            SettingsRowRadii(topDp = SETTINGS_JOINT_RADIUS_DP, bottomDp = SETTINGS_JOINT_RADIUS_DP),
            settingsRowRadii(index = 1, count = 3),
        )
    }

    // What:     `lastOfThreeRowsRoundsBottomOnly` checks that the last of three rows rounds only its bottom.
    // Why:      The last row is the only one that touches the group's bottom edge.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("last of three rows: joint top, group bottom", () => { ... });
    // ```
    /** Confirms the last of three rows takes the joint radius on top and the group radius on bottom. */
    @Test
    fun lastOfThreeRowsRoundsBottomOnly() {
        assertEquals(
            SettingsRowRadii(topDp = SETTINGS_JOINT_RADIUS_DP, bottomDp = SETTINGS_GROUP_RADIUS_DP),
            settingsRowRadii(index = 2, count = 3),
        )
    }

    // What:     `pageMarginsLeaveCardWidthOfPageMinusThirtyTwo` checks the card width on every measured stock page width.
    // Why:      The stock captures measure a 16 dp margin on both sides, so a card is the page width minus 32 dp with no maximum.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("card is page minus 32", () => { ... });
    // ```
    /** Confirms the card width follows the page width with no maximum and never goes negative. */
    @Test
    fun pageMarginsLeaveCardWidthOfPageMinusThirtyTwo() {
        assertEquals(288f, settingsCardWidthDp(pageWidthDp = 320f), 0f)
        assertEquals(819.7f, settingsCardWidthDp(pageWidthDp = 851.7f), 0.001f)
        assertEquals(1172f, settingsCardWidthDp(pageWidthDp = 1204f), 0f)
        assertEquals(0f, settingsCardWidthDp(pageWidthDp = 20f), 0f)
    }

    // What:     `rowIsSeventyTwoDpWithTwoDpBetweenRowsAndSixteenBetweenGroups` checks the stock row height and spacing constants.
    // Why:      The stock captures measure 72 dp rows, 2 dp between rows of a group and 16 dp between groups.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("stock row numbers", () => { ... });
    // ```
    /** Confirms the row height, the gap between rows and the gap between groups. */
    @Test
    fun rowIsSeventyTwoDpWithTwoDpBetweenRowsAndSixteenBetweenGroups() {
        assertEquals(72f, SETTINGS_ROW_HEIGHT_DP, 0f)
        assertEquals(2f, SETTINGS_ROW_GAP_DP, 0f)
        assertEquals(16f, SETTINGS_GROUP_GAP_DP, 0f)
        assertEquals(16f, SETTINGS_PAGE_MARGIN_DP, 0f)
    }

    // What:     `iconCircleIsFortyDpAtThirtyTwoAndTitleAtEightyFour` checks the icon circle size and the x positions of the circle and the title.
    // Why:      The stock captures put a 40 dp circle at x = 32 dp and the title at x = 84 dp.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("icon at 32, title at 84", () => { ... });
    // ```
    /** Confirms the circle diameter, its left edge and the title's left edge from the page edge. */
    @Test
    fun iconCircleIsFortyDpAtThirtyTwoAndTitleAtEightyFour() {
        assertEquals(40f, SETTINGS_ICON_CIRCLE_DP, 0f)
        assertEquals(32f, settingsIconLeftDp(), 0f)
        assertEquals(84f, settingsTitleLeftDp(), 0f)
    }

    // What:     `iconCircleIsCenteredInASeventyTwoDpRow` checks the circle's top inside a row and the glyph inside the circle.
    // Why:      A centered circle sits 16 dp from the top and bottom of a 72 dp row, and the 24 dp glyph is centered in it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("icon centered", () => { ... });
    // ```
    /** Confirms the circle's top offset in a row of the stock height and the glyph size inside it. */
    @Test
    fun iconCircleIsCenteredInASeventyTwoDpRow() {
        assertEquals(16f, settingsIconTopDp(rowHeightDp = SETTINGS_ROW_HEIGHT_DP), 0f)
        assertEquals(24f, SETTINGS_ICON_GLYPH_DP, 0f)
        assertEquals(8f, (SETTINGS_ICON_CIRCLE_DP - SETTINGS_ICON_GLYPH_DP) / 2f, 0f)
    }

    // What:     `groupHeightIsRowsPlusGaps` checks the height of groups of zero, one and two rows.
    // Why:      A group is its rows at 72 dp plus 2 dp between each pair, and an empty group has no height.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("group height", () => { ... });
    // ```
    /** Confirms the group height for zero, one and two rows. */
    @Test
    fun groupHeightIsRowsPlusGaps() {
        assertEquals(0f, settingsGroupHeightDp(rowCount = 0), 0f)
        assertEquals(72f, settingsGroupHeightDp(rowCount = 1), 0f)
        assertEquals(146f, settingsGroupHeightDp(rowCount = 2), 0f)
    }

    // What:     `selectedRowIsAPillWhereverItSits` checks the radii of a chosen row at each position of a group.
    // Why:      The stock two-pane Settings fills the chosen row as a pill of about 21.3 dp on all four corners.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("selected row: 21.3 on all corners", () => { ... });
    // ```
    /** Confirms the chosen row takes the selection radius on every corner, first, middle or last. */
    @Test
    fun selectedRowIsAPillWhereverItSits() {
        /** Radii expected for any chosen row. */
        val pill = SettingsRowRadii(topDp = SETTINGS_SELECTED_RADIUS_DP, bottomDp = SETTINGS_SELECTED_RADIUS_DP)
        assertEquals(21.3f, SETTINGS_SELECTED_RADIUS_DP, 0f)
        assertEquals(pill, settingsRowRadii(index = 0, count = 3, selected = true))
        assertEquals(pill, settingsRowRadii(index = 1, count = 3, selected = true))
        assertEquals(pill, settingsRowRadii(index = 2, count = 3, selected = true))
    }

    // What:     `paneFadeIsOneHundredFiftyMilliseconds` checks the cross-fade time.
    // Why:      The editor pane fades for about 150 ms when a left row is chosen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("fade is 150 ms", () => { ... });
    // ```
    /** Confirms the cross-fade time of the editor pane. */
    @Test
    fun paneFadeIsOneHundredFiftyMilliseconds() {
        assertEquals(150, SETTINGS_PANE_FADE_MILLIS)
    }
}
