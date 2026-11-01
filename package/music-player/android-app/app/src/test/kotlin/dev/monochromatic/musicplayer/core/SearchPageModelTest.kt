// Cases for the pure Search page model: status copy, row labels, emphasis ranges, and visit query rules.

// What:     `package dev.monochromatic.musicplayer.core` places these tests beside the model they cover.
// Why:      The tests reach the public model functions and the result types without a package hop.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     JUnit's assertions and test annotation register each case.
// Why:      Each case compares one value, so every copy string and branch is checked exactly.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `private fun folderResult(name: String): SearchResult` builds a folder result for the cases.
// Why:      Each case needs only the fields that the model reads, so the fixture stays small and explicit.
//
// In TS you'd write (pseudocode):
// ```ts
// const folderResult = (name: string): SearchResult => ({ kind: "folder", name, /* ... */ });
// ```
/** Builds a folder result whose own name is the given text. */
private fun folderResult(name: String): SearchResult = SearchResult(
    kind = SearchResultKind.FOLDER,
    name = name,
    parentContext = null,
    indexInLibrary = null,
    folderName = name,
    tier = 0,
    nameHighlights = emptyList(),
    parentHighlights = emptyList(),
)

// What:     `private fun trackResult(...)` builds a track result for the cases.
// Why:      Track cases vary by parent and library index, which are the two fields the labels depend on.
//
// In TS you'd write (pseudocode):
// ```ts
// const trackResult = (name, parent, index) => ({ kind: "track", name, parentContext: parent, indexInLibrary: index });
// ```
/** Builds a track result with the given title, parent context, and library position. */
private fun trackResult(name: String, parent: String?, index: Int): SearchResult = SearchResult(
    kind = SearchResultKind.TRACK,
    name = name,
    parentContext = parent,
    indexInLibrary = index,
    folderName = parent,
    tier = 0,
    nameHighlights = emptyList(),
    parentHighlights = emptyList(),
)

// What:     `class SearchPageModelTest` groups the status, label, emphasis, and visit cases.
// Why:      One class keeps each pure page rule next to its counterpart so gaps show up in one place.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("SearchPageModel", () => { /* cases */ });
// ```
/** Verifies every copy string, label, emphasis range, and visit query rule the Search page decides. */
class SearchPageModelTest {
    // What:     `@Test fun promptStatusUsesReviewCopy()` checks the unqueried prompt.
    // Why:      The prompt is an invitation, and its wording is fixed by the review text (D70).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("prompt uses review copy", () => { /* ... */ });
    // ```
    /** The empty query shows the prompt headline and its explanation. */
    @Test
    fun promptStatusUsesReviewCopy() {
        /** Status line produced for an empty query. */
        val line: SearchStatusLine? = searchStatusLine(SearchState.EmptyQuery)
        assertEquals("Search your music", line?.title)
        assertEquals("Type a name to explore your library.", line?.detail)
    }

    // What:     `@Test fun emptyInventoryStatusHasHeadlineOnly()` checks the confirmed empty inventory.
    // Why:      A confirmed empty inventory is not a finished no-match, so it carries no no-match support line (D69).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty inventory has a headline only", () => { /* ... */ });
    // ```
    /** A confirmed empty inventory shows its headline and no support line. */
    @Test
    fun emptyInventoryStatusHasHeadlineOnly() {
        /** Status line produced for a confirmed empty inventory. */
        val line: SearchStatusLine? = searchStatusLine(SearchState.EmptyInventory)
        assertEquals("No music found in your audio library.", line?.title)
        assertNull(line?.detail)
    }

    // What:     `@Test fun unavailableStatusNamesNoCause()` checks the known source failure.
    // Why:      D71 rejects "Library unavailable" and any recovery promise, so the copy names only the known fact.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unavailable status makes no recovery promise", () => { /* ... */ });
    // ```
    /** A known source failure shows neutral copy that avoids the rejected fixture wording. */
    @Test
    fun unavailableStatusNamesNoCause() {
        /** Status line produced for a known source failure. */
        val line: SearchStatusLine? = searchStatusLine(SearchState.Unavailable)
        assertEquals("Search is unavailable", line?.title)
        assertEquals("The current library can't be searched right now.", line?.detail)
        /** Headline text of the unavailable status, or an empty string when no status line exists. */
        val headline: String = line?.title ?: ""
        assertFalse(headline.contains("Library unavailable"))
    }

