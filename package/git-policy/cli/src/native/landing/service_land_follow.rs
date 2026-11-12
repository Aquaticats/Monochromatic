//! What:
//!  What the service's landing does once the branch has moved:
//! the worktree's index entries of the commit's paths and the
//!  conclusion state a merge,
//! cherry-pick or revert leaves behind.
//! Why:
//!  A commit Git built in a scratch repository carries no list of changes,
//! and a conclusion's state files belong to the
//!  commit that concludes it.
//! Both steps are idempotent,
//! so the same code finishes a batch a killed service left open.
//!  The index entries come from the repository itself:
//! the paths that differ between the branch before and the commit,
//!  as bytes,
//! handed to `reset` on its standard input,
//! so a name that is not UTF-8,
//! a long list and a gitlink need no
//!  special case (`doc/planning/cli-git-resident-service.md`,
//! "Landing every command line").
//! Each step has two faces:
//! the synchronous one a caller on a thread asks (`reset_entries`),
//! and the awaited one a
//!  task of the runtime asks (`reset_entries_awaiting`,
//! `derived_names_awaiting`),
//! which awaits every Git child on the runtime.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await resetEntriesAwaiting(git, repository, index, newCommit, paths); await removeConclusionState(git, repository, cleanup);
//! ```

use crate::autostash_carry::carry_stash;
use crate::conclusion_cleanup::reproduce_conclusion_cleanup;
use crate::diagnostic_log::warn;
use crate::index_holder_wait::{IndexWait, wait_for_index_release_within};
use crate::wait_ceilings::RECONCILE_LOCK_CEILING;
use crate::wait_progress::Cadence;
use crate::service_index_lock::IndexLock;
use crate::commit_index_stat_refresh::{StatRefresh, refresh_stat_data_when_stale};
use crate::commit_index_stat_staleness::WorktreeStaging;
pub use crate::service_land_copy::PrivateCopy;
use crate::service_protocol::{Cleanup, FailureCode};
use crate::transaction_git::GitContext;
use crate::transaction_journal::RefFormat;
use std::ffi::OsString;
use std::path::Path;

/// How many times the index update is tried again when the lock was gone by the time it was looked at:
/// the holder let go
///  between Git's refusal and the look,
/// so the next try is expected to take the lock.
///  Measured on 2026-11-03 on this host,
/// on a copy of this repository's index (20,739 tracked files,
/// 2.9 MB),
/// with 20 runs each
///  of `git status`,
/// `git add -A`,
/// `git reset -q` and `git update-index --refresh`:
/// Git held `index.lock` for at most 100 ms
///  (`add -A`;
/// medians 26 to 53 ms,
/// `status` at most 55 ms,
/// `reset` at most 71 ms,
/// `update-index` at most 77 ms).
/// Five tries
///  of that hold cover five holds in a row.
/// A holder that is still there is waited for by `index_holder_wait.rs`
/// until that wait's own ceiling (`MERGE_LOCK_CEILING`),
///  and does not use these tries.
const INDEX_ATTEMPTS: usize = 5;

/// The pause between index tries when the lock was gone:
/// the longest hold measured for one of Git's own short commands (see
///  `INDEX_ATTEMPTS`).
const INDEX_PAUSE: std::time::Duration = std::time::Duration::from_millis(100);

/// The entries a landing installs in the index and the reset that installs them,
/// kept in a file of their own to stay under the file-size limit.
#[path = "service_land_follow_index_reset.rs"]
mod index_reset;
pub use self::index_reset::{IndexPaths, reset_entries, reset_entries_awaiting};
pub(crate) use self::index_reset::{classify, derived_names};
use self::index_reset::reset_once;
/// The awaited derived names, read by the control that compares both faces of the reset.
#[cfg(test)]
use self::index_reset::derived_names_awaiting;

/// What:
///  Updates the index entries of the commit's paths to the commit `new`,
/// waiting while another program holds the index.
/// Why:
///   Only Git's own index lock can make this fail for a moment.
/// A holder is waited for by `index_holder_wait.rs` until `ceiling`,
/// which the caller of this function sets to `RECONCILE_LOCK_CEILING`,
/// and a holder whose PID file names a process that is gone ends the wait at once;
/// a lock that was gone by the
///  time it was looked at is tried again a few times.
/// An empty list changes nothing,
/// because `reset` with no path would
///  reset the whole index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await updateIndex(git, repository, index, newCommit, paths)
/// ```
pub fn update_index(git: &GitContext, repository: &Path, index: &Path, new: &str, paths: IndexPaths<'_>) -> Result<(), FailureCode> {
    return update_index_within(git, repository, index, RECONCILE_LOCK_CEILING, new, paths);
}

