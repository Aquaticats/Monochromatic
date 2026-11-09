//! The tests of the `/proc` scan:
//! a process it can see blocks the entry it holds,
//! and a process it cannot see blocks only when it could hold the entry at all.
//! Why:
//! The proof of "not in use" is a scan of every process and a lock,
//! and its recorded limit was that every process whose entries could not be read was skipped as another user's
//! (`doc/handover/cargo-shadow-lean-build-folders.md`).
//! A kernel thread and an ended process hold no file,
//! so the count no longer carries them;
//! a process of another user is counted once more when the path lets that user in,
//! and neither kind blocks a removal,
//! because the kernel shows a plain user no descriptors of either.

/// Files,
/// permissions and paths.
use std::{fs, os::unix::fs::PermissionsExt, path::{Path, PathBuf}, process::{Child, Command}};

/// The scan and the reachability question.
use crate::lean_proc_scan::{ProcessUse, ScanReport, other_uids_can_reach, scan_processes};
use crate::test_scratch::scratch;

/// The uid a fixture process claims when it belongs to another user.
const ANOTHER_USER: u32 = 4242;

/// The uid of the machine this test runs on.
fn own_uid() -> u32 {
    return rustix::process::geteuid().as_raw();
}

/// A child stopped when the guard drops.
struct Holder {
    /// The child.
    child: Child,
}

impl Drop for Holder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// What:
/// `fn status_text(state: &str, uid: u32, kernel_thread: bool) -> String` is the part of `/proc/<pid>/status` this scan reads.
/// Why:
/// A fixture process is a file,
/// so the fields are written the way the kernel writes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const statusText = (state: string, uid: number, kernelThread: boolean): string => `Name:\tfixture\nState:\t${state}\nUid:\t${uid}\t${uid}\t${uid}\t${uid}\n` + (kernelThread ? 'Kthread:\t1\n' : '');
/// ```
fn status_text(state: &str, uid: u32, kernel_thread: bool) -> String {
    let mut text: String = format!("Name:\tfixture\nState:\t{state}\nUid:\t{uid}\t{uid}\t{uid}\t{uid}\n");
    if kernel_thread {
        text.push_str("Kthread:\t1\n");
    }
    return text;
}

/// What:
/// `fn add_process(proc_root: &Path, pid: i32, status: &str) -> PathBuf` writes one process of a fixture procfs,
/// with no working directory and no descriptors,
/// as `/proc` shows a process this user may not inspect.
/// Why:
/// That is the case the scan has to classify.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const addProcess = (procRoot: string, pid: number, status: string): string => { const entry = join(procRoot, String(pid)); mkdirSync(entry, { recursive: true }); writeFileSync(join(entry, 'status'), status); return entry; };
/// ```
fn add_process(proc_root: &Path, pid: i32, status: &str) -> PathBuf {
    let entry: PathBuf = proc_root.join(pid.to_string());
    fs::create_dir_all(&entry).expect("a process directory");
    fs::write(entry.join("status"), status).expect("a status");
    return entry;
}

/// What:
/// `fn make_root(directory: &Path, others_may_pass: bool) -> PathBuf` makes a scanned root and a directory above it that denies
/// other users every access when asked,
/// so the reachability question has both answers in a fixture.
/// Why:
/// The repository this machine runs in is behind a directory only its owner may enter,
/// and a checkout in a world-readable place is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const makeRoot = (directory: string, othersMayPass: boolean): string => { const above = join(directory, 'above'); const root = join(above, 'verification'); mkdirSync(root, { recursive: true }); chmodSync(above, othersMayPass ? 0o755 : 0o700); return root; };
/// ```
fn make_root(directory: &Path, others_may_pass: bool) -> PathBuf {
    let above: PathBuf = directory.join("above");
    let root: PathBuf = above.join("verification");
    fs::create_dir_all(&root).expect("a root");
    let mode: u32 = if others_may_pass { 0o755 } else { 0o700 };
    fs::set_permissions(&above, fs::Permissions::from_mode(mode)).expect("a mode");
    return root;
}

