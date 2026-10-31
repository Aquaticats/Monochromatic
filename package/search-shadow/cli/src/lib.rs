//! What:
//! The crate root:
//! it lists the submodules of the `search-shadow` library.
//! The library holds all the logic (policy / PATH lookup / launching / linking).
//! `main.rs` only decides which mode to start.
//! Why:
//! Keeping the logic in a library lets the unit tests call it without starting a process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // index.ts
//! export * from './cli.ts';
//! export * from './constants.ts';
//! // ...one line per module...
//! ```

/// What:
/// `pub mod cli;` declares a public submodule named `cli` whose code is in `src/cli.rs`.
/// `pub` makes it visible to code outside the crate
/// (here `main.rs` and the integration tests).
/// Why:
/// `main.rs` calls into it for the `search-shadow` command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './cli.ts';
/// ```
pub mod cli;

/// The line and section a shadowed tool adds after its own `--version` and `--help` output.
pub mod announce;

/// The fixed values:
/// the escape-hatch variable,
/// the exit codes and the shadowed names.
pub mod constants;

/// The one place that replaces the running process with the real tool.
pub mod launch;

/// The `link` subcommand:
/// creating the symlinks named `rg` / `grep` / `find` and so on.
pub mod link;

/// The rules that refuse `rg -r` and repository-wide unrestricted ripgrep walks.
pub mod policy;

/// The refusal rule for a recursive `grep` from a repository root,
/// the home directory or `/`.
pub mod policy_grep;

/// The refusal rule for a `find` or `bfs` walk from those places without a small `-maxdepth`.
pub mod policy_find;

/// The one `unsafe` call:
/// lowering the process's disk priority.
pub mod priority;

/// Where a tree walk would start:
/// a repository root,
/// the home directory,
/// `/` or something bounded.
pub mod walk_target;

/// The PATH lookup that finds the real tool while skipping the shadow itself.
pub mod real_binary;

/// The shadow mode:
/// policy check,
/// then lookup,
/// then launch.
pub mod shim;

/// What:
/// `#[cfg(test)] mod real_binary_tests;` declares a submodule that exists only when compiling tests.
/// `#[cfg(test)]` is an attribute (a compiler instruction attached to the next line)
/// meaning "include this only in test builds".
/// The code is in `src/real_binary_tests.rs`.
/// Why:
/// The tests stay in their own file,
/// out of the production module,
/// and cost nothing in release builds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // real-binary.unit.test.ts sits beside real-binary.ts and is excluded from the build output.
/// ```
#[cfg(test)]
mod real_binary_tests;

/// The unit tests of the version and help announcements.
#[cfg(test)]
mod announce_tests;

/// The unit tests of the policy module.
#[cfg(test)]
mod policy_tests;

/// The unit tests of the link module.
#[cfg(test)]
mod link_tests;

/// The unit tests of the walk-target classifier.
#[cfg(test)]
mod walk_target_tests;

/// The unit tests of the grep and find refusal rules.
#[cfg(test)]
mod policy_walk_tests;

/// Scratch directories for the unit tests,
/// placed beside the build output.
#[cfg(test)]
mod test_scratch;
