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
//! The reachability controls build their fixture below a folder every user can walk to
//! (`a_seat_other_users_may_pass`),
//! never under the build folder,
//! because this host's home folder is mode 700 and would deny another user whatever the fixture does to its own bits
//! (issue 6551).

/// Files,
/// permissions and paths.
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{
        Path,
        PathBuf,
    },
    process::{
        Child,
        Command,
    },
};

/// The prefix of the line a test this host cannot run prints,
/// which the package's `test` task reads by name.
use crate::cgroup_test_support::NOT_RUN_PREFIX;
/// Device and inode of a fixture entry.
use crate::lean_identity::{
    Identity,
    identity_of,
};
/// The scan and the reachability question.
use crate::lean_proc_scan::{
    ProcessUse,
    ScanReport,
    other_uids_can_reach,
    scan_processes,
};
use crate::test_scratch::scratch;
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
fn status_text(
    state: &str,
    uid: u32,
    kernel_thread: bool,
) -> String {
    let mut text: String =
        format!("Name:\tfixture\nState:\t{state}\nUid:\t{uid}\t{uid}\t{uid}\t{uid}\n");
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
fn add_process(
    proc_root: &Path,
    pid: i32,
    status: &str,
) -> PathBuf {
    let entry: PathBuf = proc_root.join(pid.to_string());
    fs::create_dir_all(&entry).expect("a process directory");
    fs::write(entry.join("status"), status).expect("a status");
    return entry;
}

/// What:
/// `const OTHERS_MAY_PASS: u32` is a folder mode that grants its group and every other user reading and searching.
/// Why:
/// It is the mode the reachability fixture gives its own two folders,
/// so that only the one folder above a scanned root decides whether another user can walk to that root,
/// and it is also the mode the temporary seat below a shared folder is widened to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OTHERS_MAY_PASS = 0o755;
/// ```
const OTHERS_MAY_PASS: u32 = 0o755;

/// What:
/// `const OTHERS_MAY_NOT_PASS: u32` is a folder mode that grants its group and every other user nothing at all.
/// Why:
/// It is the one thing the reachability control varies between its two roots:
/// a root behind it must be read as unreachable,
/// and the same root behind `OTHERS_MAY_PASS` as reachable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OTHERS_MAY_NOT_PASS = 0o700;
/// ```
const OTHERS_MAY_NOT_PASS: u32 = 0o700;

/// What:
/// `const PLACES_OTHER_USERS_MAY_PASS: [&str; 2]` names the folders a Unix host usually keeps searchable and writable by
/// every user,
/// in the order the reachability controls try them.
/// Why:
/// A fixture that asks whether another user can reach a root has to stand below a folder that user can walk to,
/// and this account can create one only inside a folder it may write that already lets every user search.
/// `/dev/shm` is the memory-backed one and `/var/tmp` the disk-backed one that survives a reboot;
/// neither is the system temporary directory this repository keeps its scratch out of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PLACES_OTHER_USERS_MAY_PASS = ['/dev/shm', '/var/tmp'];
/// ```
const PLACES_OTHER_USERS_MAY_PASS: [&str; 2] = ["/dev/shm", "/var/tmp"];

/// What:
/// `fn other_users_can_pass_every_folder_above(path: &Path) -> bool` answers,
/// from the permission bits alone,
/// whether a process of another user could walk from `/` down to `path`.
/// Why:
/// The reachability controls choose where they build their fixture with this question and not with
/// `other_uids_can_reach`,
/// so a fault in the function they are checking cannot also decide whether they run at all:
/// a seat chosen by the answer under test would turn a wrong "no" into a control that reports itself not run,
/// and that wrong answer would then never be seen as a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const otherUsersCanPassEveryFolderAbove = (path: string): boolean => path.split('/').every((one) => (statSync(one).mode & 0o077) !== 0);
/// ```
fn other_users_can_pass_every_folder_above(path: &Path) -> bool {
    for ancestor in path.ancestors() {
        // `let Ok(metadata) = ... else` takes the value out of a `Result` and runs the `else` block when it is an error:
        // here a folder whose mode cannot be read is not one this helper can promise another user may pass.
        let Ok(metadata) = fs::metadata(ancestor) else {
            return false;
        };
        if metadata.permissions().mode() & 0o077 == 0 {
            return false;
        }
    }
    return true;
}

/// What:
/// `fn a_seat_other_users_may_pass(candidates: &[&str]) -> Option<tempfile::TempDir>` makes a temporary folder of this
/// control's own below the first candidate that another user can walk to,
/// and widens that folder to `OTHERS_MAY_PASS`,
/// so a fixture below it is judged by its own folders alone.
/// `Option<tempfile::TempDir>` is a temporary folder or nothing:
/// its siblings are `Result<T, E>`,
/// which carries an error,
/// and a bare `TempDir`,
/// which cannot say "this host has no such place".
/// Why:
/// A root built under the build folder can never be reached by another user on a host whose home folder denies them
/// (mode 700 on this host,
/// measured 2026-11-27),
/// so the fixture has to stand where the path lets that user in.
/// The folder is made with `tempdir_in` and never with `tempfile::tempdir`,
/// which this repository's clippy configuration refuses because it places scratch under `/tmp`:
/// here the place is the whole question the control asks,
/// and it is one of the folders every user may enter,
/// not the system temporary directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aSeatOtherUsersMayPass = (candidates: string[]): TempDir | undefined => { for (const parent of candidates) { if (!otherUsersCanPassEveryFolderAbove(parent)) continue; const seat = mkdtempSync(join(parent, 'cargo-shadow-reachability-')); chmodSync(seat, 0o755); return seat; } return undefined; };
/// ```
fn a_seat_other_users_may_pass(candidates: &[&str]) -> Option<tempfile::TempDir> {
    for candidate in candidates {
        let parent: &Path = Path::new(candidate);
        if !other_users_can_pass_every_folder_above(parent) {
            continue;
        }
        // `let Ok(seat) = ... else` keeps going with the next candidate when this folder refuses the temporary one.
        let Ok(seat) = tempfile::Builder::new()
            .prefix("cargo-shadow-reachability-")
            .tempdir_in(parent)
        else {
            continue;
        };
        if fs::set_permissions(seat.path(), fs::Permissions::from_mode(OTHERS_MAY_PASS)).is_err() {
            continue;
        }
        // `Some(seat)` constructs the present variant of `Option`: this candidate worked, and the temporary folder owns
        // its own removal when it drops.
        return Some(seat);
    }
    // `None` is the absent variant of `Option`: no candidate on this host is a folder every user can walk to.
    return None;
}

/// What:
/// `fn seat_for_a_reachability_control(candidates: &[&str]) -> tempfile::TempDir` is the folder a reachability control
/// builds its fixture in,
/// or the end of that test with the line the package's `test` task reads by name.
/// Why:
/// Where the host has no folder every user can walk to,
/// the control cannot answer its question,
/// and a test that returned early would be counted as passed.
/// `NOT_RUN_PREFIX` names the test between brackets in the task's own convention,
/// so the task leaves it out of its totals and reports the run as degraded instead of green.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seatForAReachabilityControl = (candidates: string[]): TempDir => { const seat = aSeatOtherUsersMayPass(candidates); if (seat !== undefined) return seat; throw new Error(`cargo-shadow tests: not run [${testName()}]: ...`); };
/// ```
fn seat_for_a_reachability_control(candidates: &[&str]) -> tempfile::TempDir {
    let found: Option<tempfile::TempDir> = a_seat_other_users_may_pass(candidates);
    // `let Some(seat) = found else` binds the folder when there is one and runs the `else` block when there is not.
    let Some(seat) = found else {
        let test: String = std::thread::current()
            .name()
            .map(str::to_string)
            .unwrap_or_else(|| {
                return "<a test the runner did not name>".to_string();
            });
        panic!(
            "{NOT_RUN_PREFIX}{test}]: this host has no folder that every user can walk to and this account may create one in (tried {}), so the fixture this control asks its reachability question about cannot be built here; it runs where one of those folders exists",
            candidates.join(", ")
        );
    };
    return seat;
}

/// What:
/// `fn make_root(directory: &Path, others_may_pass: bool) -> PathBuf` makes a scanned root below `directory`,
/// with one folder above the root whose mode is the question a control asks,
/// and every folder of the fixture searchable.
/// Why:
/// `other_uids_can_reach` reads every folder between `/` and the root,
/// so a fixture can only decide that question when nothing else above it does:
/// the fixture's own folders are made searchable and the one above the root is left to the caller,
/// and whether the place itself lets another user in is the caller's choice of seat
/// (`a_seat_other_users_may_pass`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const makeRoot = (directory: string, othersMayPass: boolean): string => { const above = join(directory, 'above'); const root = join(above, 'verification'); mkdirSync(root, { recursive: true }); chmodSync(directory, 0o755); chmodSync(above, othersMayPass ? 0o755 : 0o700); chmodSync(root, 0o755); return root; };
/// ```
fn make_root(
    directory: &Path,
    others_may_pass: bool,
) -> PathBuf {
    let above: PathBuf = directory.join("above");
    let root: PathBuf = above.join("verification");
    fs::create_dir_all(&root).expect("a root");
    fs::set_permissions(directory, fs::Permissions::from_mode(OTHERS_MAY_PASS)).expect("a mode");
    let mode: u32 = if others_may_pass {
        OTHERS_MAY_PASS
    } else {
        OTHERS_MAY_NOT_PASS
    };
    fs::set_permissions(&above, fs::Permissions::from_mode(mode)).expect("a mode");
    fs::set_permissions(&root, fs::Permissions::from_mode(OTHERS_MAY_PASS)).expect("a mode");
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
    assert!(
        report.blocks(&root).is_none(),
        "a kernel thread or an ended process blocked an entry: {report:?}"
    );
}

/// A live process of this user that the kernel hides from every user is the one residue the scan cannot judge,
/// and it is reported as such.
/// Why the fixture is a closed place:
/// this control's subject is a process of this user,
/// whose descriptors the kernel hides whoever may walk to the root,
/// so it asks for no reachability here and says so with the folder mode.
#[test]
fn a_process_of_this_user_that_cannot_be_read_is_the_residue_the_proof_names() {
    let scratch_root = scratch();
    let proc_root: PathBuf = scratch_root.path().join("proc");
    let root: PathBuf = make_root(scratch_root.path(), false);
    add_process(
        &proc_root,
        21,
        &status_text("S (sleeping)", own_uid(), false),
    );

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1);

    assert_eq!(
        (report.inert, report.unreadable, report.unreadable_reaching),
        (0, 1, 0),
        "{report:?}"
    );
    assert!(
        report.blocks(&root).is_none(),
        "this user's own undumpable process was dropped, not reported: {report:?}"
    );
    assert!(
        report.describe().contains("uninspectable"),
        "{}",
        report.describe()
    );
}

