//! Controls for the ending a commit has when the service made it and held it behind a reference lock,
//! and only the answer was lost:
//! the command ends with success,
//! no branch names the commit,
//! and no push of it is released.

use super::the_held_ending;
use crate::held_commit_record::{
    Held,
    write,
};
use crate::test_support::{
    fixture,
    git,
    git_text,
    remove,
};
use crate::transaction_git::GitContext;
use std::path::PathBuf;

/// The file the held commit changes.
const EDITED: &str = "edited.txt";

/// The subject of the held commit.
const SUBJECT: &str = "the held edit";

/// The lock the commit waited behind.
const LOCK: &str = "refs/heads/main.lock";

/// A repository with the branch at its base and the commit the branch does not name,
/// a journal,
/// and the record of the hold,
/// which is the durable state the service leaves before it answers.
fn held_state(label: &str) -> (PathBuf, PathBuf, PathBuf, Held) {
    let root: PathBuf = fixture(label);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(repo.as_path()).expect("the repository directory");
    git(
        repo.as_path(),
        &["init", "--quiet", "--initial-branch=main"],
    );
    std::fs::write(repo.join("base.txt"), b"base\n").expect("the base file");
    git(repo.as_path(), &["add", "--", "base.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"])
        .trim()
        .to_string();
    std::fs::write(repo.join(EDITED), b"the edit\n").expect("the edit");
    git(repo.as_path(), &["add", "--", EDITED]);
    git(repo.as_path(), &["commit", "--quiet", "--message", SUBJECT]);
    let held_commit: String = git_text(repo.as_path(), &["rev-parse", "HEAD"])
        .trim()
        .to_string();
    // The branch has not moved: the reference lock refused its write.
    git(
        repo.as_path(),
        &["update-ref", "--no-deref", "refs/heads/main", base.as_str()],
    );
    let git_dir: PathBuf = repo.join(".git");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("the journal");
    let held: Held = Held {
        seq: 1,
        journal: journal.clone(),
        repository: repo.clone(),
        common_dir: git_dir.clone(),
        git_dir: git_dir.clone(),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        old: base,
        new: held_commit.clone(),
        commits: vec![held_commit],
        index: git_dir.join("index"),
        reflog: b"commit: the held edit".to_vec(),
        locks: vec![git_dir.join(LOCK)],
        created: String::from("2026-01-01T00:00:00+00:00"),
    };
    write(git_dir.as_path(), &held).expect("the record of the hold");
    return (root, repo, journal, held);
}

/// What:
///  A commit the service held behind a reference lock ends the command with exit `0`,
///  names no landed commit,
///  and releases no push.
/// Why:
///  A received `held` answer ends exactly this way,
///  and the rule is that commits never fail,
///  so a command that learned the same thing from the record of the hold ends the same way.
#[test]
fn a_held_commit_ends_the_command_with_success_and_no_landed_commit() {
    let (root, repo, _journal, held) = held_state("landed-aftermath-held");
    let context: GitContext = GitContext {
        real_git: PathBuf::from("/usr/bin/git"),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: Some(held.common_dir.clone()),
    };
    let before: String = git_text(repo.as_path(), &["status", "--porcelain=v1"]);
    let ending: crate::service_commit::ServiceCommit =
        the_held_ending(&context, repo.as_path(), &held);
    assert_eq!(
        ending.result.exit_code, 0,
        "the commit is made and held, so the command succeeds"
    );
    assert_eq!(
        ending.result.landed_oid, None,
        "no branch names the commit yet"
    );
    assert_eq!(
        ending.result.events,
        Vec::new(),
        "the commit ran no policy on this command"
    );
    assert!(
        ending.handoff.is_none(),
        "no push is released for a commit no branch names"
    );
    assert_eq!(
        git_text(repo.as_path(), &["status", "--porcelain=v1"]),
        before,
        "the ending changes nothing in the repository"
    );
    assert_eq!(
        git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim(),
        held.old,
        "the branch has not moved"
    );
    remove(root.as_path());
}

