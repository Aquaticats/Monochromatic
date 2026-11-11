//! What:
//!  Whether a worktree's head still names the branch a commit was prepared on.
//! Why:
//!  Native `git commit` updates `HEAD`,
//! so a worktree that switched branches while the commit waited (in the editor,
//! for a
//!  signature) fails with "cannot lock ref 'HEAD'" and changes nothing.
//! The service moves the named branch itself,
//! so it
//!  asks the same question before anything moves:
//! a branch moved under a worktree that is on another branch would leave that
//!  worktree's index describing the wrong commit.
//! The answer is read from the worktree's `HEAD` file when that file can be
//!  trusted and from Git otherwise,
//! so an ordinary landing starts no extra Git.
//! Each question has two faces:
//! the synchronous one a caller on a thread asks,
//! and
//!  the awaited one a task of the runtime asks
//! (`follows_move_awaiting`,
//! `head_names_awaiting`,
//! `still_checked_out_awaiting`).
//! The awaited face reads the files
//!  as a job of the blocking pool,
//! awaited no longer than `LANDING_STATE_CEILING`,
//! and awaits its Git child on the
//!  runtime,
//! so neither a mount that stopped answering nor a Git that never ends holds a worker.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const names = await headNames({ repository, index, commonDir, reference }) // false after a switch or a detach
//! ```

use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than};
use crate::service_protocol::FailureCode;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::wait_ceilings::LANDING_STATE_CEILING;
/// A reference name as the bytes Git printed it.
use crate::ref_name::RefName;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// What a `HEAD` file says on its own.
#[derive(Debug, Eq, PartialEq)]
enum HeadFile {
    /// The head names this reference,
    /// by its own bytes.
    Branch(RefName),
    /// The head holds this object name:
    /// no branch is checked out.
    Detached(String),
    /// The file cannot answer:
    /// a repository that keeps references in tables stores a placeholder here.
    Unreadable,
}

/// The placeholder reftable repositories keep in `HEAD`,
/// so that tools which read the file do not mistake the repository for
///  an empty one.
const REFTABLE_PLACEHOLDER: &str = "refs/heads/.invalid";

fn interpret(text: &[u8]) -> HeadFile {
    // The first line only, without the whitespace its end carries; a name that is not text is still a name.
    let first: &[u8] = text.split(is_a_line_feed).next().unwrap_or(b"");
    let line: &[u8] = trim_ascii_end(first);
    if let Some(reference) = line.strip_prefix(b"ref: ") {
        if reference == REFTABLE_PLACEHOLDER.as_bytes() {
            return HeadFile::Unreadable;
        }
        return HeadFile::Branch(RefName::from_git_output(reference));
    }
    if line.is_empty() {
        return HeadFile::Unreadable;
    }
    return HeadFile::Detached(String::from_utf8_lossy(line).trim_end().to_string());
}

