// Pure rule for where a flush track row's box and its text start and end (D58, D110, D120, D130). The box carries the
// fill, the press and hover layers and the keyboard ring; the text sits in the column that the headings of the same
// pane use. Beside the crease only the text dodges it, so the box always spans the pane and only the text column
// moves.

// What:     `package dev.monochromatic.musicplayer.core` places the rule beside the pane clearance it reads.
// Why:      The rule takes plain numbers only, so it shares the core package and is unit-tested on the JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `internal const val TRACK_ROW_RING_TEXT_GAP_DP: Float = 8f` is the least space between a ring and the text.
// Why:      The ring lies just inside the row's box edge (D120); the text keeps this much room beyond the ring's inner
//           edge so the first and last letters never touch it.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRACK_ROW_RING_TEXT_GAP_DP = 8;
// ```
/** Least distance in dp from the inner edge of a row's focus ring to the start or end of its text. */
internal const val TRACK_ROW_RING_TEXT_GAP_DP: Float = 8f

// What:     `internal data class TrackRowColumns(...)` holds the four insets of one flush track row.
// Why:      The box insets say where the fill, the state layers and the ring begin and end; the text insets say where
//           the title and the supporting line begin and end, measured from the pane edge in both cases.
//
// In TS you'd write (pseudocode):
// ```ts
// type TrackRowColumns = { boxStartDp: number; boxEndDp: number; textStartDp: number; textEndDp: number };
// ```
/** Distances in dp from a pane's start and end edges to a flush row's box and to its text. */
internal data class TrackRowColumns(
    /** Distance from the pane's start edge to the start of the row's box. */
    val boxStartDp: Float,
    /** Distance from the pane's end edge to the end of the row's box. */
    val boxEndDp: Float,
    /** Distance from the pane's start edge to the start of the row's text. */
    val textStartDp: Float,
    /** Distance from the pane's end edge to the end of the row's text. */
    val textEndDp: Float,
)

// What:     `internal fun trackRowColumns(...)` decides the box and text insets of a flush track row.
// Why:      The crease rejects text only, so the box starts and ends at the pane's edges even beside the crease
//           (D130), and the text takes `paneTextInsetDp`, the same inset that the headings and the title bar use
//           (D58, D110). The floor of ring depth plus gap keeps the text clear of the ring whatever the ordinary
//           inset is, and never moves the text off the column while the ordinary inset is at least that large.
//
// In TS you'd write (pseudocode):
// ```ts
// const trackRowColumns = (p) => ({
//   boxStartDp: 0, boxEndDp: 0,
//   textStartDp: Math.max(Math.max(p.ordinaryInsetDp, p.startClearanceDp), p.ringDepthDp + p.ringTextGapDp),
//   textEndDp: Math.max(Math.max(p.ordinaryInsetDp, p.endClearanceDp), p.ringDepthDp + p.ringTextGapDp),
// });
// ```
/**
 * Returns the insets of a flush track row in a pane.
 *
 * @param startClearanceDp crease clearance at the pane's start edge, zero when that edge is not the crease.
 * @param endClearanceDp crease clearance at the pane's end edge, zero when that edge is not the crease.
 * @param ordinaryInsetDp inset of the text from a pane edge that has no crease.
 * @param ringDepthDp depth of the focus ring inside the box edge.
 * @param ringTextGapDp least room between the ring's inner edge and the text.
 */
internal fun trackRowColumns(
    startClearanceDp: Float,
    endClearanceDp: Float,
    ordinaryInsetDp: Float,
    ringDepthDp: Float,
    ringTextGapDp: Float,
): TrackRowColumns {
    /** Least text inset that keeps the ring's gap, which the heading column may exceed but not undercut. */
    val ringFloorDp: Float = ringDepthDp + ringTextGapDp
    return TrackRowColumns(
        boxStartDp = 0f,
        boxEndDp = 0f,
        textStartDp = maxOf(paneTextInsetDp(ordinaryDp = ordinaryInsetDp, clearanceDp = startClearanceDp), ringFloorDp),
        textEndDp = maxOf(paneTextInsetDp(ordinaryDp = ordinaryInsetDp, clearanceDp = endClearanceDp), ringFloorDp),
    )
}
