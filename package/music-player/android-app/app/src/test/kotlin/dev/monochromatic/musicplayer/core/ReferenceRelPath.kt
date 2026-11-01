// The old relative-path implementation, kept as the test oracle for the faster one (D112 scale work). It builds a list of
// components per path and joins them again, which is correct and slow; its answers are the specification.

// What:     `package ...core` places the oracle beside the tests that compare against it.
// Why:      The tests call both implementations through the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

/** Splits a path into its named segments, dropping empty ones and the `.` and `..` markers. */
private fun referenceComponents(path: String): List<String> =
    path.split("/").filter { it.isNotEmpty() && it != "." && it != ".." }

/** Returns how many leading segments every list shares, capped so the last segment always survives. */
private fun referencePrefixLen(lists: List<List<String>>): Int {
    val shortest = lists.minOfOrNull { it.size } ?: 0
    if (shortest == 0) return 0
    val cap = shortest - 1
    var run = 0
    while (run < cap && lists.all { it[run] == lists[0][run] }) {
        run++
    }
    return run
}

/** The old `relativeDisplayPaths`: each path without the leading segments that all paths share. */
fun referenceRelativeDisplayPaths(tracks: List<String>): List<String> {
    if (tracks.isEmpty()) return emptyList()
    val componentLists = tracks.map { referenceComponents(it) }
    val prefixLen = referencePrefixLen(componentLists)
    return componentLists.zip(tracks).map { (list, path) ->
        val relative = list.drop(prefixLen).joinToString("/")
        if (relative.isEmpty()) path else relative
    }
}
