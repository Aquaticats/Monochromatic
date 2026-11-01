//! What:
//!      This Rust crate exports [`compile`],
//!  [`Regex`],
//!  and [`RegexSet`].
//!           A Rust crate is closest to a TypeScript package module:
//!  it has private
//!           implementation files and a public API surface.
//!  The implementation matches
//!           byte slices (`&[u8]`,
//!  a borrowed read-only view of bytes,
//!  not an owned
//!           `Vec<u8>` or fixed `[u8; N]` array),
//!  and internally chooses counting,
//!           product,
//!  or derivative-DFA matcher back-ends plus set-level literal gating.
//! Why:
//!      This file is the Rust module that groups the lib implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module lib: see exported functions and types below.
//! ```
//!
//! The batch match kernels (`dfa::sheng`/`dfa::sheng2`) use explicit `std::arch` SIMD
//! intrinsics (`vpermb` / `vqtbl4q`),
//!  runtime-detected,
//!  so the crate needs no nightly
//! `portable_simd` feature.

/// What:
///     The byte-set leaf alphabet.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./charset";
/// ```
mod charset;

/// What:
///     The compile-time error type.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./error";
/// ```
mod error;

/// What:
///     The regex node algebra and its smart constructors.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./ast";
/// ```
mod ast;

/// What:
///     The boundary context resolving zero-width assertions.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./context";
/// ```
mod context;

/// What:
///     Position-dependent nullability.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./nullable";
/// ```
mod nullable;

/// What:
///     Brzozowski byte derivatives.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./derivative";
/// ```
mod derivative;

/// What:
///     Pattern parsing into the node algebra.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./parse";
/// ```
mod parse;

/// What:
///     Determinization and the compiled table.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./dfa";
/// ```
mod dfa;

/// What:
///     The counting back-end for bounded repetition.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./counting";
/// ```
mod counting;

/// What:
///     The per-pattern back-end selector.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./engine";
/// ```
mod engine;

/// What:
///     Per-rule back-end selection and the seedless-rule fold into the gate.
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
///     Builds a ruleset's rules across worker threads,
///  results in input order.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./parallel";
/// ```
mod parallel;

/// What:
///     The RegexSet-level combined literal prefilter.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./gate";
/// ```
mod gate;

/// What:
///     Greedy combination of literal-free rules into union DFAs.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./group";
/// ```
mod group;

/// What:
///     Public matcher types.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./regex";
/// ```
mod regex;

/// What:
///     Re-exports the compile-time error type.
/// Why:
///      The surrounding function uses this step to keep the matcher behavior correct at
///          this point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Same step as the Rust statement below, written with ordinary TS objects/functions.
/// ```
pub use crate::error::CompileError;

/// What:
///     Re-exports the public matcher API.
/// Why:
///      The surrounding function uses this step to keep the matcher behavior correct at
///          this point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Same step as the Rust statement below, written with ordinary TS objects/functions.
/// ```
pub use crate::regex::{Regex, RegexSet, compile};

/// Diagnostic:
///  debug-prints the parsed node of a pattern that has no usable seed.
///
/// What:
///  returns `Some(debug)` when the pattern parses and is seedless (no leading
/// seed and no required-literal seed),
///  else `None`.
///  Why:
///  a temporary probe for the
/// CsA work to enumerate exactly which rules force the literal-free second pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function debug_seedless(pattern: string): string | null {
///   // Rust body below is the implementation.
/// }
/// ```
#[doc(hidden)]
pub fn debug_seedless(pattern: &str) -> Option<String> {
    let node = crate::parse::parse(pattern).ok()?;
    let leading = crate::counting::leading_seeds(&node);
    let seeds = crate::counting::seeds_from_node(&node);
    if leading.is_empty() && seeds.is_empty() {
        return Some(format!("{node:?}"))
    } else {
        return None
    }
}

/// Diagnostic:
///  tries to build ONE combined search DFA over every pattern.
///
/// What:
///  parses each pattern,
///  alternates them under a single `Σ*` search prefix,
///  and
/// eagerly determinizes the union;
///  returns the state count on success or the
/// `CompileError` (typically `StateCap`) on a blowup.
///  Why:
///  a measured probe of the
/// "all-rules combined automaton" idea,
///  to see whether a single monolithic DFA over the
/// whole ruleset is even buildable (vs the per-rule gate-plus-fold architecture).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function try_combined_dfa(patterns: string[]): number {
///   // Rust body below is the implementation.
/// }
/// ```
#[doc(hidden)]
pub fn try_combined_dfa(patterns: &[&str]) -> Result<usize, CompileError> {
    let roots: Vec<crate::ast::node::Node> =
        patterns.iter().filter_map(|pattern| return crate::parse::parse(pattern).ok()).collect();
    let union = crate::ast::smart::alt(roots);
    let combined = crate::ast::smart::concat(vec![crate::ast::node::Node::Top, union]);
    return Ok(crate::dfa::build_dfa_within(combined, 65_534)?.num_states as usize)
}