    // What:     `@Test fun noMatchStatusNamesQuery()` checks the completed no-match headline.
    // Why:      D70 requires the headline to name the entered query, with the try-another-name support line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no match names the query", () => { /* ... */ });
    // ```
    /** A completed no-match headline names the query inside curly quotes. */
    @Test
    fun noMatchStatusNamesQuery() {
        /** Status line produced for a completed no-match on the query zzq. */
        val line: SearchStatusLine? = searchStatusLine(SearchState.NoMatch("zzq"))
        assertEquals("No results for “zzq”", line?.title)
        assertEquals("Try another name.", line?.detail)
    }

    // What:     `@Test fun resultsHaveNoStatusLine()` checks that a result list draws no status.
    // Why:      Status copy would hide the list, so Results must return null.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("results have no status line", () => { /* ... */ });
    // ```
    /** A result list produces no status line. */
    @Test
    fun resultsHaveNoStatusLine() {
        assertNull(searchStatusLine(SearchState.Results(items = emptyList(), truncated = false)))
    }

    // What:     `@Test fun folderSecondLineIsAbsent()` checks that a folder row has no context line.
    // Why:      A folder has no parent folder to name, so no second line is drawn for it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder has no second line", () => { /* ... */ });
    // ```
    /** A folder result returns no second line. */
    @Test
    fun folderSecondLineIsAbsent() {
        assertNull(searchRowSecondLine(folderResult("Camellia")))
    }

    // What:     `@Test fun trackSecondLineIsParentFolder()` checks that a track names its folder.
    // Why:      D77 uses the parent folder as the disambiguating second line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("track second line is its parent", () => { /* ... */ });
    // ```
    /** A track result returns its parent folder as the second line. */
    @Test
    fun trackSecondLineIsParentFolder() {
        assertEquals("Camellia/Live", searchRowSecondLine(trackResult("Night", "Camellia/Live", 3)))
    }

    // What:     `@Test fun rootTrackHasNoSecondLine()` checks a track at the library root.
    // Why:      A root-level track has no parent folder, so no second line is invented for it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("root track has no second line", () => { /* ... */ });
    // ```
    /** A root-level track returns no second line. */
    @Test
    fun rootTrackHasNoSecondLine() {
        assertNull(searchRowSecondLine(trackResult("Night", null, 0)))
    }

    // What:     `@Test fun rootTrackSemanticsNamesNoLibraryRoot()` checks the spoken row for a root-level track.
    // Why:      The spoken row must not describe a parent that does not exist.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("root track speech", () => { /* ... */ });
    // ```
    /** A root-level track speaks its kind and action only, with no library-root wording. */
    @Test
    fun rootTrackSemanticsNamesNoLibraryRoot() {
        /** Spoken pieces of a root-level track that is not the current track. */
        val semantics: SearchRowSemantics = searchRowSemantics(
            trackResult("Night", null, 0),
            currentTrackIndex = null,
            isPlaying = null,
        )
        assertEquals("Track", semantics.kindLabel)
        assertEquals("Select track", semantics.actionLabel)
        assertNull(semantics.stateLabel)
    }

    // What:     `@Test fun kindLabelsNameFolderAndTrack()` checks the spoken kind words.
    // Why:      The kind is spoken once in the row (D77), so each kind needs its own word.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("kind labels", () => { /* ... */ });
    // ```
    /** Folder and track results speak the words Folder and Track. */
    @Test
    fun kindLabelsNameFolderAndTrack() {
        assertEquals("Folder", searchRowKindLabel(folderResult("Camellia")))
        assertEquals("Track", searchRowKindLabel(trackResult("Night", "Live", 1)))
    }

