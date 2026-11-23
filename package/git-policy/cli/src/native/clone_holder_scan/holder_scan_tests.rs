//! What:
//!  The holder scan's own controls:
//!  a holder in another namespace is found,
//!  the reader's own namespace is not a foreign holder,
//!  a mount of the clone at the slot in another namespace is a holder even
//!  though a removal's intent stands at `unmount`,
//!  a table it cannot read is incomplete and never empty,
//!  a namespace link it cannot read is an incomplete read,
//!  a task that ended between the listing and the read is named,
//!  a dataset whose name has this one as a prefix is not a match,
//!  a bind of the clone elsewhere is a holder,
//!  a namespace whose tasks disagree about their root is incomplete and one at
//!  its root is read,
//!  a `/proc` root that cannot be listed is one incomplete read,
//!  the three bounds each stop the scan and are reported,
//!  a long reason is cut,
//!  and clear is false while a holder or an incomplete read stands.
//! Why:
//!  Each control is one sentence of the design's scan
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces")
//!  and of the project's decision of 2026-11-23
//!  (`doc/decision/cli-git-zfs-holder-scan-names-every-holder-it-can-read.md`),
//!  and each is driven from fixture text,
//!  so nothing here reads another account's processes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(scanOf(procRoot).holders.map((h) => h.namespace)).toEqual(['mnt:[9]']);
//! ```

use super::{HolderScan, HolderScanRequest, MAXIMUM_DETAIL_LINES, MAXIMUM_MOUNTS_PER_NAMESPACE, MAXIMUM_MOUNT_NAMESPACES, bounded_reason, scan_holders};
use crate::clone_holder_scan::held_mounts::HeldMount;
use crate::clone_holder_scan::incomplete_reads::{DETAILS_PAST_THE_BOUND_WORD, IncompleteRead, NAMESPACE_LINK_UNREADABLE_WORD, NO_TASK_AT_THE_NAMESPACE_ROOT_WORD};
use crate::clone_holder_scan::namespace_holder::NamespaceHolder;
use crate::mount_namespace_scan::mount_namespace_tasks::{RootIdentity, THREADS_FOLDER};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::path::{Path, PathBuf};

/// The dataset every control scans for.
const SOURCE: &str = "zpcachyos/nosnap/clones/one";

/// The path a managed attach of the clone stands at.
const SLOT: &str = "/home/user/Monochromatic/.claude/worktrees/one";

/// The nonce every fixture result answers under.
const NONCE: &str = "00112233445566778899aabbccddeeff";

/// The table a thread chrooted away from the clone renders,
/// which holds neither the clone nor an error.
const NO_MOUNT: &str = "30 20 0:20 / / ro - btrfs zpcachyos/ROOT/cos ro\n";

/// The time every fixture result carries.
const TIME: &str = "2026-11-22T16:00:00-05:00";

/// The folder a fixture task's root names,
/// which every fixture thread shares unless a control says otherwise.
const READERS_ROOT: &str = "reader-root";

/// One table row for a mount of the dataset at `mount_point`,
/// showing `root` within it.
fn mount_line(mount_id: u32, root: &str, mount_point: &str) -> String {
    return format!("{mount_id} 30 0:32 {root} {mount_point} ro,nosuid,nodev - zfs {SOURCE} ro\n");
}

/// The fixture `/proc` root and the root every thread of it starts in.
fn fixture_root(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let proc: PathBuf = root.join("proc");
    let readers: PathBuf = proc.join(READERS_ROOT);
    std::fs::create_dir_all(readers.as_path()).expect("the fixture root is made");
    return (proc, readers);
}

/// One fixture thread of one task,
/// in one namespace,
/// with the mount table it answers when one is given,
/// and with the root it renders that table from.
fn thread(proc: &Path, pid: &str, tid: &str, namespace: &str, table: Option<&str>, root_folder: &Path) {
    let folder: PathBuf = proc.join(pid).join(THREADS_FOLDER).join(tid);
    std::fs::create_dir_all(folder.join("ns")).expect("the fixture thread is made");
    std::fs::create_dir_all(folder.as_path()).expect("the fixture thread folder is made");
    std::os::unix::fs::symlink(namespace, folder.join("ns/mnt")).expect("the namespace link is made");
    std::os::unix::fs::symlink(root_folder, folder.join("root")).expect("the root link is made");
    if let Some(text) = table {
        std::fs::write(folder.join("mountinfo"), text).expect("the fixture table is written");
    }
}

