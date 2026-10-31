//! What:
//!  Runs the supported queries in file order and post-processes the result like npm browserslist's top-level call.
//! Why:
//!  Positive queries append, `not dead` subtracts from everything accumulated so far,
//!  then duplicates are removed (first kept) and the list is sorted by name ascending, version descending.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function resolveConfigText(text: string, data: BrowserData): string[]; // browserslist(parseConfig(text).defaults)
//! ```

use crate::config::{ConfigLine, parse_config};
use crate::data::BrowserData;
use crate::dead::subtract_dead;
use crate::error::{BrowserslistError, Outcome, not_implemented};
use crate::js_number::parse_int_prefix;
use crate::query::{Query, parse_query};
use crate::select::{firefox_esr, last_node_major, last_version};
use std::cmp::Ordering;
use std::collections::HashSet;

/// browserslist's message for a `not` query with nothing before it (`index.js:348-354`).
const NOT_FIRST_MESSAGE: &str = "Write any browsers query (for instance, `defaults`) before `not dead`";

/// browserslist rewrites a selected `<name> 0` to the browser's first version; no supported query selects one,
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
///  Remove duplicates keeping the first of each, then sort by name ascending and version descending (stable).
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
    keyed.sort_by(|left, right| -> Ordering {
        return left.0.0.cmp(&right.0.0).then_with(|| right.0.1.cmp(&left.0.1));
    });
    return Ok(keyed.into_iter().map(|(_, item)| item).collect());
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
///  Resolve `.browserslistrc` text against data: split into lines, run the queries, finalize.
/// Why:
///  The pure core of the crate's public API; the I/O wrapper only supplies the data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveConfigText(text: string, data: BrowserData): string[];
/// ```
pub fn resolve_config_text(text: &str, data: &BrowserData) -> Outcome<Vec<String>> {
    let lines = parse_config(text)?;
    return resolve_lines(&lines, data);
}

#[cfg(test)]
#[path = "resolve_tests.rs"]
mod tests;
