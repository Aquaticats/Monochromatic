//! What:
//! The crate root: it lists the submodules of the `cargo-shadow` library. The library holds all the logic; `main.rs`
//! only decides which mode to start.
//! Why:
//! Keeping the logic in a library lets the unit tests call it without starting a process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // index.ts
//! export * from './cli.ts';
//! // ...one line per module...
//! ```

#[cfg(not(unix))]
compile_error!("cargo-shadow supports Unix only; a cross-platform version is a later change");

/// The command-line grammar of the binary when it is started under its own name.
pub mod cli;

/// The fixed values (shadowed name, file names, exit codes).
pub mod constants;

/// The loop guard that keeps a launcher or link leading back to the shadow from being run as the real `cargo`.
pub mod guard;

/// The marker file that tells an outer shadow instance that an inner one already printed the notice.
pub mod notice_marker;

/// The PATH lookup that finds the real `cargo` while skipping the shadow itself.
pub mod real_binary;

/// The line and the section the shadow adds after cargo's `--version` and `--help`, the only places it names itself.
pub mod shadow_notice;

/// The shadow mode: install step, look up, launch, wait.
pub mod shim;

/// The command-word rules: what the shadow does for a cargo command line.
pub mod verb;

/// Scratch directories for the unit tests, placed beside the build output.
#[cfg(test)]
mod test_scratch;

#[cfg(test)]
mod guard_tests;

#[cfg(test)]
mod notice_marker_tests;

#[cfg(test)]
mod real_binary_tests;

#[cfg(test)]
mod shadow_notice_tests;

#[cfg(test)]
mod verb_tests;