/// What:
///  This child's standard error becomes a pipe filled to its whole capacity,
///  with the read end kept here and read by nobody,
///  and the number of bytes the fill put in it.
/// Why:
///   The words of a held commit are written on the command's own thread unless they are handed to the
///  thread that writes standard error,
///  and a reader that stopped draining then waits in the kernel with no end:
///  this is the stream that shows which of the two happened (issue 6421).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pipeOnStandardError = () => { const [reader, writer] = pipe(); fill(writer); dup2(writer.fd, 2); return [reader, capacity] };
/// ```
#[cfg(target_os = "linux")]
fn a_full_pipe_on_standard_error() -> (std::io::PipeReader, usize) {
    use std::io::Write;
    let (reader, writer) = std::io::pipe().expect("a pipe");
    let mut filler: std::io::PipeWriter = writer.try_clone().expect("a second write end");
    // The pipe's own capacity, which is how much one fill of it holds.
    // SAFETY: `fcntl` takes a descriptor this control owns and an integer command, and writes no memory here.
    let reported: std::ffi::c_int =
        unsafe { libc::fcntl(std::os::fd::AsRawFd::as_raw_fd(&reader), libc::F_GETPIPE_SZ) };
    assert!(
        reported > 0,
        "the kernel did not report what one pipe holds: {reported}"
    );
    let capacity: usize = usize::try_from(reported).expect("a positive capacity");
    filler
        .write_all(vec![b'p'; capacity].as_slice())
        .expect("the pipe fills");
    // What: This child's own standard error becomes the full pipe, with `dup2` on descriptor 2.
    // Why:  The writing code asks for this process's standard error, and nothing outside this child sees the change.
    // SAFETY: `dup2` copies one open descriptor of this process onto another number it names; both are this child's own.
    let copied: std::ffi::c_int = unsafe {
        libc::dup2(
            std::os::fd::AsRawFd::as_raw_fd(&writer),
            libc::STDERR_FILENO,
        )
    };
    assert_eq!(
        copied,
        libc::STDERR_FILENO,
        "this child's standard error is the pipe of this control"
    );
    return (reader, capacity);
}

/// What:
///  A commit the service made and held behind a reference lock ends the command with success
///  while nobody drains its standard error.
/// Why:
///   The words of the hold are the commit's own last note,
///  and a reader that stopped draining held them for ever when they were written on the commit's own thread
///  (issue 6421).
///  The record of the hold is the whole input of that ending,
///  so this control needs no service and no command,
///  and its own standard error is a pipe it fills first.
///  A child of two runs the body,
///  because the pipe is this process's standard error and the queue
///  that writes it belongs to the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inChild(() => { const [, capacity] = fullPipeOnStandardError(); const held = heldState(); const began = now(); expect(theHeldEnding(held).exitCode).toBe(0); expect(now() - began).toBeLessThan(5s) })
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_held_commit_s_answer_returns_while_nobody_drains_its_standard_error() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_held_commit_s_answer_returns_while_nobody_drains_its_standard_error",
        || {
            let (root, repo, _journal, held) = held_state("landed-aftermath-held-stalled");
            let (pipe, capacity): (std::io::PipeReader, usize) = a_full_pipe_on_standard_error();
            let context: GitContext = GitContext {
                real_git: PathBuf::from("/usr/bin/git"),
                overlay: Vec::new(),
                global_prefix: Vec::new(),
                common_dir: Some(held.common_dir.clone()),
            };
            let began: std::time::Instant = std::time::Instant::now();
            let ending: crate::service_commit::ServiceCommit =
                the_held_ending(&context, repo.as_path(), &held);
            let took: std::time::Duration = began.elapsed();
            assert_eq!(
                ending.result.exit_code, 0,
                "the commit is made and held, so the command succeeds"
            );
            assert_eq!(
                ending.result.landed_oid, None,
                "no branch names the commit yet"
            );
            assert!(
                took < std::time::Duration::from_secs(5),
                "the words of a held commit were said in {took:?}, so the command waited for a reader"
            );
            // What: How many bytes the pipe still holds, read without taking any.
            // Why:  The fill is the proof that nobody read the pipe, so a note that reached it cannot have.
            let mut waiting: libc::c_int = 0;
            // SAFETY: `FIONREAD` writes one integer into the buffer it is given, and the descriptor is open.
            assert_eq!(
                unsafe {
                    libc::ioctl(
                        std::os::fd::AsRawFd::as_raw_fd(&pipe),
                        libc::FIONREAD,
                        &mut waiting,
                    )
                },
                0,
                "how many bytes the pipe holds"
            );
            assert_eq!(
                usize::try_from(waiting).unwrap_or_default(),
                capacity,
                "the pipe was read by nobody, so every byte this control put in it is still there"
            );
            drop(pipe);
            remove(root.as_path());
        },
    );
}