/// A process of another user that the path lets in is counted as such,
/// and is still not a use the scan saw,
/// so the removal does not stop for it.
/// Why the fixture stands where it does:
/// the answer this control reads is the path's,
/// so the root stands below a folder every user can walk to and not below a home folder that would deny them
/// (issue 6551).
#[test]
fn a_process_of_another_user_that_can_reach_the_entry_is_counted_and_named() {
    let seat: tempfile::TempDir = seat_for_a_reachability_control(&PLACES_OTHER_USERS_MAY_PASS);
    let proc_root: PathBuf = seat.path().join("proc");
    let root: PathBuf = make_root(&seat.path().join("open"), true);
    add_process(
        &proc_root,
        31,
        &status_text("S (sleeping)", ANOTHER_USER, false),
    );

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1);

    assert_eq!(
        (report.inert, report.unreadable, report.unreadable_reaching),
        (0, 1, 1),
        "{report:?}"
    );
    assert!(
        report.blocks(&root).is_none(),
        "a process the scan never saw was read as a use: {report:?}"
    );
    assert!(
        report
            .describe()
            .contains("1 of another user reaching a root"),
        "{}",
        report.describe()
    );
}

/// A process of another user that the path denies is counted as a process this user may not inspect
/// and not as one that could hold the entry.
#[test]
fn a_process_of_another_user_the_path_denies_is_not_counted_as_reaching_it() {
    let scratch_root = scratch();
    let proc_root: PathBuf = scratch_root.path().join("proc");
    let root: PathBuf = make_root(scratch_root.path(), false);
    add_process(
        &proc_root,
        41,
        &status_text("S (sleeping)", ANOTHER_USER, false),
    );

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1);

    assert!(
        !other_uids_can_reach(&root),
        "the fixture was meant to be behind a directory no other user may enter"
    );
    assert_eq!(
        (report.unreadable, report.unreadable_reaching),
        (1, 0),
        "{report:?}"
    );
    assert!(
        report.blocks(&root).is_none(),
        "a process that cannot reach the entry was read as a use: {report:?}"
    );
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
    let child: Child = Command::new("sleep")
        .arg("120")
        .current_dir(&record)
        .spawn()
        .expect("start sleep");
    let _holder: Holder = Holder {
        child,
    };

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1);

    let blocker: &ProcessUse = report
        .blocks(&record)
        .expect("the process using the record was not seen");
    assert_eq!(blocker.how, "cwd", "{blocker:?}");
}

