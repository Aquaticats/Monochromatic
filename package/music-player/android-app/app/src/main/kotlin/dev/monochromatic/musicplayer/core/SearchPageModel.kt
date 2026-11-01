// Pure decisions for the Search page: status copy, TalkBack labels, emphasis ranges, and the query rules
// for entering and returning to Search. Nothing here touches Compose, so every rule is unit-tested on the JVM.

// What:     `package dev.monochromatic.musicplayer.core` places the page model beside the search results.
// Why:      The status and label rules read `SearchState` and `SearchResult` directly, so they share the package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `const val SEARCH_PROMPT_TITLE: String` is the headline shown before any query is typed.
// Why:      An empty query is an invitation, not a failed lookup, so it gets its own copy (D70).
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_PROMPT_TITLE = "Search your music";
// ```
/** Headline of the unqueried prompt (D70). */
const val SEARCH_PROMPT_TITLE: String = "Search your music"

// What:     `const val SEARCH_PROMPT_DETAIL: String` explains what the prompt searches.
// Why:      The review text pairs the prompt with this line so the user knows Search covers the library.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_PROMPT_DETAIL = "Type a name to explore your library.";
// ```
/** Supporting line under the unqueried prompt (D70). */
const val SEARCH_PROMPT_DETAIL: String = "Type a name to explore your library."

// What:     `const val SEARCH_NO_MATCH_PREFIX: String` begins the completed no-match headline.
// Why:      The headline names the entered query after this prefix, and the prefix is the only fixed text in it.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_NO_MATCH_PREFIX = "No results for “";
// ```
/** Opening text of the completed no-match headline, before the entered query. */
const val SEARCH_NO_MATCH_PREFIX: String = "No results for “"

// What:     `const val SEARCH_NO_MATCH_SUFFIX: String` closes the no-match headline after the query.
// Why:      Curly quotes match the review text, and the query stays visibly delimited.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_NO_MATCH_SUFFIX = "”";
// ```
/** Closing text of the completed no-match headline, after the entered query. */
const val SEARCH_NO_MATCH_SUFFIX: String = "”"

// What:     `const val SEARCH_NO_MATCH_DETAIL: String` is the support line under a completed no-match.
// Why:      D70 keeps this support line with the query named in the headline.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_NO_MATCH_DETAIL = "Try another name.";
// ```
/** Supporting line under a completed no-match headline (D70). */
const val SEARCH_NO_MATCH_DETAIL: String = "Try another name."

// What:     `const val SEARCH_UNAVAILABLE_TITLE: String` headlines a known inability to search the source.
// Why:      D71 rejects the debug fixture's "Library unavailable" because no owner can promise a recovery.
//           This wording names only the known state, so it makes no recovery promise and names no cause.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_UNAVAILABLE_TITLE = "Search is unavailable";
// ```
/** Headline for a known source failure, with no cause and no recovery promise (D71). */
const val SEARCH_UNAVAILABLE_TITLE: String = "Search is unavailable"

// What:     `const val SEARCH_UNAVAILABLE_DETAIL: String` is the support line under the unavailable headline.
// Why:      It states only what is known, so the still-visible browser and deck are not described as broken.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_UNAVAILABLE_DETAIL = "The current library can't be searched right now.";
// ```
/** Supporting line under the unavailable headline, stating only the known fact (D71). */
const val SEARCH_UNAVAILABLE_DETAIL: String = "The current library can't be searched right now."

// What:     `const val SEARCH_EMPTY_INVENTORY_TITLE: String` headlines a usable source that holds no tracks.
// Why:      A confirmed empty inventory is distinct from a finished no-match (D69), so it is not worded as one.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_EMPTY_INVENTORY_TITLE = "No music found in your audio library.";
// ```
/** Headline while the library is still being scanned and nothing has matched yet (D69, D111). */
const val SEARCH_SCANNING_TITLE: String = "Still looking through your library"

/** Supporting line under the scanning headline: the list fills in by itself, so no retry is needed (D111). */
const val SEARCH_SCANNING_DETAIL: String = "Results appear here as more music is found."

/** Headline for a confirmed empty searchable inventory (D69). */
const val SEARCH_EMPTY_INVENTORY_TITLE: String = "No music found in your audio library."

// What:     `const val SEARCH_FIELD_LABEL: String` names the query field for assistive technology.
// Why:      The field's accessible name stays "Search music" while the typed query is its separate value (D75).
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_FIELD_LABEL = "Search music";
// ```
/** Accessible name of the query field, separate from its current value (D75). */
const val SEARCH_FIELD_LABEL: String = "Search music"

