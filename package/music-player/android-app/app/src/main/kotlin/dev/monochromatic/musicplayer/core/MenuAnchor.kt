// Placement rules for the track menu popup (D7). The menu opens just below the pressed row, flips above it
// when it would overflow the bottom of the window, and stays inside the window horizontally. Everything here
// works on plain pixel integers, so the rules run in JVM unit tests without Compose.

// What:     `package dev.monochromatic.musicplayer.core` places the placement rules beside the other pure rules.
// Why:      The popup host reads the rules from this package, and the tests need no Android types.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `data class MenuAnchorRect(...)` is the window-space rectangle of the row that opened the menu.
// Why:      The placement rules need the row's edges in pixels, and the Compose side converts its bounds into this.
//
// In TS you'd write (pseudocode):
// ```ts
// type MenuAnchorRect = { left: number; top: number; right: number; bottom: number };
// ```
/** Window-space pixel edges of the row that the track menu anchors to. */
data class MenuAnchorRect(
    /** Left edge of the row in window pixels. */
    val left: Int,
    /** Top edge of the row in window pixels. */
    val top: Int,
    /** Right edge of the row in window pixels. */
    val right: Int,
    /** Bottom edge of the row in window pixels. */
    val bottom: Int,
)

// What:     `data class MenuAnchorRequest(...)` gathers every input of one placement decision.
// Why:      The call site names each measurement, and the rule takes one parameter.
//
// In TS you'd write (pseudocode):
// ```ts
// type MenuAnchorRequest = { anchor: MenuAnchorRect; popupWidth: number; popupHeight: number;
//   windowWidth: number; windowHeight: number; startInset: number };
// ```
/** Every measurement that the menu placement reads, all in window pixels. */
data class MenuAnchorRequest(
    /** Row that opened the menu, in window pixels. */
    val anchor: MenuAnchorRect,
    /** Measured width of the menu container in pixels. */
    val popupWidth: Int,
    /** Measured height of the menu container in pixels. */
    val popupHeight: Int,
    /** Width of the window that the menu must stay inside, in pixels. */
    val windowWidth: Int,
    /** Height of the window that the menu must stay inside, in pixels. */
    val windowHeight: Int,
    /** Leftmost pixel the menu may start at, from the start inset of the pane it opens in. */
    val startInset: Int,
)

// What:     `data class MenuAnchorOffset(...)` is the window-space top-left corner of the menu.
// Why:      The popup position provider returns this value unchanged as its offset.
//
// In TS you'd write (pseudocode):
// ```ts
// type MenuAnchorOffset = { x: number; y: number };
// ```
/** Window-space pixel position of the menu's top-left corner. */
data class MenuAnchorOffset(
    /** Horizontal position of the menu's left edge in window pixels. */
    val x: Int,
    /** Vertical position of the menu's top edge in window pixels. */
    val y: Int,
)

// What:     `fun menuAnchorOffset(request: MenuAnchorRequest): MenuAnchorOffset` places the menu next to its row.
// Why:      The menu sits under its row, moves above it near the bottom, and never leaves the window.
//
// In TS you'd write (pseudocode):
// ```ts
// function menuAnchorOffset(r: MenuAnchorRequest): MenuAnchorOffset {
//   const minX = r.startInset;
//   const maxX = Math.max(minX, r.windowWidth - r.popupWidth);
//   const x = Math.min(Math.max(r.anchor.left, minX), maxX);
//   const below = r.anchor.bottom;
//   const above = r.anchor.top - r.popupHeight;
//   const maxY = Math.max(0, r.windowHeight - r.popupHeight);
//   if (below <= maxY) return { x, y: below };
//   if (above >= 0) return { x, y: above };
//   return { x, y: Math.min(Math.max(below, 0), maxY) };
// }
// ```
/**
 * Places the menu below the row, above it when the row sits too low, and inside the window in both directions.
 *
 * The left edge follows the row but never starts left of the start inset, and it never crosses the right window
 * edge. When the menu fits neither below nor above the row, it is clamped to the window at the row's bottom.
 */
fun menuAnchorOffset(request: MenuAnchorRequest): MenuAnchorOffset {
    // What:     `val minX` is the leftmost position the menu may start at.
    // Why:      The start inset keeps the menu clear of the inset that the pane leaves on its left.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const minX = r.startInset;
    // ```
    /** Leftmost pixel the menu's left edge may take. */
    val minX: Int = request.startInset.coerceAtLeast(0)
    // What:     `val maxX` is the rightmost position the menu may start at without crossing the window edge.
    // Why:      A menu wider than the remaining width still starts at the inset rather than moving off the window.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const maxX = Math.max(minX, r.windowWidth - r.popupWidth);
    // ```
    /** Rightmost pixel the menu's left edge may take. */
    val maxX: Int = maxOf(minX, request.windowWidth - request.popupWidth)
    // What:     `val x` is the menu's left edge, which follows the row and is clamped into the horizontal range.
    // Why:      Near the right edge the menu slides left so its whole width stays visible.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const x = Math.min(Math.max(r.anchor.left, minX), maxX);
    // ```
    /** Left edge of the menu in window pixels. */
    val x: Int = request.anchor.left.coerceIn(minX, maxX)
    // What:     `val maxY` is the lowest top edge at which the whole menu still fits in the window.
    // Why:      Both vertical choices and the final clamp are measured against this one limit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const maxY = Math.max(0, r.windowHeight - r.popupHeight);
    // ```
    /** Lowest top edge at which the whole menu fits in the window, never negative. */
    val maxY: Int = maxOf(0, request.windowHeight - request.popupHeight)
    // What:     `val belowY` is the top edge that puts the menu just under the row.
    // Why:      Below the row is the first choice, because the menu then reads as belonging to the row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const belowY = r.anchor.bottom;
    // ```
    /** Top edge that places the menu directly under the row. */
    val belowY: Int = request.anchor.bottom
    // What:     `val aboveY` is the top edge that puts the menu just over the row.
    // Why:      The menu flips above the row only when placing it under the row would overflow the window.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const aboveY = r.anchor.top - r.popupHeight;
    // ```
    /** Top edge that places the menu directly above the row. */
    val aboveY: Int = request.anchor.top - request.popupHeight
    return when {
        belowY <= maxY -> MenuAnchorOffset(x = x, y = belowY)
        aboveY >= 0 -> MenuAnchorOffset(x = x, y = aboveY)
        else -> MenuAnchorOffset(x = x, y = belowY.coerceIn(0, maxY))
    }
}
