//! What:
//! The fixed values of the shadow:
//! the name it answers to,
//! the file that marks a pnpm workspace,
//! the file names it
//! reports on,
//! the directories it never enters,
//! and the exit codes it uses when it cannot run pnpm.
//! Why:
//! One place names each value,
//! so the code and the tests read the same constant instead of repeating a literal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // constants.ts
//! export const SHADOWED_NAME = 'pnpm';
//! ```

/// What:
/// `pub const SHADOWED_NAME: &str = "pnpm";` is a constant text value.
/// `&str` is a borrowed string slice (sibling:
/// `String`,
/// the owned growable one).
/// Constants live for the whole program,
/// so a borrowed slice is enough.
/// Why:
/// The binary started under this name runs in shadow mode;
/// the PATH search looks for this name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SHADOWED_NAME = 'pnpm';
/// ```
pub const SHADOWED_NAME: &str = "pnpm";

/// The file whose presence marks the root of a pnpm workspace.
pub const WORKSPACE_MARKER: &str = "pnpm-workspace.yaml";

/// What:
/// `pub const MANIFEST_NAMES: [&str; 4] = [...]` is a fixed-size list of four text values (sibling:
/// `Vec<&str>`,
/// a
/// growable list).
/// The size is part of the type,
/// so the compiler checks the count.
/// Why:
/// The walk reports on these file names only:
/// `package.json` is read and checked,
/// the other three are refused by
/// name,
/// which is the repository's catalog policy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MANIFEST_NAMES = ['package.json', 'package.json5', 'package.yaml', 'package.yml'] as const;
/// ```
pub const MANIFEST_NAMES: [&str; 4] = ["package.json", "package.json5", "package.yaml", "package.yml"];

/// The one manifest file name whose bytes are read.
pub const READ_MANIFEST_NAME: &str = "package.json";

/// Directory names the walk never enters,
/// whatever `.gitignore` says.
pub const SKIPPED_DIRECTORIES: [&str; 2] = [".git", "node_modules"];

/// What:
/// `pub const EXEC_FAILED_EXIT_CODE: i32 = 126;` is a signed 32-bit integer (siblings:
/// `u8`,
/// `u32`,
/// `i64`).
/// Why:
/// The shell convention for "found but could not start";
/// `i32` because `std::process::exit` takes that type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const EXEC_FAILED_EXIT_CODE = 126;
/// ```
pub const EXEC_FAILED_EXIT_CODE: i32 = 126;

/// The shell convention for "command not found".
pub const NOT_FOUND_EXIT_CODE: i32 = 127;

/// The environment variable that names the per-user runtime directory,
/// where the notice handshake leaves its marker files.
pub const RUNTIME_DIR_VARIABLE: &str = "XDG_RUNTIME_DIR";

/// The start of the name of a notice marker;
/// the process id of the instance the marker is for follows it.
pub const NOTICE_MARKER_PREFIX: &str = "pnpm-shadow-notice-";

/// A child killed by signal `n` exits as `128 + n` in a shell,
/// and the shadow reports it the same way.
pub const SIGNAL_EXIT_CODE_OFFSET: i32 = 128;