// What:     `const val SEARCH_BACK_LABEL: String` names the visible Back control.
// Why:      Back returns to the player (D64), and the spoken name must say so.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_BACK_LABEL = "Back to player";
// ```
/** Accessible name of the visible Back control, which returns to the player (D64). */
const val SEARCH_BACK_LABEL: String = "Back to player"

// What:     `const val SEARCH_CLEAR_LABEL: String` names the Clear control.
// Why:      Clear erases the query only, so its spoken name says it clears the search (D65).
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_CLEAR_LABEL = "Clear search";
// ```
/** Accessible name of the Clear control (D65). */
const val SEARCH_CLEAR_LABEL: String = "Clear search"

// What:     `const val SEARCH_FOLDER_ACTION: String` is the spoken action for a folder result.
// Why:      A folder result opens its folder without autoplay (D72), and the action label must say that.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_FOLDER_ACTION = "Open folder without autoplay";
// ```
/** Spoken action of a folder result, which opens the folder without starting playback (D72). */
const val SEARCH_FOLDER_ACTION: String = "Open folder without autoplay"

// What:     `const val SEARCH_TRACK_SELECT_ACTION: String` is the spoken action for a track that is not current.
// Why:      A different track is only selected and loaded paused (D127), so the action label is Select track (D77);
//           with Auto-play on select on (D129) it starts too and the label is Play track again.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_TRACK_SELECT_ACTION = "Select track";
// ```
/** Spoken action of a track result that is not the current track (D127, D77). */
const val SEARCH_TRACK_SELECT_ACTION: String = "Select track"

// What:     `const val SEARCH_TRACK_PLAY_ACTION: String` is the spoken action for the paused current track.
// Why:      The current track toggles playback (D127), and while it is paused the toggle starts it (D77).
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_TRACK_PLAY_ACTION = "Play track";
// ```
/** Spoken action of the current track's result while it is paused (D127, D77). */
const val SEARCH_TRACK_PLAY_ACTION: String = "Play track"

// What:     `const val SEARCH_TRACK_PAUSE_ACTION: String` is the spoken action for the playing current track.
// Why:      The current track toggles playback, and while it plays the toggle pauses it (D73, D77).
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_TRACK_PAUSE_ACTION = "Pause track";
// ```
/** Spoken action of the current track while it plays (D73, D77). */
const val SEARCH_TRACK_PAUSE_ACTION: String = "Pause track"

// What:     `const val SEARCH_TRACK_TOGGLE_ACTION: String` is the action when the playing state is unknown.
// Why:      Claiming Play or Pause without knowing the state would be false, so the label names the toggle only.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_TRACK_TOGGLE_ACTION = "Play or pause track";
// ```
/** Spoken action of the current track when its playing state is not known to the page. */
const val SEARCH_TRACK_TOGGLE_ACTION: String = "Play or pause track"

// What:     `const val SEARCH_FOLDER_KIND: String` is the spoken kind word of a folder result.
// Why:      The kind must be spoken once, because the decorative icon alone is not an accessible cue (D77).
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_FOLDER_KIND = "Folder";
// ```
/** Spoken kind word of a folder result (D77). */
const val SEARCH_FOLDER_KIND: String = "Folder"

// What:     `const val SEARCH_TRACK_KIND: String` is the spoken kind word of a track result.
// Why:      Like the folder kind, it is spoken once so the icon is not an extra accessibility stop (D77).
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_TRACK_KIND = "Track";
// ```
/** Spoken kind word of a track result (D77). */
const val SEARCH_TRACK_KIND: String = "Track"

// What:     `data class SearchStatusLine(...)` is one headline with an optional support line.
// Why:      The page draws the same two-line status anatomy for every non-result state.
//
// In TS you'd write (pseudocode):
// ```ts
// type SearchStatusLine = { title: string; detail: string | null };
// ```
/** A status headline with an optional support line. */
data class SearchStatusLine(
    /** Headline text of the status. */
    val title: String,
    /** Support text under the headline, or null when the status needs only a headline. */
    val detail: String?,
)

