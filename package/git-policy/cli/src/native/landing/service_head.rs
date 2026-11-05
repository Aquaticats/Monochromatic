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

use crate::service_protocol::FailureCode;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::path::Path;

/// What a `HEAD` file says on its own.
#[derive(Debug, Eq, PartialEq)]
enum HeadFile {
    /// The head names this reference.
    Branch(String),
    /// The head holds this object name: no branch is checked out.
    Detached(String),
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
        None => HeadFile::Detached(line.to_string()),
    };
}

/// Whether `directory` is the Git directory of a worktree of the repository whose common Git directory is `common_dir`: that
///  directory itself, or a linked worktree's, whose `commondir` file points back to it.
fn belongs_to(directory: &Path, common_dir: &Path) -> bool {
    let Ok(expected) = std::fs::canonicalize(common_dir) else { return false };
    if matches!(std::fs::canonicalize(directory), Ok(found) if found == expected) {
        return true;
    }
    let Ok(pointer) = std::fs::read_to_string(directory.join("commondir")) else { return false };
    return matches!(std::fs::canonicalize(directory.join(pointer.trim_end())), Ok(found) if found == expected);
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
    return match std::fs::read_to_string(git_dir.join("HEAD")) {
        Ok(text) => interpret(text.as_str()),
        Err(_unreadable) => HeadFile::Unreadable,
    };
}

/// What:
///  The reference `HEAD` names when Git is asked in `repository`, and nothing when `HEAD` is detached.
/// Why:
///   `git symbolic-ref --quiet HEAD` ends with status 1 for a detached head and with other statuses (128 for a repository it
///  cannot read, a failed read of the files) for a question it could not answer. Only status 1 is a detached head: any other
///  failure is no answer, and a landing that treated it as one would move a branch for a worktree whose head it never saw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named: string | undefined = await symbolicHead(git, repository) // throws when Git cannot say
/// ```
///
/// # Errors
///
/// `Unavailable` when Git cannot be started or ends with a status other than 0 and 1.
fn symbolic_head(git: &GitContext, repository: &Path) -> Result<Option<String>, FailureCode> {
    let output = match run_git(git, &GitRequest::new(repository, &["symbolic-ref", "--quiet", "HEAD"])) {
        Ok(finished) => finished,
        Err(_unstartable) => return Err(FailureCode::Unavailable),
    };
    return match output.code {
        Some(0) => Ok(Some(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string())),
        Some(1) => Ok(None),
        _ => Err(FailureCode::Unavailable),
    };
}

/// What:
///  Whether the worktree's head, after a commit moved `reference` to `new`, still shows that commit: the branch for a named
///  branch, and for a commit on `HEAD` either a detached head at `new` or the branch the head names now, when it is at `new`.
/// Why:
///   The update of a detached head follows a head that was attached to a branch since the last look, as native Git's commit does,
///  so the commit may have gone to the branch the head names. The index is updated for a head that shows the commit, whichever
///  way it shows it, and left alone for a head that shows another commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shows = await followsMove(git, { repository, gitDir, commonDir }, reference, newCommit)
/// ```
///
/// # Errors
///
/// The failure code when Git must be asked and cannot answer.
pub fn follows_move(git: &GitContext, repository: &Path, git_dir: &Path, common_dir: &Path, reference: &str, new: &str) -> Result<bool, FailureCode> {
    if reference != "HEAD" {
        return head_names(git, repository, git_dir, common_dir, reference);
    }
    if let HeadFile::Detached(value) = head_file(git_dir, common_dir) {
        return Ok(value == new);
    }
    let output = match run_git(git, &GitRequest::new(repository, &["rev-parse", "--verify", "--quiet", "HEAD"])) {
        Ok(finished) => finished,
        Err(_unstartable) => return Err(FailureCode::Unavailable),
    };
    return match output.code {
        Some(0) => Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim() == new),
        Some(1) => Ok(false),
        _ => Err(FailureCode::Unavailable),
    };
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
        HeadFile::Detached(_) => return Ok(false),
        HeadFile::Unreadable => {}
    }
    return match symbolic_head(git, repository)? {
        Some(named) => Ok(named == reference),
        None => Ok(false),
    };
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
        HeadFile::Detached(_) => return Ok(true),
        HeadFile::Unreadable => {}
    }
    return Ok(symbolic_head(git, repository)?.is_none());
}

/// Head controls stay out of the release executable.
#[cfg(test)]
#[path = "service_head_tests.rs"]
mod tests;
