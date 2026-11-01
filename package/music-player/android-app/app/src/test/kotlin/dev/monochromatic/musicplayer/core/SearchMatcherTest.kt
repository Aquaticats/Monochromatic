// Search matcher cases: normalization, query terms, relevance tiers, word starts, and highlight offsets.

// What:     `package ...core` places these tests beside the matcher they exercise.
// Why:      The tests reach `normalizeForSearch`, `searchTerms`, `matchTier`, and `highlightRanges` without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     JUnit's assertions and test annotation register each matcher case.
// Why:      Each case compares an exact normalized string, tier, or highlight list.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** Verifies the Search matcher: normalization, terms, tiers, word starts, and highlight ranges. */
class SearchMatcherTest {
    // What:     `@Test fun normalizesAccentsAndCase()` checks that accents and case are removed for matching.
    // Why:      D59 requires `cam` to match `Cam`, and accent-insensitive matching follows from the same rule.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("normalizes accents and case", () => expect(normalize("Café Ö")).toBe("cafe o"));
    // ```
    /** Verifies accented Latin letters and capitals normalize to plain lowercase letters. */
    @Test
    fun normalizesAccentsAndCase() {
        assertEquals("cafe o", normalizeForSearch("Café Ö"))
        assertEquals("aei", normalizeForSearch("ÀÉÎ"))
    }

    // What:     `@Test fun decomposedAndComposedAccentsNormalizeEqually()` compares precomposed and decomposed text.
    // Why:      A file name may store `ö` as one code point or as `o` plus a combining mark, and both must match.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("composed and decomposed match", () => expect(normalize("Cö")).toBe(normalize("Co\u0308")));
    // ```
    /** Verifies a composed letter and its decomposed spelling produce the same normalized text. */
    @Test
    fun decomposedAndComposedAccentsNormalizeEqually() {
        assertEquals(normalizeForSearch("Cö"), normalizeForSearch("Co\u0308"))
        assertEquals("co", normalizeForSearch("Co\u0308"))
    }

    // What:     `@Test fun normalizationKeepsSpacesAndDoesNotTrim()` checks that surrounding spaces survive
    //           normalization.
    // Why:      Highlight offsets map onto the original text, so the normalized form must not drop characters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("keeps surrounding spaces", () => expect(normalize("  Cam ")).toBe("  cam "));
    // ```
    /** Verifies normalization leaves surrounding spaces in place. */
    @Test
    fun normalizationKeepsSpacesAndDoesNotTrim() {
        assertEquals("  cam ", normalizeForSearch("  Cam "))
    }

    // What:     `@Test fun normalizesCompatibilityForms()` checks full-width Latin, full-width space, and
    //           halfwidth kana.
    // Why:      NFKD folds compatibility forms, so a query typed in another width still finds the name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("compatibility forms", () => expect(normalize("Ａｂｃ　ｶﾀ")).toBe("abc カタ"));
    // ```
    /** Verifies full-width letters, full-width spaces, and halfwidth katakana fold to their common forms. */
    @Test
    fun normalizesCompatibilityForms() {
        assertEquals("abc カタ", normalizeForSearch("Ａｂｃ　ｶﾀ"))
        assertEquals("a b", normalizeForSearch("A　B"))
    }

    // What:     `@Test fun removesKanaVoicingMarks()` pins that the dakuten is dropped with the other combining marks.
    // Why:      The rule removes `Mn` marks, so `が` matches `か`. That consequence is the accepted cost of the rule.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("dakuten is stripped", () => expect(normalize("が")).toBe("か"));
    // ```
    /** Verifies a voiced kana normalizes to its unvoiced base, because its voicing mark is a combining mark. */
    @Test
    fun removesKanaVoicingMarks() {
        assertEquals("か", normalizeForSearch("が"))
    }

    // What:     `@Test fun keepsSpacingMarks()` checks that a Devanagari vowel sign survives normalization.
    // Why:      Only non-spacing marks are removed, because a spacing mark can be part of an Indic letter.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("spacing mark kept", () => expect(normalize("\u0915\u093F")).toBe("\u0915\u093F"));
    // ```
    /** Verifies a spacing combining mark is not removed from an Indic syllable. */
    @Test
    fun keepsSpacingMarks() {
        assertEquals("\u0915\u093F", normalizeForSearch("\u0915\u093F"))
    }