/// `update_index` that gives up on a holder of the index lock after `ceiling`,
/// with the lock's code.
///
/// # Errors
///
/// As `update_index`.
pub fn update_index_within(git: &GitContext, repository: &Path, index: &Path, ceiling: std::time::Duration, new: &str, paths: IndexPaths<'_>) -> Result<(), FailureCode> {
    if matches!(paths, IndexPaths::Listed(listed) if listed.is_empty()) || matches!(paths, IndexPaths::Named(named) if named.is_empty()) {
        return Ok(());
    }
    // Tries that met no holder to wait for: a refusal with the lock already gone, or a failure that is not the lock's.
    let mut tries: usize = 0;
    // One ceiling for the whole update, not one per hold: a hold that ends and is taken again is waited for under this deadline, so
    //  no schedule of handoffs keeps the command waiting.
    let deadline: std::time::Instant = std::time::Instant::now() + ceiling;
    loop {
        let code: FailureCode = match reset_once(git, repository, new, paths, None) {
            Ok(()) => return Ok(()),
            Err(code) => code,
        };
        if code == FailureCode::IndexInUse {
            // The wait gets what is left of the one ceiling, so a hold that ends and is taken again cannot add a full wait.
            let left: std::time::Duration = deadline.saturating_duration_since(std::time::Instant::now());
            if left.is_zero() {
                return Err(code);
            }
            match wait_for_index_release_within(index, "commit", left, Cadence::from_environment(), &mut std::io::stderr()) {
                // The diagnostic of a wait that ended on an owner this table's numbers do not describe is already written.
                IndexWait::OwnerGone(_) | IndexWait::GaveUp(_) => return Err(code),
                // A holder let go: the index is tried again at once, and a holder that comes next is waited for as well, under the
                //  same deadline.
                IndexWait::Released(_) => {
                    if std::time::Instant::now() >= deadline {
                        return Err(code);
                    }
                    continue;
                }
                IndexWait::NotHeld => {}
            }
        }
        tries += 1;
        if tries >= INDEX_ATTEMPTS {
            return Err(code);
        }
        std::thread::sleep(INDEX_PAUSE);
    }
}

/// What:
///  Builds the index the landing will install,
/// in a file of its own beside the real index:
/// a copy of the real index with the
///  commit's paths reset to the commit and the worktree's stat data recorded.
/// Why:
///   The landing holds Git's `index.lock`,
/// so Git's own `reset` cannot take it;
/// the new index is made in another file and then
///  moved into place by the lock's release,
/// which is how Git installs an index.
///  The stat data is refreshed here and nowhere else:
/// a preparation never writes the real index
/// (`commit_index_stat_refresh.rs`),
/// and the commit after this one copies the index this landing installs,
/// so a batch pays one refresh,
/// in the step that already holds the lock,
/// instead of one per client that then waits for the lock the landing needs.
///  The copy keeps the live index's times through every write of it,
/// so Git's own question about the stat data of an entry it has not read keeps the answer the live index gave
///  (`PrivateCopy::keep_the_live_times`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const built = await buildIndex(git, repository, newCommit, paths, indexPath, seq)
/// ```
pub fn build_index(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, lock: &IndexLock, seq: u64) -> Result<PrivateCopy, FailureCode> {
    let copy: PrivateCopy = PrivateCopy::make(lock, seq)?;
    let built: std::path::PathBuf = copy.path().to_path_buf();
    let empty: bool = matches!(paths, IndexPaths::Listed(listed) if listed.is_empty()) || matches!(paths, IndexPaths::Named(named) if named.is_empty());
    let mut written: Vec<Vec<u8>> = Vec::new();
    if !empty {
        match reset_in_process(git, repository, new, paths, (lock, &copy)) {
            Some(names) => written = names,
            None => {
                reset_once(git, repository, new, paths, Some(built.as_path()))?;
                written = named_paths(paths);
            }
        }
    }
    keep_the_stat_data(git, repository, written.as_slice(), &copy);
    return Ok(copy);
}

/// What:
///  Gives the copy the live index's times back,
/// whatever a step of the landing wrote into it,
/// and never fails the landing for it.
/// Why:
///   A copy installed as the live index must be as old as the stat data its entries describe,
/// or Git trusts a stat record whose file it would otherwise read again
///  (`PrivateCopy::keep_the_live_times`).
///  The times are a cache for the reader and never the landing's result,
/// so a file system that refuses them leaves the copy installed and says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copy.keepTheLiveTimes() // logged, never thrown
/// ```
pub(crate) fn put_the_live_times_back(copy: &PrivateCopy) {
    if let Err(error) = copy.keep_the_live_times() {
        crate::diagnostic_log::warn(
            "serviceLand",
            format!(
                "the index {} keeps the time it was written at, so Git may trust stat data it would otherwise read again ({error})",
                copy.path().display()
            )
            .as_str(),
        );
    }
}

