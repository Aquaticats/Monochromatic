//! What:
//!     Counting back-end:
//!  a small structural matcher that keeps bounded-repetition counts
//!          in runtime counter-sets instead of unrolled DFA states.
//!  The eager DFA unrolls
//!          `{n,m}`,
//!  which explodes when a repeated class overlaps a literal prefix under
//!          unanchored search (an `AKIA[A-Z2-7]{16}` key is exactly that shape).
//!  This module
//!          keeps such a pattern as a [`CountingNfa`] of positions and matches it with a
//!          counting-set simulation whose serialized size is linear in the pattern,
//!  never in
//!          the repetition bound;
//!  alternation costs only follow edges,
//!  and intersection with
//!          complement runs as a synchronized [`ProductProgram`].
//! Why:
//!      This file is the Rust module that groups the counting implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module counting: see exported functions and types below.
//! ```

/// What:
///     The element leaf type and its decode-time validation.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./element";
/// ```
mod element;

/// What:
///     The bounded count bitset used by the simulation state.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./countset";
/// ```
mod countset;

/// What:
///     The required-literal prefilter derived from a counting NFA.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./prefilter";
/// ```
mod prefilter;

/// What:
///     The shared counting-NFA simulation core.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./sim";
/// ```
mod sim;

/// What:
///     The serializable counting NFA.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./nfa";
/// ```
mod nfa;

/// What:
///     The Glushkov-style NFA builder.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./build";
/// ```
mod build;

/// What:
///     The single-pattern counting-NFA search loop.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./run";
/// ```
mod run;

/// What:
///     The synchronized-product back-end for `&` and `~`.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./product";
/// ```
mod product;

/// What:
///     Re-exports the counting NFA and its builder for the engine and compiler.
/// Why:
///      The surrounding function uses this step to keep the matcher behavior correct at
///          this point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Same step as the Rust statement below, written with ordinary TS objects/functions.
/// ```
pub use build::build_nfa;

/// What:
///     Re-exports the counting NFA type for the engine.
/// Why:
///      The surrounding function uses this step to keep the matcher behavior correct at
///          this point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Same step as the Rust statement below, written with ordinary TS objects/functions.
/// ```
pub use nfa::CountingNfa;

/// What:
///     Re-exports the product program and its builder for the engine and compiler.
/// Why:
///      The surrounding function uses this step to keep the matcher behavior correct at
///          this point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Same step as the Rust statement below, written with ordinary TS objects/functions.
/// ```
pub use product::{ProductProgram, build_product};

/// What:
///     Re-exports the prefilter,
///  seed extractors,
///  and leading-seed probes for the engine.
/// Why:
///      The surrounding function uses this step to keep the matcher behavior correct at
///          this point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Same step as the Rust statement below, written with ordinary TS objects/functions.
/// ```
pub(crate) use prefilter::{
    Prefilter, leading_seeds, leading_seeds_min, seeds_from_node, seeds_from_node_min,
};
