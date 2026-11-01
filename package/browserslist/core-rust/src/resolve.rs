//! What:
//!  Runs the supported queries in file order and post-processes the result like npm browserslist's top-level call.
//! Why:
//!  Positive queries append,
//! `not dead` subtracts from everything accumulated so far,
//!  then duplicates are removed (first kept) and the list is sorted by name ascending,
//! version descending.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function resolveConfigText(text: string, data: BrowserData): string[]; // browserslist(parseConfig(text).defaults)
//! ```

/// What:
///  Import `ConfigLine, parse_config` from `crate::config`.
/// Why:
///  Config text is split into query lines before anything else runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ConfigLine, parseConfig } from './config';
/// ```
use crate::config::{ConfigLine, parse_config};
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
///  Import `subtract_dead` from `crate::dead::subtract_dead`.
/// Why:
///  `not dead` needs the dead expansion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { subtractDead } from 'crate/dead';
/// ```
use crate::dead::subtract_dead;
/// What:
///  Import `BrowserslistError, Outcome, not_implemented` from `crate::error`.
/// Why:
///  Failures use the crate's single error type,
/// so callers match one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BrowserslistError, Outcome, notImplemented } from './error';
/// ```
use crate::error::{BrowserslistError, Outcome, not_implemented};
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
///  Import `Query, parse_query` from `crate::query`.
/// Why:
///  Config lines are matched against the supported queries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Query, parseQuery } from './query';
/// ```
use crate::query::{Query, parse_query};
/// What:
///  Import `firefox_esr, last_node_major, last_version` from `crate::select`.
/// Why:
///  Each query family has its own selector.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { firefoxEsr, lastNodeMajor, lastVersion } from './select';
/// ```
use crate::select::{firefox_esr, last_node_major, last_version};
/// What:
///  Import `Ordering` from `std::cmp::Ordering`.
/// Why:
///  Sorting compares names,
/// then versions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Ordering } from 'std/cmp';
/// ```
use std::cmp::Ordering;
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

/// browserslist's message for a `not` query with nothing before it (`index.js:348-354`).
const NOT_FIRST_MESSAGE: &str = "Write any browsers query (for instance, `defaults`) before `not dead`";

/// Order two keyed results by name ascending,
/// then by version descending.
fn compare_keyed(left: &((String, [u64; 3]), String), right: &((String, [u64; 3]), String)) -> Ordering {
    match left.0.0.cmp(&right.0.0) {
        Ordering::Equal => return right.0.1.cmp(&left.0.1),
        other => return other,
    }
}

/// browserslist rewrites a selected `<name> 0` to the browser's first version;
/// no supported query selects one,
/// so meeting one means the data changed shape and the crate refuses instead of guessing.
fn refuse_zero_versions(selected: &[String]) -> Outcome<()> {
    for item in selected {
        if item.split(' ').nth(1) == Some("0") {
            return Err(not_implemented(&format!("selected entry {item:?} has version 0, which browserslist rewrites")));
        }
    }
    return Ok(());
}

/// Split `<name> <version>` and read the first three dot-separated components of the version's part before a dash.
fn sort_key(item: &str) -> Outcome<(String, [u64; 3])> {
    let Some((name, version)) = item.split_once(' ') else {
        return Err(not_implemented(&format!("result entry {item:?} is not `<name> <version>`")));
    };
    let lead = version.split('-').next().unwrap_or("");
    let mut parts = [0_u64; 3];
    for (slot, text) in parts.iter_mut().zip(lead.split('.')) {
        let Some(number) = parse_int_prefix(text) else {
            return Err(not_implemented(&format!("sorting entry {item:?}: version component {text:?} is not a number")));
        };
        *slot = number;
    }
    return Ok((name.to_string(), parts));
}

/// What:
///  Remove duplicates keeping the first of each,
/// then sort by name ascending and version descending (stable).
/// Why:
///  This is the `uniq(...).sort(...)` tail of browserslist's top-level function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finalize(selected: string[]): string[];
/// ```
pub fn finalize(selected: &[String]) -> Outcome<Vec<String>> {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut keyed: Vec<((String, [u64; 3]), String)> = Vec::new();
    for item in selected {
        if seen.insert(item.as_str()) {
            keyed.push((sort_key(item)?, item.clone()));
        }
    }
    keyed.sort_by(compare_keyed);
    let mut ordered: Vec<String> = Vec::with_capacity(keyed.len());
    for (_, item) in keyed {
        ordered.push(item);
    }
    return Ok(ordered);
}

/// What:
///  Resolve parsed config lines against data into the final list.
/// Why:
///  The one place that applies browserslist's accumulate-then-subtract rule across line boundaries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveLines(lines: ConfigLine[], data: BrowserData): string[];
/// ```
pub fn resolve_lines(lines: &[ConfigLine], data: &BrowserData) -> Outcome<Vec<String>> {
    let mut accumulated: Vec<String> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let query = parse_query(line.number, &line.text)?;
        let selected = match query {
            Query::LastVersions(browser) => last_version(browser, data)?,
            Query::FirefoxEsr => firefox_esr(data),
            Query::LastNodeMajorVersions => last_node_major(data)?,
            Query::NotDead => {
                // `not` removes from everything accumulated so far, and is an error when nothing precedes it.
                if index == 0 {
                    return Err(BrowserslistError::Browserslist { message: NOT_FIRST_MESSAGE.to_string() });
                }
                accumulated = subtract_dead(&accumulated, &data.agents)?;
                continue;
            }
        };
        refuse_zero_versions(&selected)?;
        accumulated.extend(selected);
    }
    return finalize(&accumulated);
}

/// What:
///  Resolve `.browserslistrc` text against data:
/// split into lines,
/// run the queries,
/// finalize.
/// Why:
///  The pure core of the crate's public API;
/// the I/O wrapper only supplies the data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveConfigText(text: string, data: BrowserData): string[];
/// ```
pub fn resolve_config_text(text: &str, data: &BrowserData) -> Outcome<Vec<String>> {
    let lines = parse_config(text)?;
    return resolve_lines(&lines, data);
}

/// What:
///  The unit tests of `resolve.rs`,
/// kept in the sidecar file `resolve_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './resolve.test';
/// ```
#[cfg(test)]
#[path = "resolve_tests.rs"]
mod tests;