    // What:     `@Test fun lowercasesEachCodePointSeparately()` pins per-code-point lowercasing for the final sigma.
    // Why:      Lowercasing one code point at a time keeps every character tied to one original range.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("sigma per code point", () => expect(normalize("ΟΔΟΣ")).toBe("οδοσ"));
    // ```
    /** Verifies a capital sigma lowercases to the medial form, with no context-dependent final form. */
    @Test
    fun lowercasesEachCodePointSeparately() {
        assertEquals("οδοσ", normalizeForSearch("ΟΔΟΣ"))
    }

    // What:     `@Test fun searchTermsSplitsOnWhitespaceRuns()` checks that the query becomes its normalized terms.
    // Why:      Every whitespace-separated term must match a name, so repeated spaces must not create empty terms.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("terms split", () => expect(searchTerms("  Live   CAM ")).toEqual(["live", "cam"]));
    // ```
    /** Verifies a query splits into lowercase terms regardless of repeated or surrounding whitespace. */
    @Test
    fun searchTermsSplitsOnWhitespaceRuns() {
        assertEquals(listOf("live", "cam"), searchTerms("  Live   CAM "))
    }

    // What:     `@Test fun blankQueryHasNoTerms()` checks the empty and whitespace-only queries.
    // Why:      A blank query must produce no terms, so it cannot match anything.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("blank has no terms", () => expect(searchTerms("   ")).toEqual([]));
    // ```
    /** Verifies an empty or whitespace-only query has no terms. */
    @Test
    fun blankQueryHasNoTerms() {
        assertEquals(emptyList<String>(), searchTerms(""))
        assertEquals(emptyList<String>(), searchTerms("   "))
    }

    // What:     `@Test fun combiningOnlyQueryHasNoTerms()` checks a query made only of a combining mark.
    // Why:      Normalization removes the mark, leaving nothing to match, so the query is treated as empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("combining only", () => expect(searchTerms("́")).toEqual([]));
    // ```
    /** Verifies a query that normalizes to nothing produces no terms. */
    @Test
    fun combiningOnlyQueryHasNoTerms() {
        assertEquals(emptyList<String>(), searchTerms("́"))
    }

    // What:     `@Test fun wholeNameIsTierZero()` checks that a name equal to the query ranks first.
    // Why:      An exact name is the strongest match, and it must sort before a prefix (D61's accepted `Cam`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("tier 0", () => expect(matchTier("cam", ["cam"])).toBe(0));
    // ```
    /** Verifies a name that equals the whole query is tier zero. */
    @Test
    fun wholeNameIsTierZero() {
        assertEquals(0, matchTier("cam", listOf("cam")))
    }

    // What:     `@Test fun prefixNameIsTierOne()` checks that a name starting with the term is tier one.
    // Why:      D61 places the `Camellia` prefix folder after the exact `Cam` track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("tier 1", () => expect(matchTier("camellia", ["cam"])).toBe(1));
    // ```
    /** Verifies a name that starts with the term is tier one. */
    @Test
    fun prefixNameIsTierOne() {
        assertEquals(1, matchTier("camellia", listOf("cam")))
    }

    // What:     `@Test fun laterWordIsTierTwo()` checks that a term starting a later word is tier two.
    // Why:      D60's accepted example `Live at Camellia` matches `cam` at a later word start.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("tier 2", () => expect(matchTier("live at camellia", ["cam"])).toBe(2));
    // ```
    /** Verifies a term that begins a later word, not the name, yields tier two. */
    @Test
    fun laterWordIsTierTwo() {
        assertEquals(2, matchTier("live at camellia", listOf("cam")))
    }

    // What:     `@Test fun noMatchInsideAWord()` checks that a term inside one uninterrupted word does not match.
    // Why:      `cam` must not match `Scamper` or `Dreamcam`, because neither occurrence starts a word.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no mid-word", () => expect(matchTier("scamper", ["cam"])).toBeNull());
    // ```
    /** Verifies a term that appears only inside a word does not match that name. */
    @Test
    fun noMatchInsideAWord() {
        assertNull(matchTier("scamper", listOf("cam")))
        assertNull(matchTier("dreamcam", listOf("cam")))
    }

