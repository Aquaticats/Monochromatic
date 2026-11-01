// A library whose only content is one top-level folder shows that folder's own name (D86, D3, D28), as the desktop
// player does when it is given the opened folder; its tracks keep their names relative to it.

// What:     `package ...core` matches the rule's package.
// Why:      The tests call the pure functions directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.Track
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** URI of a track of a granted folder. */
private fun safUri(name: String): String =
    "content://com.android.externalstorage.documents/tree/primary%3AMusic/document/primary%3AMusic%2F$name"

/** URI of a track of the system audio library. */
private fun mediaUri(id: Int): String = "content://media/external/audio/media/$id"

/** Checks the library root rule: what a one-folder library shows, and what stays as it was. */
class DisplayRootTest {
    /** The desktop's own case: one top-level folder under the opened folder keeps its name. */
    @Test
    fun aSingleTopLevelFolderKeepsItsNameRelativeToTheRoot() {
        /** Paths of a library whose opened folder is `lib/Music` and whose only folder is `Artist`. */
        val paths = listOf("lib/Music/Artist/01.flac", "lib/Music/Artist/Disc 2/02.flac")
        assertEquals(
            listOf("Artist/01.flac", "Artist/Disc 2/02.flac"),
            relativeDisplayPaths(paths, listOf("lib", "Music")),
        )
        assertEquals(
            "without a root the shared directory is stripped",
            listOf("01.flac", "Disc 2/02.flac"),
            relativeDisplayPaths(paths),
        )
    }

    /** Loose files of the root have bare names next to folder tracks, so they sit in the letter folders. */
    @Test
    fun looseFilesHaveBareNamesRelativeToTheRoot() {
        assertEquals(
            listOf("loose.flac", "A/x.flac"),
            relativeDisplayPaths(listOf("lib/Music/loose.flac", "lib/Music/A/x.flac"), listOf("lib", "Music")),
        )
    }

    /** A track outside the root makes the shared directory the prefix, so every name stays unambiguous. */
    @Test
    fun aRootThatDoesNotHoldEveryTrackFallsBackToTheSharedDirectory() {
        /** One track under the root and one elsewhere. */
        val paths = listOf("lib/Music/A/x.flac", "elsewhere/y.flac")
        assertEquals(relativeDisplayPaths(paths), relativeDisplayPaths(paths, listOf("lib", "Music")))
    }

    /** A track that is the root itself is not under it, so a zero-length name cannot appear. */
    @Test
    fun aTrackThatIsTheRootItselfFallsBack() {
        /** One path equal to the root. */
        val paths = listOf("lib/Music")
        assertEquals(relativeDisplayPaths(paths), relativeDisplayPaths(paths, listOf("lib", "Music")))
    }

    /** The empty root of a granted folder strips nothing, however many segments every path shares. */
    @Test
    fun anEmptyRootStripsNothing() {
        assertEquals(
            listOf("Only/a.flac", "Only/b.flac"),
            relativeDisplayPaths(listOf("Only/a.flac", "Only/b.flac"), emptyList()),
        )
    }

    /** A feed that adds tracks one at a time agrees with the one-pass result at every step, with a root. */
    @Test
    fun theIncrementalPrefixAgreesWithTheOnePassResultWithARoot() {
        /** Tracks in arrival order: the library is one folder until a loose file and a folder outside Music arrive. */
        val arrivals = listOf("Music/Artist/a.flac", "Music/Artist/Disc 2/b.flac", "Music/c.flac", "Download/d.flac")
        val root: List<String> = listOf("Music")
        val prefix = CommonPrefix(root)
        for (count in arrivals.indices) {
            prefix.add(arrivals[count])
            /** The tracks found so far. */
            val soFar: List<String> = arrivals.take(count + 1)
            assertEquals(
                "after $count",
                relativeDisplayPaths(soFar, root),
                soFar.map { path -> relativeOf(path, prefix.count) },
            )
        }
    }

    /** A granted folder with one top-level folder is built with that folder as its one page, not as loose letters. */
    @Test
    fun aGrantedFolderWithOneFolderShowsTheFolderPage() {
        /** Tracks of a grant whose only content is the folder `Artist`. */
        val tracks = listOf(
            Track(safUri("Artist%2Fa.flac"), "Artist/a.flac"),
            Track(safUri("Artist%2Fb.flac"), "Artist/b.flac"),
        )
        val built = buildLibrary(tracks)
        assertEquals(listOf("Artist/a.flac", "Artist/b.flac"), built.displayPaths)
        assertEquals(listOf("Artist"), built.pages.map { page -> page.label })
    }

    /** The system library of one folder under Music shows that folder, and a many-artist one still shows artists. */
    @Test
    fun theSystemLibraryIsRelativeToMusic() {
        /** One folder under Music. */
        val one = buildLibrary(
            listOf(Track(mediaUri(1), "Music/Artist/a.flac"), Track(mediaUri(2), "Music/Artist/b.flac")),
        )
        assertEquals(listOf("Artist/a.flac", "Artist/b.flac"), one.displayPaths)
        assertEquals(listOf("Artist"), one.pages.map { page -> page.label })
        /** Two artists under Music. */
        val many = buildLibrary(listOf(Track(mediaUri(1), "Music/Rock/a.flac"), Track(mediaUri(2), "Music/Pop/b.flac")))
        assertEquals(listOf("Rock/a.flac", "Pop/b.flac"), many.displayPaths)
        /** Loose files in Music beside a folder keep their bare names. */
        val loose = buildLibrary(listOf(Track(mediaUri(1), "Music/loose.flac"), Track(mediaUri(2), "Music/A/x.flac")))
        assertEquals(listOf("loose.flac", "A/x.flac"), loose.displayPaths)
    }

    /** Which source a URI names decides the root; an unknown source has none. */
    @Test
    fun theRootFollowsTheSourceOfTheUri() {
        assertEquals(emptyList<String>(), displayRootOf(safUri("a.flac")))
        assertEquals(listOf("Music"), displayRootOf(mediaUri(7)))
        assertNull(displayRootOf("uri-1"))
    }
}
