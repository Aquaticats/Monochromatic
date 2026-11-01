// The in-memory Search index (D112). It keeps the normalized title of every track, first and second character buckets
// that list the tracks having a word starting with a character or a character pair, and a small table of top-level
// folders. Tracks are only ever appended, so a track's number never changes while a library is being scanned.

// What:     `package dev.monochromatic.musicplayer.core` places the index beside the matcher it must agree with.
// Why:      Its answers must equal the old matcher's, so both read the same normalization and ranking rules.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

/** Number of distinct UTF-16 units, which is the size of the first-character bucket table. */
private const val UNIT_COUNT: Int = 0x10000

/** Marks a ranked candidate as a folder: tracks use the low bits for their number and folders set this bit. */
internal const val FOLDER_BIT: Int = 1 shl 31

// What:     `internal class FolderRecord(...)` is one top-level folder with its search form.
// Why:      Folders are tested on every query, so their normalized names are kept ready.
//
// In TS you'd write (pseudocode):
// ```ts
// type FolderRecord = { name: string; norm: Uint16Array };
// ```
/** One top-level folder and its trimmed normalized name. */
internal class FolderRecord(
    /** Folder name as displayed. */
    val name: String,
    /** Trimmed normalized name. */
    val norm: CharArray,
)

// What:     `internal class AppendRoom(...)` carries the enlarged storage that an append needs.
// Why:      Growing an array copies it, and a copy of a hundred megabytes under the write lock would stall a
//           keystroke. The appender is the only thread that changes the index, so it copies outside the lock.
//
// In TS you'd write (pseudocode):
// ```ts
// type AppendRoom = { paths?: string[]; chars?: Uint16Array; ends?: Int32Array; firsts: Map<number, IntList> };
// ```
/** Enlarged copies of the storage that the next append would overflow; each field is null when none is needed. */
internal class AppendRoom(
    /** Larger path table, or null. */
    val paths: Array<String?>?,
    /** Larger URI table, or null. */
    val uris: Array<String?>?,
    /** Larger title text, or null. */
    val chars: CharArray?,
    /** Larger end-offset table, or null. */
    val ends: IntArray?,
    /** Larger first-character buckets by unit, only for buckets the batch fills past their capacity. */
    val firsts: Map<Int, IntList>,
    /** Larger pair buckets by pair key, only for buckets the batch fills past their capacity. */
    val pairs: Map<Int, IntList>,
    /** Larger triple buckets by triple key, only for buckets the batch fills past their capacity. */
    val triples: Map<Long, IntList>,
)

// What:     `internal class PathReplacement(...)` is the new display paths of every track with their folder table.
// Why:      A scan can shrink the prefix that every path shares, which changes the paths the pages show and the
//           top-level folders without changing any title. The new table is built before the write lock; the lock only
//           swaps references.
//
// In TS you'd write (pseudocode):
// ```ts
// type PathReplacement = { paths: string[]; folders: FolderRecord[]; folderSlots: Map<string, number> };
// ```
/** Replacement display paths for all tracks, with the folder table they imply. */
internal class PathReplacement(
    /** New display path of each track, by track number. */
    val paths: Array<String?>,
    /** Folder table of the new paths. */
    val folders: ArrayList<FolderRecord>,
    /** Position of each folder name in [folders]. */
    val folderSlots: HashMap<String, Int>,
)

// What:     `class SearchIndex(val generation: Long)` is the searchable form of a library.
// Why:      A keystroke reads one bucket and tests only its members, instead of rebuilding a folder index and
//           normalizing every name, which cost seconds at one million tracks.
//
// In TS you'd write (pseudocode):
// ```ts
// class SearchIndex { paths: string[]; norm: Uint16Array; byFirst = new Map<number, number[]>();
//   append(batch) {} search(q) {} }
// ```
/**
 * The searchable form of one library: normalized titles, word-start buckets and the top-level folder table.
 *
 * Callers append batches in library order and may search from other threads, but only under the read-write lock that
 * `SearchService` holds: appends take the write lock, searches the read lock. The index itself takes no lock, so
 * single-threaded use (the tests, the one-shot [searchLibrary]) needs none.
 */
