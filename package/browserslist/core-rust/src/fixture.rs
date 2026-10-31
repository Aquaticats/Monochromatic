//! What:
//!  Test-only access to the committed data revision under `fixtures/data`.
//! Why:
//!  Several test modules need the same real revision;
//!  one loader keeps them honest about which files they read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const fixtureFiles = (firefoxEsr: string): DataFiles => ({ /* the four committed files */ });
//! ```

use crate::data::DataFiles;

/// Raw text of the committed caniuse-lite `agents.js`.
pub const AGENTS_JS: &str = include_str!("../fixtures/data/agents.js.txt");
/// Raw text of the committed caniuse-lite `browsers.js`.
pub const BROWSERS_JS: &str = include_str!("../fixtures/data/browsers.js.txt");
/// Raw text of the committed caniuse-lite `browserVersions.js`.
pub const BROWSER_VERSIONS_JS: &str = include_str!("../fixtures/data/browserVersions.js.txt");
/// Raw text of the committed node-releases `envs.json`.
pub const ENVS_JSON: &str = include_str!("../fixtures/data/envs.json.txt");
/// The ESR value the conformance fixtures were generated against.
pub const FIXTURE_ESR: &str = "153.6.0esr";

/// What:
///  The committed revision as `DataFiles` with a chosen ESR value.
/// Why:
///  Tests vary the ESR value without touching the four data files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fixtureFiles(firefoxEsr: string): DataFiles;
/// ```
pub fn fixture_files(firefox_esr: &str) -> DataFiles<'_> {
    return DataFiles {
        agents_js: AGENTS_JS,
        browsers_js: BROWSERS_JS,
        browser_versions_js: BROWSER_VERSIONS_JS,
        envs_json: ENVS_JSON,
        firefox_esr,
    };
}
