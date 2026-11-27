//! What:
//!  The intent records a recovery walks:
//!  the records of the merges whose process died,
//!  and the settling of each one nobody owns any more.
//! Why:
//!  A record's owner decides whether it is settled at all,
//! and a probe the account may not make reads as alive,
//!  so recovery never touches a merge that may be live;
//! the walk has a file of its own because that decision,
//! the settles it drives and the lines it
//!  writes are not the decision about one record's state
//! (`merge_isolation_recovery.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcomes = await recoverInterruptedMerges({ place, gitDir, stderr });
//! ```

/// The one job of the blocking pool that reads the records,
/// so a disk that makes a read wait holds no worker of the runtime
/// (`doc/planning/git-policy-cli-full-tokio.md`,
/// "Blocking files,
/// fsync and the runtime's own shutdown").
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::diagnostic_log::debug;
use crate::lazy_runtime::run_on_runtime;
use crate::merge_isolation_journal::owner_liveness;
use crate::merge_isolation_journal::{
    MergeIntent,
    list_merge_intents,
    parse_merge_intent,
};
use crate::merge_isolation_kept_copies_decisions::sweep_kept_copies_awaiting;
use crate::merge_isolation_recovery::settle_failed_merge_awaiting;
use crate::merge_isolation_recovery_notice::recovery_notice;
use crate::merge_isolation_recovery_place::RecoveryPlace;
use crate::merge_isolation_recovery_record::MergeRecoveryAction;
use crate::merge_isolation_recovery_record::MergeRecoveryOutcome;
use crate::merge_isolation_take_aside_names::merge_key_of_a_record;
/// What:
///  The reader that opens a record as a plain file,
///  and the bound it reads to.
/// Why:
///  Opening a pipe for reading waits for a writer that may never come,
///  and this folder's first read runs on the start of every wrapped command,
///  so a pipe at a record's name would hold every command with nothing said (`SPEC.md`,
///  "Every file this wrapper reads from its own record folders is opened as a plain file").
use crate::plain_file::{
    SMALL_FILE_LIMIT,
    read_plain_file,
};
use crate::recovery_error::{
    RecoveryError,
    io_failure,
};
use crate::transaction_owner::OwnerLiveness;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Whether the worktree's Git directory holds an intent record that recovery must look at:
/// one whose owner is
///  dead,
/// or one that cannot be read.
/// Why:
///   The common case needs one failed directory read and no lock.
///  A record of a merge that still runs is not recovered,
/// so a nested command does not touch its ancestor's merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await listMergeIntents(gitDir)).some((record) => !isLive(record))
/// ```
pub fn merge_intents_to_recover(git_dir: &Path) -> Result<bool, RecoveryError> {
    for record in list_merge_intents(git_dir)? {
        let bytes: Vec<u8> = match read_plain_file(record.as_path(), SMALL_FILE_LIMIT) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(io_failure("reading", record.as_path(), &error)),
        };
        let Some(intent) = parse_merge_intent(bytes.as_slice()) else {
            return Ok(true);
        };
        if owner_liveness(&intent, record.as_path())? == OwnerLiveness::Dead {
            return Ok(true);
        }
    }
    return Ok(false);
}

/// What:
///  Recover every intent record whose owner is dead.
/// Why:
///   A record whose owner still runs is left alone,
/// so recovery does not race a live merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverInterruptedMerges({ place, gitDir }): Promise<MergeRecoveryOutcome[]>;
/// ```
pub fn recover_interrupted_merges(
    place: &RecoveryPlace,
    git_dir: &Path,
    out: &mut dyn Write,
) -> Result<Vec<MergeRecoveryOutcome>, RecoveryError> {
    return match run_on_runtime(recover_interrupted_merges_awaiting(place, git_dir, out)) {
        Ok(found) => found,
        Err(refusal) => Err(RecoveryError(format!("{refusal}"))),
    };
}