/// What:
///  Whether one byte ends a line.
/// Why:
///   A `HEAD` file is one line,
/// and the name is the bytes before the line feed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isALineFeed = (byte: number) => byte === 10
/// ```
fn is_a_line_feed(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  The bytes without the whitespace at their end,
///  as `trim_end` treats text.
/// Why:
///  A `HEAD` file ends with a line feed,
/// and a landing compares the name it holds with the name it recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trimmed = line.replace(/\s+$/u, '')
/// ```
fn trim_ascii_end(bytes: &[u8]) -> &[u8] {
    let mut end: usize = bytes.len();
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    return &bytes[..end];
}

/// Whether `directory` is the Git directory of a worktree of the repository whose common Git directory is `common_dir`:
/// that
///  directory itself,
/// or a linked worktree's,
/// whose `commondir` file points back to it.
fn belongs_to(directory: &Path, common_dir: &Path) -> bool {
    let Ok(expected) = std::fs::canonicalize(common_dir) else { return false };
    if matches!(std::fs::canonicalize(directory), Ok(found) if found == expected) {
        return true;
    }
    let Ok(pointer) = std::fs::read_to_string(directory.join("commondir")) else { return false };
    return matches!(std::fs::canonicalize(directory.join(pointer.trim_end())), Ok(found) if found == expected);
}

/// What:
///  What the `HEAD` file of the worktree's own Git directory says,
/// when that directory is a worktree of the repository.
/// Why:
///   The directory is the one the request recorded,
/// not the one the index happens to be in:
/// an alternate index can sit in
///  another worktree's Git directory,
/// whose head is another worktree's.
fn head_file(git_dir: &Path, common_dir: &Path) -> HeadFile {
    if !belongs_to(git_dir, common_dir) {
        return HeadFile::Unreadable;
    }
    return match std::fs::read(git_dir.join("HEAD")) {
        Ok(bytes) => interpret(bytes.as_slice()),
        Err(_unreadable) => HeadFile::Unreadable,
    };
}

/// The two directories one head-file question names,
/// owned so that a job of the blocking pool can take them.
struct HeadFileQuestion {
    /// The Git directory of the worktree that asked.
    git_dir: PathBuf,
    /// The common Git directory it must belong to.
    common_dir: PathBuf,
}

/// The reading one job of the pool does:
/// `head_file` over the question's own two directories.
fn read_the_head_file(question: HeadFileQuestion) -> HeadFile {
    return head_file(question.git_dir.as_path(), question.common_dir.as_path());
}

/// What:
///  `head_file` as a job of the blocking pool,
/// awaited no longer than `ceiling`.
/// Why:
///   A task of the runtime may not wait on a filesystem:
/// `canonicalize` of the two directories and
///  the reads of `commondir` and `HEAD` wait in the kernel on a mount that stopped answering,
/// which would hold a
///  worker the runtime needs,
/// and no timer on that worker can end it.
/// A question the ceiling or a refused thread leaves without an
///  answer is answered `Unreadable`,
/// which is what a file that cannot be read answers already,
/// so the caller asks Git and
///  the person reads Git's own answer:
/// nothing waits on the read for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function headFileWithin(ceilingMs: number, gitDir: string, commonDir: string): Promise<HeadFile>;
/// ```
async fn head_file_within(ceiling: Duration, git_dir: &Path, common_dir: &Path) -> HeadFile {
    let question: HeadFileQuestion = HeadFileQuestion { git_dir: git_dir.to_path_buf(), common_dir: common_dir.to_path_buf() };
    return match run_a_job_awaited_no_longer_than(ceiling, question, read_the_head_file).await {
        TheAnswerOfAJobAwaited::Ran(answer) => answer,
        // No thread was given, or the job outlasted its ceiling: the file said nothing this task may wait for.
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => HeadFile::Unreadable,
    };
}

/// `head_file_within` with the wait a landing's own state step is granted
/// (`wait_ceilings.rs`,
/// `LANDING_STATE_CEILING`).
async fn head_file_awaiting(git_dir: &Path, common_dir: &Path) -> HeadFile {
    return head_file_within(LANDING_STATE_CEILING, git_dir, common_dir).await;
}

/// What:
///  The reference `HEAD` names when Git is asked in `repository`,
/// and nothing when `HEAD` is detached.
/// Why:
///   `git symbolic-ref --quiet HEAD` ends with status 1 for a detached head and with other statuses (128 for a repository it
///  cannot read,
/// a failed read of the files) for a question it could not answer.
/// Only status 1 is a detached head:
/// any other
///  failure is no answer,
/// and a landing that treated it as one would move a branch for a worktree whose head it never saw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named: string | undefined = await symbolicHead(git, repository) // throws when Git cannot say
/// ```
///
/// # Errors
///
/// `Unavailable` when Git cannot be started or ends with a status other than 0 and 1.
fn symbolic_head(git: &GitContext, repository: &Path) -> Result<Option<RefName>, FailureCode> {
    return named_of(run_git(git, &symbolic_head_request(repository)));
}

/// What:
///  The question `symbolic_head` asks Git.
/// Why:
///   Both faces ask the one question,
/// so neither can ask a different one,
/// and the in-process reader answers both the
///  same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['symbolic-ref', '--quiet', 'HEAD'] };
/// ```
fn symbolic_head_request(repository: &Path) -> GitRequest {
    return GitRequest::new(repository, &["symbolic-ref", "--quiet", "HEAD"]);
}

/// What:
///  The reference Git's answer to `symbolic_head_request` names,
/// or nothing for a detached head.
/// Why:
///  Both faces turn Git's answer into the answer through this one function,
/// so only status 1 is read as a detached
///  head and any other failure is no answer,
/// whichever face asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namedOf(answered: GitOutput | Error): string | undefined; // throws when Git could not answer
/// ```
///
/// # Errors
///
/// `Unavailable` when Git could not be started or ended with a status other than 0 and 1.
fn named_of(answered: std::io::Result<GitOutput>) -> Result<Option<RefName>, FailureCode> {
    let output: GitOutput = match answered {
        Ok(finished) => finished,
        Err(_unstartable) => return Err(FailureCode::Unavailable),
    };
    return match output.code {
        // The name is the bytes Git printed: a branch name is a byte string, and a reading that replaced a byte would name
        //  another ref.
        Some(0) => Ok(Some(RefName::from_git_output(output.stdout.as_slice()))),
        Some(1) => Ok(None),
        _ => Err(FailureCode::Unavailable),
    };
}

/// What:
///  `symbolic_head` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// and the
///  question is asked from the landing once that landing is a task.
/// The child's own bound and quiet window are the synchronous
///  face's,
/// so the two cannot differ in what they wait for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function symbolicHeadAwaiting(git: GitContext, repository: string): Promise<string | undefined>;
/// ```
async fn symbolic_head_awaiting(git: &GitContext, repository: &Path) -> Result<Option<RefName>, FailureCode> {
    return named_of(run_git_awaiting(git, &symbolic_head_request(repository)).await);
}

/// What:
///  Whether the worktree's head,
/// after a commit moved `reference` to `new`,
/// still shows that commit:
/// the branch for a named
///  branch,
/// and for a commit on `HEAD` either a detached head at `new` or the branch the head names now,
/// when it is at `new`.
/// Why:
///   The update of a detached head follows a head that was attached to a branch since the last look,
/// as native Git's commit does,
///  so the commit may have gone to the branch the head names.
/// The index is updated for a head that shows the commit,
/// whichever
///  way it shows it,
/// and left alone for a head that shows another commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shows = await followsMove(git, { repository, gitDir, commonDir }, reference, newCommit)
/// ```
///
/// # Errors
///
/// The failure code when Git must be asked and cannot answer.
pub fn follows_move(git: &GitContext, repository: &Path, git_dir: &Path, common_dir: &Path, reference: &RefName, new: &str) -> Result<bool, FailureCode> {
    if !reference.is_head() {
        return head_names(git, repository, git_dir, common_dir, reference);
    }
    if let HeadFile::Detached(value) = head_file(git_dir, common_dir) {
        return Ok(value == new);
    }
    return shows_the_commit(run_git(git, &head_commit_request(repository)), new);
}

/// `follows_move` with the reads and the Git child awaited on the runtime.
///
/// # Errors
///
/// As `follows_move`.
pub async fn follows_move_awaiting(git: &GitContext, repository: &Path, git_dir: &Path, common_dir: &Path, reference: &RefName, new: &str) -> Result<bool, FailureCode> {
    if !reference.is_head() {
        return head_names_awaiting(git, repository, git_dir, common_dir, reference).await;
    }
    if let HeadFile::Detached(value) = head_file_awaiting(git_dir, common_dir).await {
        return Ok(value == new);
    }
    return shows_the_commit(run_git_awaiting(git, &head_commit_request(repository)).await, new);
}

/// The question `follows_move` asks Git for a `HEAD` whose file cannot answer.
fn head_commit_request(repository: &Path) -> GitRequest {
    return GitRequest::new(repository, &["rev-parse", "--verify", "--quiet", "HEAD"]);
}

/// What:
///  Whether Git's answer to `head_commit_request` is the commit `moved_to`.
/// Why:
///  Both faces read the answer through this one function,
/// so status 1 is a head that names no commit for both of them,
///  and any other failure is no answer for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function showsTheCommit(answered: GitOutput | Error, movedTo: string): boolean; // throws when Git could not answer
/// ```
///
/// # Errors
///
/// `Unavailable` when Git could not be started or ended with a status other than 0 and 1.
fn shows_the_commit(answered: std::io::Result<GitOutput>, moved_to: &str) -> Result<bool, FailureCode> {
    let output: GitOutput = match answered {
        Ok(finished) => finished,
        Err(_unstartable) => return Err(FailureCode::Unavailable),
    };
    return match output.code {
        Some(0) => Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim() == moved_to),
        Some(1) => Ok(false),
        _ => Err(FailureCode::Unavailable),
    };
}

