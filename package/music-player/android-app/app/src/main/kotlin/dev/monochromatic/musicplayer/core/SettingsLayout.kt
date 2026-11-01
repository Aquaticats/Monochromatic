// Pure layout rules for the Settings template rows: the stock margins, row height, icon circle positions and gaps, and
// which radius each row of a grouped card gets. This file has no Compose imports, so it is unit-tested on the JVM.

// What:     `package dev.monochromatic.musicplayer.core` places these rules beside the other pure page helpers.
// Why:      The rules take plain numbers only, so they share the core package and need no import.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `internal const val SETTINGS_PAGE_MARGIN_DP: Float = 16f` is the margin between the page edge and a card.
// Why:      The stock Settings captures measure a 16 dp margin on both sides at every width from 320 dp to 1200 dp, so
//           a card is the page width minus 32 dp and the page has no maximum width.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_PAGE_MARGIN_DP = 16;
// ```
/** Margin in dp between each page edge and a grouped card. */
internal const val SETTINGS_PAGE_MARGIN_DP: Float = 16f

// What:     `internal const val SETTINGS_ROW_HEIGHT_DP: Float = 72f` is the height of a row at the default text size.
// Why:      The stock captures measure 72 dp rows (72.0 to 72.6 dp), and a larger text size grows a row past it.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_ROW_HEIGHT_DP = 72;
// ```
/** Height in dp of one row at the default text size, which is also its minimum height. */
internal const val SETTINGS_ROW_HEIGHT_DP: Float = 72f

// What:     `internal const val SETTINGS_ROW_INNER_PADDING_DP: Float = 16f` is the inset between a card edge and
//           the row's leading icon circle.
// Why:      The stock icon circle starts 32 dp from the screen edge, which is the 16 dp margin plus 16 dp inside the
//           card.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_ROW_INNER_PADDING_DP = 16;
// ```
/** Horizontal inset in dp between a card's edge and the content of a row. */
internal const val SETTINGS_ROW_INNER_PADDING_DP: Float = 16f

// What:     `internal const val SETTINGS_ICON_CIRCLE_DP: Float = 40f` is the diameter of a row's icon circle.
// Why:      The stock captures measure a 40 dp circle (39.8 to 40.2 dp) behind every leading icon.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_ICON_CIRCLE_DP = 40;
// ```
/** Diameter in dp of the filled circle that holds a row's leading icon. */
internal const val SETTINGS_ICON_CIRCLE_DP: Float = 40f

// What:     `internal const val SETTINGS_ICON_GLYPH_DP: Float = 24f` is the drawn size of the icon in its circle.
// Why:      Material Symbols are drawn on a 24 by 24 grid, and the glyph is centered in the 40 dp circle.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_ICON_GLYPH_DP = 24;
// ```
/** Size in dp of the icon glyph, which sits at the center of the icon circle. */
internal const val SETTINGS_ICON_GLYPH_DP: Float = 24f

// What:     `internal const val SETTINGS_ICON_TITLE_GAP_DP: Float = 12f` is the space between the circle and the title.
// Why:      The stock title starts 84 dp from the screen edge: 32 dp to the circle, 40 dp of circle, then 12 dp.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_ICON_TITLE_GAP_DP = 12;
// ```
/** Space in dp between the right edge of the icon circle and the start of the row's title. */
internal const val SETTINGS_ICON_TITLE_GAP_DP: Float = 12f

// What:     `internal const val SETTINGS_ROW_GAP_DP: Float = 2f` is the gap between two rows of one group.
// Why:      The stock captures show 2 dp of page background between the rows of a group, with no divider line.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_ROW_GAP_DP = 2;
// ```
/** Gap in dp between two rows of one group. */
internal const val SETTINGS_ROW_GAP_DP: Float = 2f

