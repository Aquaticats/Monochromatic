//! What:
//!  Starting one package-manager program for a worktree repair: finding it on a search path, running it in a directory with
//!  its output captured and a time limit, and saying how it ended.
//! Why:
//!  The repairs of a new worktree (`package_repair.rs`) start tools the repository uses (pnpm, Cargo, mpm-shadow). Each one
//!  must end in a plain verdict the add can word for the person: the program is missing, it failed with a reason, it ran
//!  past its limit, or it worked. Nothing here decides which tool runs or what its output means.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const program = findProgram('pnpm', process.env.PATH);
//! const run = await runTool(program, { arguments: ['install'], directory, set: [] }, limit);
//! ```

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// What:
///  One program start: its arguments, the directory it runs in, and the variables added to its environment.
/// Why:
///   The program path is found separately, so a test can name a fixture and a person's search path is read once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolCommand = { arguments: string[]; directory: string; set: [string, string][] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCommand {
    /// The arguments after the program name.
    pub arguments: Vec<OsString>,
    /// The working directory.
    pub directory: PathBuf,
    /// Variables added to the inherited environment.
    pub set: Vec<(OsString, OsString)>,
}

/// What:
///  How a started program ended.
/// Why:
///   The repair words each case differently, so they stay apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolEnd = { kind: 'succeeded' } | { kind: 'failed'; reason: string } | { kind: 'timed-out' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolEnd {
    /// Exit status zero.
    Succeeded,
    /// The program did not start, or ended with a failure status; the text names the status and the tail of its output.
    Failed(String),
    /// The program was still running when its limit passed, and was stopped.
    TimedOut,
}

/// What:
///  The end of one run and how long it took.
/// Why:
///   The time is reported to the person and measured by the docs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolRun = { end: ToolEnd; millis: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolRun {
    /// How the program ended.
    pub end: ToolEnd,
    /// Wall time from start to end, in milliseconds.
    pub millis: u128,
}

/// What:
///  The first executable file named `name` in a search path.
/// Why:
///   The program is the one the person's environment resolves; a missing one is a verdict, not a failure to start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findProgram(name: string, searchPath?: string): string | undefined;
/// ```
pub fn find_program(_name: &str, _search_path: Option<&OsStr>) -> Option<PathBuf> {
    return None;
}

/// What:
///  Run a program to its end or its limit.
/// Why:
///   The add must never wait without bound on a tool, and must keep the tail of its output for the log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runTool(program: string, command: ToolCommand, limit: number): Promise<ToolRun>;
/// ```
pub fn run_tool(_program: &Path, _command: &ToolCommand, _limit: Duration) -> ToolRun {
    return ToolRun { end: ToolEnd::Failed(String::from("not written yet")), millis: 0 };
}

/// Command controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_command_tests.rs"]
mod tests;
