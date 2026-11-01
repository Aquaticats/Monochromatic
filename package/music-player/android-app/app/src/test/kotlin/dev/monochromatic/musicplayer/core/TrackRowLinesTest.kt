// Cases for the per-row supporting lines that keep the track list from being rebuilt on every analysis step.

// What:     `package ...core` places these tests beside the line source and the entries they exercise.
// Why:      The tests reach `TrackRowLineSource` and `PlayerHostModel` through the same package without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     JUnit's assertions and test annotation register each line-source case.
// Why:      Each case compares whole lists or counts, so text, order and the number of reads are checked together.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies that row lines computed one row at a time equal the lines of a full build, and never read a page. */
class TrackRowLinesTest {
    // What:     `private fun pageOf(count: Int): List<PageEntry>` builds a page of nested tracks.
    // Why:      Nested folders give the entries headers and root files, so the fixtures cover both entry kinds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pageOf = (count: number): PageEntry[] => range(count).map((i) => ({ index: 100 + i, name: pathOf(i) }));
    // ```
    /** Builds a page whose load-order indices start at 100, so the index is never the page position. */
    private fun pageOf(count: Int): List<PageEntry> = (0 until count).map { position ->
        PageEntry(
            index = 100 + position,
            name = if (position % 3 == 0) "Track $position.flac" else "Album ${position % 4}/Track $position.flac",
        )
    }

    // What:     `private val measures: Map<Int, TrackMeasure>` gives some page tracks each kind of measure.
    // Why:      A line differs for an unread track, a length without a peak, a sampled peak and an exact peak.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const measures = new Map([[100, exact], [101, lengthOnly], [102, sampled]]);
    // ```
    /** Measures of three of the fixture tracks by load-order index; every other index has none. */
    private val someMeasures: Map<Int, TrackMeasure> = mapOf(
        100 to TrackMeasure(durationSec = 185.0, peakDbtp = -1.5, peakExact = true),
        101 to TrackMeasure(durationSec = 62.0),
        102 to TrackMeasure(durationSec = 3.0, peakDbtp = -9.25, peakExact = false),
        105 to TrackMeasure(durationSec = 0.0, peakDbtp = 0.0, peakExact = true),
    )

    // What:     `private fun linesOf(entries): Map<Int, String?>` maps each row's index to its supporting line.
    // Why:      The full build and the line source are compared per row, whatever the headers between them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const linesOf = (entries) => new Map(entries.filter(isRow).map((e) => [e.row.indexInLibrary, e.row.line]));
    // ```
    /** Returns the supporting line of every row entry by load-order index, skipping headers. */
    private fun linesOf(entries: List<TrackListEntry>): Map<Int, String?> = entries
        .filterIsInstance<TrackListEntry.RowEntry>()
        .associate { entry -> entry.row.indexInLibrary to entry.row.supportingLine }

    // What:     `@Test fun rowLinesEqualTheLinesOfAFullBuildForTheDefaultTemplate()` compares both ways to a line.
    // Why:      The host stops building lines into the entries, so each row's own line must be the line the full
    //           build gave it, for every kind of measure.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("row lines equal the full build's lines for the default template", () => { /* ... */ });
    // ```
    /** Verifies the per-row source gives every row the line that the full build gives it, default template. */
    @Test
    fun rowLinesEqualTheLinesOfAFullBuildForTheDefaultTemplate() {
        /** Page of twelve tracks. */
        val page: List<PageEntry> = pageOf(12)
        /** Lines that the full build computes for the page with the measures. */
        val full: Map<Int, String?> = linesOf(
            PlayerHostModel.rowEntriesFor(
                folderTitle = "Fixture",
                pageItems = page,
                currentIndex = null,
                trackRowsTemplate = DEFAULT_TRACK_TEMPLATE,
                measuresByIndex = someMeasures,
            ),
        )
        /** Source that answers each row's line from the same measures. */
        val source: TrackRowLineSource = TrackRowLineSource.forPage(
            pageItems = page,
            template = DEFAULT_TRACK_TEMPLATE,
            measureOf = { index -> someMeasures[index] },
        )
        assertEquals(page.size, full.size)
        assertEquals(full, page.associate { entry -> entry.index to source.lineOf(entry.index) })
    }

