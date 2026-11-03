//! What:
//!  What a worktree repair tells the person: one line when the repairs worked, and one plain line per repair that could not
//!  finish, each naming the command that finishes it.
//! Why:
//!  A repair never fails the add. The person must still learn that a worktree needs one more command, and which, so a
//!  failure is always printed (even under `--quiet`, which silences only the report of success).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.stderr.write(render(destination, findings, quiet));
//! ```

use std::path::Path;

/// What:
///  The tool a finding is about.
/// Why:
///   The success line names the tools that did work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Tool = 'pnpm' | 'cargo' | 'toolchain';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tool {
    /// pnpm relinked `node_modules`.
    Pnpm,
    /// Cargo build state was cloned and the worktree's own crates were cleaned.
    Cargo,
    /// mpm-shadow rebuilt the worktree's toolchain links and launchers.
    Toolchain,
}

/// What:
///  How one repair ended.
/// Why:
///   A success carries what was done and how long it took; a failure carries the reason and the command that finishes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = { kind: 'repaired'; detail: string; millis: number } | { kind: 'failed'; reason: string; runInstead: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The repair finished.
    Repaired {
        /// What was done, in words that follow the tool name.
        detail: String,
        /// Wall time in milliseconds.
        millis: u128,
    },
    /// The repair could not finish; the worktree is usable but still holds the source's state for this tool.
    Failed {
        /// Why, in a phrase.
        reason: String,
        /// The command that finishes it.
        run_instead: String,
    },
}

/// What:
///  One tool's outcome.
/// Why:
///   The orchestrator collects them and the renderer words them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Finding = { tool: Tool; outcome: Outcome };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    /// The tool.
    pub tool: Tool,
    /// How its repair ended.
    pub outcome: Outcome,
}

/// What:
///  The lines to print for one worktree: failures always, successes unless quiet.
/// Why:
///   Under `--quiet` the add prints nothing it can leave out, but a worktree that still needs a command is never silent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function render(destination: string, findings: Finding[], quiet: boolean): string;
/// ```
pub fn render(_destination: &Path, _findings: &[Finding], _quiet: bool) -> String {
    return String::new();
}

/// Report controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_report_tests.rs"]
mod tests;
