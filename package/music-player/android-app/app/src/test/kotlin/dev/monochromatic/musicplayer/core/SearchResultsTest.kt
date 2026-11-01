// Search result cases: the four page states, scope D matching, mixed relevance order, determinism, and the limit.

// What:     `package ...core` places these tests beside the results model they exercise.
// Why:      The tests reach `searchLibrary` and its state types through the same package without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import dev.monochromatic.musicplayer.Track` brings in the library track type used by the fixtures.
// Why:      `Track` lives in the parent package, so the test imports it by absolute name.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Track } from "dev.monochromatic.musicplayer/Track";
// ```
import dev.monochromatic.musicplayer.Track

// What:     JUnit's assertions and test annotation register each result case.
// Why:      Each case compares the state kind, the ordered names, or the flags of a result list.
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

/** Verifies Search states, scope D matching, mixed ordering, determinism, and the result limit. */
class SearchResultsTest {
    // What:     `private fun libraryOf(paths: List<String>): List<Track>` builds tracks with unique URIs.
    // Why:      Each track needs its own URI so that library positions are distinct in the results.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const libraryOf = (paths: string[]): Track[] => paths.map((path, i) => ({ uri: `uri-${i}`, displayPath: path }));
    // ```
    /** Builds one library track per display path, numbering the URIs in list order. */
    private fun libraryOf(paths: List<String>): List<Track> =
        paths.mapIndexed { index, path -> Track("uri-$index", path) }

    // What:     `private fun library(vararg paths: String): List<Track>` builds a short library from literal paths.
    // Why:      Most cases list a few paths inline, and this keeps each case readable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const library = (...paths: string[]) => libraryOf(paths);
    // ```
    /** Builds a library from literal display paths. */
    private fun library(vararg paths: String): List<Track> = libraryOf(paths.toList())

    // What:     `private fun itemsOf(state: SearchState): List<SearchResult>` returns the items of a results state.
    // Why:      Ordering cases read the result list directly, and any other state fails the test loudly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const itemsOf = (state: SearchState) => (state as { items: SearchResult[] }).items;
    // ```
    /** Returns the items of a results state, failing the test when the state is not a results state. */
    private fun itemsOf(state: SearchState): List<SearchResult> = (state as SearchState.Results).items

    // What:     `private fun namesOf(items: List<SearchResult>): List<String>` lists the names of results in order.
    // Why:      Ordering assertions compare names, which read more clearly than whole result objects.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const namesOf = (items: SearchResult[]) => items.map((item) => item.name);
    // ```
    /** Lists the names of results in their current order. */
    private fun namesOf(items: List<SearchResult>): List<String> = items.map { item -> item.name }

    // What:     `private fun trackItemsOf(items: List<SearchResult>): List<SearchResult>` keeps only track results.
    // Why:      Some cases add a folder that also matches, and they need to inspect the track row alone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const trackItemsOf = (items: SearchResult[]) => items.filter((item) => item.kind === "track");
    // ```
    /** Keeps only the track results of a result list, in their order. */
    private fun trackItemsOf(items: List<SearchResult>): List<SearchResult> =
        items.filter { item -> item.kind == SearchResultKind.TRACK }

    // What:     `@Test fun blankQueryIsEmptyQuery()` checks that a blank query is the search prompt.
    // Why:      A blank query is not a failed lookup, so it must not render a no-match verdict (D70).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("blank", () => expect(searchLibrary("  ", tracks, true)).toEqual({ kind: "emptyQuery" }));
    // ```
    /** Verifies a whitespace-only query returns the empty-query state. */
    @Test
    fun blankQueryIsEmptyQuery() {
        assertEquals(SearchState.EmptyQuery, searchLibrary("   ", library("Cam.flac"), true))
    }

    // What:     `@Test fun combiningOnlyQueryIsEmptyQuery()` checks a query that normalizes to no terms.
    // Why:      Such a query matches nothing, so it must show the prompt rather than a no-match verdict.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("combining only", () => expect(searchLibrary("\u0301", tracks, true)).toEqual({ kind: "emptyQuery" }));
    // ```
    /** Verifies a query made only of a combining mark returns the empty-query state. */
    @Test
    fun combiningOnlyQueryIsEmptyQuery() {
        assertEquals(SearchState.EmptyQuery, searchLibrary("\u0301", library("Cam.flac"), true))
    }

