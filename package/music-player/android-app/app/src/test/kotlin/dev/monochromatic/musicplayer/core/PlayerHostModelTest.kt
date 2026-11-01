// What:     `package dev.monochromatic.musicplayer.core` places the host model tests beside the rules they check.
// Why:      `PlayerHostModel` lives in this package, so the tests reach it without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import dev.monochromatic.musicplayer.Track` brings in the library track type the rebuild returns.
// Why:      The tests compare the rebuilt tracks with the expected ones.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Track } from "dev.monochromatic.musicplayer/Track";
// ```
import dev.monochromatic.musicplayer.Track

// What:     `import org.junit.Assert` brings in the assertions and the failure helper the tests use.
// Why:      Each rule is checked by comparing expected and actual values, and by asserting a thrown error.
//
// In TS you'd write (pseudocode):
// ```ts
// import { assert } from "@junit/assert";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `private val LOAD_ORDER_NAMES: List<String>` is a library whose page order differs from its load order.
// Why:      The folder page "Zed" holds load indices 0 and 2, so the page order visits 0, 2, then 1, which a rebuild
//           that trusts the page order would get wrong.
//
// In TS you'd write (pseudocode):
// ```ts
// const LOAD_ORDER_NAMES = ["Zed/b.flac", "Alpha.flac", "Zed/a.flac"];
// ```
/** Display paths in load order, arranged so the page order differs from the load order. */
private val LOAD_ORDER_NAMES: List<String> = listOf("Zed/b.flac", "Alpha.flac", "Zed/a.flac")

// What:     `private val LOAD_ORDER_URIS: List<String>` gives each library entry its own URI in load order.
// Why:      Distinct URIs let a test see which entry a rebuilt track came from.
//
// In TS you'd write (pseudocode):
// ```ts
// const LOAD_ORDER_URIS = ["content://0", "content://1", "content://2"];
// ```
/** One URI per entry of LOAD_ORDER_NAMES, in the same load order. */
private val LOAD_ORDER_URIS: List<String> = listOf("content://0", "content://1", "content://2")

// What:     `private val FOLDER_NAMES: List<String>` is a library with nested folders and a root-level file.
// Why:      Folder results must name a page for every top-level folder, and the root-level file must not add one.
//
// In TS you'd write (pseudocode):
// ```ts
// const FOLDER_NAMES = ["Camellia/Live/x.flac", "Camellia/y.flac", "Adele/1.flac", "root.flac"];
// ```
/** Display paths that cover nested folders, a plain folder, and a root-level file. */
private val FOLDER_NAMES: List<String> = listOf(
    "Camellia/Live/x.flac",
    "Camellia/y.flac",
    "Adele/1.flac",
    "root.flac",
)

// What:     `private val ALBUM_ITEMS: List<PageEntry>` are the three tracks of one folder page.
// Why:      The deck line and track rows need a page with a known playing track and a known count.
//
// In TS you'd write (pseudocode):
// ```ts
// const ALBUM_ITEMS = ["a", "b", "c"].map((name, index) => ({ index, name: `Album/${name}.flac` }));
// ```
/** Three tracks of the Album page, in load order, with the second one playing in the template tests. */
private val ALBUM_ITEMS: List<PageEntry> = listOf(
    PageEntry(index = 0, name = "Album/a.flac"),
    PageEntry(index = 1, name = "Album/b.flac"),
    PageEntry(index = 2, name = "Album/c.flac"),
)

// What:     `private val PLAYING_INDEX: Int` is the load index of the track that plays in the template tests.
// Why:      The second track plays, so its row is the second of three and its place reads "2 of 3".
//
// In TS you'd write (pseudocode):
// ```ts
// const PLAYING_INDEX = 1;
// ```
/** Load index of the playing track in the template tests. */
private const val PLAYING_INDEX: Int = 1

// What:     `private const val OFF_PAGE_INDEX: Int` is a load index that no track of the Album page holds.
// Why:      The deck line must stay empty when the playing track belongs to some other page.
//
// In TS you'd write (pseudocode):
// ```ts
// const OFF_PAGE_INDEX = 99;
// ```
/** Load index that the Album page does not hold, so the playing track is off the page. */
private const val OFF_PAGE_INDEX: Int = 99

