// Parity of the in-memory Search index with the old matcher (D112): the same tier, kind, name, library position, count,
// truncated flag and highlight ranges for randomized libraries and queries, for libraries built in one batch or many,
// and for answers served from the one-character cache while the library grows.

// What:     `package ...core` places the parity cases beside the index and the oracle.
// Why:      The cases call both implementations through the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.Track
import java.text.Collator
import java.util.Random
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** Seeds that the randomized cases run, each an independent library. */
private const val SEED_COUNT: Int = 40

/** Random queries compared per seed. */
private const val QUERIES_PER_SEED: Int = 60

/** Counts what the randomized answers exercised, so the cases prove they reached the paths they claim to test. */
private object Coverage {
    /** Answers with the truncated flag set. */
    var truncated = 0

    /** Answers that contain a folder result. */
    var folders = 0

    /** Answers that contain a loose root-level track, whose folder is its virtual letter. */
    var looseTracks = 0

    /** Answers to queries with more than one term. */
    var multiTerm = 0

    /** Answers to queries made only of separators. */
    var separatorOnly = 0

    /** Answers with at least two results that share a tier and a normalized length. */
    var ties = 0

    /** Answers that are no match. */
    var noMatch = 0

    /** Answers with at least one result. */
    var results = 0

    /** Zeroes every counter. */
    fun reset() {
        truncated = 0
        folders = 0
        looseTracks = 0
        multiTerm = 0
        separatorOnly = 0
        ties = 0
        noMatch = 0
        results = 0
    }
}

/** Compares the index answer and the oracle answer for [query] and fails with the query in the message. */
private fun assertSameAnswer(index: SearchIndex, tracks: List<Track>, query: String, label: String) {
    val expected = referenceSearchLibrary(query, tracks, true)
    assertEquals("$label: query ${query.map { it.code }}", expected, index.search(query))
    val terms = searchTerms(query)
    if (terms.size > 1) Coverage.multiTerm += 1
    if (terms.isNotEmpty() && terms.all { term -> term.all { unit -> isWordSeparator(unit) } }) Coverage.separatorOnly += 1
    if (expected is SearchState.NoMatch) Coverage.noMatch += 1
    if (expected is SearchState.Results) {
        Coverage.results += 1
        if (expected.truncated) Coverage.truncated += 1
        val folders = expected.items.filter { it.kind == SearchResultKind.FOLDER }
        if (folders.isNotEmpty()) Coverage.folders += 1
        if (expected.items.any { it.kind == SearchResultKind.TRACK && it.parentContext == null }) Coverage.looseTracks += 1
        if (expected.items.zipWithNext().any { (a, b) -> a.tier == b.tier && a.name.length == b.name.length }) Coverage.ties += 1
    }
}

/** Verifies that the index answers every query exactly like the old matcher. */
class SearchIndexParityTest {
    /**
     * Compares random queries on libraries of several sizes built in a single batch, then checks that the queries
     * reached truncation, ties, folders, loose tracks, several terms and separator-only terms.
     */
    @Test
    fun randomLibrariesMatchTheOracle() {
        Coverage.reset()
        for (seed in 0 until SEED_COUNT) {
            /** Generator for this seed. */
            val random = Random(seed.toLong())
            /** Library size: tiny libraries tie a lot, larger ones fill buckets. */
            val size: Int = intArrayOf(3, 20, 200, 800, 2500)[seed % 5]
            val tracks: List<Track> = randomFuzzLibrary(random, size)
            val index = SearchIndex()
            index.appendPaths(tracks.map { it.displayPath }, tracks.map { it.uri })
            repeat(if (size > 1000) QUERIES_PER_SEED / 3 else QUERIES_PER_SEED) {
                assertSameAnswer(index, tracks, randomQuery(random, tracks), "seed $seed size $size")
            }
        }
        assertTrue("truncated answers: ${Coverage.truncated}", Coverage.truncated > 0)
        assertTrue("answers with folders: ${Coverage.folders}", Coverage.folders > 0)
        assertTrue("answers with loose tracks: ${Coverage.looseTracks}", Coverage.looseTracks > 0)
        assertTrue("multi-term answers: ${Coverage.multiTerm}", Coverage.multiTerm > 0)
        assertTrue("separator-only answers: ${Coverage.separatorOnly}", Coverage.separatorOnly > 0)
        assertTrue("answers with ties: ${Coverage.ties}", Coverage.ties > 0)
        assertTrue("no-match answers: ${Coverage.noMatch}", Coverage.noMatch > 0)
        assertTrue("answers with results: ${Coverage.results}", Coverage.results > 0)
    }

