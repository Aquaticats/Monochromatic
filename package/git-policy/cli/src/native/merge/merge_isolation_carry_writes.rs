//! What:
//!  What a carry writes,
//! and the vocabulary of one step's writes:
//!  the index records of a batch,
//!  one file's bytes through a private name and a rename,
//!  a symbolic link,
//!  and the bounded wait for `index.lock` that begins the batch.
//! Why:
//!   Every write of one step lands as one step of the file system or of the index,
//!  so a kill leaves either what was there or the whole result;
//!  the judgements that decide a step are `merge_isolation_carry_apply.rs`,
//!  which is at its code-line limit,
//!  and the placing of one file,
//!  which refuses to replace what it did not judge,
//!  is `merge_isolation_carry_place.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeSteps({ context, root, gitDir, steps, out }); // one update-index call, then one file at a time
//! ```

use crate::merge_isolation_carry_place::{JudgedFile, Placed, is_still_the_judged_entry, place_file_awaiting};
use crate::merge_isolation_carry_reads::run_bytes_awaiting;
use crate::transaction_git::{GitContext, GitOutput};
use crate::transaction_phase::Phase;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The mode an `update-index --index-info` record gives a path it removes.
const REMOVAL_MODE: &str = "0";

/// What one layer of one path writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexWrite = { kind: 'leave' } | { kind: 'entry'; mode: string; oid: string } | { kind: 'remove' };
/// ```
pub(crate) enum IndexWrite {
    /// The layer already holds it.
    Leave,
    /// The entry to write.
    Entry(String, String),
    /// Nothing stands at the path.
    Remove,
}

/// What the worktree of one path writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileWrite = { kind: 'leave' } | { kind: 'bytes'; bytes: Uint8Array; mode: string } | { kind: 'remove' };
/// ```
pub(crate) enum FileWrite {
    /// The layer already holds it.
    Leave,
    /// The file form and the mode to write.
    Bytes(Vec<u8>, String),
    /// Nothing stands at the path.
    Remove,
}

/// One path's writes and what the report says of it.
pub(crate) struct Step {
    /// The path the step writes,
    /// from the top of the worktree.
    pub(crate) path: Vec<u8>,
    /// What its index entry becomes.
    pub(crate) index: IndexWrite,
    /// What its worktree file becomes.
    pub(crate) worktree: FileWrite,
    /// What stood at its worktree path when the step was judged,
    /// which the write checks before it replaces anything.
    pub(crate) judged: JudgedFile,
}

/// Which wait for the index lock a step makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockWait = 'before-the-apply' | 'while-finishing';
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum LockWait {
    /// The merge's own apply is about to run,
    /// so the wait's words are the merge's own and a dead owner's lock is cleared.
    BeforeTheApply,
    /// A merge that is being finished,
    /// where nothing is written when the lock stays held and the next command tries again.
    WhileFinishing,
}

/// Why a carry or a clear stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CarryFailure = { kind: 'index-held'; detail: string } | { kind: 'failed'; detail: string } | { kind: 'foreign'; path: Uint8Array } | { kind: 'changed'; path: Uint8Array };
/// ```
#[derive(Debug)]
pub enum CarryFailure {
    /// Another program held `index.lock` until the wait's bound,
    /// so nothing was written.
    IndexHeld(String),
    /// A read or a write failed;
    /// the capture still holds the bytes.
    Failed(String),
    /// One path holds bytes neither the capture nor the change explains.
    Foreign(Vec<u8>),
    /// One path was written after this step judged it,
    /// so nothing was written to it and its bytes are the person's.
    Changed(Vec<u8>),
}

/// What:
///  The index records that write these entries and remove these paths.
/// Why:
///  One `update-index` call writes every entry,
/// so the index moves between two states in one step and a kill cannot
///  leave half of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexRecords(steps: Step[], width: number): Uint8Array;
/// ```
fn index_records(steps: &[Step], width: usize) -> Vec<u8> {
    let mut records: Vec<u8> = Vec::new();
    for step in steps {
        match &step.index {
            IndexWrite::Leave => continue,
            IndexWrite::Entry(mode, oid) => records.extend_from_slice(format!("{mode} {oid}\t").as_bytes()),
            IndexWrite::Remove => records.extend_from_slice(format!("{REMOVAL_MODE} {}\t", "0".repeat(width)).as_bytes()),
        }
        records.extend_from_slice(step.path.as_slice());
        records.push(0);
    }
    return records;
}

/// The width of the object name these steps write,
/// from the first step that writes one.
/// Why:
///   A removal record repeats an all-zero object name of the same width,
///  and only an entry's own name knows what that width is.
fn entry_width(step: &Step) -> Option<usize> {
    return match &step.index {
        IndexWrite::Entry(_, oid) => Some(oid.len()),
        IndexWrite::Remove | IndexWrite::Leave => None,
    };
}

