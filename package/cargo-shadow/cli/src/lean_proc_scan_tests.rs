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
//! Two spellings are checked as well:
//! a root reached through a symbolic link holds what the kernel names without the link,
//! and a program run inside another mount namespace is a held file like an open one.

/// Files,
/// permissions and paths.
use std::{fs, os::unix::fs::PermissionsExt, path::{Path, PathBuf}, process::{Child, Command}};

/// The scan and the reachability question.
use crate::lean_proc_scan::{ProcessUse, ScanReport, other_uids_can_reach, scan_processes};
use crate::test_scratch::scratch;
/// Device and inode of a fixture entry.
use crate::lean_identity::{Deadline, Identity, identity_of};
use std::time::Duration;
/// Making the links a fixture procfs is written with.
use std::os::unix::fs::symlink;

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

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1, &Deadline::after(Duration::from_secs(60))).expect("the scan finished before its deadline");

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

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1, &Deadline::after(Duration::from_secs(60))).expect("the scan finished before its deadline");

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

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1, &Deadline::after(Duration::from_secs(60))).expect("the scan finished before its deadline");

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

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1, &Deadline::after(Duration::from_secs(60))).expect("the scan finished before its deadline");

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

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1, &Deadline::after(Duration::from_secs(60))).expect("the scan finished before its deadline");

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

/// What:
/// `fn add_visible_process(proc_root: &Path, pid: i32, namespace: &str, cwd: &Path, exe: &Path) -> PathBuf` writes one process
/// of a fixture procfs that the scan can read:
/// a working directory,
/// a program and a mount namespace.
/// Why:
/// The spelling cases need a process whose program is a file under the root,
/// which a fixture procfs can show without starting anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const addVisibleProcess = (procRoot: string, pid: number, namespace: string, cwd: string, exe: string): string => { const entry = addProcess(procRoot, pid, statusText('S (sleeping)', ownUid(), false)); mkdirSync(join(entry, 'ns'), { recursive: true }); mkdirSync(join(entry, 'fd'), { recursive: true }); symlink(namespace, join(entry, 'ns/mnt')); symlink(cwd, join(entry, 'cwd')); symlink(exe, join(entry, 'exe')); return entry; };
/// ```
fn add_visible_process(proc_root: &Path, pid: i32, namespace: &str, cwd: &Path, exe: &Path) -> PathBuf {
    let entry: PathBuf = add_process(proc_root, pid, &status_text("S (sleeping)", own_uid(), false));
    fs::create_dir_all(entry.join("ns")).expect("a namespace directory");
    fs::create_dir_all(entry.join("fd")).expect("a descriptor directory");
    symlink(namespace, entry.join("ns").join("mnt")).expect("a mount namespace");
    symlink(cwd, entry.join("cwd")).expect("a working directory");
    symlink(exe, entry.join("exe")).expect("a program");
    return entry;
}

/// What:
/// `fn add_own_namespace(proc_root: &Path, namespace: &str)` writes the namespace the scan compares against.
/// Why:
/// A process of the same namespace is read by path,
/// and one of another namespace by identity and by the paths it spells.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const addOwnNamespace = (procRoot: string, namespace: string): void => { mkdirSync(join(procRoot, 'self/ns'), { recursive: true }); symlink(namespace, join(procRoot, 'self/ns/mnt')); };
/// ```
fn add_own_namespace(proc_root: &Path, namespace: &str) {
    fs::create_dir_all(proc_root.join("self").join("ns")).expect("the scanning process's namespace directory");
    symlink(namespace, proc_root.join("self").join("ns").join("mnt")).expect("the scanning process's namespace");
}