    // What:     `@Test fun currentTrackMatchesLibraryIndex()` checks the current-track test for a track.
    // Why:      A track result is the playing track only when its library index equals the player's index (D73).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("current track matches its index", () => { /* ... */ });
    // ```
    /** A track whose library index equals the current index is the current track. */
    @Test
    fun currentTrackMatchesLibraryIndex() {
        assertTrue(isCurrentTrackResult(trackResult("Night", "Live", 4), currentTrackIndex = 4))
        assertFalse(isCurrentTrackResult(trackResult("Night", "Live", 4), currentTrackIndex = 5))
    }

    // What:     `@Test fun folderIsNeverCurrentTrack()` checks that a folder cannot be the current track.
    // Why:      Only tracks carry a library index, so a folder must never take the toggle action.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder is not the current track", () => { /* ... */ });
    // ```
    /** A folder result is never the current track, even when an index is supplied. */
    @Test
    fun folderIsNeverCurrentTrack() {
        assertFalse(isCurrentTrackResult(folderResult("Camellia"), currentTrackIndex = 0))
    }

    // What:     `@Test fun noCurrentTrackMeansNoCurrentRow()` checks an absent current index.
    // Why:      With no current track, no row may claim the current-track state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no current index means no current row", () => { /* ... */ });
    // ```
    /** A null current index marks no result as the current track. */
    @Test
    fun noCurrentTrackMeansNoCurrentRow() {
        assertFalse(isCurrentTrackResult(trackResult("Night", "Live", 0), currentTrackIndex = null))
    }

    // What:     `@Test fun folderActionOpensWithoutAutoplay()` checks the folder action label.
    // Why:      A folder result opens its folder without starting playback (D72).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder action", () => { /* ... */ });
    // ```
    /** A folder action speaks Open folder without autoplay. */
    @Test
    fun folderActionOpensWithoutAutoplay() {
        assertEquals(
            "Open folder without autoplay",
            searchRowActionLabel(folderResult("Camellia"), currentTrackIndex = 2, isPlaying = true),
        )
    }

    // What:     `@Test fun otherTrackActionSelects()` checks a track that is not current.
    // Why:      A different track is only selected (D127), so its label is Select track and not Play track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("other track selects", () => { /* ... */ });
    // ```
    /** A track that is not the current track speaks Select track. */
    @Test
    fun otherTrackActionSelects() {
        assertEquals(
            "Select track",
            searchRowActionLabel(trackResult("Night", "Live", 1), currentTrackIndex = 2, isPlaying = true),
        )
    }

    // What:     `@Test fun otherTrackActionPlaysWithAutoPlayOn()` checks a track that is not current, setting on.
    // Why:      With Auto-play on select on (D129) a result starts its track, so the label is Play track again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("other track plays with auto-play", () => { /* ... */ });
    // ```
    /** A track that is not the current track speaks Play track when Auto-play on select is on. */
    @Test
    fun otherTrackActionPlaysWithAutoPlayOn() {
        assertEquals(
            "Play track",
            searchRowActionLabel(
                trackResult("Night", "Live", 1),
                currentTrackIndex = 2,
                isPlaying = true,
                autoPlayOnSelect = true,
            ),
        )
        assertEquals(
            "Play track",
            searchRowSemantics(
                trackResult("Night", "Live", 1),
                currentTrackIndex = null,
                isPlaying = null,
                autoPlayOnSelect = true,
            ).actionLabel,
        )
    }

    /** The setting changes only the label of a track that is not current. */
    @Test
    fun autoPlayDoesNotChangeTheFolderOrCurrentTrackLabels() {
        assertEquals(
            "Open folder without autoplay",
            searchRowActionLabel(
                folderResult("Camellia"),
                currentTrackIndex = 2,
                isPlaying = true,
                autoPlayOnSelect = true,
            ),
        )
        assertEquals(
            "Pause track",
            searchRowActionLabel(
                trackResult("Night", "Live", 2),
                currentTrackIndex = 2,
                isPlaying = true,
                autoPlayOnSelect = true,
            ),
        )
    }

    // What:     `@Test fun playingCurrentTrackActionPauses()` checks the current track while it plays.
    // Why:      The current-track toggle pauses while playing (D73 and D77).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("playing current track pauses", () => { /* ... */ });
    // ```
    /** The current track speaks Pause track while playing. */
    @Test
    fun playingCurrentTrackActionPauses() {
        assertEquals(
            "Pause track",
            searchRowActionLabel(trackResult("Night", "Live", 2), currentTrackIndex = 2, isPlaying = true),
        )
    }

