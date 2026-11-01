// The old Search implementation, kept verbatim as the test oracle for the in-memory index (D112). It rebuilds the
// folder index and normalizes every name on every query, which is why it cost seconds at one million tracks, but its
// answers are the specification: the index must return exactly the same tier, kind, name, library position, count,
// truncated flag and highlight ranges.

// What:     `package ...core` places the oracle beside the tests that compare against it.
// Why:      It reaches the matcher, the folder index and the rail collator through the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.Track
import java.text.Collator

/** Separator between the segments of a display path. */
private const val PATH_SEPARATOR: Char = '/'

// What:     `private data class SearchCandidate(...)` pairs a result with the values used only for ordering.
// Why:      The display path and normalized length decide order but are not shown, so they stay out of the result.
//
// In TS you'd write (pseudocode):
// ```ts
// type SearchCandidate = { result: SearchResult; normalizedLength: number; sortPath: string };
// ```
/** A matched result together with the sort keys that order it. */
private data class SearchCandidate(
    /** The result that will be shown if the candidate survives the limit. */
    val result: SearchResult,
    /** Length of the normalized, trimmed name, used before the collator to order equal tiers. */
    val normalizedLength: Int,
    /** Display path of the folder or track, used as the last ordering key before the kind and library position. */
    val sortPath: String,
)

// What:     `private fun parentContextFor(displayPath: String): String?` returns the folder path that holds a track,
//           or null for a root-level track.
// Why:      The second line of a track result names its folder (D77). A root-level track has no parent folder, so
//           it has no second line, and no parent is invented for it.
//
// In TS you'd write (pseudocode):
// ```ts
// const parentContextFor = (path: string) => path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : null;
// ```
/** Returns the folder path of a track as displayed, or null when the track sits at the library root. */
private fun parentContextFor(displayPath: String): String? {
    /** The display path without its last segment, which is empty for a root-level track. */
    val parent: String = displayPath.substringBeforeLast(PATH_SEPARATOR, "")
    if (parent.isEmpty()) {
        return null
    }
    return parent
}

// What:     `private fun folderCandidate(entry: FolderEntry, query: String, terms: List<String>): SearchCandidate?`
//           builds a folder result when the folder's own name matches.
// Why:      Only the folder's name is matched (D60), so the folder's tracks never enter the list through it.
//
// In TS you'd write (pseudocode):
// ```ts
// function folderCandidate(entry: FolderEntry, query: string, terms: string[]): SearchCandidate | null { /* ... */ }
// ```
/** Builds a folder result for an entry whose name matches every term, or returns null when it does not. */
private fun folderCandidate(entry: FolderEntry, query: String, terms: List<String>): SearchCandidate? {
    /** Normalized, trimmed folder name that the matcher compares. */
    val normalizedName: String = normalizeForSearch(entry.name).trim()
    /** Relevance tier of the folder name, or null when the name does not match, which returns early. */
    val tier: Int = matchTier(normalizedName, terms) ?: return null
    return SearchCandidate(
        result = SearchResult(
            kind = SearchResultKind.FOLDER,
            name = entry.name,
            parentContext = null,
            indexInLibrary = null,
            folderName = entry.name,
            tier = tier,
            nameHighlights = highlightRanges(entry.name, query),
            parentHighlights = emptyList(),
        ),
        normalizedLength = normalizedName.length,
        sortPath = entry.name,
    )
}

// What:     `private fun trackCandidate(row: FolderTrackRow, folderName: String, query: String, terms: List<String>)`
//           builds a track result when the track's own title matches.
// Why:      Only the final file name without its extension is matched (D60). Its folder path is context,
//           never a source of hits.
//
// In TS you'd write (pseudocode):
// ```ts
// function trackCandidate(row, folderName, query, terms): SearchCandidate | null { /* ... */ }
// ```
/** Builds a track result for a row whose title matches every term, or returns null when it does not. */
private fun trackCandidate(
    row: FolderTrackRow,
    folderName: String,
    query: String,
    terms: List<String>,
): SearchCandidate? {
    /** Display path of the track, which supplies the title, the context, and the sort path. */
    val displayPath: String = row.track.displayPath
    /** Title of the track, which is its file name without the last extension. */
    val title: String = trackTitleFor(displayPath)
    /** Normalized, trimmed title that the matcher compares. */
    val normalizedTitle: String = normalizeForSearch(title).trim()
    /** Relevance tier of the track title, or null when the title does not match, which returns early. */
    val tier: Int = matchTier(normalizedTitle, terms) ?: return null
    /** Folder path that holds the track, shown as its context line, or null for a root-level track. */
    val parentContext: String? = parentContextFor(displayPath)
    return SearchCandidate(
        result = SearchResult(
            kind = SearchResultKind.TRACK,
            name = title,
            parentContext = parentContext,
            indexInLibrary = row.indexInLibrary,
            folderName = folderName,
            tier = tier,
            nameHighlights = highlightRanges(title, query),
            parentHighlights = if (parentContext == null) emptyList() else highlightRanges(parentContext, query),
            trackUri = row.track.uri,
        ),
        normalizedLength = normalizedTitle.length,
        sortPath = displayPath,
    )
}

