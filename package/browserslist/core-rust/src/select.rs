//! What:
//!  The selectors of the three positive query families:
//! last released version,
//! Node.js major,
//! Firefox ESR.
//! Why:
//!  Each mirrors one `select` function of npm browserslist 4.29.5 for the only count the crate supports, 1.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function select(query: Query, data: BrowserData): string[];
//! ```

/// What:
///  Import `BrowserData` from `crate::data::BrowserData`.
/// Why:
///  Selectors and the loader pass the decoded data around as one value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BrowserData } from 'crate/data';
/// ```
use crate::data::BrowserData;
/// What:
///  Import `BrowserslistError, Outcome` from `crate::error`.
/// Why:
///  Failures use the crate's single error type,
/// so callers match one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BrowserslistError, Outcome } from './error';
/// ```
use crate::error::{BrowserslistError, Outcome};
/// What:
///  Import `parse_int_prefix` from `crate::js_number::parse_int_prefix`.
/// Why:
///  Version comparison needs JavaScript's number-prefix reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseIntPrefix } from 'crate/js_number';
/// ```
use crate::js_number::parse_int_prefix;
/// What:
///  Import `Browser` from `crate::query::Browser`.
/// Why:
///  Config lines are matched against the supported queries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Browser } from 'crate/query';
/// ```
use crate::query::Browser;

/// What:
///  `last 1 <browser> versions`:
/// the single newest released version,
/// spelled `<key> <version>`.
/// Why:
///  browserslist takes `released.slice(-1)`;
/// for Android its jump rule also returns the last entry when
///  the count is at most the jump,
/// which every count of 1 is,
/// so Chrome data is never consulted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lastVersion(browser: Browser, data: BrowserData): string[];
/// ```
pub fn last_version(browser: Browser, data: &BrowserData) -> Outcome<Vec<String>> {
    let agent = data.agents.require(browser.agent_key())?;
    let released = agent.released();
    let mut entries: Vec<String> = Vec::new();
    if let Some(version) = released.last() {
        entries.push(format!("{} {}", agent.name, version));
    }
    return Ok(entries);
}

/// Read the major of a Node.js version like `26.13.0` the way `parseInt(version.split('.')[0])` does.
fn node_major(version: &str) -> Outcome<u64> {
    let first = version.split('.').next().unwrap_or("");
    let Some(major) = parse_int_prefix(first) else {
        return Err(BrowserslistError::Data {
            file: "node-releases data/processed/envs.json".to_string(),
            offset: None,
            message: format!("version {version:?} does not start with a number"),
        });
    };
    return Ok(major);
}

/// What:
///  Whether a Node.js version carries a semver pre-release part,
/// such as `27.0.0-alpha.0.0.0`:
/// a dash in the version once any `+build` suffix is removed.
/// Why:
///  The user's decision drops pre-release Node.js versions before `last 1 node major versions` is evaluated.
///  See `doc/troubleshooting/browserslist-node-major-prerelease.md`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hasPrerelease = (version: string) => version.split('+')[0].includes('-');
/// ```
pub fn has_prerelease(version: &str) -> bool {
    let without_build = version.split('+').next().unwrap_or("");
    return without_build.contains('-');
}

/// What:
///  `last 1 node major versions`:
/// every stable release whose major equals the newest stable major,
/// in file order,
///  or the whole stable list when that newest major is 0.
/// Why:
///  browserslist's `getMajorVersions` walks the file from the end and keeps releases until the major drops below
///  the minimum;
/// its `!minimum` test makes a minimum of 0 return everything.
///  Pre-release versions are dropped first,
/// which npm browserslist does not do:
/// with `27.0.0-alpha.0.0.0` listed it selects only that alpha.
/// Gotcha:
///  The result is every `26.x.y`,
/// not only the newest release.
///  This is a deliberate divergence from npm browserslist 4.29.5.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lastNodeMajor(data: BrowserData): string[]; // 'node ' + version for each kept stable release
/// ```
pub fn last_node_major(data: &BrowserData) -> Outcome<Vec<String>> {
    let mut stable: Vec<&String> = Vec::new();
    for version in &data.node_versions {
        if !has_prerelease(version) {
            stable.push(version);
        }
    }
    let Some(newest) = stable.last() else {
        return Ok(Vec::new());
    };
    let minimum = node_major(newest)?;
    let mut kept: Vec<&String> = Vec::new();
    if minimum == 0 {
        kept.extend(stable.iter());
    } else {
        for version in stable.iter().rev() {
            if node_major(version)? < minimum {
                break;
            }
            kept.push(version);
        }
        kept.reverse();
    }
    let mut entries: Vec<String> = Vec::with_capacity(kept.len());
    for version in kept {
        entries.push(format!("node {version}"));
    }
    return Ok(entries);
}

/// What:
///  `Firefox ESR`:
/// `firefox <locked major>`.
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

/// What:
///  The unit tests of `select.rs`,
/// kept in the sidecar file `select_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './select.test';
/// ```
#[cfg(test)]
#[path = "select_tests.rs"]
mod tests;
