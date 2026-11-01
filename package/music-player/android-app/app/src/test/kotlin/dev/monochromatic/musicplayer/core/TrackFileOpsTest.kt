// Unit tests for the pure track file rules: trash routes, URI parsing, MIME types, copy verification, and the Undo
// state machine. These run on the host JVM with no device.

// What:     `package dev.monochromatic.musicplayer.core` places this test beside the rules it checks.
// Why:      The test reaches the internal and public rule functions in the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertions, the expected-exception check, and the test annotation.
// Why:      Each rule outcome is compared with an explicit expected value, and state errors are checked by type.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Assert.assertThrows
import org.junit.Test

// What:     `private const val TREE_HELD: String` is the held tree URI used by the route tests.
// Why:      A fixed tree keeps each route test about one difference from this baseline.
//
// In TS you'd write (pseudocode):
// ```ts
// const TREE_HELD = "content://com.android.externalstorage.documents/tree/primary%3AMusic";
// ```
/** Tree URI of the folder the user holds in the route tests. */
private const val TREE_HELD: String = "content://com.android.externalstorage.documents/tree/primary%3AMusic"

// What:     `private const val DOC_IN_HELD: String` is a track under the held tree.
// Why:      The baseline track that the copy-back route must accept.
//
// In TS you'd write (pseudocode):
// ```ts
// const DOC_IN_HELD = `${TREE_HELD}/document/primary%3AMusic%2FAlbum%2Fa.mp3`;
// ```
/** Document URI of one track under the held tree. */
private const val DOC_IN_HELD: String =
    "content://com.android.externalstorage.documents/tree/primary%3AMusic/document/primary%3AMusic%2FAlbum%2Fa.mp3"

// What:     `private const val MEDIA_TRACK: String` is a track from the device-wide media library.
// Why:      The baseline track that the media trash request must accept.
//
// In TS you'd write (pseudocode):
// ```ts
// const MEDIA_TRACK = "content://media/external/audio/media/42";
// ```
/** Media-library URI of one track. */
private const val MEDIA_TRACK: String = "content://media/external/audio/media/42"

// What:     `class TrackFileOpsTest` groups the pure tests for the track file rules.
// Why:      Each route, parser, and state transition can be verified on the host JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("track file ops", () => { ... });
// ```
/** Verifies the trash route choice, URI parsing, MIME types, copy verification, and the Undo state transitions. */
class TrackFileOpsTest {
    // What:     `mediaStoreUriRoutesToTrashRequest` checks that a media-library track takes Android's trash request.
    // Why:      The device-wide library is the MediaStore route, and no folder grant is needed for it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("media route", () => expect(trashRouteFor(MEDIA_TRACK, null, false)).toBe("MEDIASTORE_TRASH_REQUEST"));
    // ```
    /** Confirms a media-library URI routes to the MediaStore trash request. */
    @Test
    fun mediaStoreUriRoutesToTrashRequest() {
        assertEquals(TrashRoute.MEDIASTORE_TRASH_REQUEST, trashRouteFor(MEDIA_TRACK, null, false))
    }

    // What:     `mediaStoreRouteIgnoresFolderGrant` checks that a media track needs no folder grant.
    // Why:      The media library is not behind a picked folder, so a missing grant must not block it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("media ignores grant", () => expect(trashRouteFor(MEDIA_TRACK, TREE_HELD, true)).toBe("MEDIASTORE_TRASH_REQUEST"));
    // ```
    /** Confirms the media route does not depend on the held folder or its write access. */
    @Test
    fun mediaStoreRouteIgnoresFolderGrant() {
        assertEquals(TrashRoute.MEDIASTORE_TRASH_REQUEST, trashRouteFor(MEDIA_TRACK, TREE_HELD, writableGrant = false))
    }

    // What:     `safTrackUnderHeldWritableTreeRoutesToCopyBack` checks the baseline picked-folder route.
    // Why:      A track under the held tree with write access must take the copy-back route.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("saf route", () => expect(trashRouteFor(DOC_IN_HELD, TREE_HELD, true)).toBe("SAF_DELETE_WITH_COPY_BACK"));
    // ```
    /** Confirms a track under the held writable tree routes to the copy-back flow. */
    @Test
    fun safTrackUnderHeldWritableTreeRoutesToCopyBack() {
        assertEquals(TrashRoute.SAF_DELETE_WITH_COPY_BACK, trashRouteFor(DOC_IN_HELD, TREE_HELD, writableGrant = true))
    }

