// Grammar controls for the rules list: the single-path meaning, the optional marker, and every refusal.

use super::{parse_rules_list, RulesEntry, RulesListError};
use std::path::PathBuf;

// Builds the expected entry for a path and its optional flag.
fn entry(path: &str, optional: bool) -> RulesEntry {
    return RulesEntry { path: PathBuf::from(path), optional }
}

#[test]
fn a_value_without_comma_or_marker_is_one_required_entry() {
    for value in ["rules.txt", "dir/rules.txt", "/abs/rules.txt", "./rules.txt", "-leading-hyphen", "has space.txt", "C:/rules.txt", "a\\b.txt", "100%.txt"] {
        assert_eq!(parse_rules_list(value), Ok(vec![entry(value, false)]), "{value}");
    }
}

#[test]
fn a_comma_separates_entries_in_order() {
    assert_eq!(
        parse_rules_list("forbidden-strings.append.txt,forbidden-strings.append.local.txt"),
        Ok(vec![
            entry("forbidden-strings.append.txt", false),
            entry("forbidden-strings.append.local.txt", false),
        ]),
    );
}

#[test]
fn a_trailing_question_mark_marks_only_its_own_entry() {
    assert_eq!(
        parse_rules_list("a.txt,b.txt?,c.txt"),
        Ok(vec![entry("a.txt", false), entry("b.txt", true), entry("c.txt", false)]),
    );
    assert_eq!(parse_rules_list("only?"), Ok(vec![entry("only", true)]));
}

#[test]
fn a_question_mark_inside_a_path_is_part_of_the_path() {
    assert_eq!(parse_rules_list("a?b.txt"), Ok(vec![entry("a?b.txt", false)]));
    assert_eq!(parse_rules_list("a?b.txt?"), Ok(vec![entry("a?b.txt", true)]));
}

#[test]
fn exactly_one_trailing_marker_is_consumed_so_a_path_ending_in_a_question_mark_is_named_as_optional() {
    assert_eq!(parse_rules_list("name??"), Ok(vec![entry("name?", true)]));
    assert_eq!(parse_rules_list("name???"), Ok(vec![entry("name??", true)]));
    assert_eq!(parse_rules_list("??"), Ok(vec![entry("?", true)]));
}

#[test]
fn a_path_ending_in_a_question_mark_cannot_be_named_as_required() {
    // The only spelling that ends in one '?' reads as an optional marker, never as a path character.
    assert_eq!(parse_rules_list("name?"), Ok(vec![entry("name", true)]));
}

#[test]
fn an_empty_value_is_refused() {
    assert_eq!(parse_rules_list(""), Err(RulesListError::Empty));
}

#[test]
fn every_empty_entry_is_refused_with_its_position() {
    for (value, position) in [
        (",", 1),
        ("a,", 2),
        (",a", 1),
        ("a,,b", 2),
        ("a,b,", 3),
        ("?", 1),
        ("a,?", 2),
        ("?,a", 1),
        (",?", 1),
        ("a,,", 2),
    ] {
        assert_eq!(parse_rules_list(value), Err(RulesListError::EmptyEntry { position }), "{value:?}");
    }
}

#[test]
fn whitespace_is_part_of_the_path_and_is_never_trimmed() {
    assert_eq!(
        parse_rules_list("a.txt, b.txt ,c.txt "),
        Ok(vec![entry("a.txt", false), entry(" b.txt ", false), entry("c.txt ", false)]),
    );
    assert_eq!(parse_rules_list(" "), Ok(vec![entry(" ", false)]));
    assert_eq!(parse_rules_list(" ?"), Ok(vec![entry(" ", true)]));
}

#[test]
fn a_repeated_path_is_refused_even_when_the_markers_differ() {
    assert_eq!(
        parse_rules_list("a.txt,b.txt,a.txt"),
        Err(RulesListError::DuplicatePath { first: 1, position: 3 }),
    );
    assert_eq!(
        parse_rules_list("a.txt,a.txt?"),
        Err(RulesListError::DuplicatePath { first: 1, position: 2 }),
    );
    assert_eq!(parse_rules_list("a.txt,a.txt??"), Ok(vec![entry("a.txt", false), entry("a.txt?", true)]));
}

#[test]
fn separator_lookalikes_and_shell_syntax_are_ordinary_path_characters() {
    // Delimiters other than ',' and a trailing '?' carry no meaning, so they stay in the path untouched.
    for value in ["a;b.txt", "a:b.txt", "a|b.txt", "a b|c", "$HOME/x", "`x`", "a\nb.txt", "a\tb", "\u{ff0c}.txt", "x\u{3f}\u{3f}y"] {
        assert_eq!(parse_rules_list(value), Ok(vec![entry(value, false)]), "{value:?}");
    }
    assert_eq!(parse_rules_list("a;b,c:d?"), Ok(vec![entry("a;b", false), entry("c:d", true)]));
}

#[test]
fn the_refusals_render_a_position_and_quote_no_path() {
    let secret: &str = "SECRET-NAME";
    for value in [format!("{secret},,{secret}2"), format!("{secret},{secret}"), String::new()] {
        let error: RulesListError = parse_rules_list(value.as_str()).expect_err("refused");
        assert!(!error.to_string().contains(secret), "{error}");
    }
    assert_eq!(
        RulesListError::EmptyEntry { position: 2 }.to_string(),
        "rules list entry 2 names no file (an empty entry between commas, or a bare '?')",
    );
    assert_eq!(
        RulesListError::DuplicatePath { first: 1, position: 3 }.to_string(),
        "rules list entry 3 names the same file as entry 1",
    );
    assert_eq!(
        RulesListError::Empty.to_string(),
        "the rules list is empty; name at least one rules file",
    );
}

#[test]
fn long_lists_parse_in_one_pass() {
    let names: Vec<String> = (0..5000).map(|index| return format!("rules-{index}.txt")).collect();
    let value: String = names.join(",");
    let parsed: Vec<RulesEntry> = parse_rules_list(value.as_str()).expect("long list");
    assert_eq!(parsed.len(), 5000);
    assert_eq!(parsed[4999], entry("rules-4999.txt", false));
}
