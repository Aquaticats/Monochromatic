//! What:
//! The crate root:
//! it lists the submodules of the `pnpm-shadow` library.
//! The library holds all the logic (the
//! workspace scan,
//! the report,
//! PATH lookup,
//! launching);
//! `main.rs` only decides which mode to start.
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
/// `#[cfg(not(unix))] compile_error!(...)` stops the build on any platform that is not Unix.
/// Why:
/// The PATH separator,
/// the execute bits and the signal exit codes are Unix facts;
/// Windows is a later version,
/// and a
/// build that compiled there would run wrong instead of failing loudly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.platform === 'win32') throw new Error('pnpm-shadow supports Unix only');
/// ```
#[cfg(not(unix))]
compile_error!("pnpm-shadow supports Unix only; a cross-platform version is a later change");

/// What:
/// `pub mod cli;` declares a public submodule whose code is in `src/cli.rs`.
/// `pub` makes it visible to code outside
/// the crate (here `main.rs` and the integration tests).
/// Why:
/// `main.rs` calls into it for the `pnpm-shadow` command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './cli.ts';
/// ```
pub mod cli;

/// The fixed values (shadowed name,
/// file names,
/// exit codes).
pub mod constants;

/// The loop guard that keeps a launcher or link leading back to the shadow from being run as the real `pnpm`.
pub mod guard;

/// The marker file that tells an outer shadow instance that an inner one already printed the notice.
pub mod notice_marker;

/// The PATH lookup that finds the real `pnpm` while skipping the shadow itself.
pub mod real_binary;

/// The text the shadow writes to standard error.
pub mod report;

/// The workspace root lookup and the manifest walk.
pub mod scan;

/// The line and the section the shadow adds after pnpm's `--version` and `--help`,
/// the only places it names itself.
pub mod shadow_notice;

/// The shadow mode:
/// warn,
/// look up,
/// launch,
/// wait.
pub mod shim;

/// The command-word rules:
/// whether the manifest warning makes sense for a pnpm command line.
pub mod verb;

/// Scratch directories for the unit tests,
/// placed beside the build output.
#[cfg(test)]
mod test_scratch;

/// The unit tests of the loop guard.
#[cfg(test)]
mod guard_tests;

/// The unit tests of the notice marker.
#[cfg(test)]
mod notice_marker_tests;

/// The unit tests of the PATH lookup.
#[cfg(test)]
mod real_binary_tests;

/// The unit tests of the report text.
#[cfg(test)]
mod report_tests;

/// The unit tests of the workspace scan.
#[cfg(test)]
mod scan_tests;

/// The unit tests of the notice.
#[cfg(test)]
mod shadow_notice_tests;

/// The unit tests of the child launch and exit-status mapping.
#[cfg(test)]
mod shim_tests;

/// The unit tests of the command-word rules.
#[cfg(test)]
mod verb_tests;