/// What:
///  The test-only markers one write step reaches.
/// Why:
///   A step writes its index records in one call and its worktree files one at a time after it,
/// so a control needs the instant
///  between those parts and between two files;
///  `transaction_phase` kills or pauses the process there.
pub(crate) struct StepMarks {
    /// Reached once every index record of this step is written.
    index: Phase,
    /// Reached after each worktree file of this step,
    /// numbered by occurrence.
    file: Phase,
}

/// The markers of the clear's writes.
const CLEAR_MARKS: StepMarks = StepMarks { index: Phase::MergeClearIndex, file: Phase::MergeClearFile };

/// The markers of the carry back's writes.
const CARRY_MARKS: StepMarks = StepMarks { index: Phase::MergeCarryIndex, file: Phase::MergeCarryFile };

/// What:
///  Which step of a carry these writes are,
///  and what that step does when a path moved after it was judged.
/// Why:
///   The clear hands the paths to Git's own two-way update,
/// so a path that moved after the judgement must stop it with
///  nothing written and the round made again;
/// the carry back and a putting back write each path on its own,
///  so one that moved keeps its bytes and the others still go.
///  The test-only markers follow the same three steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Batch = 'clear' | 'carry' | 'restore';
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Batch {
    /// The clear before the branch moves.
    Clear,
    /// The merge of each layer with the merge.
    Carry,
    /// The putting back of a merge that did not land.
    Restore,
}

/// What:
///  Reach one test marker,
/// or refuse the step when the marker variable is malformed.
/// Why:
///   The write step is the only place the instant between its parts exists,
/// so the marker is reached from inside it.
fn reach_mark(phase: Phase, occurrence: Option<u64>) -> Result<(), CarryFailure> {
    return match crate::transaction_phase::reach_transaction_phase(phase, occurrence) {
        Ok(()) => Ok(()),
        Err(error) => Err(CarryFailure::Failed(format!("{error}"))),
    };
}

impl Batch {
    /// The markers this batch reaches,
    /// when a control watches them.
    fn marks(self) -> Option<&'static StepMarks> {
        return match self {
            Batch::Clear => Some(&CLEAR_MARKS),
            Batch::Carry => Some(&CARRY_MARKS),
            Batch::Restore => None,
        };
    }

    /// Whether a path that moved after the judgement stops the whole batch,
    /// rather than keeping its own bytes while the rest go on.
    fn stops_when_a_path_moved(self) -> bool {
        return self == Batch::Clear;
    }
}

/// What:
///  Refuse the whole step when any path it is about to write holds something other than what it judged.
/// Why:
///   The index records and the files are separate steps,
///  and a file saved between the judgement and the first of them
///  would be replaced by the branch's content with no notice;
///  reading every identity first means the step writes
///  nothing at all when any path has moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const step of steps) if (!isStillTheJudgedEntry(root, step.path, step.judged)) throw { kind: 'changed', path: step.path };
/// ```
fn every_path_is_the_one_judged(root: &Path, steps: &[Step]) -> Result<(), CarryFailure> {
    for step in steps {
        // A step whose worktree write is nothing takes nothing away, so its path is not checked and cannot stop the step.
        if matches!(step.worktree, FileWrite::Leave) {
            continue;
        }
        if !is_still_the_judged_entry(root, step.path.as_slice(), &step.judged) {
            return Err(CarryFailure::Changed(step.path.clone()));
        }
    }
    return Ok(());
}

/// The words of a failure that stopped one step's write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failureDetail = (failure) => failure.kind === 'failed' ? failure.detail : 'it could not be written';
/// ```
fn failure_detail(failure: &CarryFailure) -> String {
    return match failure {
        CarryFailure::Failed(detail) | CarryFailure::IndexHeld(detail) => String::from(detail.as_str()),
        CarryFailure::Foreign(path) | CarryFailure::Changed(path) => crate::path_bytes::path_message_text(path.as_slice()),
    };
}

