//! Tests of the registration scan that answers which administrative directory names a destination.

use super::*;
use crate::test_support::{fixture, git, remove, repository};

/// A repository with one linked worktree at `root/topic`,
/// and its canonical common directory.
fn repository_with_a_linked_worktree(root: &Path, name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let source: PathBuf = repository(root, name);
    std::fs::write(source.join("tracked.txt"), b"tracked\n").expect("tracked");
    git(source.as_path(), &["add", "--", "tracked.txt"]);
    git(source.as_path(), &["commit", "--quiet", "--message=first"]);
    let topic: PathBuf = root.join("topic");
    git(source.as_path(), &["worktree", "add", "--quiet", "-b", "topic", topic.to_str().expect("utf8")]);
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    return (source, common, std::fs::canonicalize(topic).expect("canonical topic"));
}

/// A registration names the worktree its `gitdir` file names,
/// and its administrative directory is the one Git made.
#[test]
fn a_registration_names_the_worktree_git_registered() {
    let root: PathBuf = fixture("registration-naming-names");
    let (_source, common, topic) = repository_with_a_linked_worktree(root.as_path(), "repo");
    let answer: NamedRegistration = registration_naming(common.as_path(), topic.as_path());
    let NamedRegistration::Named(admin) = answer else {
        panic!("the registration names the worktree: {answer:?}");
    };
    assert_eq!(admin.parent(), Some(common.join("worktrees").as_path()));
    assert_eq!(admin.file_name().and_then(|name| { return name.to_str(); }), Some("topic"));
    remove(root.as_path());
}

/// A directory no registration names is answered as unregistered,
/// which is the absence a copy is discarded on.
#[test]
fn a_destination_no_registration_names_is_answered_as_none() {
    let root: PathBuf = fixture("registration-naming-none");
    let (_source, common, _topic) = repository_with_a_linked_worktree(root.as_path(), "repo");
    let stranger: PathBuf = root.join("stranger");
    std::fs::create_dir(stranger.as_path()).expect("a directory nobody registered");
    assert_eq!(registration_naming(common.as_path(), stranger.as_path()), NamedRegistration::None);
    remove(root.as_path());
}

/// A repository with no `worktrees` directory has no linked worktree,
/// so no registration names anything.
#[test]
fn a_repository_without_registrations_answers_none() {
    let root: PathBuf = fixture("registration-naming-without-worktrees");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("a common directory with no worktrees");
    assert_eq!(registration_naming(common.as_path(), root.as_path()), NamedRegistration::None);
    remove(root.as_path());
}

/// The `gitdir` file of one registration names the worktree whatever its destination's `.git` file holds:
/// a pointer a command is rewriting is not read here at all.
#[test]
fn the_answer_does_not_read_the_destinations_pointer() {
    let root: PathBuf = fixture("registration-naming-pointer-free");
    let (_source, common, topic) = repository_with_a_linked_worktree(root.as_path(), "repo");
    std::fs::write(topic.join(".git"), b"").expect("a pointer an interrupted rewrite left empty");
    let answer: NamedRegistration = registration_naming(common.as_path(), topic.as_path());
    assert!(matches!(answer, NamedRegistration::Named(_)), "{answer:?}");
    remove(root.as_path());
}

/// A registration whose `gitdir` was removed is one Git would prune,
/// and it names no worktree,
/// while a registration that cannot be read at all proves nothing.
#[test]
fn a_registration_without_a_gitdir_names_nothing() {
    let root: PathBuf = fixture("registration-naming-pruned");
    let (_source, common, topic) = repository_with_a_linked_worktree(root.as_path(), "repo");
    let admin: PathBuf = match registration_naming(common.as_path(), topic.as_path()) {
        NamedRegistration::Named(admin) => admin,
        other => panic!("the registration is there: {other:?}"),
    };
    std::fs::remove_file(admin.join("gitdir").as_path()).expect("the registration's gitdir goes, as Git's prune leaves it");
    assert_eq!(registration_naming(common.as_path(), topic.as_path()), NamedRegistration::None);
    remove(root.as_path());
}

/// A registration whose recorded worktree is gone names nothing,
/// as Git would prune it,
/// and the destination still exists.
#[test]
fn a_registration_whose_worktree_is_gone_names_nothing() {
    let root: PathBuf = fixture("registration-naming-worktree-gone");
    let (_source, common, topic) = repository_with_a_linked_worktree(root.as_path(), "repo");
    let admin: PathBuf = match registration_naming(common.as_path(), topic.as_path()) {
        NamedRegistration::Named(admin) => admin,
        other => panic!("the registration is there: {other:?}"),
    };
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(elsewhere.as_path()).expect("another directory");
    // The registration now names a directory that is not there, as a worktree removed behind Git's back leaves it.
    std::fs::write(admin.join("gitdir").as_path(), format!("{}/.git\n", elsewhere.join("gone").display()).as_bytes()).expect("a registration of a worktree that is gone");
    assert_eq!(registration_naming(common.as_path(), topic.as_path()), NamedRegistration::None);
    remove(root.as_path());
}
