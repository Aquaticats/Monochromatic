// The first visible track that the track list reports to the measure refresh, as a library index. This file has no
// Compose imports, so the conversion from a list position to a library index is unit-tested on the JVM.

// What:     `package ...core` places the report beside the list rules it reads.
// Why:      The report uses `libraryIndexNear` and the entry types without an import.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import kotlinx.coroutines.flow.Flow

// What:     `internal const val NO_VISIBLE_TRACK: Int = -1` is the value of the cell while no track is on screen.
// Why:      A library index is never negative, so -1 cannot be mistaken for a track.
//
// In TS you'd write (pseudocode):
// ```ts
// const NO_VISIBLE_TRACK = -1;
// ```
/** Value of the first visible track cell before the list has reported, or when the list shows no track. */
internal const val NO_VISIBLE_TRACK: Int = -1

// What:     `internal suspend fun reportFirstVisibleTrack(...)` hands [report] the library index of the first track on
//           screen for every first visible position the list reaches.
// Why:      The measure refresh starts reading at the first visible track, and the list knows only a position, which
//           can be a header with no library index of its own and differs from the library index once the page holds
//           headers or a subset of the library.
//
// In TS you'd write (pseudocode):
// ```ts
// async function reportFirstVisibleTrack(
//   positions: AsyncIterable<number>, entries: TrackListEntry[], report: (index: number) => void,
// ) {
//   for await (const position of positions) report(libraryIndexNear(entries, position) ?? NO_VISIBLE_TRACK);
// }
// ```
/**
 * Collects [positions] and reports, for each one, the library index of the track it shows or is nearest to.
 *
 * Reports [NO_VISIBLE_TRACK] when [entries] holds no track row. The call returns when [positions] completes.
 *
 * @param positions first visible list positions, such as the list state's first visible item index
 * @param entries entries of the shown page, rows and subfolder headers
 * @param report receives the library index of the first visible track, or [NO_VISIBLE_TRACK]
 */
internal suspend fun reportFirstVisibleTrack(
    positions: Flow<Int>,
    entries: List<TrackListEntry>,
    report: (Int) -> Unit,
) {
    positions.collect { position -> report(libraryIndexNear(entries, position) ?: NO_VISIBLE_TRACK) }
}