// What:     `private const val KNOWN_DURATION_SEC: Double` is the length of the playing track in the duration test.
// Why:      275 seconds reads as four minutes and thirty-five seconds, the example the deck template promises.
//
// In TS you'd write (pseudocode):
// ```ts
// const KNOWN_DURATION_SEC = 275;
// ```
/** Known length of the playing track in seconds, which the deck template formats as minutes and seconds. */
private const val KNOWN_DURATION_SEC: Double = 275.0

// What:     `private fun albumModel(): PlayerContentModel` builds the player model for the Album page.
// Why:      Every template test starts from the same built model, so each differs only in the template.
//
// In TS you'd write (pseudocode):
// ```ts
// const albumModel = () => buildPlayerContent({ pageLabels: ["Album"], selectedPage: 0, pageItems: ALBUM_ITEMS, ... });
// ```
/** Builds the player model for the Album page with the second track playing. */
private fun albumModel(): PlayerContentModel = buildPlayerContent(
    pageLabels = listOf("Album"),
    selectedPage = 0,
    pageItems = ALBUM_ITEMS,
    playingItems = ALBUM_ITEMS,
    playingPageIndex = 0,
    currentIndex = PLAYING_INDEX,
    playing = true,
    mode = PlaybackMode.IN_ORDER,
    positionSec = 0.0,
    durationSec = 0.0,
)

// What:     `private fun albumPlaceFor(currentIndex: Int?): PlayingPlace` gives the playing place in the Album folder.
// Why:      The host passes the playing folder's place to the template builder, and every template case needs one.
//
// In TS you'd write (pseudocode):
// ```ts
// const albumPlaceFor = (current: number | null) => playingPlaceOf("Album", ALBUM_ITEMS, current);
// ```
/** Builds the playing place of the Album folder for one playing index. */
private fun albumPlaceFor(currentIndex: Int?): PlayingPlace =
    playingPlaceOf(folderTitle = "Album", playingItems = ALBUM_ITEMS, currentIndex = currentIndex)

// What:     `private fun resultsOf(query: String): List<SearchResult>` returns the Search results for one query.
// Why:      The folder and track checks read the same results, and an empty list stands for no results.
//
// In TS you'd write (pseudocode):
// ```ts
// const resultsOf = (query: string) => searchLibrary(query, tracks).items ?? [];
// ```
/** Returns the ranked Search results for one query over the folder-test library, or none when the state has no list. */
private fun resultsOf(query: String): List<SearchResult> {
    /** Library tracks of the folder-test names, in load order. */
    val tracks: List<Track> = PlayerHostModel.libraryTracksFrom(
        uris = FOLDER_NAMES.indices.map { index -> "content://$index" },
        pages = paginate(FOLDER_NAMES),
    )
    /** Search state for the query over those tracks. */
    val state: SearchState = searchLibrary(query = query, tracks = tracks, available = true)
    return when (state) {
        is SearchState.Results -> state.items
        else -> emptyList()
    }
}

// What:     `class PlayerHostModelTest` groups the tests of the pure player host rules.
// Why:      The library rebuild, the Search activation, and the template texts are checked here on the JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("PlayerHostModel", () => { /* ... */ });
// ```
/** Tests for the library rebuild, Search activation, template application, and the editor preview. */
class PlayerHostModelTest {
    // What:     `fun libraryTracksFromRebuildsLoadOrder()` checks that the rebuild follows load order, not page order.
    // Why:      Track results carry load-order indices, so each rebuilt track must sit at its load index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("rebuild follows load order", () => expect(libraryTracksFrom(uris, pages)).toEqual(expected));
    // ```
    /** Every rebuilt track carries its own URI and display path at its load-order position. */
    @Test
    fun libraryTracksFromRebuildsLoadOrder() {
        /** The library rebuilt from the URIs and the paginated names, which the page order visits out of sequence. */
        val tracks: List<Track> = PlayerHostModel.libraryTracksFrom(
            uris = LOAD_ORDER_URIS,
            pages = paginate(LOAD_ORDER_NAMES),
        )
        assertEquals(
            listOf(
                Track(uri = "content://0", displayPath = "Zed/b.flac"),
                Track(uri = "content://1", displayPath = "Alpha.flac"),
                Track(uri = "content://2", displayPath = "Zed/a.flac"),
            ),
            tracks,
        )
    }

