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

/// Release-age and trust checks over newly locked versions.
pub mod checks;

/// Runs the real cargo for the install and update steps.
pub mod cargo_run;

/// Finds the crate roots that carry a lockfile, and snapshots and restores them.
pub mod crates;

/// The one HTTP GET the registry lookups need.
pub mod http;

/// The loop guard that keeps a launcher or link leading back to the shadow from being run as the real `cargo`.
pub mod guard;

/// Registry versions in a lockfile and the difference between lockfile sets.
pub mod lockfile;

/// The release-age and trust rules.
pub mod policy;

/// The install step, the counterpart of `pnpm install`.
pub mod install;

/// The shadow's install step: fingerprint fast path, then the install over one workspace.
pub mod install_step;

/// Reads the few cargo options the shadow cares about.
pub mod options;

/// The marker file that tells an outer shadow instance that an inner one already printed the notice.
pub mod notice_marker;

/// The crates.io lookups.
pub mod registry;

/// The PATH lookup that finds the real `cargo` while skipping the shadow itself.
pub mod real_binary;

/// The selector syntax of the catalog's exclude lists.
pub mod selectors;

/// The line and the section the shadow adds after cargo's `--version` and `--help`, the only places it names itself.
pub mod shadow_notice;

/// The shadow mode: install step, look up, launch, wait.
pub mod shim;

/// Timestamps from the registry.
pub mod time_text;

/// The rustup toolchain the repository pins.
pub mod toolchain;

/// Finds the catalog repository and the Cargo workspace of a directory.
pub mod workspace;

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
mod install_step_tests;

#[cfg(test)]
mod install_tests;

#[cfg(test)]
mod lockfile_tests;

#[cfg(test)]
mod options_tests;

#[cfg(test)]
mod policy_tests;

#[cfg(test)]
mod selectors_tests;

#[cfg(test)]
mod time_text_tests;

#[cfg(test)]
mod toolchain_tests;

#[cfg(test)]
mod verb_tests;

#[cfg(test)]
mod workspace_tests;
