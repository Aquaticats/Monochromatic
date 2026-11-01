// Folder index for the folder picker and the In order playback walk. It groups the library by its
// top-level folder (D3 and D17), then lays out each folder as one flat list (D5): the folder's own
// tracks first, then each subfolder as a header row followed by its tracks. Playback order follows the
// same list (D6).

// What:     `package dev.monochromatic.musicplayer.core` places this index beside the rail and path helpers.
// Why:      The index is pure logic over `Track` values, so it can be unit-tested on the JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import dev.monochromatic.musicplayer.Track` brings in the library track type.
// Why:      `Track` lives in the parent package, so the core package must import it by absolute name.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Track } from "dev.monochromatic.musicplayer/Track";
// ```
import dev.monochromatic.musicplayer.Track

// What:     `private const val PATH_SEPARATOR: String = "/"` is the separator between display path segments.
// Why:      Display paths use `/`, and the split must cut on the same character the path builders join with.
//
// In TS you'd write (pseudocode):
// ```ts
// const PATH_SEPARATOR = "/";
// ```
/** Separator between the segments of a relative display path. */
private const val PATH_SEPARATOR: String = "/"

// What:     `private const val CURRENT_DIRECTORY: String = "."` is the segment that names the current directory.
// Why:      Such segments are dropped so that `A/./x.flac` groups exactly like `A/x.flac`, as in RelPath.kt.
//
// In TS you'd write (pseudocode):
// ```ts
// const CURRENT_DIRECTORY = ".";
// ```
/** Path segment that refers to the directory itself and is ignored. */
private const val CURRENT_DIRECTORY: String = "."

// What:     `private const val PARENT_DIRECTORY: String = ".."` is the segment that names the parent directory.
// Why:      Such segments are dropped so that they cannot create a folder that the display path does not show.
//
// In TS you'd write (pseudocode):
// ```ts
// const PARENT_DIRECTORY = "..";
// ```
/** Path segment that refers to the parent directory and is ignored. */
private const val PARENT_DIRECTORY: String = ".."

// What:     `private const val MIN_FOLDERED_SEGMENTS: Int = 2` is the fewest segments a track path needs to sit
//           inside a folder: the folder name and the file name.
// Why:      A single-segment path names a file at the root, so it goes to the virtual entry of its initial letter.
//
// In TS you'd write (pseudocode):
// ```ts
// const MIN_FOLDERED_SEGMENTS = 2;
// ```
/** Fewest path segments a track needs to sit inside a top-level folder. */
private const val MIN_FOLDERED_SEGMENTS: Int = 2

// What:     `data class FolderTrackRow(val track: Track, val indexInLibrary: Int)` pairs a track with its
//           position in the library list it came from.
// Why:      Playback and selection address tracks by library index, so the index must travel with the track.
//
// In TS you'd write (pseudocode):
// ```ts
// type FolderTrackRow = { track: Track; indexInLibrary: number };
// ```
/** One library track together with its position in the library list. */
data class FolderTrackRow(
    /** Track as it appears in the library. */
    val track: Track,
    /** Position of the track in the library list the index was built from. */
    val indexInLibrary: Int,
)

// What:     `sealed interface FolderListItem` is one row of a folder's flat list. Its two variants are a
//           track row and a subfolder header row.
// Why:      A sealed type lets the picker render both row kinds exhaustively with no extra layer (D5).
//
// In TS you'd write (pseudocode):
// ```ts
// type FolderListItem = { kind: "track"; row: FolderTrackRow } | { kind: "header"; name: string; trackCount: number };
// ```
/** One row in a folder's flat list, either a track or a subfolder header. */
sealed interface FolderListItem {
    // What:     `data class TrackItem(val row: FolderTrackRow) : FolderListItem` is a playable track row.
    // Why:      Each track in a folder's list is wrapped so the picker can tell it apart from headers.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "track", row };
    // ```
    /** A playable track row inside a folder's list. */
    data class TrackItem(
        /** Track and its library position. */
        val row: FolderTrackRow,
    ) : FolderListItem

    // What:     `data class SubfolderHeader(val name: String, val trackCount: Int) : FolderListItem` is the header
    //           row for one subfolder.
    // Why:      The header names the subfolder and counts the tracks directly under it, and its tracks follow it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "header", name, trackCount };
    // ```
    /** A header row that names a subfolder and counts the tracks that follow it. */
    data class SubfolderHeader(
        /** Subfolder path between the top-level folder and the tracks, such as `Album/Disc 1`. */
        val name: String,
        /** Number of tracks listed directly under this header. */
        val trackCount: Int,
    ) : FolderListItem
}

