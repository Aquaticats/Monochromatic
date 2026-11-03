//! The unit tests of the Git configuration reader.

use crate::git_config::{ConfigEntry, parse_config};

fn entry(section: &str, subsection: Option<&str>, key: &str, value: &str) -> ConfigEntry {
    return ConfigEntry { section: section.to_string(), subsection: subsection.map(str::to_string), key: key.to_string(), value: value.to_string() };
}

#[test]
fn sections_subsections_keys_and_values_are_read_with_git_case_rules() {
    let text: &str = "[Core]\n\tBare = false\n[branch \"Feature/One\"]\n\tremote = origin\n\tMerge = refs/heads/main\n";

    let parsed = parse_config(text).expect("parses");

    assert_eq!(
        parsed.entries,
        vec![
            entry("core", None, "bare", "false"),
            entry("branch", Some("Feature/One"), "remote", "origin"),
            entry("branch", Some("Feature/One"), "merge", "refs/heads/main"),
        ]
    );
    assert_eq!(parsed.last("BRANCH", Some("Feature/One"), "MERGE"), Some("refs/heads/main"));
    assert_eq!(parsed.last("branch", Some("feature/one"), "merge"), None, "subsection names keep their case");
}

#[test]
fn the_older_dotted_header_lowercases_its_subsection() {
    let parsed = parse_config("[branch.Main]\n\tremote = origin\n").expect("parses");

    assert_eq!(parsed.last("branch", Some("main"), "remote"), Some("origin"));
}

#[test]
fn a_key_may_repeat_and_the_last_value_wins_for_a_single_valued_key() {
    let parsed = parse_config("[remote \"origin\"]\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n\tfetch = ^refs/heads/skip\n[core]\n\tx = 1\n\tx = 2\n").expect("parses");

    assert_eq!(parsed.values("remote", Some("origin"), "fetch").len(), 2);
    assert_eq!(parsed.last("core", None, "x"), Some("2"));
}

#[test]
fn quotes_escapes_comments_and_continuations_are_applied() {
    let text: &str = "[a]\n\tq = \"x ; y\"  ; trailing comment\n\te = a\\tb\n\tc = one \\\ntwo\n\tbare\n# whole line\n";

    let parsed = parse_config(text).expect("parses");

    assert_eq!(parsed.last("a", None, "q"), Some("x ; y"));
    assert_eq!(parsed.last("a", None, "e"), Some("a\tb"));
    assert_eq!(parsed.last("a", None, "c"), Some("one two"));
    assert_eq!(parsed.last("a", None, "bare"), Some("true"));
}

#[test]
fn a_key_on_the_header_line_is_read() {
    let parsed = parse_config("[core] bare = true\n").expect("parses");

    assert_eq!(parsed.last("core", None, "bare"), Some("true"));
}

#[test]
fn include_and_include_if_sections_are_flagged_and_not_followed() {
    assert!(parse_config("[include]\n\tpath = x\n").expect("parses").has_include);
    assert!(parse_config("[includeIf \"gitdir:/x/\"]\n\tpath = y\n").expect("parses").has_include);
    assert!(!parse_config("[core]\n\tbare = false\n").expect("parses").has_include);
}

#[test]
fn text_that_cannot_be_read_exactly_is_an_error() {
    assert!(parse_config("key = value\n").is_err(), "a line before any header");
    assert!(parse_config("[core\n").is_err(), "an unclosed header");
    assert!(parse_config("[core]\n\tx = \"open\n").is_err(), "an unclosed quote");
    assert!(parse_config("[core]\n\tx = a\\qb\n").is_err(), "an unknown escape");
    assert!(parse_config("[core]\n\t1x = y\n").is_err(), "a key starting with a digit");
    assert!(parse_config("[branch main]\n").is_err(), "a subsection without quotes");
}
