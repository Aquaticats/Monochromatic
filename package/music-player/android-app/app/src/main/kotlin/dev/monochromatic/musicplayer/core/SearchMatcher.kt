// Search matching for the Search page. It normalizes names, decides whether every query term starts a word
// of a name, ranks how closely a name matches, and reports the character ranges that D59 highlights. It has
// no Android types, so the JVM unit tests run it directly.

// What:     `package dev.monochromatic.musicplayer.core` places the matcher beside the other pure logic.
// Why:      Matching, ranking, and highlighting must be testable without Compose or a device.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import java.text.Normalizer` brings in the Unicode normalization routine.
// Why:      NFKD splits an accented letter into its base letter plus combining marks, so the marks can be dropped.
//
// In TS you'd write (pseudocode):
// ```ts
// // String.prototype.normalize("NFKD")
// ```
import java.text.Normalizer

// What:     `import java.util.Locale` brings in the locale used for case mapping.
// Why:      `Locale.ROOT` gives the same lowercase result on every device, whatever the user's language.
//
// In TS you'd write (pseudocode):
// ```ts
// // "".toLowerCase() with no locale-specific rules
// ```
import java.util.Locale

// What:     `internal fun isWordSeparator(character: Char): Boolean` tells whether a new word begins after a character.
// Why:      A term may start a name or start a word after one of these characters, but never inside a word. The
//           index tests it for every character of every name, so it is a `when` and not a boxed set lookup.
//
// In TS you'd write (pseudocode):
// ```ts
// const isWordSeparator = (c: string) => [" ", "-", "_", ".", "(", "[", "/"].includes(c);
// ```
/** Reports whether a new word begins after [character] inside a normalized name. */
internal fun isWordSeparator(character: Char): Boolean = when (character) {
    ' ', '-', '_', '.', '(', '[', '/' -> true
    else -> false
}

// What:     `private val WHITESPACE_RUN: Regex` matches one or more whitespace characters.
// Why:      The query is split on runs of whitespace so that repeated spaces produce no empty term.
//
// In TS you'd write (pseudocode):
// ```ts
// const WHITESPACE_RUN = /\s+/;
// ```
/** Matches one or more whitespace characters, used to split a query into terms. */
private val WHITESPACE_RUN: Regex = Regex("\\s+")

// What:     `private const val TIER_WHOLE_NAME: Int = 0` is the best relevance tier.
// Why:      A name that equals the whole query is the strongest answer and sorts first.
//
// In TS you'd write (pseudocode):
// ```ts
// const TIER_WHOLE_NAME = 0;
// ```
/** Relevance tier for a name that equals the whole normalized query. */
private const val TIER_WHOLE_NAME: Int = 0

// What:     `private const val TIER_NAME_PREFIX: Int = 1` is the middle relevance tier.
// Why:      A name that starts with the first term, with every term matching, ranks below an exact name.
//
// In TS you'd write (pseudocode):
// ```ts
// const TIER_NAME_PREFIX = 1;
// ```
/** Relevance tier for a name that starts with the first query term while every term matches. */
private const val TIER_NAME_PREFIX: Int = 1

// What:     `private const val TIER_WORD_START: Int = 2` is the weakest relevance tier.
// Why:      Every term matches at some word start, but no term-first prefix rule applies.
//
// In TS you'd write (pseudocode):
// ```ts
// const TIER_WORD_START = 2;
// ```
/** Relevance tier for a name where every term matches at a word start, none of them at the very start. */
private const val TIER_WORD_START: Int = 2

// What:     `data class SearchHighlight(val start: Int, val end: Int)` is one highlighted character range.
// Why:      The UI paints the range in the original text, so offsets must refer to that text,
//           not the normalized form.
//
// In TS you'd write (pseudocode):
// ```ts
// type SearchHighlight = { start: number; end: number };
// ```
/** One range of the original text to emphasize, with an inclusive start and an exclusive end. */
data class SearchHighlight(
    /** Index of the first highlighted UTF-16 unit in the original text. */
    val start: Int,
    /** Index just past the last highlighted UTF-16 unit in the original text. */
    val end: Int,
)

