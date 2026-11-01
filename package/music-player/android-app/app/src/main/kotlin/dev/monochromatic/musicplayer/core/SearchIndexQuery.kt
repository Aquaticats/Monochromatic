// Answering a Search query from the index. `search` turns a query into the same `SearchState` the old matcher
// produced: it picks the smallest bucket among the query terms, tiers its members, merges the matching folders, keeps
// the best five hundred in result order, and only then builds rows with parent context and highlights.

// What:     `package dev.monochromatic.musicplayer.core` places the query path beside the index it reads.
// Why:      D112 requires the answer to equal the old matcher's while costing milliseconds at one million tracks, so
//           the work per keystroke is the bucket size plus five hundred rows, not the library size.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

/** Path separator, which splits a display path into its parent context and its file name. */
private const val CONTEXT_SEPARATOR: Char = '/'

/** Number of leading units of a term that choose its bucket. */
private const val BUCKET_UNITS: Int = 3

/** Ratio of list sizes from which a binary search per member of the shorter list beats a merge. */
private const val GALLOP_RATIO: Int = 16

/** A bucket with no members, shared by every query term that no word starts with. */
private val EMPTY_BUCKET: IntList = IntList(IntArray(0), 0)

// What:     `private fun parentContextOf(displayPath: String): String?` returns the folder path that holds a track.
// Why:      The second line of a track result names its folder; a root-level track has no parent.
//
// In TS you'd write (pseudocode):
// ```ts
// const parentContextOf = (p: string) => (p.includes("/") ? p.slice(0, p.lastIndexOf("/")) || null : null);
// ```
/** Returns the display path without its last segment, or null for a root-level track. */
private fun parentContextOf(displayPath: String): String? {
    /** The display path without its last segment, which is empty for a root-level track. */
    val parent: String = displayPath.substringBeforeLast(CONTEXT_SEPARATOR, "")
    return parent.ifEmpty { null }
}

// What:     `private fun bucketFor(index: SearchIndex, term: String): IntList?` picks the bucket a term draws from.
// Why:      A match of the term starts at a word start and covers its first characters up to the first separator, so
//           those one to three characters select the bucket. A term that starts with a separator has no bucket.
//
// In TS you'd write (pseudocode):
// ```ts
// const bucketFor = (index: SearchIndex, term: string) => index.byFirst.get(term[0]) ?? [];
// ```
/** Returns the bucket for [term], an empty bucket when no word has that start, or null when no bucket applies. */
private fun bucketFor(index: SearchIndex, term: String): IntList? {
    /** Units of the term before its first separator, at most three. */
    val head: String = term.takeWhile { unit -> !isWordSeparator(unit) }.take(BUCKET_UNITS)
    return when (head.length) {
        0 -> null
        1 -> index.bucketFirst(head[0]) ?: EMPTY_BUCKET
        2 -> index.bucketPair(head[0], head[1]) ?: EMPTY_BUCKET
        else -> index.bucketTriple(head[0], head[1], head[2]) ?: EMPTY_BUCKET
    }
}

/** Reports whether a bucket proves [term]: the term is at most three units and none of them is a separator. */
private fun isProvenByBucket(term: String): Boolean =
    term.length <= BUCKET_UNITS && term.none { unit -> isWordSeparator(unit) }

/** Reports whether [terms] is one term of one unit that is not a separator, so the cache can answer it. */
private fun singleUnitOf(terms: List<String>): Char? {
    if (terms.size != 1 || terms[0].length != 1 || isWordSeparator(terms[0][0])) {
        return null
    }
    return terms[0][0]
}

