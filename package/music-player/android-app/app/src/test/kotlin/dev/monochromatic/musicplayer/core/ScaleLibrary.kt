// Synthetic libraries for the scale and Search tests: artist folders with mixed scripts, album subfolders, loose
// root files and long names, built from a seed so every run sees the same tracks.

// What:     `package ...core` places the generator beside the tests that use it.
// Why:      The scale, parity and ingest tests all build their libraries here.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import dev.monochromatic.musicplayer.Track` brings in the library track type the generator builds.
// Why:      `Track` lives in the parent package, so the generator imports it by absolute name.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Track } from "dev.monochromatic.musicplayer/Track";
// ```
import dev.monochromatic.musicplayer.Track

// What:     `import java.util.Random` brings in the seeded generator so every run builds the same library.
// Why:      A fixed seed makes counts and timings comparable across runs without a fixture file.
//
// In TS you'd write (pseudocode):
// ```ts
// import seedrandom from "seedrandom";
// ```
import java.util.Random

/** Seed of the default library, shared by every size so a smaller library is a prefix of the larger ones' style. */
const val SCALE_SEED: Long = 1L

/** Tracks per artist folder at every size, so one folder has the same content at 1k and at 1M. */
const val SCALE_TRACKS_PER_ARTIST: Int = 1000

/** Percentage of the library placed as loose files at the root. */
private const val LOOSE_PERCENT: Int = 3

/** Longest track title in characters. */
private const val MAX_TITLE_CHARS: Int = 200

/** Longest path component in UTF-8 bytes. */
private const val MAX_COMPONENT_BYTES: Int = 255

/** Latin syllables that every Latin artist and title word is built from. */
private val LATIN_SYLLABLES: List<String> =
    listOf("ka", "lo", "ri", "ven", "sa", "mo", "tek", "zu", "ash", "lind", "ora", "quin")

/** Kana characters that Japanese-style artist names and titles are built from. */
private val KANA: List<String> = listOf("あ", "い", "か", "き", "さ", "し", "た", "な", "ア", "カ", "サ", "タ")

/** Han characters that a few Japanese-style names and titles include. */
private val HAN: List<String> = listOf("夜", "光", "風", "海", "空", "月")

/** Cyrillic syllables that synthetic Cyrillic artist names are built from. */
private val CYRILLIC: List<String> = listOf("ма", "ра", "лу", "ну", "ко", "ти", "шо", "зе", "ри", "бо")

/** Symbol prefixes that some artist names start with. */
private val SYMBOLS: List<String> = listOf("!!!", "#", "$", "&", "+", "~", "(", "[", "*")

/** Accented Latin fragments that exercise the decomposition and mark-removal rules in a small share of names. */
private val ACCENTED: List<String> = listOf("é", "ö", "ñ", "ç", "ü", "Å", "ß", "ﬁ", "Ⅳ", "ｶ")

/** One generated library: its artist count, relative names in load order, and the tracks for those names. */
class SyntheticLibrary(
    /** Number of artist folders, not counting root files. */
    val artists: Int,
    /** Relative names in load order, loose root files first. */
    val names: List<String>,
    /** Track values, one per name, in the same order. */
    val tracks: List<Track>,
)

/** Builds one capitalised Latin word from the shared syllable pool. */
private fun latinWord(random: Random): String {
    /** Number of syllables in this word, two or three. */
    val syllables: Int = 2 + random.nextInt(2)
    /** Syllables joined into the raw word before capitalisation. */
    val raw: String = (0 until syllables).joinToString("") { LATIN_SYLLABLES[random.nextInt(LATIN_SYLLABLES.size)] }
    /** The word with its first letter capitalised and, for one word in twenty, an accented fragment appended. */
    val word: String = raw.replaceFirstChar { first -> first.uppercaseChar() }
    return if (random.nextInt(20) == 0) word + ACCENTED[random.nextInt(ACCENTED.size)] else word
}

/** Joins one to three Latin words with spaces. */
private fun latinPhrase(random: Random): String {
    /** Number of words in this phrase, one to three. */
    val words: Int = 1 + random.nextInt(3)
    return (0 until words).joinToString(" ") { latinWord(random) }
}

/** Builds a kana run, sometimes ending in a Han character. */
private fun kanaPhrase(random: Random): String {
    /** Number of kana characters in this phrase, two to five. */
    val length: Int = 2 + random.nextInt(4)
    /** The kana run before any Han character is appended. */
    val kana: String = (0 until length).joinToString("") { KANA[random.nextInt(KANA.size)] }
    return if (random.nextInt(3) == 0) kana + HAN[random.nextInt(HAN.size)] else kana
}

/** Joins one to three Cyrillic words of two to four syllables. */
private fun cyrillicPhrase(random: Random): String {
    /** Number of words in this phrase, one to three. */
    val words: Int = 1 + random.nextInt(3)
    return (0 until words).joinToString(" ") {
        /** Number of syllables in this word, two to four. */
        val syllables: Int = 2 + random.nextInt(3)
        (0 until syllables).joinToString("") { CYRILLIC[random.nextInt(CYRILLIC.size)] }
    }
}

/** Builds the folder name of artist `index`, with the same script weights the desktop generator uses. */
private fun artistName(seed: Long, index: Int): String {
    /** Generator for this artist alone, so the name depends only on the seed and the index. */
    val random = Random(seed * 1_000_003L + index)
    /** Roll deciding the writing system, from zero to ninety-nine. */
    val roll: Int = random.nextInt(100)
    return when {
        roll < 62 -> latinPhrase(random)
        roll < 74 -> kanaPhrase(random)
        roll < 82 -> cyrillicPhrase(random)
        roll < 91 -> "${1 + random.nextInt(999)} ${latinPhrase(random)}"
        roll < 95 -> "${SYMBOLS[random.nextInt(SYMBOLS.size)]} ${latinPhrase(random)}"
        else -> "${latinPhrase(random)} ${kanaPhrase(random)}"
    }
}

