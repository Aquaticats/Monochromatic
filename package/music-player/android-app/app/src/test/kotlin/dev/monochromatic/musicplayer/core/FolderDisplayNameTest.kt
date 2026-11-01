// What:     `package dev.monochromatic.musicplayer.core` places this test beside the folder name rule it checks.
// Why:      Same-package tests call `folderNameFromDocumentId` without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     JUnit assertions register each folder name check.
// Why:      Every document id shape is compared with an explicit expected name.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class FolderDisplayNameTest` groups the checks for the names that tree document ids produce.
// Why:      The first-run copy throws on a blank name, so every shape of document id must give a usable name.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("folderNameFromDocumentId", () => { ... });
// ```
/** Verifies the folder name for nested folders, volume roots, and malformed document ids. */
class FolderDisplayNameTest {
    // What:     `fun nestedFolderUsesLastSegment` checks that a nested folder is named by its last segment.
    // Why:      The first-run copy names the folder the user chose, which is the innermost segment.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("nested", () => expect(folderNameFromDocumentId("primary:Music/Albums")).toBe("Albums"));
    // ```
    /** Confirms that a nested folder gives its innermost name. */
    @Test
    fun nestedFolderUsesLastSegment() {
        assertEquals("Albums", folderNameFromDocumentId("primary:Music/Albums"))
    }

    // What:     `fun singleSegmentFolderUsesItsName` checks a folder directly under the volume root.
    // Why:      A single segment is the folder name itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("single", () => expect(folderNameFromDocumentId("primary:Music")).toBe("Music"));
    // ```
    /** Confirms that a folder directly under the volume root gives its own name. */
    @Test
    fun singleSegmentFolderUsesItsName() {
        assertEquals("Music", folderNameFromDocumentId("primary:Music"))
    }

    // What:     `fun trailingSeparatorIsIgnored` checks that a trailing slash does not hide the folder name.
    // Why:      Some providers end the path with a separator, and the name must still be the folder.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("trailing", () => expect(folderNameFromDocumentId("primary:Music/Albums/")).toBe("Albums"));
    // ```
    /** Confirms that a trailing separator is ignored when the name is read. */
    @Test
    fun trailingSeparatorIsIgnored() {
        assertEquals("Albums", folderNameFromDocumentId("primary:Music/Albums/"))
    }

    // What:     `fun primaryVolumeRootUsesInternalStorage` checks the root of the internal shared storage.
    // Why:      The root has no path segment, so the volume names it with a readable label.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("primary root", () => expect(folderNameFromDocumentId("primary:")).toBe("Internal storage"));
    // ```
    /** Confirms that the root of the primary volume is named Internal storage. */
    @Test
    fun primaryVolumeRootUsesInternalStorage() {
        assertEquals("Internal storage", folderNameFromDocumentId("primary:"))
    }

    // What:     `fun otherVolumeRootUsesVolumeId` checks the root of a removable volume.
    // Why:      A removable volume has no friendlier label in the id, so its volume id is the name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("other root", () => expect(folderNameFromDocumentId("1234-ABCD:")).toBe("1234-ABCD"));
    // ```
    /** Confirms that the root of a non-primary volume is named by its volume id. */
    @Test
    fun otherVolumeRootUsesVolumeId() {
        assertEquals("1234-ABCD", folderNameFromDocumentId("1234-ABCD:"))
    }

    // What:     `fun emptyIdUsesSelectedFolder` checks an empty document id.
    // Why:      An empty id names nothing, and the fallback keeps the copy from showing a blank headline.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty", () => expect(folderNameFromDocumentId("")).toBe("Selected folder"));
    // ```
    /** Confirms that an empty document id gives the fallback name. */
    @Test
    fun emptyIdUsesSelectedFolder() {
        assertEquals("Selected folder", folderNameFromDocumentId(""))
    }

    // What:     `fun colonOnlyIdUsesSelectedFolder` checks a document id that has only the separator.
    // Why:      With neither a volume nor a path, the fallback applies, so the name is never blank.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("colon only", () => expect(folderNameFromDocumentId(":")).toBe("Selected folder"));
    // ```
    /** Confirms that a document id with only the separator gives the fallback name. */
    @Test
    fun colonOnlyIdUsesSelectedFolder() {
        assertEquals("Selected folder", folderNameFromDocumentId(":"))
    }

    // What:     `fun resultIsNeverBlank` checks that every shape gives a name with visible text.
    // Why:      The first-run copy throws on a blank name, so the rule must hold for all shapes, not only the ones
    //           above.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("never blank", () => for (const id of shapes) expect(folderNameFromDocumentId(id).trim()).not.toBe(""));
    // ```
    /** Confirms that no tested document id shape gives a blank name. */
    @Test
    fun resultIsNeverBlank() {
        /** Document ids of several shapes, including malformed ones, that the rule must name. */
        val shapes: List<String> = listOf("", ":", "primary:", "primary: ", "primary:/", "  :  Music", "Music")
        for (shape in shapes) {
            /** Name the rule gives for this shape. */
            val name: String = folderNameFromDocumentId(shape)
            assertTrue("blank name for id '$shape'", name.isNotBlank())
        }
    }

    // What:     `fun libraryNameIsTheHeldFolder` checks that a held folder names the library.
    // Why:      The unfolded folder pane is headed by the opened folder's own name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("held folder names the library", () => expect(libraryDisplayName("Music")).toBe("Music"));
    // ```
    /** Confirms that a held folder's name is used as it is, apart from surrounding blanks. */
    @Test
    fun libraryNameIsTheHeldFolder() {
        assertEquals("Music", libraryDisplayName("Music"))
        assertEquals("Music", libraryDisplayName("  Music "))
    }

    // What:     `fun libraryNameFallsBackToMediaStore` checks the name without a held folder.
    // Why:      The whole-device library has no folder name, so the platform's name for it heads the pane.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no folder uses the MediaStore name", () => expect(libraryDisplayName(null)).toBe("MediaStore"));
    // ```
    /** Confirms that a missing or blank folder name gives the MediaStore name. */
    @Test
    fun libraryNameFallsBackToMediaStore() {
        assertEquals("Music library", libraryDisplayName(null))
        assertEquals("Music library", libraryDisplayName("   "))
    }
}
