//! What:
//!  What the service's landing does once the branch has moved: the worktree's index entries of the commit's paths and the
//!  conclusion state a merge, cherry-pick or revert leaves behind.
//! Why:
//!  A commit Git built in a scratch repository carries no list of changes, and a conclusion's state files belong to the
//!  commit that concludes it. Both steps are idempotent, so the same code finishes a batch a killed service left open.
//!  The index entries come from the repository itself: the paths that differ between the branch before and the commit,
//!  as bytes, handed to `reset` on its standard input, so a name that is not UTF-8, a long list and a gitlink need no
//!  special case (`doc/planning/cli-git-resident-service.md`, "Landing every command line").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await updateIndex(git, repository, newCommit, paths); await removeConclusionState(git, repository, cleanup);
//! ```

use super::conclusion_cleanup::reproduce_conclusion_cleanup;
use super::diagnostic_log::warn;
use super::service_index_lock::IndexLock;
use super::service_protocol::{Cleanup, FailureCode, is_zero_object};
use super::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use super::transaction_journal::RefFormat;
use std::path::Path;

/// How many times the index update is tried while another program holds the index.
const INDEX_ATTEMPTS: usize = 5;

/// The pause between index tries.
const INDEX_PAUSE: std::time::Duration = std::time::Duration::from_millis(100);