    // What:     `@Test fun unavailableWinsOverBlankQuery()` checks that a known source failure beats the prompt.
    // Why:      An explicit source failure takes precedence over ordinary empty-query copy (D69).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unavailable first", () => expect(searchLibrary("", tracks, false)).toEqual({ kind: "unavailable" }));
    // ```
    /** Verifies an unavailable source is reported even when the query is blank. */
    @Test
    fun unavailableWinsOverBlankQuery() {
        assertEquals(SearchState.Unavailable, searchLibrary("", library("Cam.flac"), false))
    }

    // What:     `@Test fun unavailableWinsOverQuery()` checks that an unavailable source ignores the query.
    // Why:      A known failure must not be hidden behind a result list built from a stale or partial source.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unavailable", () => expect(searchLibrary("cam", tracks, false)).toEqual({ kind: "unavailable" }));
    // ```
    /** Verifies an unavailable source returns the unavailable state for a non-blank query. */
    @Test
    fun unavailableWinsOverQuery() {
        assertEquals(SearchState.Unavailable, searchLibrary("cam", library("Cam.flac"), false))
    }

    // What:     `@Test fun emptyLibraryIsEmptyInventory()` pins that a usable empty library is its own state.
    // Why:      A confirmed empty inventory is different from a finished no-match (D69), so it has its own state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty inventory", () => expect(searchLibrary("cam", [], true)).toEqual({ kind: "emptyInventory" }));
    // ```
    /** Verifies an empty library with a non-blank query returns the empty-inventory state. */
    @Test
    fun emptyLibraryIsEmptyInventory() {
        assertEquals(SearchState.EmptyInventory, searchLibrary("cam", emptyList(), true))
    }

    // What:     `@Test fun emptyLibraryKeepsHigherPrecedenceStates()` checks the states that win over the inventory.
    // Why:      An unavailable source and a blank query are reported before the empty-inventory state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("precedence", () => expect(searchLibrary("", [], true)).toEqual({ kind: "emptyQuery" }));
    // ```
    /** Verifies unavailable and blank-query states are reported ahead of an empty inventory. */
    @Test
    fun emptyLibraryKeepsHigherPrecedenceStates() {
        assertEquals(SearchState.EmptyQuery, searchLibrary("  ", emptyList(), true))
        assertEquals(SearchState.Unavailable, searchLibrary("cam", emptyList(), false))
    }

    // What:     `@Test fun nonEmptyLibraryWithoutHitIsNoMatch()` checks that a searched library stays a no-match.
    // Why:      Only a library with no tracks is an empty inventory, so tracks with no hit remain a no-match.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no match", () => expect(searchLibrary("zzq", lib, true)).toEqual({ kind: "noMatch", query: "zzq" }));
    // ```
    /** Verifies a library holding tracks that do not match returns the no-match state, not the empty inventory. */
    @Test
    fun nonEmptyLibraryWithoutHitIsNoMatch() {
        assertEquals(SearchState.NoMatch("zzq"), searchLibrary("zzq", library("Cam.flac"), true))
    }

    // What:     `@Test fun noMatchCarriesTrimmedQuery()` checks that the no-match state names the entered text.
    // Why:      The page names what was typed, and surrounding spaces are not part of the text the user meant.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("trimmed", () => expect(searchLibrary("  zzq  ", tracks, true)).toEqual({ kind: "noMatch", query: "zzq" }));
    // ```
    /** Verifies the no-match state carries the query without its surrounding whitespace. */
    @Test
    fun noMatchCarriesTrimmedQuery() {
        assertEquals(SearchState.NoMatch("zzq"), searchLibrary("  zzq  ", library("Cam.flac"), true))
    }

    // What:     `@Test fun folderOwnNameMatches()` checks that a folder whose own name matches is a folder result.
    // Why:      D60 keeps matching folders visible through their own names.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder", () => expect(kinds(searchLibrary("cam", lib, true))).toEqual(["folder"]));
    // ```
    /** Verifies a top-level folder whose own name matches appears as a folder result. */
    @Test
    fun folderOwnNameMatches() {
        assertEquals(
            listOf(SearchResultKind.FOLDER),
            itemsOf(searchLibrary("cam", library("Camellia/Waltz.flac"), true)).map { item -> item.kind },
        )
    }