/// One thread in one namespace,
/// at the fixture's shared root.
fn plain_thread(proc: &Path, pid: &str, namespace: &str, table: Option<&str>, readers: &Path) {
    thread(proc, pid, pid, namespace, table, readers);
}

/// The mounted thread a namespace's own mount table names:
/// a task in the shared root with a table of its own.
fn mounted(proc: &Path, pid: &str, namespace: &str, table: &str, readers: &Path) {
    plain_thread(proc, pid, namespace, Some(table), readers);
}

/// The reader's root of one fixture.
fn readers_root_of(readers: &Path) -> RootIdentity {
    return RootIdentity::of(readers).expect("the fixture's root reads");
}

/// One scan of a fixture `/proc` root,
/// through the fixture's own reader root unless a control gives none.
fn scan_of(proc: &Path, own_namespace: Option<&str>, readers: Option<&Path>) -> HolderScan {
    let nonce: RequestNonce = RequestNonce::parse(NONCE).expect("the fixture nonce is one");
    let request: HolderScanRequest<'_> = HolderScanRequest {
        proc_root: proc,
        source: SOURCE,
        own_namespace,
        readers_root: readers.map(|folder| return readers_root_of(folder)),
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

/// The causes of one kind a scan reports.
fn causes_of<'a>(scan: &'a HolderScan, kind: &str) -> Vec<&'a IncompleteRead> {
    return scan.incomplete.iter().filter(|read| return read.kind() == kind).collect();
}

/// A holder in another namespace is found,
/// with the mount it holds.
#[test]
fn a_holder_in_another_namespace_is_found() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-holder");
    mounted(proc.as_path(), "11", "mnt:[7]", mount_line(41, "/", SLOT).as_str(), readers.as_path());
    mounted(proc.as_path(), "21", "mnt:[9]", mount_line(42, "/", "/srv/holder").as_str(), readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert_eq!(holder_namespaces(&scan), vec!["mnt:[9]"]);
    assert_eq!(scan.holder_mounts(), 1);
    assert_eq!(scan.holders[0].mounts[0].mount_point, PathBuf::from("/srv/holder"));
    assert_eq!(scan.own_mounts.len(), 1);
    assert!(!scan.is_clear());
    assert_eq!(scan.namespaces_scanned, 2);
    assert!(scan.incomplete.is_empty(), "{:?}", scan.incomplete);
}

/// The reader's own namespace is reported as its own mounts and never as a
/// foreign holder.
#[test]
fn the_readers_own_namespace_is_not_a_foreign_holder() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-own");
    mounted(proc.as_path(), "11", "mnt:[7]", mount_line(41, "/", SLOT).as_str(), readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert!(scan.holders.is_empty());
    assert_eq!(scan.own_mounts, vec![HeldMount { mount_id: 41, root: PathBuf::from("/"), mount_point: PathBuf::from(SLOT) }]);
    assert_eq!(scan.own_namespace.as_deref(), Some("mnt:[7]"));
    assert!(!scan.is_clear(), "the clone is still mounted in the reader's own namespace");
}

/// A mount of the clone's own root at the slot in another namespace is a
/// holder,
/// which is what a removal scans for while its intent stands at `unmount`.
#[test]
fn a_mount_at_the_slot_in_another_namespace_is_a_holder() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-slot-other");
    mounted(proc.as_path(), "21", "mnt:[9]", mount_line(42, "/", SLOT).as_str(), readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert_eq!(holder_namespaces(&scan), vec!["mnt:[9]"]);
    assert_eq!(scan.holders[0].mounts[0].root, PathBuf::from("/"));
    assert_eq!(scan.holders[0].mounts[0].mount_point, PathBuf::from(SLOT));
    assert!(!scan.is_clear());
}

/// A table this process may not open is reported as incomplete and never as
/// an empty namespace.
#[test]
fn a_table_it_cannot_read_is_incomplete_and_never_empty() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-unreadable");
    let task: PathBuf = proc.join("21").join(THREADS_FOLDER).join("21");
    std::fs::create_dir_all(task.join("ns")).expect("the fixture thread is made");
    std::os::unix::fs::symlink("mnt:[9]", task.join("ns/mnt")).expect("the namespace link is made");
    std::os::unix::fs::symlink(readers.as_path(), task.join("root")).expect("the root link is made");
    std::fs::create_dir_all(task.join("mountinfo")).expect("a directory stands where the table belongs");
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert!(scan.holders.is_empty());
    assert_eq!(scan.incomplete_reads(), 1);
    assert!(matches!(scan.incomplete[0], IncompleteRead::TableUnreadable { .. }), "{:?}", scan.incomplete[0]);
    assert_eq!(scan.incomplete[0].namespace(), Some("mnt:[9]"));
    assert!(!scan.is_clear(), "a namespace nobody read is not a clear one");
}

