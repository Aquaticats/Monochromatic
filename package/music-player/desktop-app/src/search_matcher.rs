//! Search matching for the Search page:
//!  normalization,
//!  query terms,
//!  word starts,
//!  relevance tiers,
//!  and highlights.
//!
//! Ported from `core/SearchMatcher.kt`.
//!  Matching runs on a normalized form of each name:
//!  NFKD,
//!  `Mn` combining
//! marks removed,
//!  then lowercased one character at a time.
//!  Highlights map back onto the original text.
//!
//! Unit of every highlight offset:
//!  `SearchHighlight::start` and `SearchHighlight::end` count Unicode scalar
//! values (Rust `char`s) in the original text,
//!  with an inclusive start and an exclusive end.
//!  Kotlin counts UTF-16
//! code units instead.
//!  The two agree for text in the Basic Multilingual Plane and differ by one for each character
//! above U+FFFF,
//!  so the UI layer converts these offsets to whatever unit its text API expects.
//!
//! Whitespace follows Kotlin rather than Rust:
//!  `trim` uses Java's `isWhitespace` or `isSpaceChar` (see
//! `is_kotlin_whitespace`),
//!  and query splitting uses the ASCII set that the Java regex `\s` matches.

/// Adds the Unicode normalization iterators to strings.
use unicode_normalization::UnicodeNormalization;

/// Tests whether a character is a non-spacing mark.
use crate::non_spacing_mark::is_non_spacing_mark;

/// What:
///      `const WORD_SEPARATORS: [char; 7]`.
///  The characters after which a new word begins inside a normalized name.
/// Why:
///       A term may start a name or start a word after one of these,
///  but never in the middle of a word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WORD_SEPARATORS = new Set([" ", "-", "_", ".", "(", "[", "/"]);
/// ```
pub(crate) const WORD_SEPARATORS: [char; 7] = [' ', '-', '_', '.', '(', '[', '/'];

/// What:
///      `pub const TIER_WHOLE_NAME: i32 = 0`.
///  The relevance tier of a name equal to the whole query.
/// Why:
///       An exact name is the strongest answer and sorts first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TIER_WHOLE_NAME = 0;
/// ```
pub const TIER_WHOLE_NAME: i32 = 0;

/// What:
///      `pub const TIER_NAME_PREFIX: i32 = 1`.
///  The tier of a name that starts with the first query term while every term matches.
/// Why:
///       A prefix ranks below an exact name but above a match that only starts a later word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TIER_NAME_PREFIX = 1;
/// ```
pub const TIER_NAME_PREFIX: i32 = 1;

/// What:
///      `pub const TIER_WORD_START: i32 = 2`.
///  The weakest tier,
///  where every term starts some word of the name.
/// Why:
///       No term-first prefix rule applies,
///  so this is the loosest match that still counts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TIER_WORD_START = 2;
/// ```
pub const TIER_WORD_START: i32 = 2;

/// What:
///      `pub struct SearchHighlight { pub start: i32, pub end: i32 }`.
///  One range of the original text to emphasize.
/// Why:
///       The UI paints the range in the original text,
///  so the offsets refer to that text,
///  not the normalized form.
///  Signed fields match Slint's `int`,
///  and let the page model reject a negative start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SearchHighlight = { start: number; end: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchHighlight {
    /// Index of the first highlighted character in the original text.
    pub start: i32,
    /// Index just past the last highlighted character in the original text.
    pub end: i32,
}

/// What:
///      `struct NormalizedText { chars, origin_starts, origin_ends }`.
///  Normalized characters with the original
///  start and end of each one.
/// Why:
///       NFKD can change the length of a text,
///  so each normalized character must map back to its source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NormalizedText = { text: string; originStarts: number[]; originEnds: number[] };
/// ```
struct NormalizedText {
    /// Normalized characters,
    ///  concatenated in the order of the original text.
    chars: Vec<char>,
    /// Original start offset of the character that produced each normalized character.
    origin_starts: Vec<usize>,
    /// Original end offset of each normalized character,
    ///  extended over the combining marks that followed it.
    origin_ends: Vec<usize>,
}

/// What:
///      `pub fn is_kotlin_whitespace(character: char) -> bool`.
///  The whitespace set of Kotlin's `Char.isWhitespace`.
/// Why:
///       Kotlin's `trim` and `isNotBlank` use Java's `isWhitespace` or `isSpaceChar`.
///  Rust's `White_Space` property
///  includes U+0085,
///  which Java excludes,
///  and omits U+001C to U+001F,
///  which Java includes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isKotlinWhitespace = (c: string) => /[\s\u001C-\u001F]/u.test(c) && c !== "\u0085";
/// ```
pub fn is_kotlin_whitespace(character: char) -> bool {
    return (character.is_whitespace() && character != '\u{85}')
        || ('\u{1C}'..='\u{1F}').contains(&character);
}