    // What:     `@Test fun parentFolderMatchDoesNotProduceTracks()` checks scope D with a parent-only match.
    // Why:      The track `Another Xronixle` must not appear for `cam` just because its folder is `Camellia` (D60).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("scope D", () => expect(names(searchLibrary("cam", lib, true))).toEqual(["Camellia"]));
    // ```
    /** Verifies a track whose only match is its parent folder is not returned. */
    @Test
    fun parentFolderMatchDoesNotProduceTracks() {
        assertEquals(
            listOf("Camellia"),
            namesOf(itemsOf(searchLibrary("cam", library("Camellia/Another Xronixle.flac"), true))),
        )
    }

    // What:     `@Test fun trackOwnTitleMatchesWithParentContext()` checks a track matched by its own title.
    // Why:      A track result must name its title, kind, folder context, and library position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("track", () => expect(searchLibrary("cam", lib, true)).toEqual(/* track Cam, Albums */));
    // ```
    /** Verifies a track whose title matches is returned with its title, folder context, and position. */
    @Test
    fun trackOwnTitleMatchesWithParentContext() {
        assertEquals(
            listOf(SearchResultKind.TRACK),
            itemsOf(searchLibrary("cam", library("Albums/Cam.flac"), true)).map { item -> item.kind },
        )
        assertEquals("Cam", itemsOf(searchLibrary("cam", library("Albums/Cam.flac"), true)).single().name)
        assertEquals("Albums", itemsOf(searchLibrary("cam", library("Albums/Cam.flac"), true)).single().parentContext)
        assertEquals(0, itemsOf(searchLibrary("cam", library("Albums/Cam.flac"), true)).single().indexInLibrary)
    }

    // What:     `@Test fun unfiledTracksAreSearchedByTitle()` checks root-level tracks, which have no folder.
    // Why:      A root-level track is still a track with a title, so it must be found by that title (D77).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unfiled", () => expect(searchLibrary("cam", [{ displayPath: "Cam.flac" }], true)).toHaveLength(1));
    // ```
    /** Verifies a root-level track is found by its title, with no context line and its letter folder as owner. */
    @Test
    fun looseTracksAreSearchedByTitleAndShowTheirLetterFolder() {
        /** The single result for the root-level track. */
        val result: SearchResult = itemsOf(searchLibrary("cam", library("Cam.flac"), true)).single()
        assertEquals(SearchResultKind.TRACK, result.kind)
        assertNull(result.parentContext)
        assertEquals("C", result.folderName)
    }

    /** Verifies the virtual folder of a loose track is its upper-case initial, with accents folded, or # otherwise. */
    @Test
    fun looseTracksNameTheFolderThePlayerShows() {
        assertEquals("A", itemsOf(searchLibrary("adele", library("Adele-Meow.mp3"), true)).single().folderName)
        assertEquals("A", itemsOf(searchLibrary("aaa", library("aaa lowercase loose.mp3"), true)).single().folderName)
        assertEquals("E", itemsOf(searchLibrary("emilie", library("Émilie.flac"), true)).single().folderName)
        assertEquals("#", itemsOf(searchLibrary("song", library("1 song.flac"), true)).single().folderName)
        assertEquals("#", itemsOf(searchLibrary("初音", library("初音.flac"), true)).single().folderName)
    }

    /** Verifies the word Unfiled is no longer the name of any group of loose tracks. */
    @Test
    fun noResultNamesAnUnfiledGroup() {
        assertEquals(SearchState.NoMatch("unfiled"), searchLibrary("unfiled", library("Other.flac"), true))
        assertEquals("O", itemsOf(searchLibrary("other", library("Other.flac"), true)).single().folderName)
    }

    // What:     `@Test fun rootTrackHasNoContextHighlight()` checks that a query cannot highlight a missing context.
    // Why:      A root-level track has no context line, so the query `un` must not highlight anything there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("root no context", () => expect(searchLibrary("un", lib, true)).toMatchObject({ parentHighlights: [] }));
    // ```
    /** Verifies a root-level track matched by its title has no context highlights for the same query. */
    @Test
    fun rootTrackHasNoContextHighlight() {
        /** The single result for the root-level track whose title starts with the query. */
        val result: SearchResult = itemsOf(searchLibrary("un", library("Un Cam.flac"), true)).single()
        assertEquals(listOf(SearchHighlight(0, 2)), result.nameHighlights)
        assertNull(result.parentContext)
        assertEquals(emptyList<SearchHighlight>(), result.parentHighlights)
    }