/** Returns the first free folder name for artist `index`, appending a counter to repeats. */
private fun uniqueFolder(seed: Long, taken: MutableSet<String>, index: Int): String {
    /** The drawn name, or a numbered stand-in when the draw comes out blank. */
    val drawn: String = artistName(seed, index).ifBlank { "artist $index" }
    var candidate: String = drawn
    var counter = 2
    while (!taken.add(candidate)) {
        candidate = "$drawn $counter"
        counter += 1
    }
    return candidate
}

/** Builds a track title, capped at [MAX_TITLE_CHARS] characters. */
private fun trackTitle(random: Random): String {
    /** Number of words in this title, long for about two in a hundred. */
    val words: Int = if (random.nextInt(100) < 2) 30 + random.nextInt(20) else 1 + random.nextInt(5)
    return (0 until words)
        .joinToString(" ") { if (random.nextInt(100) < 15) kanaPhrase(random) else latinWord(random) }
        .take(MAX_TITLE_CHARS)
}

/** Picks an audio extension: opus sixty times in a hundred, flac thirty-six, m4a three, mp3 one. */
private fun extensionFor(random: Random): String {
    /** Roll deciding the extension, from zero to ninety-nine. */
    val roll: Int = random.nextInt(100)
    return when {
        roll < 60 -> "opus"
        roll < 96 -> "flac"
        roll < 99 -> "m4a"
        else -> "mp3"
    }
}

/** Returns the UTF-8 byte length of a character, for characters in the Basic Multilingual Plane. */
private fun utf8Length(character: Char): Int = when {
    character.code < 0x80 -> 1
    character.code < 0x800 -> 2
    else -> 3
}

/** Builds one track file name, truncating the stem so the whole component fits the byte limit. */
private fun fileName(prefix: String, title: String, extension: String): String {
    /** Budget in bytes left for the stem once the extension and its dot are reserved. */
    val budget: Int = MAX_COMPONENT_BYTES - extension.length - 1
    /** Stem before truncation, prefix and title joined. */
    val stem: String = "$prefix $title"
    val kept = StringBuilder()
    var used = 0
    for (character in stem) {
        used += utf8Length(character)
        if (used > budget) break
        kept.append(character)
    }
    return "$kept.$extension"
}

/** Returns the album folder of one track in an artist with subfolders, with a CD folder on every other album. */
private fun albumDirectory(folder: String, track: Int, count: Int): String {
    /** Album number of this track, one to twelve. */
    val album: Int = 1 + track * 12 / maxOf(1, count)
    /** Album folder without its CD folder. */
    val base = "$folder/Album ${album.toString().padStart(2, '0')}"
    return if (album % 2 == 0) "$base/CD ${1 + track % 2}" else base
}

/**
 * Builds a library of [artists] folders with [tracksPerArtist] tracks each, plus loose root files, in load order.
 *
 * Names are unique. The URIs imitate MediaStore content URIs, so they differ from the display paths.
 */
fun buildSyntheticLibrary(
    artists: Int,
    tracksPerArtist: Int = SCALE_TRACKS_PER_ARTIST,
    seed: Long = SCALE_SEED,
): SyntheticLibrary {
    /** Total tracks in the library. */
    val total: Int = artists * tracksPerArtist
    /** Tracks placed as loose root files. */
    val loose: Int = total * LOOSE_PERCENT / 100
    /** Tracks left for the artist folders. */
    val artistTracks: Int = total - loose
    /** Names in load order, loose root files first. */
    val names = ArrayList<String>(total)
    /** Generator for the loose root files. */
    val rootRandom = Random(seed)
    for (number in 1..loose) {
        names.add(fileName("Loose $number -", trackTitle(rootRandom), extensionFor(rootRandom)))
    }
    /** Folder names already used, so no two artists share one. */
    val taken = HashSet<String>(artists * 2)
    for (index in 0 until artists) {
        /** Folder name of this artist. */
        val folder: String = uniqueFolder(seed, taken, index)
        /** Tracks in this artist's folder; the first remainder artists take one extra. */
        val count: Int = artistTracks / artists + (if (index < artistTracks % artists) 1 else 0)
        /** Generator for this artist's tracks, so each artist is the same at every size. */
        val random = Random(seed * 7_919L + index)
        /** Whether this artist has album subfolders, about one artist in ten. */
        val subfolders: Boolean = random.nextInt(100) < 10
        /** Width of the numeric prefix, sized to this artist's track count. */
        val width: Int = count.toString().length
        for (track in 0 until count) {
            /** Folder that holds this track: the artist folder, or an album folder inside it. */
            val directory: String = if (subfolders) albumDirectory(folder, track, count) else folder
            /** Numeric prefix that keeps this track's file name unique inside the folder. */
            val prefix: String = (track + 1).toString().padStart(width, '0')
            names.add("$directory/" + fileName(prefix, trackTitle(random), extensionFor(random)))
        }
    }
    check(HashSet(names).size == names.size) { "generated names must be unique" }
    return SyntheticLibrary(
        artists = artists,
        names = names,
        tracks = names.mapIndexed { index, name -> Track("content://media/external/audio/media/$index", name) },
    )
}