    // What:     `fun libraryTracksFromEmptyLibraryIsEmpty()` checks the rebuild with no tracks.
    // Why:      A library that is still loading holds no URIs, and the rebuild must return an empty list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty library", () => expect(libraryTracksFrom([], [])).toEqual([]));
    // ```
    /** A library with no URIs and no pages rebuilds to an empty list. */
    @Test
    fun libraryTracksFromEmptyLibraryIsEmpty() {
        assertEquals(emptyList<Track>(), PlayerHostModel.libraryTracksFrom(uris = emptyList(), pages = emptyList()))
    }

    // What:     `fun libraryTracksFromRejectsMismatchedCounts()` checks that a URI count and an entry count must agree.
    // Why:      A mismatch means the controller's stores disagree, so the rebuild must refuse rather than guess.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("mismatch throws", () => expect(() => libraryTracksFrom(["a"], [])).toThrow());
    // ```
    /** A URI with no page entry makes the rebuild throw the library error. */
    @Test
    fun libraryTracksFromRejectsMismatchedCounts() {
        assertThrows(LibraryRebuildError::class.java) {
            PlayerHostModel.libraryTracksFrom(uris = listOf("content://0"), pages = emptyList())
        }
    }

    // What:     `fun everyFolderResultNamesAPageLabel()` checks that folder results match the page tabs.
    // Why:      The Search page opens a folder by its name, which must equal a page label or the result cannot open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder results name page labels", () => queries.forEach((q) => /* folderName in labels */));
    // ```
    /** Every folder result over a set of queries names one of the page labels, and the set holds a folder result. */
    @Test
    fun everyFolderResultNamesAPageLabel() {
        /** Page labels of the folder-test library, in tab order. */
        val labels: List<String> = paginate(FOLDER_NAMES).map { page -> page.label }
        /** Queries that match folder names, track titles, or nothing, to cover every result kind. */
        val queries: List<String> = listOf("a", "e", "cam", "adele", "live", "root")
        /** Every result the queries produce, across all of them. */
        val results: List<SearchResult> = queries.flatMap { query -> resultsOf(query) }
        assertTrue(results.any { result -> result.kind == SearchResultKind.FOLDER })
        results
            .filter { result -> result.kind == SearchResultKind.FOLDER }
            .forEach { result -> assertTrue(labels.contains(result.folderName)) }
    }

    // What:     `fun trackResultIndexNamesTheSameTrack()` checks that a track result's index is its load index.
    // Why:      Activating a track result selects the track at that index (D127), so the index must name the same
    //           track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("track result index", () => expect(tracks[result.indexInLibrary].displayPath).toMatch(result.name));
    // ```
    /** Every track result's index points at a track whose title equals the result's name. */
    @Test
    fun trackResultIndexNamesTheSameTrack() {
        /** Library tracks of the load-order names, which the Search index refers to. */
        val tracks: List<Track> = PlayerHostModel.libraryTracksFrom(
            uris = LOAD_ORDER_URIS,
            pages = paginate(LOAD_ORDER_NAMES),
        )
        /** Track results the query "a" produces over the load-order library. */
        val trackResults: List<SearchResult> = searchLibrary(
            query = "a",
            tracks = tracks,
            available = true,
        ).let { state ->
            when (state) {
                is SearchState.Results -> state.items.filter { result -> result.kind == SearchResultKind.TRACK }
                else -> emptyList()
            }
        }
        assertTrue(trackResults.isNotEmpty())
        trackResults.forEach { result ->
            /** Load index the track result names, which must exist in the library. */
            val index: Int = result.indexInLibrary ?: throw IllegalStateException("Track result has no index.")
            assertEquals(result.name, trackTitleFor(tracks[index].displayPath))
        }
    }