    // What:     `@Test fun pausedCurrentTrackActionPlays()` checks the current track while paused.
    // Why:      The current-track toggle resumes while paused, so the label is Play track (D77).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("paused current track plays", () => { /* ... */ });
    // ```
    /** The current track speaks Play track while paused. */
    @Test
    fun pausedCurrentTrackActionPlays() {
        assertEquals(
            "Play track",
            searchRowActionLabel(trackResult("Night", "Live", 2), currentTrackIndex = 2, isPlaying = false),
        )
    }

    // What:     `@Test fun unknownStateCurrentTrackActionToggles()` checks the current track without state.
    // Why:      When the playing state is not known, the label names the toggle instead of a claimed state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unknown state toggles", () => { /* ... */ });
    // ```
    /** The current track speaks the toggle wording when its playing state is not known. */
    @Test
    fun unknownStateCurrentTrackActionToggles() {
        assertEquals(
            "Play or pause track",
            searchRowActionLabel(trackResult("Night", "Live", 2), currentTrackIndex = 2, isPlaying = null),
        )
    }

    // What:     `@Test fun stateLabelIsNullForOtherRows()` checks that only the current row has a state.
    // Why:      Structured state speech belongs to the current track alone (D77).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("state only on current row", () => { /* ... */ });
    // ```
    /** Folder and non-current track rows carry no playback state. */
    @Test
    fun stateLabelIsNullForOtherRows() {
        assertNull(searchRowStateLabel(folderResult("Camellia"), currentTrackIndex = 2, isPlaying = true))
        assertNull(searchRowStateLabel(trackResult("Night", "Live", 1), currentTrackIndex = 2, isPlaying = true))
    }

    // What:     `@Test fun stateLabelReportsPlayingCurrentTrack()` checks the current track state wording.
    // Why:      The state is spoken apart from the title, so it names the playing state directly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("state label for playing current", () => { /* ... */ });
    // ```
    /** The current track speaks its playing or paused state. */
    @Test
    fun stateLabelReportsPlayingCurrentTrack() {
        assertEquals(
            "Current track, playing",
            searchRowStateLabel(trackResult("Night", "Live", 2), currentTrackIndex = 2, isPlaying = true),
        )
        assertEquals(
            "Current track, paused",
            searchRowStateLabel(trackResult("Night", "Live", 2), currentTrackIndex = 2, isPlaying = false),
        )
    }

    // What:     `@Test fun semanticsBundleCombinesKindActionAndState()` checks the combined row labels.
    // Why:      The page reads all three labels from one call, so the bundle must match the pieces.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("semantics bundle", () => { /* ... */ });
    // ```
    /** The row bundle carries the kind, action, and state of a current track. */
    @Test
    fun semanticsBundleCombinesKindActionAndState() {
        /** Semantics of a playing current track row. */
        val semantics: SearchRowSemantics = searchRowSemantics(
            trackResult("Night", "Live", 2),
            currentTrackIndex = 2,
            isPlaying = true,
        )
        assertEquals("Track", semantics.kindLabel)
        assertEquals("Pause track", semantics.actionLabel)
        assertEquals("Current track, playing", semantics.stateLabel)
    }

    // What:     `@Test fun emphasisKeepsValidRanges()` checks ranges that fit the text.
    // Why:      Valid ranges pass through unchanged, in their original order, for the annotated string.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("valid ranges pass through", () => { /* ... */ });
    // ```
    /** Ranges inside the text and in their original order are returned unchanged. */
    @Test
    fun emphasisKeepsValidRanges() {
        /** Two valid ranges in one title. */
        val highlights: List<SearchHighlight> = listOf(SearchHighlight(0, 3), SearchHighlight(5, 8))
        assertEquals(highlights, searchEmphasisSpans("Cam Cama", highlights))
    }