// What:     `private fun candidatesFor(query: String, terms: List<String>, tracks: List<Track>)` returns the
//           `List<SearchCandidate>` of every folder and track that matches, before ordering and the limit.
// Why:      Folders come from the folder index without its virtual letter folders, and tracks come from every folder's
//           track rows, so each track is searched exactly once.
//
// In TS you'd write (pseudocode):
// ```ts
// function candidatesFor(query: string, terms: string[], tracks: Track[]): SearchCandidate[] { /* ... */ }
// ```
/** Gathers every matching folder result and track result for the library, unordered. */
private fun candidatesFor(query: String, terms: List<String>, tracks: List<Track>): List<SearchCandidate> {
    /** Folder entries of the library, one per top-level folder plus one virtual letter folder per initial of the loose tracks. */
    val entries: List<FolderEntry> = folderIndex(tracks)
    /** Folder results for entries whose own name matches; the virtual letter folders of loose tracks are excluded. */
    val folderMatches: List<SearchCandidate> = entries
        .filterNot { entry -> entry.loose }
        .mapNotNull { entry -> folderCandidate(entry, query, terms) }
    /** Track results for every track row of every entry, each carrying its owning folder name. */
    val trackMatches: List<SearchCandidate> = entries.flatMap { entry ->
        entry.items
            .filterIsInstance<FolderListItem.TrackItem>()
            .mapNotNull { item -> trackCandidate(item.row, entry.name, query, terms) }
    }
    return folderMatches + trackMatches
}

// What:     `private fun candidateOrder(collator: Collator): Comparator<SearchCandidate>` orders candidates
//           for display.
// Why:      Ordering runs tier first, then shorter names, then collator order, then display path, then kind
//           and library position, so two equal results never swap places between repeated searches.
//
// In TS you'd write (pseudocode):
// ```ts
// const candidateOrder = (collator: Intl.Collator) => (a, b) => a.tier - b.tier || a.len - b.len || /* ... */ 0;
// ```
/** Builds the total order used to rank candidates, using the given collator for names. */
private fun candidateOrder(collator: Collator): Comparator<SearchCandidate> =
    compareBy<SearchCandidate>({ candidate -> candidate.result.tier }, { candidate -> candidate.normalizedLength })
        .thenComparator { left, right -> collator.compare(left.result.name, right.result.name) }
        .thenComparator { left, right -> compareByCodePoint(left.sortPath, right.sortPath) }
        .thenBy { candidate -> candidate.result.kind }
        .thenComparator { left, right -> compareValues(left.result.indexInLibrary, right.result.indexInLibrary) }

// What:     `fun referenceSearchLibrary(query: String, tracks: List<Track>, available: Boolean): SearchState` answers one
//           Search query against the library.
// Why:      The page must know which of the four states applies, and a source failure must win over the query state.
//
// In TS you'd write (pseudocode):
// ```ts
// function referenceSearchLibrary(query: string, tracks: Track[], available: boolean): SearchState { /* ... */ }
// ```
/**
 * Evaluates one query against the library and reports the page state.
 *
 * An unavailable source wins over a blank query, so a known failure is never hidden behind the search prompt (D69).
 * A blank query, or one that normalizes to no terms, returns `EmptyQuery`. A usable source with no tracks returns
 * `EmptyInventory`, which is not a finished no-match (D69). A complete evaluation with no hit returns `NoMatch`.
 * Otherwise the results are ordered and cut at `SEARCH_RESULT_LIMIT`.
 */
fun referenceSearchLibrary(query: String, tracks: List<Track>, available: Boolean): SearchState {
    if (!available) {
        return SearchState.Unavailable
    }
    /** Normalized query terms, which every match must satisfy. */
    val terms: List<String> = searchTerms(query)
    if (terms.isEmpty()) {
        return SearchState.EmptyQuery
    }
    if (tracks.isEmpty()) {
        return SearchState.EmptyInventory
    }
    /** Every matching folder and track, before ordering and the limit. */
    val candidates: List<SearchCandidate> = candidatesFor(query, terms, tracks)
    if (candidates.isEmpty()) {
        return SearchState.NoMatch(query.trim())
    }
    /** Candidates in display order, with the strongest matches first. */
    val ordered: List<SearchCandidate> = candidates.sortedWith(candidateOrder(railCollator()))
    return SearchState.Results(
        items = ordered.take(SEARCH_RESULT_LIMIT).map { candidate -> candidate.result },
        truncated = ordered.size > SEARCH_RESULT_LIMIT,
    )
}