// What:     `private class NormalizedText(...)` pairs normalized text with the original offsets of each character.
// Why:      NFKD can change the length of a text, so each normalized character must map back to its source.
//
// In TS you'd write (pseudocode):
// ```ts
// type NormalizedText = { text: string; originStarts: number[]; originEnds: number[] };
// ```
/** Normalized text with the original start and end offsets of every normalized character. */
private class NormalizedText(
    /** Normalized characters, concatenated in the order of the original text. */
    val text: String,
    /** Original start offset of the code point that produced each normalized character. */
    val originStarts: List<Int>,
    /** Original end offset of each normalized character, extended over combining marks that followed it. */
    val originEnds: List<Int>,
)

// What:     `private fun isCombiningMark(codePoint: Int): Boolean` tests for a non-spacing combining mark.
// Why:      Only `Mn` marks are removed. Spacing marks (`Mc`) and enclosing marks (`Me`) can be part of
//           a letter in Indic scripts, so removing them would change the word.
//
// In TS you'd write (pseudocode):
// ```ts
// const isCombiningMark = (c: string) => /\p{Mn}/u.test(c);
// ```
/** Reports whether a code point is a non-spacing combining mark, such as an accent that follows its base letter. */
private fun isCombiningMark(codePoint: Int): Boolean =
    Character.getType(codePoint) == Character.NON_SPACING_MARK.toInt()

// What:     `private fun stripCombiningMarks(text: String): String` removes every non-spacing mark from a text.
// Why:      After NFKD, an accented letter is a base letter followed by marks, and only the base should remain.
//
// In TS you'd write (pseudocode):
// ```ts
// const stripCombiningMarks = (s: string) => s.replace(/\p{Mn}/gu, "");
// ```
/** Removes non-spacing combining marks from decomposed text, one code point at a time. */
private fun stripCombiningMarks(text: String): String {
    /** Builder that collects every code point that is not a combining mark. */
    val builder = StringBuilder()
    /** Offset of the next code point to inspect in the decomposed text. */
    var index = 0
    while (index < text.length) {
        /** Code point starting at the current offset. */
        val codePoint = text.codePointAt(index)
        if (!isCombiningMark(codePoint)) {
            builder.appendCodePoint(codePoint)
        }
        index += Character.charCount(codePoint)
    }
    return builder.toString()
}

// What:     `private fun normalizeCodePoint(codePoint: Int): String` normalizes one code point on its own.
// Why:      Normalizing one code point at a time keeps every normalized character tied to one original range,
//           and it avoids context-dependent lowercasing such as the final sigma rule.
//
// In TS you'd write (pseudocode):
// ```ts
// const normalizeCodePoint = (c: string) => c.normalize("NFKD").replace(/\p{Mn}/gu, "").toLowerCase();
// ```
/** Normalizes one code point with NFKD, drops its combining marks, and lowercases it with the root locale. */
private fun normalizeCodePoint(codePoint: Int): String {
    /** The single code point as a string, ready for decomposition. */
    val single: String = String(Character.toChars(codePoint))
    /** The code point decomposed with NFKD, so compatibility forms and accents split into parts. */
    val decomposed: String = Normalizer.normalize(single, Normalizer.Form.NFKD)
    return stripCombiningMarks(decomposed).lowercase(Locale.ROOT)
}

// What:     `internal fun normalizedPieceOf(codePoint: Int): String` exposes the one-code-point normalization.
// Why:      The Search index normalizes a million names and caches this result per code point, and it must apply the
//           same rule as the matcher or the two would disagree about a name.
//
// In TS you'd write (pseudocode):
// ```ts
// export const normalizedPieceOf = (c: string) => c.normalize("NFKD").replace(/\p{Mn}/gu, "").toLowerCase();
// ```
/** Returns the normalized form of one code point: empty for a lone combining mark, else its lowercase base letters. */
internal fun normalizedPieceOf(codePoint: Int): String = normalizeCodePoint(codePoint)

