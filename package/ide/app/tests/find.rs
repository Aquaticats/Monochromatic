//! The production matching function:
//!  plain literal,
//!  case-insensitive,
//!  in source character positions.

/// The same function and bounds the worker and the reference comparison use.
use ide_app::find::{
    FindRange, MAX_FIND_MATCHES, MAX_FIND_QUERY_CHARS, find_in_bounded_source, find_in_source,
    find_matches,
};
/// Tab expansion must keep a match's source positions addressable in display text.
use ide_app::text_projection::project_line;

/// Collect matches as plain pairs so expectations read as source character positions.
fn pairs(text: &str, query: &str) -> Vec<(usize, usize)> {
    // What: `expect` extracts the successful matches or fails the test with this message.
    // Why: These queries are valid; an error here is a defect, not an empty result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const matches = findMatches(text, query, MAX_FIND_MATCHES);
    // ```
    let matches = find_matches(text, query, MAX_FIND_MATCHES).expect("valid find text");
    assert!(!matches.truncated);
    // What: `Vec::new()` creates an empty growable list.
    // Why: The shared range list is copied into tuples only for comparison.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const result: Array<[number, number]> = [];
    // ```
    let mut result = Vec::new();
    for range in matches.ranges.iter() {
        result.push((range.start, range.end));
    }
    return result;
}

/// ASCII and non-ASCII letters match regardless of case;
///  matches never overlap.
#[test]
fn case_is_ignored_for_ascii_and_unicode_letters() {
    assert_eq!(
        pairs("Needle needle NEEDLE", "nEEdle"),
        [(0, 6), (7, 13), (14, 20)]
    );
    assert_eq!(pairs("École école", "ÉCOLE"), [(0, 5), (6, 11)]);
    assert_eq!(pairs("Привет привет", "ПРИВЕТ"), [(0, 6), (7, 13)]);
    assert_eq!(pairs("ς σ Σ", "Σ"), [(0, 1), (2, 3), (4, 5)]);
    assert_eq!(pairs("aaaa", "aa"), [(0, 2), (2, 4)]);
}

/// Regex punctuation in the find text matches itself and nothing else.
#[test]
fn regex_metacharacters_are_literal() {
    assert_eq!(pairs("a.c abc", "a.c"), [(0, 3)]);
    assert_eq!(pairs("x [y] (z) {1} a|b", "["), [(2, 3)]);
    assert_eq!(pairs("x [y] (z) {1} a|b", "(z)"), [(6, 9)]);
    assert_eq!(pairs("x [y] (z) {1} a|b", "a|b"), [(14, 17)]);
    assert_eq!(pairs("a.*b a*b", ".*"), [(1, 3)]);
    assert_eq!(pairs("path\\to ^start$ a+b?", "\\"), [(4, 5)]);
    assert_eq!(pairs("path\\to ^start$ a+b?", "^start$"), [(8, 15)]);
    assert_eq!(pairs("path\\to ^start$ a+b?", "a+b?"), [(16, 20)]);
    assert_eq!(pairs("\\d 7", "\\d"), [(0, 2)]);
}

/// No find text is a valid state without matches,
///  not a match at every position.
#[test]
fn empty_query_matches_nothing() {
    assert_eq!(pairs("anything", ""), []);
    assert_eq!(pairs("", ""), []);
    assert_eq!(pairs("", "needle"), []);
}

/// Positions count Unicode scalar values,
///  not UTF-8 bytes or UTF-16 units.
#[test]
fn positions_are_source_characters_for_cjk_and_astral_text() {
    assert_eq!(pairs("a 猫 b 猫", "猫"), [(2, 3), (6, 7)]);
    assert_eq!(pairs("猫猫 needle", "needle"), [(3, 9)]);
    assert_eq!(pairs("😀 needle 😀", "needle"), [(2, 8)]);
    assert_eq!(pairs("a😀b😀", "😀"), [(1, 2), (3, 4)]);
    assert_eq!(pairs("𝒳 x", "𝒳"), [(0, 1)]);
}

/// Combining marks are separate characters:
///  no accent folding and no composition.
#[test]
fn combining_marks_are_compared_as_written() {
    let decomposed = "cafe\u{301}";
    assert_eq!(pairs(decomposed, "e\u{301}"), [(3, 5)]);
    assert_eq!(pairs(decomposed, "cafe"), [(0, 4)]);
    assert_eq!(pairs(decomposed, "\u{301}"), [(4, 5)]);
    assert_eq!(pairs(decomposed, "caf\u{e9}"), []);
    assert_eq!(pairs("caf\u{e9}", "cafe"), []);
}

