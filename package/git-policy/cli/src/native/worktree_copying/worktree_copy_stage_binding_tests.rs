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
