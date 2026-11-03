//! What:
//!  The repair of a new worktree's package-manager state: after a rung filled the worktree and the ignored files were
//!  copied, run each tool's own offline repair so the state it keeps for the old location works at the new one.
//! Why:
//!  A copy keeps links and build state exactly as they were. pnpm's links climb out of the tree by a fixed number of
//!  levels, Cargo's build state bakes in the source's paths, and the toolchain launchers name the source's lock. Each tool
//!  knows how to rebuild its own layout, so the wrapper starts it (the owner's exemption, 2026-11-03) and never edits
//!  its files itself. A repair never fails the add: it ends in a line that names the command to run, and a tool the
//!  environment does not provide is a verdict, not an error. Another process changing the files while they are copied or
//!  repaired is the person's risk and is not defended against.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! repairWorktree({ git, source, destination, switch: { quiet, noCheckout }, environment }, process.stderr);
//! ```

use super::package_repair_report::Finding;
use super::transaction_git::GitContext;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

/// What:
///  The switches of the person's `git worktree add` that change what a repair may do.
/// Why:
///   `--quiet` silences the report of success, and `--no-checkout` asks for a worktree with no files, which no tool can
///  repair and which the wrapper leaves exactly as Git made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RepairSwitch = { quiet: boolean; noCheckout: boolean };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RepairSwitch {
    /// Whether the person passed `--quiet` (or `-q`).
    pub quiet: bool,
    /// Whether the person passed `--no-checkout`.
    pub no_checkout: bool,
}

impl RepairSwitch {
    /// What:
    ///  Read the switches from the arguments of a Git command.
    /// Why:
    ///   The ladder's plan declines forms it does not know (such as `--no-checkout`), so the switches are read from the
    ///  arguments themselves: only the words after `worktree add`, up to a lone `--`, and never the value of an option.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function fromArguments(args: string[]): RepairSwitch;
    /// ```
    pub fn from_arguments(_arguments: &[OsString]) -> RepairSwitch {
        return RepairSwitch::default();
    }
}

/// What:
///  What a repair reads from the process: the search path tools are found on, how long a tool may run, and how many threads
///  the repairs may use.
/// Why:
///   Tests give fixtures instead of the real environment, and the wrapper never reads the process more than once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RepairEnvironment = { searchPath?: string; toolLimit: number; workers: number };
/// ```
#[derive(Clone, Debug)]
pub struct RepairEnvironment {
    /// The search path programs are found on; the person's `PATH`.
    pub search_path: Option<OsString>,
    /// The longest one tool may run before it is stopped.
    pub tool_limit: Duration,
    /// The most threads the repairs use together.
    pub workers: usize,
}

impl RepairEnvironment {
    /// What:
    ///  The environment of this process.
    /// Why:
    ///   The person's environment decides which pnpm, Cargo and mpm-shadow run.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// RepairEnvironment.fromProcess();
    /// ```
    pub fn from_process() -> RepairEnvironment {
        return RepairEnvironment { search_path: None, tool_limit: Duration::ZERO, workers: 1 };
    }

    /// The search path as a borrowed value.
    pub fn path(&self) -> Option<&OsStr> {
        return self.search_path.as_deref();
    }
}

/// What:
///  Everything one repair needs.
/// Why:
///   One request per new worktree keeps the call sites short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RepairRequest = { git: GitContext; source: string; destination: string; switch: RepairSwitch; environment: RepairEnvironment };
/// ```
pub struct RepairRequest<'a> {
    /// The real Git and its overlay, used to list the worktree's tracked files.
    pub git: &'a GitContext,
    /// The checkout the ignored files came from; never written.
    pub source: &'a Path,
    /// The new worktree.
    pub destination: &'a Path,
    /// The person's switches.
    pub switch: RepairSwitch,
    /// What the repairs read from the process.
    pub environment: &'a RepairEnvironment,
}

/// What:
///  Repair the package-manager state of one new worktree and print what the person needs to know.
/// Why:
///   The tools are independent, so they run side by side; the add ends when the slowest does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await repairWorktree(request, stderr);
/// ```
pub fn repair_worktree(_request: &RepairRequest<'_>, _notices: &mut dyn Write) -> Vec<Finding> {
    return Vec::new();
}

/// Repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_tests.rs"]
mod tests;