class SearchIndex(
    /** Number of the library this index describes; a new library starts a new generation. */
    val generation: Long = 0L,
) {
    /** Display path of each track by track number; the strings are the caller's, shared and not copied. */
    internal var paths: Array<String?> = arrayOfNulls(0)
        private set

    /** URI of each track by track number, or null where the caller gave none; shared strings like [paths]. */
    internal var uris: Array<String?> = arrayOfNulls(0)
        private set

    /** Trimmed normalized titles of all tracks, concatenated in track order. */
    internal var chars: CharArray = CharArray(0)
        private set

    /** End offset in [chars] of each title; title `i` starts where title `i - 1` ends. */
    internal var ends: IntArray = IntArray(0)
        private set

    /** Number of tracks appended so far. */
    var trackCount: Int = 0
        private set

    /** Tracks having a word that starts with the unit, in track order, indexed by the unit. */
    private val byFirst: Array<IntList?> = arrayOfNulls(UNIT_COUNT)

    /** Tracks having a word that starts with the unit pair, in track order, keyed by the pair key. */
    private val byPair: HashMap<Int, IntList> = HashMap()

    /** Tracks having a word that starts with the unit triple, in track order, keyed by the triple key. */
    private val byTriple: HashMap<Long, IntList> = HashMap()

    /** Top-level folders in first-seen order. */
    internal var folders: ArrayList<FolderRecord> = ArrayList()
        private set

    /** Position of each folder name in [folders]. */
    private var folderSlots: HashMap<String, Int> = HashMap()

    /** The best-five-hundred answer of each single-character query. */
    internal var top: TopCache = TopCache()
        private set

    /** Returns the start offset in [chars] of the title of track [position]. */
    internal fun titleStart(position: Int): Int = if (position == 0) 0 else ends[position - 1]

    /** Returns the UTF-16 length of the normalized title of track [position], which orders equal tiers. */
    internal fun titleLength(position: Int): Int = ends[position] - titleStart(position)

    /** Returns the bucket of tracks having a word that starts with [unit], or null when there is none. */
    internal fun bucketFirst(unit: Char): IntList? = byFirst[unit.code]

    /** Returns the bucket of tracks having a word that starts with the pair, or null when there is none. */
    internal fun bucketPair(first: Char, second: Char): IntList? = byPair[pairKeyOf(first, second)]

    /** Returns the bucket of tracks having a word that starts with the three units, or null when there is none. */
    internal fun bucketTriple(first: Char, second: Char, third: Char): IntList? =
        byTriple[tripleKeyOf(first, second, third)]

    /** Lists the first units that have a bucket, with the bucket sizes. */
    internal fun firstUnitSizes(): List<Pair<Char, Int>> =
        byFirst.withIndex().mapNotNull { (unit, list) -> list?.let { found -> unit.toChar() to found.size } }

    /** Returns the table slots of the folders that Search offers as results: every real top-level folder. */
    internal fun listedFolderSlots(): List<Int> = folders.indices.toList()

    // What:     `internal fun prepareRoom(batch: PreparedBatch): AppendRoom` enlarges the storage a batch needs.
    // Why:      See [AppendRoom]: the copies are made before the write lock is taken. Only the appender changes the
    //           index, and the existing entries never change, so reading them here is safe.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const room = index.prepareRoom(batch); // copies outside the lock
    // ```
    /** Builds the enlarged storage for [batch], with nothing that [append] could not also build itself. */
    internal fun prepareRoom(batch: PreparedBatch): AppendRoom {
        /** Tracks after the append. */
        val tracksAfter: Int = trackCount + batch.paths.size
        /** Characters after the append. */
        val charsAfter: Int = (if (trackCount == 0) 0 else ends[trackCount - 1]) + batch.chars.size
        /** Enlarged first-unit buckets. */
        val firsts = HashMap<Int, IntList>()
        /** Enlarged unit-pair buckets. */
        val pairs = HashMap<Int, IntList>()
        /** Enlarged unit-triple buckets. */
        val triples = HashMap<Long, IntList>()
        batch.firsts.forEach { key, list ->
            byFirst[key.toInt()]?.grownFor(list.size)?.let { grown -> firsts[key.toInt()] = grown }
        }
        batch.pairs.forEach { key, list ->
            byPair[key.toInt()]?.grownFor(list.size)?.let { grown -> pairs[key.toInt()] = grown }
        }
        batch.triples.forEach { key, list ->
            byTriple[key]?.grownFor(list.size)?.let { grown -> triples[key] = grown }
        }
        return AppendRoom(
            paths = if (tracksAfter > paths.size) paths.copyOf(grownCapacity(paths.size, tracksAfter)) else null,
            uris = if (tracksAfter > uris.size) uris.copyOf(grownCapacity(uris.size, tracksAfter)) else null,
            chars = if (charsAfter > chars.size) chars.copyOf(grownCapacity(chars.size, charsAfter)) else null,
            ends = if (tracksAfter > ends.size) ends.copyOf(grownCapacity(ends.size, tracksAfter)) else null,
            firsts = firsts,
            pairs = pairs,
            triples = triples,
        )
    }

    // What:     `internal fun append(batch: PreparedBatch, room: AppendRoom)` adds a prepared batch after the tracks
    //           already known.
    // Why:      This is the push done under the write lock. It copies text, shifts offsets and extends bucket lists;
    //           all normalization and growth was done before.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // lock.write(() => index.append(batch, room));
    // ```
    /** Appends [batch], using the enlarged storage of [room] when it still fits and growing inline when it does not. */
    internal fun append(batch: PreparedBatch, room: AppendRoom) {
        /** Number of the first appended track. */
        val base: Int = trackCount
        /** Offset in [chars] where the batch text starts. */
        val charBase: Int = if (trackCount == 0) 0 else ends[trackCount - 1]
        adopt(room, base + batch.paths.size, charBase + batch.chars.size)
        System.arraycopy(batch.paths, 0, paths, base, batch.paths.size)
        batch.uris?.let { given -> System.arraycopy(given, 0, uris, base, given.size) }
        System.arraycopy(batch.chars, 0, chars, charBase, batch.chars.size)
        for (local in batch.ends.indices) {
            ends[base + local] = charBase + batch.ends[local]
        }
        trackCount = base + batch.paths.size
        batch.firsts.forEach { key, list ->
            appendRun(byFirst[key.toInt()], room.firsts[key.toInt()], list, base) { bucket ->
                byFirst[key.toInt()] = bucket
            }
        }
        batch.pairs.forEach { key, list ->
            appendRun(byPair[key.toInt()], room.pairs[key.toInt()], list, base) { bucket ->
                byPair[key.toInt()] = bucket
            }
        }
        batch.triples.forEach { key, list ->
            appendRun(byTriple[key], room.triples[key], list, base) { bucket ->
                byTriple[key] = bucket
            }
        }
        batch.folders.forEach { sighting -> noteFolder(sighting) }
    }

    /** Installs the storage of [room] and grows inline anything it did not cover. */
    private fun adopt(room: AppendRoom, tracksAfter: Int, charsAfter: Int) {
        room.paths?.let { grown -> paths = grown }
        room.uris?.let { grown -> uris = grown }
        room.chars?.let { grown -> chars = grown }
        room.ends?.let { grown -> ends = grown }
        if (tracksAfter > paths.size) {
            paths = paths.copyOf(grownCapacity(paths.size, tracksAfter))
        }
        if (tracksAfter > uris.size) {
            uris = uris.copyOf(grownCapacity(uris.size, tracksAfter))
        }
        if (charsAfter > chars.size) {
            chars = chars.copyOf(grownCapacity(chars.size, charsAfter))
        }
        if (tracksAfter > ends.size) {
            ends = ends.copyOf(grownCapacity(ends.size, tracksAfter))
        }
    }

    /** Adds a folder to the table unless it is already there. */
    private fun noteFolder(sighting: FolderSighting) {
        if (folderSlots.containsKey(sighting.name)) {
            return
        }
        folderSlots[sighting.name] = folders.size
        folders.add(FolderRecord(sighting.name, normalizedName(sighting.name)))
    }

    // What:     `internal fun prepareReplacement(newPaths: List<String>): PathReplacement` builds the folder table of
    //           new display paths for the tracks already appended.
    // Why:      See [PathReplacement]: this is the part that costs time, and it takes no lock.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const replacement = index.prepareReplacement(newPaths); // outside the lock
    // ```
    /**
     * Scans [newPaths], one per appended track in order, and returns the replacement for them, or null when a new path
     * names a different title than the track has, because the titles in the buckets would then be wrong.
     */
    internal fun prepareReplacement(newPaths: List<String>): PathReplacement? {
        require(newPaths.size == trackCount) { "Got ${newPaths.size} paths for $trackCount tracks." }
        for (position in 0 until trackCount) {
            if (!sameTitle(paths[position].orEmpty(), newPaths[position])) {
                return null
            }
        }
        /** Folders of the new paths. */
        val scan = FolderScan()
        newPaths.forEach { path -> scan.note(path) }
        /** Folder table of the new paths. */
        val table = ArrayList<FolderRecord>()
        /** Position of each folder name in the table. */
        val slots = HashMap<String, Int>()
        for (name in scan.folders) {
            slots[name] = table.size
            table.add(FolderRecord(name, normalizedName(name)))
        }
        return PathReplacement(
            paths = newPaths.toTypedArray<String?>(),
            folders = table,
            folderSlots = slots,
        )
    }

    // What:     `internal fun applyReplacement(replacement: PathReplacement)` swaps in the new display paths.
    // Why:      This is the push under the write lock: a few reference assignments. Titles and buckets do not depend
    //           on the paths, so they stay; the cached answers are dropped because their tie order read the old paths.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // lock.write(() => index.applyReplacement(replacement));
    // ```
    /** Installs [replacement], which was prepared for exactly the tracks appended now. */
    internal fun applyReplacement(replacement: PathReplacement) {
        require(replacement.paths.size == trackCount) { "Got ${replacement.paths.size} paths for $trackCount tracks." }
        paths = replacement.paths.copyOf(maxOf(paths.size, replacement.paths.size))
        folders = replacement.folders
        folderSlots = replacement.folderSlots
        top = TopCache()
    }

    // What:     `fun appendPaths(newPaths: List<String>)` prepares and appends tracks on the calling thread.
    // Why:      Single-threaded users (the tests and the one-shot search) need no lock and no split between
    //           preparing and pushing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // index.appendPaths(["Artist/Song.flac"]);
    // ```
    /** Prepares [newPaths] and appends them, in order, after the tracks already known. */
    fun appendPaths(newPaths: List<String>, newUris: List<String>? = null) {
        /** Start of the chunk being appended. */
        var from = 0
        while (from < newPaths.size) {
            /** Last track of this batch plus one. */
            val to: Int = minOf(newPaths.size, from + MAX_BATCH_TRACKS)
            /** Batch normalized and keyed without any lock; a batch is limited so its lists stay small. */
            val batch: PreparedBatch = prepareBatch(newPaths.subList(from, to), newUris?.subList(from, to))
            append(batch, prepareRoom(batch))
            from += MAX_BATCH_TRACKS
        }
    }

    /** Estimates the heap the index holds in bytes, excluding the path strings it shares with the caller. */
    fun heapBytes(): Long {
        /** Bytes of the bucket arrays at their capacity. */
        val buckets: Long = byFirst.sumOf { list -> (list?.data?.size ?: 0).toLong() * Int.SIZE_BYTES } +
            byPair.values.sumOf { list -> list.data.size.toLong() * Int.SIZE_BYTES } +
            byTriple.values.sumOf { list -> list.data.size.toLong() * Int.SIZE_BYTES }
        return buckets + chars.size.toLong() * Char.SIZE_BYTES + ends.size.toLong() * Int.SIZE_BYTES +
            (paths.size + uris.size).toLong() * Int.SIZE_BYTES
    }
}

