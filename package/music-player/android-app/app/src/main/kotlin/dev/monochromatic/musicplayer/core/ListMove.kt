// Focus ring movement inside the folder rail and the track list (D106). Both lists mix entries that can hold the ring
// (track rows, rail cells) with entries that cannot (subfolder headers), so the movement is pure logic over one flag
// per entry. It is a port of `package/music-player/desktop-app/src/list_focus.rs` and is unit-tested on the JVM.

// What:     `package dev.monochromatic.musicplayer.core` places the rules beside the other pure key rules.
// Why:      The rules take plain lists and numbers only, so they share the core package and need no import.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `enum class ListMove` names the movement a key asks of a focused list.
// Why:      The key map turns keys into these, and `moveFocus` turns them into a new position.
//
// In TS you'd write (pseudocode):
// ```ts
// type ListMove = "up" | "down" | "home" | "end" | "page-up" | "page-down";
// ```
/** Movement of the focus ring inside a list. */
enum class ListMove {
    /** One entry toward the start. */
    UP,

    /** One entry toward the end. */
    DOWN,

    /** The first entry that can hold the ring. */
    HOME,

    /** The last entry that can hold the ring. */
    END,

    /** One page toward the start. */
    PAGE_UP,

    /** One page toward the end. */
    PAGE_DOWN,
}

// What:     `private fun stepFrom(...)` moves by whole selectable entries and stops at the ends.
// Why:      Up, Down and the page keys all move by selectable entries, never wrap, and skip headers.
//
// In TS you'd write (pseudocode):
// ```ts
// function stepFrom(selectable: boolean[], current: number, steps: number, forward: boolean): number;
// ```
/** Moves [steps] selectable entries from [current] forward or backward, stopping at the last or first one. */
private fun stepFrom(selectable: List<Boolean>, current: Int, steps: Int, forward: Boolean): Int {
    /** Position reached so far. */
    var position: Int = current
    repeat(steps) {
        /** Next selectable position in the walking direction, or null when the walk reached the end. */
        val next: Int? = if (forward) {
            (position + 1 until selectable.size).firstOrNull { index -> selectable[index] }
        } else {
            (position - 1 downTo 0).firstOrNull { index -> selectable[index] }
        }
        if (next == null) {
            return position
        }
        position = next
    }
    return position
}

// What:     `fun moveFocus(selectable, current, move, page)` returns the entry the ring moves to.
// Why:      Up and Down skip entries that cannot hold the ring, Home and End jump to the first and last that can, a
//           page key moves by `page` entries, and a ring that is not on a valid entry starts at the first one (the
//           last one for End and Page down).
//
// In TS you'd write (pseudocode):
// ```ts
// function moveFocus(selectable: boolean[], current: number | null, move: ListMove, page: number): number | null;
// ```
/**
 * Returns the entry the focus ring moves to, or null when the list has no entry that can hold it.
 *
 * @param selectable one flag per entry, true when the entry can hold the ring.
 * @param current entry the ring is on now, or null when it is nowhere.
 * @param move movement that the key asked for.
 * @param page number of entries a page key moves by; values below one count as one.
 */
fun moveFocus(selectable: List<Boolean>, current: Int?, move: ListMove, page: Int): Int? {
    /** First entry that can hold the ring. */
    val first: Int = selectable.indexOfFirst { can -> can }
    if (first < 0) {
        return null
    }
    /** Last entry that can hold the ring. */
    val last: Int = selectable.indexOfLast { can -> can }
    /** The current entry when it can hold the ring, else null. */
    val valid: Int? = current?.takeIf { at -> selectable.getOrElse(at) { false } }
    if (valid == null) {
        return if (move == ListMove.END || move == ListMove.PAGE_DOWN) last else first
    }
    /** Steps a page key moves by. */
    val pageSteps: Int = page.coerceAtLeast(1)
    return when (move) {
        ListMove.HOME -> first
        ListMove.END -> last
        ListMove.UP -> stepFrom(selectable, valid, 1, false)
        ListMove.DOWN -> stepFrom(selectable, valid, 1, true)
        ListMove.PAGE_UP -> stepFrom(selectable, valid, pageSteps, false)
        ListMove.PAGE_DOWN -> stepFrom(selectable, valid, pageSteps, true)
    }
}

// What:     `fun startFocus(selectable, preferred)` picks where the ring begins when a list gains focus.
// Why:      The ring starts on the preferred entry (the playing track or the current rail cell) when it can hold the
//           ring and otherwise on the first entry that can.
//
// In TS you'd write (pseudocode):
// ```ts
// const startFocus = (selectable: boolean[], preferred: number | null) => ...;
// ```
/** Returns where the ring begins: [preferred] when it can hold the ring, else the first usable entry, else null. */
fun startFocus(selectable: List<Boolean>, preferred: Int?): Int? {
    /** The preferred entry when it can hold the ring. */
    val usable: Int? = preferred?.takeIf { at -> selectable.getOrElse(at) { false } }
    if (usable != null) {
        return usable
    }
    return selectable.indexOfFirst { can -> can }.takeIf { first -> first >= 0 }
}

// What:     `fun firstStartingWith(labels, letter)` finds the first entry whose label starts with a typed letter.
// Why:      A letter key in the rail jumps to the first cell with that glyph (D106). Entries without a label never
//           match, and case is ignored.
//
// In TS you'd write (pseudocode):
// ```ts
// const firstStartingWith = (labels: (string | null)[], letter: string) =>
//   labels.findIndex(label => label?.toLowerCase().startsWith(letter.toLowerCase())) ?? null;
// ```
/** Returns the position of the first label that starts with [letter], ignoring case, or null when none does. */
fun firstStartingWith(labels: List<String?>, letter: Char): Int? {
    /** The typed letter in lowercase, as the text it must start with. */
    val wanted: String = letter.lowercaseChar().toString()
    return labels.indexOfFirst { label -> label != null && label.lowercase().startsWith(wanted) }
        .takeIf { found -> found >= 0 }
}

// What:     `fun ringAfter(command, selectable, labels, current, page)` is where a list command puts the focus ring.
// Why:      Both one-stop lists move their ring by the same commands, so one function decides the entry, and each
//           list only draws the ring there and runs the commands that are not movement (activate, menu).
//
// In TS you'd write (pseudocode):
// ```ts
// function ringAfter(cmd: ListKeyCommand, selectable: boolean[], labels: (string | null)[], current: number | null,
//   page: number): number | null;
// ```
/**
 * Returns the entry that the focus ring moves to for a movement or letter command, or null when the command does
 * not move the ring or finds no entry.
 *
 * @param command list command of the key table.
 * @param selectable one flag per entry, true when the entry can hold the ring.
 * @param labels one label per entry for the letter jump, null for entries without one.
 * @param current entry the ring is on now, or null.
 * @param page number of entries a page key moves by.
 */
fun ringAfter(
    command: ListKeyCommand,
    selectable: List<Boolean>,
    labels: List<String?>,
    current: Int?,
    page: Int,
): Int? = when (command) {
    is KeyMoveList -> moveFocus(selectable = selectable, current = current, move = command.move, page = page)
    is KeyJumpToLetter -> firstStartingWith(labels = labels, letter = command.letter)
        ?.takeIf { found -> selectable.getOrElse(found) { false } }
    else -> null
}
