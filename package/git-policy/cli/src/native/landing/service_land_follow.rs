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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await updateIndex(git, repository, newCommit, paths); await removeConclusionState(git, repository, cleanup);
//! ```

use crate::autostash_carry::carry_stash;
use crate::conclusion_cleanup::reproduce_conclusion_cleanup;
use crate::diagnostic_log::warn;
use crate::index_holder_wait::{IndexWait, wait_for_index_release_within};
use crate::wait_ceilings::RECONCILE_LOCK_CEILING;
use crate::wait_progress::Cadence;
use crate::inner_git_words::record_words;
use crate::service_index_lock::IndexLock;
use crate::commit_index_stat_refresh::{StatRefresh, refresh_stat_data_when_stale};
use crate::commit_index_stat_staleness::WorktreeStaging;
pub use crate::service_land_copy::PrivateCopy;
use crate::service_protocol::{Cleanup, FailureCode, is_zero_object};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
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

/// Which index entries a landing updates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexPaths<'a> {
    /// The paths the request listed.
    Listed(&'a [String]),
    /// Paths given as bytes,
    /// for a client that reconciles the index without a landing.
    Named(&'a [Vec<u8>]),
    /// Every path that differs between the branch before the landing and the new commit.
    Derived {
        /// The branch before the landing;
        /// the all-zero name when the branch did not exist.
        old: &'a str,
    },
}

fn run(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>, index: Option<&Path>) -> Result<GitOutput, FailureCode> {
    // Every pathspec this module writes is its own, so no pathspec mode of the caller's reaches it
    // (`GitRequest::own_pathspecs`), whether the caller set it as a variable or as a global option.
    let mut request: GitRequest = GitRequest::new(repository, arguments).own_pathspecs();
    request.input = input;
    request.index = index.map(Path::to_path_buf);
    return match run_git(git, &request) {
        Ok(output) => Ok(output),
        Err(_unstartable) => Err(FailureCode::Unavailable),
    };
}

/// The failure a Git step that did not succeed stands for.
pub fn classify(output: &GitOutput) -> FailureCode {
    let text: String = output.error_text().to_lowercase();
    if text.contains("no space left") || text.contains("quota exceeded") {
        return FailureCode::RepositoryFull;
    }
    if text.contains("read-only file system") {
        return FailureCode::RepositoryReadOnly;
    }
    if text.contains("permission denied") {
        return FailureCode::PermissionDenied;
    }
    if text.contains("index.lock") {
        return FailureCode::IndexInUse;
    }
    return FailureCode::Unavailable;
}

/// A path in the form `reset` reads literally from the top of the repository.
fn literal(path: &[u8]) -> Vec<u8> {
    let mut spec: Vec<u8> = b":(top,literal)".to_vec();
    spec.extend_from_slice(path);
    return spec;
}

/// The names (bytes,
/// NUL-separated) of the paths that differ between `old` and `new`.
pub(crate) fn derived_names(git: &GitContext, repository: &Path, old: &str, new: &str) -> Result<Vec<u8>, FailureCode> {
    let mut arguments: Vec<&str> = vec!["diff-tree", "-r", "--name-only", "-z", "--no-renames"];
    if is_zero_object(old) {
        // A branch that did not exist: the commit is a root commit, and `--root` lists everything it holds.
        arguments.push("--root");
        arguments.push(new);
    } else {
        arguments.push(old);
        arguments.push(new);
    }
    let output: GitOutput = run(git, repository, &arguments, None, None)?;
    if !output.succeeded() {
        return Err(classify(&output));
    }
    return Ok(output.stdout);
}

/// What:
///  Whether a byte is the NUL that separates the names of a pathspec list.
/// Why:
///  `pathspec_input` splits a NUL-separated list into names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isZeroByte = (byte: number) => byte === 0;
/// ```
fn is_zero_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether a name from a NUL-separated list is not empty.
/// Why:
///  Two NULs in a row leave an empty name that names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNotEmptyName = (name: Uint8Array) => name.length > 0;
/// ```
fn is_not_empty_name(name: &&[u8]) -> bool {
    return !name.is_empty();
}

/// What:
///  A path as a pathspec that matches that path from the top of the worktree and nothing else.
/// Why:
///  `reset` is given one such spec per path the batch changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const topLiteralSpec = (path: string) => `:(top,literal)${path}`;
/// ```
fn top_literal_spec(path: &String) -> String {
    return format!(":(top,literal){path}");
}

/// The standard input of `reset --pathspec-from-file=- --pathspec-file-nul` for a list of NUL-separated names.
fn pathspec_input(names: &[u8]) -> Vec<u8> {
    let mut input: Vec<u8> = Vec::new();
    for name in names.split(is_zero_byte).filter(is_not_empty_name) {
        input.extend(literal(name));
        input.push(0);
    }
    return input;
}

/// One try of the index update.
fn reset_once(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, index: Option<&Path>) -> Result<(), FailureCode> {
    let output: GitOutput = match paths {
        IndexPaths::Listed(listed) => {
            let specs: Vec<String> = listed.iter().map(top_literal_spec).collect();
            let mut arguments: Vec<&str> = vec!["reset", "--no-refresh", new, "--"];
            arguments.extend(specs.iter().map(String::as_str));
            run(git, repository, &arguments, None, index)?
        }
        IndexPaths::Named(names) => {
            // Each name followed by a NUL, as `--pathspec-file-nul` reads them.
            let mut joined: Vec<u8> = Vec::new();
            for name in names.iter() {
                joined.extend(name.iter().copied());
                joined.push(0);
            }
            let input: Vec<u8> = pathspec_input(joined.as_slice());
            if input.is_empty() {
                return Ok(());
            }
            run(git, repository, &["reset", "--no-refresh", new, "--pathspec-from-file=-", "--pathspec-file-nul"], Some(input), index)?
        }
        IndexPaths::Derived { old } => {
            let input: Vec<u8> = pathspec_input(derived_names(git, repository, old, new)?.as_slice());
            if input.is_empty() {
                return Ok(());
            }
            run(git, repository, &["reset", "--no-refresh", new, "--pathspec-from-file=-", "--pathspec-file-nul"], Some(input), index)?
        }
    };
    if !output.succeeded() {
        return Err(classify(&output));
    }
    // The `reset` carries no `-q`: its list of paths left changed in the worktree ("Unstaged changes after reset:") is kept
    //  in the service's log.
    record_words(&["reset", "<commit>", "--", "<paths>"], &output);
    return Ok(());
}

/// What:
///  Resets the index entries of `names` in `index` to the commit `new`,
///  without waiting and without a refresh.
/// Why:
///   The landing's own index update and the repair of an update it left owed are one step
///   (`owed_index_update.rs`):
///  a path whose entry the landing left is reset the same way by both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resetEntries(git, repository, index, newCommit, names);
/// ```
///
/// # Errors
///
/// The failure code of the `reset` that was refused.
pub fn reset_entries(git: &GitContext, repository: &Path, index: Option<&Path>, new: &str, names: &[Vec<u8>]) -> Result<(), FailureCode> {
    if names.is_empty() {
        return Ok(());
    }
    return reset_once(git, repository, new, IndexPaths::Named(names), index);
}

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
                // The diagnostic of a wait that ended on a dead owner is already written.
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
/// so a batch pays one refresh, in the step that already holds the lock,
/// instead of one per client that then waits for the lock the landing needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const built = await buildIndex(git, repository, newCommit, paths, indexPath, seq)
/// ```
pub fn build_index(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, lock: &IndexLock, seq: u64) -> Result<PrivateCopy, FailureCode> {
    let copy: PrivateCopy = PrivateCopy::make(lock, seq)?;
    let built: std::path::PathBuf = copy.path().to_path_buf();
    let empty: bool = matches!(paths, IndexPaths::Listed(listed) if listed.is_empty()) || matches!(paths, IndexPaths::Named(named) if named.is_empty());
    if !empty && !reset_in_process(git, repository, new, paths, (lock, &copy)) && let Err(code) = reset_once(git, repository, new, paths, Some(built.as_path())) {
        return Err(code);
    }
    keep_the_stat_data(git, repository, paths, &copy);
    return Ok(copy);
}

/// What:
///  Records the worktree's stat data in the index the landing is about to install,
/// and answers what became of the refresh.
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await keepTheStatData({ git, repository, paths, copy }) // refresh_if_stale(copy, staging = the reset paths)
/// ```
fn keep_the_stat_data(git: &GitContext, repository: &Path, paths: IndexPaths<'_>, copy: &PrivateCopy) {
    let written: Vec<OsString> = written_names(paths);
    let outcome: StatRefresh = refresh_stat_data_when_stale(git, repository, copy.path(), WorktreeStaging::Named(written.as_slice()));
    if outcome == StatRefresh::NotKept {
        // Nothing else follows from a refresh Git did not write: the index still holds the stat data the copy carried, and
        // the commit lands on it as it is.
        return;
    }
    // Git wrote the copy without flushing it; the release of the lock flushes what it installs (`service_land_finish.rs`).
    copy.set_flushed(false);
}

/// What:
///  The paths whose entries the reset wrote,
/// as the question about the copy's stat data takes them.
/// Why:
///  Those entries hold the commit's own content,
/// and their stat data is the landing's to leave unrecorded:
/// a path the person edited after staging holds bytes the commit does not,
/// and Git's own refresh records nothing for it.
///  A list derived from the branch names no path here,
/// and the whole copy is asked about instead,
/// which costs time and never a wrong answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = paths.named.map(osString) // [] for a derived list
/// ```
fn written_names(paths: IndexPaths<'_>) -> Vec<OsString> {
    return match paths {
        IndexPaths::Listed(listed) => listed.iter().map(OsString::from).collect(),
        IndexPaths::Named(named) => named_of_bytes(named),
        IndexPaths::Derived { .. } => Vec::new(),
    };
}

/// The names a request wrote, read as operating-system text.
/// A path that is not text has no entry this landing exempts,
/// so the question about the copy is asked about it,
/// which is the conservative answer.
#[cfg(unix)]
fn named_of_bytes(named: &[Vec<u8>]) -> Vec<OsString> {
    use std::os::unix::ffi::OsStrExt;
    return named.iter().map(|path| { return std::ffi::OsStr::from_bytes(path.as_slice()).to_os_string(); }).collect();
}

/// Without a byte path on this platform,
/// no request path is exempt,
/// and the whole copy is asked about.
#[cfg(not(unix))]
fn named_of_bytes(_named: &[Vec<u8>]) -> Vec<OsString> {
    return Vec::new();
}

/// Whether the private copy (made from the live index while `lock` is held) was reset to
///  the commit without starting Git;
/// Git's own command follows when it was not.
#[cfg(target_os = "linux")]
fn reset_in_process(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, (lock, copy): (&IndexLock, &PrivateCopy)) -> bool {
    return crate::index_reset_file::reset_index_file(git, repository, new, paths, lock, copy);
}

/// Without the in-process update on this platform,
/// Git's own command always runs.
#[cfg(not(target_os = "linux"))]
fn reset_in_process(_git: &GitContext, _repository: &Path, _new: &str, _paths: IndexPaths<'_>, _landing: (&IndexLock, &PrivateCopy)) -> bool {
    return false;
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
