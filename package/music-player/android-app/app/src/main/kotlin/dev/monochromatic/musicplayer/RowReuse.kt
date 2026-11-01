// What:     `package dev.monochromatic.musicplayer` puts this helper beside `BrainPlayer`, which uses it.
// Why:      The rule is plain Kotlin with no Android types, so unit tests reach it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `fun <Item : Any, Row : Any> reuseRows(...)` builds the list of derived rows for a list of items, keeping
//           the row of the previous call at every position whose item is the very same object and that the caller
//           did not name as changed. `<Item : Any, Row : Any>` are two type parameters, each "any non-null type".
// Why:      The media session timeline needs one heavy object per queue row. A track change alters the rows of two
//           positions at most (the one that stopped being current and the one that became current), so building the
//           other rows again only costs time proportional to the queue size.
// Gotcha:   `===` is reference identity (like TS `===` on objects), not Kotlin's value equality `==`.
//
// In TS you'd write (pseudocode):
// ```ts
// function reuseRows<Item, Row>(args: {
//   previousItems: readonly Item[]; previousRows: readonly Row[]; items: readonly Item[];
//   rebuilt: readonly number[]; build: (position: number, item: Item) => Row;
// }): readonly Row[] {
//   if (args.items === args.previousItems && args.rebuilt.length === 0) return args.previousRows;
//   return args.items.map((item, position) =>
//     position < args.previousItems.length && args.previousItems[position] === item && !args.rebuilt.includes(position)
//       ? args.previousRows[position]
//       : args.build(position, item));
// }
// ```
/**
 * Builds one row per item, reusing the previous row wherever the item is the same instance and the position is not
 * named as changed.
 *
 * @param previousItems items the previous rows were built from
 * @param previousRows rows built for [previousItems], one per item
 * @param items items to build rows for
 * @param rebuilt positions whose row depends on more than the item (the current track), always built again
 * @param build creates the row for a position and its item
 * @return one row per item; the same list instance as [previousRows] when nothing needs building
 */
fun <Item : Any, Row : Any> reuseRows(
    previousItems: List<Item>,
    previousRows: List<Row>,
    items: List<Item>,
    rebuilt: Set<Int>,
    build: (Int, Item) -> Row,
): List<Row> {
    if (items === previousItems) {
        if (rebuilt.isEmpty()) {
            return previousRows
        }
        /** Copy of the previous rows that receives the rebuilt ones; the items are unchanged, so only those differ. */
        val patched: MutableList<Row> = ArrayList(previousRows)
        for (position in rebuilt) {
            if (position in items.indices) {
                patched[position] = build(position, items[position])
            }
        }
        return patched
    }
    /** Rows in item order. */
    val rows: MutableList<Row> = ArrayList(items.size)
    for (position in items.indices) {
        /** Item at this position. */
        val item: Item = items[position]
        if (position < previousItems.size && previousItems[position] === item && position !in rebuilt) {
            rows.add(previousRows[position])
        } else {
            rows.add(build(position, item))
        }
    }
    return rows
}

/**
 * Finds the positions whose row differs although their item is the same instance: the track that stopped being
 * current and the one that became current, because only the current row reports a duration.
 *
 * @param previousCurrent position of the current track when the rows were last built, or null
 * @param previousDurationMs duration of the current track in milliseconds when the rows were last built
 * @param current position of the current track now, or null
 * @param durationMs duration of the current track in milliseconds now
 * @return no positions when neither the current track nor its duration changed
 */
fun currentRowPositions(previousCurrent: Int?, previousDurationMs: Long, current: Int?, durationMs: Long): Set<Int> {
    if (current == previousCurrent && durationMs == previousDurationMs) {
        return emptySet()
    }
    return setOfNotNull(previousCurrent, current)
}
