//! What:
//!  The one entry point the file-enforcer engine calls: `.browserslistrc` text plus a lock in,
//!  the JSON text npm browserslist's pipeline writes to `.browserslistrc.resolved.local.json` out.
//! Why:
//!  The engine needs no knowledge of data files, caches or query semantics.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function renderBrowserslistrc(configText: string, lock: DataLock, source: DataSource): string;
//! ```

use crate::config::parse_config;
use crate::error::Outcome;
use crate::lock::DataLock;
use crate::query::parse_query;
use crate::render::render_resolved_json;
use crate::resolve::resolve_lines;
use crate::source::{DataSource, load_data};

/// What:
///  Resolve `.browserslistrc` text to the sorted, deduplicated target strings,
///  for example `["and_chr 156", "chrome 156", "node 26.13.0"]`.
///  Config and query errors are reported before any cache read or download.
/// Why:
///  An unsupported query must fail with "not implemented" without touching the network.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveBrowserslistrc(configText: string, lock: DataLock, source: DataSource): string[];
/// ```
pub fn resolve_browserslistrc(config_text: &str, lock: &DataLock, source: &DataSource) -> Outcome<Vec<String>> {
    let lines = parse_config(config_text)?;
    for line in &lines {
        parse_query(line.number, &line.text)?;
    }
    let data = load_data(lock, source)?;
    return resolve_lines(&lines, &data);
}

/// What:
///  Resolve and render in one call: the exact file content, `JSON.stringify(targets, null, 2)` plus a newline.
/// Why:
///  The engine writes this text straight to `.browserslistrc.resolved.local.json`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderBrowserslistrc(configText: string, lock: DataLock, source: DataSource): string;
/// ```
pub fn render_browserslistrc(config_text: &str, lock: &DataLock, source: &DataSource) -> Outcome<String> {
    let targets = resolve_browserslistrc(config_text, lock, source)?;
    return Ok(render_resolved_json(&targets));
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