/// What:
///      `fn trim_kotlin_whitespace(text: &str) -> &str`.
///  Trims the characters that `is_kotlin_whitespace` accepts.
/// Why:
///       Names are trimmed before tiering,
///  and the no-match query is trimmed before it is shown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trimKotlinWhitespace = (s: string) => s.replace(/^[\s\u001C-\u001F]+|[\s\u001C-\u001F]+$/gu, "");
/// ```
pub fn trim_kotlin_whitespace(text: &str) -> &str {
    return text.trim_matches(is_kotlin_whitespace);
}

/// What:
///      `fn is_regex_whitespace(character: char) -> bool`.
///  The ASCII set that Java's regex `\s` matches.
/// Why:
///       Query terms split on this set only,
///  so a non-ASCII space inside a term does not start a new term.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isRegexWhitespace = (c: string) => /[ \t\n\v\f\r]/.test(c);
/// ```
fn is_regex_whitespace(character: char) -> bool {
    return matches!(character, ' ' | '\t' | '\n' | '\u{0B}' | '\u{0C}' | '\r');
}

/// What:
///      `fn normalize_character(character: char) -> String`.
///  Normalizes one character on its own.
/// Why:
///       Normalizing one character at a time ties every output character to one original range,
///  and it avoids
///  context-dependent lowercasing such as the final-sigma rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const normalizeCharacter = (c: string) => c.normalize("NFKD").replace(/\p{Mn}/gu, "").toLowerCase();
/// ```
fn normalize_character(character: char) -> String {
    let single: String = character.to_string();
    return single
        .nfkd()
        .filter(|part| return !is_non_spacing_mark(*part))
        .flat_map(|part| return part.to_lowercase())
        .collect();
}

/// What:
///      `fn normalize_with_origins(original: &str) -> NormalizedText`.
///  Normalizes a text character by character
///  and records where every normalized character came from.
/// Why:
///       Matching runs on the normalized text,
///  while highlights must land on the original text's offsets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function normalizeWithOrigins(original: string): NormalizedText { /* per character, with offsets */ }
/// ```
fn normalize_with_origins(original: &str) -> NormalizedText {
    let mut chars: Vec<char> = Vec::new();
    let mut origin_starts: Vec<usize> = Vec::new();
    let mut origin_ends: Vec<usize> = Vec::new();
    for (index, character) in original.chars().enumerate() {
        let next: usize = index + 1;
        let piece: String = normalize_character(character);
        if piece.is_empty() {
            // A lone combining mark produces nothing, so it extends the range of the character before it.
            if let Some(last_end) = origin_ends.last_mut() {
                *last_end = next;
            }
            continue;
        }
        for piece_character in piece.chars() {
            chars.push(piece_character);
            origin_starts.push(index);
            origin_ends.push(next);
        }
    }
    return NormalizedText {
        chars,
        origin_starts,
        origin_ends,
    };
}

/// What:
///      `pub fn normalize_for_search(text: &str) -> String`.
///  The matching form of a name or query.
/// Why:
///       Case and accents must not change whether a name matches,
///  so both sides use this one normalization.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const normalizeForSearch = (text: string) => normalizeWithOrigins(text).text;
/// ```
pub fn normalize_for_search(text: &str) -> String {
    return normalize_with_origins(text).chars.into_iter().collect();
}

/// What:
///      `pub fn search_terms(query: &str) -> Vec<String>`.
///  Splits a query into its normalized terms.
/// Why:
///       Every term must match a name (AND),
///  so the query is split on ASCII whitespace runs after normalization.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const searchTerms = (query: string) => normalizeForSearch(query).split(/\s+/).filter(Boolean);
/// ```
pub fn search_terms(query: &str) -> Vec<String> {
    return normalize_for_search(query)
        .split(is_regex_whitespace)
        .filter(|term| return !term.is_empty())
        .map(|term| return term.to_string())
        .collect();
}

/// What:
///      `fn is_word_start(normalized: &[char], index: usize) -> bool`.
///  Tests whether a term may begin at an index.
/// Why:
///       A term matches only at the start of the text or right after a word separator,
///  so `cam` never matches
///  inside `Scamper`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isWordStart = (s: string, i: number) => i === 0 || WORD_SEPARATORS.has(s[i - 1]);
/// ```
fn is_word_start(normalized: &[char], index: usize) -> bool {
    return index == 0 || WORD_SEPARATORS.contains(&normalized[index - 1]);
}

/// What:
///      `fn word_start_occurrences(normalized: &[char], term: &[char]) -> Vec<usize>`.
///  Lists every index where the
///  term begins a word.
///  An empty term has no occurrences.
/// Why:
///       Both matching and highlighting need every occurrence,
///  not only the first one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wordStartOccurrences(s: string, term: string): number[] { /* every indexOf hit that isWordStart */ }
/// ```
fn word_start_occurrences(normalized: &[char], term: &[char]) -> Vec<usize> {
    let mut occurrences: Vec<usize> = Vec::new();
    if term.is_empty() || term.len() > normalized.len() {
        return occurrences;
    }
    for start in 0..=(normalized.len() - term.len()) {
        if normalized[start..start + term.len()] == *term && is_word_start(normalized, start) {
            occurrences.push(start);
        }
    }
    return occurrences;
}