/// What:
///  Whether the head of the worktree whose Git directory is `git_dir` names `reference`.
/// Why:
///   The file is read when it is that worktree's own,
/// and Git is asked (in the worktree) when it cannot answer (a repository
///  with reference tables).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await headNames(git, { repository, gitDir, commonDir }, reference)
/// ```
pub fn head_names(git: &GitContext, repository: &Path, git_dir: &Path, common_dir: &Path, reference: &RefName) -> Result<bool, FailureCode> {
    match head_file(git_dir, common_dir) {
        HeadFile::Branch(named) => return Ok(named == *reference),
        HeadFile::Detached(_) => return Ok(false),
        HeadFile::Unreadable => {}
    }
    return names_of(symbolic_head(git, repository)?, reference);
}

/// `head_names` with the reads and the Git child awaited on the runtime.
///
/// # Errors
///
/// As `head_names`.
pub async fn head_names_awaiting(git: &GitContext, repository: &Path, git_dir: &Path, common_dir: &Path, reference: &RefName) -> Result<bool, FailureCode> {
    match head_file_awaiting(git_dir, common_dir).await {
        HeadFile::Branch(named) => return Ok(named == *reference),
        HeadFile::Detached(_) => return Ok(false),
        HeadFile::Unreadable => {}
    }
    return names_of(symbolic_head_awaiting(git, repository).await?, reference);
}