    /** Builds the index in random batches and compares after every batch, which also exercises the cache merge. */
    @Test
    fun incrementalAppendMatchesTheOracleAfterEveryBatch() {
        for (seed in 0 until SEED_COUNT / 2) {
            val random = Random(1000L + seed)
            val tracks: List<Track> = randomFuzzLibrary(random, 600)
            val index = SearchIndex()
            var known = 0
            while (known < tracks.size) {
                /** Size of the next batch, from a single track to a hundred. */
                val step: Int = minOf(tracks.size - known, 1 + random.nextInt(100))
                index.appendPaths(
                    tracks.subList(known, known + step).map { it.displayPath },
                    tracks.subList(known, known + step).map { it.uri },
                )
                known += step
                /** The prefix of the library that the index now holds. */
                val prefix: List<Track> = tracks.subList(0, known)
                repeat(6) { assertSameAnswer(index, prefix, randomQuery(random, prefix), "seed $seed known $known") }
                for (unit in listOf('a', 'c', 'e', 'k', 'l', 'x', 'é', 'z')) {
                    assertSameAnswer(index, prefix, unit.toString(), "seed $seed known $known unit $unit")
                }
            }
        }
    }

    /**
     * An index built from raw paths and then given relative paths answers exactly like an index built from the relative
     * paths, including after cached one-character answers were computed under the old paths.
     */
    @Test
    fun replacingPathsEqualsBuildingWithThem() {
        for (seed in 0 until SEED_COUNT / 2) {
            val random = Random(5000L + seed)
            // Only paths whose title survives the stripping can be replaced in place.
            val raw = randomFuzzLibrary(random, 300)
                .map { track -> Track(track.uri, "Music/" + track.displayPath) }
                .filter { track -> trackTitleFor(track.displayPath) == trackTitleFor(relativeOf(track.displayPath, 1)) }
            val relative = raw.map { track -> Track(track.uri, relativeOf(track.displayPath, 1)) }
            val index = SearchIndex()
            index.appendPaths(raw.map { it.displayPath }, raw.map { it.uri })
            for (unit in listOf("a", "c", "k", "l", "x")) index.search(unit)
            val replacement = index.prepareReplacement(relative.map { it.displayPath })
            assertNotNull(replacement)
            index.applyReplacement(replacement!!)
            repeat(40) { assertSameAnswer(index, relative, randomQuery(random, relative), "replaced seed $seed") }
            for (unit in listOf("a", "c", "k", "l", "x")) assertSameAnswer(index, relative, unit, "replaced unit $unit")
        }
    }

