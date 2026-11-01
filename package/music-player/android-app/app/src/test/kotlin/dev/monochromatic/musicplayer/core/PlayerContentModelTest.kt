// Player content model cases: the rail, the selected page's rows and headers, the playing track, and the deck text.

// What:     `package ...core` places these tests beside the builder they exercise.
// Why:      The tests reach `buildPlayerContent` and its helpers through the same package without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import dev.monochromatic.musicplayer.TransportDeckModel` brings in the deck snapshot type for assertions.
// Why:      The deck fields are compared one by one against the builder output.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { TransportDeckModel } from "dev.monochromatic.musicplayer/TransportDeck";
// ```
import dev.monochromatic.musicplayer.Track
import dev.monochromatic.musicplayer.TransportDeckModel

// What:     JUnit's assertions and test annotation register each player content case.
// Why:      Each case compares whole values, so order, flags, and text are checked together.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class PlayerContentModelTest` groups the builder cases.
// Why:      One class keeps the fixtures and the helper that flattens the entries next to the cases using them.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("buildPlayerContent", () => { /* cases */ });
// ```
/** Verifies the player content builder: empty library, page rows and headers, index remap, and deck text. */
class PlayerContentModelTest {
    // What:     `private fun pageEntry(index: Int, name: String): PageEntry` builds one page row.
    // Why:      Each fixture names its load-order index and its full display path, the two fields the builder reads.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pageEntry = (index: number, name: string): PageEntry => ({ index, name });
    // ```
    /** Builds one page entry from its load-order index and full display path. */
    private fun pageEntry(index: Int, name: String): PageEntry = PageEntry(index = index, name = name)

    // What:     `private fun build(...)` calls the builder with fixed playback values and the given page.
    // Why:      Every case changes only the page and the playing index, so the other arguments are fixed here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const build = (labels: string[], page: number, items: PageEntry[], current: number | null) => buildPlayerContent(...);
    // ```
    /** Builds the player content for one page with fixed playback values. */
    private fun build(
        labels: List<String>,
        selectedPage: Int,
        items: List<PageEntry>,
        currentIndex: Int?,
        mode: PlaybackMode = PlaybackMode.IN_ORDER,
        playing: Boolean = true,
    ): PlayerContentModel = buildPlayerContent(
        pageLabels = labels,
        selectedPage = selectedPage,
        pageItems = items,
        playingItems = items,
        playingPageIndex = selectedPage,
        currentIndex = currentIndex,
        playing = playing,
        mode = mode,
        positionSec = FIXTURE_POSITION_SEC,
        durationSec = FIXTURE_DURATION_SEC,
    )

    // What:     `fun deckNamesPlayingTrackFromItsOwnFolder()` checks the deck with the playing folder off screen.
    // Why:      The deck must keep the playing title and its `N of M` from the playing folder, not the folder shown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("deck from playing folder", () => expect(build(shownOther, playingAlbum).deck.subtitle).toBe("2 of 3"));
    // ```
    /** With another folder on screen, the deck names the playing track and its place in its own folder. */
    @Test
    fun deckNamesPlayingTrackFromItsOwnFolder() {
        /** Entries of the folder on screen, which holds no playing track. */
        val shownItems: List<PageEntry> = listOf(pageEntry(0, "Other/One.flac"))
        /** Entries of the playing folder, whose second track is the playing one. */
        val playingItems: List<PageEntry> = listOf(
            pageEntry(1, "Album/First.flac"),
            pageEntry(2, "Album/Second.flac"),
            pageEntry(3, "Album/Third.flac"),
        )
        /** Player model with the Other folder on screen and the playing track in the Album folder. */
        val model: PlayerContentModel = buildPlayerContent(
            pageLabels = listOf("Other", "Album"),
            selectedPage = 0,
            pageItems = shownItems,
            playingItems = playingItems,
            playingPageIndex = 1,
            currentIndex = 2,
            playing = true,
            mode = PlaybackMode.IN_ORDER,
            positionSec = 0.0,
            durationSec = 0.0,
        )
        assertEquals("Second", model.deck.title)
        assertEquals("2 of 3", model.deck.subtitle)
        assertEquals("Other", model.folderTitle)
    }

