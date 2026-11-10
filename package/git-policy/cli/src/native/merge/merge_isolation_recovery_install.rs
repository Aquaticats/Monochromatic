//! What:
//!  Writing the merge into the index and the worktree,
//!  path by path and file by file,
//!  and what it does when the checkout changed under it.
//! Why:
//!  Git takes `index.lock` for the index write and dies when another program holds it,
//! so the write waits for a holder first,
//!  and one path that changed after it was judged puts all of them back and writes nothing;
//! this is the half of the recovery
//!  that writes,
//! and it has a file of its own because a kill lands in the middle of it and this is what says so
//! (`merge_isolation_recovery_steps.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const installed = await installEntries({ place, root, changes, judgements, stderr });
//! ```

use crate::inner_git_words::record_words;
use crate::lazy_runtime::run_on_runtime;
use crate::merge_isolation_changes::{Change, worktree_path};
use crate::merge_isolation_recovery_steps::{Installed, Judged, MergeRefusal, RecoveryPlace, Standing, TakenOnDisk};
use crate::merge_isolation_take_asides::{
    Take, finish_asides_awaiting, put_back, restore_what_git_did_not_write_awaiting, take_aside_awaiting,
};
use crate::recovery_error::RecoveryError;
use crate::transaction_git::GitRequest;
use crate::transaction_git_awaiting::run_git_checked_awaiting;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The refusal for a merge whose files changed after it judged them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `A merge in ${root} was interrupted, and these files changed after this command judged them: ${paths}. ...`
/// ```
fn changed_after_judging(root: &Path, paths: &[String]) -> RecoveryError {
    return RecoveryError(format!(
        "A merge in {} was interrupted while Git updated these files, and they changed after this command judged them: {}. \
         Nothing was changed by this command and the merge is still unfinished; writing the merge over them would have \
         discarded that edit. Save those edits elsewhere, restore the files with `git restore --no-enforce-worktree -- <path>`, then run any git \
         command again so recovery can finish the merge.",
        root.display(),
        paths.join(", ")
    ));
}

/// What:
///  Every file a take set aside,
///  the merge's writes and its deletions together,
///  for the paths that put them all back.
/// Why:
///  A statement that puts files back must not know which of them the merge writes and which it deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [...written, ...deleted];
/// ```
fn everything_taken(written: &[TakenOnDisk], deleted: &[TakenOnDisk]) -> Vec<TakenOnDisk> {
    let mut all: Vec<TakenOnDisk> = Vec::with_capacity(written.len() + deleted.len());
    for entry in written.iter().chain(deleted.iter()) {
        all.push(TakenOnDisk {
            location: entry.location.clone(),
            path: entry.path.clone(),
            aside: entry.aside.clone(),
            taken: entry.taken,
            expected: entry.expected.clone(),
        });
    }
    return all;
}

