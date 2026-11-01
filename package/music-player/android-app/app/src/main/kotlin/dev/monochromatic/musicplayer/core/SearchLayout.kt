// Pure layout rules for the Search page rows and status block (D126): the icon circle and slot sizes, the rounded shape
// of a result row and the margin that keeps that shape off the screen edge and the crease, and the sizes of the circle
// in the empty and no-result states. This file has no Compose imports, so it is unit-tested on the JVM, and the numbers
// are the specification that the desktop Search page copies.

// What:     `package dev.monochromatic.musicplayer.core` places these rules beside the other pure page helpers.
// Why:      The rules take plain numbers only, so they share the core package and need no import.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `internal const val SEARCH_ROW_ICON_CIRCLE_DP: Float = 40f` is the diameter of a result's icon circle.
// Why:      D109 gives a leading icon a 40 dp circle in the secondary container role, and Search rows take it.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_ROW_ICON_CIRCLE_DP = 40;
// ```
/** Diameter in dp of the filled circle that holds a result row's folder or music glyph. */
internal const val SEARCH_ROW_ICON_CIRCLE_DP: Float = SETTINGS_ICON_CIRCLE_DP

// What:     `internal const val SEARCH_ROW_ICON_SLOT_DP: Float = 48f` is the width of the leading slot of a row.
// Why:      D58 makes the Back arrow and the result icons share one horizontal center, and Back is a 48 dp target,
//           so the circle sits centered in a slot of the same 48 dp and the title starts 48 dp after the content.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_ROW_ICON_SLOT_DP = 48;
// ```
/** Width in dp of the slot that holds a row's icon circle, equal to the Back target (D58). */
internal const val SEARCH_ROW_ICON_SLOT_DP: Float = 48f

// What:     `internal const val SEARCH_ROW_SHAPE_MARGIN_DP: Float = 8f` is how far a row's rounded shape sits in.
// Why:      A rounded pressed and focus shape must not touch the screen edge or the crease, while D58 keeps the
//           icon and the title where they were, so the shape starts 8 dp before the content and the row pads it back.
//           The circle starts 4 dp inside its slot, so it keeps a 12 dp gap to the shape's edge.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_ROW_SHAPE_MARGIN_DP = 8;
// ```
/** Distance in dp from the edge of a row's rounded shape to the start of its content, on each side. */
internal const val SEARCH_ROW_SHAPE_MARGIN_DP: Float = 8f

// What:     `internal const val SEARCH_ROW_RADIUS_DP: Float = 21.3f` is the corner radius of a row's rounded shape.
// Why:      D109 gave the pressed, hovered and selected states of Settings rows this radius, and Search rows reuse it.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_ROW_RADIUS_DP = 21.3;
// ```
/** Corner radius in dp of all four corners of a result row's pressed, hovered and focus shape. */
internal const val SEARCH_ROW_RADIUS_DP: Float = SETTINGS_SELECTED_RADIUS_DP

// What:     `internal const val SEARCH_STATUS_CIRCLE_DP: Float = 72f` is the diameter of the status circle.
// Why:      The empty and no-result states put their glyph in a 72 dp circle of the secondary container role, a plain
//           circle and not the Expressive scalloped shape (A2).
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_STATUS_CIRCLE_DP = 72;
// ```
/** Diameter in dp of the circle that holds the glyph of the empty and no-result states. */
internal const val SEARCH_STATUS_CIRCLE_DP: Float = 72f

// What:     `internal const val SEARCH_STATUS_GLYPH_DP: Float = 32f` is the drawn size of the status glyph.
// Why:      The glyph is a 24 dp Material glyph scaled to 32 dp so that it fills the 72 dp circle like the 24 dp glyph
//           fills the 40 dp circle.
//
// In TS you'd write (pseudocode):
// ```ts
// const SEARCH_STATUS_GLYPH_DP = 32;
// ```
/** Size in dp of the magnifier glyph at the center of the status circle. */
internal const val SEARCH_STATUS_GLYPH_DP: Float = 32f

// What:     `internal data class SearchRowInsets(...)` holds the outer margins and the inner padding of one row.
// Why:      A row has a rounded shape that starts before its content; three numbers place both, and a pure value keeps
//           the rule that the content does not move testable.
//
// In TS you'd write (pseudocode):
// ```ts
// type SearchRowInsets = { outerStartDp: number; outerEndDp: number; innerDp: number };
// ```
/** Outer start and end margins and the inner padding of one result row, all in dp. */
internal data class SearchRowInsets(
    /** Distance from the page edge to the start of the row's rounded shape. */
    val outerStartDp: Float,
    /** Distance from the page edge to the end of the row's rounded shape. */
    val outerEndDp: Float,
    /** Padding between the rounded shape's edge and the row's content, on each side. */
    val innerDp: Float,
)

// What:     `internal fun searchRowInsets(startInsetDp, endInsetDp): SearchRowInsets` places a row's shape.
// Why:      The content must start at the page's start inset and end at its end inset (D58), and on the unfolded
//           panel the start inset already holds half the crease, so the shape starts after the crease clear zone.
//           The margin is the smaller of the standard margin and both insets, so a small inset never pushes the shape
//           off the page.
//
// In TS you'd write (pseudocode):
// ```ts
// const searchRowInsets = (start: number, end: number) => {
//   const m = Math.max(0, Math.min(8, start, end));
//   return { outerStartDp: start - m, outerEndDp: end - m, innerDp: m };
// };
// ```
/**
 * Places the rounded shape of a result row so that its content keeps [startInsetDp] and [endInsetDp].
 *
 * @param startInsetDp distance from the page edge to the start of the row content, which includes the crease clearance.
 * @param endInsetDp distance from the page edge to the end of the row content.
 */
internal fun searchRowInsets(startInsetDp: Float, endInsetDp: Float): SearchRowInsets {
    /** Margin of the shape, never larger than either inset and never negative. */
    val margin: Float = minOf(SEARCH_ROW_SHAPE_MARGIN_DP, startInsetDp, endInsetDp).coerceAtLeast(0f)
    return SearchRowInsets(outerStartDp = startInsetDp - margin, outerEndDp = endInsetDp - margin, innerDp = margin)
}
