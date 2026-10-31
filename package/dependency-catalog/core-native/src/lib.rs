//! What:     The crate root of `monochromatic-dependency-catalog`: it lists the modules and re-exports the public API.
//! Why:      The Rust file-enforcer engine links this library to render `rust-catalog.toml` into each Cargo
//!           workspace root's `[workspace.dependencies]`. Every public function takes file contents and returns
//!           file contents or drift; the library never touches the file system.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // index.ts
//! export * from './sync.ts';
//! ```

/// Catalog parsing and validation.
pub mod catalog;

/// Validation of one catalog table.
pub mod catalog_entry;

/// Validation of the supply-chain settings.
pub mod catalog_settings;

/// Error types.
pub mod error;

/// Double-quoted string rendering.
pub mod quote;

/// Single-comparator Cargo requirements.
pub mod requirement;

/// Crate-name and version-requirement validators.
pub mod syntax;

/// Catalog value types.
pub mod types;

/// Version parsing and ordering.
pub mod version;

#[cfg(test)]
mod catalog_tests;

#[cfg(test)]
mod requirement_tests;

#[cfg(test)]
mod syntax_tests;