/// A root behind a directory no other user may enter is not reachable,
/// and the same fixture in a place every user can walk to is.
/// Why the fixture stands where it does:
/// the question is read from every folder between `/` and the root,
/// and this account's home folder is mode 700 on this host,
/// so a fixture built under the build folder could never answer the second half
/// (measured 2026-11-27,
/// issue 6551).
#[test]
fn reachability_is_read_from_the_permission_bits_above_the_root() {
    let seat: tempfile::TempDir = seat_for_a_reachability_control(&PLACES_OTHER_USERS_MAY_PASS);
    let closed: PathBuf = make_root(&seat.path().join("closed"), false);
    let open: PathBuf = make_root(&seat.path().join("open"), true);

    // Both roots are searchable themselves, so the mode of the folder above each is the only difference between them.
    let closed_own: u32 = fs::metadata(&closed)
        .expect("the closed root")
        .permissions()
        .mode()
        & 0o077;
    let open_own: u32 = fs::metadata(&open)
        .expect("the open root")
        .permissions()
        .mode()
        & 0o077;
    assert_ne!(
        closed_own, 0,
        "the fixture's own root denies another user by itself, so the folder above it decides nothing: {closed:?}"
    );
    assert_eq!(
        closed_own, open_own,
        "the two roots differ in their own mode and not only in the folder above them"
    );
    assert!(
        !other_uids_can_reach(&closed),
        "a root behind a folder no other user may enter was read as reachable: {closed:?}"
    );
    assert!(
        other_uids_can_reach(&open),
        "a root in a place every user can walk to was read as unreachable: {open:?}"
    );
}

