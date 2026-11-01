// Primitive growable lists for the Search index. Boxing a list of track positions would cost an object per entry, and
// a million-track library holds several million entries, so every list here wraps a primitive array.

// What:     `package dev.monochromatic.musicplayer.core` places the list helpers beside the index that owns them.
// Why:      The index is the only user, and it needs their internals for the grow-outside-the-lock rule.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

/** Smallest capacity of a list, so a bucket that gets a first entry does not grow several times at once. */
private const val MIN_LIST_CAPACITY: Int = 8

/** Mask that keeps the low 32 bits of a packed candidate or ordinary long, which hold a track number. */
internal const val LOW_BITS_MASK: Long = 0xFFFF_FFFFL

/** Number of bits the key of a packed candidate is shifted left of the track number. */
internal const val KEY_SHIFT: Int = 32

/** Number of bits that hold a batch-local track number, which limits a prepared batch to 65,536 tracks. */
internal const val LOCAL_BITS: Int = 16

/** Most tracks one prepared batch may hold; a batch is cut there so its per-key lists stay small. */
internal const val MAX_BATCH_TRACKS: Int = 1 shl LOCAL_BITS

// What:     `internal class IntList` is a sorted-by-construction list of track positions stored in an `IntArray`.
// Why:      A bucket lists the tracks that have a word starting with a character, in track order. Tracks only arrive
//           in increasing order, so the list stays sorted and a cut at "tracks below N" is a binary search.
//
// In TS you'd write (pseudocode):
// ```ts
// class IntList { data: Int32Array; size = 0; appendShifted(source: IntList, base: number) {} }
// ```
/**
 * A growable list of ascending track positions.
 *
 * Only the thread that appends changes a list, and it replaces a full list with [grownFor]'s copy before it takes the
 * index's write lock, so readers never wait for a copy.
 */
internal class IntList(
    /** Backing storage; only the first [size] entries are meaningful. */
    var data: IntArray,
    /** Number of meaningful entries. */
    var size: Int,
) {
    /**
     * Returns a copy with room for [extra] more entries, or null when this list already has the room.
     *
     * The copy reads only the entries that exist now, which never change, so it needs no lock.
     */
    fun grownFor(extra: Int): IntList? {
        /** Entries the list must hold after the append. */
        val needed: Int = size + extra
        if (needed <= data.size) {
            return null
        }
        /** Doubling keeps the number of copies logarithmic. */
        val capacity: Int = maxOf(needed, data.size * 2, MIN_LIST_CAPACITY)
        return IntList(data.copyOf(capacity), size)
    }

    /** Appends [value], doubling the storage when it is full. */
    fun add(value: Int) {
        if (size == data.size) {
            data = data.copyOf(maxOf(size * 2, MIN_LIST_CAPACITY))
        }
        data[size] = value
        size += 1
    }

    /** Appends every entry of [source], each shifted up by [base]; the caller made room first. */
    fun appendShifted(source: IntList, base: Int) {
        for (position in 0 until source.size) {
            data[size] = source.data[position] + base
            size += 1
        }
    }

    /** Returns the number of entries below [limit], which is the index of the first entry at or above it. */
    fun countBelow(limit: Int): Int {
        /** Lowest index that may still hold the answer. */
        var low = 0
        /** One past the highest index that may still hold the answer. */
        var high: Int = size
        while (low < high) {
            /** Middle of the remaining range. */
            val middle: Int = (low + high) ushr 1
            if (data[middle] < limit) {
                low = middle + 1
            } else {
                high = middle
            }
        }
        return low
    }
}

// What:     `internal class KeyedLists` maps a 48-bit key to the ascending list of batch-local track numbers.
// Why:      A batch must group its tracks by word-start key. Sorting millions of packed entries spent most of the
//           build time; a hash table of lists groups them in one pass, and tracks are visited in ascending order, so
//           each list is born sorted and a repeated key within one track is a repeat of the last entry.
//
// In TS you'd write (pseudocode):
// ```ts
// class KeyedLists { lists = new Map<bigint, number[]>(); add(key: bigint, local: number) {} }
// ```
/** An open-addressing hash table from key to [IntList], filled in ascending local order. */
internal class KeyedLists {
    /** Keys by slot; a slot is free when its list is null. */
    private var keys: LongArray = LongArray(INITIAL_SLOTS)

    /** Lists by slot. */
    private var lists: Array<IntList?> = arrayOfNulls(INITIAL_SLOTS)

    /** Number of keys stored. */
    var size: Int = 0
        private set

    /** Adds [local] to the list of [key], ignoring a repeat of the last entry. */
    fun add(key: Long, local: Int) {
        if ((size + 1) * LOAD_DENOMINATOR > keys.size * LOAD_NUMERATOR) {
            grow()
        }
        /** Slot of the key, or the free slot where it belongs. */
        var slot: Int = slotOf(key, keys.size)
        while (true) {
            /** List stored in this slot, or null when the slot is free. */
            val list: IntList? = lists[slot]
            if (list == null) {
                keys[slot] = key
                lists[slot] = IntList(IntArray(MIN_LIST_CAPACITY), 0).also { fresh -> fresh.add(local) }
                size += 1
                return
            }
            if (keys[slot] == key) {
                if (list.data[list.size - 1] != local) {
                    list.add(local)
                }
                return
            }
            slot = (slot + 1) and (keys.size - 1)
        }
    }

    /** Calls [action] with every key and its list, in no particular order. */
    inline fun forEach(action: (key: Long, list: IntList) -> Unit) {
        for (slot in 0 until capacity()) {
            listAt(slot)?.let { list -> action(keyAt(slot), list) }
        }
    }

    /** Returns the number of slots, for iteration. */
    fun capacity(): Int = keys.size

    /** Returns the key in [slot]. */
    fun keyAt(slot: Int): Long = keys[slot]

    /** Returns the list in [slot], or null when the slot is free. */
    fun listAt(slot: Int): IntList? = lists[slot]

    /** Doubles the table and re-inserts every key. */
    private fun grow() {
        /** Keys of the table before it grew. */
        val oldKeys: LongArray = keys
        /** Lists of the table before it grew. */
        val oldLists: Array<IntList?> = lists
        keys = LongArray(oldKeys.size * 2)
        lists = arrayOfNulls(oldKeys.size * 2)
        for (slot in oldKeys.indices) {
            /** List of the old slot. */
            val list: IntList = oldLists[slot] ?: continue
            /** Slot in the larger table. */
            var target: Int = slotOf(oldKeys[slot], keys.size)
            while (lists[target] != null) {
                target = (target + 1) and (keys.size - 1)
            }
            keys[target] = oldKeys[slot]
            lists[target] = list
        }
    }
}

/** Number of slots a new table starts with; a power of two. */
private const val INITIAL_SLOTS: Int = 64

/** Numerator of the load factor at which a table grows. */
private const val LOAD_NUMERATOR: Int = 1

/** Denominator of the load factor at which a table grows. */
private const val LOAD_DENOMINATOR: Int = 2

/** Multiplier of the Fibonacci hash that spreads similar keys. */
private const val HASH_MULTIPLIER: Long = -7_046_029_254_386_353_131L

/** Returns the slot of [key] in a table of [slots] slots, which is a power of two. */
private fun slotOf(key: Long, slots: Int): Int =
    ((key * HASH_MULTIPLIER) ushr (Long.SIZE_BITS - Integer.numberOfTrailingZeros(slots))).toInt()