    // What:     `safTrackWithoutWritableGrantIsUnavailable` checks that a read-only grant refuses the trash.
    // Why:      Deleting needs write access, and a read-only grant would fail after the copy, so the route is refused early.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("read only", () => expect(trashRouteFor(DOC_IN_HELD, TREE_HELD, false)).toBe("UNAVAILABLE"));
    // ```
    /** Confirms a folder grant without write access makes the route unavailable. */
    @Test
    fun safTrackWithoutWritableGrantIsUnavailable() {
        assertEquals(TrashRoute.UNAVAILABLE, trashRouteFor(DOC_IN_HELD, TREE_HELD, writableGrant = false))
    }

    // What:     `safTrackUnderOtherTreeIsUnavailable` checks that a track from another tree is refused.
    // Why:      A track outside the held grant cannot be restored through it, so it must not be deleted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("other tree", () => expect(trashRouteFor(otherDoc, TREE_HELD, true)).toBe("UNAVAILABLE"));
    // ```
    /** Confirms a track listed under a different tree route is unavailable. */
    @Test
    fun safTrackUnderOtherTreeIsUnavailable() {
        val other: String = "content://com.android.externalstorage.documents/tree/sdcard%3AMusic/document/sdcard%3AMusic%2Fa.mp3"
        assertEquals(TrashRoute.UNAVAILABLE, trashRouteFor(other, TREE_HELD, writableGrant = true))
    }

    // What:     `treeIdPrefixIsNotAMatch` checks that a longer tree identifier does not match a shorter held one.
    // Why:      A string prefix match would let a grant on one tree cover another, so the identifiers must be equal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("prefix", () => expect(trashRouteFor("…/tree/abcd/document/x", "…/tree/abc", true)).toBe("UNAVAILABLE"));
    // ```
    /** Confirms a tree identifier that only starts with the held identifier is refused. */
    @Test
    fun treeIdPrefixIsNotAMatch() {
        val held: String = "content://com.android.externalstorage.documents/tree/abc"
        val track: String = "content://com.android.externalstorage.documents/tree/abcd/document/abcd%2Fa.mp3"
        assertEquals(TrashRoute.UNAVAILABLE, trashRouteFor(track, held, writableGrant = true))
    }

    // What:     `noHeldTreeIsUnavailable` checks that a picked-folder track with no held grant is refused.
    // Why:      Without a held tree there is nothing to restore into, so the route must be unavailable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no tree", () => expect(trashRouteFor(DOC_IN_HELD, null, true)).toBe("UNAVAILABLE"));
    // ```
    /** Confirms a folder track with no held tree is unavailable. */
    @Test
    fun noHeldTreeIsUnavailable() {
        assertEquals(TrashRoute.UNAVAILABLE, trashRouteFor(DOC_IN_HELD, null, writableGrant = true))
    }

    // What:     `safTrackWithoutTreeSegmentIsUnavailable` checks a plain document URI with no tree segment.
    // Why:      Only tree-form URIs carry the grant identity, so a bare document URI cannot be matched to a grant.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no tree segment", () => expect(trashRouteFor("…/document/x", TREE_HELD, true)).toBe("UNAVAILABLE"));
    // ```
    /** Confirms a document URI without a tree segment is unavailable. */
    @Test
    fun safTrackWithoutTreeSegmentIsUnavailable() {
        val bare: String = "content://com.android.externalstorage.documents/document/primary%3AMusic%2Fa.mp3"
        assertEquals(TrashRoute.UNAVAILABLE, trashRouteFor(bare, TREE_HELD, writableGrant = true))
    }

    // What:     `unknownAuthorityIsUnavailable` checks a provider that is neither media nor the folder provider.
    // Why:      The app has no verified route for other providers, so it refuses rather than guessing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unknown authority", () => expect(trashRouteFor("content://drive/x", null, true)).toBe("UNAVAILABLE"));
    // ```
    /** Confirms a content URI from an unknown provider is unavailable. */
    @Test
    fun unknownAuthorityIsUnavailable() {
        assertEquals(TrashRoute.UNAVAILABLE, trashRouteFor("content://drive.example/document/abc", null, true))
    }

