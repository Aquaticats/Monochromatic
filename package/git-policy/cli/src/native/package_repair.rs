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

use super::add_timing::work;
use super::diagnostic_log::debug;
use super::package_repair_cargo::repair_cargo;
use super::package_repair_pnpm::repair_pnpm;
use super::package_repair_report::{Finding, render};
use super::package_repair_toolchain::repair_toolchain;
use super::transaction_git::GitContext;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepair";

/// The options of `worktree add` whose value is the next argument.
const VALUE_OPTIONS: [&str; 3] = ["-b", "-B", "--reason"];

/// How long one tool may show no sign of work before it is stopped; a hung tool must not hold an add forever, and a tool that
///  works (an install that downloads for a long time) is never stopped for being slow.
///  Citation: none, and **unmeasured as a quiet window**. The figure was the wall-clock limit of a tool, and no log held a quiet
///  gap before `quiet_ledger.rs`; the add timing record now carries the longest gap of each tool under `quiet`
///  (`doc/planning/cli-git-shipped-limits-audit.md`, "Package repair tool limit, 600 seconds"). Owner pick 9 of that audit,
///  the coordinator's under the owner's delegation and open to the owner's veto: end only on no progress.
const TOOL_QUIET_WINDOW: Duration = Duration::from_secs(600);

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
    pub fn from_arguments(arguments: &[OsString]) -> RepairSwitch {
        let mut found: RepairSwitch = RepairSwitch::default();
        let Some(add_at) = arguments.windows(2).position(|pair: &[OsString]| { return pair[0] == "worktree" && pair[1] == "add"; }) else { return found };
        let mut words = arguments[add_at + 2..].iter();
        while let Some(word) = words.next() {
            let bytes: &[u8] = word.as_encoded_bytes();
            if bytes == b"--" {
                break;
            }
            if VALUE_OPTIONS.iter().any(|option: &&str| { return bytes == option.as_bytes(); }) {
                words.next();
            } else if bytes == b"--quiet" {
                found.quiet = true;
            } else if bytes == b"--no-checkout" {
                found.no_checkout = true;
            } else if bytes == b"--checkout" {
                found.no_checkout = false;
            } else if bytes.len() > 1 && bytes[0] == b'-' && bytes[1] != b'-' {
                // A bundle of short flags: `q` quiets, and the first flag that takes a value ends the bundle.
                let flags: &[u8] = &bytes[1..];
                let end: usize = flags.iter().position(|flag: &u8| { return *flag == b'b' || *flag == b'B'; }).unwrap_or(flags.len());
                found.quiet |= flags[..end].contains(&b'q');
                if end + 1 == flags.len() {
                    words.next();
                }
            }
        }
        return found;
    }
}

/// What:
///  How many threads the repairs use together on a host that offers `available`.
/// Why:
///   The repair of each Cargo workspace and each tool touches state of its own, so the host's whole budget is the limit (the
///  standard library reads it from the CPU quota of the control group, so a capped session gets its own share); a host that
///  reports nothing still runs one worker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// workersFor(navigator.hardwareConcurrency)
/// ```
pub fn workers_for(available: usize) -> usize {
    return available.max(1);
}

/// What:
///  What a repair reads from the process: the search path tools are found on, how long a tool may be quiet, and how many threads
///  the repairs may use.
/// Why:
///   Tests give fixtures instead of the real environment, and the wrapper never reads the process more than once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RepairEnvironment = { searchPath?: string; quietWindow: number; workers: number };
/// ```
#[derive(Clone, Debug)]
pub struct RepairEnvironment {
    /// The search path programs are found on; the person's `PATH`.
    pub search_path: Option<OsString>,
    /// How long one tool may show no sign of work before it is stopped.
    pub quiet_window: Duration,
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
        let available: usize = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        return RepairEnvironment { search_path: std::env::var_os("PATH"), quiet_window: TOOL_QUIET_WINDOW, workers: workers_for(available) };
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
pub fn repair_worktree(request: &RepairRequest<'_>, notices: &mut dyn Write) -> Vec<Finding> {
    if request.switch.no_checkout {
        debug(TAG, format!("{}: --no-checkout, so there is nothing to repair", request.destination.display()).as_str());
        return Vec::new();
    }
    let started: std::time::Instant = std::time::Instant::now();
    let (pnpm, cargo, toolchain): (Option<Finding>, Vec<Finding>, Option<Finding>) = std::thread::scope(|scope| {
        let pnpm = scope.spawn(|| { return work("repairPnpm", || { return repair_pnpm(request.destination, request.environment); }); });
        let toolchain = scope.spawn(|| { return work("repairToolchain", || { return repair_toolchain(request.destination, request.environment); }); });
        let cargo: Vec<Finding> = work("repairCargo", || { return repair_cargo(request); });
        // A tool that panicked has no finding; the add goes on and the worktree stays as the copy left it.
        let settled = |joined: std::thread::Result<Option<Finding>>, tool: &str| -> Option<Finding> {
            return joined.unwrap_or_else(|_panic| {
                debug(TAG, format!("the {tool} repair stopped unexpectedly and reports nothing").as_str());
                return None;
            });
        };
        return (settled(pnpm.join(), "pnpm"), cargo, settled(toolchain.join(), "toolchain"));
    });
    let findings: Vec<Finding> = pnpm.into_iter().chain(cargo).chain(toolchain).collect();
    debug(TAG, format!("{}: {} findings in {} ms", request.destination.display(), findings.len(), started.elapsed().as_millis()).as_str());
    let text: String = render(request.destination, findings.as_slice(), request.switch.quiet);
    // A closed standard error leaves nowhere to say it; the worktree is whole either way.
    let _written = notices.write_all(text.as_bytes());
    return findings;
}

/// Repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_tests.rs"]
mod tests;
