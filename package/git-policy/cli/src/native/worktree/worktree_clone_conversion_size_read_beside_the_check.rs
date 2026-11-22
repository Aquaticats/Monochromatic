//! What:
//!  The conversion fence's one `git ls-tree` size read, started before the source check and joined at the fence,
//! in a measurement build only.
//! Why:
//!  On rung 1 the read sits on the add's waiting path, after `checkSource` and after `git check-attr`;
//! on rung 2 it already runs beside the copies.
//! The read needs nothing the check produces except the commit `HEAD` names,
//!  which the plain ref files answer in process before the check runs,
//! so a measurement build starts it there and holds its answer for the fence.
//! The answer is used only when the commit it read is the commit the check proved,
//!  and a check that refuses drops it unused:
//! the read is of committed objects and writes nothing,
//!  so a refused or killed add leaves no state behind it.
//! This file is compiled only with the `measurement-routes` feature,
//!  so no shipped build holds the switch or the slot.
//! It is a PROTOTYPE FOR MEASUREMENT, NOT FOR MERGING:
//!  the slot is process-wide because one command is one process,
//!  and a proposal decides what a shipped overlap would look like.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! startTheSizeReadBesideTheCheck(context, identity); // the fence's map, read on a thread of its own
//! const fence = fenceTheClone(source, context); // takes the map when its commit is the checked one, else reads
//! ```

use crate::add_timing::{begin_step, begin_work, end_step, end_work};
use crate::diagnostic_log::debug;
use crate::repo_read_layout::ObjectFormat;
use crate::scoped_start::{refused_thread_sentence, spawn_named};
use crate::subvolume_index_files::repository_hash_length;
use crate::transaction_git::GitContext;
use crate::worktree_clean::head_commit;
use crate::worktree_clone_conversion_fence::blob_sizes;
use crate::worktree_identity::WorktreeIdentity;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeCloneConversionSizeReadBesideTheCheck";

/// The variable that turns the overlap on in a measurement build.
/// Why:
///   One binary has to run both arms of the measurement,
/// so the overlap is a switch and not a second build,
///  and the identical-binary control is the same binary with the switch unset.
pub const SWITCH: &str = "MONOCHROMATIC_CLONE_SIZE_READ_BESIDE";

/// The name the read's own duration is noted under in the add's record.
/// Why:
///   The read runs on a thread of its own,
/// which the adding thread's steps cannot see,
///  so its time is work and the join's wait is a step.
const WORK: &str = "fenceSizesBesideTheCheck";

/// The step name the wait for the started read is recorded under.
const JOIN_STEP: &str = "fenceSizesWaitingForTheRead";

/// The name of the thread the read runs on.
const THREAD: &str = "clone-size-read-beside-the-check";

/// A map from a tracked path to the size of the blob its source's index records.
type Sizes = HashMap<Vec<u8>, u64>;

/// What:
///  Whether a measurement build is asked to start the read beside the source check.
/// Why:
///   The switch is read once per add,
/// so every step of that add agrees on the arm it belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const overlapAsked = () => process.env.MONOCHROMATIC_CLONE_SIZE_READ_BESIDE === '1';
/// ```
pub fn asked() -> bool {
    return std::env::var(SWITCH).map(|value| return value == "1").unwrap_or(false);
}

/// What:
///  What one started read waits to answer, and the wake-up for the thread that joins it.
/// Why:
///   A thread's answer arrives once,
/// and the joining thread may ask before or after it,
///  so the two share one cell and one condition variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answer = { settled: Mutex<Result<Map<string, number>, string> | undefined>; arrived: Condvar };
/// ```
struct Answer {
    /// The read's answer, or nothing until it has one.
    settled: Mutex<Option<Result<Sizes, String>>>,
    /// Wakes the joining thread when the answer is here.
    arrived: Condvar,
}

/// What:
///  The size read one add started before its source check:
/// the commit it read and the answer it will have.
/// Why:
///   The fence may use the answer only for the commit its source's index was proven equal to,
/// so the commit travels with it and a different one discards it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SizeReadBesideTheCheck = { commit: string; answer: Answer };
/// ```
pub struct SizeReadBesideTheCheck {
    /// The commit the read listed.
    commit: String,
    /// Where its answer arrives.
    answer: Arc<Answer>,
}

/// The read this add started, until the fence takes it.
/// Why:
///   The start happens in the ladder and the answer is wanted in the fence,
/// which the start cannot call and which must not carry a second parameter through the rung;
///  one command is one process,
///  and the slot is emptied when the fence takes it,
///  so no second add can read a first add's map.
static STARTED: Mutex<Option<SizeReadBesideTheCheck>> = Mutex::new(None);

/// What:
///  The worktree root and its two Git directories,
/// or nothing for an identity without a worktree.
/// Why:
///   The read runs in the source's worktree and lists the commit `HEAD` names there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directoriesOf = (identity) => identity.kind === 'worktree' ? [identity.root, identity.gitDir, identity.commonDir] : undefined;
/// ```
fn directories_of(identity: &WorktreeIdentity) -> Option<(&Path, &Path, &Path)> {
    return match identity {
        WorktreeIdentity::MainWorktree { worktree_root, git_dir, common_dir } | WorktreeIdentity::LinkedWorktree { worktree_root, git_dir, common_dir } => Some((worktree_root.as_path(), git_dir.as_path(), common_dir.as_path())),
        WorktreeIdentity::BareRepository { .. } | WorktreeIdentity::OutsideWorktree => None,
    };
}

