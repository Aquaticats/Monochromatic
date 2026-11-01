//! What:
//!  Evaluates browserslist's own `dead` expansion against locked data,
//! and subtracts it for `not dead`.
//! Why:
//!  `dead` is six internal queries (`Baidu >= 0`,
//! `ie <= 11`,
//! `ie_mob <= 11`,
//! `bb <= 10`,
//! `op_mob <= 12.1`,
//!  `samsung 4`),
//! and the decision record wants them evaluated,
//! not replaced by a hard-coded result.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const dead = resolve(['Baidu >= 0', 'ie <= 11', 'ie_mob <= 11', 'bb <= 10', 'op_mob <= 12.1', 'samsung 4']);
//! ```

/// What:
///  Import `Agent, AgentTable` from `crate::caniuse`.
/// Why:
///  Version lists come from the decoded caniuse-lite data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Agent, AgentTable } from './caniuse';
/// ```
use crate::caniuse::{Agent, AgentTable};
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
///  Import `parse_float, parse_latest_float` from `crate::js_number`.
/// Why:
///  Version comparison needs JavaScript's number-prefix reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseFloat, parseLatestFloat } from './js_number';
/// ```
use crate::js_number::{parse_float, parse_latest_float};
/// What:
///  Import `HashSet` from `std::collections::HashSet`.
/// Why:
///  Hash tables give linear-time lookups.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { HashSet } from 'std/collections';
/// ```
use std::collections::HashSet;

/// What:
///  One of the two internal query forms `dead` uses.
/// Why:
///  A closed enum lists exactly the rules browserslist's `dead` list needs,
/// so nothing else can be expressed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DeadRule = { kind: 'at-least'; browser: string; version: string } | { kind: 'at-most'; browser: string; version: string } | { kind: 'exact'; browser: string; version: string };
/// ```
enum DeadRule {
    /// `<browser> >= <version>`:
    /// released versions whose newest member is at least the bound.
    AtLeast(
        /// The browser name as the dead list spells it.
        &'static str,
        /// The lower bound,
        /// as written.
        &'static str,
    ),
    /// `<browser> <= <version>`:
    /// released versions at most the bound.
    AtMost(
        /// The browser name as the dead list spells it.
        &'static str,
        /// The upper bound,
        /// as written.
        &'static str,
    ),
    /// `<browser> <version>`:
    /// that single version (with browserslist's alias and `.0` fallbacks).
    Exact(
        /// The browser name as the dead list spells it.
        &'static str,
        /// The version,
        /// as written.
        &'static str,
    ),
}

/// browserslist 4.29.5 `index.js:1219-1226`,
/// in order.
const DEAD_RULES: [DeadRule; 6] = [
    DeadRule::AtLeast("Baidu", "0"),
    DeadRule::AtMost("ie", "11"),
    DeadRule::AtMost("ie_mob", "11"),
    DeadRule::AtMost("bb", "10"),
    DeadRule::AtMost("op_mob", "12.1"),
    DeadRule::Exact("samsung", "4"),
];

/// Whether a dash-joined range such as `10-11` has `version` as one of its parts.
fn has_part(range: &str, version: &str) -> bool {
    for part in range.split('-') {
        if part == version {
            return true;
        }
    }
    return false;
}

/// Format `<name> <version>` the way every browserslist result is spelled.
fn entry(agent: &Agent, version: &str) -> String {
    return format!("{} {}", agent.name, version);
}

/// The ray form:
/// filter the released versions by comparing parsed numbers (NaN compares false).
fn ray(table: &AgentTable, written: &str, bound: &str, keep: fn(&str, &str) -> bool) -> Outcome<Vec<String>> {
    let agent = table.require(written)?;
    let mut entries: Vec<String> = Vec::new();
    for version in agent.released() {
        if keep(version, bound) {
            entries.push(entry(agent, version));
        }
    }
    return Ok(entries);
}

/// `>=` compares the newest member of a joined range against the bound.
fn at_least(version: &str, bound: &str) -> bool {
    return parse_latest_float(version) >= parse_latest_float(bound);
}

/// `<=` compares the leading number of the version against the bound.
fn at_most(version: &str, bound: &str) -> bool {
    return parse_float(version) <= parse_float(bound);
}

/// browserslist's `resolveVersion` plus `normalizeVersion`:
/// the version itself,
/// a joined range it belongs to,
/// or the only version a one-version browser has.
fn normalize_version(agent: &Agent, version: &str) -> Option<String> {
    let versions = agent.versions();
    if versions.contains(&version) {
        return Some(version.to_string());
    }
    let mut alias: Option<&str> = None;
    for full in versions.iter().copied() {
        if full.contains('-') && has_part(full, version) {
            alias = Some(full);
        }
    }
    if let Some(range) = alias {
        return Some(range.to_string());
    }
    if versions.len() == 1 {
        return Some(versions[0].to_string());
    }
    return None;
}

/// The single-version form:
/// try the version,
/// then browserslist's `.0` variant,
/// else fail like browserslist.
fn exact(table: &AgentTable, written: &str, version: &str) -> Outcome<Vec<String>> {
    let agent = table.require(written)?;
    if let Some(found) = normalize_version(agent, version) {
        return Ok(vec![entry(agent, &found)]);
    }
    // browserslist strips a trailing `.0` from versions that contain a dot; the only version this rule
    // takes is `4`, which has none, so only the "append .0" variant is reachable here.
    let variant = format!("{version}.0");
    if let Some(found) = normalize_version(agent, &variant) {
        return Ok(vec![entry(agent, &found)]);
    }
    return Err(BrowserslistError::Browserslist { message: format!("Unknown version {version} of {written}") });
}

/// What:
///  Every `<name> <version>` string that browserslist's `dead` query selects from this data.
/// Why:
///  `not dead` removes exactly these strings from what the earlier queries selected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deadSet(table: AgentTable): string[];
/// ```
pub fn dead_set(table: &AgentTable) -> Outcome<Vec<String>> {
    let mut selected: Vec<String> = Vec::new();
    for rule in &DEAD_RULES {
        let part = match rule {
            DeadRule::AtLeast(browser, bound) => ray(table, browser, bound, at_least)?,
            DeadRule::AtMost(browser, bound) => ray(table, browser, bound, at_most)?,
            DeadRule::Exact(browser, version) => exact(table, browser, version)?,
        };
        selected.extend(part);
    }
    return Ok(selected);
}

/// What:
///  Remove from `selected` every string that `dead` selects,
/// keeping the order of the rest.
/// Why:
///  This is the `not dead` step:
/// browserslist's `not` filters the accumulator by exact string equality.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function subtractDead(selected: string[], table: AgentTable): string[];
/// ```
pub fn subtract_dead(selected: &[String], table: &AgentTable) -> Outcome<Vec<String>> {
    let dead: HashSet<String> = dead_set(table)?.into_iter().collect();
    let mut kept: Vec<String> = Vec::new();
    for candidate in selected {
        if !dead.contains(candidate) {
            kept.push(candidate.clone());
        }
    }
    return Ok(kept);
}

/// What:
///  The unit tests of `dead.rs`,
/// kept in the sidecar file `dead_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './dead.test';
/// ```
#[cfg(test)]
#[path = "dead_tests.rs"]
mod tests;