/// What:
///  Whether the reference `HEAD` named is `wanted`.
/// Why:
///  Both faces answer this through one function,
/// so a detached head is `false` for both and a head that names another branch is
///  `false` for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesOf(named: string | undefined, wanted: string): boolean;
/// ```
fn names_of(named: Option<RefName>, wanted: &RefName) -> Result<bool, FailureCode> {
    return match named {
        Some(found) => Ok(found == *wanted),
        None => Ok(false),
    };
}

/// What:
///  Whether the worktree still has the reference a commit was prepared on checked out in the way the commit needs:
/// the branch
///  `reference` for a named branch,
/// and no branch at all (a detached head) for `HEAD`.
/// Why:
///   The landing asks it before it waits for the index lock and again once it holds the lock,
/// because the worktree can switch
///  while it waits;
/// the ref transaction that moves the branch verifies the same thing at the moment of the move.
/// The detached case
///  reads the same `HEAD` file as the branch case,
/// so neither starts a Git ordinarily.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = await stillCheckedOut(git, { repository, gitDir, commonDir }, reference)
/// ```
///
/// # Errors
///
/// The failure code when Git must be asked and cannot be started.
pub fn still_checked_out(git: &GitContext, repository: &Path, git_dir: &Path, common_dir: &Path, reference: &RefName) -> Result<bool, FailureCode> {
    if !reference.is_head() {
        return head_names(git, repository, git_dir, common_dir, reference);
    }
    match head_file(git_dir, common_dir) {
        HeadFile::Branch(_) => return Ok(false),
        HeadFile::Detached(_) => return Ok(true),
        HeadFile::Unreadable => {}
    }
    return Ok(symbolic_head(git, repository)?.is_none());
}

/// `still_checked_out` with the reads and the Git child awaited on the runtime.
///
/// # Errors
///
/// As `still_checked_out`.
pub async fn still_checked_out_awaiting(git: &GitContext, repository: &Path, git_dir: &Path, common_dir: &Path, reference: &RefName) -> Result<bool, FailureCode> {
    if !reference.is_head() {
        return head_names_awaiting(git, repository, git_dir, common_dir, reference).await;
    }
    match head_file_awaiting(git_dir, common_dir).await {
        HeadFile::Branch(_) => return Ok(false),
        HeadFile::Detached(_) => return Ok(true),
        HeadFile::Unreadable => {}
    }
    return Ok(symbolic_head_awaiting(git, repository).await?.is_none());
}

/// Head controls stay out of the release executable.
#[cfg(test)]
#[path = "service_head_tests.rs"]
mod tests;

/// Controls for the awaited faces of the head questions stay out of the release executable.
#[cfg(test)]
#[path = "service_head_awaiting_tests.rs"]
mod awaiting_tests;