    // What:     `@Test fun wordStartsAfterEachSeparator()` checks every listed separator begins a word.
    // Why:      The word-start rule covers a space, `-`, `_`, `.`, `(`, `[`, and `/`, and each must count.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("separators", () => {
    //   for (const s of [" ", "-", "_", ".", "(", "[", "/"]) expect(matchTier(`a${s}cam`, ["cam"])).toBe(2);
    // });
    // ```
    /** Verifies a term right after each separator character is a later-word match. */
    @Test
    fun wordStartsAfterEachSeparator() {
        assertEquals(2, matchTier("a cam", listOf("cam")))
        assertEquals(2, matchTier("a-cam", listOf("cam")))
        assertEquals(2, matchTier("a_cam", listOf("cam")))
        assertEquals(2, matchTier("a.cam", listOf("cam")))
        assertEquals(2, matchTier("a(cam", listOf("cam")))
        assertEquals(2, matchTier("a[cam", listOf("cam")))
        assertEquals(2, matchTier("a/cam", listOf("cam")))
    }

    // What:     `@Test fun everyTermMustMatch()` checks the AND rule across several terms.
    // Why:      A multi-term query narrows results, so one missing term rejects the name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("AND", () => expect(matchTier("cam only", ["live", "cam"])).toBeNull());
    // ```
    /** Verifies a name is rejected when any one query term has no word-start match. */
    @Test
    fun everyTermMustMatch() {
        assertNull(matchTier("cam only", listOf("live", "cam")))
    }

    // What:     `@Test fun termsMayMatchInAnyWordOrder()` checks that AND does not require the query's word order.
    // Why:      `live cam` must find `Camellia Live`, because each term starts some word of the name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("any order", () => expect(matchTier("camellia live", ["live", "cam"])).toBe(2));
    // ```
    /** Verifies every term matching at a word start yields tier two, even when the first term is not at the start. */
    @Test
    fun termsMayMatchInAnyWordOrder() {
        assertEquals(2, matchTier("camellia live", listOf("live", "cam")))
    }

    // What:     `@Test fun firstTermAtStartIsTierOne()` checks that a prefix of the first term ranks as tier one.
    // Why:      With several terms, tier one needs the name to start with the first term while every term
    //           still matches.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("tier 1 multi", () => expect(matchTier("live camellia", ["live", "cam"])).toBe(1));
    // ```
    /** Verifies a multi-term name starting with its first term is tier one. */
    @Test
    fun firstTermAtStartIsTierOne() {
        assertEquals(1, matchTier("live camellia", listOf("live", "cam")))
    }

    // What:     `@Test fun emptyTermsNeverMatch()` checks that an empty term list rejects every name.
    // Why:      An AND over no terms would otherwise be vacuously true and match the whole library.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no terms", () => expect(matchTier("cam", [])).toBeNull());
    // ```
    /** Verifies no name matches when there are no query terms. */
    @Test
    fun emptyTermsNeverMatch() {
        assertNull(matchTier("cam", emptyList()))
    }

    // What:     `@Test fun surroundingSpacesDoNotDemoteTier()` checks that the name is trimmed before tiering.
    // Why:      A stray space in a file name must not turn an exact or prefix match into a later-word match.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("trimmed", () => expect(matchTier(" cam ", ["cam"])).toBe(0));
    // ```
    /** Verifies leading and trailing spaces in the normalized name leave the tier unchanged. */
    @Test
    fun surroundingSpacesDoNotDemoteTier() {
        assertEquals(0, matchTier(" cam ", listOf("cam")))
    }

    // What:     `@Test fun highlightsAccentedWordInOriginalOffsets()` checks the `Cö shu Nie` example.
    // Why:      Highlight offsets must address the original text, here a composed `ö` after `C`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("Cö", () => expect(highlightRanges("Cö shu Nie", "co")).toEqual([{ start: 0, end: 2 }]));
    // ```
    /** Verifies a composed accented letter is highlighted as one range in the original text. */
    @Test
    fun highlightsAccentedWordInOriginalOffsets() {
        assertEquals(listOf(SearchHighlight(0, 2)), highlightRanges("Cö shu Nie", "co"))
    }

    // What:     `@Test fun highlightIncludesTrailingCombiningMark()` checks a decomposed letter with its mark.
    // Why:      The mark belongs to the letter, so the range must not split the base from its diaeresis.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("decomposed", () => expect(highlightRanges("Co\u0308 shu", "co")).toEqual([{ start: 0, end: 3 }]));
    // ```
    /** Verifies a decomposed letter's combining mark is inside the highlighted range. */
    @Test
    fun highlightIncludesTrailingCombiningMark() {
        assertEquals(listOf(SearchHighlight(0, 3)), highlightRanges("Co\u0308 shu", "co"))
    }