/// What:
///  Records the worktree's stat data in the index the landing is about to install.
/// Why:
///   The copy was made from the live index,
/// so every entry whose file moved under that index is stale in it,
/// and a commit that copies this index then reads and hashes every such file.
///  A refresh that fails or is interrupted costs the landing its time and never its result:
/// stat data is a cache,
/// the copy is installed either way,
/// and the step is ended at its class's ceiling
/// (`transaction_git_step_rerun_and_end_rules.rs`).
///  The paths the reset just wrote are left out of the question:
/// their entries hold the commit's own content,
/// and a path the person has since edited must keep no stat data that would hide the edit,
/// which is what `update-index --refresh` does with it anyway
/// (`commit_index_stat_staleness.rs`).
///  The copy is as old as the live index while Git answers that question,
/// because a stat check compares the fields it reads only to the second:
/// an edit made in the same second as the stat the live index records differs from it in nothing Git compares,
///  so only the entry being newer than the index makes Git read the file.
///  A copy written a moment ago would be newer than every entry,
/// Git would answer from the recorded stat,
/// and the person's edit would reach neither the next commit nor `git status`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await keepTheStatData({ git, repository, written, copy }) // copy.keepTheLiveTimes(); refresh_if_stale(copy, staging = `written`); copy.keepTheLiveTimes()
/// ```
fn keep_the_stat_data(git: &GitContext, repository: &Path, written: &[Vec<u8>], copy: &PrivateCopy) {
    let left_out: Vec<OsString> = exempt_names(written);
    put_the_live_times_back(copy);
    // What the copy looked like before the refresh, so the release flushes it only when the refresh rewrote it.
    let untouched: Option<(u64, std::time::SystemTime)> = file_stamp(copy.path());
    let _outcome: StatRefresh = refresh_stat_data_when_stale(git, repository, copy.path(), WorktreeStaging::Named(left_out.as_slice()));
    let rewritten: bool = file_stamp(copy.path()) != untouched;
    // The refresh replaces the copy when it records anything, so the times go back on after it as well.
    put_the_live_times_back(copy);
    if rewritten {
        // Git replaced the copy without flushing it; the release of the lock flushes what it installs (`service_land_finish.rs`).
        copy.set_flushed(false);
    }
}

/// What:
///  What one file looked like,
/// so a step that replaced it can be told from one that wrote nothing.
/// Why:
///  `git update-index --refresh` writes the index it refreshed only when it has something to record,
/// and only a copy Git replaced has to be flushed by the release
/// (`service_land_finish.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stamp = (path) => { const s = statSync(path); return [s.size, s.mtimeMs]; }
/// ```
fn file_stamp(path: &Path) -> Option<(u64, std::time::SystemTime)> {
    let metadata: std::fs::Metadata = std::fs::metadata(path).ok()?;
    return Some((metadata.len(), metadata.modified().ok()?));
}

/// The names the entries of `written` take in the question about the copy's stat data,
/// read as operating-system text.
///  A name that is not text on this platform has no entry exempted,
/// so the question is asked about it,
/// which is the conservative answer.
#[cfg(unix)]
fn exempt_names(written: &[Vec<u8>]) -> Vec<OsString> {
    use std::os::unix::ffi::OsStrExt;
    let mut names: Vec<OsString> = Vec::with_capacity(written.len());
    for path in written {
        names.push(std::ffi::OsStr::from_bytes(path.as_slice()).to_os_string());
    }
    return names;
}

/// Where a byte path is not operating-system text,
/// nothing is left out of the question,
/// and the whole copy is asked about.
#[cfg(not(unix))]
fn exempt_names(_written: &[Vec<u8>]) -> Vec<OsString> {
    return Vec::new();
}

/// What:
///  The paths a list or a name list resets,
/// with nothing for a list derived from the branch.
/// Why:
///  Git's own `reset` runs when the in-process update declined,
/// and a list or a name list is its own pathspecs;
/// a derived list is Git's to work out,
/// and the copy is then asked about as a whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = paths.listed ?? paths.named ?? []
/// ```
fn named_paths(paths: IndexPaths<'_>) -> Vec<Vec<u8>> {
    let listed: &[String] = match paths {
        IndexPaths::Listed(listed) => listed,
        IndexPaths::Named(named) => return named.to_vec(),
        IndexPaths::Derived { .. } => return Vec::new(),
    };
    let mut names: Vec<Vec<u8>> = Vec::with_capacity(listed.len());
    for name in listed {
        names.push(name.as_bytes().to_vec());
    }
    return names;
}

