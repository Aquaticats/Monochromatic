// Tests for the two-form file format, flag policy, and BOM stripping.

use super::parse_patterns;
use super::significant_line_pattern;
use super::strip_bom;
use crate::rule::frx::LoadError;
use forbidden_regex::RegexSet;

#[test]
fn literal_line_is_escaped() {
    // A short bare literal is escaped (the dot becomes a literal byte) and gated
    // behind word boundaries so it matches whole tokens, not substrings of longer
    // runs.
    let patterns = parse_patterns("a.b").expect("one literal");
    assert_eq!(patterns, vec!["\\ba\\.b\\b".to_string()]);
    assert_eq!(
        significant_line_pattern("a.b", 0).expect("classified literal"),
        "\\ba\\.b\\b",
    );
}

#[test]
fn regex_line_passes_body_and_drops_noop_flags() {
    // `m` and `x` are no-ops (the engine is always multiline and verbose), so the
    // body is passed through unchanged with the flags dropped.
    let patterns = parse_patterns("/foo[0-9]/mx").expect("regex with noop flags");
    assert_eq!(patterns, vec!["foo[0-9]".to_string()]);

    let plain = parse_patterns("/bar/").expect("regex no flags");
    assert_eq!(plain, vec!["bar".to_string()]);
}

#[test]
fn bad_flag_fails_closed_with_index() {
    // First rule good, second carries an unsupported `i`, so index 1 hard-errors.
    let err = parse_patterns("/good/\n/other/i").expect_err("bad flag");
    assert_eq!(err, LoadError::UnsupportedFlag { index: 1, flag: 'i' });

    // `m`/`x` before a bad letter still report the first offender.
    let mixed = parse_patterns("/p/mxs").expect_err("bad flag after noops");
    assert_eq!(mixed, LoadError::UnsupportedFlag { index: 0, flag: 's' });
}

#[test]
fn blanks_and_comments_are_skipped() {
    // The surviving literal is eight bytes, so boundary gating does not apply and
    // this stays a test of skip behavior alone.
    let patterns = parse_patterns("# comment\n\n  \nsurvivor").expect("one rule");
    assert_eq!(patterns, vec!["survivor".to_string()]);
}

#[test]
fn empty_or_comment_only_source_is_no_rules() {
    assert_eq!(parse_patterns("").expect_err("empty"), LoadError::NoRules);
    assert_eq!(
        parse_patterns("# only a comment\n\n").expect_err("comment only"),
        LoadError::NoRules,
    );
}

#[test]
fn leading_bom_is_stripped() {
    // A BOM before the first rule must not become part of it. The literal is eight
    // bytes so boundary gating does not obscure the BOM-strip assertion.
    let patterns = parse_patterns("\u{FEFF}abcdefgh").expect("bom rule");
    assert_eq!(patterns, vec!["abcdefgh".to_string()]);

    assert_eq!(strip_bom("plain"), "plain");
    assert_eq!(strip_bom("\u{FEFF}x"), "x");
    // A BOM followed only by a comment still yields no rules.
    assert_eq!(parse_patterns("\u{FEFF}# c").expect_err("bom comment"), LoadError::NoRules);
}

#[test]
fn slashed_line_with_nonflag_trailer_is_literal() {
    // Trailing `9` is not a lowercase flag run, so the whole line is a literal,
    // not the regex body `x`.
    let patterns = parse_patterns("/x/9").expect("literal");
    assert_eq!(patterns.len(), 1);
    assert_ne!(patterns[0], "x");
    let set = RegexSet::new(&patterns).expect("compile");
    assert!(set.is_match(b"/x/9"));
}

#[test]
fn strict_header_first_line_autodetects_tail_format() {
    // A strict header on the first significant line routes the whole file through
    // the tail-format parser: the single-line regex body passes through unchanged.
    let patterns = parse_patterns("==> qqq-e2e <==\n/foo[0-9]/\n").expect("tail parsed");
    assert_eq!(patterns, vec!["foo[0-9]".to_string()]);
}

#[test]
fn bom_then_header_is_tail_format() {
    // The BOM is stripped before autodetection, so a BOM directly before the first
    // header still selects the tail-format path. The literal is eight bytes, so no
    // boundary gating obscures the assertion.
    let patterns = parse_patterns("\u{FEFF}==> qqq-bom <==\nabcdefgh\n").expect("bom tail");
    assert_eq!(patterns, vec!["abcdefgh".to_string()]);
}

#[test]
fn legacy_first_line_starting_with_equals_routes_to_legacy() {
    // A first line starting with `=` is not a strict header, so the whole file
    // parses byte-for-byte as legacy: two lines yield two literal rules.
    let patterns =
        parse_patterns("=legacy-literal\nother-legacy\n").expect("legacy routed");
    assert_eq!(patterns.len(), 2);
    let set = RegexSet::new(&patterns).expect("compile legacy");
    assert!(set.is_match(b"x =legacy-literal y"));
    assert!(set.is_match(b"z other-legacy w"));
}

#[test]
fn tail_format_multiple_sections_parse_in_file_order() {
    // A multi-section tail file parses end-to-end through the public entry: the
    // first section's single-line regex and the second's bare literal both survive
    // in file order, projected to the unchanged `Vec<String>` shape.
    let patterns = parse_patterns("==> qqq-first <==\n/foo[0-9]/\n==> qqq-second <==\nabcdefgh\n")
        .expect("two tail sections");
    assert_eq!(patterns, vec!["foo[0-9]".to_string(), "abcdefgh".to_string()]);
}

#[test]
fn tail_format_duplicate_name_fails_closed_end_to_end() {
    // Name uniqueness is enforced over the whole loaded input at the public entry.
    let err = parse_patterns("==> qqq-same <==\naaaa\n==> qqq-same <==\nbbbb\n")
        .expect_err("duplicate name through public entry");
    assert_eq!(err, LoadError::DuplicateName { first_line: 1, line: 3 });
}
