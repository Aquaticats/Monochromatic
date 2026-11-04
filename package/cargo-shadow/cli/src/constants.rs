//! What:
//! The fixed values of the shadow:
//! the name it answers to,
//! the file that marks a catalog repository,
//! the directories
//! it never enters,
//! and the exit codes it uses when it cannot run cargo.
//! Why:
//! One place names each value,
//! so the code and the tests read the same constant instead of repeating a literal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // constants.ts
//! export const SHADOWED_NAME = 'cargo';
//! ```

/// What:
/// `pub const SHADOWED_NAME: &str = "cargo";` is a constant text value.
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
/// export const SHADOWED_NAME = 'cargo';
/// ```
pub const SHADOWED_NAME: &str = "cargo";

/// The name of the binary itself.
/// Cargo runs any `cargo-<name>` executable on PATH as `cargo <name>`,
/// so this one
/// also answers to `cargo shadow <command>`.
pub const OWN_NAME: &str = "cargo-shadow";

/// The word cargo passes as the first argument when it runs `cargo-shadow` as the subcommand `cargo shadow`.
pub const PLUGIN_WORD: &str = "shadow";

/// The file whose presence marks the root of a repository that uses the Rust dependency catalog.
pub const CATALOG_MARKER: &str = "rust-catalog.toml";

/// The manifest file name of a Cargo package or workspace.
pub const MANIFEST_NAME: &str = "Cargo.toml";

/// The lockfile name in each crate root.
pub const LOCKFILE_NAME: &str = "Cargo.lock";

/// The file that pins the toolchains the repository builds with.
pub const TOOLCHAIN_LOCK_NAME: &str = "toolchain.lock.json";

/// Directory names the walks never enter.
pub const SKIPPED_DIRECTORIES: [&str; 3] = [".git", "node_modules", "target"];

/// The shell convention for "found but could not start".
pub const EXEC_FAILED_EXIT_CODE: i32 = 126;

/// The shell convention for "command not found".
pub const NOT_FOUND_EXIT_CODE: i32 = 127;

/// The exit code of a refused or failed install step.
pub const INSTALL_FAILED_EXIT_CODE: i32 = 1;

/// The environment variable that names the per-user runtime directory,
/// where the notice handshake leaves its marker files.
pub const RUNTIME_DIR_VARIABLE: &str = "XDG_RUNTIME_DIR";

/// The start of the name of a notice marker;
/// the process id of the instance the marker is for follows it.
pub const NOTICE_MARKER_PREFIX: &str = "cargo-shadow-notice-";

/// A child killed by signal `n` exits as `128 + n` in a shell,
/// and the shadow reports it the same way.
pub const SIGNAL_EXIT_CODE_OFFSET: i32 = 128;
