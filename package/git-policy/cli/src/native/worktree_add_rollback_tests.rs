//! Controls for the binding of an add and the removal of what it made: the pair the repository confirms is bound and
//!  removed, every object that is not the bound one is kept, and a removal that cannot finish says so.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, git, git_text, remove, repository};
use crate::worktree_ladder_test_support::{commit_all, tracked};
use std::os::unix::fs::PermissionsExt;

/// An ordinary repository with a worktree registered without a checkout beside it: the repository, the destination, and the
///  administrative directory.
fn registered(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), "file.txt", b"content\n");
    commit_all(repo.as_path(), "first");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    return (repo, destination, admin);
}

/// A bound worktree holding only Git's pointer, or a partial checkout, is removed with its registration, and the repository
///  lists the main worktree alone afterwards.
#[test]
fn a_bound_incomplete_worktree_is_removed_with_its_registration() {
    for partial in [false, true] {
        let root: PathBuf = fixture(if partial { "rollback-partial" } else { "rollback-pointer" });
        let (repo, destination, admin) = registered(root.as_path());
        let bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
        if partial {
            std::fs::create_dir_all(destination.join("dir")).expect("directory");
            std::fs::write(destination.join("dir/file"), b"half").expect("half a checkout");
        }
        assert_eq!(remove_bound(&bound.binding), Removal::Removed);
        assert!(!destination.exists() && !admin.exists(), "partial {partial}");
        assert_eq!(git_text(repo.as_path(), &["worktree", "list", "--porcelain"]).lines().filter(|line: &&str| { return line.starts_with("worktree "); }).count(), 1);
        assert!(repo.join("file.txt").is_file(), "the main worktree is untouched");
        remove(root.as_path());
    }
}

/// A pair the repository does not confirm is not bound: a pointer outside this repository's worktrees, one that names another
///  worktree's registration (an edited pointer), a registration that names another place, and a destination reached through a
///  link.
#[test]
fn a_pair_the_repository_does_not_confirm_is_not_bound() {
    let root: PathBuf = fixture("rollback-unconfirmed");
    let (repo, destination, admin) = registered(root.as_path());
    let common: PathBuf = repo.join(".git");
    // A pointer at a directory outside the repository's worktrees.
    let foreign: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical").join("foreign");
    std::fs::create_dir_all(foreign.join("admin")).expect("foreign admin");
    std::fs::write(foreign.join("admin/gitdir"), format!("{}\n", foreign.join("look/.git").display())).expect("gitdir");
    std::fs::create_dir_all(foreign.join("look")).expect("look");
    std::fs::write(foreign.join("look/.git"), format!("gitdir: {}\n", foreign.join("admin").display())).expect("pointer");
    assert!(bind_registration(common.as_path(), foreign.join("look").as_path()).expect_err("not registered").contains("not registered"));
    // A pointer edited to name the source's own git directory.
    let edited: PathBuf = foreign.join("edited");
    std::fs::create_dir_all(edited.as_path()).expect("edited");
    std::fs::write(edited.join(".git"), format!("gitdir: {}\n", common.display())).expect("pointer at the main git directory");
    assert!(bind_registration(common.as_path(), edited.as_path()).is_err(), "the main git directory is not a registration");
    // A pointer edited to name another worktree's registration: its back-link names another place.
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "other", "../other"]);
    std::fs::write(destination.join(".git"), format!("gitdir: {}\n", repo.join(".git/worktrees/other").display())).expect("edited pointer");
    assert!(bind_registration(common.as_path(), destination.as_path()).expect_err("not named back").contains("does not name"));
    std::fs::write(destination.join(".git"), format!("gitdir: {}\n", admin.display())).expect("restored");
    // A destination named through a link.
    let link: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical").join("link");
    std::os::unix::fs::symlink(destination.as_path(), link.as_path()).expect("link");
    assert!(bind_registration(common.as_path(), link.as_path()).is_err(), "a link is not the directory");
    // Positive control: the real pair binds.
    assert!(bind_registration(common.as_path(), destination.as_path()).is_ok());
    remove(root.as_path());
}

