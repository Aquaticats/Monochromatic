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
use super::diagnostic_log::debug;
/// The intent records and their files.
use super::merge_isolation_journal::{
    MergeIntent, list_merge_intents, parse_merge_intent, remove_merge_intent,
};
/// The raw differences and the worktree states.
use super::merge_isolation_changes::{
    Change, WorktreeState, parse_raw_changes, worktree_path, worktree_state,
};
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// The place recovery runs Git in.
use super::recovery_landing::RecoveryPlace;
/// Owner liveness of a record.
use super::transaction_owner::{OwnerLiveness, OwnerLivenessError, classify_transaction_owner_record};
/// Running real Git.
use super::transaction_git::{GitRequest, run_git, run_git_checked};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What recovery did for one record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergeRecoveryAction = 'owner-active' | 'discarded' | 'completed' | 'already-landed';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MergeRecoveryAction {
    /// The owner still runs; nothing was touched.
    OwnerActive,
    /// The merge never touched the worktree or the index; the record was removed.
    Discarded,
    /// Git had started updating the worktree; the merge was completed and the branch moved.
    Completed,
    /// The branch already names the merge commit; only the record was removed.
    AlreadyLanded,
}

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
///  A record of a merge that still runs must not make a nested command wait for the landing lock its ancestor holds.
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
///  Recover every intent record whose owner is dead; the caller holds the landing lock.
/// Why:
///   Recovery never races a live merge, which holds the landing lock for its whole run.
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
        let action: MergeRecoveryAction = recover_one(place, record.as_path(), &intent)?;
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
    let output = run_git(&place.git, &request)
        .map_err(|error| { return RecoveryError(format!("git rev-parse could not start: {error}")); })?;
    if !output.succeeded() {
        return Ok(None);
    }
    return Ok(Some(String::from_utf8_lossy(&output.stdout).trim().to_owned()));
}

/// What:
///  The paths the merge changes, with both sides of each.
/// Why:
///   They are exactly the paths Git updates for the merge and the only ones recovery may touch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function mergeChanges({ old, next }): Promise<Change[]>;
/// ```
fn merge_changes(place: &RecoveryPlace, root: &Path, intent: &MergeIntent) -> Result<Vec<Change>, RecoveryError> {
    let request: GitRequest = GitRequest::new(
        root,
        &["diff-tree", "-r", "-z", "--raw", "--no-renames", "--no-abbrev", intent.old_oid.as_str(), intent.new_oid.as_str()],
    );
    let output = run_git_checked(&place.git, &request).map_err(|error| { return RecoveryError(error.to_string()); })?;
    return parse_raw_changes(output.stdout.as_slice()).map_err(RecoveryError);
}

/// What:
///  The paths whose index entry differs from the old commit's tree.
/// Why:
///   A path of the merge whose entry already differs shows that Git had begun to install the merge's index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function stagedAgainst(oldOid: string): Promise<Uint8Array[]>;
/// ```
fn staged_against(place: &RecoveryPlace, root: &Path, old_oid: &str) -> Result<Vec<Vec<u8>>, RecoveryError> {
    let request: GitRequest =
        GitRequest::new(root, &["diff-index", "--cached", "--name-only", "-z", "--no-renames", old_oid, "--"]);
    let output = run_git_checked(&place.git, &request).map_err(|error| { return RecoveryError(error.to_string()); })?;
    return Ok(output
        .stdout
        .split(|byte| { return *byte == 0; })
        .filter(|path| { return !path.is_empty(); })
        .map(<[u8]>::to_vec)
        .collect());
}

/// What the worktree holds at a changed path, against the two sides of the merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Standing = 'old' | 'new' | 'foreign';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Standing {
    /// The old content, or no file where the old side has none.
    Old,
    /// The new content, or no file where the new side has none.
    New,
    /// Anything else.
    Foreign,
}

/// What:
///  Compare a worktree state with the two sides of a change.
/// Why:
///   The old side wins a tie: a change that does not alter the content is not evidence of a started merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function standing(change: Change, state: WorktreeState): Standing;
/// ```
fn standing(change: &Change, state: &WorktreeState) -> Standing {
    let old_matches: bool = match state {
        WorktreeState::Missing => !change.existed(),
        WorktreeState::Content(oid) => change.existed() && *oid == change.old_oid,
        WorktreeState::Other => false,
    };
    if old_matches {
        return Standing::Old;
    }
    let new_matches: bool = match state {
        WorktreeState::Missing => !change.exists_after(),
        WorktreeState::Content(oid) => change.exists_after() && *oid == change.new_oid,
        WorktreeState::Other => false,
    };
    if new_matches {
        return Standing::New;
    }
    return Standing::Foreign;
}

