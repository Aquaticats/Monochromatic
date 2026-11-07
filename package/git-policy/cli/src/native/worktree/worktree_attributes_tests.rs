//! Controls for the byte-level lexer and matcher of attributes lines:
//! Git's blanks and NUL rule,
//! the byte order mark,
//! ASCII
//!  case folding only,
//! and "may apply" for every spelling the matcher does not model.

use super::*;

/// `*` and `?` stop at a slash,
/// every other byte stands for itself.
#[test]
fn a_wildcard_matches_like_a_shell_pattern_without_crossing_a_slash() {
    assert!(wildcard(b"*.dat", b"data.dat"));
    assert!(!wildcard(b"*.dat", b"data.txt"));
    assert!(wildcard(b"a?c", b"abc"));
    assert!(!wildcard(b"a?c", b"a/c"));
    assert!(!wildcard(b"*.dat", b"dir/data.dat"));
    assert!(wildcard(b"dir/*.dat", b"dir/data.dat"));
    assert!(wildcard(b"*", b""));
    assert!(!wildcard(b"", b"x"));
}

/// `?` matches one byte,
/// whatever the byte is,
/// and the two bytes of a letter outside ASCII are two bytes.
#[test]
fn a_question_mark_matches_one_byte_of_any_value() {
    assert!(wildcard(b"?.txt", &[0xFF, b'.', b't', b'x', b't']));
    assert!(wildcard(b"??.txt", "\u{130}.txt".as_bytes()));
    assert!(!wildcard(b"?.txt", "\u{130}.txt".as_bytes()));
    assert!(may_apply(b"??.txt", b"", "\u{130}.txt".as_bytes()));
}

/// Only ASCII letters fold,
/// so a letter outside ASCII never matches a different byte sequence.
#[test]
fn only_ascii_letters_fold() {
    assert!(wildcard(b"DATA.DAT", b"data.dat"));
    assert!(!wildcard("\u{130}".as_bytes(), b"i"));
}

/// A pattern without a slash matches the name at any depth below its directory,
/// one with a slash is anchored there,
/// and a
///  spelling the matcher does not model answers yes.
#[test]
fn a_pattern_applies_by_name_or_by_path_below_its_directory() {
    assert!(may_apply(b"*.dat", b"", b"deep/er/data.dat"));
    assert!(!may_apply(b"*.dat", b"", b"deep/er/data.txt"));
    assert!(may_apply(b"*.dat", b"deep", b"deep/er/data.dat"));
    assert!(!may_apply(b"*.dat", b"other", b"deep/er/data.dat"));
    assert!(may_apply(b"deep/*.dat", b"", b"deep/data.dat"));
    assert!(!may_apply(b"deep/*.dat", b"", b"other/deep/data.dat"));
    assert!(may_apply(b"/data.dat", b"", b"data.dat"));
    assert!(may_apply(b"**/data.dat", b"", b"x/data.dat"), "an unmodelled spelling answers yes");
    assert!(may_apply(b"[ab].dat", b"", b"z.dat"), "a bracket class is not modelled");
    assert!(may_apply(b"\"quoted name\"", b"", b"z.dat"));
    assert!(may_apply(b"DATA.DAT", b"", b"data.dat"), "case never makes a match less likely");
}

/// A comment,
/// a blank line and a macro definition are told apart from a pattern line.
#[test]
fn attribute_lines_are_split_into_patterns_macros_and_tokens() {
    let lines: Vec<AttributeLine> = parse_attribute_lines(b"# comment\n\n*.dat filter=x -text\n[attr]scrambled ident\n");
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], AttributeLine { subject: b"*.dat".to_vec(), is_macro: false, quoted: false, tokens: vec![b"filter=x".to_vec(), b"-text".to_vec()] });
    assert_eq!(lines[1], AttributeLine { subject: b"scrambled".to_vec(), is_macro: true, quoted: false, tokens: vec![b"ident".to_vec()] });
    assert_eq!(attribute_name(b"-filter"), b"filter");
    assert_eq!(attribute_name(b"working-tree-encoding=UTF-16"), b"working-tree-encoding");
}

/// Git's blanks are space,
/// tab,
/// carriage return and line feed:
/// a no-break space belongs to the pattern.
#[test]
fn only_the_four_blanks_separate_tokens() {
    let lines: Vec<AttributeLine> = parse_attribute_lines("a\u{a0}b ident\r\n".as_bytes());
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].subject, "a\u{a0}b".as_bytes());
    assert_eq!(lines[0].tokens, vec![b"ident".to_vec()]);
    let tabbed: Vec<AttributeLine> = parse_attribute_lines(b"*.dat\tident\r\n");
    assert_eq!(tabbed[0].tokens, vec![b"ident".to_vec()]);
}

/// A line's text ends at its first NUL,
/// as a C string does.
#[test]
fn a_nul_ends_the_text_of_a_line() {
    let lines: Vec<AttributeLine> = parse_attribute_lines(b"target ident\0junk\nother -text\n");
    assert_eq!(lines[0].tokens, vec![b"ident".to_vec()]);
    assert_eq!(lines[1].subject, b"other");
}

/// The byte order mark of the first line is skipped,
/// as `read_attr_from_file` skips it.
#[test]
fn a_byte_order_mark_is_not_part_of_the_first_pattern() {
    let lines: Vec<AttributeLine> = parse_attribute_lines(b"\xEF\xBB\xBF*.dat ident\n*.rs ident\n");
    assert_eq!(lines[0].subject, b"*.dat");
    assert_eq!(lines[1].subject, b"*.rs");
}

/// A quoted line is flagged and is never a macro definition,
/// whatever its text holds:
/// the caller refuses on the quote.
#[test]
fn a_quoted_line_is_flagged_and_never_a_macro() {
    let lines: Vec<AttributeLine> = parse_attribute_lines(b"\"target\" ident note=[attr]x\n\"[attr]m\" ident\n\"target\"ident\n");
    assert_eq!(lines.len(), 3);
    assert!(lines.iter().all(|line: &AttributeLine| { return line.quoted && !line.is_macro; }));
}

/// How many bytes the pattern of the work-budget control has.
const LONG_PATTERN_BYTES: usize = 100_000;

/// How many bytes the path of the work-budget control has.
const LONG_PATH_BYTES: usize = 4_000;

/// A pattern and a path whose comparison would take more work than the budget say the pattern may apply,
/// at once.
#[test]
fn work_past_the_budget_answers_may_apply() {
    let mut pattern: Vec<u8> = b"a/".to_vec();
    pattern.extend(std::iter::repeat_n(b'x', LONG_PATTERN_BYTES));
    let path: Vec<u8> = std::iter::repeat_n(b'y', LONG_PATH_BYTES).collect();
    assert!(may_apply(pattern.as_slice(), b"", path.as_slice()), "past the budget the answer is yes, never a no that was not worked out");
    assert!(!may_apply(b"a/x", b"", b"b/x"), "within the budget the answer is still exact");
}