/// What:
///  Write every planned entry and file,
/// after a bounded wait for `index.lock`.
/// Why:
///   The index is the one thing two commands cannot write at once,
/// so the wait is named and bounded,
/// and a lock
///  that outlasts it leaves everything as it was.
///  Both Git starts are awaited,
///  because a start enters the runtime and is refused from inside it.
///  A clear checks every worktree path before its index records go,
/// and every batch takes each path aside under a private name
///  before its own bytes are replaced,
/// so a file saved in either window is kept,
///  and the private name of a file a program can still write into is kept and said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeStepsAwaiting({ context, root, gitDir, steps, batch, out }): Promise<StepResult>;
/// ```
pub(crate) async fn write_steps_awaiting<W: std::io::Write + ?Sized>(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    steps: &[Step],
    transaction_id: &str,
    wait: LockWait,
    batch: Batch,
    out: &mut W,
) -> Result<StepResult, CarryFailure> {
    let waited = match wait {
        LockWait::BeforeTheApply => crate::index_holder_wait::wait_for_index_release_before_a_merge_that_clears_a_dead_lock(git_dir.join("index").as_path(), out),
        LockWait::WhileFinishing => crate::index_holder_wait::wait_for_index_release_while_finishing_a_merge(git_dir.join("index").as_path(), out),
    };
    if !matches!(
        waited,
        crate::index_holder_wait::IndexWait::NotHeld | crate::index_holder_wait::IndexWait::Released(_)
    ) {
        return Err(CarryFailure::IndexHeld(git_dir.join("index.lock").display().to_string()));
    }
    if batch.stops_when_a_path_moved() {
        every_path_is_the_one_judged(root, steps)?;
    }
    let width: usize = steps.iter().find_map(entry_width).unwrap_or(40);
    let records: Vec<u8> = index_records(steps, width);
    if !records.is_empty() {
        let arguments: Vec<OsString> = vec![OsString::from("update-index"), OsString::from("-z"), OsString::from("--index-info")];
        let written: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), Some(records)).await.map_err(CarryFailure::Failed)?;
        if !written.succeeded() {
            let detail: String = written.error_text();
            if detail.contains("index.lock") {
                return Err(CarryFailure::IndexHeld(detail));
            }
            return Err(CarryFailure::Failed(detail));
        }
    }
    if let Some(marks) = batch.marks() {
        reach_mark(marks.index, None)?;
    }
    let mut wrote_a_file: bool = false;
    let mut written: u64 = 0;
    let mut changed: Vec<Vec<u8>> = Vec::new();
    for step in steps {
        if !matches!(step.worktree, FileWrite::Leave) {
            wrote_a_file = true;
        }
        let path: String = String::from_utf8_lossy(step.path.as_slice()).into_owned();
        // The private names this one path's write may not remove, because a program could still write into them.
        //  A deletion keeps such a name too, which is why the notice below says the write happened and not that a file was
        //  replaced.
        let mut held: Vec<PathBuf> = Vec::new();
        let placed: Placed = match place_file_awaiting(context, root, step.path.as_slice(), &step.judged, &step.worktree, transaction_id, &mut held).await {
            Ok(found) => found,
            Err(failure) => {
                let _ = out.write_all(
                    format!(
                        "monochromatic-git-policy-cli: {path} could not be written ({}); the bytes this merge carries are kept, and the next \
                         git command in this worktree finishes carrying them.\n",
                        failure_detail(&failure)
                    )
                    .as_bytes(),
                );
                return Err(failure);
            }
        };
        for aside in &held {
            let _ = out.write_all(
                format!(
                    "monochromatic-git-policy-cli: {path} was written, and the file it held before this merge took it can still be written \
                     by a program that opened it, so it is kept at {}; nothing of the person's was removed.\n",
                    aside.display()
                )
                .as_bytes(),
            );
        }
        match placed {
            Placed::Written | Placed::Nothing => {}
            Placed::Changed { kept } => {
                if let Some(waits) = kept {
                    let _ = out.write_all(
                        format!(
                            "monochromatic-git-policy-cli: {path} was written again while this merge ran, and the file it held before that is \
                             kept at {}; nothing was removed.\n",
                            waits.display()
                        )
                        .as_bytes(),
                    );
                }
                changed.push(step.path.clone());
            }
        }
        if !matches!(step.worktree, FileWrite::Leave) {
            written += 1;
            if let Some(marks) = batch.marks() {
                reach_mark(marks.file, Some(written))?;
            }
        }
    }
    // A file this step wrote carries a new modification time and size, and Git compares those with the index's record of the
    //  file before it updates a path (`read-tree` refuses an entry that is not up to date). The refresh re-reads the record for
    //  every entry whose content still hashes to what the index says, so the next step reads the checkout as it is.
    if wrote_a_file {
        let arguments: Vec<OsString> = vec![OsString::from("update-index"), OsString::from("-q"), OsString::from("--refresh")];
        let refreshed: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), None).await.map_err(CarryFailure::Failed)?;
        if !refreshed.succeeded() && refreshed.code != Some(1) {
            return Err(CarryFailure::Failed(refreshed.error_text()));
        }
    }
    return Ok(StepResult { changed });
}

/// What one step of writes came to,
/// beside the paths it wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StepResult = { changed: Uint8Array[] };
/// ```
pub(crate) struct StepResult {
    /// The paths another write took,
    /// whose bytes are the person's and were left alone.
    pub(crate) changed: Vec<Vec<u8>>,
}