// What:     `data class SearchRowSemantics(...)` holds the spoken pieces of one result row.
// Why:      The page merges the row into one stop, so kind, state, and action are kept as separate fields.
//
// In TS you'd write (pseudocode):
// ```ts
// type SearchRowSemantics = { kindLabel: string; actionLabel: string; stateLabel: string | null };
// ```
/** The spoken kind, action, and optional playback state of one result row. */
data class SearchRowSemantics(
    /** Kind word of the result, spoken once. */
    val kindLabel: String,
    /** Action the row performs when activated. */
    val actionLabel: String,
    /** Playback state of the row when it is the current track, or null otherwise. */
    val stateLabel: String?,
)

// What:     `fun searchStatusLine(state: SearchState, complete: Boolean): SearchStatusLine?` picks the copy for a
//           non-result state.
// Why:      Each state needs its own truthful wording (D69 to D71), and a result list has no status line.
//
// In TS you'd write (pseudocode):
// ```ts
// function searchStatusLine(state: SearchState): StatusLine | null { /* switch on state.kind */ }
// ```
/**
 * Returns the headline and support line for a Search state, or null when the state draws a result list.
 *
 * An unavailable source is worded without a cause or recovery (D71). A completed no-match names the query (D70).
 * While the scan is unfinished ([complete] is false), a no-match and an empty inventory are not verdicts: the page
 * says it is still looking and that results will appear by themselves (D69, D111). It never hides the query field or
 * disables Search.
 */
fun searchStatusLine(state: SearchState, complete: Boolean = true): SearchStatusLine? =
    when {
        !complete && (state is SearchState.NoMatch || state == SearchState.EmptyInventory) ->
            SearchStatusLine(title = SEARCH_SCANNING_TITLE, detail = SEARCH_SCANNING_DETAIL)
        else -> completedStatusLine(state)
    }

/** Returns the status line of a state that is a verdict, or null when the state draws a result list. */
private fun completedStatusLine(state: SearchState): SearchStatusLine? =
    when (state) {
        SearchState.EmptyQuery -> SearchStatusLine(title = SEARCH_PROMPT_TITLE, detail = SEARCH_PROMPT_DETAIL)
        SearchState.EmptyInventory -> SearchStatusLine(title = SEARCH_EMPTY_INVENTORY_TITLE, detail = null)
        SearchState.Unavailable -> SearchStatusLine(
            title = SEARCH_UNAVAILABLE_TITLE,
            detail = SEARCH_UNAVAILABLE_DETAIL,
        )
        is SearchState.NoMatch -> SearchStatusLine(
            title = SEARCH_NO_MATCH_PREFIX + state.query + SEARCH_NO_MATCH_SUFFIX,
            detail = SEARCH_NO_MATCH_DETAIL,
        )
        is SearchState.Results -> null
    }

// What:     `fun searchRowSecondLine(result: SearchResult): String?` gives the parent folder line of a row.
// Why:      A track names its parent folder (D77). A root-level track has no parent folder and so gets no second
//           line, and a folder has no second line either.
//
// In TS you'd write (pseudocode):
// ```ts
// const searchRowSecondLine = (r) => r.kind === "track" ? r.parentContext : null;
// ```
/** Returns the parent folder of a track as its second line, or null for a folder or a root-level track. */
fun searchRowSecondLine(result: SearchResult): String? =
    when (result.kind) {
        SearchResultKind.FOLDER -> null
        SearchResultKind.TRACK -> result.parentContext
    }

// What:     `fun searchRowKindLabel(result: SearchResult): String` gives the spoken kind word of a row.
// Why:      The kind is spoken once in the merged row (D77), so the decorative icon carries this label.
//
// In TS you'd write (pseudocode):
// ```ts
// const searchRowKindLabel = (r) => r.kind === "folder" ? "Folder" : "Track";
// ```
/** Returns the spoken kind word, Folder or Track, for a result. */
fun searchRowKindLabel(result: SearchResult): String =
    when (result.kind) {
        SearchResultKind.FOLDER -> SEARCH_FOLDER_KIND
        SearchResultKind.TRACK -> SEARCH_TRACK_KIND
    }

// What:     `fun isCurrentTrackResult(result: SearchResult, currentTrackIndex: Int?): Boolean` finds the playing row.
// Why:      A track result is the current track when its library index matches the player's index (D73).
//
// In TS you'd write (pseudocode):
// ```ts
// const isCurrentTrackResult = (r, idx) => r.kind === "track" && idx !== null && r.indexInLibrary === idx;
// ```
/** Reports whether a result is the track the player currently holds. */
fun isCurrentTrackResult(result: SearchResult, currentTrackIndex: Int?): Boolean {
    if (result.kind != SearchResultKind.TRACK) {
        return false
    }
    if (currentTrackIndex == null) {
        return false
    }
    return result.indexInLibrary == currentTrackIndex
}