    // What:     `fun searchActivationSelectsAnotherTrack()` checks that a track other than the current one is selected.
    // Why:      A different track is selected from its load index and does not start (D124, D127).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("other track is selected", () => expect(activation(track(3), 1)).toEqual(selectTrack(3)));
    // ```
    /** A track result for a track other than the current one selects that track. */
    @Test
    fun searchActivationSelectsAnotherTrack() {
        /** Track result naming load index 3 while track 1 is current. */
        val result = trackResult(indexInLibrary = 3)
        assertEquals(
            SearchActivation.SelectTrack(indexInLibrary = 3),
            PlayerHostModel.searchActivationFor(result = result, pageLabels = listOf("A"), currentIndex = 1),
        )
    }

    // What:     `fun searchActivationTogglesTheCurrentTrack()` checks that the current track's result toggles playback.
    // Why:      Selecting the current track again would restart it, so its result pauses or starts it instead (D127).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("current track toggles", () => expect(activation(trackResult(2), 2)).toEqual({ kind: "togglePlayback" }));
    // ```
    /** A track result for the current track toggles playback and does not select the track again. */
    @Test
    fun searchActivationTogglesTheCurrentTrack() {
        /** Track result naming the playing load index. */
        val result = trackResult(indexInLibrary = 2)
        assertEquals(
            SearchActivation.TogglePlayback,
            PlayerHostModel.searchActivationFor(result = result, pageLabels = listOf("A"), currentIndex = 2),
        )
    }

    // What:     `fun searchActivationOpensFolderPage()` checks that a folder result selects its page.
    // Why:      A folder result shows its folder without autoplay (D72), so it maps to the page with its name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder opens page", () => expect(activation(folder("B"), ["A", "B"])).toEqual(openFolder(1)));
    // ```
    /** A folder result selects the page whose label equals the folder name. */
    @Test
    fun searchActivationOpensFolderPage() {
        /** Folder result for the second page of the labels. */
        val result = folderResult(name = "B")
        assertEquals(
            SearchActivation.OpenFolder(page = 1),
            PlayerHostModel.searchActivationFor(result = result, pageLabels = listOf("A", "B"), currentIndex = null),
        )
    }

    // What:     `fun searchActivationStaysForUnknownFolder()` checks a folder result that names no page.
    // Why:      A result the player cannot open must stay on Search rather than claim navigation succeeded (D74).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unknown folder stays", () => expect(activation(folder("Nope"), ["A"])).toEqual({ kind: "stayInSearch" }));
    // ```
    /** A folder result whose name matches no page label keeps the Search page open. */
    @Test
    fun searchActivationStaysForUnknownFolder() {
        assertEquals(
            SearchActivation.StayInSearch,
            PlayerHostModel.searchActivationFor(
                result = folderResult(name = "Nope"),
                pageLabels = listOf("A"),
                currentIndex = null,
            ),
        )
    }

    // What:     `fun contentWithTemplatesSetsDeckLine()` checks that the playing template sets the deck line.
    // Why:      The deck shows the playing template's text, so a saved playing template must replace the built line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("deck line from template", () => expect(contentWithTemplates(...).deck.subtitle).toBe("2 of 3 Album"));
    // ```
    /** The playing template's text becomes the deck subtitle, trimmed of its trailing blanks. */
    @Test
    fun contentWithTemplatesSetsDeckLine() {
        /** Model with the playing template that reads the track and its folder total. */
        val model: PlayerContentModel = PlayerHostModel.contentWithTemplates(
            base = albumModel(),
            pageItems = ALBUM_ITEMS,
            currentIndex = PLAYING_INDEX,
            playingPlace = albumPlaceFor(PLAYING_INDEX),
            playingMeasure = null,
            durationSec = 0.0,
            trackRowsTemplate = DEFAULT_TRACK_TEMPLATE,
            playingTemplate = "\$mi(track)\$ of \$mi(total)\$ \$mi(peak)\$",
        )
        assertEquals("2 of 3", model.deck.subtitle)
    }

