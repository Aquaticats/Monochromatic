//! What:
//!  Shared support for the `monochromatic-xml-edit` fuzz sidecar.
//! Why:
//!  Every target needs the same structured generator and the same invariant checks,
//!  so they live here once instead of being copied into each harness.
//!
//! Raw bytes almost never form well-formed XML,
//!  so the generator builds documents from unstructured input
//!  and the targets assert properties over the result rather than hoping for lucky parses.

/// What:
/// Structured document generator and the single-mutation helper.
/// Why:
/// Coverage comes from valid and near-valid XML,
/// not from random noise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as generators from './generators';
/// ```
pub mod generators;

/// What:
/// Property checks shared by every fuzz target.
/// Why:
/// One implementation of each promise keeps the targets asserting the contract the unit suites assert.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as invariants from './invariants';
/// ```
pub mod invariants;

/// What:
/// Re-export the generated document type.
/// Why:
/// Targets import one name instead of a module path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { GeneratedDocument } from './generators';
/// ```
pub use generators::GeneratedDocument;

/// What:
/// Generator tests,
/// including a validity sweep over many byte patterns.
/// Why:
/// Targets assume validity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './generators.test';
/// ```
#[cfg(test)]
mod generators_tests;

/// What:
/// Invariant tests with negative controls.
/// Why:
/// Each check is shown to panic on doctored input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './invariants.test';
/// ```
#[cfg(test)]
mod invariants_tests;
