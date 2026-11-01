// Search results for the Search page. It turns the library and a query into one of five states: no query
// yet, unavailable, an empty library, no match, or a ranked list of folder and track results. Folder names
// and track titles are the only matched fields (D60), and folders and tracks share one relevance order (D61).

// What:     `package dev.monochromatic.musicplayer.core` places the results beside the matcher and folder index.
// Why:      Ranking and the state rules are pure logic over `Track` values, so they can be unit-tested on the JVM.
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

// What:     `const val SEARCH_RESULT_LIMIT: Int = 500` caps how many results one search returns.
// Why:      A huge library must not build an unbounded list, and the result reports when the cap cut it short.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_RESULT_LIMIT = 500;
// ```
/** Largest number of results one search returns before the list is cut short. */
const val SEARCH_RESULT_LIMIT: Int = 500

// What:     `enum class SearchResultKind { FOLDER, TRACK }` names the two kinds of Search result.
// Why:      The row needs to know whether a tap opens a folder (D72) or plays a track (D73).
//
// In TS you'd write (pseudocode):
// ```ts
// type SearchResultKind = "folder" | "track";
// ```
/** The two kinds of entry a Search result can describe. */
enum class SearchResultKind {
    // What:     `FOLDER` is a top-level folder whose own name matched.
    // Why:      A folder result opens its folder without starting playback (D72).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const FOLDER = "folder";
    // ```
    /** A top-level folder whose own name matched the query. */
    FOLDER,

    // What:     `TRACK` is a track whose own file name, without its extension, matched the query.
    // Why:      A track result selects the track, and also starts it with Auto-play on select on (D127, D129; the
    //           current track's result toggles play and pause) and carries its folder as context (D77).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const TRACK = "track";
    // ```
    /** A track whose own title, from its file name without the extension, matched the query. */
    TRACK,
}

// What:     `data class SearchResult(...)` is one matched folder or track, ready to draw.
// Why:      The row needs the title, the context line, the highlights for both, and the action target in one value.
//
// In TS you'd write (pseudocode):
// ```ts
// type SearchResult = { kind: SearchResultKind; name: string; parentContext: string | null; /* ... */ };
// ```
/** One matched folder or track in a Search result list. */
data class SearchResult(
    /** Whether this result is a folder or a track. */
    val kind: SearchResultKind,
    /** Folder name, or track title without its extension. */
    val name: String,
    /**
     * Folder path that holds the track as displayed. Null for a folder, and null for a root-level track,
     * which has no folder to name.
     */
    val parentContext: String?,
    /** Position of the track in the library list, or null for a folder. */
    val indexInLibrary: Int?,
    /** Top-level folder that owns this result, which is the folder itself for a folder result. */
    val folderName: String?,
    /** Relevance tier, where 0 is the best match and a larger value is a weaker match. */
    val tier: Int,
    /** Ranges of the name to emphasize, in the name's own offsets. */
    val nameHighlights: List<SearchHighlight>,
    /** Ranges of the parent context to emphasize, in the context's own offsets, and empty for a folder. */
    val parentHighlights: List<SearchHighlight>,
    /**
     * URI of the track, or null for a folder and for an index built without URIs. The running app resolves a tap and
     * the playing row through it, because [indexInLibrary] is the track's number in the index, which is its arrival
     * number while a scan is running and differs from its position in the sorted library.
     */
    val trackUri: String? = null,
)

// What:     `sealed interface SearchState` lists the five states the Search page can be in.
// Why:      The page must tell an empty prompt, an unavailable library, a finished no-match, and results apart (D69).
//
// In TS you'd write (pseudocode):
// ```ts
// type SearchState =
//   | { kind: "emptyQuery" }
//   | { kind: "unavailable" }
//   | { kind: "emptyInventory" }
//   | { kind: "noMatch"; query: string }
//   | { kind: "results"; items: SearchResult[]; truncated: boolean };
// ```
/** The state of the Search page for one query and library snapshot. */
sealed interface SearchState {
    // What:     `data object EmptyQuery` is the state for a blank query.
    // Why:      A blank query shows the search invitation, not a failed lookup (D70).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "emptyQuery" }
    // ```
    /** No usable query was typed, so the page shows its invitation. */
    data object EmptyQuery : SearchState

    // What:     `data object EmptyInventory` is the state for a usable source that holds no tracks.
    // Why:      A confirmed empty inventory is not a finished no-match (D69), so it gets its own copy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "emptyInventory" }
    // ```
    /** The source is usable and confirmed to hold no tracks, so no name can be searched. */
    data object EmptyInventory : SearchState

    // What:     `data object Unavailable` is the state when the current source cannot be searched.
    // Why:      A known source failure takes precedence over every query state (D69 and D71).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "unavailable" }
    // ```
    /** The library cannot be searched right now, so no result claim is made. */
    data object Unavailable : SearchState

    // What:     `data class NoMatch(val query: String)` is the state for a finished search with no hits.
    // Why:      The page names the entered text so the user knows which query found nothing (D70).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "noMatch"; query: string }
    // ```
    /** The query was evaluated against the library and no folder or track name matched it. */
    data class NoMatch(
        /** The entered query, trimmed of surrounding whitespace. */
        val query: String,
    ) : SearchState

    // What:     `data class Results(...)` is the state for one or more matches.
    // Why:      Results carry their order, and the truncated flag tells the page the list is not the whole answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "results"; items: SearchResult[]; truncated: boolean }
    // ```
    /** Ranked matches for a query, with a flag that says whether the limit cut the list short. */
    data class Results(
        /** Matched folders and tracks in relevance order, at most `SEARCH_RESULT_LIMIT` of them. */
        val items: List<SearchResult>,
        /** Whether more matches existed than `items` holds. */
        val truncated: Boolean,
    ) : SearchState
}

// What:     `fun searchLibrary(query: String, tracks: List<Track>, available: Boolean): SearchState` answers one
//           Search query against a list of tracks by indexing them once.
// Why:      The page must know which of the five states applies, and a source failure must win over the query state.
//           The running app keeps one `SearchIndex` alive instead (D112); this one-shot form serves callers that hold a
//           plain list, and it builds the same index, so both give the same answers.
//
// In TS you'd write (pseudocode):
// ```ts
// function searchLibrary(query: string, tracks: Track[], available: boolean): SearchState { /* index, then search */ }
// ```
/**
 * Evaluates one query against the library and reports the page state.
 *
 * An unavailable source wins over a blank query, so a known failure is never hidden behind the search prompt (D69).
 * A blank query, or one that normalizes to no terms, returns `EmptyQuery`. A usable source with no tracks returns
 * `EmptyInventory`, which is not a finished no-match (D69). A complete evaluation with no hit returns `NoMatch`.
 * Otherwise the results are ordered and cut at `SEARCH_RESULT_LIMIT`.
 */
fun searchLibrary(query: String, tracks: List<Track>, available: Boolean): SearchState {
    if (!available) {
        return SearchState.Unavailable
    }
    /** Index of the given tracks, in list order so a track's number is its position in the list. */
    val index = SearchIndex()
    index.appendPaths(tracks.map { track -> track.displayPath }, tracks.map { track -> track.uri })
    return index.search(query)
}
