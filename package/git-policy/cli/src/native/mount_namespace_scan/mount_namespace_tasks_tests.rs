//! What:
//!  Controls of the thread grouping:
//!  two threads of one namespace are one group,
//!  the group's threads are in tid order,
//!  a thread that ended before its link was read is left out,
//!  a thread whose link answers another error is kept and named,
//!  a task's own thread is read when its leader's link is gone,
//!  and a `/proc` root that cannot be listed is a refusal naming it.
//! Why:
//!  The scan's cost follows the namespaces,
//!  so a grouping that made one group per thread would read one table per
//!  thread,
//!  and a dropped link is exactly how a scan went blind while answering
//!  clear,
//!  which is the integration pass's finding M1.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(threadsByMountNamespace({ procRoot: fixture }).namespaces).toEqual([{ namespace: 'mnt:[7]', threads: [...] }]);
//! ```

use super::{
    ListingRefusal, ListingRequest, MountNamespaceTasks, NamespaceListing, NamespaceRoot, RootAt, RootIdentity, TaskLink, ThreadAt, UnreadableLink, is_a_thread_that_ended, task_link,
    threads_by_mount_namespace,
};
use crate::testing::test_support::fixture;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// One fixture thread of one task,
/// with the namespace its `ns/mnt` link names when one is given,
/// and with the mode its `ns` folder is given when one is.
fn thread(root: &Path, pid: &str, tid: &str, namespace: Option<&str>, ns_mode: Option<u32>) {
    let folder: PathBuf = root.join(pid).join("task").join(tid);
    std::fs::create_dir_all(folder.join("ns")).expect("the fixture thread is made");
    std::fs::create_dir_all(folder.join("root")).expect("the fixture root is made");
    if let Some(target) = namespace {
        std::os::unix::fs::symlink(target, folder.join("ns/mnt")).expect("the namespace link is made");
    }
    if let Some(mode) = ns_mode {
        std::fs::set_permissions(folder.join("ns"), std::fs::Permissions::from_mode(mode)).expect("the fixture folder holds its mode");
    }
}

/// One fixture `/proc` root with the given threads,
/// read with no reader's root,
/// which is what a tree of files can describe.
fn listing_of(root: &Path) -> NamespaceListing {
    return threads_by_mount_namespace(&ListingRequest { proc_root: root, readers_root: None }).expect("the fixture lists");
}

/// Two threads of one namespace are one group,
/// and its threads are in the kernel's tid order,
/// whatever order the listing answered in.
#[test]
fn two_threads_of_one_namespace_are_one_group_in_tid_order() {
    let root: PathBuf = fixture("mount-namespace-threads-one-group");
    thread(root.as_path(), "12", "12", Some("mnt:[7]"), None);
    thread(root.as_path(), "11", "11", Some("mnt:[7]"), None);
    thread(root.as_path(), "21", "21", Some("mnt:[9]"), None);
    let listing: NamespaceListing = listing_of(root.as_path());
    assert_eq!(listing.namespaces.len(), 2);
    assert_eq!(listing.namespaces[0].namespace, String::from("mnt:[7]"));
    assert_eq!(listing.namespaces[0].threads.len(), 2);
    assert_eq!(listing.namespaces[0].threads[0].thread, root.join("11/task/11"));
    assert_eq!(listing.namespaces[0].threads[1].thread, root.join("12/task/12"));
    assert_eq!(listing.namespaces[1].namespace, String::from("mnt:[9]"));
}

/// A thread of one task in a namespace of its own is its own group,
/// which is what a thread that unshared a mount namespace looks like.
#[test]
fn a_thread_with_a_namespace_of_its_own_is_its_own_group() {
    let root: PathBuf = fixture("mount-namespace-threads-own");
    thread(root.as_path(), "11", "11", Some("mnt:[7]"), None);
    thread(root.as_path(), "11", "12", Some("mnt:[8]"), None);
    let listing: NamespaceListing = listing_of(root.as_path());
    assert_eq!(listing.namespaces.len(), 2);
    for group in &listing.namespaces {
        assert_eq!(group.threads.len(), 1);
    }
}

/// A thread that ended before its link was read is left out,
/// and a directory that is no task is left out with it.
#[test]
fn a_thread_without_a_namespace_link_is_left_out() {
    let root: PathBuf = fixture("mount-namespace-threads-no-link");
    thread(root.as_path(), "11", "11", Some("mnt:[7]"), None);
    thread(root.as_path(), "31", "31", None, None);
    std::fs::create_dir_all(root.join("self")).expect("the non-task directory is made");
    std::fs::create_dir_all(root.join("acpi")).expect("the second non-task directory is made");
    let listing: NamespaceListing = listing_of(root.as_path());
    assert_eq!(listing.namespaces.len(), 1);
    assert_eq!(listing.namespaces[0].threads.len(), 1);
    assert!(listing.unreadable.is_empty());
    assert_eq!(task_link(root.join("31/task/31").as_path()), TaskLink::Ended);
}

