// Track list entry cases for the rows and headers the track list pane draws (D5, D6, D81, D99).

// What:     `package ...core` places these tests beside the entries they exercise.
// Why:      The tests reach `trackListEntries` and its row types through the same package without imports.
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

// What:     JUnit's value assertion and test annotation register each track list case.
// Why:      Each case compares whole entry lists or single values, so order, flags, and text are checked together.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** Verifies track titles, row mapping, current flags, headers, supporting lines, and the empty folder. */
class TrackListEntriesTest {
    // What:     `private fun track(displayPath: String): Track` builds a fixture track from its display path.
    // Why:      Only the display path affects the entries, so the URI can be derived from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const track = (displayPath: string): Track => ({ uri: `content://${displayPath}`, displayPath });
    // ```
    /** Builds a library track whose URI is derived from its display path. */
    private fun track(displayPath: String): Track = Track("content://$displayPath", displayPath)

    // What:     `private fun onlyFolder(tracks: List<Track>): FolderEntry` returns the single folder the index makes.
    // Why:      Each entry case starts from the folder index, so the fixtures match what production builds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const onlyFolder = (tracks: Track[]): FolderEntry => folderIndex(tracks)[0];
    // ```
    /** Builds the folder index from the tracks and returns its first folder entry. */
    private fun onlyFolder(tracks: List<Track>): FolderEntry = folderIndex(tracks).first()

    // What:     `@Test fun titleDropsOnlyTheLastExtension()` checks that a dotted name keeps its inner dots.
    // Why:      A title such as `01 Song.v2.flac` must keep `01 Song.v2`, because only the last extension goes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("title drops only the last extension", () => { /* ... */ });
    // ```
    /** Verifies that only the final extension is removed from a dotted file name. */
    @Test
    fun titleDropsOnlyTheLastExtension() {
        assertEquals("01 Song.v2", trackTitleFor("Camellia/Album/01 Song.v2.flac"))
    }

    // What:     `@Test fun titleDropsEachKindOfExtension()` checks several common audio extensions.
    // Why:      The title rule must not depend on the extension's length or spelling.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("title drops each kind of extension", () => { /* ... */ });
    // ```
    /** Verifies four audio extensions of different lengths are each removed from their file names. */
    @Test
    fun titleDropsEachKindOfExtension() {
        /** Display paths with four different audio extensions. */
        val paths: List<String> = listOf("a.mp3", "b.m4a", "c.opus", "d.ogg")
        assertEquals(listOf("a", "b", "c", "d"), paths.map { path -> trackTitleFor(path) })
    }

    // What:     `@Test fun titleStripsEveryFolderSegment()` checks a deeply nested path.
    // Why:      The row title is the file name only, so no folder segment may leak into it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("title strips every folder segment", () => { /* ... */ });
    // ```
    /** Verifies a file nested under several folders gives only its own name as the title. */
    @Test
    fun titleStripsEveryFolderSegment() {
        assertEquals("03 X", trackTitleFor("Camellia/Album/Disc 1/03 X.flac"))
    }

    // What:     `@Test fun titleKeepsNameWithoutExtension()` checks a name with no dot.
    // Why:      Such a name has no extension to remove, so it must come back unchanged.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("title keeps a name without an extension", () => { /* ... */ });
    // ```
    /** Verifies a file name with no dot is returned whole. */
    @Test
    fun titleKeepsNameWithoutExtension() {
        assertEquals("Song", trackTitleFor("Camellia/Song"))
    }

    // What:     `@Test fun titleKeepsHiddenFileNameWhole()` checks a name that starts with a dot.
    // Why:      A leading dot marks a hidden file, not an extension, so the whole name stays as the title.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("title keeps a hidden file name whole", () => { /* ... */ });
    // ```
    /** Verifies a hidden file name keeps its leading dot. */
    @Test
    fun titleKeepsHiddenFileNameWhole() {
        assertEquals(".hidden", trackTitleFor("Camellia/.hidden"))
    }

    // What:     `@Test fun rowsFollowFolderOrderWithLibraryIndexes()` checks order and indexes in a mixed folder.
    // Why:      The pane must draw own tracks first, then the header, then its tracks, each with its library index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("rows follow folder order with library indexes", () => { /* ... */ });
    // ```
    /** Verifies own tracks lead, then a subfolder header with its tracks, all in the folder's order. */
    @Test
    fun rowsFollowFolderOrderWithLibraryIndexes() {
        /** Folder with two own tracks and one track in a subfolder, supplied out of name order. */
        val folder: FolderEntry = onlyFolder(
            listOf(track("Camellia/b.flac"), track("Camellia/a.flac"), track("Camellia/Album/c.flac")),
        )
        assertEquals(
            listOf(
                TrackListEntry.RowEntry(TrackRowModel(0, "b", null, false)),
                TrackListEntry.RowEntry(TrackRowModel(1, "a", null, false)),
                TrackListEntry.HeaderEntry("Album", 1),
                TrackListEntry.RowEntry(TrackRowModel(2, "c", null, false)),
            ),
            trackListEntries(folder, null),
        )
    }