// What:     `data class FolderEntry(val name: String, val items: List<FolderListItem>, val trackCount: Int)` is
//           one top-level folder with its flat list and its total track count.
// Why:      The picker shows each folder's name and count, and playback walks `items` in order.
//
// In TS you'd write (pseudocode):
// ```ts
// type FolderEntry = { name: string; items: FolderListItem[]; trackCount: number };
// ```
/** One top-level folder, its flat list of rows, and the total number of tracks under it. */
data class FolderEntry(
    /** Top-level folder name, or the letter (A to Z, or #) of the loose tracks of a loose entry. */
    val name: String,
    /** Flat list of track rows and subfolder headers in playback order. */
    val items: List<FolderListItem>,
    /** Every track under this folder, including tracks in deeper subfolders. */
    val trackCount: Int,
    /** Whether this entry is a virtual folder of loose root-level tracks, named by their initial letter (D28). */
    val loose: Boolean = false,
)

// What:     `private data class PlacedRow(val folder: String?, val subfolder: String, val row: FolderTrackRow)`
//           records where one track sits: its top-level folder (null at the root) and its subfolder path.
// Why:      Placement is computed once per track, so grouping and sorting work on plain values.
//
// In TS you'd write (pseudocode):
// ```ts
// type PlacedRow = { folder: string | null; subfolder: string; row: FolderTrackRow };
// ```
/** One track with its top-level folder and the subfolder path between that folder and the file. */
private data class PlacedRow(
    /** Top-level folder name, or null when the track sits at the library root. */
    val folder: String?,
    /** Subfolder path between the top-level folder and the file, empty when the track is directly inside. */
    val subfolder: String,
    /** Track and its library position. */
    val row: FolderTrackRow,
)

// What:     `private fun splitSegments(displayPath: String): List<String>` cuts a display path into its
//           non-empty segments, dropping `.` and `..`.
// Why:      `RelPath.kt` keeps its own splitter private, so folder grouping repeats the same rule here.
//
// In TS you'd write (pseudocode):
// ```ts
// const splitSegments = (p: string) => p.split("/").filter((s) => s !== "" && s !== "." && s !== "..");
// ```
/** Splits a relative display path into its meaningful segments. */
private fun splitSegments(displayPath: String): List<String> =
    displayPath.split(PATH_SEPARATOR).filter { segment ->
        segment.isNotEmpty() && segment != CURRENT_DIRECTORY && segment != PARENT_DIRECTORY
    }

// What:     `private fun placeTrack(row: FolderTrackRow): PlacedRow` finds a track's top-level folder and its
//           subfolder path. Paths with fewer than two segments are placed at the root.
// Why:      The top-level folder is the first segment, and the subfolder path is every segment between it and
//           the file name, joined through `joinDisplayPath` so the name is sanitized the same way.
//
// In TS you'd write (pseudocode):
// ```ts
// function placeTrack(row: FolderTrackRow): PlacedRow { /* first segment, then the middle segments */ }
// ```
/** Places one track under its top-level folder and subfolder path, or at the root. */
private fun placeTrack(row: FolderTrackRow): PlacedRow {
    /** Non-empty segments of the track's display path, file name last. */
    val segments = splitSegments(row.track.displayPath)
    if (segments.size < MIN_FOLDERED_SEGMENTS) {
        return PlacedRow(null, "", row)
    }
    /** Top-level folder, which is the first segment of the path. */
    val folder = segments.first()
    /** Path from just below the top-level folder down to the file's directory, joined with the shared sanitizer. */
    val subfolder = segments
        .subList(1, segments.size - 1)
        .fold("") { prefix, segment -> joinDisplayPath(prefix, segment) }
    return PlacedRow(folder, subfolder, row)
}

// What:     `private fun folderNameOrder(): Comparator<String>` sorts names case-insensitively and
//           accent-insensitively, breaking ties by code point.
// Why:      The rail collator alone leaves names such as `Camellia` and `camellia` tied, and a tie-break keeps
//           the picker order stable across runs.
//
// In TS you'd write (pseudocode):
// ```ts
// const folderNameOrder = (a: string, b: string) => collator.compare(a, b) || (a < b ? -1 : a > b ? 1 : 0);
// ```
/** Builds the comparator used to order folder names and subfolder headers. */
private fun folderNameOrder(): Comparator<String> {
    /** Collator that compares names case-insensitively and accent-insensitively. */
    val collator = railCollator()
    return Comparator<String> { left, right ->
        /** Collator result, zero when the names tie at primary strength. */
        val primary = collator.compare(left, right)
        if (primary != 0) primary else compareByCodePoint(left, right)
    }
}

