//! What:
//!  `monochromatic-browserslist` resolves this repository's `.browserslistrc` to the target list npm browserslist
//!  4.29.5 produces, for the query forms the repository uses, from caniuse-lite and node-releases data pinned by hash.
//! Why:
//!  The user decided to replace the third-party Rust browserslist crates with an owned implementation that supports
//!  only the queries in use and fails with "not implemented" on everything else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export { resolveBrowserslistrc, renderResolvedJson, parseLock, type DataLock } from './index';
//! ```

pub mod caniuse;
pub mod config;
pub mod data;
pub mod dead;
pub mod error;
pub mod firefox_esr;
#[cfg(test)]
mod fixture;
pub mod js_literal;
pub mod js_number;
pub mod node_releases;
pub mod query;
pub mod render;
pub mod resolve;
pub mod select;
