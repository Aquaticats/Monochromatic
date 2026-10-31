//! What:
//! The crate root:
//! it lists the submodules of the `mpm-shadow` library.
//! The library holds all the logic
//!           (version parsing,
//! the registry scan,
//! the prune rule,
//! PATH lookup,
//! launching);
//! `main.rs` only decides which
//!           mode to start.
//! Why:
//! Keeping the logic in a library lets the unit tests call it without starting a process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // index.ts
//! export * from './cli.ts';
//! // ...one line per module...
//! ```

/// What:
/// `pub mod cli;` declares a public submodule named `cli` whose code is in `src/cli.rs`.
/// `pub` makes it
///           visible to code outside the crate (here `main.rs` and the integration tests).
/// Why:
/// `main.rs` calls into it for the `mpm-shadow` command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './cli.ts';
/// ```
pub mod cli;

/// Cargo's home directory from the environment.
pub mod cargo_home;

/// The fixed values (shadowed name,
/// crate prefix,
/// exit codes).
pub mod constants;

/// Cargo's install record:
/// which version of each crate is installed.
pub mod installed;

/// The prune rule and the deletion.
pub mod prune;

/// The PATH lookup that finds the real `mpm` while skipping the shadow itself.
pub mod real_binary;

/// The cached archives and extracted sources of this repository's own crates.
pub mod registry;

/// The shadow mode:
/// lookup,
/// launch,
/// wait,
/// prune.
pub mod shim;

/// The toolchain plan and lock,
/// and the commands that act on them.
pub mod toolchain;

/// Crate-name and version splitting,
/// and version order.
pub mod version;

/// What:
/// `#[cfg(test)] mod version_tests;` declares a submodule that exists only when compiling tests.
///           `#[cfg(test)]` is an attribute (a compiler instruction attached to the next line) meaning "include this
///           only in test builds".
/// The code is in `src/version_tests.rs`.
/// Why:
/// The tests stay in their own file,
/// out of the production module,
/// and cost nothing in release builds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // version.unit.test.ts sits beside version.ts and is excluded from the build output.
/// ```
#[cfg(test)]
mod version_tests;

/// The unit tests of the prune rule and the deletion.
#[cfg(test)]
mod prune_tests;

/// The unit tests of the PATH lookup.
#[cfg(test)]
mod real_binary_tests;

/// The unit tests of the child launch and exit-status mapping.
#[cfg(test)]
mod shim_tests;

/// The unit tests of the cargo home lookup.
#[cfg(test)]
mod cargo_home_tests;

/// Scratch directories for the unit tests,
/// placed beside the build output.
#[cfg(test)]
mod test_scratch;
