//! What:
//!  Tests for the three positive selectors on synthetic and committed data.
//! Why:
//!  The node-major walk has edge cases (empty list, major 0, mixed majors) the real data never reaches.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('selectors', () => { /* last version, node major, ESR */ });
//! ```

use crate::caniuse::{Agent, AgentTable};
use crate::data::BrowserData;
use crate::fixture::fixture_files;
use crate::query::Browser;
use crate::select::{firefox_esr, last_node_major, last_version};

/// Build data around synthetic agents and Node.js versions.
fn data(agents: Vec<Agent>, node_versions: &[&str]) -> BrowserData {
    return BrowserData {
        agents: AgentTable::new(agents),
        node_versions: node_versions.iter().map(|version| (*version).to_string()).collect(),
        firefox_esr_major: 7,
    };
}

/// Build an agent whose last three slots are reserved.
fn agent(name: &str, slots: &[Option<&str>]) -> Agent {
    return Agent { name: name.to_string(), slots: slots.iter().map(|slot| slot.map(str::to_string)).collect() };
}

/// The newest released version is the last non-empty slot before the three reserved ones.
#[test]
fn last_version_skips_reserved_slots() {
    let chrome = agent("chrome", &[Some("1"), Some("2"), None, Some("3"), Some("4"), Some("5")]);
    let synthetic = data(vec![chrome], &[]);
    assert_eq!(last_version(Browser::Chrome, &synthetic).expect("selects"), vec!["chrome 2"]);
}

/// A browser with no released version selects nothing, and a missing browser fails.
#[test]
fn last_version_edge_cases() {
    let empty = data(vec![agent("chrome", &[None, None, Some("9")])], &[]);
    assert_eq!(last_version(Browser::Chrome, &empty).expect("selects"), Vec::<String>::new());
    assert_eq!(last_version(Browser::Android, &empty).expect_err("missing").to_string(), "Unknown browser android");
}

/// Mobile aliases resolve to their caniuse keys and spell results with them.
#[test]
fn last_version_uses_caniuse_keys() {
    let synthetic = data(
        vec![agent("and_chr", &[Some("7"), None, None, None]), agent("and_ff", &[Some("8"), None, None, None])],
        &[],
    );
    assert_eq!(last_version(Browser::ChromeAndroid, &synthetic).expect("selects"), vec!["and_chr 7"]);
    assert_eq!(last_version(Browser::FirefoxAndroid, &synthetic).expect("selects"), vec!["and_ff 8"]);
}

/// Every release of the newest major is kept in file order.
#[test]
fn node_major_keeps_the_newest_major() {
    let synthetic = data(vec![], &["0.2.0", "24.9.0", "25.0.0", "26.0.0", "26.1.0", "26.13.0"]);
    assert_eq!(
        last_node_major(&synthetic).expect("selects"),
        vec!["node 26.0.0", "node 26.1.0", "node 26.13.0"]
    );
}

/// An empty list selects nothing; a newest major of 0 selects the whole list.
#[test]
fn node_major_edge_cases() {
    assert_eq!(last_node_major(&data(vec![], &[])).expect("selects"), Vec::<String>::new());
    assert_eq!(
        last_node_major(&data(vec![], &["0.2.0", "0.3.0"])).expect("selects"),
        vec!["node 0.2.0", "node 0.3.0"]
    );
}

/// A version that does not start with a number is a data error, never skipped.
#[test]
fn node_major_rejects_unreadable_versions() {
    let message = last_node_major(&data(vec![], &["x.1.0"])).expect_err("rejected").to_string();
    assert_eq!(message.contains("does not start with a number"), true, "{message}");
    let message = last_node_major(&data(vec![], &["1.0.0", "bad", "2.0.0"])).expect_err("rejected").to_string();
    assert_eq!(message.contains("\"bad\""), true, "{message}");
}

/// ESR is the locked major.
#[test]
fn esr_uses_the_locked_major() {
    assert_eq!(firefox_esr(&data(vec![], &[])), vec!["firefox 7"]);
}

/// The committed revision selects the values the spec records for it.
#[test]
fn committed_revision_values() {
    let real = BrowserData::from_files(fixture_files("153.6.0esr")).expect("assembles");
    assert_eq!(last_version(Browser::Chrome, &real).expect("chrome"), vec!["chrome 156"]);
    assert_eq!(last_version(Browser::ChromeAndroid, &real).expect("and_chr"), vec!["and_chr 156"]);
    assert_eq!(last_version(Browser::Android, &real).expect("android"), vec!["android 156"]);
    assert_eq!(last_version(Browser::FirefoxAndroid, &real).expect("and_ff"), vec!["and_ff 159"]);
    let node = last_node_major(&real).expect("node");
    assert_eq!(node.first().map(String::as_str), Some("node 26.0.0"));
    assert_eq!(node.iter().all(|entry| entry.starts_with("node 26.")), true);
}