/// Line terminators are ordinary characters;
///  a space does not stand in for a newline or tab.
#[test]
fn whitespace_and_line_terminators_are_literal() {
    assert_eq!(pairs("one\ntwo", "one two"), []);
    assert_eq!(pairs("one\ttwo", "one two"), []);
    assert_eq!(pairs("one\ntwo", "one\ntwo"), [(0, 7)]);
    assert_eq!(pairs("a\r\nneedle\r\nneedle", "needle"), [(3, 9), (11, 17)]);
    assert_eq!(pairs("a  b a b", " b"), [(2, 4), (6, 8)]);
}

/// A match after a tab keeps its source positions while display text stands one space in for the tab.
#[test]
fn tab_expansion_preserves_match_correspondence() {
    let line = "\ta\tneedle";
    assert_eq!(pairs(line, "needle"), [(3, 9)]);
    let projection = project_line(line);
    let start = projection.source_to_byte[3];
    let end = projection.source_to_byte[9];
    assert_eq!(&projection.text[start..end], "needle");
    assert_eq!(pairs(line, "\t"), [(0, 1), (2, 3)]);
    // The tab's pixel width is the shaper's business; the projection gives it exactly one display byte.
    assert_eq!(
        projection.source_to_byte[1] - projection.source_to_byte[0],
        1
    );
}

/// The retained list stops at its limit and says so.
#[test]
fn match_count_is_bounded_and_reports_truncation() {
    let text = "ab ".repeat(5);
    let bounded = find_matches(&text, "ab", 3).expect("bounded matches");
    assert_eq!(
        bounded.ranges.len(),
        3,
        "matches beyond the limit were retained"
    );
    assert!(bounded.truncated);
    assert_eq!(bounded.ranges[2], FindRange { start: 6, end: 8 });
    let exact = find_matches(&text, "ab", 5).expect("exact matches");
    assert_eq!(exact.ranges.len(), 5);
    assert!(
        !exact.truncated,
        "an exactly full list must not claim more matches"
    );
}

/// Over-long find text is a diagnostic naming the bound,
///  not a silent empty result.
#[test]
fn query_length_is_bounded_with_a_diagnostic() {
    let longest = "a".repeat(MAX_FIND_QUERY_CHARS);
    let text = format!("x{longest}x");
    assert_eq!(pairs(&text, &longest), [(1, MAX_FIND_QUERY_CHARS + 1)]);
    let too_long = "a".repeat(MAX_FIND_QUERY_CHARS + 1);
    let error = find_matches(&text, &too_long, MAX_FIND_MATCHES)
        .expect_err("an over-long query was accepted");
    assert!(error.to_string().contains("at most 1000 characters"));
}

/// Source larger than the stated bound is refused before it is copied.
#[test]
fn source_size_is_bounded_with_a_diagnostic() {
    // What: `Rope::from_str` copies text into Helix's chunked document storage.
    // Why: The worker searches a rope snapshot, so the bound is exercised through the same entry point.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const source = Rope.from('needle needle');
    // ```
    let source = helix_core::Rope::from_str("needle needle");
    let accepted = find_in_bounded_source(&source, "needle", 13, 10).expect("source at the bound");
    assert_eq!(accepted.ranges.len(), 2);
    let error = find_in_bounded_source(&source, "needle", 12, 10)
        .expect_err("source above the bound was searched");
    assert!(error.to_string().contains("at most 12 bytes"));
    let empty = find_in_bounded_source(&source, "", 0, 10).expect("no query needs no copy");
    assert!(empty.ranges.is_empty());
    let production = find_in_source(&source, "NEEDLE").expect("production bounds");
    assert_eq!(production.ranges.len(), 2);
}

/// Many matches in a large source stay within the production match bound.
#[test]
fn large_source_respects_the_production_match_bound() {
    let source = helix_core::Rope::from_str(&"needle\n".repeat(MAX_FIND_MATCHES + 50));
    let matches = find_in_source(&source, "needle").expect("large source");
    assert_eq!(matches.ranges.len(), MAX_FIND_MATCHES);
    assert!(matches.truncated);
    let last = matches.ranges[MAX_FIND_MATCHES - 1];
    assert_eq!(last.start, (MAX_FIND_MATCHES - 1) * 7);
    assert_eq!(last.end, last.start + 6);
}