    // What:     `@Test fun highlightsEveryWordStart()` checks that each word-start occurrence is highlighted.
    // Why:      D59 emphasizes every visible match, so a repeated word start must produce a second range.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("all starts", () => expect(highlightRanges("Cam Live Cam", "cam")).toEqual([...]));
    // ```
    /** Verifies every word-start occurrence of the term gets its own highlight range. */
    @Test
    fun highlightsEveryWordStart() {
        assertEquals(
            listOf(SearchHighlight(0, 3), SearchHighlight(9, 12)),
            highlightRanges("Cam Live Cam", "cam"),
        )
    }

    // What:     `@Test fun highlightSkipsMidWordOccurrences()` checks that `Scamper` is not highlighted for `cam`.
    // Why:      Highlights and matches must agree, so only the word-start `Cam` is emphasized.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("skip mid-word", () => expect(highlightRanges("Scamper Cam", "cam")).toEqual([{ start: 8, end: 11 }]));
    // ```
    /** Verifies an occurrence inside a word is not highlighted while a word-start occurrence is. */
    @Test
    fun highlightSkipsMidWordOccurrences() {
        assertEquals(listOf(SearchHighlight(8, 11)), highlightRanges("Scamper Cam", "cam"))
    }

    // What:     `@Test fun highlightsJapaneseWordStartAfterSpace()` checks a Japanese term after a space.
    // Why:      Japanese names follow the same word-start rule, so `ラジオ` after a space must be emphasized.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("Japanese", () => expect(highlightRanges("東京 ラジオ", "ラジオ")).toEqual([{ start: 3, end: 6 }]));
    // ```
    /** Verifies a Japanese term at a word start after a space is highlighted in original offsets. */
    @Test
    fun highlightsJapaneseWordStartAfterSpace() {
        assertEquals(listOf(SearchHighlight(3, 6)), highlightRanges("東京 ラジオ", "ラジオ"))
    }

    // What:     `@Test fun unspacedJapaneseHasNoLaterWordMatch()` checks that a Japanese fragment inside a run
    //           does not match.
    // Why:      Unspaced Japanese has no word boundaries in this rule, so a middle fragment must not be highlighted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("unspaced", () => expect(highlightRanges("あいうえお", "いう")).toEqual([]));
    // ```
    /** Verifies a fragment in the middle of an unspaced Japanese run is not highlighted. */
    @Test
    fun unspacedJapaneseHasNoLaterWordMatch() {
        assertEquals(emptyList<SearchHighlight>(), highlightRanges("あいうえお", "いう"))
    }

    // What:     `@Test fun highlightCountsSurrogatePairsAsOneCodePoint()` checks an astral character before the match.
    // Why:      The range must count UTF-16 units, so a two-unit character shifts the offset by two.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("astral", () => expect(highlightRanges("\u{2000B} Cam", "cam")).toEqual([{ start: 3, end: 6 }]));
    // ```
    /** Verifies an astral code point before the match shifts the highlight by its two UTF-16 units. */
    @Test
    fun highlightCountsSurrogatePairsAsOneCodePoint() {
        assertEquals(listOf(SearchHighlight(3, 6)), highlightRanges("𠀋 Cam", "cam"))
    }

    // What:     `@Test fun overlappingTermsMergeIntoOneRange()` checks two terms that cover the same letters.
    // Why:      Painting the same letters twice would look wrong, so overlapping ranges become one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("merge", () => expect(highlightRanges("Camellia", "cam ca")).toEqual([{ start: 0, end: 3 }]));
    // ```
    /** Verifies two overlapping term occurrences produce one merged highlight range. */
    @Test
    fun overlappingTermsMergeIntoOneRange() {
        assertEquals(listOf(SearchHighlight(0, 3)), highlightRanges("Camellia", "cam ca"))
    }

    // What:     `@Test fun blankQueryAndEmptyTextHighlightNothing()` checks the no-op inputs for highlighting.
    // Why:      A blank query or an empty name must produce no ranges, so the UI draws no emphasis.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no-op", () => expect(highlightRanges("Cam", "")).toEqual([]));
    // ```
    /** Verifies a blank query and an empty text both return an empty highlight list. */
    @Test
    fun blankQueryAndEmptyTextHighlightNothing() {
        assertEquals(emptyList<SearchHighlight>(), highlightRanges("Cam", "   "))
        assertEquals(emptyList<SearchHighlight>(), highlightRanges("", "cam"))
    }
}
