//! What:
//!  Recovery of an isolated merge whose process died: discard it when it never touched the worktree or the index,
//!  complete it when Git had started to update them.
//! Why:
//!  The merge commit exists before the branch moves, and Git updates the worktree and the index of the paths the
//!  merge changes before it moves the branch.
//!  A process killed in between leaves files and index entries of the merge beside a branch that still names the
//!  old commit, which the next commit would then revert.
//!  Completing the merge from its record finishes exactly what Git had started, for exactly the paths the merge
//!  changes, and leaves every other staged entry and every unstaged change as it was.
//!  A path that holds neither its old nor its new content belongs to someone else, so recovery stops and keeps the
//!  record instead of overwriting it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const record of await listMergeIntents(gitDir)) await recoverMerge({ record, place });
//! ```

/// Debug diagnostics.
use crate::diagnostic_log::debug;
/// The intent records and their files.
use crate::merge_isolation_journal::{
    MergeIntent, list_merge_intents, parse_merge_intent,
};
/// A merge whose branch moved while it was prepared.
use crate::merge_isolation_moved::settle_moved_branch;
/// The steps shared with the settling of a moved branch; named here so the paths outside this file stay the same.
pub use crate::merge_isolation_recovery_steps::{MergeRecoveryAction, RecoveryPlace, Standing};
use crate::merge_isolation_recovery_steps::{index_holds_the_merge, install_entries, merge_changes, remove_record, standing};
/// The raw differences and the worktree states.
use crate::merge_isolation_changes::{
    Change, WorktreeState, worktree_state,
};
/// The fail-closed recovery failure.
use crate::recovery_error::{RecoveryError, io_failure};
/// Owner liveness of a record.
use crate::transaction_owner::{OwnerLiveness, OwnerLivenessError, classify_transaction_owner_record};
/// Running real Git.
use crate::transaction_git::{GitRequest, run_git, run_git_checked};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// One record recovery visited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergeRecoveryOutcome = { record: string; action: MergeRecoveryAction };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergeRecoveryOutcome {
    /// The record's file.
    pub record: PathBuf,
    /// What recovery did.
    pub action: MergeRecoveryAction,
}

/// What:
///  Whether the worktree's Git directory holds an intent record that recovery must look at: one whose owner is
///  dead, or one that cannot be read.
/// Why:
///   The common case needs one failed directory read and no lock.
///  A record of a merge that still runs is not recovered, so a nested command does not touch its ancestor's merge.
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
///   A record whose owner still runs is left alone, so recovery does not race a live merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverInterruptedMerges({ place, gitDir }): Promise<MergeRecoveryOutcome[]>;
/// ```
pub fn recover_interrupted_merges(
    place: &RecoveryPlace,
    git_dir: &Path,
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
    for (record, intent, liveness) in planned {
        let action: MergeRecoveryAction = if liveness == OwnerLiveness::Alive {
            debug("recoverInterruptedMerges", format!("the owner of {} still runs", record.display()).as_str());
            MergeRecoveryAction::OwnerActive
        } else {
            settle_failed_merge(place, record.as_path(), &intent)?
        };
        outcomes.push(MergeRecoveryOutcome { record, action });
    }
    return Ok(outcomes);
}

/// What:
///  Whether the record's owner still runs, treating a probe the account may not make as alive.
/// Why:
///   Recovery must never touch a merge that may be live.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await classifyTransactionOwnerRecord(owner, path); } catch (e) { if (e.code === 'EPERM') return 'alive'; throw e; }
/// ```
fn owner_liveness(intent: &MergeIntent, record: &Path) -> Result<OwnerLiveness, RecoveryError> {
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
        Err(OwnerLivenessError::ExpiredForeign(text)) => return Err(RecoveryError(text)),
        Err(OwnerLivenessError::Identity(error)) => {
            return Err(RecoveryError(format!(
                "reading the identity of the owner {} of the merge record {} failed: {error}",
                intent.owner.owner_pid,
                record.display()
            )));
        }
    }
}

/// What:
///  The commit a ref names, or nothing when it names none.
/// Why:
///   The branch value decides whether the merge landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveCommit(reference: string): Promise<string | undefined>;
/// ```
fn resolve_commit(place: &RecoveryPlace, root: &Path, reference: &str) -> Result<Option<String>, RecoveryError> {
    let revision: String = format!("{reference}^{{commit}}");
    let request: GitRequest = GitRequest::new(root, &["rev-parse", "--verify", "--quiet", revision.as_str()]);
    let output = match run_git(&place.git, &request) {
        Ok(done) => done,
        Err(error) => return Err(RecoveryError(format!("git rev-parse could not start: {error}"))),
    };
    if !output.succeeded() {
        return Ok(None);
    }
    return Ok(Some(String::from_utf8_lossy(&output.stdout).trim().to_owned()));
}

