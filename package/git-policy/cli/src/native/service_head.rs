//! What:
//!  Whether a worktree's head still names the branch a commit was prepared on.
//! Why:
//!  Native `git commit` updates `HEAD`, so a worktree that switched branches while the commit waited (in the editor, for a
//!  signature) fails with "cannot lock ref 'HEAD'" and changes nothing. The service moves the named branch itself, so it
//!  asks the same question before anything moves: a branch moved under a worktree that is on another branch would leave that
//!  worktree's index describing the wrong commit. The answer is read from the worktree's `HEAD` file when that file can be
//!  trusted and from Git otherwise, so an ordinary landing starts no extra Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const names = await headNames({ repository, index, commonDir, reference }) // false after a switch or a detach
//! ```

use super::service_protocol::FailureCode;
use super::transaction_git::{GitContext, GitRequest, run_git};
use std::path::Path;

/// What a `HEAD` file says on its own.
#[derive(Debug, Eq, PartialEq)]
enum HeadFile {
    /// The head names this reference.
    Branch(String),
    /// The head holds an object name: no branch is checked out.
    Detached,
    /// The file cannot answer: a repository that keeps references in tables stores a placeholder here.
    Unreadable,
}

/// The placeholder reftable repositories keep in `HEAD`, so that tools which read the file do not mistake the repository for
///  an empty one.
const REFTABLE_PLACEHOLDER: &str = "refs/heads/.invalid";

fn interpret(text: &str) -> HeadFile {
    let line: &str = text.lines().next().unwrap_or("").trim_end();
    return match line.strip_prefix("ref: ") {
        Some(reference) if reference == REFTABLE_PLACEHOLDER => HeadFile::Unreadable,
        Some(reference) => HeadFile::Branch(reference.to_string()),
        None if line.is_empty() => HeadFile::Unreadable,
        None => HeadFile::Detached,
    };
}

/// Whether `directory` is the Git directory of a worktree of the repository whose common Git directory is `common_dir`: that
///  directory itself, or a linked worktree's, whose `commondir` file points back to it.
fn belongs_to(directory: &Path, common_dir: &Path) -> bool {
    let Ok(expected) = std::fs::canonicalize(common_dir) else { return false };
    if std::fs::canonicalize(directory).is_ok_and(|found| { return found == expected; }) {
        return true;
    }
    let Ok(pointer) = std::fs::read_to_string(directory.join("commondir")) else { return false };
    return std::fs::canonicalize(directory.join(pointer.trim_end())).is_ok_and(|found| { return found == expected; });
}

/// What:
///  What the `HEAD` file of the worktree's own Git directory says, when that directory is a worktree of the repository.
/// Why:
///   The directory is the one the request recorded, not the one the index happens to be in: an alternate index can sit in
///  another worktree's Git directory, whose head is another worktree's.
fn head_file(git_dir: &Path, common_dir: &Path) -> HeadFile {
    if !belongs_to(git_dir, common_dir) {
        return HeadFile::Unreadable;
    }
    return std::fs::read_to_string(git_dir.join("HEAD")).ok().map_or(HeadFile::Unreadable, |text| { return interpret(text.as_str()); });
}

/// What:
///  Whether the head of the worktree whose Git directory is `git_dir` names `reference`.
/// Why:
///   The file is read when it is that worktree's own, and Git is asked (in the worktree) when it cannot answer (a repository
///  with reference tables).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await headNames(git, { repository, gitDir, commonDir }, reference)
/// ```
pub fn head_names(git: &GitContext, repository: &Path, git_dir: &Path, common_dir: &Path, reference: &str) -> Result<bool, FailureCode> {
    match head_file(git_dir, common_dir) {
        HeadFile::Branch(named) => return Ok(named == reference),
        HeadFile::Detached => return Ok(false),
        HeadFile::Unreadable => {}
    }
    let output = run_git(git, &GitRequest::new(repository, &["symbolic-ref", "--quiet", "HEAD"])).map_err(|_| { return FailureCode::Unavailable; })?;
    return Ok(output.succeeded() && String::from_utf8_lossy(output.stdout.as_slice()).trim() == reference);
}

/// What:
///  Whether the worktree still has the reference a commit was prepared on checked out in the way the commit needs: the branch
///  `reference` for a named branch, and no branch at all (a detached head) for `HEAD`.
/// Why:
///   The landing asks it before it waits for the index lock and again once it holds the lock, because the worktree can switch
///  while it waits; the ref transaction that moves the branch verifies the same thing at the moment of the move. The detached case
///  reads the same `HEAD` file as the branch case, so neither starts a Git ordinarily.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = await stillCheckedOut(git, { repository, gitDir, commonDir }, reference)
/// ```
///
/// # Errors
///
/// The failure code when Git must be asked and cannot be started.
pub fn still_checked_out(git: &GitContext, repository: &Path, git_dir: &Path, common_dir: &Path, reference: &str) -> Result<bool, FailureCode> {
    if reference != "HEAD" {
        return head_names(git, repository, git_dir, common_dir, reference);
    }
    match head_file(git_dir, common_dir) {
        HeadFile::Branch(_) => return Ok(false),
        HeadFile::Detached => return Ok(true),
        HeadFile::Unreadable => {}
    }
    let output = run_git(git, &GitRequest::new(repository, &["symbolic-ref", "--quiet", "HEAD"])).map_err(|_| { return FailureCode::Unavailable; })?;
    return Ok(!output.succeeded());
}

/// Head controls stay out of the release executable.
#[cfg(test)]
#[path = "service_head_tests.rs"]
mod tests;