/// A namespace link the scan may not read is an incomplete read naming the
/// thread,
/// so a handler without the capability to read it is never clear.
#[test]
fn a_link_it_cannot_read_is_an_incomplete_read() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-link-unreadable");
    let thread_folder: PathBuf = proc.join("31").join(THREADS_FOLDER).join("31");
    std::fs::create_dir_all(thread_folder.as_path()).expect("the fixture task is made");
    std::fs::write(thread_folder.join("ns"), "").expect("a file stands where the namespace folder belongs");
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    let links: Vec<&IncompleteRead> = causes_of(&scan, NAMESPACE_LINK_UNREADABLE_WORD);
    assert_eq!(links.len(), 1, "{:?}", scan.incomplete);
    assert_eq!(links[0].task(), Some(thread_folder.as_path()));
    assert!(!scan.is_clear());
    assert_eq!(scan.namespaces_scanned, 0, "a thread whose namespace is unknown is no namespace read");
}

/// A task that exits between the listing and the read is named as an
/// incomplete read,
/// and its namespace is still read through its next thread.
/// A namespace whose every thread ended is reported as having no live member.
#[test]
fn a_task_that_ended_is_named_and_its_namespace_is_still_read() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-ended");
    plain_thread(proc.as_path(), "21", "mnt:[9]", None, readers.as_path());
    mounted(proc.as_path(), "22", "mnt:[9]", mount_line(42, "/", "/srv/holder").as_str(), readers.as_path());
    plain_thread(proc.as_path(), "51", "mnt:[5]", None, readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert_eq!(holder_namespaces(&scan), vec!["mnt:[9]"]);
    let ended: Vec<&IncompleteRead> = causes_of(&scan, crate::clone_holder_scan::incomplete_reads::TASK_ENDED_WORD);
    assert_eq!(ended.len(), 1, "{:?}", scan.incomplete);
    assert_eq!(ended[0].task(), Some(proc.join("21/task/21").as_path()));
    let unknown: Vec<&IncompleteRead> = causes_of(&scan, crate::clone_holder_scan::incomplete_reads::NO_LIVE_MEMBER_WORD);
    assert_eq!(unknown.len(), 1, "{:?}", scan.incomplete);
    assert_eq!(unknown[0].namespace(), Some("mnt:[5]"));
    assert!(!scan.is_clear());
}

/// A mount of another dataset whose name has this one as a prefix is not a
/// holder of this one.
#[test]
fn a_prefix_of_the_name_is_another_dataset() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-prefix");
    let table: String = format!("42 30 0:32 / /srv/holder ro - zfs {SOURCE}-extra ro\n");
    mounted(proc.as_path(), "21", "mnt:[9]", table.as_str(), readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert!(scan.holders.is_empty());
    assert!(scan.incomplete.is_empty());
    assert!(scan.is_clear());
}

/// A bind of the clone elsewhere is a holder,
/// whatever root within the clone it shows.
#[test]
fn a_bind_elsewhere_is_a_holder() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-bind");
    mounted(proc.as_path(), "21", "mnt:[9]", mount_line(42, "/sub", "/srv/elsewhere").as_str(), readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert_eq!(holder_namespaces(&scan), vec!["mnt:[9]"]);
    assert_eq!(scan.holders[0].mounts[0].root, PathBuf::from("/sub"));
    assert!(!scan.holders[0].mounts[0].is_the_dataset_root());
}

/// A namespace whose tasks disagree about their root,
/// none of them at the reader's,
/// is reported as one whose own root this reader cannot name,
/// and the table it read is still scanned.
#[test]
fn a_namespace_whose_tasks_disagree_about_their_root_is_incomplete() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-root-disagree");
    let first_root: PathBuf = proc.join("first-root");
    let second_root: PathBuf = proc.join("second-root");
    std::fs::create_dir_all(first_root.as_path()).expect("the first root is made");
    std::fs::create_dir_all(second_root.as_path()).expect("the second root is made");
    thread(proc.as_path(), "21", "21", "mnt:[9]", Some(mount_line(42, "/", "/srv/holder").as_str()), first_root.as_path());
    thread(proc.as_path(), "22", "22", "mnt:[9]", Some(NO_MOUNT), second_root.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    let unknown: Vec<&IncompleteRead> = causes_of(&scan, NO_TASK_AT_THE_NAMESPACE_ROOT_WORD);
    assert_eq!(unknown.len(), 1, "{:?}", scan.incomplete);
    assert_eq!(unknown[0].namespace(), Some("mnt:[9]"));
    assert!(!scan.is_clear());
}