/// What:
///  The read itself:
/// the sizes of the commit's tree as one `git ls-tree` reports them,
/// timed as work of its own thread.
/// Why:
///   The thread has no recording of the adding thread's steps,
/// so the only place its cost can be read from the record is the work sum.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runTheRead = ({ context, root, commit, answer }) => { const began = beginWork(); const sizes = sizesOfTheCommit(context, root, commit); endWork('fenceSizesBesideTheCheck', began); settle(answer, sizes); };
/// ```
fn run_the_read((context, root, commit, answer): (GitContext, PathBuf, String, Arc<Answer>)) {
    let began: std::time::Instant = begin_work();
    let sizes: Result<Sizes, String> = blob_sizes(&context, root.as_path(), commit.as_str());
    end_work(WORK, began);
    if let Ok(mut settled) = answer.settled.lock() {
        *settled = Some(sizes);
    }
    answer.arrived.notify_all();
}

/// What:
///  Start the fence's size read now,
/// beside the source check the ladder runs next,
/// when this measurement build is asked to.
/// Why:
///   The read needs the worktree root and the commit `HEAD` names,
/// both of which the plain ref files answer without the check;
/// it reads committed objects only,
///  so it touches nothing the check has not proved and writes nothing.
/// A thread the system refuses loses only the overlap:
/// the fence then reads the map itself, as it always has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startBesideTheCheck(context, identity) { if (!asked()) return; const started = readyRead(context, identity); if (started) put(STARTED, started); }
/// ```
pub fn start_beside_the_check(context: &GitContext, identity: &WorktreeIdentity) {
    if !asked() {
        return;
    }
    let Some((root, git_dir, common_dir)): Option<(&Path, &Path, &Path)> = directories_of(identity) else {
        return;
    };
    let Some(hash_length): Option<usize> = repository_hash_length(common_dir) else {
        return;
    };
    let format: ObjectFormat = if hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let Some(commit): Option<String> = head_commit(git_dir, common_dir, format) else {
        debug(TAG, "the commit HEAD names is not a plain ref, so the size read does not start beside the check");
        return;
    };
    let answer: Arc<Answer> = Arc::new(Answer { settled: Mutex::new(None), arrived: Condvar::new() });
    let state: (GitContext, PathBuf, String, Arc<Answer>) = (context.clone(), root.to_path_buf(), commit.clone(), Arc::clone(&answer));
    if let Err(error) = spawn_named(THREAD, move || {
        return run_the_read(state);
    }) {
        debug(TAG, refused_thread_sentence("read the fence's blob sizes beside the source check", &error).as_str());
        return;
    }
    let started: SizeReadBesideTheCheck = SizeReadBesideTheCheck { commit, answer };
    if let Ok(mut slot) = STARTED.lock() {
        *slot = Some(started);
    }
}

/// What:
///  The read this add started, when it listed `commit`,
/// taken out of the slot.
/// Why:
///   A read of another commit is not this fence's answer,
/// and leaving it in the slot would let a later add spend its wait on it;
/// so a mismatched read is dropped here,
///  and its thread ends on its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeStarted(commit) { const started = take(STARTED); return started?.commit === commit ? started : undefined; }
/// ```
pub fn take_started(commit: &str) -> Option<SizeReadBesideTheCheck> {
    let started: SizeReadBesideTheCheck = STARTED.lock().ok().and_then(|mut slot| return slot.take())?;
    if started.commit != commit {
        debug(TAG, format!("the read started beside the check listed {} and the fence needs {commit}, so it is dropped unused", started.commit).as_str());
        return None;
    }
    return Some(started);
}

impl SizeReadBesideTheCheck {
    /// What:
    ///  The read's answer,
    /// waiting for it when it has not arrived yet,
    /// with the wait recorded as a step of the adding thread.
    /// Why:
    ///   The wait is what the overlap saves,
    /// so it is the number the record has to show:
    ///  a wait near nothing means the read fit beside the check and the attributes,
    ///  and a wait near the read's own time means the two steps did not fit.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// join() { const entered = beginStep('fenceSizesWaitingForTheRead'); const answer = this.answer.waitFor(); endStep(entered); return answer; }
    /// ```
    pub fn join(self) -> Result<Sizes, String> {
        let entered: bool = begin_step(JOIN_STEP);
        let mut settled = match self.answer.settled.lock() {
            Ok(settled) => settled,
            Err(poisoned) => poisoned.into_inner(),
        };
        loop {
            if let Some(answer) = settled.take() {
                end_step(entered);
                return answer;
            }
            settled = match self.answer.arrived.wait(settled) {
                Ok(settled) => settled,
                Err(poisoned) => poisoned.into_inner(),
            };
        }
    }
}
