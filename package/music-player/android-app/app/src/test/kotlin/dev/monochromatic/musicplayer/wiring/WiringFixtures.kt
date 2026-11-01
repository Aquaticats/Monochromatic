// Invented data and bounds shared by the Compose wiring tests, which render the real track list pane and player host
// under Robolectric. Nothing here comes from a real library: every title, folder and index is made up.

// What:     `package ...wiring` places the Robolectric Compose tests beside the other test packages of the app.
// Why:      The fixtures use only public and `internal` members of the app module, which its tests may reach.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.wiring

import android.content.ContentProvider
import android.content.ContentValues
import android.database.Cursor
import android.net.Uri
import dev.monochromatic.musicplayer.core.PageEntry
import dev.monochromatic.musicplayer.core.TrackListEntry
import dev.monochromatic.musicplayer.core.TrackRowModel
import java.util.concurrent.CopyOnWriteArrayList

// What:     `internal const val WAIT_LIMIT_MS: Long = 10_000` is the longest wall-clock wait of one condition.
// Why:      A mutant that stops a scroll or a marker from ever happening must fail the wait, not hang the Gradle run.
//
// In TS you'd write (pseudocode):
// ```ts
// const WAIT_LIMIT_MS = 10_000;
// ```
/** Longest wall-clock wait, in milliseconds, for one condition of a wiring test. */
internal const val WAIT_LIMIT_MS: Long = 10_000

// What:     `internal const val CASE_LIMIT_MS: Long = 90_000` is the longest run of one whole test case.
// Why:      Robolectric runs a timed test on its own main thread and interrupts it when the limit passes, so a stuck
//           frame loop becomes a failure of that one case.
//
// In TS you'd write (pseudocode):
// ```ts
// const CASE_LIMIT_MS = 90_000;
// ```
/** Longest run, in milliseconds, of one test case, passed as the `timeout` of its `@Test`. */
internal const val CASE_LIMIT_MS: Long = 90_000

// What:     `internal const val FIRST_LIBRARY_INDEX: Int = 1000` is the library index of the first invented track.
// Why:      A position in the list and a library index are different numbers in the app. Offsetting the index keeps a
//           wiring mistake that passes one for the other from reaching the right row by luck.
//
// In TS you'd write (pseudocode):
// ```ts
// const FIRST_LIBRARY_INDEX = 1000;
// ```
/** Library index of the invented track at position zero of a page; later tracks count up from it. */
internal const val FIRST_LIBRARY_INDEX: Int = 1000

// What:     `internal fun titleOf(track: Int): String` names the invented track with the given number.
// Why:      Zero-padded numbers keep every title unique as a whole, so a test finds one row by its exact text.
//
// In TS you'd write (pseudocode):
// ```ts
// const titleOf = (track: number): string => `Track ${String(track).padStart(2, "0")}`;
// ```
/** Title of the invented track with the given number, such as `Track 07`. */
internal fun titleOf(track: Int): String = "Track " + track.toString().padStart(2, '0')

// What:     `internal fun libraryIndexOf(track: Int): Int` gives the library index of the invented track.
// Why:      The pane and the host name the playing track by library index, never by list position.
//
// In TS you'd write (pseudocode):
// ```ts
// const libraryIndexOf = (track: number): number => FIRST_LIBRARY_INDEX + track;
// ```
/** Library index of the invented track with the given number. */
internal fun libraryIndexOf(track: Int): Int = FIRST_LIBRARY_INDEX + track

// What:     `internal fun trackEntry(track: Int, flaggedCurrent: Boolean): TrackListEntry` builds one invented row.
// Why:      The host builds its rows with no playing track, so rows start unflagged; a case may flag one on purpose to
//           show that the flag a row arrives with never decides what is drawn.
//
// In TS you'd write (pseudocode):
// ```ts
// const trackEntry = (track: number, flaggedCurrent = false): TrackListEntry =>
//   ({ kind: "row", row: { indexInLibrary: libraryIndexOf(track), title: titleOf(track), supportingLine: null, isCurrent: flaggedCurrent } });
// ```
/** One row entry for the invented track with the given number, unflagged unless [flaggedCurrent] is set. */
internal fun trackEntry(track: Int, flaggedCurrent: Boolean = false): TrackListEntry = TrackListEntry.RowEntry(
    TrackRowModel(
        indexInLibrary = libraryIndexOf(track),
        title = titleOf(track),
        supportingLine = null,
        isCurrent = flaggedCurrent,
    ),
)