    // What:     `private fun rowKeys(entries: List<TrackListEntry>): List<String>` names each entry for comparison.
    // Why:      A row is compared by its load-order index and a header by its name and count, in one readable list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rowKeys = (entries: TrackListEntry[]): string[] => entries.map(keyOf);
    // ```
    /** Names each entry as its load index for a row or its name and count for a header. */
    private fun rowKeys(entries: List<TrackListEntry>): List<String> = entries.map { entry ->
        when (entry) {
            is TrackListEntry.RowEntry -> "row:${entry.row.indexInLibrary}"
            is TrackListEntry.HeaderEntry -> "header:${entry.name}:${entry.trackCount}"
        }
    }

    // What:     `@Test fun emptyLibraryGivesEmptyContent()` checks that no pages means no rail, rows, or deck text.
    // Why:      A fresh install has no library yet, and the screen must still build without crashing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty library gives empty content", () => { /* ... */ });
    // ```
    /** A library with no pages gives no rail, no rows, and an empty deck title and subtitle. */
    @Test
    fun emptyLibraryGivesEmptyContent() {
        /** Content built from no pages and no tracks. */
        val content: PlayerContentModel = build(labels = emptyList(), selectedPage = 0, items = emptyList(), currentIndex = null)
        assertTrue(content.sections.isEmpty())
        assertEquals(null, content.currentFolder)
        assertEquals("", content.folderTitle)
        assertTrue(content.entries.isEmpty())
        assertEquals("", content.deck.title)
        assertEquals("", content.deck.subtitle)
        assertEquals("", content.deck.shuffleFolderLabel)
    }

    // What:     `@Test fun folderPageGivesOwnTracksThenHeadersWithLoadIndices()` checks the folder page layout.
    // Why:      Own tracks come first, then each subfolder header with its tracks, and rows carry the load-order index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder page gives own tracks then headers with load indices", () => { /* ... */ });
    // ```
    /** A folder page lists its own tracks first, then a subfolder header and its tracks, using load-order indices. */
    @Test
    fun folderPageGivesOwnTracksThenHeadersWithLoadIndices() {
        /** Page items in load order, with a subfolder track between two own tracks. */
        val items: List<PageEntry> = listOf(
            pageEntry(index = 5, name = "Camellia/Live/Ghost (Live).flac"),
            pageEntry(index = 9, name = "Camellia/01 Song.flac"),
            pageEntry(index = 12, name = "Camellia/Live/Dokuhebi (Live).flac"),
            pageEntry(index = 13, name = "Camellia/02 Other.flac"),
        )
        /** Content for the Camellia page, selected as the first page. */
        val content: PlayerContentModel = build(labels = listOf("Camellia"), selectedPage = 0, items = items, currentIndex = null)
        assertEquals(
            listOf("row:9", "row:13", "header:Live:2", "row:5", "row:12"),
            rowKeys(content.entries),
        )
    }

    // What:     `@Test fun rootLetterPageListsRootTracksWithLoadIndices()` checks a letter page at the library root.
    // Why:      Root tracks form the Unfiled entry, and a letter page must still list them in load order with their indices.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("root letter page lists root tracks", () => { /* ... */ });
    // ```
    /** A letter page of root-level tracks lists them in load order under their own indices. */
    @Test
    fun rootLetterPageListsRootTracksWithLoadIndices() {
        /** Root-level tracks on the letter A page, in load order. */
        val items: List<PageEntry> = listOf(
            pageEntry(index = 3, name = "Apple.flac"),
            pageEntry(index = 7, name = "Ant.mp3"),
        )
        /** Content for the letter A page, selected as the second page. */
        val content: PlayerContentModel = build(labels = listOf("Camellia", "A"), selectedPage = 1, items = items, currentIndex = 7)
        assertEquals(listOf("row:3", "row:7"), rowKeys(content.entries))
        assertEquals("A", content.currentFolder)
        assertEquals("A", content.folderTitle)
    }