/// What:
///  Recover one record whose owner is dead.
/// Why:
///   The branch value says whether the merge landed; otherwise the paths say whether it started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverOne({ place, record, intent }): Promise<MergeRecoveryAction>;
/// ```
fn recover_one(place: &RecoveryPlace, record: &Path, intent: &MergeIntent) -> Result<MergeRecoveryAction, RecoveryError> {
    if owner_liveness(intent, record)? == OwnerLiveness::Alive {
        debug("recoverInterruptedMerges", format!("the owner of {} still runs", record.display()).as_str());
        return Ok(MergeRecoveryAction::OwnerActive);
    }
    return settle_failed_merge(place, record, intent);
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
    if current.as_deref() != Some(intent.old_oid.as_str()) {
        return Err(RecoveryError(format!(
            "{target} names {} although the interrupted merge in {} expected {}. Nothing was changed and the record \
             {} was kept. Check `git status` and `git log {target}`; remove the record when {} is as you want it.",
            current.as_deref().unwrap_or("no commit"),
            root.display(),
            intent.old_oid,
            record.display(),
            target
        )));
    }
    let changes: Vec<Change> = merge_changes(place, root, intent)?;
    let mut standings: Vec<Standing> = Vec::with_capacity(changes.len());
    for change in &changes {
        let state: WorktreeState = worktree_state(&place.git, root, change.path.as_slice()).map_err(RecoveryError)?;
        standings.push(standing(change, &state));
    }
    let staged: Vec<Vec<u8>> = staged_against(place, root, intent.old_oid.as_str())?;
    let index_started: bool = changes.iter().any(|change| { return staged.contains(&change.path); });
    // Only merged content, or an index entry of the merge, shows that Git began; another change is not its doing.
    let worktree_started: bool = standings.contains(&Standing::New);
    if !index_started && !worktree_started {
        remove_record(record)?;
        return Ok(MergeRecoveryAction::Discarded);
    }
    let foreign: Vec<String> = changes
        .iter()
        .zip(standings.iter())
        .filter(|(_, found)| { return **found == Standing::Foreign; })
        .map(|(change, _)| { return String::from_utf8_lossy(change.path.as_slice()).into_owned(); })
        .collect();
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
///  Remove a record, naming it when that fails.
/// Why:
///   A record that cannot be removed would be recovered again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(record, { force: true });
/// ```
fn remove_record(record: &Path) -> Result<(), RecoveryError> {
    return remove_merge_intent(record).map_err(|error| { return io_failure("removing", record, &error); });
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
    let mut index_input: Vec<u8> = Vec::new();
    for change in changes {
        if change.exists_after() {
            index_input.extend_from_slice(format!("{} {}\t", change.new_mode, change.new_oid).as_bytes());
        } else {
            index_input.extend_from_slice(format!("0 {}\t", "0".repeat(change.old_oid.len())).as_bytes());
        }
        index_input.extend_from_slice(change.path.as_slice());
        index_input.push(0);
    }
    let mut update: GitRequest = GitRequest::new(root, &["update-index", "-z", "--index-info"]);
    update.input = Some(index_input);
    run_git_checked(&place.git, &update).map_err(|error| { return RecoveryError(error.to_string()); })?;
    let mut checkout_input: Vec<u8> = Vec::new();
    for (change, found) in changes.iter().zip(standings.iter()) {
        let location: PathBuf = worktree_path(root, change.path.as_slice());
        if !change.exists_after() {
            remove_file_and_empty_parents(root, location.as_path())?;
        } else if *found != Standing::New {
            checkout_input.extend_from_slice(change.path.as_slice());
            checkout_input.push(0);
        }
    }
    if !checkout_input.is_empty() {
        let mut checkout: GitRequest = GitRequest::new(root, &["checkout-index", "--force", "--quiet", "-z", "--stdin"]);
        checkout.input = Some(checkout_input);
        run_git_checked(&place.git, &checkout).map_err(|error| { return RecoveryError(error.to_string()); })?;
    }
    let message: String = String::from("merge: Merge made by the 'ort' strategy. (completed by cli-git recovery)");
    let mut arguments: Vec<&str> = vec!["update-ref", "-m", message.as_str()];
    if intent.target_ref.is_none() {
        arguments.push("--no-deref");
    }
    arguments.push(intent.target_ref.as_deref().unwrap_or("HEAD"));
    arguments.push(intent.new_oid.as_str());
    arguments.push(intent.old_oid.as_str());
    run_git_checked(&place.git, &GitRequest::new(root, &arguments))
        .map_err(|error| { return RecoveryError(error.to_string()); })?;
    return Ok(());
}

/// What:
///  Delete a file the merge deletes, and the directories that it leaves empty up to the worktree top.
/// Why:
///   Git removes a deleted file's emptied directories too; a directory that still holds anything stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(file, { force: true }); for (const directory of parents(file)) { try { await rmdir(directory); } catch { break; } }
/// ```
fn remove_file_and_empty_parents(root: &Path, file: &Path) -> Result<(), RecoveryError> {
    match std::fs::remove_file(file) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(io_failure("removing", file, &error)),
    }
    let mut directory: Option<&Path> = file.parent();
    while let Some(current) = directory {
        if current == root || !current.starts_with(root) || std::fs::remove_dir(current).is_err() {
            break;
        }
        directory = current.parent();
    }
    return Ok(());
}

/// Recovery of records whose owner is dead, alive or unreadable.
#[cfg(test)]
#[path = "merge_isolation_recovery_tests.rs"]
mod tests;