    // What:     `mediaPrefixIsNotAnAuthorityMatch` checks an authority that only starts with the media word.
    // Why:      A prefix match would send an unrelated provider to the media trash request, so the authority must be exact.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("media prefix", () => expect(trashRouteFor("content://mediafoo/x", null, true)).toBe("UNAVAILABLE"));
    // ```
    /** Confirms an authority beginning with the media word is not treated as the media library. */
    @Test
    fun mediaPrefixIsNotAnAuthorityMatch() {
        assertEquals(TrashRoute.UNAVAILABLE, trashRouteFor("content://mediafoo/external/audio/media/1", null, true))
    }

    // What:     `nonContentUriIsUnavailable` checks a file URI, which the library never produces.
    // Why:      A plain path has no provider, so no trash route can own it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("file uri", () => expect(trashRouteFor("file:///sdcard/a.mp3", null, true)).toBe("UNAVAILABLE"));
    // ```
    /** Confirms a non-content URI is unavailable. */
    @Test
    fun nonContentUriIsUnavailable() {
        assertEquals(TrashRoute.UNAVAILABLE, trashRouteFor("file:///sdcard/Music/a.mp3", TREE_HELD, true))
    }

    // What:     `emptyUriIsUnavailable` checks an empty track URI.
    // Why:      An empty string names no file, so it must not reach a route.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty", () => expect(trashRouteFor("", null, true)).toBe("UNAVAILABLE"));
    // ```
    /** Confirms an empty URI is unavailable. */
    @Test
    fun emptyUriIsUnavailable() {
        assertEquals(TrashRoute.UNAVAILABLE, trashRouteFor("", TREE_HELD, true))
    }

    // What:     `authorityReadsHost` checks the authority of a content URI.
    // Why:      The route depends on this text, so its extraction is pinned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("authority", () => expect(authorityOf(DOC_IN_HELD)).toBe("com.android.externalstorage.documents"));
    // ```
    /** Confirms the authority is the text between the scheme and the first slash. */
    @Test
    fun authorityReadsHost() {
        assertEquals("com.android.externalstorage.documents", authorityOf(DOC_IN_HELD))
    }

    // What:     `authorityNullForNonContent` checks a string without the content scheme.
    // Why:      A non-content string has no authority, so the function returns null rather than a fragment.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("authority null", () => expect(authorityOf("file:///a")).toBeNull());
    // ```
    /** Confirms a non-content string yields no authority. */
    @Test
    fun authorityNullForNonContent() {
        assertNull(authorityOf("file:///a"))
    }

    // What:     `authorityNullWhenEmpty` checks a content URI with an empty authority.
    // Why:      An empty authority would match nothing, so it is reported as absent.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty authority", () => expect(authorityOf("content:///x")).toBeNull());
    // ```
    /** Confirms an empty authority is reported as absent. */
    @Test
    fun authorityNullWhenEmpty() {
        assertNull(authorityOf("content:///x"))
    }

    // What:     `treeIdReadsSegmentBeforeDocument` checks the tree identifier of a document URI.
    // Why:      The grant comparison uses this identifier, so it must stop at the next slash.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("tree id", () => expect(treeIdOf(DOC_IN_HELD)).toBe("primary%3AMusic"));
    // ```
    /** Confirms the tree identifier stops at the slash that starts the document segment. */
    @Test
    fun treeIdReadsSegmentBeforeDocument() {
        assertEquals("primary%3AMusic", treeIdOf(DOC_IN_HELD))
    }

    // What:     `treeIdReadsTreeRootWithoutDocument` checks a tree URI with no document segment.
    // Why:      The held grant is stored as a tree root, so this form must give the same identifier.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("tree root", () => expect(treeIdOf(TREE_HELD)).toBe("primary%3AMusic"));
    // ```
    /** Confirms a bare tree URI yields the identifier at its end. */
    @Test
    fun treeIdReadsTreeRootWithoutDocument() {
        assertEquals("primary%3AMusic", treeIdOf(TREE_HELD))
    }