// What:     `fun searchRowActionLabel(...)` names what activating a row does.
// Why:      D77 requires the real action in the spoken row; the current track's label depends on its playing state.
//
// In TS you'd write (pseudocode):
// ```ts
// const searchRowActionLabel = (r, idx, playing) => ...; // folder, select, play, pause, or toggle
// ```
/**
 * Returns the spoken action of a result row.
 *
 * A folder opens without autoplay. Another track is selected, or played when [autoPlayOnSelect] is on (D129). The
 * current track pauses while playing and plays while paused. When the page
 * does not know the playing state, the current track is labelled as a toggle rather than a claimed state.
 */
fun searchRowActionLabel(
    result: SearchResult,
    currentTrackIndex: Int?,
    isPlaying: Boolean?,
    autoPlayOnSelect: Boolean = false,
): String {
    if (result.kind == SearchResultKind.FOLDER) {
        return SEARCH_FOLDER_ACTION
    }
    if (!isCurrentTrackResult(result, currentTrackIndex)) {
        return if (autoPlayOnSelect) SEARCH_TRACK_PLAY_ACTION else SEARCH_TRACK_SELECT_ACTION
    }
    return when (isPlaying) {
        true -> SEARCH_TRACK_PAUSE_ACTION
        false -> SEARCH_TRACK_PLAY_ACTION
        null -> SEARCH_TRACK_TOGGLE_ACTION
    }
}

// What:     `fun searchRowStateLabel(...)` names the current-track state, or returns null for other rows.
// Why:      The state is structured speech (D77), kept apart from the title so it is not a copied sentence.
//
// In TS you'd write (pseudocode):
// ```ts
// const searchRowStateLabel = (r, idx, playing) => isCurrent ? (playing ? "Current track, playing" : ...) : null;
// ```
/** Returns the playback state of the current track row, or null when the row is not the current track. */
fun searchRowStateLabel(result: SearchResult, currentTrackIndex: Int?, isPlaying: Boolean?): String? {
    if (!isCurrentTrackResult(result, currentTrackIndex)) {
        return null
    }
    return when (isPlaying) {
        true -> "Current track, playing"
        false -> "Current track, paused"
        null -> "Current track"
    }
}

// What:     `fun searchRowSemantics(...)` bundles the spoken kind, action, and state of one row.
// Why:      The page sets these three values on one merged node, so they are computed in one place.
//
// In TS you'd write (pseudocode):
// ```ts
// const searchRowSemantics = (r, idx, playing) => ({ kindLabel, actionLabel, stateLabel });
// ```
/** Bundles the spoken kind, action, and current-track state of one result row. */
fun searchRowSemantics(
    result: SearchResult,
    currentTrackIndex: Int?,
    isPlaying: Boolean?,
    autoPlayOnSelect: Boolean = false,
): SearchRowSemantics =
    SearchRowSemantics(
        kindLabel = searchRowKindLabel(result),
        actionLabel = searchRowActionLabel(result, currentTrackIndex, isPlaying, autoPlayOnSelect),
        stateLabel = searchRowStateLabel(result, currentTrackIndex, isPlaying),
    )

// What:     `fun searchEmphasisSpans(text: String, highlights: List<SearchHighlight>): List<SearchHighlight>`
//           passes highlight ranges through when they fit the text.
// Why:      A range outside the text would crash the annotated string, so each range is checked against the text.
//
// In TS you'd write (pseudocode):
// ```ts
// const searchEmphasisSpans = (text, hs) => hs.filter(h => h.start >= 0 && h.start < h.end && h.end <= text.length);
// ```
/** Keeps the highlight ranges that are non-empty and lie inside the text, in their original order. */
fun searchEmphasisSpans(text: String, highlights: List<SearchHighlight>): List<SearchHighlight> =
    highlights.filter { highlight ->
        highlight.start >= 0 && highlight.start < highlight.end && highlight.end <= text.length
    }

// What:     `fun initialQueryForVisit(previous: String?, sameVisit: Boolean): String` picks the query at entry.
// Why:      A new visit starts empty (D66), while a same visit keeps what the user typed.
//
// In TS you'd write (pseudocode):
// ```ts
// const initialQueryForVisit = (prev, same) => same ? (prev ?? "") : "";
// ```
/** Returns the query a Search visit starts with: empty for a new visit, and the kept query within one visit. */
fun initialQueryForVisit(previous: String?, sameVisit: Boolean): String {
    if (!sameVisit) {
        return ""
    }
    return previous.orEmpty()
}