    // What:     `@Test fun emphasisDropsOutOfRangeRanges()` checks ranges that run past the text.
    // Why:      A range past the end would crash the annotated string, so it must be dropped.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("out of range dropped", () => { /* ... */ });
    // ```
    /** A range that ends past the text, or starts before it, is dropped. */
    @Test
    fun emphasisDropsOutOfRangeRanges() {
        /** A range that ends one past the last character of a three-character text. */
        val past: List<SearchHighlight> = listOf(SearchHighlight(0, 4))
        assertEquals(emptyList<SearchHighlight>(), searchEmphasisSpans("Cam", past))
        /** A range that starts before the text. */
        val before: List<SearchHighlight> = listOf(SearchHighlight(-1, 2))
        assertEquals(emptyList<SearchHighlight>(), searchEmphasisSpans("Cam", before))
    }

    // What:     `@Test fun emphasisDropsEmptyRanges()` checks zero-width ranges.
    // Why:      An empty range paints nothing, so it is not a highlight.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty range dropped", () => { /* ... */ });
    // ```
    /** A range whose start equals its end is dropped. */
    @Test
    fun emphasisDropsEmptyRanges() {
        assertEquals(emptyList<SearchHighlight>(), searchEmphasisSpans("Cam", listOf(SearchHighlight(1, 1))))
    }

    // What:     `@Test fun newVisitStartsEmpty()` checks entry after leaving Search.
    // Why:      A new visit starts with an empty query even when the previous text exists (D66).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("new visit empty", () => { /* ... */ });
    // ```
    /** A new visit ignores the previous query and starts empty. */
    @Test
    fun newVisitStartsEmpty() {
        assertEquals("", initialQueryForVisit(previous = "cam", sameVisit = false))
    }

    // What:     `@Test fun sameVisitKeepsQuery()` checks a visit that continues.
    // Why:      Within one visit the typed query stays as the user left it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("same visit keeps query", () => { /* ... */ });
    // ```
    /** A continuing visit keeps the previous query. */
    @Test
    fun sameVisitKeepsQuery() {
        assertEquals("cam", initialQueryForVisit(previous = "cam", sameVisit = true))
    }

    // What:     `@Test fun sameVisitWithoutPreviousIsEmpty()` checks a null previous query.
    // Why:      A continuing visit with no stored query must still produce an empty string, not a null.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("same visit null previous", () => { /* ... */ });
    // ```
    /** A continuing visit with no previous query starts empty. */
    @Test
    fun sameVisitWithoutPreviousIsEmpty() {
        assertEquals("", initialQueryForVisit(previous = null, sameVisit = true))
    }

    // What:     `@Test fun focusRequestedOnlyOnNewVisit()` checks the entry focus rule.
    // Why:      Opening Search requests edit focus (D63), while a continuing visit does not request it again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("focus on new visit only", () => { /* ... */ });
    // ```
    /** Edit focus is requested for a new visit and not for a continuing one. */
    @Test
    fun focusRequestedOnlyOnNewVisit() {
        assertTrue(shouldRequestQueryFocus(sameVisit = false))
        assertFalse(shouldRequestQueryFocus(sameVisit = true))
    }

    // What:     `@Test fun unchangedQueryPreservesRowVisibility()` checks same-query refocus.
    // Why:      D68 keeps the intended row visible when the query has not changed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("same query keeps row", () => { /* ... */ });
    // ```
    /** An unchanged query keeps the current result position. */
    @Test
    fun unchangedQueryPreservesRowVisibility() {
        assertTrue(shouldPreserveRowVisibility(previousQuery = "cam", currentQuery = "cam"))
    }

    // What:     `@Test fun changedQueryResetsRowVisibility()` checks a changed query.
    // Why:      A new query has new results, so the old position is not preserved.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("changed query resets row", () => { /* ... */ });
    // ```
    /** A changed query does not keep the previous result position. */
    @Test
    fun changedQueryResetsRowVisibility() {
        assertFalse(shouldPreserveRowVisibility(previousQuery = "cam", currentQuery = "cami"))
    }

    // What:     `@Test fun clearVisibleForNonBlankQuery()` checks the Clear control with text.
    // Why:      Clear appears only when there is text to remove.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("clear visible", () => { /* ... */ });
    // ```
    /** A query with visible text shows the Clear control. */
    @Test
    fun clearVisibleForNonBlankQuery() {
        assertTrue(isClearVisible("cam"))
    }