// What:     `private fun intersect(...)` intersects two ascending lists of track numbers.
// Why:      A query with several terms matches the tracks in every term's bucket. A merge costs the sum of the sizes,
//           and when one list is much longer a binary search per member of the shorter one costs less.
//
// In TS you'd write (pseudocode):
// ```ts
// const intersect = (a: number[], b: number[]) => a.filter((x) => binarySearch(b, x) >= 0);
// ```
/** Returns the track numbers present in both `a[0, aSize)` and `b[0, bSize)`, ascending. */
private fun intersect(a: IntArray, aSize: Int, b: IntArray, bSize: Int): IntArray {
    /** Matches found so far, with room for the shorter list. */
    val found = IntArray(minOf(aSize, bSize))
    /** Number of matches found so far. */
    var count = 0
    if (aSize * GALLOP_RATIO < bSize || bSize * GALLOP_RATIO < aSize) {
        /** The shorter list, whose members are looked up in the longer one. */
        val shortFirst: Boolean = aSize < bSize
        /** The shorter list. */
        val small: IntArray = if (shortFirst) a else b
        /** Entries of the shorter list. */
        val smallSize: Int = if (shortFirst) aSize else bSize
        /** The longer list. */
        val large: IntArray = if (shortFirst) b else a
        /** Entries of the longer list. */
        val largeSize: Int = if (shortFirst) bSize else aSize
        for (position in 0 until smallSize) {
            if (java.util.Arrays.binarySearch(large, 0, largeSize, small[position]) >= 0) {
                found[count] = small[position]
                count += 1
            }
        }
        return found.copyOf(count)
    }
    /** Position in the first list. */
    var left = 0
    /** Position in the second list. */
    var right = 0
    while (left < aSize && right < bSize) {
        if (a[left] < b[right]) {
            left += 1
        } else if (a[left] > b[right]) {
            right += 1
        } else {
            found[count] = a[left]
            count += 1
            left += 1
            right += 1
        }
    }
    return found.copyOf(count)
}

/** The candidates that every bucketed term allows, as the smallest bucket's members narrowed by the others. */
private fun candidatesOf(buckets: List<IntList>): IntArray {
    /** Buckets from the smallest, so the first intersection starts from the fewest members. */
    val ordered: List<IntList> = buckets.sortedBy { bucket -> bucket.size }
    /** Candidates after the intersections so far. */
    var current: IntArray = ordered[0].data
    /** Number of entries in the candidates. */
    var currentSize: Int = ordered[0].size
    for (position in 1 until ordered.size) {
        current = intersect(current, currentSize, ordered[position].data, ordered[position].size)
        currentSize = current.size
    }
    return if (current.size == currentSize) current else current.copyOf(currentSize)
}

/** Ranks the tracks that match [terms]: from the cache for one character, else from the terms' buckets. */
private fun SearchIndex.trackMatches(
    terms: List<String>,
    termChars: Array<CharArray>,
    joined: CharArray,
): RankedAnswer {
    /** The only unit of a one-character query, or null. */
    val unit: Char? = singleUnitOf(terms)
    if (unit != null) {
        return singleUnitTracks(this, unit)
    }
    /** The buckets of the terms that have one. */
    val buckets: List<IntList> = terms.mapNotNull { term -> bucketFor(this, term) }
    /** The terms that no bucket proves, which each candidate's name must be scanned for. */
    val unproven: Array<CharArray> = terms.indices.filter { position -> !isProvenByBucket(terms[position]) }
        .map { position -> termChars[position] }.toTypedArray()
    /** Packed matches among the candidates. */
    val matches: LongArray = if (buckets.isEmpty()) {
        tierVerified(this, everyTrack(), unproven, termChars, joined)
    } else if (unproven.isEmpty()) {
        /** Candidates that every bucket allows, all proven matches. */
        val proven: IntArray = candidatesOf(buckets)
        tierMatched(this, proven, 0, proven.size, termChars[0], joined)
    } else {
        tierVerified(this, candidatesOf(buckets), unproven, termChars, joined)
    }
    return RankedAnswer(selectBest(this, matches, matches.size, TRACK_KEEP), matches.size)
}

/** Lists every track number, for a query whose terms all start with a separator. */
private fun SearchIndex.everyTrack(): IntArray = IntArray(trackCount) { position -> position }

/** Ranks the listed folders that match. Loose root-level tracks have no folder result of their own (D28). */
private fun SearchIndex.folderMatches(termChars: Array<CharArray>, joined: CharArray): LongArray {
    /** Packed matching folders. */
    val found = ArrayList<Long>()
    for (slot in listedFolderSlots()) {
        /** The folder's record. */
        val record: FolderRecord = folders[slot]
        /** Tier of the folder name, or [NO_TIER]. */
        val tier: Int = fastTier(record.norm, 0, record.norm.size, termChars, joined)
        if (tier != NO_TIER) {
            found.add(packCandidate(tier, record.norm.size, FOLDER_BIT or slot))
        }
    }
    return found.toLongArray()
}