    // What:     `treeIdNullWithoutTreeSegment` checks a URI with no tree segment.
    // Why:      A URI without the segment has no grant identity, so the function reports absence.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no tree", () => expect(treeIdOf("content://a/document/b")).toBeNull());
    // ```
    /** Confirms a URI without the tree segment yields no identifier. */
    @Test
    fun treeIdNullWithoutTreeSegment() {
        assertNull(treeIdOf("content://a/document/b"))
    }

    // What:     `treeIdNullWhenEmpty` checks an empty tree identifier.
    // Why:      An empty identifier would match every empty tree, so it is reported as absent.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty tree id", () => expect(treeIdOf("content://a/tree//document/b")).toBeNull());
    // ```
    /** Confirms an empty tree identifier is reported as absent. */
    @Test
    fun treeIdNullWhenEmpty() {
        assertNull(treeIdOf("content://a/tree//document/b"))
    }

    // What:     `fileNameTakesLastSegment` checks a display path with folders.
    // Why:      The menu title and Copy filename use only the final segment.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("file name", () => expect(trackFileNameOf("Album/Song.mp3")).toBe("Song.mp3"));
    // ```
    /** Confirms the file name is the text after the last slash. */
    @Test
    fun fileNameTakesLastSegment() {
        assertEquals("Song.mp3", trackFileNameOf("Album/Disc 1/Song.mp3"))
    }

    // What:     `fileNameWithoutSlashIsWhole` checks a display path with no folder.
    // Why:      A track directly in the root has no slash, and its name is the whole path.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("bare name", () => expect(trackFileNameOf("Song.mp3")).toBe("Song.mp3"));
    // ```
    /** Confirms a path with no slash is its own file name. */
    @Test
    fun fileNameWithoutSlashIsWhole() {
        assertEquals("Song.mp3", trackFileNameOf("Song.mp3"))
    }

    // What:     `mimeTypeForKnownExtensions` checks several accepted extensions, including upper case.
    // Why:      The copy-back creates documents with these types, and the extension match is case-insensitive.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("mime", () => expect(mimeTypeFor("A.FLAC")).toBe("audio/flac"));
    // ```
    /** Confirms known extensions map to their audio types regardless of letter case. */
    @Test
    fun mimeTypeForKnownExtensions() {
        assertEquals("audio/flac", mimeTypeFor("A.FLAC"))
        assertEquals("audio/mpeg", mimeTypeFor("b.mp3"))
        assertEquals("audio/mp4", mimeTypeFor("c.m4a"))
    }

    // What:     `mimeTypeForUnknownExtensionIsGeneric` checks an extension the table does not know.
    // Why:      A document must still get a type, and the generic one is the neutral fallback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unknown mime", () => expect(mimeTypeFor("x.xyz")).toBe("application/octet-stream"));
    // ```
    /** Confirms an unknown extension yields the generic type. */
    @Test
    fun mimeTypeForUnknownExtensionIsGeneric() {
        assertEquals("application/octet-stream", mimeTypeFor("x.xyz"))
    }

    // What:     `mimeTypeForNameWithoutDotIsGeneric` checks a name with no extension.
    // Why:      The extension is empty, so the lookup must not match an accidental key.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no dot", () => expect(mimeTypeFor("track")).toBe("application/octet-stream"));
    // ```
    /** Confirms a name without a dot yields the generic type. */
    @Test
    fun mimeTypeForNameWithoutDotIsGeneric() {
        assertEquals("application/octet-stream", mimeTypeFor("track"))
    }

    // What:     `mimeTypeCoversEveryAcceptedExtension` checks that the table covers every accepted audio extension.
    // Why:      An accepted extension without a type would make Undo fall back to a generic type for that file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("coverage", () => for (const ext of AUDIO_EXTENSIONS) expect(mimeTypeFor(`a.${ext}`)).not.toBe(generic));
    // ```
    /** Confirms every extension in the accepted audio set maps to a specific type. */
    @Test
    fun mimeTypeCoversEveryAcceptedExtension() {
        for (extension in AUDIO_EXTENSIONS) {
            assertFalse("no MIME type for $extension", mimeTypeFor("a.$extension") == TRASH_MIME_DEFAULT)
        }
    }

    // What:     `shortNameKeepsShortName` checks a name that fits the limit.
    // Why:      A short name must reach the message unchanged.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("short", () => expect(shortName("a.mp3")).toBe("a.mp3"));
    // ```
    /** Confirms a name within the limit is returned as is. */
    @Test
    fun shortNameKeepsShortName() {
        assertEquals("a.mp3", shortName("a.mp3"))
    }