    /** Loose tracks and folders named like a letter or like the old group match the old folder index, one at a time. */
    @Test
    fun looseTrackLettersMatchTheOracle() {
        val libraries = listOf(
            listOf("./Unfiled/x.flac"),
            listOf("Unfiled/x.flac"),
            listOf("Unfiled/a/x.flac"),
            listOf("./Unfiled/x.flac", "Unfiled/y.flac"),
            listOf("./Unfiled/x.flac", "Unfiled/a/y.flac"),
            listOf("x.flac"),
            listOf("Unfiled/"),
            listOf("Unfiled/", "./Unfiled/x.flac"),
            listOf("Unfiled/.", "Other/x.flac"),
            listOf("Unfiled", "Unfiled/z.flac"),
            listOf("unfiled/x.flac"),
            listOf("\u00c9milie.flac", "Eno.flac", "echo.flac", "E/x.flac"),
            listOf("\u521d\u97f3.flac", "1 song.flac", "\u00d8.flac", "\u00df.flac"),
            listOf("A/x.flac", "Abba.flac", "aaa.flac"),
        )
        for (paths in libraries) {
            val tracks = paths.mapIndexed { position, path -> Track("u$position", path) }
            val index = SearchIndex()
            index.appendPaths(paths, tracks.map { it.uri })
            for (query in listOf("unf", "unfiled", "x", "u", "z", "other", "e", "emilie", "a", "e ")) {
                assertSameAnswer(index, tracks, query, "library $paths")
            }
        }
    }

    /** A replacement whose new path has another title than the track is refused, because the buckets would be wrong. */
    @Test
    fun aReplacementThatChangesATitleIsRefused() {
        val index = SearchIndex()
        index.appendPaths(listOf("Music/Unfiled/", "Music/a/b.flac"), listOf("u1", "u2"))
        assertNull(index.prepareReplacement(listOf("Unfiled", "a/b.flac")))
        assertNotNull(index.prepareReplacement(listOf("Music/Unfiled/", "a/b.flac")))
    }

    /** Checks the normalized text and tier of random names and terms against the matcher's own functions. */
    @Test
    fun fastTierAndNormalizedTextMatchTheMatcher() {
        val random = Random(7L)
        val buffer = TextBuffer()
        repeat(4000) {
            val path: String = randomPath(random)
            val title: String = trackTitleFor(path)
            normalizeTitleInto(buffer, path, (titleRangeOf(path) ushr KEY_SHIFT).toInt(), (titleRangeOf(path) and LOW_BITS_MASK).toInt())
            /** Text the index stores for this title. */
            val stored = String(buffer.chars, 0, buffer.size)
            val expected: String = normalizeForSearch(title).trim()
            assertEquals("normalized ${title.map { it.code }}", expected, stored)
            /** A random query and its terms. */
            val terms: List<String> = searchTerms(randomQuery(random, listOf(Track("u", path))))
            if (terms.isNotEmpty()) {
                val termChars: Array<CharArray> = Array(terms.size) { terms[it].toCharArray() }
                val joined: CharArray = terms.joinToString(" ").toCharArray()
                val tier: Int = fastTier(stored.toCharArray(), 0, stored.length, termChars, joined)
                assertEquals("tier of '$stored' for $terms", matchTier(stored, terms) ?: NO_TIER, tier)
            }
        }
    }

    /** Checks that comparing collation keys orders names exactly as the collator compares them. */
    @Test
    fun collationKeysOrderLikeTheCollator() {
        val random = Random(11L)
        val collator: Collator = railCollator()
        val names: List<String> = List(400) { trackTitleFor(randomPath(random)) }
        for (left in names) {
            for (right in names.take(40)) {
                assertEquals(
                    "'$left' vs '$right'",
                    Integer.signum(collator.compare(left, right)),
                    Integer.signum(collator.getCollationKey(left).compareTo(collator.getCollationKey(right))),
                )
            }
        }
    }

    /** Checks the unit tests' own edge: a library with no tracks and a blank query keep the old states. */
    @Test
    fun emptyStatesMatchTheOracle() {
        val index = SearchIndex()
        assertEquals(referenceSearchLibrary("a", emptyList(), true), index.search("a"))
        index.appendPaths(listOf("A/b.flac"), listOf("u"))
        assertEquals(referenceSearchLibrary("  ", listOf(Track("u", "A/b.flac")), true), index.search("  "))
        assertTrue(index.search("zzz") is SearchState.NoMatch)
    }
}
