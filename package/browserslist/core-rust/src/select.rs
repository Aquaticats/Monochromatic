//! What:
//!  The selectors of the three positive query families: last released version, Node.js major, Firefox ESR.
//! Why:
//!  Each mirrors one `select` function of npm browserslist 4.29.5 for the only count the crate supports, 1.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function select(query: Query, data: BrowserData): string[];
//! ```

use crate::data::BrowserData;
use crate::error::{BrowserslistError, Outcome};
use crate::js_number::parse_int_prefix;
use crate::query::Browser;

/// What:
///  `last 1 <browser> versions`: the single newest released version, spelled `<key> <version>`.
/// Why:
///  browserslist takes `released.slice(-1)`; for Android its jump rule also returns the last entry when
///  the count is at most the jump, which every count of 1 is, so Chrome data is never consulted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lastVersion(browser: Browser, data: BrowserData): string[];
/// ```
pub fn last_version(browser: Browser, data: &BrowserData) -> Outcome<Vec<String>> {
    let agent = data.agents.require(browser.agent_key())?;
    let released = agent.released();
    return Ok(released.last().map(|version| format!("{} {}", agent.name, version)).into_iter().collect());
}

/// Read the major of a Node.js version like `26.13.0` the way `parseInt(version.split('.')[0])` does.
fn node_major(version: &str) -> Outcome<u64> {
    let first = version.split('.').next().unwrap_or("");
    return parse_int_prefix(first).ok_or_else(|| {
        return BrowserslistError::Data {
            file: "node-releases data/processed/envs.json".to_string(),
            offset: None,
            message: format!("version {version:?} does not start with a number"),
        };
    });
}

/// What:
///  `last 1 node major versions`: every release whose major equals the newest major, in file order,
///  or the whole list when that newest major is 0.
/// Why:
///  browserslist's `getMajorVersions` walks the file from the end and keeps releases until the major drops below
///  the minimum; its `!minimum` test makes a minimum of 0 return everything.
/// Gotcha:
///  The result is every `26.x.y`, not only the newest release.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lastNodeMajor(data: BrowserData): string[]; // 'node ' + version for each kept release
/// ```
pub fn last_node_major(data: &BrowserData) -> Outcome<Vec<String>> {
    let Some(newest) = data.node_versions.last() else {
        return Ok(Vec::new());
    };
    let minimum = node_major(newest)?;
    let mut kept: Vec<&String> = Vec::new();
    if minimum == 0 {
        kept.extend(data.node_versions.iter());
    } else {
        for version in data.node_versions.iter().rev() {
            if node_major(version)? < minimum {
                break;
            }
            kept.push(version);
        }
        kept.reverse();
    }
    return Ok(kept.into_iter().map(|version| format!("node {version}")).collect());
}

/// What:
///  `Firefox ESR`: `firefox <locked major>`.
/// Why:
///  The decision record sources the major from Mozilla product-details through the lock,
///  where npm browserslist hard-codes a constant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firefoxEsr = (data: BrowserData) => [`firefox ${data.firefoxEsrMajor}`];
/// ```
pub fn firefox_esr(data: &BrowserData) -> Vec<String> {
    return vec![format!("firefox {}", data.firefox_esr_major)];
}

#[cfg(test)]
#[path = "select_tests.rs"]
mod tests;