/// A namespace whose tasks disagree and one of them is at the reader's root is
/// read there,
/// so the mount a chrooted task omits is still found.
#[test]
fn a_namespace_with_a_thread_at_the_readers_root_is_read_there() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-root-readers");
    let chroot_root: PathBuf = proc.join("chroot-root");
    std::fs::create_dir_all(chroot_root.as_path()).expect("the chroot root is made");
    thread(proc.as_path(), "21", "21", "mnt:[9]", Some(NO_MOUNT), chroot_root.as_path());
    thread(proc.as_path(), "21", "22", "mnt:[9]", Some(mount_line(42, "/", "/srv/holder").as_str()), readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert_eq!(holder_namespaces(&scan), vec!["mnt:[9]"]);
    assert!(scan.incomplete.is_empty(), "{:?}", scan.incomplete);
    assert_eq!(scan.holders[0].task, proc.join("21/task/22"));
}

/// A `/proc` root that cannot be listed is one incomplete read,
/// and no holder is invented from it.
#[test]
fn an_unlistable_proc_root_is_one_incomplete_read() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-missing");
    let missing: PathBuf = proc.join("no-such-proc");
    let scan: HolderScan = scan_of(missing.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert!(scan.holders.is_empty());
    assert!(scan.own_mounts.is_empty());
    assert_eq!(scan.incomplete_reads(), 1);
    assert!(matches!(scan.incomplete[0], IncompleteRead::Listing { .. }), "{:?}", scan.incomplete[0]);
    assert_eq!(scan.namespaces_scanned, 0);
    assert!(!scan.is_clear());
}

/// A holder's task is the thread its table came from,
/// so a person can find the process.
#[test]
fn a_holder_names_the_task_its_table_came_from() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-task");
    mounted(proc.as_path(), "21", "mnt:[9]", mount_line(42, "/", "/srv/holder").as_str(), readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    let holder: &NamespaceHolder = &scan.holders[0];
    assert_eq!(holder.task, proc.join("21/task/21"));
    assert_eq!(holder.mount_count(), 1);
}

/// A namespace that holds the clone more often than a report lists is listed
/// to the bound,
/// and the rest is one incomplete read rather than a silent omission.
#[test]
fn a_namespace_past_the_mount_bound_lists_the_bound_and_says_so() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-mount-bound");
    let mut table: String = String::new();
    for mount_id in 0..(MAXIMUM_MOUNTS_PER_NAMESPACE + 2) {
        table.push_str(bind_line(u32::try_from(41 + mount_id).expect("the fixture's mount id fits")).as_str());
    }
    mounted(proc.as_path(), "21", "mnt:[9]", table.as_str(), readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert_eq!(scan.holder_namespaces(), 1);
    assert_eq!(scan.holder_mounts(), MAXIMUM_MOUNTS_PER_NAMESPACE);
    let saying: Vec<&IncompleteRead> = scan.incomplete.iter().filter(|read| return matches!(read, IncompleteRead::MountsPastTheBound { .. })).collect();
    assert_eq!(saying.len(), 1, "{:?}", scan.incomplete);
    assert_eq!(saying[0].namespace(), Some("mnt:[9]"));
    assert!(!scan.is_clear());
}

/// One fixture namespace whose two threads answer no table,
/// which is two causes for the budget controls.
fn unreadable_namespace(proc: &Path, number: usize, readers: &Path) {
    let pid: String = format!("{}", 1000 + number);
    for tid in ["1", "2"] {
        let task: PathBuf = proc.join(pid.as_str()).join(THREADS_FOLDER).join(format!("{pid}{tid}"));
        std::fs::create_dir_all(task.join("ns")).expect("the fixture thread is made");
        std::os::unix::fs::symlink(format!("mnt:[1{number:04}]"), task.join("ns/mnt")).expect("the namespace link is made");
        std::os::unix::fs::symlink(readers, task.join("root")).expect("the root link is made");
        std::fs::create_dir_all(task.join("mountinfo")).expect("a directory stands where the table belongs");
    }
}

