//! What:
//! What one mechanism of a run reports and prints,
//! and what the run hands it when it is reached.
//! Why:
//! A run of hours shows each mechanism's result when that mechanism ends,
//! not only at the very end,
//! and every mechanism reads the same deadline,
//! checkouts and recorded positions.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type MechanismReport = { lines: string[]; resumeAfter: string | undefined };
//! function say(summary: string[], lines: string[]): void
//! type RunContext = { options: RunOptions; deadline: Deadline; environment: GuardEnvironment; checkouts: string[]; pinned: Set<string>; base: string | undefined; rustupHome: string; progress: Progress };
//! ```
/// Imports what the caller asked a run for.
use super::mechanisms_and_options::RunOptions;
/// Imports the environment the guards read.
use crate::lean_guard::GuardEnvironment;
/// Imports the deadline that covers the whole run.
use crate::lean_identity::Deadline;
/// Imports where the last run stopped in each long pass.
use crate::lean_progress::Progress;
/// Imports the set of toolchain names the checkouts pin.
use std::collections::HashSet;
/// Imports the borrowed and owned path types the report and the context carry.
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `pub struct MechanismReport` is what one mechanism of a run did.
/// Why:
/// A run needs its lines and,
/// for the two long passes,
/// where the next run continues.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MechanismReport {
    /// The lines the mechanism reports.
    pub lines: Vec<String>,
    /// The last item a pass judged,
    /// when the deadline stopped it before the end of the list it was given;
    /// nothing when the pass finished,
    /// so the next run starts at the front.
    pub resume_after: Option<PathBuf>,
}

/// What:
/// `pub(super) fn say(summary: &mut Vec<String>, lines: Vec<String>)` prints each line at once and keeps it.
/// Why:
/// A run of hours shows each mechanism's result when that mechanism ends,
/// not only at the very end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function say(summary: string[], lines: string[]): void
/// ```
pub(super) fn say(
    summary: &mut Vec<String>,
    lines: Vec<String>,
) {
    for line in lines {
        crate::lean_run_report::summary_line(&line);
        summary.push(line);
    }
}

/// What a mechanism is given when the run reaches it.
pub(crate) struct RunContext<'a> {
    /// What the caller asked for.
    pub(crate) options: &'a RunOptions,
    /// The run's deadline.
    pub(crate) deadline: Deadline,
    /// The real `/proc` the guards read.
    pub(crate) environment: &'a GuardEnvironment,
    /// The checkouts the run found.
    pub(crate) checkouts: &'a [PathBuf],
    /// The toolchains a checkout's lock pins.
    pub(crate) pinned: &'a HashSet<String>,
    /// The host build base,
    /// when the Cargo configuration has one.
    pub(crate) base: Option<&'a Path>,
    /// The rustup home the toolchains live in.
    pub(crate) rustup_home: &'a Path,
    /// Where the last run stopped in each long pass.
    pub(crate) progress: &'a Progress,
}