/// A kernel thread and an ended process hold no file,
/// so neither is a process that may hold an entry.
#[test]
fn a_kernel_thread_and_an_ended_process_cannot_hold_an_entry() {
    let scratch_root = scratch();
    let proc_root: PathBuf = scratch_root.path().join("proc");
    let root: PathBuf = make_root(scratch_root.path(), false);
    add_process(&proc_root, 11, &status_text("S (sleeping)", 0, true));
    add_process(&proc_root, 12, &status_text("Z (zombie)", own_uid(), false));

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1);

    assert_eq!((report.inert, report.unreadable), (2, 0), "{report:?}");
    assert!(report.blocks(&root).is_none(), "a kernel thread or an ended process blocked an entry: {report:?}");
}

/// A live process of this user that the kernel hides from every user is the one residue the scan cannot judge,
/// and it is reported as such.
#[test]
fn a_process_of_this_user_that_cannot_be_read_is_the_residue_the_proof_names() {
    let scratch_root = scratch();
    let proc_root: PathBuf = scratch_root.path().join("proc");
    let root: PathBuf = make_root(scratch_root.path(), true);
    add_process(&proc_root, 21, &status_text("S (sleeping)", own_uid(), false));

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1);

    assert_eq!((report.inert, report.unreadable, report.unreadable_reaching), (0, 1, 0), "{report:?}");
    assert!(report.blocks(&root).is_none(), "this user's own undumpable process was dropped, not reported: {report:?}");
    assert!(report.describe().contains("uninspectable"), "{}", report.describe());
}

/// A process of another user that the path lets in is counted as such,
/// and is still not a use the scan saw,
/// so the removal does not stop for it.
#[test]
fn a_process_of_another_user_that_can_reach_the_entry_is_counted_and_named() {
    let scratch_root = scratch();
    let proc_root: PathBuf = scratch_root.path().join("proc");
    let root: PathBuf = make_root(scratch_root.path(), true);
    add_process(&proc_root, 31, &status_text("S (sleeping)", ANOTHER_USER, false));

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1);

    assert_eq!((report.inert, report.unreadable, report.unreadable_reaching), (0, 1, 1), "{report:?}");
    assert!(report.blocks(&root).is_none(), "a process the scan never saw was read as a use: {report:?}");
    assert!(report.describe().contains("1 of another user reaching a root"), "{}", report.describe());
}

/// A process of another user that the path denies is counted as a process this user may not inspect
/// and not as one that could hold the entry.
#[test]
fn a_process_of_another_user_the_path_denies_is_not_counted_as_reaching_it() {
    let scratch_root = scratch();
    let proc_root: PathBuf = scratch_root.path().join("proc");
    let root: PathBuf = make_root(scratch_root.path(), false);
    add_process(&proc_root, 41, &status_text("S (sleeping)", ANOTHER_USER, false));

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1);

    assert!(!other_uids_can_reach(&root), "the fixture was meant to be behind a directory no other user may enter");
    assert_eq!((report.unreadable, report.unreadable_reaching), (1, 0), "{report:?}");
    assert!(report.blocks(&root).is_none(), "a process that cannot reach the entry was read as a use: {report:?}");
}

/// The scan still finds the uses it always found:
/// a process whose working directory is under the root blocks the entry,
/// whatever the reachability of other users.
#[test]
fn a_process_working_under_the_entry_still_blocks_it() {
    let scratch_root = scratch();
    let proc_root: PathBuf = PathBuf::from("/proc");
    let root: PathBuf = scratch_root.path().join("verification");
    let record: PathBuf = root.join("native-fixture");
    fs::create_dir_all(&record).expect("a record");
    let child: Child = Command::new("sleep").arg("120").current_dir(&record).spawn().expect("start sleep");
    let _holder: Holder = Holder { child };

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1);

    let blocker: &ProcessUse = report.blocks(&record).expect("the process using the record was not seen");
    assert_eq!(blocker.how, "cwd", "{blocker:?}");
}

/// A root behind a directory no other user may enter is not reachable,
/// and one in a world-searchable place is.
#[test]
fn reachability_is_read_from_the_permission_bits_above_the_root() {
    let scratch_root = scratch();
    let closed: PathBuf = make_root(&scratch_root.path().join("closed"), false);
    let open: PathBuf = make_root(&scratch_root.path().join("open"), true);

    assert!(!other_uids_can_reach(&closed));
    assert!(other_uids_can_reach(&open));
}
