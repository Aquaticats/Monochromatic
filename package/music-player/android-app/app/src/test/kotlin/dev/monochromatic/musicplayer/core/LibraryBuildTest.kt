// The library build derives every list the player reads, off the main thread, and must agree with the first way of
// deriving them: relative paths from the reference implementation, pages from `paginate`, and the page of an index
// from `pageOfIndex`. The queue's prepared form must answer exactly like its computed form.

// What:     `package ...core` places the cases beside the build.
// Why:      They call the build, the queue and the reference code through the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.Track
import java.util.Random
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** Verifies the build and the queue's prepared pages. */
class LibraryBuildTest {
    /** A random library with unique URIs in load order. */
    private fun randomTracks(random: Random, size: Int): List<Track> =
        List(size) { Track("uri-$it", (if (random.nextBoolean()) "Music/" else "") + randomPath(random)) }

    /** Every derived list equals the first way of deriving it, on random libraries with awkward paths. */
    @Test
    fun theBuildAgreesWithTheReferenceDerivations() {
        val random = Random(21L)
        repeat(200) { round ->
            val tracks = randomTracks(random, 1 + random.nextInt(60))
            val built = buildLibrary(tracks)
            assertEquals(tracks.map { it.uri }, built.uris)
            assertEquals(tracks.map { it.displayPath }, built.rawPaths)
            assertEquals("round $round", referenceRelativeDisplayPaths(built.rawPaths), built.displayPaths)
            assertEquals(paginate(built.displayPaths), built.pages)
            for (index in tracks.indices) {
                assertEquals("round $round index $index", pageOfIndex(built.pages, index), built.pageOfIndex[index].takeIf { it >= 0 })
            }
        }
    }

    /** The build can locate a URI so the main thread does not have to scan for it. */
    @Test
    fun theBuildLocatesTheHintedUri() {
        val tracks = List(50) { Track("u$it", "A/$it.flac") }
        assertEquals(17, buildLibrary(tracks, "u17").hintIndex)
        assertEquals(-1, buildLibrary(tracks, "missing").hintIndex)
        assertNull(buildLibrary(tracks).hintUri)
    }

    /** A queue given the prepared lists behaves exactly like a queue that computes them. */
    @Test
    fun aPreparedQueueEqualsAComputedQueue() {
        val random = Random(33L)
        repeat(100) { round ->
            val tracks = randomTracks(random, 2 + random.nextInt(40))
            val built = buildLibrary(tracks)
            val computed = Queue.withRngSeed(1L)
            val prepared = Queue.withRngSeed(1L)
            computed.setTracks(built.rawPaths)
            prepared.setTracks(built.rawPaths, built)
            assertEquals("round $round", computed.displayPaths(), prepared.displayPaths())
            assertEquals(computed.pages(), prepared.pages())
            val pick = random.nextInt(tracks.size)
            assertEquals(computed.playIndex(pick), prepared.playIndex(pick))
            assertEquals(computed.playbackOrder(), prepared.playbackOrder())
            assertEquals(computed.currentIndex(), prepared.currentIndex())
            // Replacing the tracks keeps the displayed page by identity in both.
            val next = buildLibrary(randomTracks(random, 2 + random.nextInt(40)))
            computed.setTracks(next.rawPaths)
            prepared.setTracks(next.rawPaths, next)
            assertEquals(computed.playbackOrder(), prepared.playbackOrder())
        }
    }
}
