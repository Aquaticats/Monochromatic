//! What:
//!     The compiled deterministic automaton:
//!  byte classes,
//!  builder,
//!  table,
//!  and runner.
//! Why:
//!      This file is the Rust module that groups the dfa implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module dfa: see exported functions and types below.
//! ```

/// What:
///     Byte-class equivalence computation.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./classes";
/// ```
mod classes;

/// What:
///     Eager determinization from a node into a `Dfa`.
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
///     Moore state minimization.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./minimize";
/// ```
mod minimize;

/// What:
///     The serializable `Dfa` table and its match loop.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./table";
/// ```
pub mod table;

/// What:
///     Batched multi-line match kernels (scalar,
///  interleaved,
///  branchless tight).
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./batch";
/// ```
pub mod batch;

/// What:
///     Sheng-style in-register transition kernel (`vpermb`/`vqtbl4q`).
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./sheng";
/// ```
pub mod sheng;

/// What:
///     Two-byte composed Sheng kernel (one permute per two input bytes).
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./sheng2";
/// ```
pub mod sheng2;

/// What:
///     Re-exports the capped builder entry point.
/// Why:
///      The surrounding function uses this step to keep the matcher behavior correct at
///          this point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Same step as the Rust statement below, written with ordinary TS objects/functions.
/// ```
pub use build::build_dfa_within;

/// What:
///     Re-exports the minimizer.
/// Why:
///      The surrounding function uses this step to keep the matcher behavior correct at
///          this point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Same step as the Rust statement below, written with ordinary TS objects/functions.
/// ```
pub use minimize::minimize;
