//! What:
//!  Tests for assembling `BrowserData` from the committed fixture revision.
//! Why:
//!  Each input file can fail on its own, and the diagnostics must name the file at fault.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('BrowserData.fromFiles', () => { /* success and per-file failures */ });
//! ```

use crate::data::BrowserData;
use crate::fixture::fixture_files;

/// The committed revision assembles with ESR major 153.
#[test]
fn assembles_the_committed_revision() {
    let data = BrowserData::from_files(fixture_files("153.6.0esr")).expect("fixture assembles");
    assert_eq!(data.firefox_esr_major, 153);
    assert_eq!(data.agents.get("chrome").is_some(), true);
    assert_eq!(data.node_versions.is_empty(), false);
}

/// Each file's failure surfaces, and a bad ESR value fails with its own diagnostic.
#[test]
fn each_input_can_fail() {
    let mut envs_files = fixture_files("153.6.0esr");
    envs_files.envs_json = "[";
    assert_eq!(BrowserData::from_files(envs_files).expect_err("envs").to_string().contains("envs.json"), true);
    let mut agents_files = fixture_files("153.6.0esr");
    agents_files.agents_js = "nope";
    assert_eq!(BrowserData::from_files(agents_files).expect_err("agents").to_string().contains("agents.js"), true);
    let message = BrowserData::from_files(fixture_files("153")).expect_err("esr").to_string();
    assert_eq!(message.contains("<major>.<minor>.<patch>esr"), true);
}