// What:     `private fun normalizeWithOrigins(original: String): NormalizedText` normalizes a text and records
//           where every normalized character came from.
// Why:      Matching runs on the normalized text, while highlights must land on the original text's offsets.
//
// In TS you'd write (pseudocode):
// ```ts
// function normalizeWithOrigins(original: string): NormalizedText { /* per code point, with offsets */ }
// ```
/** Normalizes a text code point by code point, keeping the original offsets of every output character. */
private fun normalizeWithOrigins(original: String): NormalizedText {
    /** Builder that collects the normalized characters in order. */
    val builder = StringBuilder()
    /** Original start offset of the code point that produced each normalized character. */
    val starts = ArrayList<Int>()
    /** Original end offset of each normalized character. */
    val ends = ArrayList<Int>()
    /** Offset of the next code point to read from the original text. */
    var index = 0
    while (index < original.length) {
        /** Code point starting at the current offset. */
        val codePoint = original.codePointAt(index)
        /** Offset just past this code point in the original text. */
        val next = index + Character.charCount(codePoint)
        /** Normalized characters produced by this code point, empty for a lone combining mark. */
        val piece = normalizeCodePoint(codePoint)
        if (piece.isEmpty()) {
            if (ends.isNotEmpty()) {
                ends[ends.lastIndex] = next
            }
        } else {
            for (character in piece) {
                builder.append(character)
                starts.add(index)
                ends.add(next)
            }
        }
        index = next
    }
    return NormalizedText(builder.toString(), starts, ends)
}

// What:     `fun normalizeForSearch(text: String): String` returns the matching form of a name or query.
// Why:      Case and accents must not change whether a name matches, so both sides use this one normalization.
//
// In TS you'd write (pseudocode):
// ```ts
// const normalizeForSearch = (text: string) => normalizeWithOrigins(text).text;
// ```
/**
 * Returns the NFKD, combining-mark-free, lowercased form of a text, with its spaces left as they were.
 *
 * Kana voicing marks are combining marks, so `が` normalizes to the same text as `か`, and halfwidth katakana
 * normalizes to full-width katakana. Hiragana and katakana remain distinct.
 */
fun normalizeForSearch(text: String): String = normalizeWithOrigins(text).text

// What:     `private fun isWordStart(normalized: String, index: Int): Boolean` tests whether a term may
//           begin at an index.
// Why:      A term matches only at the start of the text or right after a word separator, so `cam` never matches
//           inside `Scamper`.
//
// In TS you'd write (pseudocode):
// ```ts
// const isWordStart = (s: string, i: number) => i === 0 || WORD_SEPARATORS.has(s[i - 1]);
// ```
/** Reports whether an index in a normalized text begins a word. */
private fun isWordStart(normalized: String, index: Int): Boolean =
    index == 0 || isWordSeparator(normalized[index - 1])

// What:     `private fun wordStartOccurrences(normalized: String, term: String): List<Int>` lists every index where
//           the term begins a word.
// Why:      Both matching and highlighting need every occurrence, not only the first one.
//
// In TS you'd write (pseudocode):
// ```ts
// function wordStartOccurrences(s: string, term: string): number[] { /* every indexOf hit that isWordStart */ }
// ```
/** Lists every index in a normalized text where the term occurs at a word start. */
private fun wordStartOccurrences(normalized: String, term: String): List<Int> {
    /** Indices where the term occurs at a word start, in ascending order. */
    val occurrences = ArrayList<Int>()
    /** Next index where the term occurs, or a negative value when none remains. */
    var found = normalized.indexOf(term)
    while (found >= 0) {
        if (isWordStart(normalized, found)) {
            occurrences.add(found)
        }
        found = normalized.indexOf(term, found + 1)
    }
    return occurrences
}

// What:     `private fun mergeOverlapping(ranges: List<SearchHighlight>): List<SearchHighlight>` joins ranges that
//           overlap or touch, and returns the rest sorted by start.
// Why:      Two query terms can cover the same letters, and painting them twice would show a doubled range.
//
// In TS you'd write (pseudocode):
// ```ts
// function mergeOverlapping(ranges: Range[]): Range[] { /* sort by start, then merge */ }
// ```
/** Sorts highlight ranges by start and merges every pair that overlaps or touches. */
private fun mergeOverlapping(ranges: List<SearchHighlight>): List<SearchHighlight> {
    /** Result under construction, with each entry covering a disjoint span. */
    val merged = ArrayList<SearchHighlight>()
    /** Ranges in ascending order of start, then end. */
    val sorted: List<SearchHighlight> = ranges.sortedWith(
        compareBy<SearchHighlight>({ highlight -> highlight.start }, { highlight -> highlight.end }),
    )
    for (range in sorted) {
        /** The most recent merged range, or null before the first one. */
        val last: SearchHighlight? = merged.lastOrNull()
        if (last == null || range.start > last.end) {
            merged.add(range)
        } else {
            merged[merged.lastIndex] = SearchHighlight(last.start, maxOf(last.end, range.end))
        }
    }
    return merged
}