    // What:     `shortNameTruncatesLongName` checks a name over the limit.
    // Why:      The shortened name must end with the ellipsis and stay exactly at the limit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("long", () => expect(shortName("x".repeat(60)).length).toBe(48));
    // ```
    /** Confirms a long name is cut to the limit and ends with the ellipsis. */
    @Test
    fun shortNameTruncatesLongName() {
        val shortened: String = shortName("x".repeat(60))
        assertEquals(TRASH_NAME_MAX_CHARS, shortened.length)
        assertTrue(shortened.endsWith(TRASH_ELLIPSIS))
    }

    // What:     `copyMatchesEqualSize` checks a copy whose byte count equals the reported size.
    // Why:      Equal sizes are the only evidence that lets the original be deleted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("match", () => expect(copyMatches(10, 10)).toBe(true));
    // ```
    /** Confirms equal sizes verify the copy. */
    @Test
    fun copyMatchesEqualSize() {
        assertTrue(copyMatches(expectedBytes = 10L, copiedBytes = 10L))
    }

    // What:     `copyMatchesRejectsUnknownSize` checks a provider that reports no size.
    // Why:      Without a reported size the copy cannot be verified, so the delete must not run.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unknown", () => expect(copyMatches(null, 10)).toBe(false));
    // ```
    /** Confirms an unknown reported size never verifies a copy. */
    @Test
    fun copyMatchesRejectsUnknownSize() {
        assertFalse(copyMatches(expectedBytes = null, copiedBytes = 10L))
    }

    // What:     `copyMatchesRejectsMismatch` checks a copy shorter than the reported size.
    // Why:      A short copy means the bytes are incomplete, and deleting the original would lose data.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("mismatch", () => expect(copyMatches(10, 9)).toBe(false));
    // ```
    /** Confirms a shorter copy does not verify. */
    @Test
    fun copyMatchesRejectsMismatch() {
        assertFalse(copyMatches(expectedBytes = 10L, copiedBytes = 9L))
    }

    // What:     `copyMatchesRejectsNegativeExpectedSize` checks a reported size that is not a real size.
    // Why:      A negative size is a provider error, and it must not verify any copy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("negative", () => expect(copyMatches(-1, -1)).toBe(false));
    // ```
    /** Confirms a negative reported size never verifies a copy. */
    @Test
    fun copyMatchesRejectsNegativeExpectedSize() {
        assertFalse(copyMatches(expectedBytes = -1L, copiedBytes = -1L))
    }

    // What:     `recordRejectsBlankId` checks that a record needs an id.
    // Why:      The id names the message and the cache folder, so a blank one would break cleanup.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("blank id", () => expect(() => record({ id: " " })).toThrow());
    // ```
    /** Confirms a blank id is rejected at construction. */
    @Test
    fun recordRejectsBlankId() {
        assertThrows(IllegalArgumentException::class.java) { sampleRecord(id = " ") }
    }

    // What:     `recordRejectsBlankName` checks that a record needs a file name.
    // Why:      Undo recreates the document under this name, so a blank name cannot be restored.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("blank name", () => expect(() => record({ displayName: "" })).toThrow());
    // ```
    /** Confirms a blank display name is rejected at construction. */
    @Test
    fun recordRejectsBlankName() {
        assertThrows(IllegalArgumentException::class.java) { sampleRecord(displayName = "") }
    }

    // What:     `recordRejectsNegativeSize` checks that a record's size is not negative.
    // Why:      A negative size can never match a restored document, so the record would never verify.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("negative size", () => expect(() => record({ sizeBytes: -1 })).toThrow());
    // ```
    /** Confirms a negative size is rejected at construction. */
    @Test
    fun recordRejectsNegativeSize() {
        assertThrows(IllegalArgumentException::class.java) { sampleRecord(sizeBytes = -1L) }
    }

    // What:     `beginRestoreMovesPendingToRestoring` checks the start of an Undo.
    // Why:      A restoring record is protected from cleanup, so the state change must happen first.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("begin", () => expect(record().beginRestore().state).toBe("RESTORING"));
    // ```
    /** Confirms a pending record moves to restoring. */
    @Test
    fun beginRestoreMovesPendingToRestoring() {
        assertEquals(TrashUndoState.RESTORING, sampleRecord().beginRestore().state)
    }

