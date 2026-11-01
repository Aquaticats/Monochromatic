// Tier test and best-five-hundred selection for the Search index. `fastTier` gives the relevance tier of a normalized
// name without allocating, and `selectBest` keeps the best candidates in the result order, computing collation keys
// only for candidates that the integer keys cannot separate.

// What:     `package dev.monochromatic.musicplayer.core` places ranking beside the index and the old matcher.
// Why:      Ranking must reproduce the old result order exactly, and the tests compare the two directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import java.text.CollationKey
import java.text.Collator

/** Relevance tier of a name that equals the whole normalized query. */
private const val RANK_WHOLE_NAME: Int = 0

/** Relevance tier of a name that starts with the first term while every term matches. */
private const val RANK_NAME_PREFIX: Int = 1

/** Relevance tier of a name where every term matches at a later word start. */
private const val RANK_WORD_START: Int = 2

/** Shift that places the tier above the length in a packed candidate. */
private const val TIER_SHIFT: Int = 60

/** Largest normalized length a packed candidate can hold; longer names tie at this length. */
private const val MAX_PACKED_LENGTH: Int = (1 shl 28) - 1

/** Mask for the length bits of a packed candidate once the track number is shifted out. */
private const val LENGTH_MASK: Long = MAX_PACKED_LENGTH.toLong()

/** Value returned by [fastTier] when a name does not match. */
internal const val NO_TIER: Int = -1

/** Number of best tracks one cached answer keeps, and the number of results a search returns. */
internal const val TRACK_KEEP: Int = SEARCH_RESULT_LIMIT

// What:     `internal fun packCandidate(tier: Int, length: Int, id: Int): Long` packs a matched candidate in a long.
// Why:      Matched candidates number in the hundreds of thousands for a one-character query, so each is a primitive.
//           The tier and length sit in the high bits, so ordering the longs orders candidates by those two keys first.
//
// In TS you'd write (pseudocode):
// ```ts
// const packCandidate = (tier: number, length: number, id: number) =>
//   (BigInt(tier) << 60n) | (BigInt(length) << 32n) | BigInt(id >>> 0);
// ```
/** Packs a candidate's tier, normalized length and track number (or folder bit and slot) into one long. */
internal fun packCandidate(tier: Int, length: Int, id: Int): Long =
    (tier.toLong() shl TIER_SHIFT) or (minOf(length, MAX_PACKED_LENGTH).toLong() shl KEY_SHIFT) or
        (id.toLong() and LOW_BITS_MASK)

/** Returns the tier of a packed candidate. */
internal fun tierOfCandidate(packed: Long): Int = (packed ushr TIER_SHIFT).toInt()

/** Returns the normalized length of a packed candidate. */
internal fun lengthOfCandidate(packed: Long): Int = ((packed ushr KEY_SHIFT) and LENGTH_MASK).toInt()

/** Returns the track number, or the folder bit and slot, of a packed candidate. */
internal fun idOfCandidate(packed: Long): Int = packed.toInt()

/** Returns the tier-and-length key of a packed candidate, the only part the integer selection compares. */
private fun rankKeyOf(packed: Long): Long = packed ushr KEY_SHIFT

// What:     `private fun hasWordStart(...)` tests whether a term occurs where a word starts in a name.
// Why:      A term matches only at the start of the name or right after a separator, so `cam` never matches inside
//           `scamper`. It reads the characters in place, so it allocates nothing.
//
// In TS you'd write (pseudocode):
// ```ts
// const hasWordStart = (name: string, term: string) => [...wordStarts(name)].some((i) => name.startsWith(term, i));
// ```
/** Reports whether [term] occurs at a word start inside `chars[start, end)`. */
private fun hasWordStart(chars: CharArray, start: Int, end: Int, term: CharArray): Boolean {
    /** Last offset at which the whole term could still fit. */
    val limit: Int = end - term.size
    /** First unit of the term, tested before the rest. */
    val head: Char = term[0]
    /** Offset being tested. */
    var index: Int = start
    while (index <= limit) {
        if (chars[index] == head &&
            (index == start || isWordSeparator(chars[index - 1])) &&
            regionStartsWith(chars, index, term)
        ) {
            return true
        }
        index += 1
    }
    return false
}

/** Reports whether every one of [terms] occurs at a word start inside `chars[start, end)`. */
internal fun allHaveWordStart(chars: CharArray, start: Int, end: Int, terms: Array<CharArray>): Boolean {
    for (term in terms) {
        if (!hasWordStart(chars, start, end, term)) {
            return false
        }
    }
    return true
}

/** Reports whether `chars` holds [prefix] at [offset]; the caller guarantees the prefix fits. */
private fun regionStartsWith(chars: CharArray, offset: Int, prefix: CharArray): Boolean {
    for (position in prefix.indices) {
        if (chars[offset + position] != prefix[position]) {
            return false
        }
    }
    return true
}

