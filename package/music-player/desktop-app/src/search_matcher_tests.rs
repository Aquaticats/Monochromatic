// What:     Unit tests for `search_matcher.rs`, pulled in by
//           `#[cfg(test)] #[path = "search_matcher_tests.rs"] mod tests;` at the bottom of
//           `search_matcher.rs`. Mirrors every `@Test` of `SearchMatcherTest.kt`, with the same inputs and
//           expected values. Highlight offsets are character offsets, so the one astral case expects (2, 5)
//           where the Kotlin expects UTF-16 offsets (3, 6). The whitespace controls at the end have no Kotlin case.
// Why:      Keep the tests beside the code without inflating `search_matcher.rs`. Reaches the module items via
//           `use super::*` because this file is the `tests` CHILD of search_matcher.
use super::*;

// What:     `fn terms(words: &[&str]) -> Vec<String>` test helper: owns the query terms a case passes in.
// Why:      `match_tier` takes owned terms, and the cases are written with string literals.
fn terms(words: &[&str]) -> Vec<String> {
    return words.iter().map(|word| return word.to_string()).collect();
}

// What:     `fn tier(name: &str, words: &[&str]) -> Option<i32>` test helper: tiers a name against literal terms.
// Why:      Most cases tier one name, and this keeps each assertion on one line.
fn tier(name: &str, words: &[&str]) -> Option<i32> {
    return match_tier(name, &terms(words));
}

// What:     `fn span(start: i32, end: i32) -> SearchHighlight` test helper: builds one expected highlight range.
// Why:      Highlight cases compare lists of ranges, and the literal struct form is long.
fn span(start: i32, end: i32) -> SearchHighlight {
    return SearchHighlight { start, end };
}