    // What:     `@Test fun playingTrackInsidePageMarksItsRowAndDeck()` checks the current row and deck text.
    // Why:      The playing row is highlighted and the deck names that track with its position in the page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("playing track inside the page marks its row and deck", () => { /* ... */ });
    // ```
    /** A playing track on the page marks its row and fills the deck title and its position subtitle. */
    @Test
    fun playingTrackInsidePageMarksItsRowAndDeck() {
        /** Two tracks on the page; the playing one is the second row in playback order. */
        val items: List<PageEntry> = listOf(
            pageEntry(index = 9, name = "Camellia/01 Song.flac"),
            pageEntry(index = 10, name = "Camellia/02 Other.flac"),
        )
        /** Content with the second track playing. */
        val content: PlayerContentModel = build(labels = listOf("Camellia"), selectedPage = 0, items = items, currentIndex = 10)
        /** Whether each row of the page is the playing row, in playback order. */
        val currentRows: List<Boolean> = content.entries
            .filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row.isCurrent }
        assertEquals(listOf(false, true), currentRows)
        assertEquals("02 Other", content.deck.title)
        assertEquals("2 of 2", content.deck.subtitle)
    }

    // What:     `@Test fun playingTrackOutsidePageGivesEmptyDeckText()` checks a playing track on another page.
    // Why:      The page does not list that track, so no row is current and the deck names nothing from this page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("playing track outside the page gives empty deck text", () => { /* ... */ });
    // ```
    /** A playing track that is not on the selected page leaves every row unmarked and the deck text empty. */
    @Test
    fun playingTrackOutsidePageGivesEmptyDeckText() {
        /** One track on the page, while the playing track has a load index that the page does not hold. */
        val items: List<PageEntry> = listOf(pageEntry(index = 9, name = "Camellia/01 Song.flac"))
        /** Content with a playing index that no page entry carries. */
        val content: PlayerContentModel = build(labels = listOf("Camellia"), selectedPage = 0, items = items, currentIndex = 40)
        assertTrue(content.entries.filterIsInstance<TrackListEntry.RowEntry>().none { entry -> entry.row.isCurrent })
        assertEquals("", content.deck.title)
        assertEquals("", content.deck.subtitle)
    }

    // What:     `@Test fun singlePageGivesOneRailSectionWithItsName()` checks a library with one page.
    // Why:      A one-page library must still produce a rail with that page as the only cell.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("single page gives one rail section", () => { /* ... */ });
    // ```
    /** A library with one page gives one rail section whose only cell lists that page. */
    @Test
    fun singlePageGivesOneRailSectionWithItsName() {
        /** Content for the only page of the library. */
        val content: PlayerContentModel = build(
            labels = listOf("Camellia"),
            selectedPage = 0,
            items = listOf(pageEntry(index = 0, name = "Camellia/01 Song.flac")),
            currentIndex = 0,
        )
        assertEquals(1, content.sections.size)
        assertEquals(listOf("Camellia"), content.sections.single().cells.single().names)
        assertEquals("Camellia", content.currentFolder)
    }

    // What:     `@Test fun selectedIndexOutsidePageListGivesNoFolder()` checks a selected page past the end of the list.
    // Why:      A stale selection must not name a folder that does not exist, so the heading is empty instead.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("selected index outside the page list gives no folder", () => { /* ... */ });
    // ```
    /** A selected index past the page list gives no current folder and an empty folder title. */
    @Test
    fun selectedIndexOutsidePageListGivesNoFolder() {
        /** Content with a selected page that the list does not contain. */
        val content: PlayerContentModel = build(labels = listOf("Camellia"), selectedPage = 4, items = emptyList(), currentIndex = null)
        assertEquals(null, content.currentFolder)
        assertEquals("", content.folderTitle)
        assertEquals("", content.deck.shuffleFolderLabel)
    }

    // What:     `@Test fun deckTitleDropsOnlyTheLastExtension()` checks the title rule on a dotted file name.
    // Why:      The deck title follows the track row title, so an inner dot must survive as it does in the rows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("deck title drops only the last extension", () => { /* ... */ });
    // ```
    /** The deck title keeps an inner dot of the file name and drops only the last extension. */
    @Test
    fun deckTitleDropsOnlyTheLastExtension() {
        /** Content whose playing track has a dotted file name. */
        val content: PlayerContentModel = build(
            labels = listOf("Camellia"),
            selectedPage = 0,
            items = listOf(pageEntry(index = 2, name = "Camellia/01 Song.v2.flac")),
            currentIndex = 2,
        )
        assertEquals("01 Song.v2", content.deck.title)
    }

    // What:     `@Test fun deckPassesPlaybackValuesThrough()` checks the values the builder copies unchanged.
    // Why:      Position, duration, playing state, and mode must reach the deck exactly as the controller reports them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("deck passes playback values through", () => { /* ... */ });
    // ```
    /** Position, duration, playing flag, and mode pass through to the deck unchanged, and the folder label is the title. */
    @Test
    fun deckPassesPlaybackValuesThrough() {
        /** Content with paused playback in shuffle-all mode. */
        val content: PlayerContentModel = build(
            labels = listOf("Camellia"),
            selectedPage = 0,
            items = listOf(pageEntry(index = 0, name = "Camellia/01 Song.flac")),
            currentIndex = 0,
            mode = PlaybackMode.SHUFFLE_ALL,
            playing = false,
        )
        /** Deck snapshot the builder produced. */
        val deck: TransportDeckModel = content.deck
        assertEquals(FIXTURE_POSITION_SEC, deck.positionSec, 0.0)
        assertEquals(FIXTURE_DURATION_SEC, deck.durationSec, 0.0)
        assertEquals(false, deck.playing)
        assertEquals(PlaybackMode.SHUFFLE_ALL, deck.mode)
        assertEquals("Camellia", deck.shuffleFolderLabel)
    }

    // What:     `@Test fun subtitleCountsRowsInPlaybackOrder()` checks that the position counts rows, not headers.
    // Why:      The subtitle matches what the track list shows, and subfolder headers are not tracks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("subtitle counts rows in playback order", () => { /* ... */ });
    // ```
    /** The subtitle position skips subfolder headers, so a track after a header counts as its row number. */
    @Test
    fun subtitleCountsRowsInPlaybackOrder() {
        /** Page with one own track and a subfolder track after the header. */
        val items: List<PageEntry> = listOf(
            pageEntry(index = 1, name = "Camellia/01 Song.flac"),
            pageEntry(index = 2, name = "Camellia/Live/02 Live.flac"),
        )
        /** Content with the subfolder track playing. */
        val content: PlayerContentModel = build(labels = listOf("Camellia"), selectedPage = 0, items = items, currentIndex = 2)
        assertEquals("2 of 2", content.deck.subtitle)
    }

    // What:     `@Test fun withLoadIndicesReplacesTrackPositionsOnly()` checks the remap helper directly.
    // Why:      The helper must map each track to its page entry's load index and leave headers untouched.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("withLoadIndices replaces track positions only", () => { /* ... */ });
    // ```
    /** The remap helper replaces each track's list position with its page entry index and keeps headers as they are. */
    @Test
    fun withLoadIndicesReplacesTrackPositionsOnly() {
        /** Track at list position 0, which the page entry at position 0 maps to load index 42. */
        val track = FolderListItem.TrackItem(FolderTrackRow(Track(uri = "", displayPath = "Camellia/A.flac"), 0))
        /** Header that carries no track index. */
        val header = FolderListItem.SubfolderHeader(name = "Live", trackCount = 1)
        /** Page entries with a load index that differs from the list position. */
        val pageItems: List<PageEntry> = listOf(pageEntry(index = 42, name = "Camellia/A.flac"))
        assertEquals(
            listOf<FolderListItem>(
                FolderListItem.TrackItem(FolderTrackRow(Track(uri = "", displayPath = "Camellia/A.flac"), 42)),
                header,
            ),
            withLoadIndices(items = listOf(track, header), pageItems = pageItems),
        )
    }

    // What:     `@Test fun pageFolderForKeepsEveryPageTrackAsOneFolder()` checks the single flat entry of a page.
    // Why:      The track list draws one folder, so every page track must appear in that entry and its count must match.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("pageFolderFor keeps every page track as one folder", () => { /* ... */ });
    // ```
    /** A page becomes one folder entry that carries every page track and the page track count. */
    @Test
    fun pageFolderForKeepsEveryPageTrackAsOneFolder() {
        /** Page with two own tracks and one subfolder track. */
        val items: List<PageEntry> = listOf(
            pageEntry(index = 4, name = "Camellia/01 Song.flac"),
            pageEntry(index = 5, name = "Camellia/Live/02 Live.flac"),
            pageEntry(index = 6, name = "Camellia/03 Other.flac"),
        )
        /** Folder entry built for the page. */
        val folder: FolderEntry = pageFolderFor(folderTitle = "Camellia", pageItems = items)
        assertEquals("Camellia", folder.name)
        assertEquals(3, folder.trackCount)
        assertEquals(3, folder.items.filterIsInstance<FolderListItem.TrackItem>().size)
        assertEquals(1, folder.items.filterIsInstance<FolderListItem.SubfolderHeader>().size)
    }

    // What:     `@Test fun subtitleIsEmptyWhenNoRowIsPlaying()` checks the subtitle helper on its own.
    // Why:      A negative row position must give an empty subtitle, so the deck shows no misleading count.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("subtitle is empty when no row is playing", () => { /* ... */ });
    // ```
    /** A negative row position gives an empty subtitle, and a valid position gives the one-based count. */
    @Test
    fun subtitleIsEmptyWhenNoRowIsPlaying() {
        assertEquals("", subtitleFor(rowPosition = -1, pageTrackCount = 5))
        assertEquals("1 of 5", subtitleFor(rowPosition = 0, pageTrackCount = 5))
    }

    // What:     `@Test fun deckFolderLabelIsThePlayingTracksDirectory()` checks the deck's folder label in a subfolder.
    // Why:      The heading shows the folder of the playing track, which is the directory above its file name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("deck folder label is the playing directory", () => { /* ... */ });
    // ```
    /** The deck's folder label is the directory directly holding the playing track. */
    @Test
    fun deckFolderLabelIsThePlayingTracksDirectory() {
        /** Content for the Camellia page, whose second track sits in the Live subfolder. */
        val content: PlayerContentModel = build(
            labels = listOf("Camellia"),
            selectedPage = 0,
            items = listOf(pageEntry(index = 4, name = "Camellia/Live/Ghost.flac")),
            currentIndex = 4,
        )
        assertEquals("Live", content.deck.folderLabel)
    }

    // What:     `@Test fun deckFolderLabelFallsBackToThePageForRootTracks()` checks a track at the page root.
    // Why:      A track with no directory part has the page name as its folder in the deck heading.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("deck folder label falls back to the page", () => { /* ... */ });
    // ```
    /** A playing track at the root of its page gets the page label as its deck folder label. */
    @Test
    fun deckFolderLabelFallsBackToThePageForRootTracks() {
        /** Content for the Camellia page, whose only track sits at the page root. */
        val content: PlayerContentModel = build(
            labels = listOf("Camellia"),
            selectedPage = 0,
            items = listOf(pageEntry(index = 0, name = "Ghost.flac")),
            currentIndex = 0,
        )
        assertEquals("Camellia", content.deck.folderLabel)
    }

    // What:     `@Test fun deckFolderLabelIsEmptyWhenNothingPlays()` checks the deck folder label when nothing plays.
    // Why:      With nothing playing the heading has no folder part, so the label must be empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("deck folder label is empty when nothing plays", () => { /* ... */ });
    // ```
    /** With no playing track the deck's folder label is an empty text. */
    @Test
    fun deckFolderLabelIsEmptyWhenNothingPlays() {
        /** Content for the Camellia page with no playing track. */
        val content: PlayerContentModel = build(
            labels = listOf("Camellia"),
            selectedPage = 0,
            items = listOf(pageEntry(index = 0, name = "Camellia/Live/Ghost.flac")),
            currentIndex = null,
        )
        assertEquals("", content.deck.folderLabel)
    }

    // What:     `private companion object` holds the fixed playback values shared by every builder case.
    // Why:      The cases pass the same position and duration, so the constants are named once here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const FIXTURE = { positionSec: 66, durationSec: 275 };
    // ```
    /** Fixed playback values that the builder cases pass through unchanged. */
    private companion object {
        /** Playhead position of the fixture, one minute and six seconds in. */
        const val FIXTURE_POSITION_SEC: Double = 66.0

        /** Length of the fixture track, four minutes and thirty-five seconds. */
        const val FIXTURE_DURATION_SEC: Double = 275.0
    }
}