/// A link that answers an error other than the thread's ending is kept and
/// named,
/// so a scan that may not read it reports it instead of dropping it.
#[test]
fn a_link_that_cannot_be_read_is_kept_and_named() {
    let root: PathBuf = fixture("mount-namespace-threads-unreadable");
    thread(root.as_path(), "11", "11", Some("mnt:[7]"), None);
    thread(root.as_path(), "41", "41", Some("mnt:[9]"), Some(0o000));
    let listing: NamespaceListing = listing_of(root.as_path());
    let unreadable: Vec<&UnreadableLink> = listing.unreadable.iter().collect();
    assert_eq!(unreadable.len(), 1, "{unreadable:?}");
    assert_eq!(unreadable[0].thread, root.join("41/task/41"));
    assert!(!unreadable[0].reason.is_empty());
    std::fs::set_permissions(root.join("41/task/41/ns"), std::fs::Permissions::from_mode(0o755)).expect("the fixture folder is readable again");
}

/// A process whose leader's link is gone is still read through the thread that
/// lives,
/// which is what a task whose main thread ended looks like.
#[test]
fn a_process_is_read_through_the_thread_that_lives() {
    let root: PathBuf = fixture("mount-namespace-threads-leader-gone");
    thread(root.as_path(), "51", "52", Some("mnt:[9]"), None);
    let listing: NamespaceListing = listing_of(root.as_path());
    assert_eq!(listing.namespaces.len(), 1);
    assert_eq!(listing.namespaces[0].namespace, String::from("mnt:[9]"));
    assert_eq!(listing.namespaces[0].threads[0].thread, root.join("51/task/52"));
}

/// A name that is no pid is skipped,
/// whether it holds no digits or a number too large for one,
/// and its folder holds a thread the kernel would name as `/proc/self` does:
/// a number.
#[test]
fn a_name_that_is_no_pid_is_skipped() {
    let root: PathBuf = fixture("mount-namespace-threads-no-pid");
    thread(root.as_path(), "11", "11", Some("mnt:[7]"), None);
    thread(root.as_path(), "self", "11", Some("mnt:[9]"), None);
    thread(root.as_path(), "4294967296", "4294967296", Some("mnt:[9]"), None);
    // A tree whose task folder holds a name that is no number is no tree the kernel makes, and it is read as no
    // thread rather than as the thread numbered one.
    thread(root.as_path(), "13", "self", Some("mnt:[9]"), None);
    let listing: NamespaceListing = listing_of(root.as_path());
    assert_eq!(listing.namespaces.len(), 1, "{:?}", listing.namespaces);
    assert_eq!(listing.namespaces[0].namespace, String::from("mnt:[7]"));
    assert_eq!(listing.namespaces[0].threads.len(), 1);
}

/// A `/proc` root that cannot be listed is a refusal,
/// and its sentence names the directory and the way forward.
#[test]
fn an_unlistable_proc_root_is_a_refusal() {
    let root: PathBuf = fixture("mount-namespace-threads-missing");
    let missing: PathBuf = root.join("no-such-proc");
    let refusal: ListingRefusal = threads_by_mount_namespace(&ListingRequest { proc_root: missing.as_path(), readers_root: None }).expect_err("a missing root cannot be listed");
    assert_eq!(refusal.path, missing);
    let said: String = refusal.message();
    assert!(said.contains(missing.display().to_string().as_str()), "{said}");
    assert!(said.contains("has not shown there is none"), "{said}");
}

/// The group's own root is the reader's when a thread is at it,
/// the one every thread shares when none is,
/// and unknown when the threads disagree and none is at the reader's.
#[test]
fn a_namespace_root_is_the_readers_the_shared_one_or_unknown() {
    let root: PathBuf = fixture("mount-namespace-threads-roots");
    let first: RootIdentity = RootIdentity::of(root.as_path()).expect("the fixture's own identity reads");
    let other: RootIdentity = RootIdentity { device: first.device, inode: first.inode + 1 };
    let readers: MountNamespaceTasks = MountNamespaceTasks {
        namespace: String::from("mnt:[7]"),
        threads: vec![
            ThreadAt { thread: root.join("11/task/11"), root: RootAt::Known(first) },
            ThreadAt { thread: root.join("11/task/12"), root: RootAt::Known(other) },
        ],
    };
    assert_eq!(readers.namespace_root(Some(first)), NamespaceRoot::Readers);
    let shared: MountNamespaceTasks = MountNamespaceTasks {
        namespace: String::from("mnt:[8]"),
        threads: vec![ThreadAt { thread: root.join("21/task/21"), root: RootAt::Known(other) }, ThreadAt { thread: root.join("21/task/22"), root: RootAt::Known(other) }],
    };
    assert_eq!(shared.namespace_root(Some(first)), NamespaceRoot::Shared(other));
    assert_eq!(shared.namespace_root(None), NamespaceRoot::Shared(other));
    assert_eq!(readers.namespace_root(Some(other)), NamespaceRoot::Readers);
}

/// A thread that is going answers `ENOENT`, `ESRCH` or `EINVAL`,
/// and every other error is a refusal this reader reports.
#[test]
fn a_thread_that_ends_is_what_these_three_errors_mean() {
    for errno in [libc::ENOENT, libc::ESRCH, libc::EINVAL] {
        let ended: std::io::Error = std::io::Error::from_raw_os_error(errno);
        assert!(is_a_thread_that_ended(&ended), "errno {errno} is a thread that ended");
    }
    for errno in [libc::EACCES, libc::EPERM, libc::ENOTDIR, libc::EIO] {
        let refused: std::io::Error = std::io::Error::from_raw_os_error(errno);
        assert!(!is_a_thread_that_ended(&refused), "errno {errno} is a refusal to report");
    }
}
