//! What:
//!  Controls of one namespace's table read:
//!  the first live task answers and no later task is read,
//!  a task that ended is kept as an attempt and the next task answers,
//!  a table this process may not open is kept as its own kind,
//!  and a namespace whose tasks all ended has no table at all.
//! Why:
//!  The holder scan reports every incomplete read and never calls an unread
//!  namespace empty,
//!  which it can only do if this reader tells the two failures apart and keeps
//!  the attempts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readOneNamespace(group).attempts.map((a) => a.kind)).toEqual(['ended', 'read']);
//! ```

use super::{NamespaceTableRead, TaskTableAttempt, read_one_namespace};
use crate::mount_namespace_scan::mount_namespace_tasks::MountNamespaceTasks;
use crate::testing::test_support::fixture;
use std::path::{Path, PathBuf};

/// The mount line one fixture table holds,
/// which is a mount of the dataset the controls scan for.
pub const ONE_MOUNT: &str = "41 30 0:32 / /home/user/Monochromatic ro,nosuid,nodev - zfs zpcachyos/nosnap/clones/one ro\n";

/// One fixture task with a mount table,
/// or with nothing where the table belongs.
fn task_with_table(root: &Path, pid: &str, table: Option<&str>) {
    let folder: PathBuf = root.join(pid);
    std::fs::create_dir_all(folder.as_path()).expect("the fixture task is made");
    if let Some(text) = table {
        std::fs::write(folder.join("mountinfo"), text).expect("the fixture table is written");
    }
}

/// The first live task answers,
/// and the tasks after it are not read.
#[test]
fn the_first_task_that_answers_is_the_namespace_s_table() {
    let root: PathBuf = fixture("namespace-table-first");
    task_with_table(root.as_path(), "11", Some(ONE_MOUNT));
    task_with_table(root.as_path(), "12", Some(ONE_MOUNT));
    let read: NamespaceTableRead = read_one_namespace(&MountNamespaceTasks { namespace: String::from("mnt:[7]"), tasks: vec![root.join("11"), root.join("12")] });
    assert_eq!(read.attempts.len(), 1);
    let (task, mounts) = read.table().expect("the first task answered");
    assert_eq!(task, root.join("11"));
    assert_eq!(mounts.len(), 1);
    assert_eq!(mounts[0].source, String::from("zpcachyos/nosnap/clones/one"));
    assert!(!read.has_no_live_member());
}

/// A task that ended between the listing and the read is kept as its own
/// attempt,
/// and the namespace's next task answers.
#[test]
fn a_task_that_ended_is_an_attempt_and_the_next_task_answers() {
    let root: PathBuf = fixture("namespace-table-ended");
    task_with_table(root.as_path(), "11", None);
    task_with_table(root.as_path(), "12", Some(ONE_MOUNT));
    let read: NamespaceTableRead = read_one_namespace(&MountNamespaceTasks { namespace: String::from("mnt:[7]"), tasks: vec![root.join("11"), root.join("12")] });
    assert_eq!(read.attempts.len(), 2);
    assert!(matches!(read.attempts[0], TaskTableAttempt::Ended { .. }), "{:?}", read.attempts[0]);
    assert_eq!(read.attempts[0].task(), root.join("11"));
    assert!(matches!(read.attempts[1], TaskTableAttempt::Read { .. }));
    assert!(read.table().is_some());
}

/// A table this process may not open is its own kind of attempt,
/// so a caller can tell it from a task that ended,
/// and a namespace with no table says so.
#[test]
fn an_unreadable_table_is_not_a_task_that_ended() {
    let root: PathBuf = fixture("namespace-table-unreadable");
    let task: PathBuf = root.join("11");
    std::fs::create_dir_all(task.join("mountinfo")).expect("a directory stands where the table belongs");
    let read: NamespaceTableRead = read_one_namespace(&MountNamespaceTasks { namespace: String::from("mnt:[7]"), tasks: vec![task.clone()] });
    assert_eq!(read.attempts.len(), 1);
    assert!(matches!(read.attempts[0], TaskTableAttempt::Unreadable { .. }), "{:?}", read.attempts[0]);
    assert_eq!(read.attempts[0].task(), task);
    assert!(read.table().is_none());
    assert!(read.has_no_live_member());
}

/// A namespace whose every task ended has no table,
/// which is a namespace with no live member to read through.
#[test]
fn a_namespace_whose_every_task_ended_has_no_table() {
    let root: PathBuf = fixture("namespace-table-none");
    task_with_table(root.as_path(), "11", None);
    task_with_table(root.as_path(), "12", None);
    let read: NamespaceTableRead = read_one_namespace(&MountNamespaceTasks { namespace: String::from("mnt:[9]"), tasks: vec![root.join("11"), root.join("12")] });
    assert_eq!(read.namespace, String::from("mnt:[9]"));
    assert_eq!(read.attempts.len(), 2);
    assert!(read.attempts.iter().all(|attempt| return matches!(attempt, TaskTableAttempt::Ended { .. })));
    assert!(read.has_no_live_member());
}