// What:     `fun searchTerms(query: String): List<String>` splits a query into its normalized terms.
// Why:      Every term must match a name (AND), so the query is split on whitespace after normalization.
//
// In TS you'd write (pseudocode):
// ```ts
// const searchTerms = (query: string) => normalizeForSearch(query).split(/\s+/).filter(Boolean);
// ```
/** Returns the normalized whitespace-separated terms of a query, or an empty list for a blank query. */
fun searchTerms(query: String): List<String> =
    normalizeForSearch(query).split(WHITESPACE_RUN).filter { term -> term.isNotEmpty() }

// What:     `fun highlightRanges(text: String, query: String): List<SearchHighlight>` finds every term occurrence
//           that starts a word of the text, in the text's own offsets.
// Why:      D59 emphasizes each visible match in place, so the ranges must map onto the original characters.
//
// In TS you'd write (pseudocode):
// ```ts
// function highlightRanges(text: string, query: string): Range[] { /* word-start hits, mapped back */ }
// ```
/**
 * Returns the ranges of the original text that every query term covers at a word start.
 *
 * Combining marks that follow a matched letter are included in its range, so a decomposed `o` plus a combining
 * diaeresis, queried with `co`, highlights all three code units. Overlapping or touching ranges are merged, and
 * the result is sorted by start.
 */
fun highlightRanges(text: String, query: String): List<SearchHighlight> =
    highlightRangesForTerms(text, searchTerms(query))

// What:     `fun highlightRangesForTerms(text: String, terms: List<String>): List<SearchHighlight>` is
//           `highlightRanges` for a query that was already split into normalized terms.
// Why:      A result list highlights up to five hundred names for one query, and the terms must be normalized once,
//           not once per name.
//
// In TS you'd write (pseudocode):
// ```ts
// function highlightRangesForTerms(text: string, terms: string[]): Range[] { /* same as highlightRanges */ }
// ```
/** Returns the highlight ranges of [text] for terms that `searchTerms` already produced. */
fun highlightRangesForTerms(text: String, terms: List<String>): List<SearchHighlight> {
    if (terms.isEmpty()) {
        return emptyList()
    }
    /** The text normalized with the original offsets of each character. */
    val normalized: NormalizedText = normalizeWithOrigins(text)
    /** One range per word-start occurrence of each term, in original offsets. */
    val ranges: List<SearchHighlight> = terms.flatMap { term ->
        wordStartOccurrences(normalized.text, term).map { occurrence ->
            SearchHighlight(
                normalized.originStarts[occurrence],
                normalized.originEnds[occurrence + term.length - 1],
            )
        }
    }
    return mergeOverlapping(ranges)
}

// What:     `fun matchTier(normalizedName: String, terms: List<String>): Int?` returns how closely a name matches,
//           or null when it does not match.
// Why:      Results sort by tier, so an exact name precedes a prefix, and a prefix precedes a later-word match.
//
// In TS you'd write (pseudocode):
// ```ts
// function matchTier(name: string, terms: string[]): number | null { /* 0, 1, 2, or null */ }
// ```
/**
 * Returns tier 0 when the name equals the whole query, tier 1 when it starts with the first term, and tier 2
 * when every term starts a word. Returns null when any term fails, and also when there are no terms, so an
 * empty query cannot match every name.
 *
 * The name is trimmed before comparison, so surrounding spaces do not demote an exact or prefix match.
 */
fun matchTier(normalizedName: String, terms: List<String>): Int? {
    if (terms.isEmpty()) {
        return null
    }
    /** The normalized name without its surrounding whitespace. */
    val name: String = normalizedName.trim()
    /** Whether every term starts at least one word of the name. */
    val everyTermMatches: Boolean = terms.all { term -> wordStartOccurrences(name, term).isNotEmpty() }
    if (!everyTermMatches) {
        return null
    }
    if (name == terms.joinToString(" ")) {
        return TIER_WHOLE_NAME
    }
    if (name.startsWith(terms.first())) {
        return TIER_NAME_PREFIX
    }
    return TIER_WORD_START
}