/// A root a person spelled through a symbolic link holds what the kernel names without that link,
/// so the program a process runs under it blocks the entry.
/// Why this is the case that made the proof false:
/// `/proc` names a held file the way the kernel holds it,
/// and a build base under a linked home directory matched none of those names,
/// so a sweep removed a test binary a process was running (probe 2026-11-11:
/// `doc/handover/cargo-shadow-lean-build-folders.md`).
#[test]
fn a_root_spelled_through_a_symbolic_link_holds_the_program_a_process_runs_under_it() {
    let scratch_root = scratch();
    let proc_root: PathBuf = scratch_root.path().join("proc");
    let real: PathBuf = scratch_root.path().join("real");
    let root: PathBuf = real.join("verification");
    let program: PathBuf = root.join("native-fixture");
    fs::create_dir_all(&program).expect("a program");
    let linked: PathBuf = scratch_root.path().join("link");
    symlink(&real, &linked).expect("a link to the real directory");
    let linked_root: PathBuf = linked.join("verification");
    add_own_namespace(&proc_root, "mnt:[fixture]");
    add_visible_process(&proc_root, 51, "mnt:[fixture]", scratch_root.path(), &program);

    let report: ScanReport = scan_processes(&proc_root, &[linked_root.as_path()], &[], -1, &Deadline::after(Duration::from_secs(60))).expect("the scan finished before its deadline");

    let blocker: &ProcessUse = report.blocks(&linked_root).expect("a program run under the linked root was read as no use");
    assert_eq!(blocker.how, "exe", "{blocker:?}");
    assert!(report.blocks(&linked_root.join("native-fixture")).is_some(), "the program itself was not blocked: {report:?}");
}

/// A program run inside another mount namespace is a held file like an open one,
/// so a bind mount of the root at the root's own path blocks the entry.
#[test]
fn a_program_run_in_another_mount_namespace_blocks_the_root_spelled_the_same_way() {
    let scratch_root = scratch();
    let proc_root: PathBuf = scratch_root.path().join("proc");
    let root: PathBuf = scratch_root.path().join("verification");
    let program: PathBuf = root.join("native-fixture");
    fs::create_dir_all(&program).expect("a program");
    add_own_namespace(&proc_root, "mnt:[here]");
    add_visible_process(&proc_root, 61, "mnt:[a container]", scratch_root.path(), &program);

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1, &Deadline::after(Duration::from_secs(60))).expect("the scan finished before its deadline");

    assert_eq!(report.foreign_namespace, 1, "{report:?}");
    let blocker: &ProcessUse = report.blocks(&root).expect("a program run in another mount namespace was read as no use");
    assert_eq!(blocker.how, "exe", "{blocker:?}");
}

/// A process of another mount namespace that holds one of the identities the scan was given holds the whole root,
/// so every entry of it is blocked.
/// Why:
/// The identities handed in are the root's build locks,
/// and a container's cargo holds them while it builds.
#[test]
fn a_process_in_another_mount_namespace_holding_a_build_lock_blocks_every_entry_of_the_root() {
    let scratch_root = scratch();
    let proc_root: PathBuf = scratch_root.path().join("proc");
    let root: PathBuf = scratch_root.path().join("verification");
    let elsewhere: PathBuf = scratch_root.path().join("elsewhere");
    fs::create_dir_all(&elsewhere).expect("a working directory");
    let lock: PathBuf = root.join(".cargo-build-lock");
    fs::create_dir_all(&root).expect("a root");
    fs::write(&lock, b"").expect("a lock file");
    add_own_namespace(&proc_root, "mnt:[here]");
    let entry: PathBuf = add_visible_process(&proc_root, 71, "mnt:[a container]", &elsewhere, &elsewhere.join("cargo"));
    symlink(&lock, entry.join("fd").join("3")).expect("a descriptor on the lock");
    let identities: [Identity; 1] = [identity_of(&lock).expect("the lock's identity")];

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &identities, -1, &Deadline::after(Duration::from_secs(60))).expect("the scan finished before its deadline");

    let blocker: &ProcessUse = report.blocks(&root.join("build").join("fixture-0123456789abcdef")).expect("a container's build lock blocked no entry");
    assert_eq!(blocker.how, "identity", "{blocker:?}");
    assert_eq!(blocker.pid, 71, "{blocker:?}");
}
