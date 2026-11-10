//! Controls of the registration record a staged copy carries,
//! and of the check every installation makes against it.

use super::*;
use crate::test_support::{fixture, git, remove, repository};
use crate::worktree_add_binding::bind_registration;

/// A repository with a linked worktree beside it,
/// returned as its canonical common directory and that worktree's registration.
fn registered_worktree(name: &str) -> (PathBuf, PathBuf, crate::worktree_add_binding::Binding) {
    let root: PathBuf = fixture(name);
    let source: PathBuf = repository(root.as_path(), "source");
    let topic: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical root").join("topic");
    git(
        source.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            topic.to_str().expect("utf8"),
        ],
    );
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let binding = bind_registration(common.as_path(), topic.as_path()).expect("binding").binding;
    return (root, common, binding);
}

/// A stage container of a fixture,
/// made private as the staging step makes it.
fn container(root: &Path) -> PathBuf {
    let made: PathBuf = root.join(".cli-git-worktree-copy-stage");
    crate::private_storage::create_private_directory(made.as_path()).expect("container");
    return made;
}

/// A recorded binding reads back as the same registration,
/// and a stage that records none reads as none.
#[test]
fn a_recorded_binding_reads_back_whole_and_an_absent_one_reads_as_none() {
    let (root, _common, binding) = registered_worktree("stage-binding-round-trip");
    let stage: PathBuf = container(root.as_path());
    assert_eq!(stage_binding(stage.as_path()).expect("absent"), None);
    record_stage_binding(stage.as_path(), &binding).expect("recorded");
    assert_eq!(stage_binding(stage.as_path()).expect("read").as_ref(), Some(&binding));
    remove(root.as_path());
}

/// A binding record that is not a registration is reported and never guessed at.
#[test]
fn an_unreadable_binding_record_is_a_failure() {
    let (root, _common, _binding) = registered_worktree("stage-binding-unreadable");
    let stage: PathBuf = container(root.as_path());
    std::fs::write(stage.join(STAGE_BINDING_NAME), b"destination=zz\nphase=planned\n").expect("record");
    let failure: WorktreeCopyError = stage_binding(stage.as_path()).expect_err("unreadable");
    assert!(failure.0.contains("is unreadable"), "{}", failure.0);
    remove(root.as_path());
}

/// A binding still holds while the registration it names is the one at the destination,
/// and stops holding once another registration stands there.
#[test]
fn a_binding_holds_until_the_registration_behind_the_path_is_another() {
    let (root, common, binding) = registered_worktree("stage-binding-holds");
    verify_stage_binding(common.as_path(), &binding).expect("the registration is the bound one");
    git(
        std::fs::canonicalize(root.as_path()).expect("canonical root").join("source").as_path(),
        &[
            "worktree",
            "remove",
            "--force",
            binding.destination.to_str().expect("utf8"),
        ],
    );
    git(
        std::fs::canonicalize(root.as_path()).expect("canonical root").join("source").as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            "-b",
            "second",
            binding.destination.to_str().expect("utf8"),
        ],
    );
    let reason: String = verify_stage_binding(common.as_path(), &binding).expect_err("another registration");
    assert!(reason.contains(binding.admin.to_str().expect("utf8")), "the recorded registration is named: {reason}");
    assert!(reason.contains("is registered by"), "{reason}");
    remove(root.as_path());
}

/// How long a record that must not wait may take to answer in a control.
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(20);

/// A named pipe at `path`, which blocks whoever opens it for reading until a writer comes.
fn make_pipe(path: &Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("path text");
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(result, 0, "mkfifo {}", path.display());
}

/// What:
///  A named pipe where a stage's binding record sits is a failure with a reason,
///  answered without waiting for a writer.
/// Why:
///  Opening a pipe for reading waits for its writer,
/// so a recovery that reached the binding never returned,
///  the stage it was settling stayed where it was and the forwarded Git command never ran;
/// the registration scan beside this read already refuses a file that is not a plain one
/// (`SPEC.md`,
/// "Every wait has a bound";
/// a cross-vendor review's finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(stage/registration); const answer = await within(PATIENCE, () => stageBinding(stage)); expect(answer).toEqual(failure naming the stage);
/// ```
#[test]
fn a_named_pipe_where_the_binding_record_sits_is_a_failure_not_a_wait() {
    let (root, _common, _binding) = registered_worktree("stage-binding-pipe");
    let stage: PathBuf = container(root.as_path());
    make_pipe(stage.join(STAGE_BINDING_NAME).as_path());
    let (sender, receiver) = std::sync::mpsc::channel::<Result<Option<crate::worktree_add_binding::Binding>, WorktreeCopyError>>();
    let settled: PathBuf = stage.clone();
    std::thread::spawn(move || {
        let _ = sender.send(stage_binding(settled.as_path()));
    });
    let answer: Result<Option<crate::worktree_add_binding::Binding>, WorktreeCopyError> = match receiver.recv_timeout(PATIENCE) {
        Ok(found) => found,
        Err(_) => panic!("the stage's binding record waited for a writer of a pipe for {PATIENCE:?}"),
    };
    let failure: WorktreeCopyError = answer.err().expect("a pipe is not a registration this stage can carry");
    assert!(failure.0.contains(STAGE_BINDING_NAME), "the record is named: {}", failure.0);
    assert!(failure.0.contains(stage.to_string_lossy().as_ref()), "the stage is named: {}", failure.0);
    remove(root.as_path());
}