    // What:     `@Test fun unfiledGroupIsNotAFolderResult()` checks that the root-level group is not a folder result.
    // Why:      The `Unfiled` group is a container for root-level tracks, not a folder the user can open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unfiled not folder", () =>
    //   expect(searchLibrary("unf", lib, true)).toEqual({ kind: "noMatch", query: "unf" }));
    // ```
    /** Verifies a virtual letter folder does not appear as a folder result even when its letter matches. */
    @Test
    fun letterFolderOfLooseTracksIsNotAFolderResult() {
        assertEquals(
            listOf(SearchResultKind.TRACK),
            itemsOf(searchLibrary("a", library("Adele.flac"), true)).map { item -> item.kind },
        )
    }

    /** Verifies a real folder named like a loose letter is one folder result, beside the loose track's own result. */
    @Test
    fun realFolderNamedLikeALetterIsTheOnlyFolderResult() {
        /** Results for the query "a" over a real folder A and a loose track whose letter is A. */
        val items = itemsOf(searchLibrary("a", library("A/Song.flac", "Adele.flac"), true))
        assertEquals(1, items.count { item -> item.kind == SearchResultKind.FOLDER })
        assertEquals("A", items.single { item -> item.kind == SearchResultKind.TRACK }.folderName)
    }

    // What:     `@Test fun realFolderNamedUnfiledIsAFolderResult()` checks a real folder that uses the reserved name.
    // Why:      A real folder must stay searchable by its own name, even though it shares the reserved label.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("real Unfiled", () => expect(names(searchLibrary("unf", lib, true))).toEqual(["Unfiled"]));
    // ```
    /** Verifies a real top-level folder named `Unfiled` is returned as a folder result. */
    @Test
    fun realFolderNamedUnfiledIsAFolderResult() {
        assertEquals(
            listOf("Unfiled"),
            namesOf(itemsOf(searchLibrary("unf", library("Unfiled/Song.flac"), true))),
        )
    }

    // What:     `@Test fun realFolderNamedUnfiledKeepsOnlyItsOwnFolderResult()` mixes a real folder and root tracks.
    // Why:      Adding root-level tracks must not duplicate the real folder or add the root-level group as a result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("both", () => expect(names(searchLibrary("unf", lib, true))).toEqual(["Unfiled"]));
    // ```
    /** Verifies a real folder named `Unfiled` alongside root-level tracks yields exactly one folder result. */
    @Test
    fun realFolderNamedUnfiledKeepsOnlyItsOwnFolderResult() {
        assertEquals(
            listOf("Unfiled"),
            namesOf(itemsOf(searchLibrary("unf", library("Unfiled/Song.flac", "Root.flac"), true))),
        )
    }

    // What:     `@Test fun mixedOrderFollowsTierThenLength()` checks the accepted `cam` example order.
    // Why:      The exact `Cam` track precedes the `Camellia` folder, and the later-word match comes last (D61).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("mixed", () =>
    //   expect(names(searchLibrary("cam", lib, true)))
    //     .toEqual(["Cam", "Camellia", "Camellia Waltz", "Live at Camellia"]));
    // ```
    /** Verifies folders and tracks interleave by tier, then by name length, for the accepted `cam` examples. */
    @Test
    fun mixedOrderFollowsTierThenLength() {
        assertEquals(
            listOf("Cam", "Camellia", "Camellia Waltz", "Live at Camellia"),
            namesOf(
                itemsOf(
                    searchLibrary(
                        "cam",
                        library(
                            "Other/Live at Camellia.flac",
                            "Other/Camellia Waltz.flac",
                            "Other/Cam.flac",
                            "Camellia/Interlude.flac",
                        ),
                        true,
                    ),
                ),
            ),
        )
    }

    // What:     `@Test fun tierBeatsShorterLength()` checks that a longer tier-one name ranks above a shorter
    //           tier-two name.
    // Why:      Tier comes before length, so a short later-word match never outranks a prefix match.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("tier first", () => expect(names(searchLibrary("cam", lib, true))).toEqual(["Camellia Waltz", "Live Cam"]));
    // ```
    /** Verifies a prefix match ranks above a shorter later-word match. */
    @Test
    fun tierBeatsShorterLength() {
        assertEquals(
            listOf("Camellia Waltz", "Live Cam"),
            namesOf(itemsOf(searchLibrary("cam", library("Other/Live Cam.flac", "Other/Camellia Waltz.flac"), true))),
        )
    }

