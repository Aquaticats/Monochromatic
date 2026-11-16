//! What:
//!  A named pipe planted at a record's name in the work queue,
//!  and what a wrapped command does with it.
//! Why:
//!  A name in the queue can be made by any process that may write the repository's Git directory,
//!  and opening a named pipe for reading waits for a writer that may never come.
//! The second review of the work-record change measured the cost:
//!  with the built wrapper as `git`,
//!  `git add a-file` in a repository that holds a pipe in `pending/` or in `failed/` ran until its 20-second bound
//!  ended it,
//!  where the same command without the pipe took under a tenth of a second
//!  (2026-11-16, `reproductions/fifo-through-the-wrapper.log` in the review folder).
//!  The claim step's own open is fixed in `landing/work_item.rs` (`plain_file::open_plain_file`),
//! and the two reads a command makes are not:
//!  `landing/service_handoff.rs` (`queued_push_notices` reads `pending/push-*`) and
//!  `worktree/worktree_remove_hand_over.rs` (`requeue_removals_an_earlier_release_refused` reads every `failed/` entry).
//!  So the two controls here are held back from the suite with their reason,
//! and they are expected to fail on `main` and on this branch until those two files read their records as plain files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const taken = mkfifo(`${work}/failed/subvolume-delete-0…`); const began = Date.now(); const added = git(['add', 'a-file']); expect(Date.now() - began).toBeLessThan(20_000); expect(added.status).toBe(0);
//! ```

use super::support::{Fixture, bounded_for, fixture, wrapper_first_path};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// How long a command that reads the queue is given:
/// the bound the review used,
/// which is two hundred times what the same command takes in a repository with no pipe.
const A_BOUND_A_COMMAND_NEVER_TAKES: &str = "20";

/// Real Git in the fixture's hermetic environment.
fn in_the_fixture(fixture: &Fixture, directory: &Path, arguments: &[&OsStr]) -> Output {
    let mut command: Command = Command::new("/usr/bin/git");
    command.env_clear().env("PATH", "/usr/bin:/bin").env("HOME", fixture.root.join("home")).env("GIT_CONFIG_NOSYSTEM", "1");
    command.env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config");
    command.current_dir(directory);
    return command.args(arguments).output().expect("real Git in the fixture");
}

/// A repository with one commit and a file to add,
///  so a wrapped `add` runs the queue's own steps and writes one record.
fn a_repository(fixture: &Fixture, root: &Path) -> PathBuf {
    std::fs::create_dir_all(root).expect("repository directory");
    let initialized: Output = in_the_fixture(fixture, root, &[OsStr::new("init"), OsStr::new("--quiet")]);
    assert!(initialized.status.success(), "git init: {initialized:?}");
    std::fs::write(root.join("a-file"), b"x\n").expect("a file to add");
    return root.to_path_buf();
}

/// The work directory of a repository, made whole before a control plants anything in it.
fn the_work_directory(repository: &Path) -> PathBuf {
    let work: PathBuf = repository.join(".git").join("monochromatic-git-policy-cli").join("work");
    for state in ["pending", "claimed", "recovering", "done", "failed"] {
        std::fs::create_dir_all(work.join(state)).expect("a state directory");
    }
    return work;
}

/// A named pipe at `path`, which `mkfifo` makes and nothing opens.
fn a_named_pipe(path: &Path) {
    let made: Output = Command::new("mkfifo").arg(path).output().expect("mkfifo");
    assert!(made.status.success(), "mkfifo {}: {made:?}", path.display());
}

/// A wrapped `add` of one path, with the fixture's bound.
fn a_wrapped_add(fixture: &Fixture, repository: &Path) -> Output {
    let path = wrapper_first_path(fixture);
    let mut command: Command = bounded_for(fixture, &fixture.wrapper, &path, A_BOUND_A_COMMAND_NEVER_TAKES);
    return command.current_dir(repository).args(["add", "a-file"]).output().expect("the wrapped add");
}

/// The same command without the pipe is the control each held-back control below compares against:
/// it must end on its own, and it must succeed, or nothing below says anything about a pipe.
fn i_am_the_control(fixture: &Fixture, repository: &Path) {
    let added: Output = a_wrapped_add(fixture, repository);
    let said: String = format!("the wrapped add without a pipe: {:?} {}", added.status, String::from_utf8_lossy(added.stderr.as_slice()));
    assert!(added.status.success(), "{said}");
}

/// A named pipe at a record's name in `failed` does not hold a wrapped command:
/// `requeue_removals_an_earlier_release_refused` reads every entry of that directory.
#[test]
#[ignore = "fails on main and on this branch: worktree/worktree_remove_hand_over.rs reads a record with read_to_string, which waits for ever on a named pipe"]
fn a_named_pipe_in_failed_does_not_hold_a_wrapped_command() {
    let fixture: Fixture = fixture("work-record-pipe-failed");
    let repository: PathBuf = a_repository(&fixture, fixture.root.join("repo").as_path());
    i_am_the_control(&fixture, repository.as_path());
    a_named_pipe(the_work_directory(repository.as_path()).join("failed").join("subvolume-delete-0123456789abcdef0123456789abcdef01234567-0123456789abcdef").as_path());
    let added: Output = a_wrapped_add(&fixture, repository.as_path());
    super::support_fixture_directories::remove(&fixture);
    assert!(added.status.success(), "a pipe in failed held the add: {:?} {}", added.status, String::from_utf8_lossy(added.stderr.as_slice()));
}

/// A named pipe at a record's name in `pending` does not hold a wrapped command:
/// `queued_push_notices` reads the `pending` records a push is waiting as.
#[test]
#[ignore = "fails on main and on this branch: landing/service_handoff.rs reads a pending push record with read_to_string, which waits for ever on a named pipe"]
fn a_named_pipe_in_pending_does_not_hold_a_wrapped_command() {
    let fixture: Fixture = fixture("work-record-pipe-pending");
    let repository: PathBuf = a_repository(&fixture, fixture.root.join("repo").as_path());
    i_am_the_control(&fixture, repository.as_path());
    let pipe: PathBuf = the_work_directory(repository.as_path()).join("pending").join("push-0123456789abcdef0123456789abcdef01234567-f921bd05e68b0374");
    a_named_pipe(pipe.as_path());
    // A record younger than the queue's own bound is a push in flight and is not named, so the pipe is aged past it.
    let aged: Output = Command::new("touch").args(["-d", "2020-01-01"]).arg(pipe.as_path()).output().expect("touch");
    assert!(aged.status.success(), "age the pipe: {aged:?}");
    let added: Output = a_wrapped_add(&fixture, repository.as_path());
    super::support_fixture_directories::remove(&fixture);
    assert!(added.status.success(), "a pipe in pending held the add: {:?} {}", added.status, String::from_utf8_lossy(added.stderr.as_slice()));
}
