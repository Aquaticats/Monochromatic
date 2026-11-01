// Deck heading text cases: the folder label that the deck shows before the playing track's title.

// What:     `package ...core` places these tests beside the function they exercise.
// Why:      The tests reach `deckFolderLabelFor` through the same package without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     JUnit's assertions and test annotation register each folder label case.
// Why:      Each case compares one expected label with the function's result.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `class DeckHeadingTest` groups the folder label cases.
// Why:      One class keeps the cases for each path shape together.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("deckFolderLabelFor", () => { /* cases */ });
// ```
/** Verifies the folder label the deck heading shows for the playing track. */
class DeckHeadingTest {
    // What:     `@Test fun subfolderNameIsTheDirectlyHoldingDirectory()` checks a track inside a subfolder.
    // Why:      The label is the directory directly above the file, not the page name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("subfolder label", () => expect(deckFolderLabelFor("Camellia/Live/Ghost.flac", "Camellia")).toBe("Live"));
    // ```
    /** A track inside a subfolder is labelled by the directory directly above its file name. */
    @Test
    fun subfolderNameIsTheDirectlyHoldingDirectory() {
        assertEquals("Live", deckFolderLabelFor(displayPath = "Camellia/Live/Ghost.flac", pageLabel = "Camellia"))
    }

    // What:     `@Test fun deepestDirectoryIsUsed()` checks a track nested two folders deep.
    // Why:      Only the innermost directory names the track's folder, so the deeper name must win.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("deepest directory", () => expect(deckFolderLabelFor("A/B/Disc 1/G.flac", "A")).toBe("Disc 1"));
    // ```
    /** A track nested two folders deep is labelled by its innermost directory. */
    @Test
    fun deepestDirectoryIsUsed() {
        assertEquals(
            "Disc 1",
            deckFolderLabelFor(displayPath = "Camellia/Live/Disc 1/Ghost.flac", pageLabel = "Camellia"),
        )
    }

    // What:     `@Test fun directChildUsesTheParentFolderName()` checks a track directly in a top-level folder.
    // Why:      The directory above the file is the top-level folder, and the page label must not replace it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("direct child", () => expect(deckFolderLabelFor("Camellia/Ghost.flac", "Other")).toBe("Camellia"));
    // ```
    /** A track directly inside a folder is labelled by that folder, even when the page label differs. */
    @Test
    fun directChildUsesTheParentFolderName() {
        assertEquals("Camellia", deckFolderLabelFor(displayPath = "Camellia/Ghost.flac", pageLabel = "Other"))
    }

    // What:     `@Test fun rootTrackUsesThePageLabel()` checks a track with no directory part.
    // Why:      A track at the root of its page has no folder of its own, so the page name is the folder.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("root track", () => expect(deckFolderLabelFor("Ghost.flac", "Camellia")).toBe("Camellia"));
    // ```
    /** A track with no directory part is labelled by the page label. */
    @Test
    fun rootTrackUsesThePageLabel() {
        assertEquals("Camellia", deckFolderLabelFor(displayPath = "Ghost.flac", pageLabel = "Camellia"))
    }

    // What:     `@Test fun blankDirectorySegmentUsesThePageLabel()` checks a path with an empty directory segment.
    // Why:      An empty segment names no folder, so the page label is used rather than an empty heading part.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("blank segment", () => expect(deckFolderLabelFor("Camellia//Ghost.flac", "Other")).toBe("Other"));
    // ```
    /** A path whose directory segment is blank is labelled by the page label. */
    @Test
    fun blankDirectorySegmentUsesThePageLabel() {
        assertEquals("Other", deckFolderLabelFor(displayPath = "Camellia//Ghost.flac", pageLabel = "Other"))
    }

    // What:     `@Test fun emptyPathGivesEmptyLabel()` checks that no playing track gives no label.
    // Why:      When nothing plays the deck has no folder part, so the label must be empty even with a page label.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty path", () => expect(deckFolderLabelFor("", "Camellia")).toBe(""));
    // ```
    /** An empty display path gives an empty label, even when a page label is given. */
    @Test
    fun emptyPathGivesEmptyLabel() {
        assertEquals("", deckFolderLabelFor(displayPath = "", pageLabel = "Camellia"))
    }
}