/// What:
///  Settle a merge that is known not to be running: discard it if it never began, otherwise finish it.
/// Why:
///   The merge's own process calls this when Git reported a failure, and recovery calls it for a dead owner; both
///  need the same decision from the same evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function settleFailedMerge({ place, record, intent }): Promise<MergeRecoveryAction>;
/// ```
pub fn settle_failed_merge(
    place: &RecoveryPlace,
    record: &Path,
    intent: &MergeIntent,
) -> Result<MergeRecoveryAction, RecoveryError> {
    let root: &Path = Path::new(intent.repository_root.as_str());
    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
    let current: Option<String> = resolve_commit(place, root, target)?;
    if current.as_deref() == Some(intent.new_oid.as_str()) {
        remove_record(record)?;
        return Ok(MergeRecoveryAction::AlreadyLanded);
    }
    match current.as_deref() {
        Some(head) if head == intent.old_oid => {}
        Some(head) => return settle_moved_branch(place, record, intent, head),
        None => {
            return Err(RecoveryError(format!(
                "{target} names no commit although the interrupted merge in {} expected {}. Nothing was changed and the \
                 record {} was kept. Check `git status` and `git log {target}`; remove the record when {target} is as you \
                 want it.",
                root.display(),
                intent.old_oid,
                record.display()
            )));
        }
    }
    let changes: Vec<Change> = merge_changes(place, root, intent)?;
    let mut standings: Vec<Standing> = Vec::with_capacity(changes.len());
    for change in &changes {
        let state: WorktreeState = worktree_state(&place.git, root, change.path.as_slice()).map_err(RecoveryError)?;
        standings.push(standing(change, &state));
    }
    let index_started: bool = index_holds_the_merge(place, root, intent.old_oid.as_str(), changes.as_slice())?;
    // Only merged content, or an index entry of the merge, shows that Git began; another change is not its doing.
    let worktree_started: bool = standings.contains(&Standing::New);
    if !index_started && !worktree_started {
        remove_record(record)?;
        return Ok(MergeRecoveryAction::Discarded);
    }
    let mut foreign: Vec<String> = Vec::new();
    for (change, found) in changes.iter().zip(standings.iter()) {
        if *found == Standing::Foreign {
            foreign.push(String::from_utf8_lossy(change.path.as_slice()).into_owned());
        }
    }
    if !foreign.is_empty() {
        return Err(RecoveryError(format!(
            "A merge in {} was interrupted while Git updated these files, and they now hold neither their old nor \
             their merged content: {}. Nothing was changed and the record {} was kept. Save your edits to them \
             elsewhere, restore them with `git restore -- <path>`, then run any git command again so recovery can \
             finish the merge; or remove the record to abandon the merge and check `git status` yourself.",
            root.display(),
            foreign.join(", "),
            record.display()
        )));
    }
    complete(place, root, intent, &changes, &standings)?;
    remove_record(record)?;
    return Ok(MergeRecoveryAction::Completed);
}

/// What:
///  Bring the index entries and the worktree files of the changed paths to the merged state, then move the branch.
/// Why:
///   The index entries are set first so a file written by `checkout-index` is recorded with the object it holds;
///  deletions go before writes so a file replaced by a directory of the same name can be created;
///  the branch moves last, by compare-and-swap against the old commit, so a branch that moved meanwhile is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function complete({ place, root, intent, changes, standings }): Promise<void>;
/// ```
fn complete(
    place: &RecoveryPlace,
    root: &Path,
    intent: &MergeIntent,
    changes: &[Change],
    standings: &[Standing],
) -> Result<(), RecoveryError> {
    install_entries(place, root, changes, standings)?;
    let message: String = String::from("merge: Merge made by the 'ort' strategy. (completed by cli-git recovery)");
    let mut arguments: Vec<&str> = vec!["update-ref", "-m", message.as_str()];
    if intent.target_ref.is_none() {
        arguments.push("--no-deref");
    }
    arguments.push(intent.target_ref.as_deref().unwrap_or("HEAD"));
    arguments.push(intent.new_oid.as_str());
    arguments.push(intent.old_oid.as_str());
    if let Err(error) = run_git_checked(&place.git, &GitRequest::new(root, &arguments)) {
        return Err(RecoveryError(error.to_string()));
    }
    return Ok(());
}

/// Recovery of records whose owner is dead, alive or unreadable.
#[cfg(test)]
#[path = "merge_isolation_recovery_tests.rs"]
mod tests;