    // What:     `beginRestoreTwiceIsRejected` checks that a second Undo cannot start while one runs.
    // Why:      Two restores of one file would create two documents, so the second start must fail.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("twice", () => expect(() => record().beginRestore().beginRestore()).toThrow());
    // ```
    /** Confirms a restoring record cannot begin another restore. */
    @Test
    fun beginRestoreTwiceIsRejected() {
        val restoring: TrashRecord = sampleRecord().beginRestore()
        assertThrows(IllegalArgumentException::class.java) { restoring.beginRestore() }
    }

    // What:     `finishRestoreRecordsRestored` checks a successful Undo.
    // Why:      A restored record offers no further Undo, so the state must read restored.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("restored", () => expect(record().beginRestore().finishRestore(true).state).toBe("RESTORED"));
    // ```
    /** Confirms a successful restore ends in the restored state. */
    @Test
    fun finishRestoreRecordsRestored() {
        assertEquals(TrashUndoState.RESTORED, sampleRecord().beginRestore().finishRestore(restored = true).state)
    }

    // What:     `finishRestoreRecordsFailed` checks a failed Undo.
    // Why:      A failed record must not look restored, and it offers no retry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("failed", () => expect(record().beginRestore().finishRestore(false).state).toBe("FAILED"));
    // ```
    /** Confirms a failed restore ends in the failed state. */
    @Test
    fun finishRestoreRecordsFailed() {
        assertEquals(TrashUndoState.FAILED, sampleRecord().beginRestore().finishRestore(restored = false).state)
    }

    // What:     `finishRestoreFromPendingIsRejected` checks that an outcome cannot be recorded without a restore.
    // Why:      A pending record that was never restoring has no outcome to record, so the call must fail.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("finish pending", () => expect(() => record().finishRestore(true)).toThrow());
    // ```
    /** Confirms finishing a record that is not restoring is rejected. */
    @Test
    fun finishRestoreFromPendingIsRejected() {
        assertThrows(IllegalArgumentException::class.java) { sampleRecord().finishRestore(restored = true) }
    }

    // What:     `restoredRecordCannotRestoreAgain` checks that a finished Undo cannot run again.
    // Why:      A restored file already exists at its place, so a second restore would duplicate it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no re-restore", () => expect(() => record().beginRestore().finishRestore(true).beginRestore()).toThrow());
    // ```
    /** Confirms a restored record cannot begin a restore again. */
    @Test
    fun restoredRecordCannotRestoreAgain() {
        val restored: TrashRecord = sampleRecord().beginRestore().finishRestore(restored = true)
        assertThrows(IllegalArgumentException::class.java) { restored.beginRestore() }
    }

    // What:     `messageIdUsesRecordId` checks that each record's Undo message has its own id.
    // Why:      A message found by id must be the one for this record, never another trash.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("message id", () => expect(record({ id: "r1" }).messageId).toBe("trash-undo-r1"));
    // ```
    /** Confirms the Undo message id is built from the record id. */
    @Test
    fun messageIdUsesRecordId() {
        assertEquals("trash-undo-r1", sampleRecord(id = "r1").messageId)
    }

    // What:     `sampleRecord(...)` builds a valid pending record for the tests in this class.
    // Why:      Each record test changes one field, so one helper keeps the baseline in one place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function sampleRecord(overrides?: Partial<TrashRecord>): TrashRecord { ... }
    // ```
    /** Builds a pending record for the Android SAF route with the given overrides applied. */
    private fun sampleRecord(
        id: String = "r1",
        displayName: String = "a.mp3",
        sizeBytes: Long = 10L,
    ): TrashRecord = TrashRecord(
        id = id,
        displayName = displayName,
        route = TrashRoute.SAF_DELETE_WITH_COPY_BACK,
        trackUri = DOC_IN_HELD,
        treeUri = TREE_HELD,
        parentDocumentId = "primary:Music/Album",
        mimeType = "audio/mpeg",
        copyPath = "/cache/trash/r1/a.mp3",
        sizeBytes = sizeBytes,
    )
}
