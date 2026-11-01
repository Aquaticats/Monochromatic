// Pure rules for how far text keeps from the crease on the unfolded player (E2, D110). Settings and Search
// already dodge the crease; this file gives the player's two panes the same rule: the folder picker and the deck
// on the left, the track list on the right. Every text on the unfolded screen then clears the band by the same
// amount.

// What:     `package dev.monochromatic.musicplayer.core` places the rule beside the pane split it builds on.
// Why:      The rule takes plain numbers only, so it shares the core package and is unit-tested on the JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `internal class PaneClearanceException` signals a density that cannot convert pixels to dp.
// Why:      A non-positive density has no physical meaning, so the rule throws instead of guessing a clearance.
//
// In TS you'd write (pseudocode):
// ```ts
// class PaneClearanceError extends Error {}
// ```
/** Thrown when the density is not positive. */
internal class PaneClearanceException(message: String) : IllegalArgumentException(message)

// What:     `internal data class PaneClearance(startDp, endDp)` is the clearance text keeps at each side of a pane.
// Why:      The left pane keeps clear of the crease at its end and the right pane at its start, and each pane's
//           composables read the value of their own side.
//
// In TS you'd write (pseudocode):
// ```ts
// type PaneClearance = { startDp: number; endDp: number };
// ```
/** Distance in dp that text keeps clear at the start and at the end of one pane. */
internal data class PaneClearance(
    /** Clearance at the pane's start edge in dp, zero when the start edge is not the crease. */
    val startDp: Float,
    /** Clearance at the pane's end edge in dp, zero when the end edge is not the crease. */
    val endDp: Float,
)

// What:     `internal val NO_PANE_CLEARANCE: PaneClearance` is the clearance of a pane next to no crease.
// Why:      A window without a crease, and the folded cover, keep the ordinary insets only (D110).
//
// In TS you'd write (pseudocode):
// ```ts
// const NO_PANE_CLEARANCE = { startDp: 0, endDp: 0 };
// ```
/** Clearance of a pane that has no crease beside it. */
internal val NO_PANE_CLEARANCE: PaneClearance = PaneClearance(startDp = 0f, endDp = 0f)

// What:     `internal fun leftPaneClearance(creaseWidthPx, density)` is the clearance of the pane left of the crease.
// Why:      Text in the left pane ends half of `max(7.5 mm, crease width)` before the shared edge, so the clearance
//           is at the pane's end; a null crease width means the window has no crease and nothing is added.
//
// In TS you'd write (pseudocode):
// ```ts
// const leftPaneClearance = (creaseWidthPx, density) =>
//   creaseWidthPx === null ? NONE : { startDp: 0, endDp: creaseHalfClearanceDp(creaseWidthPx, density) };
// ```
/**
 * Returns the clearance of the left pane, at its end edge, for a window whose crease is [creaseWidthPx] wide.
 *
 * @param creaseWidthPx crease width in physical pixels as the platform reports it, or null when there is no crease.
 * @param density physical pixels per dp of the current display.
 * @throws PaneClearanceException when the density is not positive.
 */
internal fun leftPaneClearance(creaseWidthPx: Float?, density: Float): PaneClearance {
    if (density <= 0f) {
        throw PaneClearanceException("Density must be positive to convert the crease clearance, but was $density.")
    }
    if (creaseWidthPx == null) {
        return NO_PANE_CLEARANCE
    }
    return PaneClearance(
        startDp = 0f,
        endDp = creaseHalfClearanceDp(creaseWidthPx = creaseWidthPx, density = density),
    )
}

// What:     `internal fun rightPaneClearance(creaseWidthPx, density)` is the clearance of the pane right of the crease.
// Why:      Text in the right pane starts half of `max(7.5 mm, crease width)` after the shared edge, so the clearance
//           is at the pane's start.
//
// In TS you'd write (pseudocode):
// ```ts
// const rightPaneClearance = (creaseWidthPx, density) =>
//   creaseWidthPx === null ? NONE : { startDp: creaseHalfClearanceDp(creaseWidthPx, density), endDp: 0 };
// ```
/**
 * Returns the clearance of the right pane, at its start edge, for a window whose crease is [creaseWidthPx] wide.
 *
 * @param creaseWidthPx crease width in physical pixels as the platform reports it, or null when there is no crease.
 * @param density physical pixels per dp of the current display.
 * @throws PaneClearanceException when the density is not positive.
 */
internal fun rightPaneClearance(creaseWidthPx: Float?, density: Float): PaneClearance {
    if (density <= 0f) {
        throw PaneClearanceException("Density must be positive to convert the crease clearance, but was $density.")
    }
    if (creaseWidthPx == null) {
        return NO_PANE_CLEARANCE
    }
    return PaneClearance(
        startDp = creaseHalfClearanceDp(creaseWidthPx = creaseWidthPx, density = density),
        endDp = 0f,
    )
}

// What:     `internal fun paneTextInsetDp(ordinaryDp, clearanceDp)` is the inset a text keeps from a pane edge.
// Why:      A text keeps its ordinary inset (16 dp for most rows) unless the crease needs more, and the larger value
//           wins, so the ordinary look is unchanged wherever the crease is far enough away.
//
// In TS you'd write (pseudocode):
// ```ts
// const paneTextInsetDp = (ordinaryDp, clearanceDp) => Math.max(ordinaryDp, clearanceDp);
// ```
/** Returns the larger of a text's ordinary inset and the crease clearance, in dp. */
internal fun paneTextInsetDp(ordinaryDp: Float, clearanceDp: Float): Float = maxOf(ordinaryDp, clearanceDp)