/// A directory that replaced the bound destination is not the add's: nothing is removed there, and the registration stays
///  because it still serves whatever is at the destination.
#[test]
fn a_replacement_at_the_destination_is_kept_with_the_registration() {
    let root: PathBuf = fixture("rollback-replaced");
    let (repo, destination, admin) = registered(root.as_path());
    let bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    std::fs::rename(destination.as_path(), destination.with_file_name("topic.moved")).expect("moved away");
    std::fs::create_dir(destination.as_path()).expect("another directory at the path");
    std::fs::write(destination.join("theirs.txt"), b"someone's work").expect("their file");
    let outcome: Removal = remove_bound(&bound.binding);
    assert!(matches!(&outcome, Removal::Kept { replaced: true, .. }), "{outcome:?}");
    assert_eq!(std::fs::read(destination.join("theirs.txt")).expect("kept"), b"someone's work");
    assert!(admin.exists(), "the registration stays while its destination is not removed");
    remove(root.as_path());
}

/// A registration that was replaced is kept too, and the destination, which is still the bound one, goes.
#[test]
fn a_replaced_registration_is_kept() {
    let root: PathBuf = fixture("rollback-admin-replaced");
    let (repo, destination, admin) = registered(root.as_path());
    let bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    std::fs::rename(admin.as_path(), admin.with_file_name("topic.moved")).expect("moved");
    std::fs::create_dir(admin.as_path()).expect("another directory");
    std::fs::write(admin.join("marker"), b"theirs").expect("their file");
    let outcome: Removal = remove_bound(&bound.binding);
    assert!(matches!(&outcome, Removal::Kept { replaced: true, .. }), "{outcome:?}");
    assert!(!destination.exists(), "the bound destination went");
    assert_eq!(std::fs::read(admin.join("marker")).expect("kept"), b"theirs");
    remove(root.as_path());
}

/// A removal that cannot finish is reported as pending, not as replaced, and removes nothing it cannot account for.
#[test]
fn a_removal_that_fails_is_reported_as_pending() {
    let root: PathBuf = fixture("rollback-failure");
    let (repo, destination, admin) = registered(root.as_path());
    let bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    let parent: PathBuf = destination.parent().expect("parent").to_path_buf();
    std::fs::set_permissions(parent.as_path(), std::fs::Permissions::from_mode(0o500)).expect("read-only parent");
    let outcome: Removal = remove_bound(&bound.binding);
    std::fs::set_permissions(parent.as_path(), std::fs::Permissions::from_mode(0o755)).expect("writable again");
    assert!(matches!(&outcome, Removal::Kept { replaced: false, .. }), "{outcome:?}");
    assert!(destination.join(".git").exists() && admin.exists(), "nothing was removed");
    remove(root.as_path());
}

/// The record text of an add round-trips, a name with a newline cannot forge a field, and anything unreadable is nothing.
#[test]
fn a_record_round_trips_and_a_forged_one_is_nothing() {
    let planned: PathBuf = PathBuf::from("/base/line\nphase=registered");
    assert_eq!(decode_record(planned_record(planned.as_path()).as_str()), Some(Recorded::Planned(planned)));
    let binding: Binding = Binding {
        destination: PathBuf::from("/base/topic"),
        destination_id: ObjectId { device: 1, inode: 2, birth: Some((3, 4)) },
        admin: PathBuf::from("/repo/.git/worktrees/topic"),
        admin_id: ObjectId { device: 5, inode: 6, birth: None },
        pointer: b"gitdir: /repo/.git/worktrees/topic\n".to_vec(),
    };
    assert_eq!(decode_record(binding.encode().as_str()), Some(Recorded::Registered(binding.clone())));
    for broken in ["", "destination=zz\nphase=planned\n", "destination=2f\nphase=other\n", "phase=planned\n"] {
        assert_eq!(decode_record(broken), None, "{broken:?}");
    }
    let missing_field: String = binding.encode().replace("pointer=", "pointe=");
    assert_eq!(decode_record(missing_field.as_str()), None);
}

/// A bound destination that is a subvolume with contents is removed whole.
#[test]
fn btrfs_a_subvolume_destination_with_contents_is_removed() {
    let Some(root) = btrfs_fixture("rollback-subvolume") else { return };
    let (repo, destination, admin) = registered(root.as_path());
    std::fs::remove_dir_all(destination.as_path()).expect("replace the directory");
    crate::fs_btrfs_ioctl::subvolume_create_at(destination.as_path()).expect("subvolume");
    std::fs::write(destination.join(".git"), format!("gitdir: {}\n", admin.display())).expect("pointer");
    let bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    std::fs::create_dir_all(destination.join("deep/er")).expect("nested");
    std::fs::write(destination.join("deep/er/file"), b"x").expect("file");
    assert_eq!(remove_bound(&bound.binding), Removal::Removed);
    assert!(!destination.exists() && !admin.exists());
    remove(root.as_path());
}