    // What:     `@Test fun rowLinesEqualTheLinesOfAFullBuildForACustomTemplate()` repeats the comparison.
    // Why:      A user template reads the folder and file fields too, which come from the display path of each row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("row lines equal the full build's lines for a custom template", () => { /* ... */ });
    // ```
    /** Verifies the per-row source matches the full build for a template that reads path fields and the peak. */
    @Test
    fun rowLinesEqualTheLinesOfAFullBuildForACustomTemplate() {
        /** Template with a folder field, a file field and the peak. */
        val template = "\$mi(folder)\$ / \$mi(file)\$ \$mi(peak)\$"
        /** Page of twelve tracks. */
        val page: List<PageEntry> = pageOf(12)
        /** Lines that the full build computes for the page with the measures. */
        val full: Map<Int, String?> = linesOf(
            PlayerHostModel.rowEntriesFor(
                folderTitle = "Fixture",
                pageItems = page,
                currentIndex = null,
                trackRowsTemplate = template,
                measuresByIndex = someMeasures,
            ),
        )
        /** Source that answers each row's line from the same measures. */
        val source: TrackRowLineSource = TrackRowLineSource.forPage(
            pageItems = page,
            template = template,
            measureOf = { index -> someMeasures[index] },
        )
        assertEquals(full, page.associate { entry -> entry.index to source.lineOf(entry.index) })
    }

    // What:     `@Test fun entriesWithoutLinesKeepTheirShapeAndCarryNoLine()` checks the list built for the host.
    // Why:      The host's list must hold the same titles, headers and playing flag as the full build, with no line,
    //           because the lines come from the source.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("entries without lines keep their shape and carry no line", () => { /* ... */ });
    // ```
    /** Verifies the line-free list equals the full list with every row's line removed. */
    @Test
    fun entriesWithoutLinesKeepTheirShapeAndCarryNoLine() {
        /** Page of twelve tracks. */
        val page: List<PageEntry> = pageOf(12)
        /** Full build with lines, the playing track being the fourth. */
        val full: List<TrackListEntry> = PlayerHostModel.rowEntriesFor(
            folderTitle = "Fixture",
            pageItems = page,
            currentIndex = 103,
            trackRowsTemplate = DEFAULT_TRACK_TEMPLATE,
            measuresByIndex = someMeasures,
        )
        /** Build for the host, without lines. */
        val bare: List<TrackListEntry> = PlayerHostModel.rowEntriesFor(
            folderTitle = "Fixture",
            pageItems = page,
            currentIndex = 103,
            trackRowsTemplate = DEFAULT_TRACK_TEMPLATE,
            measuresByIndex = someMeasures,
            withSupportingLines = false,
        )
        /** The full list with each row's line cleared. */
        val fullWithoutLines: List<TrackListEntry> = full.map { entry ->
            when (entry) {
                is TrackListEntry.RowEntry -> TrackListEntry.RowEntry(entry.row.copy(supportingLine = null))
                is TrackListEntry.HeaderEntry -> entry
            }
        }
        assertEquals(fullWithoutLines, bare)
        assertTrue(linesOf(bare).values.all { line -> line == null })
    }

    // What:     `@Test fun entriesWithoutLinesDoNotDependOnTheMeasures()` builds the list under two measure sets.
    // Why:      The host leaves the measures out of the keys of its list, so a new measure must not change the list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("entries without lines do not depend on the measures", () => { /* ... */ });
    // ```
    /** Verifies the line-free list is equal whether the measures are empty or full. */
    @Test
    fun entriesWithoutLinesDoNotDependOnTheMeasures() {
        /** Page of twelve tracks. */
        val page: List<PageEntry> = pageOf(12)
        /** Line-free build with no measure known. */
        val unmeasured: List<TrackListEntry> = PlayerHostModel.rowEntriesFor(
            folderTitle = "Fixture",
            pageItems = page,
            currentIndex = null,
            trackRowsTemplate = DEFAULT_TRACK_TEMPLATE,
            withSupportingLines = false,
        )
        /** Line-free build with some measures known. */
        val measured: List<TrackListEntry> = PlayerHostModel.rowEntriesFor(
            folderTitle = "Fixture",
            pageItems = page,
            currentIndex = null,
            trackRowsTemplate = DEFAULT_TRACK_TEMPLATE,
            measuresByIndex = someMeasures,
            withSupportingLines = false,
        )
        assertEquals(unmeasured, measured)
    }

