// Batch preparation for the Search index: everything about a group of new tracks that can be computed without
// touching the index. D112 appends tracks under a write lock that holds only the push, so every cost that grows with
// the text (normalization, key extraction, sorting keys into runs) happens here, on the caller's thread, first.

// What:     `package dev.monochromatic.musicplayer.core` places batch preparation beside the index it feeds.
// Why:      The batch type is the index's private input, and the tests build batches the same way the feed does.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

/** Separator between the segments of a display path. */
private const val SEGMENT_SEPARATOR: Char = '/'

/** Separator between a file name and its extension. */
private const val DOT: Char = '.'

/** Segment that names the directory itself and is ignored. */
private const val CURRENT_DIRECTORY: String = "."

/** Segment that names the parent directory and is ignored. */
private const val PARENT_DIRECTORY: String = ".."

/** Number of meaningful segments from which a path has a subfolder below its top-level folder. */
private const val SUBFOLDER_SEGMENTS: Int = 3

/** Fewest meaningful segments a track needs to sit inside a top-level folder. */
private const val FOLDERED_SEGMENTS: Int = 2

/** Bits per UTF-16 unit, used to pack a pair of units into one key. */
private const val UNIT_BITS: Int = 16

// What:     `internal class FolderSighting(...)` is a top-level folder that a batch names.
// Why:      The index lists each folder once.
//
// In TS you'd write (pseudocode):
// ```ts
// type FolderSighting = { name: string };
// ```
/** A top-level folder named by at least one track of a batch. */
internal class FolderSighting(
    /** First meaningful path segment of a track that has two or more segments. */
    val name: String,
)

// What:     `internal class PreparedBatch(...)` is a group of tracks ready to be appended with a copy.
// Why:      Holding the normalized titles, their ends, the grouped key lists and the folder notes lets the index append
//           in time proportional to a copy, not to normalization.
//
// In TS you'd write (pseudocode):
// ```ts
// type PreparedBatch = { paths: string[]; chars: Uint16Array; ends: Int32Array; firstRuns: BigInt64Array; /* ... */ };
// ```
/** A batch of tracks prepared off the lock. */
internal class PreparedBatch(
    /** Display paths of the batch tracks in batch order; the strings are the caller's, not copies. */
    val paths: Array<String>,
    /** URIs of the batch tracks in batch order, or null when the caller has none; also the caller's strings. */
    val uris: Array<String>?,
    /** Trimmed normalized titles of the batch tracks, concatenated in batch order. */
    val chars: CharArray,
    /** Batch-local end offset in [chars] of each title; title `i` starts where title `i - 1` ends. */
    val ends: IntArray,
    /** Ascending batch-local track numbers by unit, for words that start with that unit. */
    val firsts: KeyedLists,
    /** Ascending batch-local track numbers by unit pair, for words that start with those two units. */
    val pairs: KeyedLists,
    /** Ascending batch-local track numbers by unit triple, for words that start with those three units. */
    val triples: KeyedLists,
    /** Top-level folders named by tracks with two or more meaningful segments. */
    val folders: List<FolderSighting>,
)

// What:     `internal fun pairKeyOf(first: Char, second: Char): Int` packs two UTF-16 units into one map key.
// Why:      The pair buckets are keyed by an `Int`, which a hash map stores without a pair object per key.
//
// In TS you'd write (pseudocode):
// ```ts
// const pairKeyOf = (a: string, b: string) => (a.charCodeAt(0) << 16) | b.charCodeAt(0);
// ```
/** Returns the bucket key of a two-unit word start. */
internal fun pairKeyOf(first: Char, second: Char): Int = (first.code shl UNIT_BITS) or second.code

/** Returns the bucket key of a three-unit word start, which needs 48 bits. */
internal fun tripleKeyOf(first: Char, second: Char, third: Char): Long =
    (pairKeyOf(first, second).toLong() and LOW_BITS_MASK shl UNIT_BITS) or third.code.toLong()

// What:     `internal fun titleRangeOf(path: String): Long` finds where a track's title sits inside its display path.
// Why:      The title is the file name without its last extension, as `trackTitleFor` defines it, and finding its
//           range avoids one substring per track.
//
// In TS you'd write (pseudocode):
// ```ts
// const titleRangeOf = (p: string): [number, number] => { /* after the last slash, before the last dot */ };
// ```
/** Returns the start of the title in the high 32 bits and its end in the low 32 bits. */
internal fun titleRangeOf(path: String): Long {
    /** Offset of the file name, which follows the last separator. */
    val nameStart: Int = path.lastIndexOf(SEGMENT_SEPARATOR) + 1
    /** Offset of the last dot inside the file name, or a value before the name when it has none. */
    val dot: Int = path.lastIndexOf(DOT)
    /** Offset just past the title: the dot when the name has an extension, else the end of the path. */
    val titleEnd: Int = if (dot > nameStart) dot else path.length
    return (nameStart.toLong() shl KEY_SHIFT) or titleEnd.toLong()
}

// What:     `private fun collectKeys(...)` adds the bucket keys of one normalized title to the batch's entries.
// Why:      A query term can match only where a word starts, so a track belongs in the bucket of each first character
//           and each first character pair found at a word start. A separator never starts a bucket key.
//
// In TS you'd write (pseudocode):
// ```ts
// function collectKeys(chars: Uint16Array, start: number, end: number, local: number, keys: BucketKeys) {}
// ```
/** Adds track [local] to the lists of the first-unit, first-pair and first-triple keys of `chars[start, end)`. */
private fun collectKeys(chars: CharArray, start: Int, end: Int, local: Int, keys: BucketKeys) {
    for (index in start until end) {
        /** Unit at this offset. */
        val unit: Char = chars[index]
        if (isWordSeparator(unit) || (index > start && !isWordSeparator(chars[index - 1]))) {
            continue
        }
        keys.firsts.add(unit.code.toLong(), local)
        if (index + 1 < end && !isWordSeparator(chars[index + 1])) {
            keys.pairs.add(pairKeyOf(unit, chars[index + 1]).toLong() and LOW_BITS_MASK, local)
            if (index + 2 < end && !isWordSeparator(chars[index + 2])) {
                keys.triples.add(tripleKeyOf(unit, chars[index + 1], chars[index + 2]), local)
            }
        }
    }
}