/// Which index entries a landing updates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexPaths<'a> {
    /// The paths the request listed.
    Listed(&'a [String]),
    /// Paths given as bytes, for a client that reconciles the index without a landing.
    Named(&'a [Vec<u8>]),
    /// Every path that differs between the branch before the landing and the new commit.
    Derived {
        /// The branch before the landing; the all-zero name when the branch did not exist.
        old: &'a str,
    },
}

fn run(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>, index: Option<&Path>) -> Result<GitOutput, FailureCode> {
    let mut request: GitRequest = GitRequest::new(repository, arguments);
    request.input = input;
    request.index = index.map(Path::to_path_buf);
    return run_git(git, &request).map_err(|_| { return FailureCode::Unavailable; });
}

/// The failure a Git step that did not succeed stands for.
pub fn classify(output: &GitOutput) -> FailureCode {
    let text: String = output.error_text().to_lowercase();
    if text.contains("no space left") || text.contains("quota exceeded") {
        return FailureCode::RepositoryFull;
    }
    if text.contains("read-only file system") || text.contains("permission denied") {
        return FailureCode::RepositoryReadOnly;
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

/// The names (bytes, NUL-separated) of the paths that differ between `old` and `new`.
fn derived_names(git: &GitContext, repository: &Path, old: &str, new: &str) -> Result<Vec<u8>, FailureCode> {
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

/// The standard input of `reset --pathspec-from-file=- --pathspec-file-nul` for a list of NUL-separated names.
fn pathspec_input(names: &[u8]) -> Vec<u8> {
    let mut input: Vec<u8> = Vec::new();
    for name in names.split(|byte| { return *byte == 0; }).filter(|name| { return !name.is_empty(); }) {
        input.extend(literal(name));
        input.push(0);
    }
    return input;
}

/// One try of the index update.
fn reset_once(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, index: Option<&Path>) -> Result<(), FailureCode> {
    let output: GitOutput = match paths {
        IndexPaths::Listed(listed) => {
            let specs: Vec<String> = listed.iter().map(|path| { return format!(":(top,literal){path}"); }).collect();
            let mut arguments: Vec<&str> = vec!["reset", "--no-refresh", new, "--"];
            arguments.extend(specs.iter().map(String::as_str));
            run(git, repository, &arguments, None, index)?
        }
        IndexPaths::Named(names) => {
            let joined: Vec<u8> = names.iter().flat_map(|name| { return name.iter().copied().chain(std::iter::once(0)); }).collect();
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
    return Ok(());
}

/// What:
///  Updates the index entries of the commit's paths to the commit `new`, trying again while another program holds the
///  index.
/// Why:
///   Only Git's own index lock can make this fail for a moment, and the bound is the smallest that rides out a concurrent
///  `git add`; an empty list changes nothing, because `reset` with no path would reset the whole index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await updateIndex(git, repository, newCommit, paths)
/// ```
pub fn update_index(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>) -> Result<(), FailureCode> {
    if matches!(paths, IndexPaths::Listed(listed) if listed.is_empty()) || matches!(paths, IndexPaths::Named(named) if named.is_empty()) {
        return Ok(());
    }
    let mut last: FailureCode = FailureCode::Unavailable;
    for attempt in 0..INDEX_ATTEMPTS {
        match reset_once(git, repository, new, paths, None) {
            Ok(()) => return Ok(()),
            Err(code) => last = code,
        }
        if attempt + 1 < INDEX_ATTEMPTS {
            std::thread::sleep(INDEX_PAUSE);
        }
    }
    return Err(last);
}

/// What:
///  Builds the index the landing will install, in a file of its own beside the real index: a copy of the real index with the
///  commit's paths reset to the commit.
/// Why:
///   The landing holds Git's `index.lock`, so Git's own `reset` cannot take it; the new index is made in another file and then
///  moved into place by the lock's release, which is how Git installs an index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const built = await buildIndex(git, repository, newCommit, paths, indexPath, seq)
/// ```
pub fn build_index(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, lock: &IndexLock, seq: u64) -> Result<std::path::PathBuf, FailureCode> {
    let index: &Path = lock.index();
    let built: std::path::PathBuf = private_index_path(index, seq);
    let _stale = std::fs::remove_file(&built);
    if index.exists() {
        std::fs::copy(index, &built).map_err(|error| { return storage_failure(&error); })?;
    }
    let empty: bool = matches!(paths, IndexPaths::Listed(listed) if listed.is_empty()) || matches!(paths, IndexPaths::Named(named) if named.is_empty());
    if !empty && !reset_in_process(git, repository, new, paths, (lock, seq)) && let Err(code) = reset_once(git, repository, new, paths, Some(built.as_path())) {
        let _removed = std::fs::remove_file(&built);
        return Err(code);
    }
    return Ok(built);
}

/// Whether the private index of batch `seq` (the copy of the live index made while `lock` is held) was reset to
///  the commit without starting Git; Git's own command follows when it was not.
#[cfg(target_os = "linux")]
fn reset_in_process(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, (lock, seq): (&IndexLock, u64)) -> bool {
    return super::index_reset_file::reset_index_file(git, repository, new, paths, lock, seq);
}

/// Without the in-process update on this platform, Git's own command always runs.
#[cfg(not(target_os = "linux"))]
fn reset_in_process(_git: &GitContext, _repository: &Path, _new: &str, _paths: IndexPaths<'_>, _landing: (&IndexLock, u64)) -> bool {
    return false;
}

/// What:
///  The only name a landing gives its private copy of the live index `live` for batch `seq`.
/// Why:
///   `build_index` makes the copy under this name and the in-process update works on this name alone, so no other file beside
///  the live index (another client's index chosen by `GIT_INDEX_FILE`, another batch's copy) can be replaced by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const built = join(dirname(live), `index.cli-git-${seq}`);
/// ```
pub fn private_index_path(live: &Path, seq: u64) -> std::path::PathBuf {
    return live.with_file_name(format!("index.cli-git-{seq}"));
}

/// The failure a storage error stands for.
pub fn storage_failure(error: &std::io::Error) -> FailureCode {
    return match error.raw_os_error() {
        Some(code) if code == 28 || code == 122 => FailureCode::RepositoryFull,
        Some(code) if code == 30 || code == 13 => FailureCode::RepositoryReadOnly,
        _ => FailureCode::Unavailable,
    };
}

/// What:
///  Removes the conclusion state real Git removed in the scratch repository, from the worktree's Git directory.
/// Why:
///   Each entry goes only while it still holds the bytes copied when the commit was prepared, so a later command's merge
///  state is never removed; after a first run nothing matches any more, so a second run changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeConclusionState(git, repository, cleanup)
/// ```
pub fn remove_conclusion_state(git: &GitContext, repository: &Path, cleanup: &Cleanup) -> bool {
    // The scratch directory is the client's; once the client is gone and its directory with it there is nothing left to compare.
    if !cleanup.scratch.exists() {
        return true;
    }
    let format: RefFormat = if cleanup.reftable { RefFormat::Reftable } else { RefFormat::Files };
    let shadow = cleanup.scratch.join("shadow");
    return match reproduce_conclusion_cleanup(git, repository, cleanup.git_dir.as_path(), shadow.as_path(), cleanup.scratch.as_path(), format) {
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