/// What:
///      `fn chars_of(text: &str) -> Vec<char>`.
///  The characters of a text,
///  which every offset counts.
/// Why:
///       Terms and names are compared character by character,
///  so their offsets stay in one unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const charsOf = (s: string) => [...s];
/// ```
fn chars_of(text: &str) -> Vec<char> {
    return text.chars().collect();
}

/// What:
///      `fn to_offset(index: usize) -> i32`.
///  Converts a character index into a highlight offset.
/// Why:
///       Highlight offsets are `i32` for Slint,
///  and a text cannot hold more than `i32::MAX` characters in this app.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const toOffset = (index: number) => index;
/// ```
pub fn to_offset(index: usize) -> i32 {
    return i32::try_from(index).expect("text offsets fit in an i32");
}

/// What:
///      `fn merge_overlapping(ranges: Vec<SearchHighlight>) -> Vec<SearchHighlight>`.
///  Sorts ranges by start and
///  merges every pair that overlaps or touches.
/// Why:
///       Two query terms can cover the same letters,
///  and painting them twice would show a doubled range.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeOverlapping(ranges: Range[]): Range[] { /* sort by start, then merge */ }
/// ```
fn merge_overlapping(ranges: Vec<SearchHighlight>) -> Vec<SearchHighlight> {
    let mut sorted: Vec<SearchHighlight> = ranges;
    sorted.sort_by(|left, right| return (left.start, left.end).cmp(&(right.start, right.end)));
    let mut merged: Vec<SearchHighlight> = Vec::new();
    for range in sorted {
        let touches_last: bool = merged
            .last()
            .is_some_and(|last| return range.start <= last.end);
        if touches_last {
            if let Some(last) = merged.last_mut() {
                last.end = last.end.max(range.end);
            }
        } else {
            merged.push(range);
        }
    }
    return merged;
}

/// What:
///      `pub fn highlight_ranges(text: &str, query: &str) -> Vec<SearchHighlight>`.
///  Finds every query term occurrence
///  that starts a word of the text,
///  in the text's own character offsets.
/// Why:
///       Each visible match is emphasized in place,
///  so the ranges must map onto the original characters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function highlightRanges(text: string, query: string): Range[] { /* word-start hits, mapped back */ }
/// ```
pub fn highlight_ranges(text: &str, query: &str) -> Vec<SearchHighlight> {
    return highlight_ranges_for_terms(text, &search_terms(query));
}

/// What:
///      `pub fn highlight_ranges_for_terms(text: &str, terms: &[String]) -> Vec<SearchHighlight>`.
///  The highlight ranges of a text for terms that are already normalized.
/// Why:
///       The Search index highlights up to five hundred rows per query,
///  and splitting the query once instead of once per row keeps that loop to the highlight work itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function highlightRangesForTerms(text: string, terms: string[]): Range[] { /* word-start hits, mapped back */ }
/// ```
pub fn highlight_ranges_for_terms(text: &str, terms: &[String]) -> Vec<SearchHighlight> {
    if terms.is_empty() {
        return Vec::new();
    }
    let normalized: NormalizedText = normalize_with_origins(text);
    let mut ranges: Vec<SearchHighlight> = Vec::new();
    for term in terms {
        let term_chars: Vec<char> = chars_of(term);
        for occurrence in word_start_occurrences(&normalized.chars, &term_chars) {
            let last: usize = occurrence + term_chars.len() - 1;
            ranges.push(SearchHighlight {
                start: to_offset(normalized.origin_starts[occurrence]),
                end: to_offset(normalized.origin_ends[last]),
            });
        }
    }
    return merge_overlapping(ranges);
}

/// What:
///      `pub fn match_tier(normalized_name: &str, terms: &[String]) -> Option<i32>`.
///  How closely a name matches,
///  or `None` when it does not match.
/// Why:
///       Results sort by tier,
///  so an exact name precedes a prefix,
///  and a prefix precedes a later-word match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function matchTier(name: string, terms: string[]): number | null { /* 0, 1, 2, or null */ }
/// ```
pub fn match_tier(normalized_name: &str, terms: &[String]) -> Option<i32> {
    if terms.is_empty() {
        return None;
    }
    let name: &str = trim_kotlin_whitespace(normalized_name);
    let name_chars: Vec<char> = chars_of(name);
    let every_term_matches: bool = terms
        .iter()
        .all(|term| return !word_start_occurrences(&name_chars, &chars_of(term)).is_empty());
    if !every_term_matches {
        return None;
    }
    if name == terms.join(" ") {
        return Some(TIER_WHOLE_NAME);
    }
    if terms
        .first()
        .is_some_and(|first_term| return name.starts_with(first_term.as_str()))
    {
        return Some(TIER_NAME_PREFIX);
    }
    return Some(TIER_WORD_START);
}

/// Unit tests in `search_matcher_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "search_matcher_tests.rs"]
mod tests;
