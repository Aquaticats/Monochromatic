//! What:
//!  The sweep of a stopped push's reference locks that the stop's bound did not let it finish:
//! recorded inside the
//!  repository's own Git directory,
//! and done by the next fetch,
//! push or service start of that repository.
//! Why:
//!  A bound that is reached ends the waiting and not the duty.
//! A stop answers inside `STOP_CEILING`,
//! and
//!  the sweep of the locks its push left is one of the jobs that bound pays for
//! (`push/stop_of_a_push_and_sweep_of_its_locks.rs`,
//!  `SWEEP_AFTER_THE_STOP`).
//! When the blocking pool cannot run that sweep inside the bound,
//! the locks stay as they were:
//! an
//!  orphan `packed-refs.new` a killed Git staged stays and is never renamed or removed,
//! and a reference lock that nothing proves
//!  stale stays and refuses every later update of that reference with "Unable to create ... File exists".
//! A line about it
//!  on a stream nobody reads is not a record (`OWED_SWEEPS` was exactly that: a list written and never read),
//! so
//!  the duty is written where the repository's own next reader finds it:
//! `<git dir>/monochromatic-git-policy-cli/owed-lock-sweep`.
//!  Who reads it:
//!  - the wrapper's own next fetch or push in that repository,
//! before it clears the stale locks it already judges
//!  (`lock/push_lock_clearing.rs`);
//!  - the service's next push or fetch of that repository,
//! which is the retry of the stopped push itself
//!  (`push/bounded_push.rs`,
//! `run_bounded_push_awaiting`);
//!  - the service's next start for that worktree (`service/service_main.rs`).
//!  Each of them judges the locks of that repository and removes the proof it finds,
//! and names with the `rm --` command the
//!  locks it will not remove;
//! the marker goes once that judgement has run,
//! because the judgement is the duty and the marker only
//!  carries it to the judgement.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! record(ledger, why);            // write the marker into each root's monochromatic-git-policy-cli folder
//! if (anyOwed(roots)) settle(roots, judge); // the next fetch, push or service start: judge, clear, drop the marker
//! ```

/// How a lock candidate is judged,
/// and the words for the ones left.
use crate::stale_ref_lock_judge::Judge;
/// The judgement and conditional removal of the reference locks below some Git directories.
use crate::stale_ref_locks::clear_stale_ref_locks;
/// What one clearing found.
pub use crate::stale_ref_lock_found::Outcome;
/// The lock files a stopped push leaves,
/// and the ledger that finds them.
use crate::stopped_git_locks::LockLedger;
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::warn;
/// The paths this record is written at and read from.
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// The folder inside a Git directory the wrapper keeps its own files in,
/// which `service_work` uses for its work directory as well.
pub const WRAPPER_FOLDER: &str = "monochromatic-git-policy-cli";

/// The name of the marker a stop writes when its sweep did not finish.
pub const MARKER: &str = "owed-lock-sweep";

/// What:
///  The marker path of each of `roots`.
/// Why:
///   The record belongs to the repository,
/// so it lives where every reader of that repository looks:
/// its Git directory
///  (and its common directory, which a worktree's locks may sit in).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const markersOf = (roots) => roots.map((root) => path.join(root, WRAPPER_FOLDER, MARKER));
/// ```
pub fn markers_of(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut markers: Vec<PathBuf> = Vec::new();
    for root in roots {
        markers.push(root.join(WRAPPER_FOLDER).join(MARKER));
    }
    return markers;
}

/// What:
///  The markers that are there among `roots`.
/// Why:
///   A reader pays three `stat` calls to learn whether a sweep is owed,
/// so the service's every push can ask without paying
///  for a walk of `refs/` it does not need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owedMarkers = (roots) => markersOf(roots).filter(exists);
/// ```
pub fn owed_markers_of(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut there: Vec<PathBuf> = Vec::new();
    for marker in markers_of(roots) {
        if marker.is_file() {
            there.push(marker);
        }
    }
    return there;
}

/// Whether any of `roots` has a sweep owing.
pub fn any_owed(roots: &[PathBuf]) -> bool {
    return !owed_markers_of(roots).is_empty();
}

/// What:
///  Record that the sweep of `ledger`'s locks is owed,
/// with `why` written beside it for whoever finds it.
/// Why:
///   A record nothing reads is not a record,
/// so the marker is written where the next reader of that repository looks,
/// and it
///  says why it is there,
/// because the line a person may never see is not the only thing that should.
/// Returns the markers written,
/// so a control can read them and no caller has to guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = (ledger, why) => markersOf(ledger.roots()).map((marker) => { mkdir(dirname(marker)); write(marker, `${why}\n`); return marker });
/// ```
pub fn record(ledger: &LockLedger, why: &str) -> Vec<PathBuf> {
    let mut written: Vec<PathBuf> = Vec::new();
    for marker in markers_of(ledger.roots()) {
        let Some(folder) = marker.parent() else {
            continue;
        };
        if std::fs::create_dir_all(folder).is_err() {
            continue;
        }
        // The marker is a line and no more: a reader that finds it needs the reason and the fact, never a transcript.
        if std::fs::write(marker.as_path(), format!("a stopped push's lock sweep did not finish inside the bound of its stop: {why}\n")).is_ok() {
            written.push(marker);
        }
    }
    return written;
}