    // What:     `@Test fun shorterNameFirstWithinTier()` checks that equal tiers order by normalized length.
    // Why:      Within a tier, a shorter name is the closer match, so `Cams` precedes `Camellia`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("length", () => expect(names(searchLibrary("cam", lib, true))).toEqual(["Cams", "Camellia"]));
    // ```
    /** Verifies shorter names precede longer names within the same tier. */
    @Test
    fun shorterNameFirstWithinTier() {
        assertEquals(
            listOf("Cams", "Camellia"),
            namesOf(itemsOf(searchLibrary("cam", library("A/Camellia.flac", "A/Cams.flac"), true))),
        )
    }

    // What:     `@Test fun collatorOrdersEqualLengthNames()` checks that equal-length names use the rail collator.
    // Why:      Names of the same tier and length are ordered as the folder rail orders names, not by raw code units.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("collator", () => expect(names(searchLibrary("cam", lib, true))).toEqual(["Camas", "Cambo"]));
    // ```
    /** Verifies names of equal length are ordered by the locale-aware collator. */
    @Test
    fun collatorOrdersEqualLengthNames() {
        assertEquals(
            listOf("Camas", "Cambo"),
            namesOf(itemsOf(searchLibrary("cam", library("A/Cambo.flac", "A/Camas.flac"), true))),
        )
    }

    // What:     `@Test fun equalNamesOrderByDisplayPath()` checks two tracks with the same title in different folders.
    // Why:      Equal titles are distinguished by the display path, so their order is fixed by the path (D77).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("path", () => expect(parents(searchLibrary("cam", lib, true))).toEqual(["A", "B"]));
    // ```
    /** Verifies equal titles order by their display paths, not by library position. */
    @Test
    fun equalNamesOrderByDisplayPath() {
        assertEquals(
            listOf("A", "B"),
            itemsOf(searchLibrary("cam", library("B/Cam.flac", "A/Cam.flac"), true)).map { item -> item.parentContext },
        )
    }

    // What:     `@Test fun identicalDisplayPathsOrderByLibraryIndex()` checks the final tie-break.
    // Why:      Two tracks with the same path must still come out in one fixed order, by library position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("index", () => expect(indices(searchLibrary("cam", lib, true))).toEqual([0, 1]));
    // ```
    /** Verifies tracks with identical display paths are ordered by their library positions. */
    @Test
    fun identicalDisplayPathsOrderByLibraryIndex() {
        assertEquals(
            listOf(0, 1),
            itemsOf(searchLibrary("cam", library("A/Cam.flac", "A/Cam.flac"), true))
                .map { item -> item.indexInLibrary },
        )
    }

    // What:     `@Test fun sameNameFolderAndTrackOrderByPath()` checks a folder and a track with the same name.
    // Why:      The display path decides their order, and the folder `Cam` sorts before `Cam.flac`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder and track", () => expect(kinds(searchLibrary("cam", lib, true))).toEqual(["folder", "track"]));
    // ```
    /** Verifies a folder and a track with the same name are ordered by display path. */
    @Test
    fun sameNameFolderAndTrackOrderByPath() {
        assertEquals(
            listOf(SearchResultKind.FOLDER, SearchResultKind.TRACK),
            itemsOf(searchLibrary("cam", library("Cam/Other.flac", "Cam.flac"), true)).map { item -> item.kind },
        )
    }

    // What:     `@Test fun limitCapsResultsAndFlagsTruncation()` checks one result over the limit.
    // Why:      The list must stop at the limit, and the state must say that more matches existed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("over limit", () => expect(searchLibrary("cam", big, true)).toMatchObject({ truncated: true }));
    // ```
    /** Verifies a library with one more match than the limit returns exactly the limit and a truncated flag. */
    @Test
    fun limitCapsResultsAndFlagsTruncation() {
        /** Display paths of one more track than the limit allows. */
        val paths: List<String> = (0..SEARCH_RESULT_LIMIT).map { index -> "Cam$index.flac" }
        /** The results state for the oversized library. */
        val state: SearchState.Results = searchLibrary("cam", libraryOf(paths), true) as SearchState.Results
        assertEquals(SEARCH_RESULT_LIMIT, state.items.size)
        assertTrue(state.truncated)
    }

