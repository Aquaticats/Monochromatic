//! What:
//!  `monochromatic-browserslist` resolves this repository's `.browserslistrc` to the target list npm browserslist
//!  4.29.5 produces,
//! for the query forms the repository uses,
//! from caniuse-lite and node-releases data pinned by hash.
//! Why:
//!  The user decided to replace the third-party Rust browserslist crates with an owned implementation that supports
//!  only the queries in use and fails with "not implemented" on everything else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export { resolveBrowserslistrc, renderBrowserslistrc } from './api';
//! export { DataLock } from './lock';
//! export { DataSource } from './source';
//! ```

/// What:
///  The engine-facing entry points `resolve_browserslistrc` and `render_browserslistrc`.
/// Why:
///  Callers need only these two functions plus a lock and a data source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './api';
/// ```
pub mod api;
/// What:
///  Decoding of caniuse-lite's packed agent data.
/// Why:
///  The browser version lists every query reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './caniuse';
/// ```
pub mod caniuse;
/// What:
///  The command-line surface used by the `monochromatic-browserslist` binary.
/// Why:
///  Kept in the library so the commands are testable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './cli';
/// ```
pub mod cli;
/// What:
///  `.browserslistrc` text to query lines.
/// Why:
///  Accepts comments and blank lines;
/// refuses the other `parseConfig` features.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './config';
/// ```
pub mod config;
/// What:
///  The decoded data one resolve run reads.
/// Why:
///  Resolution is a pure function of query text plus this value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './data';
/// ```
pub mod data;
/// What:
///  browserslist's `dead` expansion and the `not dead` subtraction.
/// Why:
///  Public so tests (and the engine,
/// if it wants) can run the subtraction on a chosen vector.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './dead';
/// ```
pub mod dead;
/// What:
///  The crate's single error type.
/// Why:
///  One type for all failures,
/// with a distinct "not implemented" variant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './error';
/// ```
pub mod error;
/// What:
///  A loopback HTTP server for tests.
/// Why:
///  Tests stay off the real network and share one set of fixtures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // test helper './fake_registry'
/// ```
#[cfg(test)]
mod fake_registry;
/// What:
///  Firefox ESR value parsing.
/// Why:
///  Shared by the lock and the update step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './firefoxEsr';
/// ```
pub mod firefox_esr;
/// What:
///  The bounded blocking HTTP GET.
/// Why:
///  Used inside the crate;
/// the entry points re-export what callers need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module './fetch'
/// ```
mod fetch;
/// What:
///  Test-only fixtures and helpers.
/// Why:
///  Tests stay off the real network and share one set of fixtures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // test helper './fixture'
/// ```
#[cfg(test)]
mod fixture;
/// What:
///  npm integrity parsing and verification.
/// Why:
///  Used inside the crate;
/// the entry points re-export what callers need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module './integrity'
/// ```
mod integrity;
/// What:
///  The parser for the packed-JavaScript literal subset.
/// Why:
///  Public for the adversarial tests and for reuse by a future caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './jsLiteral';
/// ```
pub mod js_literal;
/// What:
///  JavaScript number-reader ports.
/// Why:
///  Used inside the crate;
/// the entry points re-export what callers need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module './js_number'
/// ```
mod js_number;
/// What:
///  The lock that pins the data revisions.
/// Why:
///  The engine parses the committed lock and hands it to `resolve_browserslistrc`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './lock';
/// ```
pub mod lock;
/// What:
///  The node-releases reader.
/// Why:
///  Used inside the crate;
/// the entry points re-export what callers need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module './node_releases'
/// ```
mod node_releases;
/// What:
///  The seven supported queries.
/// Why:
///  Used inside the crate;
/// the entry points re-export what callers need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module './query'
/// ```
mod query;
/// What:
///  The `JSON.stringify` layout.
/// Why:
///  Used inside the crate;
/// the entry points re-export what callers need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module './render'
/// ```
mod render;
/// What:
///  Query accumulation,
/// deduplication and sorting.
/// Why:
///  Used inside the crate;
/// the entry points re-export what callers need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module './resolve'
/// ```
mod resolve;
/// What:
///  The positive query selectors.
/// Why:
///  Used inside the crate;
/// the entry points re-export what callers need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module './select'
/// ```
mod select;
/// What:
///  The pinned-tarball cache and data loader.
/// Why:
///  Public for `DataSource`,
/// the one knob (cache directory and registry) callers may turn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './source';
/// ```
pub mod source;
/// What:
///  In-memory npm tarball reading.
/// Why:
///  Used inside the crate;
/// the entry points re-export what callers need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module './tarball'
/// ```
mod tarball;
/// What:
///  The update step that builds a new lock from the newest published data.
/// Why:
///  Run by the update binary,
/// never by resolve.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './update';
/// ```
pub mod update;

/// What:
///  Re-export `render_browserslistrc, resolve_browserslistrc` from `api`.
/// Why:
///  Callers import these from the crate root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { renderBrowserslistrc, resolveBrowserslistrc } from 'api';
/// ```
pub use api::{render_browserslistrc, resolve_browserslistrc};
/// What:
///  Re-export `BrowserData, DataFiles` from `data`.
/// Why:
///  Callers import these from the crate root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BrowserData, DataFiles } from 'data';
/// ```
pub use data::{BrowserData, DataFiles};
/// What:
///  Re-export `BrowserslistError, Outcome` from `error`.
/// Why:
///  Callers import these from the crate root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BrowserslistError, Outcome } from 'error';
/// ```
pub use error::{BrowserslistError, Outcome};
/// What:
///  Re-export `DataLock, PinnedPackage` from `lock`.
/// Why:
///  Callers import these from the crate root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DataLock, PinnedPackage } from 'lock';
/// ```
pub use lock::{DataLock, PinnedPackage};
/// What:
///  Re-export `render_resolved_json` from `render::render_resolved_json`.
/// Why:
///  Callers import these from the crate root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { renderResolvedJson } from 'render';
/// ```
pub use render::render_resolved_json;
/// What:
///  Re-export `finalize, resolve_config_text` from `resolve`.
/// Why:
///  Callers import these from the crate root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { finalize, resolveConfigText } from 'resolve';
/// ```
pub use resolve::{finalize, resolve_config_text};
/// What:
///  Re-export `DataSource` from `source::DataSource`.
/// Why:
///  Callers import these from the crate root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DataSource } from 'source';
/// ```
pub use source::DataSource;