    // What:     `@Test fun currentFlagMarksOnlyTheMatchingIndex()` checks the playing row.
    // Why:      Only the row whose library index is the playing index may be flagged as current.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("current flag marks only the matching index", () => { /* ... */ });
    // ```
    /** Verifies exactly the row with the playing library index is flagged as current. */
    @Test
    fun currentFlagMarksOnlyTheMatchingIndex() {
        /** Folder of three tracks in library order. */
        val folder: FolderEntry = onlyFolder(listOf(track("Camellia/a.flac"), track("Camellia/b.flac"), track("Camellia/c.flac")))
        val rows: List<TrackRowModel> = trackListEntries(folder, 1).filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row }
        assertEquals(listOf(false, true, false), rows.map { row -> row.isCurrent })
    }

    // What:     `@Test fun currentIndexOutsideFolderMarksNoRow()` checks a playing track in another folder.
    // Why:      A playing index with no row in this folder must leave every row unflagged.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("current index outside the folder marks no row", () => { /* ... */ });
    // ```
    /** Verifies no row is flagged when the playing index names a track outside this folder. */
    @Test
    fun currentIndexOutsideFolderMarksNoRow() {
        /** Folder of two tracks at library indexes 0 and 1. */
        val folder: FolderEntry = onlyFolder(listOf(track("Camellia/a.flac"), track("Camellia/b.flac")))
        val rows: List<TrackRowModel> = trackListEntries(folder, 99).filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row }
        assertEquals(listOf(false, false), rows.map { row -> row.isCurrent })
    }

    // What:     `@Test fun headerCountsOnlyDirectTracks()` checks that a header counts only the tracks listed under it.
    // Why:      D5 counts the tracks directly under each subfolder, so a deeper subfolder's track belongs to its own header.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("header counts only direct tracks", () => { /* ... */ });
    // ```
    /** Verifies each subfolder header counts its own direct tracks, with nested subfolders as separate headers. */
    @Test
    fun headerCountsOnlyDirectTracks() {
        /** Folder with one track in `Album` and one track in the deeper `Album/Disc 1`. */
        val folder: FolderEntry = onlyFolder(listOf(track("Camellia/Album/Disc 1/x.flac"), track("Camellia/Album/y.flac")))
        assertEquals(
            listOf(
                TrackListEntry.HeaderEntry("Album", 1),
                TrackListEntry.RowEntry(TrackRowModel(1, "y", null, false)),
                TrackListEntry.HeaderEntry("Album/Disc 1", 1),
                TrackListEntry.RowEntry(TrackRowModel(0, "x", null, false)),
            ),
            trackListEntries(folder, null),
        )
    }

    // What:     `@Test fun defaultTemplateWithUnknownLengthAndPeakGivesNoLine()` checks the default template.
    // Why:      Production does not know duration or true peak per row, so the default line must show no second line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("default template with unknown length and peak gives no line", () => { /* ... */ });
    // ```
    /** Verifies the default supporting line is absent when both its fields are unknown. */
    @Test
    fun defaultTemplateWithUnknownLengthAndPeakGivesNoLine() {
        /** Folder with one track whose duration and peak are unknown. */
        val folder: FolderEntry = onlyFolder(listOf(track("Camellia/Song.flac")))
        val rows: List<TrackRowModel> = trackListEntries(folder, null).filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row }
        assertNull(rows.single().supportingLine)
    }

    // What:     `@Test fun lengthOnlyTemplateGivesNoLine()` checks a template that shows only the duration.
    // Why:      With no duration the formatted field is blank, and the line must be dropped rather than shown empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("length-only template gives no line", () => { /* ... */ });
    // ```
    /** Verifies a duration-only template yields no supporting line when the duration is unknown. */
    @Test
    fun lengthOnlyTemplateGivesNoLine() {
        /** Folder with one track whose duration is unknown. */
        val folder: FolderEntry = onlyFolder(listOf(track("Camellia/Song.flac")))
        val rows: List<TrackRowModel> = trackListEntries(folder, null, "\$tf(mi(len), m:ss)\$")
            .filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row }
        assertNull(rows.single().supportingLine)
    }

    // What:     `@Test fun peakOnlyTemplateGivesNoLine()` checks a template that shows only the true peak.
    // Why:      With no peak the field is blank, so the row must show no second line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("peak-only template gives no line", () => { /* ... */ });
    // ```
    /** Verifies a peak-only template yields no supporting line when the peak is unknown. */
    @Test
    fun peakOnlyTemplateGivesNoLine() {
        /** Folder with one track whose true peak is unknown. */
        val folder: FolderEntry = onlyFolder(listOf(track("Camellia/Song.flac")))
        val rows: List<TrackRowModel> = trackListEntries(folder, null, "\$mi(peak)\$")
            .filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row }
        assertNull(rows.single().supportingLine)
    }

    // What:     `@Test fun customTemplateShowsFolderAndFileName()` checks a custom template with two known fields.
    // Why:      A user template must render its known fields, with its literal text kept between them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("custom template shows folder and file name", () => { /* ... */ });
    // ```
    /** Verifies a custom template shows the parent folder and the file name around its literal slash. */
    @Test
    fun customTemplateShowsFolderAndFileName() {
        /** Folder holding one track in a subfolder named `Album`. */
        val folder: FolderEntry = onlyFolder(listOf(track("Camellia/Album/01 Song.flac")))
        val rows: List<TrackRowModel> = trackListEntries(folder, null, "\$mi(folder)\$ / \$mi(file)\$")
            .filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row }
        assertEquals("Album / 01 Song", rows.single().supportingLine)
    }

    // What:     `@Test fun customTemplateTrimsBlankFromUnknownField()` checks that a blank field's space is removed.
    // Why:      An unknown peak leaves a trailing space that must not appear as a visible offset in the line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("custom template trims the blank from an unknown field", () => { /* ... */ });
    // ```
    /** Verifies the shown text is trimmed when a trailing field is unknown. */
    @Test
    fun customTemplateTrimsBlankFromUnknownField() {
        /** Folder holding one track at the library root of its folder. */
        val folder: FolderEntry = onlyFolder(listOf(track("Camellia/01 Song.flac")))
        val rows: List<TrackRowModel> = trackListEntries(folder, null, "\$mi(file)\$ \$mi(peak)\$")
            .filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row }
        assertEquals("01 Song", rows.single().supportingLine)
    }

    // What:     `@Test fun invalidTemplateGivesNoLine()` checks a template naming an unknown field.
    // Why:      A refused template must show no error text on a row, so its supporting line is null.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("invalid template gives no line", () => { /* ... */ });
    // ```
    /** Verifies a template that names an unknown field yields no supporting line. */
    @Test
    fun invalidTemplateGivesNoLine() {
        /** Folder with one track. */
        val folder: FolderEntry = onlyFolder(listOf(track("Camellia/Song.flac")))
        val rows: List<TrackRowModel> = trackListEntries(folder, null, "\$mi(nope)\$")
            .filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row }
        assertNull(rows.single().supportingLine)
    }

    // What:     `@Test fun rootFileFolderFieldGivesNoLine()` checks the `folder` field for a file at the library root.
    // Why:      A root-level file has no folder to show, so a template that shows only the folder must give no line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("root file folder field gives no line", () => { /* ... */ });
    // ```
    /** Verifies a root-level file in its virtual letter entry yields no supporting line for a folder-only template. */
    @Test
    fun rootFileFolderFieldGivesNoLine() {
        /** Letter entry holding one track at the library root. */
        val folder: FolderEntry = folderIndex(listOf(track("Loose.flac"))).single()
        val rows: List<TrackRowModel> = trackListEntries(folder, null, "\$mi(folder)\$")
            .filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row }
        assertEquals("L", folder.name)
        assertEquals(listOf<String?>(null), rows.map { row -> row.supportingLine })
    }

    // What:     `@Test fun nestedFolderFieldShowsInnermostName()` checks the `folder` field inside a subfolder.
    // Why:      The folder field is the innermost folder, which is the parent of the file, not the top-level folder.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("nested folder field shows the innermost name", () => { /* ... */ });
    // ```
    /** Verifies the folder field names the file's own folder inside a deeper path. */
    @Test
    fun nestedFolderFieldShowsInnermostName() {
        /** Folder holding one track inside `Album/Disc 1`. */
        val folder: FolderEntry = onlyFolder(listOf(track("Camellia/Album/Disc 1/03 X.flac")))
        val rows: List<TrackRowModel> = trackListEntries(folder, null, "\$mi(folder)\$")
            .filterIsInstance<TrackListEntry.RowEntry>()
            .map { entry -> entry.row }
        assertEquals(listOf<String?>("Disc 1"), rows.map { row -> row.supportingLine })
    }

    // What:     `@Test fun emptyFolderGivesEmptyList()` checks a folder with no items.
    // Why:      An empty folder must produce no entries rather than a placeholder row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty folder gives empty list", () => { /* ... */ });
    // ```
    /** Verifies a folder with no items maps to an empty entry list. */
    @Test
    fun emptyFolderGivesEmptyList() {
        /** Folder with no items and no tracks. */
        val folder = FolderEntry("Empty", emptyList(), 0)
        assertEquals(emptyList<TrackListEntry>(), trackListEntries(folder, null))
    }

    // What:     `@Test fun headerTrackCountMatchesSubfolderSize()` checks a single header's count.
    // Why:      The header count is what the pane shows beside the subfolder name, so it must match the tracks listed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("header track count matches subfolder size", () => { /* ... */ });
    // ```
    /** Verifies a subfolder header counts the three tracks listed under it. */
    @Test
    fun headerTrackCountMatchesSubfolderSize() {
        /** Folder with three tracks in one subfolder and none directly inside. */
        val folder: FolderEntry = onlyFolder(
            listOf(track("Camellia/Album/a.flac"), track("Camellia/Album/b.flac"), track("Camellia/Album/c.flac")),
        )
        assertEquals(listOf(TrackListEntry.HeaderEntry("Album", 3)), trackListEntries(folder, null).take(1))
    }
}
