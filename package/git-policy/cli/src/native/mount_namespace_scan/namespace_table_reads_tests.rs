//! What:
//!  Controls of one namespace's table read:
//!  the first thread at the namespace's root answers and no later thread is
//!  read,
//!  a thread that ended is kept as an attempt and the next thread answers,
//!  a table this process may not open is kept as its own kind,
//!  a namespace whose threads all ended has no table at all,
//!  and a thread at the namespace's root is preferred over a chrooted one.
//! Why:
//!  The holder scan reports every incomplete read and never calls an unread
//!  namespace empty,
//!  which it can only do if this reader tells the two failures apart and keeps
//!  the attempts;
//!  and a chrooted thread's table omits the mounts above its root without any
//!  error,
//!  which is how a namespace holding the clone was read as empty,
//!  the integration pass's finding M3.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readOneNamespace(group, readersRoot).attempts.map((a) => a.kind)).toEqual(['ended', 'read']);
//! ```

use super::{NamespaceTableRead, TaskTableAttempt, read_one_namespace};
use crate::mount_namespace_scan::mount_namespace_tasks::{MountNamespaceTasks, RootAt, RootIdentity, ThreadAt};
use crate::testing::test_support::fixture;
use std::path::{Path, PathBuf};

/// The mount line one fixture table holds,
/// which is a mount of the dataset the controls scan for.
pub const ONE_MOUNT: &str = "41 30 0:32 / /home/user/Monochromatic ro,nosuid,nodev - zfs zpcachyos/nosnap/clones/one ro\n";

/// The mount line a chrooted thread renders when the clone is above its root,
/// which is no mount of the dataset at all.
pub const NO_MOUNT: &str = "30 20 0:20 / / ro - btrfs zpcachyos/ROOT/cos ro\n";

/// One fixture thread with a mount table,
/// or with nothing where the table belongs.
fn thread_with_table(root: &Path, pid: &str, tid: &str, table: Option<&str>) -> PathBuf {
    let folder: PathBuf = root.join(pid).join("task").join(tid);
    std::fs::create_dir_all(folder.as_path()).expect("the fixture thread is made");
    if let Some(text) = table {
        std::fs::write(folder.join("mountinfo"), text).expect("the fixture table is written");
    }
    return folder;
}

/// One group of the given threads,
/// each with the root it is at.
fn group_of(namespace: &str, threads: Vec<(PathBuf, RootAt)>) -> MountNamespaceTasks {
    return MountNamespaceTasks { namespace: String::from(namespace), threads: threads.into_iter().map(|(path, root)| return ThreadAt { thread: path, root }).collect() };
}

/// The identity of one fixture folder,
/// which stands in for a root.
fn root_at(path: &Path) -> RootIdentity {
    return RootIdentity::of(path).expect("the fixture's own identity reads");
}

/// The first thread that answers is the namespace's table,
/// and the threads after it are not read.
#[test]
fn the_first_thread_that_answers_is_the_namespace_s_table() {
    let root: PathBuf = fixture("namespace-table-first");
    let first: PathBuf = thread_with_table(root.as_path(), "11", "11", Some(ONE_MOUNT));
    let second: PathBuf = thread_with_table(root.as_path(), "11", "12", Some(ONE_MOUNT));
    let readers: RootIdentity = root_at(root.as_path());
    let group: MountNamespaceTasks = group_of("mnt:[7]", vec![(first.clone(), RootAt::Known(readers)), (second, RootAt::Known(readers))]);
    let read: NamespaceTableRead = read_one_namespace(&group, Some(readers));
    assert_eq!(read.attempts.len(), 1);
    let (thread, mounts) = read.table().expect("the first thread answered");
    assert_eq!(thread, first);
    assert_eq!(mounts.len(), 1);
    assert_eq!(mounts[0].source, String::from("zpcachyos/nosnap/clones/one"));
    assert!(!read.has_no_live_thread());
}