// What:     `internal const val SETTINGS_GROUP_GAP_DP: Float = 16f` is the gap between two groups.
// Why:      The stock captures show 16 dp of page background between one grouped card and the next.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_GROUP_GAP_DP = 16;
// ```
/** Gap in dp between two grouped cards. */
internal const val SETTINGS_GROUP_GAP_DP: Float = 16f

// What:     `internal const val SETTINGS_SELECTED_RADIUS_DP: Float = 21.3f` is the corner radius of the chosen row.
// Why:      The stock two-pane Settings draws the chosen row as a filled pill whose corner radius measures 21.3 dp
//           at 851 dp wide and 22.2 dp at 720 dp wide (`measurements-split-layout.txt`), rounder than the group
//           corners. The brief said about 27.5 dp, which no measurement file contains, so the measured value is used.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_SELECTED_RADIUS_DP = 21.3;
// ```
/** Corner radius in dp of all four corners of the chosen row's fill. */
internal const val SETTINGS_SELECTED_RADIUS_DP: Float = 21.3f

// What:     `internal const val SETTINGS_PANE_FADE_MILLIS: Int = 150` is the cross-fade time of the editor pane.
// Why:      The stock two-pane Settings cross-fades the right pane when a left row is chosen, with no slide; the
//           system animation scale still stretches or removes the fade.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_PANE_FADE_MILLIS = 150;
// ```
/** Duration in milliseconds of the cross-fade between two editors, before the system animation scale applies. */
internal const val SETTINGS_PANE_FADE_MILLIS: Int = 150

// What:     `internal fun settingsCardWidthDp(pageWidthDp: Float): Float` gives the card width for a page width.
// Why:      The card is the page width minus the margin on each side, with no maximum and never below zero.
//
// In TS you'd write (pseudocode):
// ```ts
// const settingsCardWidthDp = (pageWidthDp: number) => Math.max(0, pageWidthDp - 2 * 16);
// ```
/** Card width in dp on a page [pageWidthDp] wide, or zero when the page is narrower than both margins. */
internal fun settingsCardWidthDp(pageWidthDp: Float): Float =
    (pageWidthDp - 2f * SETTINGS_PAGE_MARGIN_DP).coerceAtLeast(0f)

// What:     `internal fun settingsIconLeftDp(): Float` gives the icon circle's distance from the page edge.
// Why:      The circle sits 16 dp inside a card that is itself 16 dp in from the page edge.
//
// In TS you'd write (pseudocode):
// ```ts
// const settingsIconLeftDp = () => 16 + 16;
// ```
/** Distance in dp from the page edge to the left side of the icon circle. */
internal fun settingsIconLeftDp(): Float = SETTINGS_PAGE_MARGIN_DP + SETTINGS_ROW_INNER_PADDING_DP

// What:     `internal fun settingsTitleLeftDp(): Float` gives the title's distance from the page edge.
// Why:      The title follows the icon circle and the gap after it, which lands on the stock 84 dp.
//
// In TS you'd write (pseudocode):
// ```ts
// const settingsTitleLeftDp = () => settingsIconLeftDp() + 40 + 12;
// ```
/** Distance in dp from the page edge to the left side of the row's title and summary. */
internal fun settingsTitleLeftDp(): Float =
    settingsIconLeftDp() + SETTINGS_ICON_CIRCLE_DP + SETTINGS_ICON_TITLE_GAP_DP

// What:     `internal fun settingsIconTopDp(rowHeightDp: Float): Float` gives the icon circle's top inside a row.
// Why:      The circle is centered vertically, so a 72 dp row puts it 16 dp from the top and the bottom.
//
// In TS you'd write (pseudocode):
// ```ts
// const settingsIconTopDp = (rowHeightDp: number) => (rowHeightDp - 40) / 2;
// ```
/** Distance in dp from the top of a row [rowHeightDp] tall to the top of its icon circle. */
internal fun settingsIconTopDp(rowHeightDp: Float): Float = (rowHeightDp - SETTINGS_ICON_CIRCLE_DP) / 2f

