//! What:
//!  What a clone of one directory tree finds and makes:
//! the entries a walk of the source names,
//! what each one is,
//! what a finished clone holds,
//! which of the process's places its steps take,
//! and the two words a failed step is said in.
//! Why:
//!  Every step of the clone names the entry it could not handle and reads one list of entries,
//! so the entries and the places they are placed in are one vocabulary of this folder,
//! and the steps that walk,
//! place and set modes share it without any of them owning it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Kind = 'directory' | 'file' | 'link';
//! type Entry = { relative: string; kind: Kind };
//! type Cloned = { made: number; leftOut: string[] };
//! type PlacesOfAClone = 'everyJob' | 'slowWork';
//! const PARALLEL_MIN_ENTRIES = PARALLEL_BREAK_EVEN_ENTRIES;
//! const sayWhere = (attempt, path, what) => value; // throws `${path} ${what} (${error})`
//! const runACloneStep = (places, state, work) => runBlockingJobWithinTheBudget(state, work);
//! ```

/// Import the refusal a job of the blocking pool answers with.
use crate::blocking_job::BlockingJobError;
/// Import the two runners that hold the places a clone's steps take.
use crate::blocking_jobs_within_the_budget::{
    run_a_job_or_in_turn,
    run_a_slow_job_or_in_turn,
};
/// Import the borrowed and owned filesystem paths an entry is named by.
use std::path::{
    Path,
    PathBuf,
};
/// Which of the process's places a tree clone's steps take.
/// Why:
///  The clone is the heaviest step of a new worktree's prewarm,
/// which is work nobody waits for,
/// and the same function clones a build directory for a command a person waits for.
/// The two may not be covered by the same bound:
/// work nobody waits for leaves one place of the process's bound free of itself
/// (`SLOW_WORK_RESERVE` in `blocking_jobs_within_the_budget.rs`),
/// and work a person waits for takes its place beside everything else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlacesOfAClone = 'everyJob' | 'slowWork';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlacesOfAClone {
    /// The process's general places,
    /// which every other job of the process shares.
    EveryJob,
    /// The places slow work may take,
    /// which leave one place of the process's bound free of it.
    SlowWork,
}

/// What:
///  Run one step of a clone as a job of the blocking pool,
///  with the places `places` names held for as long as it runs.
/// Why:
///   The walk,
/// each slice of files and the modes are the same shape of job,
/// and only the places they take differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await runACloneStep(places, state, work);
/// ```
pub(super) async fn run_a_clone_step<S: Clone + Send + 'static, T: Send + 'static>(
    places: PlacesOfAClone,
    state: S,
    work: fn(S) -> T,
) -> Result<T, BlockingJobError> {
    return match places {
        PlacesOfAClone::EveryJob => run_a_job_or_in_turn(state, work).await,
        PlacesOfAClone::SlowWork => run_a_slow_job_or_in_turn(state, work).await,
    };
}

/// What a walk of the source finds at one path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Kind {
    /// A directory,
    /// recreated and entered.
    Directory,
    /// A regular file,
    /// cloned.
    File,
    /// A symbolic link,
    /// recreated with its exact target.
    Link,
}

/// One entry of the source,
/// relative to its root.
pub(super) struct Entry {
    /// The path below the root.
    pub(super) relative: PathBuf,
    /// What it is.
    pub(super) kind: Kind,
}

/// The entries below which a clone stays on one thread,
/// `PARALLEL_BREAK_EVEN_ENTRIES`;
/// starting threads costs more than a very small
///  clone.
/// Measured with `clone_tree` itself forced to 1,
/// 2,
/// 4 and 8 workers (2026-11-03,
/// five runs each in alternating orders,
///  other sessions running,
/// a debug build):
/// 0.6 ms on every count for 10 entries,
/// 1.2 ms on 8 workers against 1.8 ms on one for 30,
///  8.1 ms against 20.7 ms for 296,
/// and 13.4 ms against 39.2 ms for 555,
/// so the old threshold of 512 left clones of up to 511 leaves
///  on one thread for up to 12 ms more.
/// Picked by the coordinator under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// option A).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARALLEL_MIN_ENTRIES = PARALLEL_BREAK_EVEN_ENTRIES;
/// ```
pub(super) const PARALLEL_MIN_ENTRIES: usize = crate::parallel_budget::PARALLEL_BREAK_EVEN_ENTRIES;

/// What a clone made and what it left out.
#[derive(Debug, Eq, PartialEq)]
pub struct Cloned {
    /// The entries of the source that the destination now holds.
    pub made: usize,
    /// The paths below the source that could not be read and so are not in the destination.
    pub left_out: Vec<PathBuf>,
}

/// What:
///  Say which path an input or output step failed on:
/// `<path> <what> (<error>)`.
///  `Result<T, std::io::Error>` is "the value,
/// or an input/output error".
/// Why:
///   Every step of the clone names the entry it could not handle,
/// and a named function stands where a closure that
///  built that text stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sayWhere<T>(attempt: () => T, path: string, what: string): T; // throws `${path} ${what} (${error})`
/// ```
pub(super) fn say_where<T>(
    result: Result<T, std::io::Error>,
    path: &Path,
    what: &str,
) -> Result<T, String> {
    return match result {
        Ok(value) => Ok(value),
        Err(error) => Err(format!("{} {what} ({error})", path.display())),
    };
}

/// Order two entries by their path below the source,
/// so parents sort before their children.
/// `sort_by` takes this
///  function by path;
/// the sort stays stable.
pub(super) fn compare_by_relative_path(
    left: &Entry,
    right: &Entry,
) -> std::cmp::Ordering {
    return left.relative.cmp(&right.relative);
}
