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

/// A registration whose `gitdir` file is gone for a moment names nothing that could be read:
/// a command that unlinks the file before writing it again leaves it so,
/// and the registration itself stays.
/// No answer about the destination is proven by a file that was not there to read.
#[test]
fn a_gitdir_that_is_gone_proves_nothing_about_a_registration() {
    let root: PathBuf = fixture("registration-naming-gitdir-gone");
    let (_source, common, topic) = repository_with_a_linked_worktree(root.as_path(), "repo");
    let admin: PathBuf = match registration_naming(common.as_path(), topic.as_path()) {
        NamedRegistration::Named(admin) => admin,
        other => panic!("the registration is there: {other:?}"),
    };
    std::fs::remove_file(admin.join("gitdir").as_path()).expect("the registration's gitdir is gone while it is written again");
    assert_eq!(registration_naming(common.as_path(), topic.as_path()), NamedRegistration::Unknown);
    remove(root.as_path());
}

/// A registration whose `gitdir` is empty names nothing that could be read:
/// a command that truncates the file before rewriting it leaves it so,
/// and the registration itself stays.
#[test]
fn an_empty_gitdir_proves_nothing_about_a_registration() {
    let root: PathBuf = fixture("registration-naming-gitdir-empty");
    let (_source, common, topic) = repository_with_a_linked_worktree(root.as_path(), "repo");
    let admin: PathBuf = match registration_naming(common.as_path(), topic.as_path()) {
        NamedRegistration::Named(admin) => admin,
        other => panic!("the registration is there: {other:?}"),
    };
    std::fs::write(admin.join("gitdir").as_path(), b"").expect("a gitdir cut off before its path");
    assert_eq!(registration_naming(common.as_path(), topic.as_path()), NamedRegistration::Unknown);
    remove(root.as_path());
}

/// A registration whose recorded worktree cannot be resolved proves nothing:
/// a directory another command makes unsearchable for a moment leaves every path through it unresolved,
/// while the registration,
/// the destination and its pointer stay unchanged.
#[cfg(unix)]
#[test]
fn a_gitdir_that_cannot_be_resolved_proves_nothing_about_a_registration() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("registration-naming-unresolvable");
    let (_source, common, topic) = repository_with_a_linked_worktree(root.as_path(), "repo");
    let admin: PathBuf = match registration_naming(common.as_path(), topic.as_path()) {
        NamedRegistration::Named(admin) => admin,
        other => panic!("the registration is there: {other:?}"),
    };
    let closed: PathBuf = root.join("closed");
    std::fs::create_dir(closed.as_path()).expect("a directory another command closes for a moment");
    std::fs::set_permissions(closed.as_path(), std::fs::Permissions::from_mode(0o000)).expect("no search through it");
    std::fs::write(admin.join("gitdir").as_path(), format!("{}/see/.git\n", closed.display()).as_bytes()).expect("a gitdir recorded through the closed directory");
    let answer: NamedRegistration = registration_naming(common.as_path(), topic.as_path());
    std::fs::set_permissions(closed.as_path(), std::fs::Permissions::from_mode(0o700)).expect("searchable again");
    assert_eq!(answer, NamedRegistration::Unknown);
    remove(root.as_path());
}

/// A recorded path is bytes,
/// and the first of them can be a space:
/// `bind_registration` removes the terminating line feed alone,
/// so a registration whose `gitdir` begins with a space names the worktree inside the registration,
/// not the sibling those bytes name once the space is trimmed away.
#[cfg(unix)]
#[test]
fn a_recorded_path_keeps_the_space_it_begins_with() {
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("registration-naming-leading-space");
    let common: PathBuf = root.join("common");
    let admin: PathBuf = common.join("worktrees/odd");
    let destination: PathBuf = admin.join(std::ffi::OsStr::from_bytes(b" \xff"));
    std::fs::create_dir_all(destination.as_path()).expect("the worktree the registration names");
    std::fs::write(admin.join("gitdir").as_path(), b" \xff/.git\n").expect("a gitdir recorded relative to the registration");
    let answer: NamedRegistration = registration_naming(common.as_path(), destination.as_path());
    assert_eq!(answer, NamedRegistration::Named(admin), "{answer:?}");
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

/// Git keeps a worktree's directory object when it moves it,
/// so the object a record captured finds the worktree at the path its registration names now.
#[test]
fn a_moved_worktree_is_found_by_the_object_its_record_captured() {
    let root: PathBuf = fixture("registration-naming-moved-object");
    let (source, common, topic) = repository_with_a_linked_worktree(root.as_path(), "repo");
    let identity: crate::fs_beneath::ObjectId = crate::worktree_add_binding::path_identity(topic.as_path()).expect("the topic's identity");
    let moved: PathBuf = root.join("moved");
    git(source.as_path(), &["worktree", "move", topic.to_str().expect("utf8"), moved.to_str().expect("utf8")]);
    let moved_root: PathBuf = std::fs::canonicalize(moved).expect("canonical moved");
    assert_eq!(
        registered_root_with_identity(common.as_path(), identity),
        Some(moved_root),
        "the registration names the worktree where it stands now"
    );
    remove(root.as_path());
}

/// No registration holds a directory that is not one of this repository's worktrees,
/// and a common directory with no `worktrees` directory holds no object at all.
#[test]
fn an_object_no_registration_holds_is_answered_as_none() {
    let root: PathBuf = fixture("registration-naming-no-object");
    let (_source, common, _topic) = repository_with_a_linked_worktree(root.as_path(), "repo");
    let stranger: PathBuf = root.join("stranger");
    std::fs::create_dir(stranger.as_path()).expect("a directory nobody registered");
    let stranger_identity: crate::fs_beneath::ObjectId = crate::worktree_add_binding::path_identity(stranger.as_path()).expect("the stranger's identity");
    assert_eq!(registered_root_with_identity(common.as_path(), stranger_identity), None);
    let unregistered: PathBuf = root.join("empty-common");
    std::fs::create_dir(unregistered.as_path()).expect("a common directory with no worktrees");
    assert_eq!(
        registered_root_with_identity(unregistered.as_path(), stranger_identity),
        None,
        "a repository with no registration holds no object"
    );
    remove(root.as_path());
}

/// A registration whose `gitdir` is gone for a moment names no root,
/// and the scan answers without it rather than on a guess:
/// a record it would have named is kept,
/// never settled.
#[test]
fn a_registration_whose_gitdir_is_gone_is_skipped() {
    let root: PathBuf = fixture("registration-naming-gitdir-gone-object");
    let (_source, common, topic) = repository_with_a_linked_worktree(root.as_path(), "repo");
    let identity: crate::fs_beneath::ObjectId = crate::worktree_add_binding::path_identity(topic.as_path()).expect("the topic's identity");
    let admin: PathBuf = match registration_naming(common.as_path(), topic.as_path()) {
        NamedRegistration::Named(admin) => admin,
        other => panic!("the registration is there: {other:?}"),
    };
    std::fs::remove_file(admin.join("gitdir").as_path()).expect("the registration's gitdir is gone while it is written again");
    assert_eq!(
        registered_root_with_identity(common.as_path(), identity),
        None,
        "a gitdir that is not there names no root"
    );
    remove(root.as_path());
}