/// The names the private copy (made from the live index while `lock` is held) was reset at without starting Git,
/// or nothing when Git's own command has to run.
#[cfg(target_os = "linux")]
fn reset_in_process(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, (lock, copy): (&IndexLock, &PrivateCopy)) -> Option<Vec<Vec<u8>>> {
    return crate::index_reset_file::reset_index_file(git, repository, new, paths, lock, copy);
}

/// Without the in-process update on this platform,
/// Git's own command always runs.
#[cfg(not(target_os = "linux"))]
fn reset_in_process(_git: &GitContext, _repository: &Path, _new: &str, _paths: IndexPaths<'_>, _landing: (&IndexLock, &PrivateCopy)) -> Option<Vec<Vec<u8>>> {
    return None;
}

/// The failure a storage error stands for.
pub fn storage_failure(error: &std::io::Error) -> FailureCode {
    return match error.raw_os_error() {
        Some(code) if code == 28 || code == 122 => FailureCode::RepositoryFull,
        Some(code) if code == libc::EROFS => FailureCode::RepositoryReadOnly,
        Some(code) if code == libc::EACCES => FailureCode::PermissionDenied,
        _ => FailureCode::Unavailable,
    };
}

pub(crate) fn cleanup_done(git: &GitContext, repository: &Path, intent: &crate::service_journal::Intent, journal: &Path) -> bool {
    let Some(cleanup) = intent.cleanup.as_ref() else { return true };
    // The client's own scratch directory is the service's to remove when that client is gone, so the copy the landing kept in
    //  the journal is what the comparison reads once it is: a cleanup that failed once would otherwise remove nothing and answer
    //  success, and the next commit would conclude the same merge again.
    let kept: std::path::PathBuf = crate::conclusion_state_copy::path_of(journal, intent.seq);
    let surviving: Option<&Path> = kept.exists().then_some(kept.as_path());
    return remove_conclusion_state(git, repository, cleanup, surviving);
}

/// What:
///  Removes the conclusion state real Git removed in the scratch repository,
/// from the worktree's Git directory.
/// Why:
///   Each entry goes only while it still holds the bytes copied when the commit was prepared,
/// so a later command's merge
///  state is never removed;
/// after a first run nothing matches any more,
/// so a second run changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeConclusionState(git, repository, cleanup)
/// ```
pub fn remove_conclusion_state(git: &GitContext, repository: &Path, cleanup: &Cleanup, surviving: Option<&Path>) -> bool {
    // The scratch directory is the client's, and the service removes it when that client is gone; the copy the landing kept in its
    //  journal is what the comparison reads then, and without either nothing can be compared, so nothing is removed.
    let compared: &Path = if cleanup.scratch.exists() {
        cleanup.scratch.as_path()
    } else {
        match surviving {
            Some(kept) if kept.exists() => kept,
            _ => {
                warn(
                    "serviceLandFollow",
                    format!(
                        "the conclusion state of {} was left as it is: the directory it was compared against, {}, is gone and no copy of it was kept",
                        cleanup.git_dir.display(),
                        cleanup.scratch.display()
                    )
                    .as_str(),
                );
                return false;
            }
        }
    };
    let format: RefFormat = if cleanup.reftable { RefFormat::Reftable } else { RefFormat::Files };
    let shadow: std::path::PathBuf = compared.join("shadow");
    // The stash Git stored in the scratch repository when its apply conflicted is stored in the repository before the cleanup
    //  removes the pseudoref that named it; if that fails the state stays, so the cleanup runs again.
    if !carry_stash(git, repository, cleanup) {
        return false;
    }
    return match reproduce_conclusion_cleanup(git, repository, cleanup.git_dir.as_path(), shadow.as_path(), compared, format) {
        Ok(()) => true,
        Err(error) => {
            warn("serviceLandFollow", format!("removing the conclusion state failed: {}", error.0).as_str());
            false
        }
    };
}

/// Controls for the landing's follow-up stay out of the release executable.
#[cfg(test)]
#[path = "service_land_follow_tests.rs"]
mod tests;

/// Controls for the awaited steps of the follow-up stay out of the release executable.
#[cfg(test)]
#[path = "service_land_follow_awaiting_tests.rs"]
mod awaiting_tests;