/** The three builders that collect a batch's bucket entries. */
private class BucketKeys {
    /** Track numbers by words that start with one unit. */
    val firsts = KeyedLists()

    /** Track numbers by words that start with two units. */
    val pairs = KeyedLists()

    /** Track numbers by words that start with three units. */
    val triples = KeyedLists()
}

/** Result of counting the meaningful segments of a path: the first one and how many there are, up to three. */
internal class FolderOfPath(
    /** First meaningful segment, or null when the path has none. */
    val first: String?,
    /** Number of meaningful segments, counted only up to [SUBFOLDER_SEGMENTS]. */
    val segments: Int,
)

// What:     `internal fun folderOf(path: String): FolderOfPath` finds a path's top-level folder.
// Why:      The folder index drops empty, `.` and `..` segments before it decides where a track lives, so the Search
//           index must drop them too to find the same folder.
//
// In TS you'd write (pseudocode):
// ```ts
// const folderOf = (p: string) => {
//   const s = p.split("/").filter(x => x && x !== "." && x !== ".."); return { first: s[0], n: s.length };
// };
// ```
/** Returns the first meaningful segment of [path] and the number of meaningful segments up to three. */
internal fun folderOf(path: String): FolderOfPath {
    /** First meaningful segment found so far. */
    var first: String? = null
    /** Meaningful segments counted so far. */
    var count = 0
    /** Start of the segment being read. */
    var start = 0
    while (start <= path.length && count < SUBFOLDER_SEGMENTS) {
        /** Offset of the separator that ends this segment, or the path length for the last one. */
        val stop: Int = path.indexOf(SEGMENT_SEPARATOR, start).let { found -> if (found < 0) path.length else found }
        /** The segment between the two separators. */
        val segment: String = path.substring(start, stop)
        if (segment.isNotEmpty() && segment != CURRENT_DIRECTORY && segment != PARENT_DIRECTORY) {
            if (first == null) {
                first = segment
            }
            count += 1
        }
        start = stop + 1
    }
    return FolderOfPath(first, count)
}

// What:     `internal class FolderScan` collects the top-level folders of display paths.
// Why:      A prepared batch and a path replacement both need the same note: which top-level folders exist. A
//           loose root-level track is not a folder; its virtual letter folder is derived when a result is built.
//
// In TS you'd write (pseudocode):
// ```ts
// class FolderScan { folders = new Set<string>(); note(path: string) {} }
// ```
/** Folders named by a run of display paths. */
internal class FolderScan {
    /** Folder names in first-seen order. */
    val folders = LinkedHashSet<String>()

    /** Notes the folder of one display path. */
    fun note(path: String) {
        /** Top-level folder of the path and its segment count. */
        val folder: FolderOfPath = folderOf(path)
        /** Name of the top-level folder, or null. */
        val name: String? = folder.first
        if (name != null && folder.segments >= FOLDERED_SEGMENTS) {
            folders.add(name)
        }
    }
}

// What:     `internal fun prepareBatch(paths: List<String>): PreparedBatch` builds the appendable form of tracks.
// Why:      This is the only part of an append that costs time proportional to the text, and it takes no lock.
//
// In TS you'd write (pseudocode):
// ```ts
// function prepareBatch(paths: string[]): PreparedBatch { /* normalize, key, sort */ }
// ```
/** Normalizes the titles of [paths], extracts their bucket keys and notes their top-level folders. */
internal fun prepareBatch(paths: List<String>, uris: List<String>? = null): PreparedBatch {
    require(paths.size <= MAX_BATCH_TRACKS) { "A batch holds at most $MAX_BATCH_TRACKS tracks, got ${paths.size}." }
    require(uris == null || uris.size == paths.size) { "Got ${uris?.size} URIs for ${paths.size} paths." }
    /** Buffer that each title is normalized into before it is copied to the batch text. */
    val title = TextBuffer()
    /** Normalized text of the whole batch, grown by doubling. */
    var chars = CharArray(maxOf(paths.size * 2, FOLDERED_SEGMENTS))
    /** Used length of [chars]. */
    var used = 0
    /** End offset of each title. */
    val ends = IntArray(paths.size)
    /** Bucket lists of the batch. */
    val keys = BucketKeys()
    /** Folders of the batch. */
    val folderScan = FolderScan()
    for ((local, path) in paths.withIndex()) {
        /** Range of the title inside the path, packed. */
        val range: Long = titleRangeOf(path)
        normalizeTitleInto(title, path, (range ushr KEY_SHIFT).toInt(), (range and LOW_BITS_MASK).toInt())
        if (used + title.size > chars.size) {
            chars = chars.copyOf(maxOf(chars.size * 2, used + title.size))
        }
        System.arraycopy(title.chars, 0, chars, used, title.size)
        collectKeys(chars, used, used + title.size, local, keys)
        used += title.size
        ends[local] = used
        folderScan.note(path)
    }
    return PreparedBatch(
        paths = paths.toTypedArray(),
        uris = uris?.toTypedArray(),
        chars = chars.copyOf(used),
        ends = ends,
        firsts = keys.firsts,
        pairs = keys.pairs,
        triples = keys.triples,
        folders = folderScan.folders.map { name -> FolderSighting(name) },
    )
}
