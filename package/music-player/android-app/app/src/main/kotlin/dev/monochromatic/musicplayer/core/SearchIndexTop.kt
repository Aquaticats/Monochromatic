// Best-five-hundred answer per first character, kept exact while the library grows. A one-character query matches a
// large share of the library, so ranking its whole bucket costs tens of milliseconds at one million tracks. The
// library only grows and the result order is total, so the best five hundred of the first N tracks plus every match
// among the later ones is exactly the best five hundred of all of them.

// What:     `package dev.monochromatic.musicplayer.core` places the cache beside the index it serves.
// Why:      The cache reads the index's buckets and ranking, and the warm-up pass reads the cache.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import java.util.concurrent.atomic.AtomicReferenceArray

/** Number of distinct UTF-16 units, which is the size of the cache table. */
private const val UNIT_COUNT: Int = 0x10000

// What:     `internal class TopEntry(...)` is the exact answer of a one-character query over the first tracks.
// Why:      See the file description: serving it plus the tracks appended since is exact, and storing the merge lets
//           the next keystroke start from it.
//
// In TS you'd write (pseudocode):
// ```ts
// type TopEntry = { through: number; matched: number; best: bigint[] };
// ```
/** The best tracks and the match count of a one-character query over tracks numbered below [through]. */
internal class TopEntry(
    /** Tracks numbered below this are covered. */
    val through: Int,
    /** Number of covered tracks that match. */
    val matched: Int,
    /** The best `TRACK_KEEP` covered matches, packed, in result order. */
    val best: LongArray,
)

// What:     `internal class TopCache` stores one [TopEntry] per first character.
// Why:      The query worker and the warm-up pass both read and store entries. Every entry is exact for the tracks it
//           covers, so whichever write lands last is a valid starting point and no lock is needed.
//
// In TS you'd write (pseudocode):
// ```ts
// class TopCache { entries = new Map<number, TopEntry>(); }
// ```
/** The cache of one-character answers, safe to share between threads. */
internal class TopCache {
    /** One slot per UTF-16 unit; a slot is null until that character has been asked about. */
    private val entries: AtomicReferenceArray<TopEntry> = AtomicReferenceArray(UNIT_COUNT)

    /** Returns the entry of [unit], or null when it was never computed. */
    fun get(unit: Char): TopEntry? = entries.get(unit.code)

    /** Stores the entry of [unit]; the newest merge replaces the older one. */
    fun put(unit: Char, entry: TopEntry) {
        entries.set(unit.code, entry)
    }

    /** Returns the number of tracks the entry of [unit] covers, or null without an entry. */
    fun covered(unit: Char): Int? = get(unit)?.through
}

/** A ranked answer: the best packed candidates in result order and the number of matches behind them. */
internal class RankedAnswer(
    /** The best candidates, packed, in result order. */
    val best: LongArray,
    /** Number of candidates that matched, which may exceed the length of [best]. */
    val matched: Int,
)

// What:     `internal fun tierMatched(...)` tiers bucket members whose bucket key is the whole term.
// Why:      Such a member is a proven match, so tiering it reads a few characters instead of scanning the name. Both
//           the cache merge and the short-term queries rank their members this way.
//
// In TS you'd write (pseudocode):
// ```ts
// const tierMatched = (index, ids, from, to, first, joined) =>
//   ids.slice(from, to).map((id) => pack(tierOfMatched(index, id)));
// ```
/** Tiers the proven-match members `members[from, to)` and returns them packed, in member order. */
internal fun tierMatched(
    index: SearchIndex,
    members: IntArray,
    from: Int,
    to: Int,
    first: CharArray,
    joined: CharArray,
): LongArray {
    /** Packed candidates, one per member. */
    val packed = LongArray(to - from)
    /** Normalized titles of all tracks. */
    val chars: CharArray = index.chars
    for (position in from until to) {
        /** Track number of this member. */
        val track: Int = members[position]
        /** Start offset of the member's title. */
        val start: Int = index.titleStart(track)
        /** End offset of the member's title. */
        val end: Int = index.ends[track]
        packed[position - from] = packCandidate(tierOfMatched(chars, start, end, first, joined), end - start, track)
    }
    return packed
}