/** Reports whether two display paths have the same title text, without building the titles. */
private fun sameTitle(left: String, right: String): Boolean {
    /** Title range of the left path. */
    val leftRange: Long = titleRangeOf(left)
    /** Title range of the right path. */
    val rightRange: Long = titleRangeOf(right)
    /** Start of the left title. */
    val leftFrom: Int = (leftRange ushr KEY_SHIFT).toInt()
    /** Start of the right title. */
    val rightFrom: Int = (rightRange ushr KEY_SHIFT).toInt()
    /** Length of the left title. */
    val length: Int = (leftRange and LOW_BITS_MASK).toInt() - leftFrom
    return length == (rightRange and LOW_BITS_MASK).toInt() - rightFrom &&
        left.regionMatches(leftFrom, right, rightFrom, length)
}

/** Returns the trimmed normalized form of a folder name as characters. */
private fun normalizedName(name: String): CharArray {
    /** Buffer for the folder's normalized name. */
    val buffer = TextBuffer()
    normalizeTitleInto(buffer, name)
    return buffer.chars.copyOf(buffer.size)
}

/** Returns a capacity of at least [needed], at least one and a half times [current] so repeated growth is rare. */
private fun grownCapacity(current: Int, needed: Int): Int = maxOf(needed, current + current / 2)

/**
 * Appends one batch list to the bucket that already exists, to its enlarged copy, or to a new bucket, and hands the
 * bucket that now holds the run to [store].
 *
 * The enlarged copy was made before the write lock; a bucket that is still too small grows here, inline.
 */
private inline fun appendRun(
    existing: IntList?,
    enlarged: IntList?,
    batchList: IntList,
    base: Int,
    store: (IntList) -> Unit,
) {
    /** Bucket that receives the batch list. */
    var bucket: IntList = enlarged ?: existing ?: IntList(IntArray(batchList.size), 0)
    bucket.grownFor(batchList.size)?.let { grown -> bucket = grown }
    store(bucket)
    bucket.appendShifted(batchList, base)
}
