//! What:
//!     End-to-end tests exercising the public API across the compilation boundary:
//!  the one key literal
//!  every module checks against,
//!  and the module declarations that split the tests by role.
//! Why:
//!      The test binary keeps one shared input here,
//!  so a module that needs the key imports it
//!          instead of restating the bytes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // integration.test.ts: the shared key, and the describe blocks imported from sibling files.
//! ```

/// What:
///     A valid 20-byte AWS-style key:
///  `AKIA` plus 16 bytes from `[A-Z2-7]`.
/// Why:
///      The program gives this fixed value a name so every caller uses the same setting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const AKIA_KEY: string = "AKIAABCDEFGHIJKLMNO\x50";
/// ```
const AKIA_KEY: &str = "AKIAABCDEFGHIJKLMNO\x50";

/// What:
///     The pattern dialect itself:
///  which constructs parse,
///  which are rejected,
///  and what each
///  accepted construct matches.
/// Why:
///      The package keeps that concept in a separate Rust file so the test binary can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./dialect_of_one_pattern";
/// ```
mod dialect_of_one_pattern;

/// What:
///     How one line is routed through a ruleset:
///  which rule ids a line reports,
///  the `^`
///  rules checked at line start,
///  and the weak inner seeds.
/// Why:
///      The package keeps that concept in a separate Rust file so the test binary can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./routing_a_line_through_a_ruleset";
/// ```
mod routing_a_line_through_a_ruleset;

/// What:
///     Saving a compiled set to bytes and loading one back,
///  including the sets and single
///  patterns a decode must reject.
/// Why:
///      The package keeps that concept in a separate Rust file so the test binary can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./saving_and_reloading_a_compiled_set";
/// ```
mod saving_and_reloading_a_compiled_set;

/// What:
///     The counts a built set reports,
///  the bench's profiling hooks,
///  and the two other
///  diagnostics.
/// Why:
///      The package keeps that concept in a separate Rust file so the test binary can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./diagnostic_counts_and_profiling_hooks";
/// ```
mod diagnostic_counts_and_profiling_hooks;