/// A thread that ended between the listing and the read is kept as its own
/// attempt,
/// and the namespace's next thread answers.
#[test]
fn a_thread_that_ended_is_an_attempt_and_the_next_thread_answers() {
    let root: PathBuf = fixture("namespace-table-ended");
    let gone: PathBuf = thread_with_table(root.as_path(), "11", "11", None);
    let here: PathBuf = thread_with_table(root.as_path(), "11", "12", Some(ONE_MOUNT));
    let readers: RootIdentity = root_at(root.as_path());
    let group: MountNamespaceTasks = group_of("mnt:[7]", vec![(gone.clone(), RootAt::Known(readers)), (here, RootAt::Known(readers))]);
    let read: NamespaceTableRead = read_one_namespace(&group, Some(readers));
    assert_eq!(read.attempts.len(), 2);
    assert!(matches!(read.attempts[0], TaskTableAttempt::Ended { .. }), "{:?}", read.attempts[0]);
    assert_eq!(read.attempts[0].thread(), gone);
    assert!(matches!(read.attempts[1], TaskTableAttempt::Read { .. }));
    assert!(read.table().is_some());
}

/// A table this process may not open is its own kind of attempt,
/// so a caller can tell it from a thread that ended,
/// and a namespace with no table says so.
#[test]
fn an_unreadable_table_is_not_a_thread_that_ended() {
    let root: PathBuf = fixture("namespace-table-unreadable");
    let thread: PathBuf = root.join("11").join("task").join("11");
    std::fs::create_dir_all(thread.join("mountinfo")).expect("a directory stands where the table belongs");
    let readers: RootIdentity = root_at(root.as_path());
    let read: NamespaceTableRead = read_one_namespace(&group_of("mnt:[7]", vec![(thread.clone(), RootAt::Known(readers))]), Some(readers));
    assert_eq!(read.attempts.len(), 1);
    assert!(matches!(read.attempts[0], TaskTableAttempt::Unreadable { .. }), "{:?}", read.attempts[0]);
    assert_eq!(read.attempts[0].thread(), thread);
    assert!(read.table().is_none());
    assert!(read.has_no_live_thread());
}

/// A namespace whose every thread ended has no table,
/// which is a namespace with no live thread to read through.
#[test]
fn a_namespace_whose_every_thread_ended_has_no_table() {
    let root: PathBuf = fixture("namespace-table-none");
    let first: PathBuf = thread_with_table(root.as_path(), "11", "11", None);
    let second: PathBuf = thread_with_table(root.as_path(), "11", "12", None);
    let readers: RootIdentity = root_at(root.as_path());
    let read: NamespaceTableRead = read_one_namespace(&group_of("mnt:[9]", vec![(first, RootAt::Known(readers)), (second, RootAt::Known(readers))]), Some(readers));
    assert_eq!(read.namespace, String::from("mnt:[9]"));
    assert_eq!(read.attempts.len(), 2);
    assert!(read.attempts.iter().all(|attempt| return matches!(attempt, TaskTableAttempt::Ended { .. })));
    assert!(read.has_no_live_thread());
}

/// A thread at the namespace's root is read before a chrooted one,
/// whose table omits the clone without answering any error.
#[test]
fn a_thread_at_the_namespaces_root_is_read_before_a_chrooted_one() {
    let root: PathBuf = fixture("namespace-table-root");
    let chroot_folder: PathBuf = root.join("chroot");
    std::fs::create_dir_all(chroot_folder.as_path()).expect("the chroot target is made");
    let chrooted: PathBuf = thread_with_table(root.as_path(), "11", "11", Some(NO_MOUNT));
    let plain: PathBuf = thread_with_table(root.as_path(), "11", "12", Some(ONE_MOUNT));
    let readers: RootIdentity = root_at(root.as_path());
    let chroot_root: RootIdentity = root_at(chroot_folder.as_path());
    let group: MountNamespaceTasks = group_of("mnt:[7]", vec![(chrooted.clone(), RootAt::Known(chroot_root)), (plain.clone(), RootAt::Known(readers))]);
    let at_the_readers_root: NamespaceTableRead = read_one_namespace(&group, Some(readers));
    let (thread, mounts) = at_the_readers_root.table().expect("a thread answered");
    assert_eq!(thread, plain);
    assert_eq!(mounts.len(), 1);
    let without_a_reader_root: NamespaceTableRead = read_one_namespace(&group, None);
    assert!(without_a_reader_root.table().is_some());
    assert_eq!(without_a_reader_root.attempts.len(), 1);
}
