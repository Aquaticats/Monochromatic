//! What:
//! The crate root:
//! declares the modules and re-exports the public surface.
//! Why:
//! Consumers import `monochromatic_catalog_specifier::{check_manifest, Finding, FindingKind}` and never reach into modules.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // index.ts
//! export { checkManifest, checkManifestBytes, checkManifestPath } from './check';
//! export type { Finding, FindingKind } from './finding';
//! export { isAllowedSpecifier, type SectionKind } from './grammar';
//! ```

/// What:
/// `mod check;` declares a child module whose code lives in `src/check.rs`.
/// Without `pub`,
/// the module itself stays private.
/// Why:
/// The byte-to-findings walk is an implementation detail;
/// only its functions are re-exported below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './check';
/// ```
mod check;
/// Declares the module holding the finding types.
mod finding;
/// Declares the module holding the accepted-specifier grammar.
mod grammar;

/// What:
/// `pub use check::{...};` re-exports the three check functions at the crate root,
/// so callers write one short path.
/// Why:
/// The README contract names these entry points;
/// the module layout may change without breaking callers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { checkManifest, checkManifestBytes, checkManifestPath } from './check';
/// ```
pub use check::{check_manifest, check_manifest_bytes, check_manifest_path};
/// Re-exports the finding types.
pub use finding::{Finding, FindingKind};
/// Re-exports the grammar test and its section kind.
pub use grammar::{is_allowed_specifier, SectionKind};

/// What:
/// `#[cfg(test)]` compiles the next item only when running tests;
/// `mod check_tests;` declares the test module.
/// Why:
/// Tests live in sibling `*_tests.rs` files like the rest of the repository's Rust crates,
/// and ship in no release build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // check.unit.test.ts sits beside check.ts and is excluded from the build
/// ```
#[cfg(test)]
mod check_tests;
/// Tests for the grammar.
#[cfg(test)]
mod grammar_tests;
/// Tests that run every case in `fixtures/vectors.json`.
#[cfg(test)]
mod vectors_tests;
