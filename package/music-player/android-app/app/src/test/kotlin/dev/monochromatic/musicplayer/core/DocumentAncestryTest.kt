package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies the API 26 to 28 fallback for the provider's containment check, as a pure function of two ids. */
class DocumentAncestryTest {
    /** Confirms a volume root's children and deeper descendants are inside it. */
    @Test
    fun volumeRootContainsItsDescendants() {
        assertTrue(isDescendantDocumentId(parentId = "primary:", documentId = "primary:Music"))
        assertTrue(isDescendantDocumentId(parentId = "primary:", documentId = "primary:Music/a/b.flac"))
    }

    /** Confirms a folder contains its children and deeper descendants. */
    @Test
    fun folderContainsItsDescendants() {
        assertTrue(isDescendantDocumentId(parentId = "primary:Music", documentId = "primary:Music/a.flac"))
        assertTrue(isDescendantDocumentId(parentId = "primary:Music", documentId = "primary:Music/Rock/a.flac"))
    }

    /** Confirms a sibling whose name only starts with the folder name is not inside the folder. */
    @Test
    fun siblingWithSharedNamePrefixIsNotInside() {
        assertFalse(isDescendantDocumentId(parentId = "primary:Music", documentId = "primary:Music2/a.flac"))
        assertFalse(isDescendantDocumentId(parentId = "primary:Music", documentId = "primary:MusicBox"))
    }

    /** Confirms a document is not its own descendant and an empty remainder does not count. */
    @Test
    fun documentIsNotItsOwnDescendant() {
        assertFalse(isDescendantDocumentId(parentId = "primary:Music", documentId = "primary:Music"))
        assertFalse(isDescendantDocumentId(parentId = "primary:Music", documentId = "primary:Music/"))
        assertFalse(isDescendantDocumentId(parentId = "primary:", documentId = "primary:"))
    }

    /** Confirms an ancestor is not inside its own descendant and another volume is never inside this one. */
    @Test
    fun unrelatedIdsAreNotInside() {
        assertFalse(isDescendantDocumentId(parentId = "primary:Music/Rock", documentId = "primary:Music"))
        assertFalse(isDescendantDocumentId(parentId = "primary:", documentId = "1A2B-3C4D:Music"))
    }

    /** Confirms the prefix is the id itself for a volume root and the id plus a slash for a folder. */
    @Test
    fun prefixEndsWithSeparator() {
        assertEquals("primary:", documentIdPrefixOf("primary:"))
        assertEquals("primary:Music/", documentIdPrefixOf("primary:Music"))
    }
}