// What:     `internal fun flatPage(count: Int): List<TrackListEntry>` builds a page of rows with no subfolders.
// Why:      A page longer than the screen is the smallest page on which "where the list scrolled to" can be seen.
//
// In TS you'd write (pseudocode):
// ```ts
// const flatPage = (count: number): TrackListEntry[] => range(count).map((t) => trackEntry(t));
// ```
/** A page of [count] invented rows, numbered from zero, with no subfolder headers. */
internal fun flatPage(count: Int): List<TrackListEntry> = (0 until count).map { track -> trackEntry(track) }

// What:     `internal fun foldedPage(): List<TrackListEntry>` builds a page with two subfolder headers.
// Why:      The folder reveal scrolls to a header, and a header shifts every later row's position from its number.
//
// In TS you'd write (pseudocode):
// ```ts
// const foldedPage = () => [...rows(0, 20), header("Alpha"), ...rows(20, 40), header("Beta"), ...rows(40, 60)];
// ```
/** Twenty own rows, then the header "Alpha" with twenty rows, then the header "Beta" with twenty rows. */
internal fun foldedPage(): List<TrackListEntry> =
    (0 until 20).map { track -> trackEntry(track) } +
        TrackListEntry.HeaderEntry(name = "Alpha", trackCount = 20) +
        (20 until 40).map { track -> trackEntry(track) } +
        TrackListEntry.HeaderEntry(name = "Beta", trackCount = 20) +
        (40 until 60).map { track -> trackEntry(track) }

// What:     `internal fun pageItemsOf(count: Int, folder: String): List<PageEntry>` builds the page the host draws.
// Why:      The host builds its rows from the controller's page entries; their names are display paths that start
//           with the folder, so the folder's own tracks become plain rows.
//
// In TS you'd write (pseudocode):
// ```ts
// const pageItemsOf = (count: number, folder: string) => range(count).map((t) => ({ index: libraryIndexOf(t), name: `${folder}/${titleOf(t)}.mp3` }));
// ```
/** Page entries of [count] invented tracks in [folder], whose display paths are `folder/Track NN.mp3`. */
internal fun pageItemsOf(count: Int, folder: String): List<PageEntry> = (0 until count).map { track ->
    PageEntry(index = libraryIndexOf(track), name = "$folder/${titleOf(track)}.mp3")
}

// What:     `internal class RecordingProvider : ContentProvider()` is a content provider that only records its queries.
// Why:      The measure refresh asks the content resolver about each track's file first; the order of those questions
//           is the order the refresh read the tracks in, which is how a case sees where a pass started.
//
// In TS you'd write (pseudocode):
// ```ts
// class RecordingProvider { queried: string[] = []; query(uri: string) { this.queried.push(uri); return null; } }
// ```
/** Answers every query with no rows and keeps the URIs asked about, in order, for a case to read. */
internal class RecordingProvider : ContentProvider() {
    /** URIs queried so far, appended from the reader threads of the refresh and read from the test thread. */
    val queried: MutableList<String> = CopyOnWriteArrayList()

    /** Reports the provider as ready; it holds nothing to set up. */
    override fun onCreate(): Boolean = true

    /** Records [uri] and answers with no cursor, which the refresh reads as an unknown file. */
    override fun query(
        uri: Uri,
        projection: Array<out String>?,
        selection: String?,
        selectionArgs: Array<out String>?,
        sortOrder: String?,
    ): Cursor? {
        queried += uri.toString()
        return null
    }

    /** Has no type for any URI. */
    override fun getType(uri: Uri): String? = null

    /** Refuses inserts, which the refresh never makes. */
    override fun insert(uri: Uri, values: ContentValues?): Uri? = null

    /** Refuses deletes, which the refresh never makes. */
    override fun delete(uri: Uri, selection: String?, selectionArgs: Array<out String>?): Int = 0

    /** Refuses updates, which the refresh never makes. */
    override fun update(uri: Uri, values: ContentValues?, selection: String?, selectionArgs: Array<out String>?): Int = 0
}
