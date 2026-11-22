//! What:
//!  The holder scan's own controls:
//!  a holder in another namespace is found,
//!  the reader's own namespace is not a foreign holder,
//!  a table it cannot read is incomplete and never empty,
//!  a task that ended between the listing and the read is named and its
//!  namespace is still read,
//!  a namespace with no live member is unknown,
//!  a dataset whose name has this one as a prefix is not a match,
//!  a bind of the clone elsewhere is a holder,
//!  a managed attach is set aside only under a standing `unmount` intent,
//!  and a `/proc` root that cannot be listed is one incomplete read.
//! Why:
//!  Each control is one sentence of the design's scan
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces"),
//!  and each is driven from fixture text,
//!  so nothing here reads another account's processes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(scanOf(procRoot).holders.map((h) => h.namespace)).toEqual(['mnt:[9]']);
//! ```

use super::{HolderScan, HolderScanRequest, scan_holders};
use super::super::incomplete_reads::IncompleteRead;
use super::super::namespace_holder::NamespaceHolder;
use super::super::standing_unmount::StandingUnmount;
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_intent::{MountIntent, MountOperation};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::path::{Path, PathBuf};

/// The dataset every control scans for.
const SOURCE: &str = "zpcachyos/nosnap/clones/one";

/// The path the fixture's managed attach stands at.
const SLOT: &str = "/home/user/Monochromatic/.claude/worktrees/one";

/// The nonce every fixture result answers under.
const NONCE: &str = "00112233445566778899aabbccddeeff";

/// The time every fixture result carries.
const TIME: &str = "2026-11-22T16:00:00-05:00";

/// One table row for a mount of the dataset at `mount_point`,
/// showing `root` within it.
fn mount_line(mount_id: u32, root: &str, mount_point: &str) -> String {
    return format!("{mount_id} 30 0:32 {root} {mount_point} ro,nosuid,nodev - zfs {SOURCE} ro\n");
}

/// One fixture task in one namespace,
/// with the mount table it answers when one is given.
fn task(proc_root: &Path, pid: &str, namespace: &str, table: Option<&str>) {
    let folder: PathBuf = proc_root.join(pid);
    std::fs::create_dir_all(folder.join("ns")).expect("the fixture task is made");
    std::os::unix::fs::symlink(namespace, folder.join("ns/mnt")).expect("the namespace link is made");
    if let Some(text) = table {
        std::fs::write(folder.join("mountinfo"), text).expect("the fixture table is written");
    }
}

/// One scan of a fixture `/proc` root.
fn scan_of(proc_root: &Path, own_namespace: Option<&str>, standing: Option<&StandingUnmount>) -> HolderScan {
    let nonce: RequestNonce = RequestNonce::parse(NONCE).expect("the fixture nonce is one");
    let request: HolderScanRequest<'_> = HolderScanRequest {
        proc_root,
        source: SOURCE,
        own_namespace,
        standing_unmount: standing,
        dataset: DatasetGuid::parse("42").expect("the fixture GUID is canonical"),
        nonce: &nonce,
        time: String::from(TIME),
    };
    return scan_holders(&request);
}

/// The namespaces a scan reports as holders,
/// in the order it reports them.
fn holder_namespaces(scan: &HolderScan) -> Vec<&str> {
    return scan.holders.iter().map(|holder| return holder.namespace()).collect();
}

/// A holder in another namespace is found,
/// with the mount it holds.
#[test]
fn a_holder_in_another_namespace_is_found() {
    let root: PathBuf = fixture("holder-scan-holder");
    task(root.as_path(), "11", "mnt:[7]", Some(mount_line(41, "/", SLOT).as_str()));
    task(root.as_path(), "21", "mnt:[9]", Some(mount_line(42, "/", "/srv/holder").as_str()));
    let scan: HolderScan = scan_of(root.as_path(), Some("mnt:[7]"), None);
    assert_eq!(holder_namespaces(&scan), vec!["mnt:[9]"]);
    assert_eq!(scan.holder_mounts(), 1);
    assert_eq!(scan.holders[0].mounts[0].mount_point, PathBuf::from("/srv/holder"));
    assert_eq!(scan.own_mounts.len(), 1);
    assert!(!scan.is_clear());
    assert_eq!(scan.namespaces_scanned, 2);
}