/// What:
///  The same recovery of every intent record whose owner is dead,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   Each record is settled through Git,
/// and a synchronous start registers the step's pipes by entering the runtime,
///  which is refused from inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverInterruptedMerges({ place, gitDir, stderr }): Promise<MergeRecoveryOutcome[]>;
/// ```
pub async fn recover_interrupted_merges_awaiting<W: std::io::Write + ?Sized>(
    place: &RecoveryPlace,
    git_dir: &Path,
    out: &mut W,
) -> Result<Vec<MergeRecoveryOutcome>, RecoveryError> {
    let mut outcomes: Vec<MergeRecoveryOutcome> = Vec::new();
    // Every record is read and its owner judged before any is settled, so a record of another namespace older than the patience
    //  stops the command with nothing changed (owner, 2026-11-04, "Refuse and name it (Recommended)"; sol-1 round 3, P1 C).
    // Listing the records and reading each record's bytes are file calls that can wait on a disk, so one job of the blocking
    //  pool makes them all before the first record is judged.
    let records_of: PathBuf = git_dir.to_path_buf();
    let read: Result<
        Result<Vec<(PathBuf, Vec<u8>)>, RecoveryError>,
        crate::blocking_job::BlockingJobError,
    > = run_blocking_job_within_the_budget(move || {
        let mut kept: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        for record in list_merge_intents(records_of.as_path())? {
            let bytes: Vec<u8> = match read_plain_file(record.as_path(), SMALL_FILE_LIMIT) {
                Ok(found) => found,
                // A record a merge removed after the listing is one this command has nothing to settle.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(io_failure("reading", record.as_path(), &error)),
            };
            kept.push((record, bytes));
        }
        return Ok(kept);
    })
    .await;
    let records: Vec<(PathBuf, Vec<u8>)> = match read {
        Ok(Ok(found)) => found,
        Ok(Err(failure)) => return Err(failure),
        // A folder the pool never read is no evidence that no merge was interrupted, so a record is never dropped from the walk.
        Err(refused) => {
            return Err(RecoveryError(format!(
                "the merge records could not be read: {refused}"
            )));
        }
    };
    let mut planned: Vec<(PathBuf, MergeIntent, OwnerLiveness)> = Vec::new();
    for (record, bytes) in records {
        let Some(intent) = parse_merge_intent(bytes.as_slice()) else {
            return Err(RecoveryError(format!(
                "The merge record {} is malformed. It was left in place; if no merge is running in this worktree, \
                 remove it, and check the branch and `git status` before merging again.",
                record.display()
            )));
        };
        let liveness: OwnerLiveness = owner_liveness(&intent, record.as_path())?;
        planned.push((record, intent, liveness));
    }
    // A copy a merge kept beside a path is decided before any merge's paths are judged: a copy the person's file holds again, or
    //  one an unfinished merge names, must be in its place before those paths are read (`merge_isolation_kept_copies.rs`).
    // The owners are judged first because a copy names its merge: one that merge still runs is that merge's to finish, and a
    //  command beside it that put the copy back would make Git refuse the checkout the merge is in the middle of making.
    let mut running: Vec<String> = Vec::new();
    for (record, _, liveness) in &planned {
        if *liveness != OwnerLiveness::Alive {
            continue;
        }
        running.push(merge_key_of_a_record(record.as_path()));
    }
    // A control reaches here to write the state a merge that started after this scan leaves, which is the one thing the sweep
    //  has to read the records again for.
    crate::service_fault::reach("recovery-owners-planned");
    sweep_kept_copies_awaiting(place, running.as_slice(), out).await;
    for (record, intent, liveness) in planned {
        let action: MergeRecoveryAction = if liveness == OwnerLiveness::Alive {
            debug(
                "recoverInterruptedMerges",
                format!("the owner of {} still runs", record.display()).as_str(),
            );
            MergeRecoveryAction::OwnerActive
        } else {
            match settle_failed_merge_awaiting(place, record.as_path(), &intent, out).await {
                Ok(settled) => settled,
                // A merge that was saved on a side ref has nothing of itself in the checkout, so a command that cannot finish it
                //  has nothing to be stopped for: it says where the merge is and what blocked it, and goes on. A begun merge's
                //  refusal still stops the command, because the checkout may hold its files.
                Err(refusal) if !refusal.began && intent.side_ref.is_some() => {
                    let side_ref: &str = intent.side_ref.as_deref().unwrap_or("its side ref");
                    let _noted = out.write_all(
                        format!(
                            "monochromatic-git-policy-cli: the merge saved on {side_ref} was not finished, and nothing of it is in the \
                             checkout; this command goes on. {}\n",
                            refusal.error.0
                        )
                        .as_bytes(),
                    );
                    MergeRecoveryAction::DeferredRefused
                }
                Err(refusal) => {
                    // Part of the merge is in the index or the worktree, so the command must not go on reading the checkout.
                    let side_ref: &str = intent.side_ref.as_deref().unwrap_or("its record");
                    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
                    // The refusal's own sentence ends with a period, and this one follows it inside a sentence of its own, so
                    //  the period it already carries is not written twice.
                    let said: &str = refusal
                        .error
                        .0
                        .strip_suffix('.')
                        .unwrap_or(refusal.error.0.as_str());
                    return Err(RecoveryError(format!(
                        "A merge saved on {side_ref} for {target} is partly installed in this worktree: {said}. Nothing more was changed \
                         and the record {} was kept; check `git status` and `git stash list` before you commit anything.",
                        record.display()
                    )));
                }
            }
        };
        let outcome: MergeRecoveryOutcome = MergeRecoveryOutcome {
            record,
            action,
            target_ref: intent.target_ref.clone(),
            new_oid: intent.new_oid.clone(),
            side_ref: intent.side_ref.clone(),
        };
        if let Some(line) = recovery_notice(&outcome) {
            // A closed terminal is not a reason to stop the command the person asked for.
            let _noted = out.write_all(line.as_bytes());
        }
        outcomes.push(outcome);
    }
    return Ok(outcomes);
}