/// What:
///  Put the index entries and the worktree files of the changed paths to the state each change ends in.
/// Why:
///   Git takes `index.lock` for the index write and dies when another program holds it,
/// so the write waits for
///  a holder first,
/// bounded and naming it (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 2)
///  when part of the merge is already in the checkout;
/// a merge with nothing of itself in the checkout does not wait at
///  all,
/// because the next command finishes it once the lock is free
/// (finding 3 of the merge-and-pull audit of 2026-11-09).
///  A lock that outlasts the bound leaves everything as it was and the record for the next command,
/// and a lock whose PID file
///  names a process that no longer runs is cleared on the commit service's proof of disuse,
/// so the merge happens
///  instead of every command exiting 2 until the person deletes the file (finding 4 of the merge-and-pull audit
///  of 2026-11-09).
///   Every path that is not already at the state the change ends in is taken aside before anything is written,
/// and
///  the taken file is checked against what was judged,
/// so a file saved in between is never replaced with no notice
///  (finding 4);
/// one path that changed after it was judged puts all of them back and writes nothing.
///  The index entries are set first so a file written by `checkout-index` is recorded with the object it holds.
/// The
///  merge's completion and the roll back to a moved branch both end here,
/// with a different end state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installEntries({ place, root, changes, judgements, stderr }): Promise<Installed>;
/// ```
pub async fn install_entries_awaiting<W: std::io::Write + ?Sized>(
    place: &RecoveryPlace,
    root: &Path,
    changes: &[Change],
    judgements: &[Judged],
    installed_before: bool,
    out: &mut W,
) -> Result<Installed, MergeRefusal> {
    // Whether the index may be written now, and why not: `merge_isolation_recovery_index_lock.rs`.
    if !crate::merge_isolation_recovery_index_lock::may_write(place.git_dir.as_path(), installed_before, out) {
        return Ok(Installed::IndexHeld);
    }
    // The take comes first: nothing of the merge is written while a file that was judged may already have changed.
    let mut taken_writes: Vec<TakenOnDisk> = Vec::new();
    let mut taken_deletions: Vec<TakenOnDisk> = Vec::new();
    let mut checkout_input: Vec<u8> = Vec::new();
    let mut changed: Vec<String> = Vec::new();
    for (change, judged) in changes.iter().zip(judgements.iter()) {
        let location: PathBuf = worktree_path(root, change.path.as_slice());
        if judged.standing == Standing::New {
            // The path already holds the state the change ends in.
            continue;
        }
        if change.exists_after() {
            // Every path the merge writes is checked out, whether a file stood there or not.
            checkout_input.extend_from_slice(change.path.as_slice());
            checkout_input.push(0);
        }
        if judged.standing == Standing::Directory {
            // The directory holds only the branch's files, which the merge's own deletions take, so there is nothing of
            //  anyone's to set aside; the deletions empty it before the checkout writes the file.
            continue;
        }
        match take_aside_awaiting(place, root, change.path.as_slice(), location.as_path(), judged).await {
            Ok(Take::Nothing) => {}
            Ok(Take::Changed) => changed.push(String::from_utf8_lossy(change.path.as_slice()).into_owned()),
            Ok(Take::Aside { aside, taken, expected }) => {
                let entry: TakenOnDisk = TakenOnDisk { location, path: change.path.clone(), aside, taken, expected };
                if change.exists_after() {
                    taken_writes.push(entry);
                } else {
                    taken_deletions.push(entry);
                }
            }
            Err(failure) => {
                // A take that stops must not leave the files it already took under their private names; the one it stopped on is
                //  in the failure, because the rename came before the read that failed.
                let left: Vec<String> = put_back(&everything_taken(&taken_writes, &taken_deletions));
                let mut refusal: String = failure.error.0;
                if !left.is_empty() {
                    refusal.push_str(format!(" Files that could not be put back: {}", left.join(", ")).as_str());
                }
                let began: bool = installed_before || failure.stranded.is_some() || !left.is_empty();
                return Err(MergeRefusal { error: RecoveryError(refusal), began });
            }
        }
    }
    if !changed.is_empty() {
        let left: Vec<String> = put_back(&everything_taken(&taken_writes, &taken_deletions));
        let mut refusal: String = changed_after_judging(root, changed.as_slice()).0;
        refusal.push_str(left.join(", ").as_str());
        return Err(MergeRefusal { error: RecoveryError(refusal), began: installed_before || !left.is_empty() });
    }
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
    if let Err(error) = run_git_checked_awaiting(&place.git, &update).await {
        // The files are aside and the index is untouched: put them back rather than leave half a merge.
        let left: Vec<String> = put_back(&everything_taken(&taken_writes, &taken_deletions));
        return Err(MergeRefusal { error: RecoveryError(error.to_string()), began: installed_before || !left.is_empty() });
    }
    // From here the index holds the merge's entries, so a command that stops must not go on reading the checkout.
    let began: bool = true;
    // A deleted path's file was taken aside, so its directory can be reached even when the merge leaves it empty; the
    //  directories go once every aside name in them is gone,
    // and they go before the checkout,
    // because a merge may put a file
    //  where one of them stood.
    let mut deletions: Vec<PathBuf> = Vec::new();
    for change in changes {
        if change.exists_after() {
            continue;
        }
        // Every path the merge deletes has its emptied directories pruned,
        // whether or not it already stands at its end state:
        // a
        //  recovery killed between finishing a child's private name and removing the directory it emptied leaves that
        //  directory behind,
        // and only an empty directory is ever removed here.
        deletions.push(worktree_path(root, change.path.as_slice()));
    }
    if let Err(error) = finish_asides_awaiting(place, root, &taken_deletions, out).await {
        return Err(MergeRefusal { error, began });
    }
    for location in &deletions {
        remove_empty_parents(root, location.as_path());
    }
    if !checkout_input.is_empty() {
        // Without `--force`: a file that appeared where the merge writes is skipped and Git says so,
        //  so the person's bytes are never replaced in the instant between the take and this write.
        let mut checkout: GitRequest = GitRequest::new(root, &["checkout-index", "-z", "--stdin"]);
        checkout.input = Some(checkout_input);
        let checked_out = match run_git_checked_awaiting(&place.git, &checkout).await {
            Ok(done) => done,
            Err(error) => {
                let restored: Result<(), RecoveryError> = restore_what_git_did_not_write_awaiting(place, root, &taken_writes, out).await;
                return Err(match restored {
                    Ok(()) => MergeRefusal { error: RecoveryError(error.to_string()), began },
                    Err(left) => MergeRefusal { error: RecoveryError(format!("{error} ({})", left.0)), began },
                });
            }
        };
        // The checkout carries no `--quiet`; what Git said of it is kept in the journal.
        record_words(&["checkout-index", "-z", "--stdin"], &checked_out);
    }
    if let Err(error) = finish_asides_awaiting(place, root, &taken_writes, out).await {
        return Err(MergeRefusal { error, began });
    }
    return Ok(Installed::Written);
}

/// What:
///  The same install of a merge's index entries and files,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The install starts Git twice,
/// and a synchronous start registers the step's pipes by entering the runtime,
///  which is refused from inside it;
/// the notices are the awaited body's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installEntries({ place, root, changes, judgements, stderr }): Promise<Installed>;
/// ```
pub fn install_entries(
    place: &RecoveryPlace,
    root: &Path,
    changes: &[Change],
    judgements: &[Judged],
    installed_before: bool,
    out: &mut dyn Write,
) -> Result<Installed, MergeRefusal> {
    return match run_on_runtime(install_entries_awaiting(place, root, changes, judgements, installed_before, out)) {
        Ok(found) => found,
        // The runtime refused before anything was written, so nothing of the merge is in the checkout.
        Err(refusal) => Err(MergeRefusal { error: RecoveryError(format!("{refusal}")), began: installed_before }),
    };
}

/// What:
///  Remove the directories a deleted path leaves empty up to the worktree top.
/// Why:
///   Git removes a deleted file's emptied directories too,
/// and the take has already removed the file itself;
/// a
///  directory that still holds anything stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const directory of parents(file)) { try { await rmdir(directory); } catch { break; } }
/// ```
pub fn remove_empty_parents(root: &Path, file: &Path) {
    let mut directory: Option<&Path> = file.parent();
    while let Some(current) = directory {
        if current == root || !current.starts_with(root) || std::fs::remove_dir(current).is_err() {
            break;
        }
        directory = current.parent();
    }
}
