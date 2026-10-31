//! What:
//!  Tests for query accumulation, `not dead` placement, deduplication and the final sort.
//! Why:
//!  Ordering and dedup rules are where a port most easily drifts from npm browserslist, and the first-query rule
//!  for `not` is an error path with an exact message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('resolve', () => { /* accumulate, dedupe, sort, errors */ });
//! ```

use crate::caniuse::{Agent, AgentTable};
use crate::data::BrowserData;
use crate::fixture::fixture_files;
use crate::resolve::{finalize, resolve_config_text};

/// The seven lines of the repository's `.browserslistrc`, with its leading comment.
const REPOSITORY_RC: &str = "# .browserslistrc\nlast 1 Chrome versions\nlast 1 ChromeAndroid versions\nlast 1 Android versions\nFirefox ESR\nlast 1 FirefoxAndroid versions\nlast 1 node major versions\nnot dead\n";

/// Load the committed revision.
fn committed() -> BrowserData {
    return BrowserData::from_files(fixture_files("153.6.0esr")).expect("fixture assembles");
}

/// Build owned strings from literals.
fn owned(items: &[&str]) -> Vec<String> {
    return items.iter().map(|item| (*item).to_string()).collect();
}

/// Names sort ascending bytewise (so `_` sorts before letters) and versions descending by number.
#[test]
fn finalize_sorts_names_and_versions() {
    let sorted = finalize(&owned(&[
        "node 26.1.0", "firefox 153", "chrome 9", "chrome 10", "and_chr 156", "android 4.4.3-4.4.4", "android 4.4", "android 156",
        "android 4.2-4.3", "and_ff 159", "node 26.13.0", "node 26.2.0", "node 26.2.10", "node 26.2.9",
    ]))
    .expect("sorts");
    assert_eq!(
        sorted,
        owned(&[
            "and_chr 156", "and_ff 159", "android 156", "android 4.4.3-4.4.4", "android 4.4", "android 4.2-4.3", "chrome 10",
            "chrome 9", "firefox 153", "node 26.13.0", "node 26.2.10", "node 26.2.9", "node 26.2.0", "node 26.1.0",
        ])
    );
}

/// Duplicates keep their first position, and equal sort keys keep their incoming order (stable sort).
#[test]
fn finalize_dedupes_and_is_stable() {
    assert_eq!(finalize(&owned(&["b 1", "a 1", "b 1", "a 1"])).expect("sorts"), owned(&["a 1", "b 1"]));
    assert_eq!(finalize(&owned(&["x 1.2.3.4", "x 1.2.3.5"])).expect("sorts"), owned(&["x 1.2.3.4", "x 1.2.3.5"]));
    assert_eq!(finalize(&owned(&["x 1.2.3.5", "x 1.2.3.4"])).expect("sorts"), owned(&["x 1.2.3.5", "x 1.2.3.4"]));
    assert_eq!(finalize(&[]).expect("empty"), Vec::<String>::new());
}

/// Entries that cannot be sorted like browserslist sorts them are refused, not guessed.
#[test]
fn finalize_refuses_unsortable_entries() {
    assert_eq!(finalize(&owned(&["nospace"])).expect_err("shape").to_string().starts_with("not implemented: "), true);
    let message = finalize(&owned(&["safari TP"])).expect_err("non-numeric").to_string();
    assert_eq!(message.starts_with("not implemented: "), true, "{message}");
    assert_eq!(message.contains("\"TP\""), true, "{message}");
}

/// The repository's seven lines resolve to the sorted union, with `not dead` removing nothing.
#[test]
fn resolves_the_repository_file() {
    let result = resolve_config_text(REPOSITORY_RC, &committed()).expect("resolves");
    assert_eq!(
        result.iter().take(5).map(String::as_str).collect::<Vec<&str>>(),
        vec!["and_chr 156", "and_ff 159", "android 156", "chrome 156", "firefox 153"]
    );
    assert_eq!(result.iter().skip(5).all(|entry| entry.starts_with("node 26.")), true);
    assert_eq!(result.len() > 5, true);
}

/// Line order does not change the result, and repeating a line adds nothing.
#[test]
fn order_and_repetition_do_not_matter() {
    let data = committed();
    let forward = resolve_config_text(REPOSITORY_RC, &data).expect("resolves");
    let reversed = resolve_config_text(
        "last 1 node major versions\nlast 1 FirefoxAndroid versions\nFirefox ESR\nlast 1 Android versions\nlast 1 ChromeAndroid versions\nlast 1 Chrome versions\nlast 1 Chrome versions\nnot dead\n",
        &data,
    )
    .expect("resolves");
    assert_eq!(forward, reversed);
}

/// `not dead` first is browserslist's own error; so is it after only blank and comment lines.
#[test]
fn not_dead_needs_a_query_before_it() {
    for text in ["not dead", "# c\n\nnot dead\nlast 1 Chrome versions", "not dead\nnot dead"] {
        let message = resolve_config_text(text, &committed()).expect_err("rejected").to_string();
        assert_eq!(message, "Write any browsers query (for instance, `defaults`) before `not dead`", "{text:?}");
    }
}

/// Empty and comment-only configs resolve to an empty list, like `browserslist([])`.
#[test]
fn empty_config_resolves_to_nothing() {
    assert_eq!(resolve_config_text("", &committed()).expect("empty"), Vec::<String>::new());
    assert_eq!(resolve_config_text("# only\n", &committed()).expect("empty"), Vec::<String>::new());
}

/// Each supported line alone resolves, and `not dead` alone fails.
#[test]
fn each_line_alone() {
    let data = committed();
    assert_eq!(resolve_config_text("last 1 Chrome versions", &data).expect("chrome"), owned(&["chrome 156"]));
    assert_eq!(resolve_config_text("Firefox ESR", &data).expect("esr"), owned(&["firefox 153"]));
    assert_eq!(resolve_config_text("not dead", &data).is_err(), true);
}

/// Unsupported queries and config features fail before anything is selected.
#[test]
fn refusals_propagate() {
    for text in ["last 2 Chrome versions", "defaults", "a, b", "[x]", "not dead\r\n"] {
        let message = resolve_config_text(text, &committed()).expect_err("refused").to_string();
        assert_eq!(message.starts_with("not implemented: "), true, "{text:?}: {message}");
    }
}

/// A selected `<name> 0` entry is refused because browserslist would rewrite it.
#[test]
fn zero_version_is_refused() {
    let data = BrowserData {
        agents: AgentTable::new(vec![Agent { name: "chrome".to_string(), slots: vec![Some("0".to_string()), None, None, None] }]),
        node_versions: Vec::new(),
        firefox_esr_major: 1,
    };
    let message = resolve_config_text("last 1 Chrome versions", &data).expect_err("refused").to_string();
    assert_eq!(message.starts_with("not implemented: "), true, "{message}");
}

/// A missing browser in the data is browserslist's `Unknown browser` error.
#[test]
fn missing_browser_is_reported() {
    let data = BrowserData { agents: AgentTable::new(Vec::new()), node_versions: Vec::new(), firefox_esr_major: 1 };
    let error = resolve_config_text("last 1 Android versions", &data).expect_err("missing");
    assert_eq!(error.to_string(), "Unknown browser android");
}