/// What:
///  Judge the locks of `roots` and drop every owed marker of theirs,
/// answering what the judgement found.
/// Why:
///   The judgement is the duty the marker carries,
/// so it runs before the marker goes;
/// a marker that
///  outlived its judgement would send every later reader through the walk again.
///  The caller says the outcome's notice,
/// which names the locks the judgement will not remove and the `rm --` command
///  that clears each.
/// No place of the blocking budget is taken here,
/// because this is the caller's own judgement and its
///  caller decided where it runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settle = (roots, judge) => { const outcome = clear(roots, judge); for (const marker of owedMarkers(roots)) unlink(marker); return outcome };
/// ```
pub fn settle(roots: &[PathBuf], judge: &Judge) -> Outcome {
    let markers: Vec<PathBuf> = owed_markers_of(roots);
    let outcome: Outcome = clear_stale_ref_locks(roots, judge);
    for marker in markers {
        // The marker goes whether or not a proof was found: the judgement ran, which is what it carried to.
        let _removed = std::fs::remove_file(marker.as_path());
    }
    return outcome;
}

/// What:
///  Settle what a stopped push's sweep owes for `roots`,
/// if anything,
/// and say what the judgement found.
/// Why:
///   A reader that meets the marker names what it did in a line,
/// so the person who saw the owed line sees the
///  judgement that followed it.
/// Nothing happens when no marker is there,
/// and that is what keeps the service's every push cheap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settleIfOwed = (roots, judge) => anyOwed(roots) ? say(settle(roots, judge)) : undefined;
/// ```
pub fn settle_if_owed(roots: &[PathBuf], judge: &Judge) -> Option<Outcome> {
    if !any_owed(roots) {
        return None;
    }
    let outcome: Outcome = settle(roots, judge);
    let notice: String = outcome.notice();
    if !notice.is_empty() {
        warn(TAG, notice.trim_end());
    }
    return Some(outcome);
}

/// The judge a control states for the readers of this record,
/// and nothing outside one.
/// Why:
///   A reader judges with the process table of the whole machine,
/// which refuses to decide while a Git of another account cannot be
///  looked into,
/// and the margin it keeps is seven seconds,
/// so a control of the reading cannot see a removal with it.
/// The
///  control states its own judge instead,
/// as the stop's own controls state theirs (`LockLedger::with_judge`).
#[cfg(test)]
static CONTROL_JUDGE: std::sync::Mutex<Option<Judge>> = std::sync::Mutex::new(None);

/// State `judge` as the judge of every reader of this record in this process.
#[cfg(test)]
pub(crate) fn a_control_states_the_judge(judge: Judge) {
    if let Ok(mut slot) = CONTROL_JUDGE.lock() {
        *slot = Some(judge);
    }
}

/// What:
///  The judge a reader of this record uses:
/// the control's, where one is stated,
/// and the process table of the whole machine otherwise.
/// Why:
///   Every reader asks the same question,
/// so they must all ask it of the same judge,
/// and a control can replace that judge without
///  reaching into each reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theJudgeForARecord = () => controlJudge ?? Judge.here();
/// ```
pub fn the_judge_for_a_record() -> Judge {
    #[cfg(test)]
    if let Ok(slot) = CONTROL_JUDGE.lock()
        && let Some(stated) = slot.clone()
    {
        return stated;
    }
    return Judge::here();
}

/// What:
///  Settle what a stop owed in one Git directory,
/// as the service's start for that worktree does,
/// and answer what the judgement found.
/// Why:
///   A service that starts is the next look the repository gets after a stop that owed a sweep,
/// and it serves pushes
///  before any person's command of that worktree may run,
/// so its start is where the record is met first when nothing else
///  ran in between.
/// The judgement reads the process table,
/// so this is called from a place that may do blocking work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settleAtAServiceStart = (gitDir, judge) => settleIfOwed([gitDir], judge);
/// ```
pub fn settle_at_a_service_start(git_dir: &Path, judge: &Judge) -> Option<Outcome> {
    return settle_if_owed(std::slice::from_ref(&git_dir.to_path_buf()), judge);
}

/// What:
///  The marker of one Git directory,
/// for a reader that has the directory and not a command's roots.
/// Why:
///   A service's start knows the worktree it serves and not a command,
/// so it asks by the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const markerIn = (gitDir) => path.join(gitDir, WRAPPER_FOLDER, MARKER);
/// ```
pub fn marker_in(git_dir: &Path) -> PathBuf {
    return git_dir.join(WRAPPER_FOLDER).join(MARKER);
}

/// The record's own behaviour is controlled in the test module.
#[cfg(test)]
#[path = "owed_lock_sweeps_tests.rs"]
mod tests;
