//! What:
//!  The intent records a recovery walks:
//!  the one record of a merge whose process died,
//!  whoever may settle it,
//!  and what a command is told about each.
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

use crate::diagnostic_log::debug;
use crate::lazy_runtime::run_on_runtime;
use crate::merge_isolation_journal::{MergeIntent, list_merge_intents, parse_merge_intent};
use crate::merge_isolation_kept_copies_decisions::sweep_kept_copies_awaiting;
use crate::merge_isolation_take_aside_names::merge_key_of_a_record;
use crate::merge_isolation_recovery::settle_failed_merge_awaiting;
use crate::merge_isolation_recovery_notice::recovery_notice;
use crate::merge_isolation_recovery_steps::{MergeRecoveryAction, RecoveryPlace};
use crate::recovery_error::{RecoveryError, io_failure};
use crate::transaction_owner::{OwnerLiveness, OwnerLivenessError, classify_transaction_owner_record};
use std::io::Write;
use std::path::{Path, PathBuf};

/// One record recovery visited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergeRecoveryOutcome = { record: string; action: MergeRecoveryAction; targetRef?: string; newOid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergeRecoveryOutcome {
    /// The record's file.
    pub record: PathBuf,
    /// What recovery did.
    pub action: MergeRecoveryAction,
    /// The ref the record names,
    /// or nothing when the merge ran on a detached `HEAD`.
    pub target_ref: Option<String>,
    /// The merge commit the record names.
    pub new_oid: String,
    /// The side ref that keeps the merge commit,
    /// for a merge that was saved because the index was locked.
    pub side_ref: Option<String>,
}

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
        let bytes: Vec<u8> = match std::fs::read(record.as_path()) {
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
    let mut planned: Vec<(PathBuf, MergeIntent, OwnerLiveness)> = Vec::new();
    for record in list_merge_intents(git_dir)? {
        let bytes: Vec<u8> = match std::fs::read(record.as_path()) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(io_failure("reading", record.as_path(), &error)),
        };
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
            debug("recoverInterruptedMerges", format!("the owner of {} still runs", record.display()).as_str());
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
                    let said: &str = refusal.error.0.strip_suffix('.').unwrap_or(refusal.error.0.as_str());
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

/// What:
///  Whether the record's owner still runs,
/// treating a probe the account may not make as alive.
/// Why:
///   Recovery must never touch a merge that may be live.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await classifyTransactionOwnerRecord(owner, path); } catch (e) { if (e.code === 'EPERM') return 'alive'; throw e; }
/// ```
pub(crate) fn owner_liveness(intent: &MergeIntent, record: &Path) -> Result<OwnerLiveness, RecoveryError> {
    match classify_transaction_owner_record(&intent.owner, record) {
        Ok(found) => return Ok(found),
        Err(OwnerLivenessError::Probe(error)) if error.raw_os_error() == Some(libc::EPERM) => {
            return Ok(OwnerLiveness::Alive);
        }
        Err(OwnerLivenessError::Probe(error)) => {
            return Err(RecoveryError(format!(
                "probing the owner {} of the merge record {} failed: {error}",
                intent.owner.owner_pid,
                record.display()
            )));
        }
        // Owner, 2026-11-04, "Refuse and name it (Recommended)": the command stops, and nothing has been changed.
        Err(OwnerLivenessError::ForeignRecordRefused(text)) => return Err(RecoveryError(text)),
        Err(OwnerLivenessError::Identity(error)) => {
            return Err(RecoveryError(format!(
                "reading the identity of the owner {} of the merge record {} failed: {error}",
                intent.owner.owner_pid,
                record.display()
            )));
        }
    }
}
