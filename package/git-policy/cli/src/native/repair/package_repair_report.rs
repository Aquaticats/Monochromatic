//! What:
//!  What a worktree repair tells the person:
//! one line when the repairs worked,
//! and one plain line per repair that could not
//!  finish,
//! each naming the command that finishes it.
//! Why:
//!  A repair never fails the add.
//! The person must still learn that a worktree needs one more command,
//! and which,
//! so a
//!  failure is always printed (even under `--quiet`,
//! which silences only the report of success).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.stderr.write(render(destination, findings, quiet));
//! ```

use crate::package_repair_command::{ToolEnd, ToolRun};
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
///   A success carries what was done and how long it took;
/// a failure carries the reason and the command that finishes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = { kind: 'repaired'; detail: string; millis: number } | { kind: 'failed'; reason: string; runInstead: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The repair finished.
    Repaired {
        /// What was done,
        /// in words that follow the tool name.
        detail: String,
        /// Wall time in milliseconds.
        millis: u128,
    },
    /// The repair could not finish;
    /// the worktree is usable but still holds the source's state for this tool.
    Failed {
        /// Why,
        /// in a phrase.
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
///  The finding for one tool run:
/// a success worded by the caller,
/// or a failure whose reason is the run's own.
/// Why:
///   Every tool ends the same four ways,
/// so the wording of a timeout lives in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingFrom(tool: Tool, run: ToolRun, detail: string, runInstead: string): Finding;
/// ```
pub fn finding_from(tool: Tool, run: &ToolRun, detail: &str, run_instead: &str) -> Finding {
    let outcome: Outcome = match &run.end {
        ToolEnd::Succeeded => Outcome::Repaired { detail: detail.to_owned(), millis: run.millis },
        ToolEnd::Failed(reason) => Outcome::Failed { reason: reason.clone(), run_instead: run_instead.to_owned() },
        ToolEnd::TimedOut => Outcome::Failed { reason: String::from("it showed no sign of work for the quiet window and was stopped"), run_instead: run_instead.to_owned() },
        ToolEnd::CeilingReached(ceiling) => Outcome::Failed { reason: ceiling_reason(tool, *ceiling), run_instead: run_instead.to_owned() },
    };
    return Finding { tool, outcome };
}

/// What:
///  The reason of a tool that was stopped at the ceiling:
/// the tool,
/// what it was doing,
/// how long it ran,
/// and that its whole process
///  group was stopped.
/// Why:
///   The owner,
/// 2026-11-04,
/// "Silence plus a ceiling (Recommended)":
/// the tool is "ended with an error naming the tool".
/// The command that
///  finishes the repair is the finding's `run_instead`,
/// which the line prints after this reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ceilingReason(tool: Tool, ceilingMs: number): string;
/// ```
pub fn ceiling_reason(tool: Tool, ceiling: std::time::Duration) -> String {
    let doing: &str = match tool {
        Tool::Pnpm => "pnpm, relinking node_modules",
        Tool::Cargo => "cargo, cloning and cleaning build state",
        Tool::Toolchain => "mpm-shadow, rebuilding the toolchain links",
    };
    return format!(
        "{doing}, ran for {} seconds, the longest the wrapper lets one repair tool run, and was stopped with its whole process group; if the repository is very large, run it by hand",
        ceiling.as_secs()
    );
}

/// What:
///  The finding for a tool the search path does not provide.
/// Why:
///   A missing tool is a verdict with the command to run once it is installed,
/// never an error of the add.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingTool(tool: Tool, program: string, runInstead: string): Finding;
/// ```
pub fn missing_tool(tool: Tool, program: &str, run_instead: &str) -> Finding {
    return Finding { tool, outcome: Outcome::Failed { reason: format!("{program} was not found on the search path"), run_instead: run_instead.to_owned() } };
}

/// What:
///  A time in milliseconds as seconds with one decimal.
/// Why:
///   The person reads a repair's time in seconds;
/// one decimal is the precision the measurements are made to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function seconds(millis: number): string; // "2.1 s"
/// ```
fn seconds(millis: u128) -> String {
    let tenths: u128 = millis / 100;
    return format!("{}.{} s", tenths / 10, tenths % 10);
}

/// What:
///  The text on one line.
/// Why:
///   A tool's output tail spans lines,
/// and each failure is one line the person can act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.split(/\s+/).join(' ');
/// ```
fn single_line(text: &str) -> String {
    return text.split_whitespace().collect::<Vec<&str>>().join(" ");
}

/// What:
///  The lines to print for one worktree:
/// failures always,
/// successes unless quiet.
/// Why:
///   Under `--quiet` the add prints nothing it can leave out,
/// but a worktree that still needs a command is never silent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function render(destination: string, findings: Finding[], quiet: boolean): string;
/// ```
pub fn render(destination: &Path, findings: &[Finding], quiet: bool) -> String {
    let shown: String = destination.display().to_string();
    let mut text: String = String::new();
    let mut worked: Vec<String> = Vec::new();
    let mut partial: bool = false;
    for finding in findings {
        match &finding.outcome {
            Outcome::Repaired { detail, millis } => worked.push(format!("{detail} in {}", seconds(*millis))),
            Outcome::Failed { .. } => partial = true,
        }
    }
    if !worked.is_empty() && !quiet {
        let scope: &str = if partial { "part of the package-manager state" } else { "the package-manager state" };
        text.push_str(format!("monochromatic-git-policy-cli: made {scope} of {shown} work at its new location: {}.\n", worked.join("; ")).as_str());
    }
    for finding in findings {
        if let Outcome::Failed { reason, run_instead } = &finding.outcome {
            text.push_str(format!("monochromatic-git-policy-cli: the worktree {shown} still needs one step ({}); run {}.\n", single_line(reason), single_line(run_instead)).as_str());
        }
    }
    return text;
}

/// Report controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_report_tests.rs"]
mod tests;