    // What:     `@Test fun exactlyLimitIsNotTruncated()` checks a library with exactly the limit of matches.
    // Why:      Reaching the limit is not the same as exceeding it, so the flag stays false.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("at limit", () => expect(searchLibrary("cam", atLimit, true)).toMatchObject({ truncated: false }));
    // ```
    /** Verifies a library with exactly the limit of matches is not flagged as truncated. */
    @Test
    fun exactlyLimitIsNotTruncated() {
        /** Display paths of exactly as many tracks as the limit allows. */
        val paths: List<String> = (1..SEARCH_RESULT_LIMIT).map { index -> "Cam$index.flac" }
        /** The results state for the library at the limit. */
        val state: SearchState.Results = searchLibrary("cam", libraryOf(paths), true) as SearchState.Results
        assertEquals(SEARCH_RESULT_LIMIT, state.items.size)
        assertFalse(state.truncated)
    }

    // What:     `@Test fun limitKeepsStrongestMatches()` checks that the cut happens after ordering.
    // Why:      The exact `Cam` track must survive the limit even though it is the last track added.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("sort then cut", () => expect(first(searchLibrary("cam", lib, true))).toBe("Cam"));
    // ```
    /** Verifies the strongest match is kept when weaker matches fill past the limit. */
    @Test
    fun limitKeepsStrongestMatches() {
        /** Weak later-word matches that fill the limit on their own. */
        val weakPaths: List<String> = (1..SEARCH_RESULT_LIMIT).map { index -> "Live Cam$index.flac" }
        /** The library with the exact match appended last. */
        val state: SearchState.Results =
            searchLibrary("cam", libraryOf(weakPaths + "Cam.flac"), true) as SearchState.Results
        assertEquals("Cam", state.items.first().name)
        assertEquals(0, state.items.first().tier)
        assertEquals(SEARCH_RESULT_LIMIT, state.items.size)
        assertTrue(state.truncated)
    }

    // What:     `@Test fun indexInLibraryIsPreserved()` checks that each track keeps its library position.
    // Why:      Playing a result needs the position in the library list, not the position in the result list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("index", () => expect(indices(searchLibrary("cam", lib, true))).toEqual([1, 2]));
    // ```
    /** Verifies tracks in the results carry the positions they had in the library list. */
    @Test
    fun indexInLibraryIsPreserved() {
        assertEquals(
            listOf(1, 2),
            itemsOf(searchLibrary("cam", library("x/a.flac", "Cam.flac", "y/Cam.flac"), true))
                .map { item -> item.indexInLibrary },
        )
    }

    // What:     `@Test fun parentContextIsTheDisplayedFolderPath()` checks a nested track's context line.
    // Why:      The second line names the folder as displayed, including every subfolder segment (D77).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("context", () => expect(parentOf(searchLibrary("cam", lib, true))).toBe("Album/Disc 1"));
    // ```
    /** Verifies the parent context keeps every folder segment between the top-level folder and the file. */
    @Test
    fun parentContextIsTheDisplayedFolderPath() {
        /** The single track result for the nested file. */
        val result: SearchResult = itemsOf(searchLibrary("cam", library("Album/Disc 1/Cam.flac"), true)).single()
        assertEquals("Album/Disc 1", result.parentContext)
        assertEquals("Album", result.folderName)
    }

    // What:     `@Test fun parentHighlightsMarkWordStartsInContext()` checks highlights on the context line.
    // Why:      D59 emphasizes the query in the parent context too, while the parent never produces a match.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("parent highlight", () => expect(track.parentHighlights).toEqual([{ start: 4, end: 7 }]));
    // ```
    /** Verifies the query is highlighted in the parent context at its word start. */
    @Test
    fun parentHighlightsMarkWordStartsInContext() {
        /** The track result for the file inside the folder whose name also contains the query. */
        val results: List<SearchResult> = itemsOf(searchLibrary("cam", library("Mix Cam/Cam.flac"), true))
        /** The track row, with the folder row of the same query set aside. */
        val result: SearchResult = trackItemsOf(results).single()
        assertEquals(listOf(SearchHighlight(0, 3)), result.nameHighlights)
        assertEquals("Mix Cam", result.parentContext)
        assertEquals(listOf(SearchHighlight(4, 7)), result.parentHighlights)
    }