/// A place no other user may enter is never taken as the seat of a reachability fixture,
/// so a control learns from the seat itself that it cannot build its fixture here.
#[test]
fn a_seat_is_not_taken_in_a_place_no_other_user_may_pass() {
    let scratch_root: tempfile::TempDir = scratch();
    let closed: PathBuf = scratch_root.path().join("closed");
    fs::create_dir_all(&closed).expect("a closed place");
    fs::set_permissions(&closed, fs::Permissions::from_mode(OTHERS_MAY_NOT_PASS)).expect("a mode");
    let only_closed: [&str; 1] = [closed.to_str().expect("a UTF-8 path")];

    let seat: Option<tempfile::TempDir> = a_seat_other_users_may_pass(&only_closed);

    assert!(
        seat.is_none(),
        "a folder no other user may enter was taken as the place a fixture is built in: {only_closed:?}"
    );
}

/// What:
/// A control whose host has no folder every user can walk to ends with the line the package's `test` task reads,
/// not with a failure and not with a quiet pass.
/// Why:
/// That task finds the line by the test's own name,
/// leaves the test out of its totals,
/// and prints the run as degraded,
/// which is how a host without such a folder reports these controls as not run instead of failing them
/// (issue 6551).
#[test]
#[should_panic(expected = "cargo-shadow tests: not run [")]
fn a_control_whose_host_has_no_place_other_users_may_pass_reports_itself_not_run() {
    let scratch_root: tempfile::TempDir = scratch();
    let closed: PathBuf = scratch_root.path().join("closed");
    fs::create_dir_all(&closed).expect("a closed place");
    fs::set_permissions(&closed, fs::Permissions::from_mode(OTHERS_MAY_NOT_PASS)).expect("a mode");
    let nowhere: [&str; 1] = [closed.to_str().expect("a UTF-8 path")];

    let _seat: tempfile::TempDir = seat_for_a_reachability_control(&nowhere);
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
fn add_visible_process(
    proc_root: &Path,
    pid: i32,
    namespace: &str,
    cwd: &Path,
    exe: &Path,
) -> PathBuf {
    let entry: PathBuf = add_process(
        proc_root,
        pid,
        &status_text("S (sleeping)", own_uid(), false),
    );
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
fn add_own_namespace(
    proc_root: &Path,
    namespace: &str,
) {
    fs::create_dir_all(proc_root.join("self").join("ns"))
        .expect("the scanning process's namespace directory");
    symlink(namespace, proc_root.join("self").join("ns").join("mnt"))
        .expect("the scanning process's namespace");
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
    add_visible_process(
        &proc_root,
        51,
        "mnt:[fixture]",
        scratch_root.path(),
        &program,
    );

    let report: ScanReport = scan_processes(&proc_root, &[linked_root.as_path()], &[], -1);

    let blocker: &ProcessUse = report
        .blocks(&linked_root)
        .expect("a program run under the linked root was read as no use");
    assert_eq!(blocker.how, "exe", "{blocker:?}");
    assert!(
        report.blocks(&linked_root.join("native-fixture")).is_some(),
        "the program itself was not blocked: {report:?}"
    );
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
    add_visible_process(
        &proc_root,
        61,
        "mnt:[a container]",
        scratch_root.path(),
        &program,
    );

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &[], -1);

    assert_eq!(report.foreign_namespace, 1, "{report:?}");
    let blocker: &ProcessUse = report
        .blocks(&root)
        .expect("a program run in another mount namespace was read as no use");
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
    let entry: PathBuf = add_visible_process(
        &proc_root,
        71,
        "mnt:[a container]",
        &elsewhere,
        &elsewhere.join("cargo"),
    );
    symlink(&lock, entry.join("fd").join("3")).expect("a descriptor on the lock");
    let identities: [Identity; 1] = [identity_of(&lock).expect("the lock's identity")];

    let report: ScanReport = scan_processes(&proc_root, &[root.as_path()], &identities, -1);

    let blocker: &ProcessUse = report
        .blocks(&root.join("build").join("fixture-0123456789abcdef"))
        .expect("a container's build lock blocked no entry");
    assert_eq!(blocker.how, "identity", "{blocker:?}");
    assert_eq!(blocker.pid, 71, "{blocker:?}");
}
