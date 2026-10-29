//! What:     Fixed values shared by the shadow modules: the escape-hatch variable name, the exit codes and the names
//!           of the shadowed tools.
//! Why:      One place to read and change them, so no other file repeats a literal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // constants.ts
//! export const ALLOW_WIDE_VARIABLE = 'SEARCH_SHADOW_ALLOW_WIDE';
//! ```

/// What:     `pub const ALLOW_WIDE_VARIABLE: &str = "..."` is a public constant string slice (`&str`, a borrowed read-only
///           piece of text; sibling: `String`, an owned growable text).
/// Why:      The environment variable name that lets one deliberate full walk through; `&str` and not `String` because
///           a constant is baked into the binary and never changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const ALLOW_WIDE_VARIABLE = 'SEARCH_SHADOW_ALLOW_WIDE';
/// ```
pub const ALLOW_WIDE_VARIABLE: &str = "SEARCH_SHADOW_ALLOW_WIDE";

/// What:     `pub const REFUSED_EXIT_CODE: i32 = 2;` is a public constant 32-bit signed integer (siblings: `u8`, `u32`,
///           `i64`).
/// Why:      The exit code when the policy refuses a walk; `i32` and not `u8` because `std::process::exit` takes an `i32`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const REFUSED_EXIT_CODE = 2;
/// ```
pub const REFUSED_EXIT_CODE: i32 = 2;

/// The exit code when the real tool is not on PATH (the shell convention for "command not found").
pub const NOT_FOUND_EXIT_CODE: i32 = 127;

/// The exit code when the real tool was found but could not be started (the shell convention).
pub const EXEC_FAILED_EXIT_CODE: i32 = 126;

/// The exit code for a bad `search-shadow` command line.
pub const USAGE_EXIT_CODE: i32 = 2;

/// The tool names that get a shadow link: the same names the real tools use on PATH.
pub const SHADOWED_NAMES: &[&str] = &["rg", "grep", "find"];