    // What:     `@Test fun folderResultHasNoTrackFields()` checks the fields a folder result leaves empty.
    // Why:      A folder has no library position and no parent context, so those fields must be null or empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder fields", () => expect(folder).toMatchObject({ parentContext: null, indexInLibrary: null }));
    // ```
    /** Verifies a folder result carries its own name, owning folder, and no context or position. */
    @Test
    fun folderResultHasNoTrackFields() {
        /** The single folder result for the library. */
        val result: SearchResult = itemsOf(searchLibrary("cam", library("Camellia/Waltz.flac"), true)).single()
        assertEquals(SearchResultKind.FOLDER, result.kind)
        assertEquals("Camellia", result.folderName)
        assertNull(result.parentContext)
        assertNull(result.indexInLibrary)
        assertEquals(emptyList<SearchHighlight>(), result.parentHighlights)
        assertEquals(listOf(SearchHighlight(0, 3)), result.nameHighlights)
    }

    // What:     `@Test fun trackFolderNameIsTheTopLevelFolder()` checks the owning folder of a nested track.
    // Why:      Selecting the owning page needs the top-level folder, which is the page the folder index names.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("owner", () => expect(searchLibrary("cam", lib, true).folderName).toBe("Top"));
    // ```
    /** Verifies a nested track's folder name is the top-level folder rather than its immediate parent. */
    @Test
    fun trackFolderNameIsTheTopLevelFolder() {
        /** The single track result for the nested file. */
        val result: SearchResult = itemsOf(searchLibrary("cam", library("Top/Sub/Cam.flac"), true)).single()
        assertEquals("Top", result.folderName)
        assertEquals("Top/Sub", result.parentContext)
    }

    // What:     `@Test fun extensionIsNotMatched()` checks that the file extension is not part of the title.
    // Why:      The title is the file name without its last extension, so `flac` must not match `Dream.flac`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("extension", () => expect(searchLibrary("flac", lib, true)).toEqual({ kind: "noMatch", query: "flac" }));
    // ```
    /** Verifies a query for the extension alone does not match a track. */
    @Test
    fun extensionIsNotMatched() {
        assertEquals(SearchState.NoMatch("flac"), searchLibrary("flac", library("Dream.flac"), true))
    }

    // What:     `@Test fun multiTermTrackMatchUsesTitleOnly()` checks that every term must match the title.
    // Why:      With two terms, the title `Live at Camellia` matches `live cam` through its first term at the start.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("multi", () => expect(searchLibrary("live cam", lib, true)).toMatchObject({ tier: 1 }));
    // ```
    /** Verifies a two-term query matches a title whose first word is the first term. */
    @Test
    fun multiTermTrackMatchUsesTitleOnly() {
        assertEquals(1, itemsOf(searchLibrary("live cam", library("Other/Live at Camellia.flac"), true)).single().tier)
    }

    // What:     `@Test fun multiTermNoMatchWhenOneTermIsMissing()` checks the AND rule across the results.
    // Why:      A result list must not show a track that only satisfies part of the query.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("AND results", () =>
    //   expect(searchLibrary("live zzz", lib, true)).toEqual({ kind: "noMatch", query: "live zzz" }));
    // ```
    /** Verifies a track satisfying only part of a multi-term query is not returned. */
    @Test
    fun multiTermNoMatchWhenOneTermIsMissing() {
        assertEquals(
            SearchState.NoMatch("live zzz"),
            searchLibrary("live zzz", library("Other/Live at Camellia.flac"), true),
        )
    }

    // What:     `@Test fun japaneseTrackTitleMatches()` checks a Japanese title that starts with the query.
    // Why:      Japanese titles use the same prefix rule, so `ラジオ` finds `ラジオ体操` at tier one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("Japanese", () => expect(searchLibrary("ラジオ", lib, true)).toMatchObject({ tier: 1 }));
    // ```
    /** Verifies a Japanese track title that starts with the query is returned at tier one. */
    @Test
    fun japaneseTrackTitleMatches() {
        /** The single track result for the Japanese title. */
        val result: SearchResult = itemsOf(searchLibrary("ラジオ", library("演歌/ラジオ体操.flac"), true)).single()
        assertEquals("ラジオ体操", result.name)
        assertEquals(1, result.tier)
    }
}
