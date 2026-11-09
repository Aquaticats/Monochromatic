//! What:
//!  Recovery of an isolated merge whose process died:
//! discard it when it never touched the worktree or the index,
//!  complete it when Git had started to update them.
//! Why:
//!  The merge commit exists before the branch moves,
//! and Git updates the worktree and the index of the paths the
//!  merge changes before it moves the branch.
//!  A process killed in between leaves files and index entries of the merge beside a branch that still names the
//!  old commit,
//! which the next commit would then revert.
//!  Completing the merge from its record finishes exactly what Git had started,
//! for exactly the paths the merge
//!  changes,
//! and leaves every other staged entry and every unstaged change as it was.
//!  A path that holds neither its old nor its new content belongs to someone else,
//! so recovery stops and keeps the
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
/// The steps shared with the settling of a moved branch;
/// named here so the paths outside this file stay the same.
pub use crate::merge_isolation_recovery_cleanup::{drop_record_and_side_ref, partly_installed, untouched};
pub use crate::merge_isolation_recovery_notice::recovery_notice;
pub use crate::merge_isolation_recovery_steps::{
    Installed, Judged, MergeRecoveryAction, MergeRefusal, RecoveryPlace, Standing, TakenOnDisk, finish_asides,
};
use crate::merge_isolation_recovery_cleanup::{drop_record, drop_side_ref};
use crate::merge_isolation_recovery_steps::{
    index_standings, install_entries, judge_changes, merge_changes, put_back_stranded_takes,
};
/// The raw differences and the worktree states.
use crate::merge_isolation_changes::Change;
/// The fail-closed recovery failure.
use crate::recovery_error::{RecoveryError, io_failure};
/// Owner liveness of a record.
use crate::transaction_owner::{OwnerLiveness, OwnerLivenessError, classify_transaction_owner_record};
/// Running real Git.
use crate::transaction_git::{GitRequest, run_git, run_git_checked};
/// Writing the notices of what recovery did.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
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
            match settle_failed_merge(place, record.as_path(), &intent, out) {
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
                    return Err(RecoveryError(format!(
                        "A merge saved on {side_ref} for {target} is partly installed in this worktree: {}. Nothing more was changed \
                         and the record {} was kept; check `git status` and `git stash list` before you commit anything.",
                        refusal.error.0,
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
///  Whether an index entry is the merge's own,
/// which is the only evidence that Git began to install it.
/// Why:
///   Named rather than written as a closure inside `any`,
/// and it says what the check means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsTheMergesEntry = (standing: Standing) => standing === 'new';
/// ```
fn holds_the_merges_entry(standing: &Standing) -> bool {
    return *standing == Standing::New;
}

/// What:
///  Whether the worktree holds the merge's content at this path,
/// which is the only evidence that Git began.
/// Why:
///   Named rather than written as a closure inside `any`,
/// and it says what the check means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsTheMerge = (judged: Judged) => judged.standing === 'new';
/// ```
fn holds_the_merge(judged: &Judged) -> bool {
    return judged.standing == Standing::New;
}

/// What:
///  The branch `HEAD` names in `root`,
/// or nothing when it names a commit itself.
/// Why:
///   The checkout a merge installs into is the one its branch is checked out in;
/// `symbolic-ref` says which branch that is,
/// and fails
///  on a detached `HEAD`,
/// which the record names with no target ref of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = await git(['symbolic-ref', '--quiet', 'HEAD']); return named.succeeded ? named.stdout.trim() : undefined;
/// ```
fn symbolic_ref(place: &RecoveryPlace, root: &Path) -> Result<Option<String>, RecoveryError> {
    let request: GitRequest = GitRequest::new(root, &["symbolic-ref", "--quiet", "HEAD"]);
    let output = match run_git(&place.git, &request) {
        Ok(done) => done,
        Err(error) => return Err(RecoveryError(format!("git symbolic-ref could not start: {error}"))),
    };
    if !output.succeeded() {
        return Ok(None);
    }
    return Ok(Some(String::from_utf8_lossy(&output.stdout).trim().to_owned()));
}

/// What:
///  The commit a ref names,
/// or nothing when it names none.
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
///  Settle a merge that is known not to be running:
/// discard it if it never began,
/// otherwise finish it.
/// Why:
///   The merge's own process calls this when Git reported a failure,
/// and recovery calls it for a dead owner;
/// both
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
    out: &mut dyn Write,
) -> Result<MergeRecoveryAction, MergeRefusal> {
    let root: &Path = Path::new(intent.repository_root.as_str());
    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
    let current: Option<String> = resolve_commit(place, root, target).map_err(untouched)?;
    if current.as_deref() == Some(intent.new_oid.as_str()) {
        drop_record_and_side_ref(place, root, record, intent).map_err(untouched)?;
        return Ok(MergeRecoveryAction::AlreadyLanded);
    }
    // The checkout belongs to the branch the merge was computed on: installing it into another branch's worktree would put
    //  this merge's files and index into a branch it is not for, and move a branch from a checkout that is not its own.
    let checked_out: Option<String> = symbolic_ref(place, root).map_err(untouched)?;
    if checked_out != intent.target_ref {
        return Err(untouched(RecoveryError(format!(
            "A merge saved for {target} in {} was not finished: this worktree has {} checked out, and finishing the merge would \
             put it into that branch's checkout instead. Nothing was changed, the record {} was kept and the merge's commit is \
             still where it was saved. Check out {target} in this worktree and run any git command again.",
            root.display(),
            checked_out.as_deref().unwrap_or("a detached HEAD"),
            record.display()
        ))));
    }
    match current.as_deref() {
        Some(head) if head == intent.old_oid => {}
        Some(head) => {
            if intent.side_ref.is_some() {
                // A merge that a command already reported as saved must not vanish: the record goes, so nothing stops a later
                //  command, and the side ref keeps its commit and is named.
                drop_record(record).map_err(untouched)?;
                return Ok(MergeRecoveryAction::SavedBranchMoved);
            }
            let action: MergeRecoveryAction = settle_moved_branch(place, record, intent, head, out)?;
            if action == MergeRecoveryAction::BranchMoved {
                drop_side_ref(place, root, intent);
            }
            return Ok(action);
        }
        None => {
            return Err(untouched(RecoveryError(format!(
                "{target} names no commit although the interrupted merge in {} expected {}. Nothing was changed and the \
                 record {} was kept. Check `git status` and `git log {target}`; remove the record when {target} is as you \
                 want it.",
                root.display(),
                intent.old_oid,
                record.display()
            ))));
        }
    }
    let changes: Vec<Change> = merge_changes(place, root, intent).map_err(untouched)?;
    // A file a command that stopped left under its private name is put back before the paths are judged: the path is empty
    //  while the file waits beside it, and would otherwise be read as someone else's.
    put_back_stranded_takes(root, changes.as_slice(), out);
    let judgements: Vec<Judged> = judge_changes(place, root, changes.as_slice()).map_err(untouched)?;
    let index_standing: Vec<Standing> = index_standings(place, root, intent.old_oid.as_str(), changes.as_slice()).map_err(untouched)?;
    let index_started: bool = index_standing.iter().any(holds_the_merges_entry);
    // Only merged content, or an index entry of the merge, shows that Git began; another change is not its doing.
    let worktree_started: bool = judgements.iter().any(holds_the_merge);
    // A merge that was saved on a side ref is finished even though nothing of it is in the checkout yet: the record with a side
    //  ref is what says the merge should land, not the checkout (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
    //  finding 3).
    if intent.side_ref.is_none() && !index_started && !worktree_started {
        drop_record_and_side_ref(place, root, record, intent).map_err(untouched)?;
        return Ok(MergeRecoveryAction::Discarded);
    }
    let mut foreign: Vec<String> = Vec::new();
    for (change, judged) in changes.iter().zip(judgements.iter()) {
        if judged.standing == Standing::Foreign {
            foreign.push(String::from_utf8_lossy(change.path.as_slice()).into_owned());
        }
    }
    if !foreign.is_empty() {
        return Err(untouched(RecoveryError(format!(
            "A merge in {} was interrupted while Git updated these files, and they now hold neither their old nor \
             their merged content: {}. Nothing was changed and the record {} was kept. Save your edits to them \
             elsewhere, restore them with `git restore -- <path>`, then run any git command again so recovery can \
             finish the merge; or remove the record to abandon the merge and check `git status` yourself.",
            root.display(),
            foreign.join(", "),
            record.display()
        ))));
    }
    // A staged version of a path the merge replaces has no other name, so it is never written over: the finish stops and names
    //  the paths, and the staged content stays in the index.
    let mut staged: Vec<String> = Vec::new();
    for (change, standing) in changes.iter().zip(index_standing.iter()) {
        if *standing == Standing::Foreign {
            staged.push(String::from_utf8_lossy(change.path.as_slice()).into_owned());
        }
    }
    if !staged.is_empty() {
        return Err(untouched(RecoveryError(format!(
            "A merge of {target} in {} is unfinished, and the index holds staged content for these paths that the merge would \
             replace: {}. Nothing was changed, the record {} was kept, and the staged content is still in the index. Commit or \
             unstage it, then run any git command again so recovery can finish the merge; or remove the record to abandon the \
             merge and check `git status` yourself.",
            root.display(),
            staged.join(", "),
            record.display()
        ))));
    }
    // A lock that outlasts the wait leaves everything as it was; the record stays for the next command, which finishes the
    //  same merge (the recovery is idempotent).
    if complete(place, root, intent, &changes, &judgements, out)? == Installed::IndexHeld {
        return Ok(MergeRecoveryAction::IndexHeld);
    }
    drop_record_and_side_ref(place, root, record, intent).map_err(partly_installed)?;
    return Ok(MergeRecoveryAction::Completed);
}

/// What:
///  Bring the index entries and the worktree files of the changed paths to the merged state,
/// then move the branch.
/// Why:
///   The index entries are set first so a file written by `checkout-index` is recorded with the object it holds;
///  deletions go before writes so a file replaced by a directory of the same name can be created;
///  the branch moves last,
/// by compare-and-swap against the old commit,
/// so a branch that moved meanwhile is left alone.
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
    judgements: &[Judged],
    out: &mut dyn Write,
) -> Result<Installed, MergeRefusal> {
    if install_entries(place, root, changes, judgements, out)? == Installed::IndexHeld {
        return Ok(Installed::IndexHeld);
    }
    let message: String = String::from("merge: Merge made by the 'ort' strategy. (completed by monochromatic-git-policy-cli recovery)");
    let mut arguments: Vec<&str> = vec!["update-ref", "-m", message.as_str()];
    if intent.target_ref.is_none() {
        arguments.push("--no-deref");
    }
    arguments.push(intent.target_ref.as_deref().unwrap_or("HEAD"));
    arguments.push(intent.new_oid.as_str());
    arguments.push(intent.old_oid.as_str());
    if let Err(error) = run_git_checked(&place.git, &GitRequest::new(root, &arguments)) {
        // The index and every file of the merge are in place and the branch did not move.
        return Err(MergeRefusal { error: RecoveryError(error.to_string()), began: true });
    }
    return Ok(Installed::Written);
}

/// Recovery of records whose owner is dead,
/// alive or unreadable.
#[cfg(test)]
#[path = "merge_isolation_recovery_tests.rs"]
mod tests;