/// One mount line of the clone at a bind the fixture lists,
/// which the budget controls use to fill one namespace's line.
fn bind_line(mount_id: u32) -> String {
    return mount_line(mount_id, "/", format!("/srv/bind{mount_id}").as_str());
}

/// The namespace bound stops the reading,
/// is recorded as one incomplete read,
/// and no namespace after it is named.
#[test]
fn the_namespace_bound_stops_the_reading_and_is_recorded() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-namespace-bound");
    for number in 0..=MAXIMUM_MOUNT_NAMESPACES {
        plain_thread(proc.as_path(), format!("{}", 2000 + number).as_str(), format!("mnt:[2{number:04}]").as_str(), None, readers.as_path());
    }
    mounted(proc.as_path(), "9000", "mnt:[9999]", mount_line(42, "/", "/srv/holder").as_str(), readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    let bound: Vec<&IncompleteRead> = scan.incomplete.iter().filter(|read| return matches!(read, IncompleteRead::NamespacesPastTheBound { .. })).collect();
    assert_eq!(bound.len(), 1, "{:?}", scan.incomplete);
    assert_eq!(scan.namespaces_scanned, MAXIMUM_MOUNT_NAMESPACES);
    assert!(scan.holders.is_empty(), "a namespace past the bound is in no list: {:?}", scan.holders);
    assert!(!scan.is_clear());
}

/// The detail budget stops the causes,
/// records the marker,
/// and no holder after it is named.
#[test]
fn the_detail_budget_stops_the_causes_and_is_recorded() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-detail-budget");
    for number in 0..(MAXIMUM_DETAIL_LINES / 2 + 8) {
        unreadable_namespace(proc.as_path(), number, readers.as_path());
    }
    mounted(proc.as_path(), "9000", "mnt:[9999]", mount_line(42, "/", "/srv/holder").as_str(), readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert_eq!(scan.incomplete_reads(), MAXIMUM_DETAIL_LINES + 1, "the budget's own findings and its marker");
    assert_eq!(scan.incomplete.last().map(|read| return read.kind()), Some(DETAILS_PAST_THE_BOUND_WORD));
    assert!(scan.holders.is_empty(), "a holder after the budget is in no list: {:?}", scan.holders);
    assert!(!scan.is_clear());
}

/// The detail budget stops a namespace's mount listing at the bound,
/// and the marker says so.
#[test]
fn the_detail_budget_stops_a_namespace_s_mounts_at_the_bound() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-detail-mounts");
    for number in 0..(MAXIMUM_DETAIL_LINES / 2 - 1) {
        unreadable_namespace(proc.as_path(), number, readers.as_path());
    }
    let mut table: String = String::new();
    for mount_id in 0..MAXIMUM_MOUNTS_PER_NAMESPACE {
        table.push_str(bind_line(u32::try_from(41 + mount_id).expect("the fixture's mount id fits")).as_str());
    }
    mounted(proc.as_path(), "9000", "mnt:[9999]", table.as_str(), readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert_eq!(scan.holder_mounts(), 2, "the last two findings of the budget are the first two mounts");
    assert_eq!(scan.incomplete.last().map(|read| return read.kind()), Some(DETAILS_PAST_THE_BOUND_WORD));
    assert!(!scan.is_clear());
}

/// A reason longer than the bound is cut,
/// and the cut says so.
#[test]
fn a_long_reason_is_cut() {
    let long: String = "x".repeat(super::MAXIMUM_REASON_BYTES * 2);
    let cut: String = bounded_reason(long.as_str());
    assert!(cut.len() <= super::MAXIMUM_REASON_BYTES + 64, "{}", cut.len());
    assert!(cut.ends_with("(the rest of this reason was cut)"), "{cut}");
    assert_eq!(bounded_reason("short"), String::from("short"));
}

/// A result with holders and no own mount and no incomplete read is not
/// clear.
#[test]
fn clear_is_false_while_a_holder_stands() {
    let (proc, readers): (PathBuf, PathBuf) = fixture_root("holder-scan-clear-holders");
    mounted(proc.as_path(), "21", "mnt:[9]", mount_line(42, "/", "/srv/holder").as_str(), readers.as_path());
    let scan: HolderScan = scan_of(proc.as_path(), Some("mnt:[7]"), Some(readers.as_path()));
    assert!(scan.own_mounts.is_empty());
    assert!(scan.incomplete.is_empty());
    assert_eq!(scan.holder_namespaces(), 1);
    assert!(!scan.is_clear());
}
