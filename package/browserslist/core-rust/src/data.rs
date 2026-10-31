//! What:
//!  The decoded data one resolve run reads: browser agents, Node.js release versions, and the Firefox ESR major.
//! Why:
//!  Resolution is a pure function of the query text plus this value,
//!  so tests can hand it synthetic or committed data without any network or cache.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type BrowserData = { agents: AgentTable; nodeVersions: string[]; firefoxEsrMajor: number };
//! ```

use crate::caniuse::{AgentTable, parse_agents};
use crate::error::Outcome;
use crate::firefox_esr::parse_esr_major;
use crate::node_releases::parse_node_versions;

/// What:
///  The four raw data files plus the raw ESR value, borrowed as text.
/// Why:
///  A single named struct keeps the five same-typed text arguments from being swapped by mistake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DataFiles = { agentsJs: string; browsersJs: string; browserVersionsJs: string; envsJson: string; firefoxEsr: string };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DataFiles<'a> {
    /// Text of caniuse-lite `data/agents.js`.
    pub agents_js: &'a str,
    /// Text of caniuse-lite `data/browsers.js`.
    pub browsers_js: &'a str,
    /// Text of caniuse-lite `data/browserVersions.js`.
    pub browser_versions_js: &'a str,
    /// Text of node-releases `data/processed/envs.json`.
    pub envs_json: &'a str,
    /// The locked `FIREFOX_ESR` value, for example `153.6.0esr`.
    pub firefox_esr: &'a str,
}

/// What:
///  Everything the selectors read.
/// Why:
///  One owned value is passed through the whole resolution.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BrowserData = { agents: AgentTable; nodeVersions: string[]; firefoxEsrMajor: number };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserData {
    /// Browsers and their version slots.
    pub agents: AgentTable,
    /// Node.js release versions in file order.
    pub node_versions: Vec<String>,
    /// The Firefox ESR major version.
    pub firefox_esr_major: u64,
}

impl BrowserData {
    /// What:
    ///  Decode the raw files into data, failing on the first malformed file.
    /// Why:
    ///  Lock-pinned downloads and committed fixtures both enter through this one door.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function fromFiles(files: DataFiles): BrowserData;
    /// ```
    pub fn from_files(files: DataFiles<'_>) -> Outcome<Self> {
        let agents = parse_agents(files.agents_js, files.browsers_js, files.browser_versions_js)?;
        let node_versions = parse_node_versions(files.envs_json)?;
        let firefox_esr_major = parse_esr_major(files.firefox_esr)?;
        return Ok(Self { agents, node_versions, firefox_esr_major });
    }
}

#[cfg(test)]
#[path = "data_tests.rs"]
mod tests;
