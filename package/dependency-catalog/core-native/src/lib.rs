//! What:     The crate root of `monochromatic-dependency-catalog`: it lists the modules and re-exports the public API.
//! Why:      The Rust file-enforcer engine links this library to render `rust-catalog.toml` into each Cargo
//!           workspace root's `[workspace.dependencies]`. Every public function takes file contents and returns
//!           file contents or drift; the library never touches the file system.
//!
//! The public API is three pure functions:
//! `parse_catalog` validates catalog text, `sync` returns the expected contents of the catalog and every manifest
//! (write mode writes them), and `check` returns only the files that drift (check mode diffs them).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // index.ts
//! export { parseRustCatalog } from './catalog.ts';
//! export { syncRustCatalog } from './sync.ts';
//! export { checkRustCatalog } from './check.ts';
//! ```

/// Adoption planning for literal versions.
pub mod adopt;

/// Default-catalog floor planning for registry literals.
pub mod adopt_registry;

/// Named-catalog literal checks.
pub mod adopt_selected;

/// Helpers shared by the adoption planners.
pub mod adopt_shared;

/// Catalog parsing and validation.
pub mod catalog;

/// Validation of one catalog table.
pub mod catalog_entry;

/// Validation of the supply-chain settings.
pub mod catalog_settings;

/// The change report.
pub mod change;

/// Check mode.
pub mod check;

/// Reading one crate manifest.
pub mod crate_info;

/// The per-crate default-features choice.
pub mod default_features;

/// Format-preserving catalog edits.
pub mod edit_catalog;

/// Format-preserving manifest edits.
pub mod edit_manifest;

/// Error types.
pub mod error;

/// Forward-slash path arithmetic.
pub mod posix_path;

/// Double-quoted string rendering.
pub mod quote;

/// TOML text for catalog-owned values.
pub mod render;

/// Single-comparator Cargo requirements.
pub mod requirement;

/// Resolving one crate against the catalog.
pub mod resolve;

/// The use-site model.
pub mod site;

/// Finding and classifying use sites.
pub mod site_classify;

/// Crate-name and version-requirement validators.
pub mod syntax;

/// The pure sync.
pub mod sync;

/// Catalog value types.
pub mod types;

/// Version parsing and ordering.
pub mod version;

pub use crate::catalog::parse_catalog;
pub use crate::change::Change;
pub use crate::check::{DriftReport, check};
pub use crate::crate_info::ManifestFile;
pub use crate::error::{CatalogIssue, Error, SyncIssue};
pub use crate::sync::{FileResult, SyncInput, SyncOutput, sync};
pub use crate::types::{CatalogDocument, Entry, EntryMap, Settings, TrustPolicy};

#[cfg(test)]
mod catalog_tests;

#[cfg(test)]
mod requirement_tests;

#[cfg(test)]
mod syntax_tests;
