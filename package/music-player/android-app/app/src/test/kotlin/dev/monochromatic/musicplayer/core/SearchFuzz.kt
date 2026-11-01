// Randomized libraries and queries for the Search parity tests: small vocabularies so names collide and tie,
// plus accents, compatibility forms, combining marks, separators, dot segments, supplementary and lone surrogates.

// What:     `package ...core` places the generators beside the tests that use them.
// Why:      The tests reach the library types through the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.Track
import java.util.Random

/** Word pieces that names are built from: plain letters, accents, compatibility forms, kana, Han and Cyrillic. */
private val WORDS: List<String> = listOf(
    "a", "ab", "abc", "camellia", "cam", "scamper", "live", "at", "the", "lin", "ka", "kalo", "Ka", "KA", "éa",
    "école", "Ångström", "naïve", "ñandú", "ßeta", "ﬁne", "Ⅳ", "ｶﾞ", "が", "か", "ア", "夜光", "ма", "Σίσυφος", "İstanbul",
    "éclair", "ö", "x", "y", "zz", "1", "2024", "00", "😀smile", "😀", "z​z",
)

/** Separators and odd joiners between the words of a name. */
private val JOINERS: List<String> = listOf(" ", " ", " ", "-", "_", ".", "(", "[", "  ", " ", "\t", "--", " - ")

/** Segments that make a path odd: dots, empties, the root-group lookalike and lone surrogates. */
private val ODD_SEGMENTS: List<String> = listOf(".", "..", "", "Unfiled", "unfiled", "Unfiled ", "\uD800x", "\uDC00y")

/** File extensions, including none and a hidden-file shape. */
private val EXTENSIONS: List<String> = listOf(".flac", ".opus", ".mp3", ".m4a", "", ".", ".tar.gz")

/** Builds one random word-and-joiner phrase of one to four words. */
private fun phrase(random: Random): String {
    /** Words in this phrase. */
    val count: Int = 1 + random.nextInt(4)
    val builder = StringBuilder()
    for (position in 0 until count) {
        if (position > 0) {
            builder.append(JOINERS[random.nextInt(JOINERS.size)])
        }
        builder.append(WORDS[random.nextInt(WORDS.size)])
    }
    return builder.toString()
}

/** Paths that decide which virtual letter folder a loose track sits in, and folders named like the old group. */
private val UNFILED_PATHS: List<String> = listOf(
    "Unfiled/", "Unfiled/.", "Unfiled//", "./Unfiled/x.flac", "Unfiled/x.flac", "Unfiled/a/b.flac", "Unfiled/..",
    "unfiled/x.flac", "x.flac", "Unfiled", "\u00c9milie.flac", "\u00e9lan.flac", "\u00f1and\u00fa.flac",
    "\u521d\u97f3.flac", "1 song.flac", "\u00d8.flac", "\u00df.flac", "E/x.flac", "Eno.flac",
)

/** Builds one random display path: zero to three folder segments, then a file name. */
fun randomPath(random: Random): String {
    if (random.nextInt(UNFILED_ONE_IN) == 0) {
        return UNFILED_PATHS[random.nextInt(UNFILED_PATHS.size)]
    }
    /** Number of folder segments, mostly one or two. */
    val depth: Int = random.nextInt(4)
    val builder = StringBuilder()
    for (level in 0 until depth) {
        if (random.nextInt(12) == 0) {
            builder.append(ODD_SEGMENTS[random.nextInt(ODD_SEGMENTS.size)])
        } else {
            builder.append(phrase(random))
        }
        builder.append('/')
    }
    /** Optional leading space or dot to exercise trimming and hidden files. */
    val lead: String = if (random.nextInt(15) == 0) JOINERS[random.nextInt(JOINERS.size)] else ""
    builder.append(lead).append(phrase(random)).append(EXTENSIONS[random.nextInt(EXTENSIONS.size)])
    return builder.toString()
}

/** Builds a random library of [size] tracks, with unique URIs and display paths that may repeat. */
fun randomFuzzLibrary(random: Random, size: Int): List<Track> =
    List(size) { position -> Track("uri-$position", randomPath(random)) }

/** Builds one random query from the library: a word-start slice of a name, a few words, or hostile text. */
fun randomQuery(random: Random, tracks: List<Track>): String {
    /** The choice between the query shapes. */
    val shape: Int = random.nextInt(10)
    if (shape == 0) {
        return WORDS[random.nextInt(WORDS.size)] + JOINERS[random.nextInt(JOINERS.size)] +
            WORDS[random.nextInt(WORDS.size)]
    }
    if (shape == 1) {
        return String(CharArray(1 + random.nextInt(3)) { HOSTILE_UNITS[random.nextInt(HOSTILE_UNITS.length)] })
    }
    /** A name to cut a slice from. */
    val name: String = tracks[random.nextInt(tracks.size)].displayPath.substringAfterLast('/')
    if (name.isEmpty()) {
        return "a"
    }
    /** Where the slice starts: a word start most of the time. */
    val start: Int = random.nextInt(name.length)
    /** How many units the slice holds. */
    val length: Int = 1 + random.nextInt(8)
    /** The slice, cut at the end of the name. */
    val slice: String = name.substring(start, minOf(name.length, start + length))
    return when (random.nextInt(4)) {
        0 -> slice.uppercase()
        1 -> " $slice "
        2 -> slice + " " + WORDS[random.nextInt(WORDS.size)]
        else -> slice
    }
}

/** One path in this many is drawn from the Unfiled lookalikes. */
private const val UNFILED_ONE_IN: Int = 30

/** Units that make hostile queries: separators, regex metacharacters, controls, surrogates and marks. */
private const val HOSTILE_UNITS: String = " -_.([/\\*?+^$|{}\u0000́😀\uDC00 \t"
