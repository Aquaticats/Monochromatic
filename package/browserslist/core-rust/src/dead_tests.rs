//! What:
//!  Tests for the `dead` expansion and the `not dead` subtraction,
//! on the committed revision and on synthetic data.
//! Why:
//!  The real selection contains no dead browser,
//! so the subtraction path needs a synthetic vector
//!  (produced by the real npm package,
//! per the spec) plus a positive control that must differ without it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('dead', () => { /* expansion, subtraction vector, alias and fallback paths */ });
//! ```

use crate::caniuse::{Agent, AgentTable};
use crate::data::BrowserData;
use crate::dead::{dead_set, subtract_dead};
use crate::error::BrowserslistError;
use crate::fixture::fixture_files;

/// Build an agent whose literal versions are all released:
/// three empty reserved slots follow them.
fn agent(name: &str, versions: &[&str]) -> Agent {
    let mut slots: Vec<Option<String>> = versions.iter().map(|version| return Some((*version).to_string())).collect();
    slots.extend([None, None, None]);
    return Agent { name: name.to_string(), slots };
}

/// Build owned strings from literals.
fn owned(items: &[&str]) -> Vec<String> {
    return items.iter().map(|item| return (*item).to_string()).collect();
}

/// Load the committed revision.
fn committed() -> BrowserData {
    return BrowserData::from_files(fixture_files("153.6.0esr")).expect("fixture assembles");
}

/// The expansion on the real revision is the 19 strings the spec records.
#[test]
fn expansion_matches_the_spec() {
    let mut actual = dead_set(&committed().agents).expect("expands");
    actual.sort();
    let mut expected = owned(&[
        "baidu 13.52", "bb 10", "bb 7", "ie 11", "ie 10", "ie 9", "ie 8", "ie 7", "ie 6", "ie 5.5", "ie_mob 11",
        "ie_mob 10", "op_mob 12.1", "op_mob 12", "op_mob 11.5", "op_mob 11.1", "op_mob 11", "op_mob 10", "samsung 4",
    ]);
    expected.sort();
    assert_eq!(actual, expected);
}

/// The spec's vector:
/// 14 strings before `not dead`,
/// the three non-dead ones after.
#[test]
fn subtraction_vector_from_the_spec() {
    let before = owned(&[
        "baidu 13.52", "bb 10", "bb 7", "chrome 156", "ie 11", "ie 10", "ie 5.5", "ie_mob 11", "ie_mob 10",
        "op_mob 80", "op_mob 12.1", "op_mob 10", "samsung 30", "samsung 4",
    ]);
    let after = subtract_dead(&before, &committed().agents).expect("subtracts");
    assert_eq!(after, owned(&["chrome 156", "op_mob 80", "samsung 30"]));
}

/// Positive control:
/// without the subtraction the vector keeps all 14,
/// so a harness that skipped it would fail.
#[test]
fn control_fails_when_not_dead_is_skipped() {
    let before = owned(&["chrome 156", "ie 11", "op_mob 80", "samsung 30", "samsung 4"]);
    let after = subtract_dead(&before, &committed().agents).expect("subtracts");
    // An implementation that ignored `not dead` would return `before` unchanged.
    assert_ne!(after, before);
    assert_eq!(after, owned(&["chrome 156", "op_mob 80", "samsung 30"]));
}

/// Nothing outside the dead set is removed,
/// and the order of survivors is kept.
#[test]
fn survivors_keep_their_order() {
    let before = owned(&["samsung 30", "chrome 156", "ie 11", "android 4.4", "firefox 153"]);
    let after = subtract_dead(&before, &committed().agents).expect("subtracts");
    assert_eq!(after, owned(&["samsung 30", "chrome 156", "android 4.4", "firefox 153"]));
}

/// A missing browser fails with browserslist's `Unknown browser` wording,
/// named as the dead list spells it.
#[test]
fn missing_browser_fails() {
    let table = AgentTable::new(vec![agent("ie", &["10", "11"])]);
    let error = dead_set(&table).expect_err("baidu missing");
    assert_eq!(error, BrowserslistError::Browserslist { message: "Unknown browser Baidu".to_string() });
}

/// Build a table holding all six dead browsers so one rule at a time can be varied.
fn six_browsers(samsung: Agent) -> AgentTable {
    return AgentTable::new(vec![
        agent("baidu", &["1.5", "2-3", "x"]),
        agent("ie", &["9", "12", "11.5"]),
        agent("ie_mob", &["10"]),
        agent("bb", &["7", "10", "11"]),
        agent("op_mob", &["11-12.1", "12.1", "13", "TP"]),
        samsung,
    ]);
}

/// Ray rules compare numbers:
/// ranges by their newest member for `>=`,
/// TP never matches,
/// bounds are inclusive.
#[test]
fn ray_comparisons() {
    let table = six_browsers(agent("samsung", &["4", "5"]));
    let dead = dead_set(&table).expect("expands");
    assert!(dead.contains(&"baidu 1.5".to_string()));
    assert!(dead.contains(&"baidu 2-3".to_string()));
    assert!(!dead.contains(&"baidu x".to_string()));
    assert!(dead.contains(&"ie 9".to_string()));
    assert!(!dead.contains(&"ie 12".to_string()));
    assert!(!dead.contains(&"ie 11.5".to_string()));
    assert!(dead.contains(&"bb 10".to_string()));
    assert!(!dead.contains(&"bb 11".to_string()));
    assert!(dead.contains(&"op_mob 11-12.1".to_string()));
    assert!(dead.contains(&"op_mob 12.1".to_string()));
    assert!(!dead.contains(&"op_mob 13".to_string()));
    assert!(!dead.contains(&"op_mob TP".to_string()));
}

/// The single-version rule:
/// exact hit,
/// joined-range alias,
/// `.0` variant,
/// one-version fallback,
/// and failure.
#[test]
fn samsung_version_resolution() {
    let exact = dead_set(&six_browsers(agent("samsung", &["4", "5"]))).expect("exact");
    assert_eq!(exact.last().map(String::as_str), Some("samsung 4"));
    let aliased = dead_set(&six_browsers(agent("samsung", &["3.5-4", "5"]))).expect("alias");
    assert_eq!(aliased.last().map(String::as_str), Some("samsung 3.5-4"));
    let dotted = dead_set(&six_browsers(agent("samsung", &["4.0", "5"]))).expect("dot zero");
    assert_eq!(dotted.last().map(String::as_str), Some("samsung 4.0"));
    let only = dead_set(&six_browsers(agent("samsung", &["9"]))).expect("only version");
    assert_eq!(only.last().map(String::as_str), Some("samsung 9"));
    let error = dead_set(&six_browsers(agent("samsung", &["5", "6"]))).expect_err("unknown version");
    assert_eq!(error, BrowserslistError::Browserslist { message: "Unknown version 4 of samsung".to_string() });
}