    // What:     `fun contentWithTemplatesKeepsBuiltLineWhenRefused()` checks the fallback to the built "N of M" line.
    // Why:      A refused playing template must not blank the deck, so the built line stays.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("refused falls back", () => expect(deckSubtitle(badTemplate)).toBe("2 of 3"));
    // ```
    /** A playing template that names an unknown field leaves the built "N of M" line in place. */
    @Test
    fun contentWithTemplatesKeepsBuiltLineWhenRefused() {
        /** Model with a playing template that names a field the playing template does not have. */
        val model: PlayerContentModel = PlayerHostModel.contentWithTemplates(
            base = albumModel(),
            pageItems = ALBUM_ITEMS,
            currentIndex = PLAYING_INDEX,
            playingPlace = albumPlaceFor(PLAYING_INDEX),
            playingMeasure = null,
            durationSec = 0.0,
            trackRowsTemplate = DEFAULT_TRACK_TEMPLATE,
            playingTemplate = "\$mi(nothing)\$",
        )
        assertEquals("2 of 3", model.deck.subtitle)
    }

    // What:     `fun contentWithTemplatesSetsTrackRowLines()` checks that the track-row template sets each row's line.
    // Why:      Each row's supporting line comes from the track-row template, so a saved template must reach the rows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("rows use template", () => expect(rows[0].supportingLine).toBe("a in Album"));
    // ```
    /** The track-row template sets the supporting line of the first row. */
    @Test
    fun contentWithTemplatesSetsTrackRowLines() {
        /** Model whose track rows follow a template that reads the title and the folder. */
        val model: PlayerContentModel = PlayerHostModel.contentWithTemplates(
            base = albumModel(),
            pageItems = ALBUM_ITEMS,
            currentIndex = PLAYING_INDEX,
            playingPlace = albumPlaceFor(PLAYING_INDEX),
            playingMeasure = null,
            durationSec = 0.0,
            trackRowsTemplate = "\$mi(title)\$ in \$mi(folder)\$",
            playingTemplate = TemplateKind.PLAYING_TRACK.defaultTemplate,
        )
        /** Track rows of the model, without the header entries. */
        val rows: List<TrackRowModel> = model.entries
            .filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row }
        assertEquals("a in Album", rows.first().supportingLine)
    }

    // What:     `fun editorPreviewUsesStandInsWhenPageIsEmpty()` checks the stand-in preview for an empty page.
    // Why:      Stand-in values show only while no track is available to preview (D95).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty page uses stand-ins", () => expect(editorWithPreview(editor, []).usesSampleValues).toBe(true));
    // ```
    /** An empty page leaves the editor on the stand-in tracks and says so. */
    @Test
    fun editorPreviewUsesStandInsWhenPageIsEmpty() {
        /** Editor with no page tracks to preview. */
        val editor: TemplateEditorState = PlayerHostModel.editorWithPreview(
            editor = PlayerHostModel.restoredEditor(defaultTemplates()),
            pageItems = emptyList(),
        )
        assertTrue(editor.usesSampleValues)
        assertEquals(TEMPLATE_SAMPLE_TRACKS, editor.previewTracks)
    }

    // What:     `fun editorPreviewFromLibraryBuildsSettingsLines()` checks that library preview tracks give every line.
    // Why:      A library preview with a read duration yields every Settings line, with no peak yet.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("library preview lines", () => expect(Object.keys(editor.settingsLines)).toHaveLength(2));
    // ```
    /** A page preview with a read duration and no peak yields a Settings line for every template. */
    @Test
    fun editorPreviewFromLibraryBuildsSettingsLines() {
        /** Measures of the first Album row, with a known duration and no peak yet. */
        val measures: Map<Int, TrackMeasure> = mapOf(
            ALBUM_ITEMS.first().index to TrackMeasure(durationSec = KNOWN_DURATION_SEC),
        )
        /** Editor previewing the first two tracks of the Album page with their measures. */
        val editor: TemplateEditorState = PlayerHostModel.editorWithPreview(
            editor = PlayerHostModel.restoredEditor(defaultTemplates()),
            pageItems = ALBUM_ITEMS,
            measuresByIndex = measures,
        )
        assertFalse(editor.usesSampleValues)
        assertEquals(2, editor.previewTracks.size)
        assertEquals(1, editor.previewTracks.first().placeInFolder)
        assertEquals(TemplateKind.entries.toSet(), editor.settingsLines.keys)
    }

    // What:     `fun editorPreviewSettingsLineShowsMeasuresWhenPresent()` checks that measured rows fill Settings.
    // Why:      The Track rows line came out empty while the rows showed duration and peak; this fails if it returns.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("settings line has measures", () => expect(editor.settingsLine("track")).toBe("4:35 −1.2 dBTP"));
    // ```
    /** A measured first row gives the Track rows line its duration and peak, never an empty text. */
    @Test
    fun editorPreviewSettingsLineShowsMeasuresWhenPresent() {
        /** Measure of the Album page's first row, with a 275 second duration and an exact −1.2 dBTP peak. */
        val measures: Map<Int, TrackMeasure> = mapOf(
            ALBUM_ITEMS.first().index to
                TrackMeasure(durationSec = KNOWN_DURATION_SEC, peakDbtp = -1.2, peakExact = true),
        )
        /** Editor previewing the Album page with its measures. */
        val editor: TemplateEditorState = PlayerHostModel.editorWithPreview(
            editor = PlayerHostModel.restoredEditor(defaultTemplates()),
            pageItems = ALBUM_ITEMS,
            measuresByIndex = measures,
        )
        assertFalse(editor.usesSampleValues)
        assertEquals("4:35 −1.2 dBTP", editor.settingsLine(TemplateKind.TRACK_ROWS))
        assertTrue(editor.settingsLine(TemplateKind.TRACK_ROWS).isNotEmpty())
    }

    // What:     `fun editorPreviewUnreadTracksUseStandInsWithNote()` checks the fallback before any track is read.
    // Why:      With tracks but no read measure the Settings lines must not be empty, and the note must not claim
    //           the library is empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unread uses stand-ins", () => expect(editorWithPreview(editor, page, {}).sampleNote).toBe(UNREAD));
    // ```
    /** Tracks with no measure yet show the stand-in lines and a note that says the values are not read yet. */
    @Test
    fun editorPreviewUnreadTracksUseStandInsWithNote() {
        /** Editor previewing the Album page before any measure exists. */
        val editor: TemplateEditorState = PlayerHostModel.editorWithPreview(
            editor = PlayerHostModel.restoredEditor(defaultTemplates()),
            pageItems = ALBUM_ITEMS,
            measuresByIndex = emptyMap(),
        )
        assertTrue(editor.usesSampleValues)
        assertEquals(TEMPLATE_UNREAD_NOTE, editor.sampleNote)
        assertEquals(listOf(TEMPLATE_UNREAD_NOTE), editor.previewNotes)
        assertTrue(editor.settingsLine(TemplateKind.TRACK_ROWS).isNotEmpty())
    }

    // What:     `fun deckLineShowsDurationWhenKnown()` checks that a known duration reaches the playing template.
    // Why:      The playing track's length is the only duration the deck template can read, so it must arrive.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("duration reaches deck", () => expect(deckLine("$tf(mi(len), m:ss)$", 275)).toBe("4:35"));
    // ```
    /** A playing template that reads the length shows the duration of the playing track once it is known. */
    @Test
    fun deckLineShowsDurationWhenKnown() {
        /** Model whose playing template reads the length, with a known duration of 275 seconds. */
        val model: PlayerContentModel = PlayerHostModel.contentWithTemplates(
            base = albumModel(),
            pageItems = ALBUM_ITEMS,
            currentIndex = PLAYING_INDEX,
            playingPlace = albumPlaceFor(PLAYING_INDEX),
            playingMeasure = null,
            durationSec = KNOWN_DURATION_SEC,
            trackRowsTemplate = DEFAULT_TRACK_TEMPLATE,
            playingTemplate = "\$tf(mi(len), m:ss)\$",
        )
        assertEquals("4:35", model.deck.subtitle)
    }

    // What:     `fun deckLineIsEmptyWhenPlayingTrackIsNotOnPage()` checks a playing index that the page lacks.
    // Why:      A playing track on another page must leave the deck line empty, not show a stale place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("off page is blank", () => expect(contentWithTemplates({ currentIndex: 99 }).deck.subtitle).toBe(""));
    // ```
    /** A playing index that no row of the page holds gives the deck an empty line. */
    @Test
    fun deckLineIsEmptyWhenPlayingTrackIsNotOnPage() {
        /** Model whose playing index names no track of the Album page. */
        val model: PlayerContentModel = PlayerHostModel.contentWithTemplates(
            base = albumModel(),
            pageItems = ALBUM_ITEMS,
            currentIndex = OFF_PAGE_INDEX,
            playingPlace = albumPlaceFor(OFF_PAGE_INDEX),
            playingMeasure = null,
            durationSec = 0.0,
            trackRowsTemplate = DEFAULT_TRACK_TEMPLATE,
            playingTemplate = TemplateKind.PLAYING_TRACK.defaultTemplate,
        )
        assertEquals("", model.deck.subtitle)
    }

    // What:     `fun searchActivationStaysForTrackWithoutIndex()` checks a track result that names no load index.
    // Why:      A track result without an index cannot be played, so it must stay on Search (D74).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("track without index stays", () => expect(activation(trackNoIndex, 1)).toEqual(stayInSearch));
    // ```
    /** A track result with no load-order index keeps the Search page open. */
    @Test
    fun searchActivationStaysForTrackWithoutIndex() {
        assertEquals(
            SearchActivation.StayInSearch,
            PlayerHostModel.searchActivationFor(
                result = trackResult(indexInLibrary = 3).copy(indexInLibrary = null),
                pageLabels = listOf("A"),
                currentIndex = PLAYING_INDEX,
            ),
        )
    }

    // What:     `fun searchActivationStaysForFolderWithoutName()` checks a folder result that names no folder.
    // Why:      A folder result with no folder name has no page to select, so it must stay on Search (D74).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder without name stays", () => expect(activation(folderNoName)).toEqual(stayInSearch));
    // ```
    /** A folder result with no folder name keeps the Search page open. */
    @Test
    fun searchActivationStaysForFolderWithoutName() {
        assertEquals(
            SearchActivation.StayInSearch,
            PlayerHostModel.searchActivationFor(
                result = folderResult(name = "B").copy(folderName = null),
                pageLabels = listOf("A", "B"),
                currentIndex = null,
            ),
        )
    }

    // What:     `private fun defaultTemplates(): Map<TemplateKind, String>` gives each template its default text.
    // Why:      Editor tests start from the stored texts, and the defaults are the texts a fresh store returns.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const defaultTemplates = () => Object.fromEntries(kinds.map((k) => [k, k.defaultTemplate]));
    // ```
    /** Each template's default text, keyed by template kind. */
    private fun defaultTemplates(): Map<TemplateKind, String> =
        TemplateKind.entries.associateWith { kind -> kind.defaultTemplate }

    // What:     `private fun trackResult(indexInLibrary: Int): SearchResult` builds one track result for a test.
    // Why:      The activation tests need a track result with only its index and kind set.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const trackResult = (indexInLibrary: number): SearchResult => ({ kind: "track", indexInLibrary, ... });
    // ```
    /** Builds one track result naming the given load index, with the other fields left empty. */
    private fun trackResult(indexInLibrary: Int): SearchResult = SearchResult(
        kind = SearchResultKind.TRACK,
        name = "Track",
        parentContext = "Album",
        indexInLibrary = indexInLibrary,
        folderName = "Album",
        tier = 0,
        nameHighlights = emptyList(),
        parentHighlights = emptyList(),
    )

    // What:     `private fun folderResult(name: String): SearchResult` builds one folder result for a test.
    // Why:      The activation tests need a folder result with only its name set.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const folderResult = (name: string): SearchResult => ({ kind: "folder", name, folderName: name, ... });
    // ```
    /** Builds one folder result with the given name, with the other fields left empty. */
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
}
