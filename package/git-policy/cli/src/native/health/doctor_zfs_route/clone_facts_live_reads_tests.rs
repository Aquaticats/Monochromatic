//! What:
//!  The controls for the release answers that need no pool:
//!  a path's state,
//!  and which tasks the caller's own session keeps.
//! Why:
//!  This file's reader opens `/dev/zfs` behind the compatibility gate the
//!  moment it is made,
//!  so no host control may make one
//!  (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
//!  "No test on the host opens `/dev/zfs`").
//!  The answers that need the pool are covered by the gatherer's guest arm,
//!  and the two that do not are the ones a report every person reads depends
//!  on,
//!  so they are proved here.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theStateOfAPath(fixture)).toEqual({ kind: 'directory', owner, mode: 448 });
//! expect(theTasksInTheCallersSession([process.pid])).toEqual([process.pid]);
//! ```

/// The two answers this file proves.
use super::the_state_of_a_path;
use super::the_tasks_in_the_callers_session;
/// What stands at one path.
use crate::doctor_zfs_route::PathState;
/// One fixture folder of the control's own.
use crate::test_support::{
    fixture,
    remove,
};
use std::os::unix::fs::{
    PermissionsExt as _,
    symlink,
};

/// A task number no process on this machine can hold.
const A_NUMBER_NO_TASK_HOLDS: u32 = 999_999_999;

/// The permission bits a directory of this account starts with.
const FOLDER_MODE: u32 = 0o700;

/// What:
///  A folder is a directory with its owner and mode,
///  and a path nothing stands at is absent.
#[test]
fn a_path_is_read_as_a_directory_or_as_absent() {
    let folder: std::path::PathBuf = fixture("clone-facts-path-state");
    let inside: std::path::PathBuf = folder.join("a-folder");
    std::fs::create_dir(inside.as_path()).expect("the folder");
    std::fs::set_permissions(
        inside.as_path(),
        std::fs::Permissions::from_mode(FOLDER_MODE),
    )
    .expect("the folder's mode");
    let state: PathState = the_state_of_a_path(inside.as_path());
    let PathState::Directory {
        owner,
        mode,
    } = state
    else {
        panic!("a folder is a directory: {state:?}");
    };
    assert_eq!(owner, rustix::process::geteuid().as_raw());
    assert_eq!(mode, FOLDER_MODE);
    assert_eq!(
        the_state_of_a_path(folder.join("nothing-here").as_path()),
        PathState::Absent
    );
    remove(folder.as_path());
}

/// What:
///  A look this account is refused answers unreadable,
///  never absent.
/// Why:
///  "Nothing stands there" and "I may not look" send a person to two different
///  places,
///  and a registration that cannot be looked at must not read as one that is
///  gone.
#[test]
fn a_refused_look_is_unreadable_and_not_absent() {
    let folder: std::path::PathBuf = fixture("clone-facts-refused-look");
    let closed: std::path::PathBuf = folder.join("closed");
    std::fs::create_dir(closed.as_path()).expect("the closed folder");
    std::fs::set_permissions(closed.as_path(), std::fs::Permissions::from_mode(0o000))
        .expect("the closed folder's mode");
    let state: PathState = the_state_of_a_path(closed.join("inside").as_path());
    std::fs::set_permissions(
        closed.as_path(),
        std::fs::Permissions::from_mode(FOLDER_MODE),
    )
    .expect("the folder is readable again");
    let PathState::Unreadable {
        why,
    } = state
    else {
        panic!("a refused look is unreadable: {state:?}");
    };
    assert!(!why.is_empty(), "the refusal says what it was");
    remove(folder.as_path());
}

/// What:
///  A link is its own kind of entry and not a file the route reads.
#[test]
fn a_symbolic_link_is_another_kind() {
    let folder: std::path::PathBuf = fixture("clone-facts-symlink-state");
    let target: std::path::PathBuf = folder.join("a-file");
    std::fs::write(target.as_path(), b"bytes\n").expect("the file");
    let link: std::path::PathBuf = folder.join("a-link");
    symlink(target.as_path(), link.as_path()).expect("the link");
    let state: PathState = the_state_of_a_path(link.as_path());
    let PathState::WrongKind {
        found,
    } = state
    else {
        panic!("a link is another kind: {state:?}");
    };
    assert!(found.contains("link"), "{found}");
    remove(folder.as_path());
}

/// What:
///  The caller's own task is in its session,
///  and a number no process holds is in none.
#[test]
fn the_callers_task_is_in_its_session() {
    let ours: u32 = std::process::id();
    let mine: Vec<u32> = the_tasks_in_the_callers_session(&[A_NUMBER_NO_TASK_HOLDS, ours]);
    assert_eq!(mine, Vec::from([ours]));
}

/// What:
///  No task at all is no member at all.
#[test]
fn a_report_with_no_holder_names_no_member() {
    assert!(the_tasks_in_the_callers_session(&[]).is_empty());
}