// What:     `internal fun fastTier(...)` gives the relevance tier of a normalized name for normalized terms.
// Why:      It is `matchTier` without the allocations: every term must occur at a word start; the name equal to the
//           terms joined by a space is tier 0, a name that starts with the first term is tier 1, any other match is 2.
//
// In TS you'd write (pseudocode):
// ```ts
// const fastTier = (name: string, terms: string[]) => matchTier(name, terms);
// ```
/** Returns the tier of the trimmed normalized name `chars[start, end)`, or [NO_TIER] when a term fails. */
internal fun fastTier(chars: CharArray, start: Int, end: Int, terms: Array<CharArray>, joined: CharArray): Int {
    for (term in terms) {
        if (!hasWordStart(chars, start, end, term)) {
            return NO_TIER
        }
    }
    /** Length of the name in UTF-16 units. */
    val length: Int = end - start
    if (length == joined.size && regionStartsWith(chars, start, joined)) {
        return RANK_WHOLE_NAME
    }
    if (length >= terms[0].size && regionStartsWith(chars, start, terms[0])) {
        return RANK_NAME_PREFIX
    }
    return RANK_WORD_START
}

// What:     `internal fun tierOfMatched(...)` gives the relevance tier of a name that already matches every term.
// Why:      A candidate that came out of a bucket whose key is the whole term needs no scan: the bucket proves the term
//           starts a word. Only the whole-name and name-prefix comparisons remain, and they read a few characters.
//
// In TS you'd write (pseudocode):
// ```ts
// const tierOfMatched = (name: string, terms: string[]) =>
//   name === terms.join(" ") ? 0 : name.startsWith(terms[0]) ? 1 : 2;
// ```
/** Returns the tier of the trimmed normalized name `chars[start, end)`, given that every term starts a word of it. */
internal fun tierOfMatched(chars: CharArray, start: Int, end: Int, first: CharArray, joined: CharArray): Int {
    /** Length of the name in UTF-16 units. */
    val length: Int = end - start
    if (length == joined.size && regionStartsWith(chars, start, joined)) {
        return RANK_WHOLE_NAME
    }
    if (length >= first.size && regionStartsWith(chars, start, first)) {
        return RANK_NAME_PREFIX
    }
    return RANK_WORD_START
}

/** A candidate with the string keys of the result order, built only for candidates that survive the integer keys. */
private class Keyed(
    /** The packed candidate. */
    val packed: Long,
    /** Displayed name, which the collation key is built from when the ASCII shortcut does not apply. */
    val name: String,
    /** Weight string of the name when it is plain ASCII, else null. */
    val ascii: ByteArray?,
    /** Display path of a track, or name of a folder. */
    val sortPath: String,
) {
    /** Collation key of the name, built on first use. */
    var collationKey: CollationKey? = null
}

// What:     `private fun keyedFor(index: SearchIndex, packed: Long): Keyed` adds the string keys.
// Why:      The result order breaks ties by collated name, then by display path, and building those is deferred until
//           the integer keys have cut the field down.
//
// In TS you'd write (pseudocode):
// ```ts
// const keyedFor = (index: SearchIndex, packed: bigint) => ({ packed, name, ascii, sortPath });
// ```
/** Builds the name, the ASCII weight string and the sort path of one candidate. */
private fun keyedFor(index: SearchIndex, packed: Long): Keyed {
    /** Track number, or folder bit and slot. */
    val id: Int = idOfCandidate(packed)
    if (id and FOLDER_BIT == 0) {
        /** Display path of the track, which is both its sort path and the source of its title. */
        val path: String = index.paths[id].orEmpty()
        /** Title of the track, which is the name that the collator orders. */
        val title: String = trackTitleFor(path)
        return Keyed(packed, title, AsciiCollation.keyOf(title), path)
    }
    /** Name of the folder, which is both its displayed name and its sort path. */
    val name: String = folderNameOf(index, id and FOLDER_BIT.inv())
    return Keyed(packed, name, AsciiCollation.keyOf(name), name)
}

/** Compares the collated names of two candidates, with the weight strings when both names are ASCII. */
private fun compareNames(collator: Collator, left: Keyed, right: Keyed): Int {
    if (left.ascii != null && right.ascii != null) {
        return AsciiCollation.compare(left.ascii, right.ascii)
    }
    return collationKeyOf(collator, left).compareTo(collationKeyOf(collator, right))
}

/** Returns the collation key of a candidate's name, building it on first use. */
private fun collationKeyOf(collator: Collator, candidate: Keyed): CollationKey =
    candidate.collationKey ?: collator.getCollationKey(candidate.name).also { key -> candidate.collationKey = key }

/** Returns the displayed name of the folder in [slot]. */
internal fun folderNameOf(index: SearchIndex, slot: Int): String = index.folders[slot].name