// What:     `#[test]` marks the next function as a test case.
// Why:      D59 requires `cam` to match `Cam`, and accent-insensitive matching follows from the same rule.
#[test]
fn normalizes_accents_and_case() {
    assert_eq!(normalize_for_search("Café Ö"), "cafe o");
    assert_eq!(normalize_for_search("ÀÉÎ"), "aei");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A file name may store `ö` as one code point or as `o` plus a combining mark, and both must match.
#[test]
fn decomposed_and_composed_accents_normalize_equally() {
    assert_eq!(
        normalize_for_search("Cö"),
        normalize_for_search("Co\u{0308}")
    );
    assert_eq!(normalize_for_search("Co\u{0308}"), "co");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Highlight offsets map onto the original text, so the normalized form must not drop characters.
#[test]
fn normalization_keeps_spaces_and_does_not_trim() {
    assert_eq!(normalize_for_search("  Cam "), "  cam ");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      NFKD folds compatibility forms, so a query typed in another width still finds the name.
#[test]
fn normalizes_compatibility_forms() {
    assert_eq!(normalize_for_search("Ａｂｃ　ｶﾀ"), "abc カタ");
    assert_eq!(normalize_for_search("A　B"), "a b");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The rule removes `Mn` marks, so `が` matches `か`. That consequence is the accepted cost of the rule.
#[test]
fn removes_kana_voicing_marks() {
    assert_eq!(normalize_for_search("が"), "か");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Only non-spacing marks are removed, because a spacing mark can be part of an Indic letter.
#[test]
fn keeps_spacing_marks() {
    assert_eq!(normalize_for_search("\u{0915}\u{093F}"), "\u{0915}\u{093F}");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Lowercasing one character at a time keeps every character tied to one original range, and avoids the final sigma rule.
#[test]
fn lowercases_each_code_point_separately() {
    assert_eq!(normalize_for_search("ΟΔΟΣ"), "οδοσ");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Every whitespace-separated term must match a name, so repeated spaces must not create empty terms.
#[test]
fn search_terms_splits_on_whitespace_runs() {
    assert_eq!(search_terms("  Live   CAM "), vec!["live", "cam"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A blank query must produce no terms, so it cannot match anything.
#[test]
fn blank_query_has_no_terms() {
    assert_eq!(search_terms(""), Vec::<String>::new());
    assert_eq!(search_terms("   "), Vec::<String>::new());
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Normalization removes the mark, leaving nothing to match, so the query is treated as empty.
#[test]
fn combining_only_query_has_no_terms() {
    assert_eq!(search_terms("\u{0301}"), Vec::<String>::new());
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An exact name is the strongest match, and it must sort before a prefix (D61's accepted `Cam`).
#[test]
fn whole_name_is_tier_zero() {
    assert_eq!(tier("cam", &["cam"]), Some(0));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      D61 places the `Camellia` prefix folder after the exact `Cam` track.
#[test]
fn prefix_name_is_tier_one() {
    assert_eq!(tier("camellia", &["cam"]), Some(1));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      D60's accepted example `Live at Camellia` matches `cam` at a later word start.
#[test]
fn later_word_is_tier_two() {
    assert_eq!(tier("live at camellia", &["cam"]), Some(2));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      `cam` must not match `Scamper` or `Dreamcam`, because neither occurrence starts a word.
#[test]
fn no_match_inside_a_word() {
    assert_eq!(tier("scamper", &["cam"]), None);
    assert_eq!(tier("dreamcam", &["cam"]), None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The word-start rule covers a space, `-`, `_`, `.`, `(`, `[`, and `/`, and each must count.
#[test]
fn word_starts_after_each_separator() {
    assert_eq!(tier("a cam", &["cam"]), Some(2));
    assert_eq!(tier("a-cam", &["cam"]), Some(2));
    assert_eq!(tier("a_cam", &["cam"]), Some(2));
    assert_eq!(tier("a.cam", &["cam"]), Some(2));
    assert_eq!(tier("a(cam", &["cam"]), Some(2));
    assert_eq!(tier("a[cam", &["cam"]), Some(2));
    assert_eq!(tier("a/cam", &["cam"]), Some(2));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A multi-term query narrows results, so one missing term rejects the name.
#[test]
fn every_term_must_match() {
    assert_eq!(tier("cam only", &["live", "cam"]), None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      `live cam` must find `Camellia Live`, because each term starts some word of the name.
#[test]
fn terms_may_match_in_any_word_order() {
    assert_eq!(tier("camellia live", &["live", "cam"]), Some(2));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      With several terms, tier one needs the name to start with the first term while every term still matches.
#[test]
fn first_term_at_start_is_tier_one() {
    assert_eq!(tier("live camellia", &["live", "cam"]), Some(1));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An AND over no terms would otherwise be vacuously true and match the whole library.
#[test]
fn empty_terms_never_match() {
    assert_eq!(match_tier("cam", &[]), None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A stray space in a file name must not turn an exact or prefix match into a later-word match.
#[test]
fn surrounding_spaces_do_not_demote_tier() {
    assert_eq!(tier(" cam ", &["cam"]), Some(0));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Highlight offsets must address the original text, here a composed `ö` after `C`.
#[test]
fn highlights_accented_word_in_original_offsets() {
    assert_eq!(highlight_ranges("Cö shu Nie", "co"), vec![span(0, 2)]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The mark belongs to the letter, so the range must not split the base from its diaeresis.
#[test]
fn highlight_includes_trailing_combining_mark() {
    assert_eq!(highlight_ranges("Co\u{0308} shu", "co"), vec![span(0, 3)]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      D59 emphasizes every visible match, so a repeated word start must produce a second range.
#[test]
fn highlights_every_word_start() {
    assert_eq!(
        highlight_ranges("Cam Live Cam", "cam"),
        vec![span(0, 3), span(9, 12)]
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Highlights and matches must agree, so only the word-start `Cam` is emphasized.
#[test]
fn highlight_skips_mid_word_occurrences() {
    assert_eq!(highlight_ranges("Scamper Cam", "cam"), vec![span(8, 11)]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Japanese names follow the same word-start rule, so `ラジオ` after a space must be emphasized.
#[test]
fn highlights_japanese_word_start_after_space() {
    assert_eq!(highlight_ranges("東京 ラジオ", "ラジオ"), vec![span(3, 6)]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Unspaced Japanese has no word boundaries in this rule, so a middle fragment must not be highlighted.
#[test]
fn unspaced_japanese_has_no_later_word_match() {
    assert_eq!(
        highlight_ranges("あいうえお", "いう"),
        Vec::<SearchHighlight>::new()
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The range counts characters, so an astral character before the match shifts the offset by one.
//           The Kotlin test expects (3, 6) in UTF-16 units, and the same match is (2, 5) in characters.
#[test]
fn highlight_counts_surrogate_pairs_as_one_code_point() {
    assert_eq!(highlight_ranges("𠀋 Cam", "cam"), vec![span(2, 5)]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Painting the same letters twice would look wrong, so overlapping ranges become one.
#[test]
fn overlapping_terms_merge_into_one_range() {
    assert_eq!(highlight_ranges("Camellia", "cam ca"), vec![span(0, 3)]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A blank query or an empty name must produce no ranges, so the UI draws no emphasis.
#[test]
fn blank_query_and_empty_text_highlight_nothing() {
    assert_eq!(
        highlight_ranges("Cam", "   "),
        Vec::<SearchHighlight>::new()
    );
    assert_eq!(highlight_ranges("", "cam"), Vec::<SearchHighlight>::new());
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Query splitting uses the ASCII set of Java's `\s`, so a vertical tab splits a query.
#[test]
fn query_splits_on_vertical_tab() {
    assert_eq!(search_terms("a\u{0B}b"), vec!["a", "b"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      U+001C is not in Java's `\s`, so `a\u{1C}b` stays one term even though it is Kotlin whitespace.
#[test]
fn query_does_not_split_on_information_separator() {
    assert_eq!(search_terms("a\u{1C}b"), vec!["a\u{1C}b"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Names are trimmed with Kotlin's whitespace set, so an information separator around a name does not demote it.
#[test]
fn surrounding_information_separators_are_trimmed() {
    assert_eq!(tier("\u{1C}cam\u{1F}", &["cam"]), Some(0));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      U+0085 is Rust whitespace but not Kotlin whitespace, so it must not be trimmed from a name.
#[test]
fn next_line_control_is_not_kotlin_whitespace() {
    assert!(!is_kotlin_whitespace('\u{85}'));
    assert!(is_kotlin_whitespace('\u{A0}'));
    assert!(is_kotlin_whitespace('\u{1C}'));
    assert!(!is_kotlin_whitespace('a'));
}
