// The fast relative-path code must give exactly the first implementation's answers on every kind of path, including
// empty, `.` and `..` segments, leading and trailing separators, and one-segment libraries.

// What:     `package ...core` places the cases beside the code they check.
// Why:      The cases call the internal prefix and path functions through the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import java.util.Random
import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Test

/** Segments that random paths are built from, with the awkward ones over-represented. */
private val SEGMENTS: List<String> = listOf("a", "b", "Music", "Rock", "x.flac", ".", "..", "", "a b", "é", "..a", ".a")

/** Builds a random path of zero to five segments, with optional leading and trailing separators. */
private fun oddPath(random: Random): String {
    val count = random.nextInt(6)
    val body = List(count) { SEGMENTS[random.nextInt(SEGMENTS.size)] }.joinToString("/")
    val lead = if (random.nextInt(8) == 0) "/" else ""
    val tail = if (random.nextInt(8) == 0) "/" else ""
    return lead + body + tail
}

/** Verifies the fast relative paths against the first implementation. */
class RelPathFastTest {
    /** Compares whole lists of random paths, many of which share a first segment so a prefix is found. */
    @Test
    fun randomListsMatchTheFirstImplementation() {
        val random = Random(5L)
        repeat(3000) { round ->
            val size = 1 + random.nextInt(12)
            val shared = if (random.nextBoolean()) "Music/" else ""
            val paths = List(size) { shared + oddPath(random) }
            assertEquals("round $round $paths", referenceRelativeDisplayPaths(paths), relativeDisplayPaths(paths))
        }
    }

    /** The prefix kept one path at a time equals the prefix of the paths so far, at every length of the list. */
    @Test
    fun theIncrementalPrefixMatchesAtEveryLength() {
        val random = Random(6L)
        repeat(500) { round ->
            val paths = List(1 + random.nextInt(10)) { (if (random.nextBoolean()) "Music/" else "") + oddPath(random) }
            val prefix = CommonPrefix()
            for ((position, path) in paths.withIndex()) {
                prefix.add(path)
                val soFar = paths.subList(0, position + 1)
                val expected = referenceRelativeDisplayPaths(soFar)
                assertEquals("round $round $soFar", expected, soFar.map { relativeOf(it, prefix.count) })
            }
        }
    }

    /** A path that needs no change comes back as the same string, which is what keeps a million paths from copying. */
    @Test
    fun anUnchangedPathIsTheSameString() {
        val path = "Artist/Album/01 Song.flac"
        assertSame(path, relativeOf(path, 0))
        assertEquals(listOf(path), relativeDisplayPaths(listOf(path, "Other/x.flac")).take(1))
    }

    /** One folder shared by all tracks is stripped but the file name always survives. */
    @Test
    fun theFileNameSurvivesAnyPrefix() {
        assertEquals(listOf("a.flac"), relativeDisplayPaths(listOf("Music/Rock/a.flac")))
        assertEquals(listOf("Rock/a.flac", "Pop/b.flac"), relativeDisplayPaths(listOf("Music/Rock/a.flac", "Music/Pop/b.flac")))
        assertEquals(emptyList<String>(), relativeDisplayPaths(emptyList()))
        assertEquals(listOf("//"), relativeDisplayPaths(listOf("//")))
    }
}