/// The reader's own namespace is reported as its own mounts and never as a
/// foreign holder.
#[test]
fn the_readers_own_namespace_is_not_a_foreign_holder() {
    let root: PathBuf = fixture("holder-scan-own");
    task(root.as_path(), "11", "mnt:[7]", Some(mount_line(41, "/", SLOT).as_str()));
    let scan: HolderScan = scan_of(root.as_path(), Some("mnt:[7]"), None);
    assert!(scan.holders.is_empty());
    assert_eq!(scan.holder_namespaces(), 0);
    assert_eq!(scan.own_mounts, vec![super::super::held_mounts::HeldMount { mount_id: 41, root: PathBuf::from("/"), mount_point: PathBuf::from(SLOT) }]);
    assert_eq!(scan.own_namespace.as_deref(), Some("mnt:[7]"));
    assert!(!scan.is_clear(), "the clone is still mounted in the reader's own namespace");
}

/// A table this process may not open is reported as incomplete and never as
/// an empty namespace.
#[test]
fn a_table_it_cannot_read_is_incomplete_and_never_empty() {
    let root: PathBuf = fixture("holder-scan-unreadable");
    let task_folder: PathBuf = root.join("21");
    std::fs::create_dir_all(task_folder.join("ns")).expect("the fixture task is made");
    std::os::unix::fs::symlink("mnt:[9]", task_folder.join("ns/mnt")).expect("the namespace link is made");
    std::fs::create_dir_all(task_folder.join("mountinfo")).expect("a directory stands where the table belongs");
    let scan: HolderScan = scan_of(root.as_path(), Some("mnt:[7]"), None);
    assert!(scan.holders.is_empty());
    assert_eq!(scan.incomplete_reads(), 1);
    assert!(matches!(scan.incomplete[0], IncompleteRead::TableUnreadable { .. }), "{:?}", scan.incomplete[0]);
    assert_eq!(scan.incomplete[0].namespace(), Some("mnt:[9]"));
    assert!(!scan.is_clear(), "a namespace nobody read is not a clear one");
}

/// A task that exits between the listing and the read is named as an
/// incomplete read,
/// and its namespace is still read through its next task.
/// A namespace whose every task ended is reported as having no live member.
#[test]
fn a_task_that_ended_is_named_and_its_namespace_is_still_read() {
    let root: PathBuf = fixture("holder-scan-ended");
    task(root.as_path(), "21", "mnt:[9]", None);
    task(root.as_path(), "22", "mnt:[9]", Some(mount_line(42, "/", "/srv/holder").as_str()));
    task(root.as_path(), "51", "mnt:[5]", None);
    let scan: HolderScan = scan_of(root.as_path(), Some("mnt:[7]"), None);
    assert_eq!(holder_namespaces(&scan), vec!["mnt:[9]"]);
    let ended: Vec<&IncompleteRead> = scan.incomplete.iter().filter(|read| return matches!(read, IncompleteRead::TaskEnded { .. })).collect();
    assert_eq!(ended.len(), 1, "{:?}", scan.incomplete);
    assert_eq!(ended[0].task(), Some(root.join("21").as_path()));
    let unknown: Vec<&IncompleteRead> = scan.incomplete.iter().filter(|read| return matches!(read, IncompleteRead::NoLiveMember { .. })).collect();
    assert_eq!(unknown.len(), 1, "{:?}", scan.incomplete);
    assert_eq!(unknown[0].namespace(), Some("mnt:[5]"));
    assert!(!scan.is_clear());
}

/// A mount of another dataset whose name has this one as a prefix is not a
/// holder of this one.
#[test]
fn a_prefix_of_the_name_is_another_dataset() {
    let root: PathBuf = fixture("holder-scan-prefix");
    task(root.as_path(), "21", "mnt:[9]", Some(format!("42 30 0:32 / /srv/holder ro - zfs {SOURCE}-extra ro\n").as_str()));
    let scan: HolderScan = scan_of(root.as_path(), Some("mnt:[7]"), None);
    assert!(scan.holders.is_empty());
    assert!(scan.incomplete.is_empty());
    assert!(scan.is_clear());
}