    // What:     `@Test fun clearHiddenForBlankQuery()` checks the Clear control without text.
    // Why:      A blank query has nothing to clear, so the control is hidden.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("clear hidden for blank", () => { /* ... */ });
    // ```
    /** A blank or empty query hides the Clear control. */
    @Test
    fun clearHiddenForBlankQuery() {
        assertFalse(isClearVisible(""))
        assertFalse(isClearVisible("   "))
    }

    // What:     `@Test fun restoredResultsStartAtFirstIndex()` checks the restored-query position.
    // Why:      A restored query begins at the top of its results, never a deep prior position (D67).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("restored starts at zero", () => { /* ... */ });
    // ```
    /** A restored query starts at the first result index. */
    @Test
    fun restoredResultsStartAtFirstIndex() {
        assertEquals(0, SEARCH_RESTORED_RESULT_INDEX)
    }
}

/** Verifies the scan-aware status lines and the helpers that map a Search answer to what the page draws (D69, D111). */
class SearchScanStatusTest {
    /** Builds an answer around a state. */
    private fun answer(state: SearchState, complete: Boolean = true): SearchAnswer =
        SearchAnswer(1L, "q", state, 0, complete, false, 0L, 0.0)

    /** An unfinished scan turns the two nothing-here verdicts into the scanning copy and leaves the rest alone. */
    @Test
    fun anUnfinishedScanIsNotAVerdict() {
        val scanning = searchStatusLine(SearchState.NoMatch("zz"), complete = false)
        assertEquals(SEARCH_SCANNING_TITLE, scanning?.title)
        assertEquals(SEARCH_SCANNING_DETAIL, scanning?.detail)
        assertEquals(SEARCH_SCANNING_TITLE, searchStatusLine(SearchState.EmptyInventory, complete = false)?.title)
        assertEquals(SEARCH_EMPTY_INVENTORY_TITLE, searchStatusLine(SearchState.EmptyInventory, complete = true)?.title)
        assertEquals(SEARCH_NO_MATCH_PREFIX + "zz" + SEARCH_NO_MATCH_SUFFIX, searchStatusLine(SearchState.NoMatch("zz"))?.title)
        assertEquals(SEARCH_PROMPT_TITLE, searchStatusLine(SearchState.EmptyQuery, complete = false)?.title)
        assertNull(searchStatusLine(SearchState.Results(emptyList(), false), complete = false))
    }

    /** A blank query is the invitation at once; a query without an answer draws an empty list; no service is unavailable. */
    @Test
    fun theStateDrawnFollowsTheQueryAndTheAnswer() {
        assertEquals(SearchState.EmptyQuery, searchStateFor("  ", null, hasService = true))
        assertEquals(SearchState.Unavailable, searchStateFor("a", null, hasService = false))
        assertEquals(SearchState.Results(emptyList(), false), searchStateFor("a", null, hasService = true))
        assertEquals(
            SearchState.Results(emptyList(), false),
            searchStateFor("a", answer(SearchState.EmptyQuery), hasService = true),
        )
        val noMatch = SearchState.NoMatch("a")
        assertEquals(noMatch, searchStateFor("a", answer(noMatch), hasService = true))
    }

    /** The playing row is found by URI among the shown results, whatever number the index gave it. */
    @Test
    fun thePlayingRowIsFoundByUri() {
        val rows = SearchState.Results(
            items = listOf(
                SearchResult(SearchResultKind.FOLDER, "F", null, null, "F", 0, emptyList(), emptyList()),
                SearchResult(SearchResultKind.TRACK, "t", "F", 41, "F", 1, emptyList(), emptyList(), "u-41"),
                SearchResult(SearchResultKind.TRACK, "t2", "F", 97, "F", 2, emptyList(), emptyList(), "u-97"),
            ),
            truncated = false,
        )
        assertEquals(97, playingResultId(rows, "u-97"))
        assertNull(playingResultId(rows, "u-1"))
        assertNull(playingResultId(rows, null))
        assertNull(playingResultId(SearchState.EmptyQuery, "u-97"))
    }
}
