//! The unit tests of the selector syntax,
//! ported from the TypeScript tool's `selectors.unit.test.ts`.

use crate::selectors::{ParsedSelector, is_excluded, matches_pattern, parse_selector, prune_selectors};

use std::collections::BTreeSet;

fn texts(items: &[&str]) -> Vec<String> {
    return items.iter().map(|item| { return item.to_string(); }).collect();
}

#[test]
fn a_bare_name_names_every_version() {
    assert_eq!(parse_selector("ring"), ParsedSelector { pattern: "ring".to_string(), versions: Vec::new() });
}

#[test]
fn versions_after_the_at_sign_split_on_double_bars_and_are_trimmed() {
    let parsed: ParsedSelector = parse_selector("zbus@5.19.0 || 5.20.1");

    assert_eq!(parsed.pattern, "zbus");
    assert_eq!(parsed.versions, texts(&["5.19.0", "5.20.1"]));
}

#[test]
fn a_pattern_without_a_star_must_equal_the_name() {
    assert!(matches_pattern("serde", "serde"));
    assert!(!matches_pattern("serde", "serde_json"));
}

#[test]
fn stars_match_any_run_including_none() {
    assert!(matches_pattern("windows*", "windows"));
    assert!(matches_pattern("windows*", "windows_x86_64"));
    assert!(matches_pattern("*-sys", "libc-sys"));
    assert!(matches_pattern("a*b*c", "a--b--c"));
    assert!(!matches_pattern("a*b*c", "a--c"));
    assert!(!matches_pattern("ab*ba", "aba"));
}

#[test]
fn a_selector_with_versions_exempts_only_those_versions() {
    let selectors: Vec<String> = texts(&["zbus@5.19.0"]);

    assert!(is_excluded(&selectors, "zbus", "5.19.0"));
    assert!(!is_excluded(&selectors, "zbus", "5.19.1"));
    assert!(!is_excluded(&selectors, "other", "5.19.0"));
}

#[test]
fn a_bare_selector_exempts_every_version_of_the_matching_names() {
    assert!(is_excluded(&texts(&["ring"]), "ring", "0.17.8"));
    assert!(is_excluded(&texts(&["wayland-*"]), "wayland-server", "0.31.14"));
}

#[test]
fn pruning_keeps_wildcards_locked_names_and_only_the_locked_versions() {
    let locked: BTreeSet<String> = ["ring@0.17.8", "zbus@5.19.0"].iter().map(|item| { return item.to_string(); }).collect();
    let selectors: Vec<String> = texts(&["wayland-*", "ring", "gone", "zbus@5.19.0 || 5.18.0", "old@1.0.0"]);

    let kept: Vec<String> = prune_selectors(&selectors, &locked);

    assert_eq!(kept, texts(&["wayland-*", "ring", "zbus@5.19.0"]));
}
