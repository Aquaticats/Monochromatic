// Which row of the open track menu holds the keyboard ring (D114): Down and Up move it, Enter runs it, and the menu
// starts with no row ringed so a letter or Escape stays the quickest way out. The rule has no Compose types so the
// JVM tests can state it exactly.

// What:     `package dev.monochromatic.musicplayer.core` places the rule beside the menu's other pure rules.
// Why:      The popup composable only draws the ring and forwards the keys.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `fun menuRowAfter(current: Int?, count: Int, step: Int): Int?` moves the menu's ring by one row.
// Why:      The first Down lands on the first row and the first Up on the last, and moving past either end wraps,
//           as the platform's own menus do.
//
// In TS you'd write (pseudocode):
// ```ts
// const menuRowAfter = (current: number | null, count: number, step: number) =>
//   count === 0 ? null : current === null ? (step > 0 ? 0 : count - 1) : (current + step + count) % count;
// ```
/**
 * Returns the row that holds the ring after moving [step] rows (1 for Down, -1 for Up) from [current], or null when the
 * menu has no rows.
 */
fun menuRowAfter(current: Int?, count: Int, step: Int): Int? {
    if (count <= 0) {
        return null
    }
    if (current == null) {
        return if (step > 0) 0 else count - 1
    }
    return Math.floorMod(current + step, count)
}