// What:     `internal fun tierVerified(...)` tiers candidate tracks after checking terms that no bucket proves.
// Why:      A term longer than the bucket key, or one that starts with a separator, still has to be found at a word
//           start of each candidate's name; the other terms are already proven by the buckets that produced the
//           candidates.
//
// In TS you'd write (pseudocode):
// ```ts
// const tierVerified = (index, ids, unproven, terms, joined) =>
//   ids.filter((id) => unproven.every((t) => hasWordStart(id, t)));
// ```
/** Scans each candidate for the [unproven] terms and returns the matching ones packed, in candidate order. */
internal fun tierVerified(
    index: SearchIndex,
    candidates: IntArray,
    unproven: Array<CharArray>,
    terms: Array<CharArray>,
    joined: CharArray,
): LongArray {
    /** Matches found so far, with room for every candidate. */
    val found = LongArray(candidates.size)
    /** Number of matches found so far. */
    var count = 0
    /** Normalized titles of all tracks. */
    val chars: CharArray = index.chars
    for (track in candidates) {
        /** Start offset of the candidate's title. */
        val start: Int = index.titleStart(track)
        /** End offset of the candidate's title. */
        val end: Int = index.ends[track]
        if (allHaveWordStart(chars, start, end, unproven)) {
            found[count] = packCandidate(tierOfMatched(chars, start, end, terms[0], joined), end - start, track)
            count += 1
        }
    }
    return found.copyOf(count)
}

// What:     `internal fun singleUnitTracks(index: SearchIndex, unit: Char): RankedAnswer` ranks a one-character query.
// Why:      It serves the cached answer merged with the tracks appended after it, and stores the merge, so the next
//           keystroke starts from it.
//
// In TS you'd write (pseudocode):
// ```ts
// function singleUnitTracks(index: SearchIndex, unit: string): RankedAnswer { /* cached + delta */ }
// ```
/** Returns the best tracks for the query made of the single character [unit], with the number of matching tracks. */
internal fun singleUnitTracks(index: SearchIndex, unit: Char): RankedAnswer {
    /** The only term, which is also the joined terms. */
    val joined: CharArray = charArrayOf(unit)
    /** Tracks having a word that starts with the unit. */
    val bucket: IntList = index.bucketFirst(unit) ?: return RankedAnswer(LongArray(0), 0)
    /** Tracks known now, which this answer will cover. */
    val now: Int = index.trackCount
    /** The previous answer, if one exists. */
    val cached: TopEntry? = index.top.get(unit)
    /** Bucket position from which tracks are new to the cached answer. */
    val start: Int = if (cached == null) 0 else bucket.countBelow(cached.through)
    if (cached != null && start == bucket.size) {
        return RankedAnswer(cached.best, cached.matched)
    }
    /** Matches among the tracks the cached answer does not cover. */
    val fresh: LongArray = tierMatched(index, bucket.data, start, bucket.size, joined, joined)
    /** Candidates to select from: the cached best, then the fresh matches. */
    val combined: LongArray = if (cached == null) fresh else cached.best + fresh
    /** Number of matches among all covered tracks. */
    val matched: Int = (cached?.matched ?: 0) + fresh.size
    /** The best candidates of everything covered. */
    val best: LongArray = selectBest(index, combined, combined.size, TRACK_KEEP)
    index.top.put(unit, TopEntry(now, matched, best))
    return RankedAnswer(best, matched)
}

// What:     `fun SearchIndex.unitsBehind(): List<Char>` lists the first characters whose answer needs a refresh.
// Why:      A background pass refreshes them one at a time, largest bucket first, so the first keystroke of a common
//           letter is a cache read.
//
// In TS you'd write (pseudocode):
// ```ts
// const unitsBehind = (index: SearchIndex) => index.firstUnitSizes().filter(isBehind).sort(bySizeDesc);
// ```
/** Returns the first characters whose cached answer is missing or covers fewer tracks than the index holds. */
fun SearchIndex.unitsBehind(): List<Char> =
    firstUnitSizes()
        .filter { (unit, size) -> size > 0 && !isWordSeparator(unit) && (top.covered(unit) ?: -1) < trackCount }
        .sortedWith(compareByDescending<Pair<Char, Int>> { entry -> entry.second }.thenBy { entry -> entry.first })
        .map { entry -> entry.first }

/** Brings the cached answer of one first character up to date, so the next keystroke of it reads the cache. */
fun SearchIndex.warm(unit: Char) {
    if (!isWordSeparator(unit)) {
        singleUnitTracks(this, unit)
    }
}