/** Builds the row of one folder candidate. */
private fun SearchIndex.folderResult(packed: Long, terms: List<String>): SearchResult {
    /** Displayed name of the folder. */
    val name: String = folderNameOf(this, idOfCandidate(packed) and FOLDER_BIT.inv())
    return SearchResult(
        kind = SearchResultKind.FOLDER,
        name = name,
        parentContext = null,
        indexInLibrary = null,
        folderName = name,
        tier = tierOfCandidate(packed),
        nameHighlights = highlightRangesForTerms(name, terms),
        parentHighlights = emptyList(),
    )
}

/** Builds the row of one track candidate, with its title, folder context and highlights. */
private fun SearchIndex.trackResult(packed: Long, terms: List<String>): SearchResult {
    /** Track number. */
    val position: Int = idOfCandidate(packed)
    /** Display path of the track. */
    val path: String = paths[position].orEmpty()
    /** Title of the track, which is its file name without the last extension. */
    val title: String = trackTitleFor(path)
    /** Folder path of the track as displayed, or null at the root. */
    val parentContext: String? = parentContextOf(path)
    /** Top-level folder of the path and the number of meaningful segments. */
    val folder: FolderOfPath = folderOf(path)
    return SearchResult(
        kind = SearchResultKind.TRACK,
        name = title,
        parentContext = parentContext,
        indexInLibrary = position,
        folderName = if (folder.first != null && folder.segments >= 2) {
            folder.first
        } else {
            looseFolderLabel(folder.first.orEmpty())
        },
        tier = tierOfCandidate(packed),
        nameHighlights = highlightRangesForTerms(title, terms),
        parentHighlights = if (parentContext == null) emptyList() else highlightRangesForTerms(parentContext, terms),
        trackUri = uris[position],
    )
}

// What:     `fun SearchIndex.search(query: String): SearchState` answers one query over every track appended so far.
// Why:      It is the whole of Search: the same states, order, cap and highlights as the old matcher.
//
// In TS you'd write (pseudocode):
// ```ts
// const state = index.search("cam"); // { kind: "results", items, truncated }
// ```
/**
 * Evaluates [query] against every track appended so far and reports the page state.
 *
 * A blank query returns `EmptyQuery`, an index without tracks `EmptyInventory`, and a complete evaluation with no hit
 * `NoMatch`. Otherwise the results are ordered and cut at `SEARCH_RESULT_LIMIT`.
 */
fun SearchIndex.search(query: String): SearchState {
    /** Normalized query terms, which every match must satisfy. */
    val terms: List<String> = searchTerms(query)
    if (terms.isEmpty()) {
        return SearchState.EmptyQuery
    }
    if (trackCount == 0) {
        return SearchState.EmptyInventory
    }
    /** The terms as character arrays, which the tier test reads without allocating. */
    val termChars: Array<CharArray> = Array(terms.size) { position -> terms[position].toCharArray() }
    /** The terms joined by a space, which only a whole-name match equals. */
    val joined: CharArray = terms.joinToString(" ").toCharArray()
    /** Best tracks and the number of matching tracks. */
    val tracks: RankedAnswer = trackMatches(terms, termChars, joined)
    /** Matching folders, packed. */
    val folderCandidates: LongArray = folderMatches(termChars, joined)
    /** Number of matches of both kinds. */
    val total: Int = tracks.matched + folderCandidates.size
    if (total == 0) {
        return SearchState.NoMatch(query.trim())
    }
    /** Track candidates followed by folder candidates, to be cut together. */
    val combined: LongArray = tracks.best + folderCandidates
    /** The best candidates of both kinds in result order. */
    val best: LongArray = selectBest(this, combined, combined.size, SEARCH_RESULT_LIMIT)
    return SearchState.Results(
        items = best.map { packed ->
            if (idOfCandidate(packed) and FOLDER_BIT != 0) folderResult(packed, terms) else trackResult(packed, terms)
        },
        truncated = total > SEARCH_RESULT_LIMIT,
    )
}
