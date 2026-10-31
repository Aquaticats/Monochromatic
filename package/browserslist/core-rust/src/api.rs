//! What:
//!  The one entry point the file-enforcer engine calls:
//! `.browserslistrc` text plus a lock in,
//!  the JSON text npm browserslist's pipeline writes to `.browserslistrc.resolved.local.json` out.
//! Why:
//!  The engine needs no knowledge of data files,
//! caches or query semantics.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function renderBrowserslistrc(configText: string, lock: DataLock, source: DataSource): string;
//! ```

/// What:
///  Import `parse_config` from `crate::config::parse_config`.
/// Why:
///  Config text is split into query lines before anything else runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseConfig } from 'crate/config';
/// ```
use crate::config::parse_config;
/// What:
///  Import `Outcome` from `crate::error::Outcome`.
/// Why:
///  Failures use the crate's single error type,
/// so callers match one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Outcome } from 'crate/error';
/// ```
use crate::error::Outcome;
/// What:
///  Import `DataLock` from `crate::lock::DataLock`.
/// Why:
///  The lock says which data revisions to use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DataLock } from 'crate/lock';
/// ```
use crate::lock::DataLock;
/// What:
///  Import `parse_query` from `crate::query::parse_query`.
/// Why:
///  Config lines are matched against the supported queries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseQuery } from 'crate/query';
/// ```
use crate::query::parse_query;
/// What:
///  Import `render_resolved_json` from `crate::render::render_resolved_json`.
/// Why:
///  The output must match the layout of `JSON.stringify`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { renderResolvedJson } from 'crate/render';
/// ```
use crate::render::render_resolved_json;
/// What:
///  Import `resolve_lines` from `crate::resolve::resolve_lines`.
/// Why:
///  Resolution turns query lines into the final target list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { resolveLines } from 'crate/resolve';
/// ```
use crate::resolve::resolve_lines;
/// What:
///  Import `DataSource, load_data` from `crate::source`.
/// Why:
///  Pinned data is loaded through the cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DataSource, loadData } from './source';
/// ```
use crate::source::{DataSource, load_data};

/// What:
///  Resolve `.browserslistrc` text to the sorted,
/// deduplicated target strings,
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
///  Resolve and render in one call:
/// the exact file content,
/// `JSON.stringify(targets, null, 2)` plus a newline.
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

/// What:
///  The unit tests of `api.rs`,
/// kept in the sidecar file `api_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './api.test';
/// ```
#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