// What:     `private fun headerAndTracks(subfolder: String, rows: List<FolderTrackRow>): List<FolderListItem>`
//           returns one subfolder's header followed by its tracks in library order.
// Why:      D5 shows each subfolder as a header row with its count, followed by that subfolder's tracks.
//
// In TS you'd write (pseudocode):
// ```ts
// function headerAndTracks(name: string, rows: FolderTrackRow[]): FolderListItem[] {
//   return [{ kind: "header", name, trackCount: rows.length }, ...rows.map((row) => ({ kind: "track", row }))];
// }
// ```
/** Builds one subfolder's header row followed by its track rows. */
private fun headerAndTracks(subfolder: String, rows: List<FolderTrackRow>): List<FolderListItem> {
    /** Header naming this subfolder, carrying the number of tracks directly under it. */
    val header: FolderListItem = FolderListItem.SubfolderHeader(subfolder, rows.size)
    return listOf(header) + rows.map { row -> FolderListItem.TrackItem(row) }
}

// What:     `private fun folderEntry(name: String, placed: List<PlacedRow>): FolderEntry` builds one top-level
//           folder's flat list: its own tracks first, then each subfolder sorted by name.
// Why:      D5 and D6 put the folder's own tracks first and then the subfolders in name order, with no
//           extra layer. Grouping by subfolder once keeps the build linear in the number of tracks.
//
// In TS you'd write (pseudocode):
// ```ts
// function folderEntry(name: string, placed: PlacedRow[]): FolderEntry { /* direct tracks, then sorted groups */ }
// ```
/** Builds one top-level folder's flat list from its placed tracks. */
private fun folderEntry(name: String, placed: List<PlacedRow>): FolderEntry {
    /** Track rows sitting directly inside this folder, in library order. */
    val ownTracks: List<FolderListItem> = placed
        .filter { placedRow -> placedRow.subfolder.isEmpty() }
        .map { placedRow -> FolderListItem.TrackItem(placedRow.row) }
    /** Track rows grouped by subfolder path, in library order within each group. */
    val bySubfolder: Map<String, List<FolderTrackRow>> = placed
        .filter { placedRow -> placedRow.subfolder.isNotEmpty() }
        .groupBy({ placedRow -> placedRow.subfolder }, { placedRow -> placedRow.row })
    /** Subfolder headers and tracks, with subfolders in name order. */
    val subfolderItems: List<FolderListItem> = bySubfolder.keys
        .sortedWith(folderNameOrder())
        .flatMap { subfolder -> headerAndTracks(subfolder, bySubfolder.getValue(subfolder)) }
    return FolderEntry(name, ownTracks + subfolderItems, placed.size)
}

// What:     `fun folderIndex(tracks: List<Track>): List<FolderEntry>` groups the library by top-level folder
//           and returns the folder entries in name order. Root-level tracks form one virtual entry per initial
//           letter, named by that letter (the page they sit on in the picker), listed after the folders, `#` last. An
//           empty library gives an empty list.
// Why:      The folder picker and In order playback both need the same flat list, so it is built once here.
//
// In TS you'd write (pseudocode):
// ```ts
// export function folderIndex(tracks: Track[]): FolderEntry[] { /* group by first segment, sort, flatten */ }
// ```
/** Groups the library into folder entries with flat, playback-ordered track lists. */
fun folderIndex(tracks: List<Track>): List<FolderEntry> {
    /** Every track with its library position and its placement. */
    val placed: List<PlacedRow> = tracks.mapIndexed { index, track ->
        placeTrack(FolderTrackRow(track, index))
    }
    /** Placed tracks grouped by top-level folder, with root-level tracks under a null key. */
    val byFolder: Map<String?, List<PlacedRow>> = placed.groupBy { placedRow -> placedRow.folder }
    /** Folder entries sorted by name, each built from its own placed tracks. */
    val folders: List<FolderEntry> = byFolder.keys
        .filterNotNull()
        .sortedWith(folderNameOrder())
        .map { name -> folderEntry(name, byFolder.getValue(name)) }
    /** Root-level tracks in library order, grouped under the letter of the page each one sits on. */
    val looseByLabel: Map<String, List<PlacedRow>> = byFolder[null].orEmpty().groupBy { placedRow ->
        looseFolderLabel(splitSegments(placedRow.row.track.displayPath).firstOrNull().orEmpty())
    }
    /** Virtual entries of the loose tracks: letters in order, then `#`, each listing its tracks in library order. */
    val looseEntries: List<FolderEntry> = looseByLabel.keys
        .sortedWith(compareBy({ label -> label == CATCH_ALL_LABEL }, { label -> label }))
        .map { label ->
            /** Tracks that sit on this letter's page. */
            val rows: List<PlacedRow> = looseByLabel.getValue(label)
            FolderEntry(
                label,
                rows.map { placedRow -> FolderListItem.TrackItem(placedRow.row) },
                rows.size,
                loose = true,
            )
        }
    return folders + looseEntries
}