    // What:     `@Test fun aRowLineFollowsItsMeasureWithoutNewSource()` changes a measure under one source.
    // Why:      A row asks the source each time its measure changes, so the answer must follow the reader and must
    //           not be remembered from the first call.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("a row line follows its measure without a new source", () => { /* ... */ });
    // ```
    /** Verifies the same source gives the new line after the measure it reads changes. */
    @Test
    fun aRowLineFollowsItsMeasureWithoutNewSource() {
        /** Measures the source reads, which the test changes in place. */
        val live: MutableMap<Int, TrackMeasure> = mutableMapOf(100 to TrackMeasure(durationSec = 62.0))
        /** Source over a page of twelve tracks. */
        val source: TrackRowLineSource = TrackRowLineSource.forPage(
            pageItems = pageOf(12),
            template = DEFAULT_TRACK_TEMPLATE,
            measureOf = { index -> live[index] },
        )
        assertEquals("1:02", source.lineOf(100))
        live[100] = TrackMeasure(durationSec = 185.0, peakDbtp = -1.5, peakExact = true)
        assertEquals("3:05 ${MEASURE_MINUS_SIGN}1.5 dBTP", source.lineOf(100))
        assertNull(source.lineOf(101))
    }

    // What:     `@Test fun oneRowLineReadsOneMeasureAndBuildingReadsNone()` counts the reads of the measure reader.
    // Why:      This is the cost model: a page of N tracks must cost nothing per measure, and a row must cost one read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("one row line reads one measure and building reads none", () => { /* ... */ });
    // ```
    /** Verifies that building the source and the line-free list read no measure, and one row line reads one. */
    @Test
    fun oneRowLineReadsOneMeasureAndBuildingReadsNone() {
        /** Count of calls to the measure reader. */
        var reads = 0
        /** Page of 2,000 tracks. */
        val page: List<PageEntry> = pageOf(2_000)
        /** Source whose reader counts its calls. */
        val source: TrackRowLineSource = TrackRowLineSource.forPage(
            pageItems = page,
            template = DEFAULT_TRACK_TEMPLATE,
            measureOf = { _ ->
                reads += 1
                null
            },
        )
        PlayerHostModel.rowEntriesFor(
            folderTitle = "Fixture",
            pageItems = page,
            currentIndex = null,
            trackRowsTemplate = DEFAULT_TRACK_TEMPLATE,
            withSupportingLines = false,
        )
        assertEquals(0, reads)
        source.lineOf(1_000)
        assertEquals(1, reads)
    }

    // What:     `@Test fun anIndexOutsideThePageHasNoLineAndReadsNothing()` asks for an index the page lacks.
    // Why:      A stale row for a replaced page must show no line, and must not read a measure of another page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("an index outside the page has no line and reads nothing", () => { /* ... */ });
    // ```
    /** Verifies an index that is not on the page gives null without calling the measure reader. */
    @Test
    fun anIndexOutsideThePageHasNoLineAndReadsNothing() {
        /** Count of calls to the measure reader. */
        var reads = 0
        /** Source over a page whose indices run from 100 to 111. */
        val source: TrackRowLineSource = TrackRowLineSource.forPage(
            pageItems = pageOf(12),
            template = DEFAULT_TRACK_TEMPLATE,
            measureOf = { _ ->
                reads += 1
                null
            },
        )
        assertNull(source.lineOf(5))
        assertEquals(0, reads)
    }

    // What:     `@Test fun contentUsesTheRailAndPlayingPlaceItIsGiven()` passes ready-made pieces to the builder.
    // Why:      The host builds the rail and the playing place once and hands them in, so the builder must use them
    //           and not index the playing folder again on every position tick.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("content uses the rail and playing place it is given", () => { /* ... */ });
    // ```
    /** Verifies the builder returns the given rail instance and takes the deck line from the given place. */
    @Test
    fun contentUsesTheRailAndPlayingPlaceItIsGiven() {
        /** Page of three tracks, also the playing folder. */
        val page: List<PageEntry> = pageOf(3)
        /** Rail that the builder must reuse as it is. */
        val rail: List<RailSection> = railFor(listOf("Fixture"))
        /** Place that differs from what the builder would compute: the third track of three. */
        val given = PlayingPlace(entry = page[2], rowPosition = 2, folderTrackCount = 3)
        /** Content built with the ready-made pieces. */
        val content: PlayerContentModel = buildPlayerContent(
            pageLabels = listOf("Fixture"),
            selectedPage = 0,
            pageItems = page,
            playingItems = page,
            playingPageIndex = 0,
            currentIndex = 100,
            playing = false,
            mode = PlaybackMode.IN_ORDER,
            positionSec = 0.0,
            durationSec = 0.0,
            sections = rail,
            playingPlace = given,
        )
        assertSame(rail, content.sections)
        assertEquals(subtitleFor(rowPosition = 2, pageTrackCount = 3), content.deck.subtitle)
    }
}
