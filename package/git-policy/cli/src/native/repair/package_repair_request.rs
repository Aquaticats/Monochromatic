//! What:
//!  What a repair of a new worktree reads and is asked:
//! the person's switches,
//! what the process offers,
//! and the request
//!  that holds both with the paths of the repair.
//! Why:
//!  The repair and each tool's repair (pnpm,
//! Cargo,
//! the toolchain) all read these,
//! and the repair imports the tools;
//! the
//!  shared types live in a file that imports none of them,
//! so no two of these files use each other in a loop.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type RepairRequest = { git: GitContext; source: string; destination: string; switch: RepairSwitch; environment: RepairEnvironment };
//! ```

/// The two bounds one tool runs under.
use crate::package_repair_command::ToolLimits;
/// The real Git and its overlay.
use crate::transaction_git::GitContext;
/// Operating-system text,
/// and its borrowed form.
use std::ffi::{OsStr, OsString};
/// A borrowed filesystem path.
use std::path::Path;
/// A length of time.
use std::time::Duration;

/// The options of `worktree add` whose value is the next argument.
const VALUE_OPTIONS: [&str; 3] = ["-b", "-B", "--reason"];

/// How long one tool may show no sign of work before it is stopped;
/// a hung tool must not hold an add forever,
/// and a tool that
///  works (an install that downloads for a long time) is never stopped for being slow.
///  Citation:
/// none,
/// and **unmeasured as a quiet window**.
/// The figure was the wall-clock limit of a tool,
/// and no log held a quiet
///  gap before `quiet_ledger.rs`;
/// the add timing record now carries the longest gap of each tool under `quiet`
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Package repair tool limit,
/// 600 seconds").
/// Owner pick 9 of that audit,
///  the coordinator's under the owner's delegation and open to the owner's veto:
/// end only on no progress.
const TOOL_QUIET_WINDOW: Duration = Duration::from_secs(600);

/// What:
///  The switches of the person's `git worktree add` that change what a repair may do.
/// Why:
///   `--quiet` silences the report of success,
/// and `--no-checkout` asks for a worktree with no files,
/// which no tool can
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

/// Whether a letter of a short-option cluster is `b` or `B`,
/// the options of `worktree add` that take a branch name.
///  `&u8` borrows the letter,
/// as `position` hands it over.
fn is_branch_letter(flag: &u8) -> bool {
    return *flag == b'b' || *flag == b'B';
}

impl RepairSwitch {
    /// What:
    ///  Read the switches from the arguments of a Git command.
    /// Why:
    ///   The ladder's plan declines forms it does not know (such as `--no-checkout`),
    /// so the switches are read from the
    ///  arguments themselves:
    /// only the words after `worktree add`,
    /// up to a lone `--`,
    /// and never the value of an option.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function fromArguments(args: string[]): RepairSwitch;
    /// ```
    pub fn from_arguments(arguments: &[OsString]) -> RepairSwitch {
        let mut found: RepairSwitch = RepairSwitch::default();
        let mut add_position: Option<usize> = None;
        for (position, pair) in arguments.windows(2).enumerate() {
            if pair[0] == "worktree" && pair[1] == "add" {
                add_position = Some(position);
                break;
            }
        }
        let Some(add_at) = add_position else { return found };
        let mut words = arguments[add_at + 2..].iter();
        while let Some(word) = words.next() {
            let bytes: &[u8] = word.as_encoded_bytes();
            if bytes == b"--" {
                break;
            }
            let mut takes_value: bool = false;
            for option in VALUE_OPTIONS {
                if bytes == option.as_bytes() {
                    takes_value = true;
                    break;
                }
            }
            if takes_value {
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
                let end: usize = match flags.iter().position(is_branch_letter) {
                    Some(at) => at,
                    None => flags.len(),
                };
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
///   The repair of each Cargo workspace and each tool touches state of its own,
/// so the host's whole budget is the limit (the
///  standard library reads it from the CPU quota of the control group,
/// so a capped session gets its own share);
/// a host that
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
///  What a repair reads from the process:
/// the search path tools are found on,
/// how long a tool may be quiet,
/// and how many threads
///  the repairs may use.
/// Why:
///   Tests give fixtures instead of the real environment,
/// and the wrapper never reads the process more than once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RepairEnvironment = { searchPath?: string; quietWindow: number; workers: number };
/// ```
#[derive(Clone, Debug)]
pub struct RepairEnvironment {
    /// The search path programs are found on;
    /// the person's `PATH`.
    pub search_path: Option<OsString>,
    /// How long one tool may show no sign of work before it is stopped.
    pub quiet_window: Duration,
    /// The longest one tool runs,
    /// whatever it shows (`wait_ceilings.rs`,
    /// `REPAIR_TOOL_CEILING`).
    pub ceiling: Duration,
    /// The most threads the repairs use together.
    pub workers: usize,
}

impl RepairEnvironment {
    /// What:
    ///  The environment of this process.
    /// Why:
    ///   The person's environment decides which pnpm,
    /// Cargo and mpm-shadow run.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// RepairEnvironment.fromProcess();
    /// ```
    pub fn from_process() -> RepairEnvironment {
        let available: usize = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        return RepairEnvironment { search_path: std::env::var_os("PATH"), quiet_window: TOOL_QUIET_WINDOW, ceiling: crate::wait_ceilings::REPAIR_TOOL_CEILING, workers: workers_for(available) };
    }

    /// The two bounds one tool runs under.
    pub fn limits(&self) -> ToolLimits {
        return ToolLimits { window: self.quiet_window, ceiling: self.ceiling };
    }

    /// The search path as a borrowed value.
    pub fn path(&self) -> Option<&OsStr> {
        return self.search_path.as_deref();
    }
}

/// What:
///  The variables a tool's own children read:
/// the search path the add resolved the program on.
/// Why:
///   A repair's tool is a program that starts others (in this repository the pnpm-shadow runs the real pnpm,
/// and mpm-shadow runs `pnpm` and `cargo`),
/// and the owner decided the repair starts what the person's path names.
/// A service runs on its own path,
/// which may hold another pnpm or none,
/// so the recorded path is handed to the tool itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withThePersonsPath = (environment) => environment.searchPath === undefined ? [] : [['PATH', environment.searchPath]];
/// ```
pub fn with_the_persons_path(environment: &RepairEnvironment) -> Vec<(OsString, OsString)> {
    return match environment.path() {
        None => Vec::new(),
        Some(search_path) => vec![(OsString::from(PATH_VARIABLE), search_path.to_os_string())],
    };
}

/// What:
///  The variable a tool's children find their programs on.
const PATH_VARIABLE: &str = "PATH";

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
    /// The real Git and its overlay,
    /// used to list the worktree's tracked files.
    pub git: &'a GitContext,
    /// The checkout the ignored files came from;
    /// never written.
    pub source: &'a Path,
    /// The new worktree.
    pub destination: &'a Path,
    /// The person's switches.
    pub switch: RepairSwitch,
    /// What the repairs read from the process.
    pub environment: &'a RepairEnvironment,
}
