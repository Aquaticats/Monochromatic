//! What:
//! Fixed values shared by the shadow modules:
//! the escape-hatch variable name,
//! the exit codes and the names of the shadowed tools.
//! Why:
//! One place to read and change them,
//! so no other file repeats a literal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // constants.ts
//! export const ALLOW_WIDE_VARIABLE = 'SEARCH_SHADOW_ALLOW_WIDE';
//! ```

/// What:
/// `pub const ALLOW_WIDE_VARIABLE: &str = "..."` is a public constant string slice
/// (`&str` is a borrowed read-only piece of text;
/// the sibling `String` is an owned growable text).
/// Why:
/// The environment variable name that lets one deliberate full walk through.
/// It is `&str` and not `String` because a constant is baked into the binary and never changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const ALLOW_WIDE_VARIABLE = 'SEARCH_SHADOW_ALLOW_WIDE';
/// ```
pub const ALLOW_WIDE_VARIABLE: &str = "SEARCH_SHADOW_ALLOW_WIDE";

/// What:
/// `pub const REFUSED_EXIT_CODE: i32 = 2;` is a public constant 32-bit signed integer
/// (the siblings are `u8` / `u32` / `i64`).
/// Why:
/// The exit code when the policy refuses a walk.
/// It is `i32` and not `u8` because `std::process::exit` takes an `i32`.
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

/// What:
/// `pub const MAX_OUTPUT_BYTES: u64 = 4 * 1024 * 1024;` is the most of the real tool's standard output the shadow passes
/// through in one run (`u64` because the copy compares a running byte count with this cap).
/// Why:
/// The incident that asked for it (issue 3474) wrote 13,328,044,032 bytes of `rg` output into a 16 GiB tmpfs,
/// and the owner decided "Cap its output" on 2026-11-10.
/// 4 MiB matches the output cap this repository already ships for a push
/// (`package/git-policy/cli/src/native/push/bounded_push.rs`), so one number is enough to remember,
/// and it leaves the largest honest answer this repository produces whole:
/// `rg --files` over the checkout measured 1,553,741 bytes on 2026-11-10.
/// Past 4 MiB no person reads the text and no tool reads it whole, so the head is what is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MAX_OUTPUT_BYTES = 4 * 1024 * 1024;
/// ```
pub const MAX_OUTPUT_BYTES: u64 = 4 * 1024 * 1024;

/// The exit code for a run the cap stopped:
/// the convention of `timeout(1)`,
/// which this repository's push cap also uses.
/// It is not a refusal (2): the walk was wanted, and what was cut is the text after the cap.
pub const CAPPED_EXIT_CODE: i32 = 124;

/// The exit code for a run whose output could not be passed on (a full disk, an input/output error):
/// the shell's convention for a program that did not do its work.
pub const OUTPUT_FAILED_EXIT_CODE: i32 = 1;

/// The exit code for a bad `search-shadow` command line.
pub const USAGE_EXIT_CODE: i32 = 2;

/// What:
/// `pub const SHADOWED_NAMES: &[&str] = &[...]` is the public list of tool names that get a shadow link:
/// the names the real tools use on PATH.
/// A link is made only for names whose real tool exists on this host,
/// so the list may name tools that are not installed
/// (for example `fdfind` is the Debian name of `fd`).
/// Why:
/// The `link` subcommand reads it.
/// The toolchain plan does not repeat it,
/// so adding a tool is one change here.
/// The groups are searchers
/// (`rg` / `grep` / `egrep` / `fgrep` / `ugrep` / `ug` / `ag` / `ack`),
/// finders (`find` / `fd` / `fdfind` / `bfs` / `tree`)
/// and whole-tree readers (`du` / `ncdu` / `tokei` / `cloc`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SHADOWED_NAMES: readonly string[] = ['rg', 'grep', 'egrep', /* ... */ 'cloc'];
/// ```
pub const SHADOWED_NAMES: &[&str] = &[
    "rg", "grep", "egrep", "fgrep", "ugrep", "ug", "ag", "ack", "find", "fd", "fdfind", "bfs", "tree", "du", "ncdu", "tokei",
    "cloc",
];