// What:     `fun shouldRequestQueryFocus(sameVisit: Boolean): Boolean` says whether entry focuses the field.
// Why:      Opening Search requests edit focus and a keyboard (D63), but a same-visit recomposition does not.
//
// In TS you'd write (pseudocode):
// ```ts
// const shouldRequestQueryFocus = (same) => !same;
// ```
/** Reports whether the query field should take focus, which happens only when a new visit opens. */
fun shouldRequestQueryFocus(sameVisit: Boolean): Boolean = !sameVisit

// What:     `fun shouldPreserveRowVisibility(previousQuery: String, currentQuery: String): Boolean` compares queries.
// Why:      Refocusing without a query change must keep the intended row visible (D68), and a changed query resets it.
//
// In TS you'd write (pseudocode):
// ```ts
// const shouldPreserveRowVisibility = (prev, cur) => prev === cur;
// ```
/** Reports whether the result position should be kept, which holds only when the query text is unchanged. */
fun shouldPreserveRowVisibility(previousQuery: String, currentQuery: String): Boolean = previousQuery == currentQuery

// What:     `fun isClearVisible(query: String): Boolean` says whether the Clear control appears.
// Why:      Clear is shown only when there is something to clear (D65 and the header rule).
//
// In TS you'd write (pseudocode):
// ```ts
// const isClearVisible = (q) => q.trim().length > 0;
// ```
/** Reports whether the Clear control is shown, which requires a query that is not blank. */
fun isClearVisible(query: String): Boolean = query.isNotBlank()

// What:     `const val SEARCH_RESTORED_RESULT_INDEX: Int = 0` is the position a restored query starts from.
// Why:      If a restored query ever returns, its results start at the top rather than a deep position (D67).
//
// In TS you'd write (pseudocode):
// ```ts
// export const SEARCH_RESTORED_RESULT_INDEX = 0;
// ```
/** First result index, which is where a restored query begins (D67). */
const val SEARCH_RESTORED_RESULT_INDEX: Int = 0

// What:     `fun searchStateFor(query: String, answer: SearchAnswer?, hasService: Boolean): SearchState` picks what the
//           page draws for the typed query.
// Why:      A blank query shows the invitation at once, without waiting for the worker (D70). A query whose answer has
//           not arrived keeps drawing an empty list, not a verdict, because no evidence about it exists yet (D69).
//
// In TS you'd write (pseudocode):
// ```ts
// const searchStateFor = (q: string, answer: SearchAnswer | null, hasService: boolean): SearchState => { /* ... */ };
// ```
/**
 * Returns the state the Search page draws for [query] given the newest [answer] of this visit.
 *
 * Without a service the page is unavailable (D71). A blank query is the invitation. A query that has no answer yet,
 * or only an answer to a blank query, draws an empty result list until its answer arrives.
 */
fun searchStateFor(query: String, answer: SearchAnswer?, hasService: Boolean): SearchState {
    if (!hasService) {
        return SearchState.Unavailable
    }
    if (searchTerms(query).isEmpty()) {
        return SearchState.EmptyQuery
    }
    /** The answer's state, unless it answered a blank query. */
    val answered: SearchState? = answer?.state?.takeIf { state -> state != SearchState.EmptyQuery }
    return answered ?: SearchState.Results(items = emptyList(), truncated = false)
}

// What:     `fun playingResultId(state: SearchState, playingUri: String?): Int?` finds the result row that is playing.
// Why:      A result's number is its number in the Search index, not its position in the library, so the playing track
//           is found by URI among the rows that are shown, which are at most `SEARCH_RESULT_LIMIT`.
//
// In TS you'd write (pseudocode):
// ```ts
// const playingResultId = (state: SearchState, uri: string | null) => items(state).find((r) => r.trackUri === uri)?.id;
// ```
/** Returns the index-number of the shown track result whose URI is [playingUri], or null when none is shown. */
fun playingResultId(state: SearchState, playingUri: String?): Int? {
    if (playingUri == null || state !is SearchState.Results) {
        return null
    }
    return state.items.firstOrNull { result ->
        result.kind == SearchResultKind.TRACK && result.trackUri == playingUri
    }?.indexInLibrary
}