// What:     `private fun fullOrder(): Comparator<Keyed>` is the result order beyond tier and length.
// Why:      It is the old order: tier, shorter names, collation order, display path by code point, folder before track,
//           then library position, so two equal results never swap places between repeated searches.
//
// In TS you'd write (pseudocode):
// ```ts
// const fullOrder = (a, b) => a.tier - b.tier || a.len - b.len || a.key.compareTo(b.key) || /* ... */ 0;
// ```
/** The complete result order over [Keyed] candidates, comparing collated names with [collator]. */
private fun fullOrder(collator: Collator): Comparator<Keyed> =
    Comparator { left, right ->
        /** Tier and length compared as one key. */
        val byRank: Int = rankKeyOf(left.packed).compareTo(rankKeyOf(right.packed))
        if (byRank != 0) {
            byRank
        } else {
            /** Collated name comparison. */
            val byName: Int = compareNames(collator, left, right)
            if (byName != 0) byName else compareTail(left, right)
        }
    }

/** Orders two candidates that tie on rank and collated name: display path, folder before track, then number. */
private fun compareTail(left: Keyed, right: Keyed): Int {
    /** Display path comparison by code point. */
    val byPath: Int = compareByCodePoint(left.sortPath, right.sortPath)
    if (byPath != 0) {
        return byPath
    }
    /** Whether each candidate is a track, so false sorts first and a folder precedes a track. */
    val leftTrack: Boolean = idOfCandidate(left.packed) and FOLDER_BIT == 0
    /** Whether the right candidate is a track. */
    val rightTrack: Boolean = idOfCandidate(right.packed) and FOLDER_BIT == 0
    if (leftTrack != rightTrack) {
        return leftTrack.compareTo(rightTrack)
    }
    return idOfCandidate(left.packed).compareTo(idOfCandidate(right.packed))
}

// What:     `private fun selectKth(values: LongArray, size: Int, k: Int)` partially sorts by tier and length.
// Why:      Finding the cut at the five-hundredth best candidate needs a selection, not a sort of every match.
//
// In TS you'd write (pseudocode):
// ```ts
// function selectKth(values: BigInt64Array, size: number, k: number) { /* quickselect on the rank key */ }
// ```
/** Rearranges `values[0, size)` so that the entry at [k] has its final rank key and no later entry ranks before it. */
private fun selectKth(values: LongArray, size: Int, k: Int) {
    /** Lowest index that may hold the answer. */
    var low = 0
    /** Highest index that may hold the answer. */
    var high: Int = size - 1
    while (low < high) {
        /** Rank key that splits the range. */
        val pivot: Long = rankKeyOf(values[(low + high) ushr 1])
        /** Position scanning up from the low end. */
        var left: Int = low
        /** Position scanning down from the high end. */
        var right: Int = high
        while (left <= right) {
            while (rankKeyOf(values[left]) < pivot) left += 1
            while (rankKeyOf(values[right]) > pivot) right -= 1
            if (left <= right) {
                /** Entry swapped across the split. */
                val moved: Long = values[left]
                values[left] = values[right]
                values[right] = moved
                left += 1
                right -= 1
            }
        }
        if (k <= right) {
            high = right
        } else if (k >= left) {
            low = left
        } else {
            return
        }
    }
}

// What:     `internal fun selectBest(index: SearchIndex, matched: LongArray, size: Int, keep: Int): LongArray` keeps
//           the best candidates in result order.
// Why:      Partitioning by the integer keys finds the cut without string keys. Candidates strictly better than the cut
//           are all kept; those equal to the cut are ordered by the string keys to fill the remaining places. Only
//           those and the kept ones get string keys, instead of every match.
//
// In TS you'd write (pseudocode):
// ```ts
// function selectBest(matched: bigint[], keep: number): bigint[] { /* nth element, then tie-break */ }
// ```
/** Returns the best [keep] of `matched[0, size)` in result order; the input array is rearranged. */
internal fun selectBest(index: SearchIndex, matched: LongArray, size: Int, keep: Int): LongArray {
    if (keep == 0 || size == 0) {
        return LongArray(0)
    }
    /** Collator for this selection; collators are not safe to share between threads. */
    val collator: Collator = railCollator()
    /** Candidates that survive the integer cut, with their string keys. */
    val survivors = ArrayList<Keyed>(minOf(size, keep))
    if (size > keep) {
        selectKth(matched, size, keep - 1)
        /** Rank key of the last place. */
        val cut: Long = rankKeyOf(matched[keep - 1])
        /** Candidates that tie with the last place and compete for what remains. */
        val tied = ArrayList<Keyed>()
        for (position in 0 until size) {
            /** Rank key of this candidate. */
            val key: Long = rankKeyOf(matched[position])
            if (key < cut) {
                survivors.add(keyedFor(index, matched[position]))
            } else if (key == cut) {
                tied.add(keyedFor(index, matched[position]))
            }
        }
        /** Places left for the tied candidates after every better candidate took one. */
        val places: Int = keep - survivors.size
        tied.sortWith(fullOrder(collator))
        survivors.addAll(tied.subList(0, places))
    } else {
        for (position in 0 until size) {
            survivors.add(keyedFor(index, matched[position]))
        }
    }
    survivors.sortWith(fullOrder(collator))
    return LongArray(survivors.size) { position -> survivors[position].packed }
}
