//! What:
//!      Shared support for the `monochromatic-jsonc-edit` fuzz sidecar.
//! Why:
//!       Every target needs the same structured generator and the same invariant checks,
//!  so they
//!           live here once instead of being copied into each harness.
//!
//! Raw bytes almost never form valid JSONC,
//!  so the generator builds documents from unstructured
//! input and the targets assert properties over the result rather than hoping for lucky parses.

/// What:
///      Structured document and path generators built on `arbitrary`.
/// Why:
///       Coverage comes from valid and near-valid JSONC shapes,
///  not from random noise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as generators from './generators';
/// ```
pub mod generators;

/// What:
///      Property checks shared by every fuzz target.
/// Why:
///       One implementation of "canonical emission is stable" keeps the targets honest about
///           asserting the same contract the unit and fixture suites assert.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as invariants from './invariants';
/// ```
pub mod invariants;

/// What:
///      Address selection over a parsed document.
/// Why:
///       Edit targets must address members that exist,
///  chosen by the fuzzer rather than guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as pathPick from './pathPick';
/// ```
pub mod path_pick;

/// What:
///      Re-export the generated document type.
/// Why:
///       Targets import one name instead of reaching into the module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { GeneratedDocument } from './generators';
/// ```
pub use generators::GeneratedDocument;

/// What:
///      Re-export the shared invariant checks.
/// Why:
///       Targets read as a list of properties rather than a list of module paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { assertCanonicalStability, assertCommentsPreserved, assertDepthBound, assertTreesEqual } from './invariants';
/// ```
pub use invariants::{
    assert_canonical_stability, assert_comments_preserved, assert_depth_bound, assert_trees_equal,
    collect_comments,
};

/// What:
///      Re-export address selection.
/// Why:
///       Edit targets need it and nothing else from that module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { randomPath } from './pathPick';
/// ```
pub use path_pick::random_path;

/// What:
///      Generator unit tests.
/// Why:
///       A campaign is only as trustworthy as the documents its generator produces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './generators.test';
/// ```
#[cfg(test)]
mod generators_tests;

/// What:
///      Invariant unit tests,
///  including the negative controls that prove each check can fail.
/// Why:
///       An invariant that cannot fail turns a clean campaign into silence rather than evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './invariants.test';
/// ```
#[cfg(test)]
mod invariants_tests;

/// What:
///      Address-selection unit tests.
/// Why:
///       The edit campaign depends on drawn addresses resolving.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './pathPick.test';
/// ```
#[cfg(test)]
mod path_pick_tests;