/// A bind of the clone elsewhere is a holder,
/// whatever root within the clone it shows.
#[test]
fn a_bind_elsewhere_is_a_holder() {
    let root: PathBuf = fixture("holder-scan-bind");
    task(root.as_path(), "21", "mnt:[9]", Some(mount_line(42, "/sub", "/srv/elsewhere").as_str()));
    let scan: HolderScan = scan_of(root.as_path(), Some("mnt:[7]"), None);
    assert_eq!(holder_namespaces(&scan), vec!["mnt:[9]"]);
    assert_eq!(scan.holders[0].mounts[0].root, PathBuf::from("/sub"));
    assert!(!scan.holders[0].mounts[0].is_the_dataset_root());
}

/// The managed attach is set aside when the clone carries a standing `unmount`
/// intent and the caller knows its slot.
#[test]
fn the_managed_attach_is_set_aside_under_a_standing_unmount_intent() {
    let root: PathBuf = fixture("holder-scan-standing");
    task(root.as_path(), "11", "mnt:[7]", Some(mount_line(41, "/", SLOT).as_str()));
    task(root.as_path(), "21", "mnt:[9]", Some(mount_line(42, "/", SLOT).as_str()));
    let intent: MountIntent = MountIntent { operation: MountOperation::Unmount, nonce: RequestNonce::parse(NONCE).expect("the fixture nonce is one") };
    let standing: StandingUnmount = StandingUnmount::of(Some(&intent), Some(Path::new(SLOT))).expect("the fixture intent stands");
    let scan: HolderScan = scan_of(root.as_path(), Some("mnt:[7]"), Some(&standing));
    assert!(scan.holders.is_empty(), "{:?}", scan.holders);
    assert!(scan.own_mounts.is_empty(), "{:?}", scan.own_mounts);
    assert_eq!(scan.excluded.iter().map(|holder| return holder.mount_count()).sum::<usize>(), 2);
    assert_eq!(scan.excluded[0].namespace(), "mnt:[7]");
    assert_eq!(scan.excluded[1].namespace(), "mnt:[9]");
}

/// The same mounts are holders when the clone carries no standing `unmount`
/// intent.
#[test]
fn a_managed_attach_without_an_unmount_intent_is_a_holder() {
    let root: PathBuf = fixture("holder-scan-no-standing");
    task(root.as_path(), "11", "mnt:[7]", Some(mount_line(41, "/", SLOT).as_str()));
    task(root.as_path(), "21", "mnt:[9]", Some(mount_line(42, "/", SLOT).as_str()));
    let scan: HolderScan = scan_of(root.as_path(), Some("mnt:[7]"), None);
    assert_eq!(holder_namespaces(&scan), vec!["mnt:[9]"]);
    assert_eq!(scan.own_mounts.len(), 1);
    assert!(scan.excluded.is_empty());
}

/// A `/proc` root that cannot be listed is one incomplete read,
/// and no holder is invented from it.
#[test]
fn an_unlistable_proc_root_is_one_incomplete_read() {
    let root: PathBuf = fixture("holder-scan-missing");
    let missing: PathBuf = root.join("no-such-proc");
    let scan: HolderScan = scan_of(missing.as_path(), Some("mnt:[7]"), None);
    assert!(scan.holders.is_empty());
    assert!(scan.own_mounts.is_empty());
    assert_eq!(scan.incomplete_reads(), 1);
    assert!(matches!(scan.incomplete[0], IncompleteRead::Listing { .. }), "{:?}", scan.incomplete[0]);
    assert_eq!(scan.namespaces_scanned, 0);
    assert!(!scan.is_clear());
}

/// A holder's task is the live task its table came from,
/// so a person can find the process.
#[test]
fn a_holder_names_the_task_its_table_came_from() {
    let root: PathBuf = fixture("holder-scan-task");
    task(root.as_path(), "21", "mnt:[9]", Some(mount_line(42, "/", "/srv/holder").as_str()));
    let scan: HolderScan = scan_of(root.as_path(), Some("mnt:[7]"), None);
    let holder: &NamespaceHolder = &scan.holders[0];
    assert_eq!(holder.task, root.join("21"));
    assert_eq!(holder.mount_count(), 1);
}
