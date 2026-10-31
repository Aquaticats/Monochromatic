//! What:
//! The fixed values the other modules share:
//! the name the shadow answers to,
//! the crate-name prefix that marks
//!           this repository's own crates,
//! and the exit codes.
//! Why:
//! One place to read and change them,
//! so no module repeats a bare text or number.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // constants.ts
//! export const SHADOWED_NAME = 'mpm';
//! ```

/// What:
/// `pub const SHADOWED_NAME: &str = "mpm";` declares a public constant.
/// `&str` is a borrowed read-only text
///           slice that lives for the whole program (siblings:
/// `String`,
/// the owned growable text).
/// Why:
/// Started under this name (a link named `mpm`),
/// the binary acts as the shadow;
/// `&str` and not `String`
///           because a literal baked into the program needs no allocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SHADOWED_NAME = 'mpm';
/// ```
pub const SHADOWED_NAME: &str = "mpm";

/// Name prefix every crate this repository publishes carries (`monochromatic-lint`,
/// `monochromatic-git-policy-cli`).
/// Only entries with this prefix are ever deleted from the registry.
pub const OWN_CRATE_PREFIX: &str = "monochromatic-";

/// Suffix of a crate archive in the registry's `cache` directory.
pub const CRATE_ARCHIVE_SUFFIX: &str = ".crate";

/// What:
/// `pub const USAGE_EXIT_CODE: i32 = 2;` declares a constant 32-bit signed integer (siblings:
/// `u32`,
/// `i64`).
/// Why:
/// The exit code for a bad `mpm-shadow` command line,
/// matching the convention clap and the other shadow use;
///           `i32` and not `u8` because `std::process::exit` takes an `i32`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const USAGE_EXIT_CODE = 2;
/// ```
pub const USAGE_EXIT_CODE: i32 = 2;

/// The real `mpm` was found but could not be started (the shell convention for "found but not runnable").
pub const EXEC_FAILED_EXIT_CODE: i32 = 126;

/// No real `mpm` is on PATH (the shell convention for "command not found").
pub const NOT_FOUND_EXIT_CODE: i32 = 127;

/// The offset a shell adds to a signal number to report death by that signal (`128 + 15` for SIGTERM).
pub const SIGNAL_EXIT_CODE_OFFSET: i32 = 128;