// What:     `internal fun settingsGroupHeightDp(rowCount: Int): Float` gives a group's height at the default text size.
// Why:      A group is its rows at the stock height plus the 2 dp gap between each pair, so the page can be checked
//           against the measured stack.
//
// In TS you'd write (pseudocode):
// ```ts
// const settingsGroupHeightDp = (rowCount: number) => rowCount * 72 + Math.max(0, rowCount - 1) * 2;
// ```
/** Height in dp of a group of [rowCount] rows at the default text size, or zero for an empty group. */
internal fun settingsGroupHeightDp(rowCount: Int): Float =
    if (rowCount <= 0) 0f else rowCount * SETTINGS_ROW_HEIGHT_DP + (rowCount - 1) * SETTINGS_ROW_GAP_DP

// What:     `internal const val SETTINGS_GROUP_RADIUS_DP: Float = 14f` is the corner radius at the outer ends of
//           a group.
// Why:      The stock Settings captures measure about 14 dp at the first row's top and the last row's bottom.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_GROUP_RADIUS_DP = 14;
// ```
/** Corner radius in dp at the outer ends of a grouped card: the top of its first row and the bottom of its last. */
internal const val SETTINGS_GROUP_RADIUS_DP: Float = 14f

// What:     `internal const val SETTINGS_JOINT_RADIUS_DP: Float = 2f` is the corner radius where two rows meet.
// Why:      The stock captures show rows separated by a small gap with only slightly rounded inner corners.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_JOINT_RADIUS_DP = 2;
// ```
/** Corner radius in dp at the edges where two rows of one group meet. */
internal const val SETTINGS_JOINT_RADIUS_DP: Float = 2f

// What:     `internal data class SettingsRowRadii` holds the top and bottom corner radius of one row.
// Why:      A row needs two radii, one for its top corners and one for its bottom corners, and a pure pair
//           keeps them testable.
//
// In TS you'd write (pseudocode):
// ```ts
// type SettingsRowRadii = { topDp: number; bottomDp: number };
// ```
/** Top and bottom corner radii of one grouped row, both in dp. */
internal data class SettingsRowRadii(
    /** Radius in dp applied to both top corners of the row. */
    val topDp: Float,
    /** Radius in dp applied to both bottom corners of the row. */
    val bottomDp: Float,
)

// What:     `internal fun settingsRowRadii(index: Int, count: Int): SettingsRowRadii` gives the row at [index]
//           of [count] rows its corner radii.
// Why:      A group's first row rounds its top and its last row rounds its bottom, while rows in between take
//           the small joint radius, so the gap between rows shows the page background. A group of one row
//           rounds all four corners.
//
// In TS you'd write (pseudocode):
// ```ts
// const settingsRowRadii = (index: number, count: number, selected = false) => selected
//   ? { topDp: 21.3, bottomDp: 21.3 }
//   : { topDp: index === 0 ? 14 : 2, bottomDp: index === count - 1 ? 14 : 2 };
// ```
/**
 * Radii for the row at [index] of [count] rows, where [index] lies in 0 until [count].
 * The [selected] row is a pill with the selection radius on all four corners wherever it sits in the group.
 */
internal fun settingsRowRadii(index: Int, count: Int, selected: Boolean = false): SettingsRowRadii {
    if (selected) {
        return SettingsRowRadii(topDp = SETTINGS_SELECTED_RADIUS_DP, bottomDp = SETTINGS_SELECTED_RADIUS_DP)
    }
    /** Whether this row is the first of its group, which takes the group radius on top. */
    val isFirst: Boolean = index == 0
    /** Whether this row is the last of its group, which takes the group radius on the bottom. */
    val isLast: Boolean = index == count - 1
    return SettingsRowRadii(
        topDp = if (isFirst) SETTINGS_GROUP_RADIUS_DP else SETTINGS_JOINT_RADIUS_DP,
        bottomDp = if (isLast) SETTINGS_GROUP_RADIUS_DP else SETTINGS_JOINT_RADIUS_DP,
    )
}
