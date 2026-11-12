//! Controls for the scan of processes that use a directory or a file.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::MetadataExt;
use std::process::Command;

/// What:
///  A sleeping process that stands in for a live wrapper or Git process,
///  stopped and reaped when the test ends.
/// Why:
///   A control of "a live process holds this" needs a real process number that exists,
///  and a failed assertion must not leave the sleeper running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await using sleeper = spawnSleeper({ cwd, stdin });
/// ```
#[cfg(unix)]
struct Sleeper(std::process::Child);

/// Starting a sleeper.
#[cfg(unix)]
impl Sleeper {
    /// What:
    ///  Start `sleep` with a working directory and a file as its standard input.
    /// Why:
    ///   The working directory and the open descriptor are the two ways the scan sees a process use a path.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// spawn('sleep', ['120'], { cwd, stdio: [openSync(file), 'ignore', 'ignore'] })
    /// ```
    fn start(directory: &Path, held: Option<&Path>) -> Sleeper {
        let mut command: Command = Command::new("sleep");
        command.arg("120").current_dir(directory).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
        match held {
            Some(file) => command.stdin(std::fs::File::open(file).expect("the file to hold open")),
            None => command.stdin(std::process::Stdio::null()),
        };
        return Sleeper(command.spawn().expect("the sleeper starts"));
    }

    /// The sleeper's process number.
    fn pid(&self) -> i64 {
        return i64::from(self.0.id());
    }
}

/// Stopping a sleeper.
#[cfg(unix)]
impl Drop for Sleeper {
    /// Kill and reap the process.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// A process whose working directory lies inside the directory is found,
/// and a process elsewhere is not.
#[cfg(target_os = "linux")]
#[test]
fn a_working_directory_inside_the_directory_is_a_use() {
    let root: PathBuf = fixture("process-use-cwd").canonicalize().expect("canonical root");
    let inside: PathBuf = root.join("transaction/hooks");
    std::fs::create_dir_all(&inside).expect("inside");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");
    let target: PathBuf = root.join("transaction");
    let bystander: Sleeper = Sleeper::start(elsewhere.as_path(), None);
    let before: UseScan = processes_using(&UseTarget::Inside(target.as_path()));
    assert!(!before.users.iter().any(|user| { return user.pid == bystander.pid(); }), "{before:?}");
    let holder: Sleeper = Sleeper::start(inside.as_path(), None);
    let after: UseScan = processes_using(&UseTarget::Inside(target.as_path()));
    assert!(after.users.iter().any(|user| { return user.pid == holder.pid(); }), "{after:?}");
    assert!(!after.users.iter().any(|user| { return user.pid == bystander.pid(); }), "{after:?}");
    drop(holder);
    drop(bystander);
    remove(root.as_path());
}

/// An open descriptor on a file inside the directory is a use,
/// whatever the working directory.
#[cfg(target_os = "linux")]
#[test]
fn an_open_descriptor_inside_the_directory_is_a_use() {
    let root: PathBuf = fixture("process-use-fd").canonicalize().expect("canonical root");
    let target: PathBuf = root.join("transaction");
    std::fs::create_dir_all(&target).expect("transaction");
    std::fs::write(target.join("landing-1.json"), b"{}").expect("record");
    let holder: Sleeper = Sleeper::start(root.as_path(), Some(target.join("landing-1.json").as_path()));
    let scan: UseScan = processes_using(&UseTarget::Inside(target.as_path()));
    assert!(scan.users.iter().any(|user| { return user.pid == holder.pid(); }), "{scan:?}");
    assert_eq!(scan.users.iter().find(|user| { return user.pid == holder.pid(); }).and_then(|user| { return user.command.clone(); }), Some(String::from("sleep")));
    drop(holder);
    remove(root.as_path());
}

/// A file is recognized by device and inode,
/// so a second name for the lock is the lock.
#[cfg(target_os = "linux")]
#[test]
fn an_open_descriptor_is_recognized_by_identity_under_any_name() {
    let root: PathBuf = fixture("process-use-identity").canonicalize().expect("canonical root");
    std::fs::write(root.join("index.lock"), b"").expect("lock");
    std::fs::hard_link(root.join("index.lock"), root.join("alias")).expect("alias");
    let metadata: std::fs::Metadata = std::fs::metadata(root.join("index.lock")).expect("lock metadata");
    let target: UseTarget = UseTarget::File { device: metadata.dev(), inode: metadata.ino() };
    assert!(processes_using(&target).users.is_empty());
    let holder: Sleeper = Sleeper::start(root.as_path(), Some(root.join("alias").as_path()));
    let scan: UseScan = processes_using(&target);
    assert!(scan.users.iter().any(|user| { return user.pid == holder.pid(); }), "{scan:?}");
    drop(holder);
    remove(root.as_path());
}

/// A descriptor is matched to the file by the device number and the inode the file has.
/// Why:
///  The mount number and inode the kernel writes for a descriptor are neither unique nor
///  complete (audit of 2026-11-09,
///  finding 4):
///  a bind mount of the same file has another mount number,
///  so that holder was not found at all,
///  and two files of two btrfs subvolumes reached through one mount share both numbers.
///  The record's inode number is the cheap test that comes first;
///  the answer is the file the copied descriptor has open,
///  never the text of the descriptor's link (cross-vendor review of 2026-11-09,
///  finding 3).
#[test]
fn a_descriptor_is_matched_by_device_and_inode() {
    let root: PathBuf = fixture("process-use-fdinfo");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("lock");
    let alias: PathBuf = root.join("alias");
    std::fs::hard_link(&lock, &alias).expect("a second name of the lock");
    let other: PathBuf = root.join("other");
    std::fs::write(&other, b"x").expect("another file");
    let metadata: std::fs::Metadata = std::fs::metadata(&lock).expect("lock metadata");
    let (device, inode): (u64, u64) = (metadata.dev(), metadata.ino());
    let holder: Sleeper = Sleeper::start(root.as_path(), Some(alias.as_path()));
    let process: PathBuf = PathBuf::from("/proc").join(holder.pid().to_string());
    assert!(
        descriptor_is_file(process.as_path(), std::ffi::OsStr::new("0"), device, inode).expect("read"),
        "the same file is the same file through any of its names"
    );
    let elsewhere: std::fs::Metadata = std::fs::metadata(&other).expect("the other file's metadata");
    assert!(
        !descriptor_is_file(process.as_path(), std::ffi::OsStr::new("0"), elsewhere.dev(), elsewhere.ino()).expect("read"),
        "another file is not the lock, whatever the descriptor is"
    );
    drop(holder);
    // A table this process cannot ask the kernel about is an error, never a guess from the text of a link: the record
    // names the lock's inode and the link resolves to the lock, and still nothing is concluded.
    let mut gone: std::process::Child = Command::new("true").spawn().expect("start true");
    let gone_pid: u32 = gone.id();
    gone.wait().expect("reap true");
    let table: PathBuf = root.join("table").join(gone_pid.to_string());
    std::fs::create_dir_all(table.join("fd")).expect("the fake descriptor directory");
    std::fs::create_dir_all(table.join("fdinfo")).expect("the fake record directory");
    std::fs::write(table.join("fdinfo").join("0"), format!("pos:\t0\nflags:\t0100000\nmnt_id:\t1\nino:\t{inode}\n")).expect("a record naming the lock's inode");
    std::os::unix::fs::symlink(&lock, table.join("fd").join("0")).expect("a link to the lock");
    assert!(
        descriptor_is_file(table.as_path(), std::ffi::OsStr::new("0"), device, inode).is_err(),
        "a process number no process has is never resolved from a link's target"
    );
    // A record that names another inode is not resolved at all.
    std::fs::write(table.join("fdinfo").join("0"), "pos:\t0\nflags:\t0100000\nmnt_id:\t1\nino:\t2\n").expect("another record");
    assert!(!descriptor_is_file(table.as_path(), std::ffi::OsStr::new("0"), device, inode).expect("read"), "a record that names another inode is not resolved");
    remove(root.as_path());
}

/// Two files of two btrfs subvolumes that share an inode number on one mount are two files:
/// a process holding one is not a user of the other,
/// and the same scan does find that process when it is asked about the file it holds
/// (audit of 2026-11-09,
/// finding 4).
#[cfg(target_os = "linux")]
#[test]
fn two_subvolumes_that_share_an_inode_number_are_two_files() {
    let Some(root) = crate::fs_btrfs_test_support::btrfs_fixture("process-use-subvolumes") else { return };
    let first: PathBuf = root.join("sv-a");
    let second: PathBuf = root.join("sv-b");
    crate::fs_btrfs_ioctl::subvolume_create_at(first.as_path()).expect("the first subvolume");
    crate::fs_btrfs_ioctl::subvolume_create_at(second.as_path()).expect("the second subvolume");
    let lock: PathBuf = first.join("index.lock");
    let held: PathBuf = second.join("held");
    std::fs::write(&lock, b"").expect("lock");
    std::fs::write(&held, b"x").expect("the file the other subvolume holds");
    let lock_metadata: std::fs::Metadata = std::fs::metadata(&lock).expect("lock metadata");
    let held_metadata: std::fs::Metadata = std::fs::metadata(&held).expect("held metadata");
    assert_eq!(lock_metadata.ino(), held_metadata.ino(), "the two subvolumes number their first file alike");
    assert_ne!(lock_metadata.dev(), held_metadata.dev(), "and give it a device number of its own");
    let holder: Sleeper = Sleeper::start(root.as_path(), Some(held.as_path()));
    let own: UseScan = processes_using(&UseTarget::File { device: held_metadata.dev(), inode: held_metadata.ino() });
    assert!(own.users.iter().any(|user| { return user.pid == holder.pid(); }), "the scan finds the process that holds the file: {own:?}");
    let lock_users: UseScan = processes_using(&UseTarget::File { device: lock_metadata.dev(), inode: lock_metadata.ino() });
    assert!(!lock_users.users.iter().any(|user| { return user.pid == holder.pid(); }), "a process holding another subvolume's file is no user of the lock: {lock_users:?}");
    drop(holder);
    remove(root.as_path());
}

/// The scanning process is never its own user.
#[cfg(target_os = "linux")]
#[test]
fn the_scanning_process_is_not_reported() {
    let root: PathBuf = fixture("process-use-self").canonicalize().expect("canonical root");
    let held: std::fs::File = std::fs::File::create(root.join("held")).expect("held");
    let metadata: std::fs::Metadata = held.metadata().expect("metadata");
    let scan: UseScan = processes_using(&UseTarget::File { device: metadata.dev(), inode: metadata.ino() });
    assert!(!scan.users.iter().any(|user| { return user.pid == i64::from(std::process::id()); }), "{scan:?}");
    drop(held);
    remove(root.as_path());
}

/// A process table that cannot be listed is reported,
/// not read as empty.
#[test]
fn an_unreadable_process_table_is_reported() {
    let scan: UseScan = processes_using_through(Path::new("/nonexistent-proc-root"), &UseTarget::Inside(Path::new("/")), 1);
    assert!(scan.users.is_empty());
    assert!(scan.unreadable.is_empty(), "{scan:?}");
    assert!(scan.unavailable.as_deref().is_some_and(|reason| { return reason.contains("/nonexistent-proc-root"); }), "{scan:?}");
}

/// A descriptor whose link text is not a path is of no file,
/// and its record is answered without being opened.
/// Why:
///  The record of the anonymous descriptor is a directory here, which no read can open:
///  a scan that opened it would report a refusal instead of the answer.
///  The file beside it is the control that the gate lets a file through:
///  its record names the lock and its link is the lock.
#[test]
fn a_descriptor_that_is_not_a_file_is_answered_without_its_record() {
    let root: PathBuf = fixture("process-use-not-a-file");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("lock");
    let metadata: std::fs::Metadata = std::fs::metadata(&lock).expect("lock metadata");
    let table: PathBuf = root.join("table").join("4242");
    std::fs::create_dir_all(table.join("fd")).expect("the descriptor directory");
    std::fs::create_dir_all(table.join("fdinfo")).expect("the record directory");
    std::fs::write(table.join("mountinfo"), "7 1 0:43 / / rw - btrfs none rw\n").expect("a mount table");
    std::os::unix::fs::symlink("anon_inode:inotify", table.join("fd").join("3")).expect("a kernel anonymous inode's link");
    std::fs::create_dir_all(table.join("fdinfo").join("3")).expect("a record no read can open");
    assert!(
        !descriptor_is_file(table.as_path(), std::ffi::OsStr::new("3"), metadata.dev(), metadata.ino()).expect("a descriptor that is no file is answered, not refused"),
        "a kernel anonymous inode cannot be the lock"
    );
    std::fs::write(table.join("fdinfo").join("4"), format!("pos:\t0\nflags:\t0100000\nmnt_id:\t7\nino:\t{}\n", metadata.ino())).expect("a record naming the lock");
    std::os::unix::fs::symlink(&lock, table.join("fd").join("4")).expect("a descriptor of the control's own");
    assert!(
        descriptor_is_file(table.as_path(), std::ffi::OsStr::new("4"), metadata.dev(), metadata.ino()).expect("the file is read"),
        "a descriptor whose text is a path is still matched by its record and its file"
    );
    remove(root.as_path());
}

/// A pipe is the file a scan for an anonymous file looks for,
/// and a kernel anonymous inode beside it is not read at all.
/// Why:
///  A step's own output pipe is found by its mount number and inode,
///  so the gate must leave a pipe's record alone.
#[test]
fn a_mounted_scan_reads_a_pipe_and_skips_a_kernel_anonymous_inode() {
    let root: PathBuf = fixture("process-use-pipe");
    let table: PathBuf = root.join("table").join("4242");
    std::fs::create_dir_all(table.join("fd")).expect("the descriptor directory");
    std::fs::create_dir_all(table.join("fdinfo")).expect("the record directory");
    let mount: u64 = 9;
    let inode: u64 = 42;
    std::os::unix::fs::symlink("pipe:[42]", table.join("fd").join("5")).expect("a pipe's link");
    std::fs::write(table.join("fdinfo").join("5"), format!("pos:\t0\nflags:\t00\nmnt_id:\t{mount}\nino:\t{inode}\n")).expect("the pipe's record");
    assert!(
        mounted_descriptor_is_file(table.as_path(), std::ffi::OsStr::new("5"), mount, inode).expect("a pipe is read"),
        "a descriptor of the pipe is the pipe"
    );
    std::os::unix::fs::symlink("anon_inode:inotify", table.join("fd").join("6")).expect("a kernel anonymous inode's link");
    std::fs::create_dir_all(table.join("fdinfo").join("6")).expect("a record no read can open");
    assert!(
        !mounted_descriptor_is_file(table.as_path(), std::ffi::OsStr::new("6"), mount, inode).expect("a kernel anonymous inode is answered, not refused"),
        "a kernel anonymous inode is not a pipe"
    );
    remove(root.as_path());
}

/// A record longer than the bound is a descriptor that may hold the file,
/// never a read the scan takes.
#[test]
fn a_record_past_the_bound_is_a_file_that_may_be_held() {
    let root: PathBuf = fixture("process-use-record-bound");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("lock");
    let metadata: std::fs::Metadata = std::fs::metadata(&lock).expect("lock metadata");
    let table: PathBuf = root.join("table").join("4242");
    std::fs::create_dir_all(table.join("fd")).expect("the descriptor directory");
    std::fs::create_dir_all(table.join("fdinfo")).expect("the record directory");
    std::os::unix::fs::symlink(&lock, table.join("fd").join("3")).expect("a descriptor of the control's own");
    let head: String = format!("pos:\t0\nmnt_id:\t7\nino:\t{}\n", metadata.ino());
    let filler: String = "x".repeat(crate::descriptor_record::RECORD_BYTES_AT_MOST - head.len() + 1);
    std::fs::write(table.join("fdinfo").join("3"), format!("{head}{filler}\n")).expect("a record past the bound");
    match descriptor_is_file_with(table.as_path(), std::ffi::OsStr::new("3"), metadata.dev(), metadata.ino(), read_the_link) {
        Err(Unread::File(reason)) => assert!(
            reason.contains(&crate::descriptor_record::RECORD_BYTES_AT_MOST.to_string()),
            "the reason says how long the scan reads: {reason}"
        ),
        other => panic!("a record past the bound is a file that may be held: {other:?}"),
    }
    remove(root.as_path());
}

/// The reader of a descriptor's open file that a control uses for its own `/proc`:
/// the file its link names,
/// which is what a directory of links can say and what no judgement reads.
fn read_the_link(process: &Path, fd: &std::ffi::OsStr) -> std::io::Result<crate::open_file_of_a_descriptor::OpenedFile> {
    use std::os::unix::fs::MetadataExt as _;
    let opened: std::fs::Metadata = std::fs::metadata(process.join("fd").join(fd))?;
    return Ok(crate::open_file_of_a_descriptor::OpenedFile { device: opened.dev(), inode: opened.ino() });
}

/// A scan that reached the moment it was given says it did not finish,
/// and one given no moment reports the whole table.
/// Why:
///  Every caller reads an unfinished table as a proof it may not act on,
///  and a walk that stopped must never read as a table with nothing in it.
#[test]
fn a_scan_that_reached_its_moment_says_it_did_not_finish() {
    let root: PathBuf = fixture("process-use-bound");
    let process: PathBuf = root.join("proc").join("4242");
    std::fs::create_dir_all(process.join("fdinfo")).expect("the record directory");
    std::fs::create_dir_all(process.join("fd")).expect("the descriptor directory");
    let stopped: UseScan = processes_using_until(root.join("proc").as_path(), &UseTarget::Inside(Path::new("/")), 1, Some(std::time::Instant::now()));
    assert!(stopped.users.is_empty(), "{stopped:?}");
    assert!(stopped.unavailable.as_deref().is_some_and(|reason| { return reason.contains("time bound"); }), "{stopped:?}");
    let whole: UseScan = processes_using_until(root.join("proc").as_path(), &UseTarget::Inside(Path::new("/")), 1, None);
    assert!(whole.unavailable.is_none(), "a scan with no moment reports the table it walked: {whole:?}");
    remove(root.as_path());
}

/// The variable that makes this test binary a process holding one inotify descriptor with many watches.
const HOLD_WATCHES_ROOT: &str = "CLI_GIT_TEST_HOLD_WATCHES_ROOT";
/// The number of watches that descriptor holds.
const HOLD_WATCHES_COUNT: &str = "CLI_GIT_TEST_HOLD_WATCHES_COUNT";

/// The test this control starts as its own holder process,
/// and the only test such a process runs.
/// Why:
///  A record whose size this control decides needs a descriptor of a live process,
///  and the smallest such program is a copy of this test binary running one test of it.
const HOLD_WATCHES_TEST: &str = "process::process_use::tests::a_process_that_holds_that_many_watches_blocks";

/// What:
///  Hold one inotify descriptor with `HOLD_WATCHES_COUNT` watches and block until standard input is closed.
/// Why:
///   This is the fixture of issue 6287:
///  a program of the account that watches a tree,
///  whose descriptor's record the kernel builds one line per watch for.
#[test]
fn a_process_that_holds_that_many_watches_blocks() {
    let Some(watches) = std::env::var_os(HOLD_WATCHES_ROOT) else {
        return;
    };
    let count: usize = std::env::var(HOLD_WATCHES_COUNT).expect("a watch count").parse().expect("a number");
    let watching: PathBuf = PathBuf::from(watches);
    let descriptor: i32 = hold_watches(watching.as_path(), count);
    println!("ready pid={} fd={descriptor} watches={count}", std::process::id());
    use std::io::Write as _;
    std::io::stdout().flush().expect("the ready line");
    let mut buffer: [u8; 1] = [0; 1];
    use std::io::Read as _;
    let _ = std::io::stdin().read(&mut buffer);
}

/// What:
///  The inotify flags of a descriptor this process alone holds.
/// Why:
///   `IN_CLOEXEC` keeps it out of a child, and `IN_NONBLOCK` keeps nothing waiting on it.
const IN_CLOEXEC: i32 = 0o2000000;
/// The same non-blocking flag.
const IN_NONBLOCK: i32 = 0o4000;
/// Every inotify event, as a program that watches a project asks for.
const IN_ALL_EVENTS: u32 = 0x0000_0fff;

/// What:
///  One inotify descriptor holding a watch on each of `count` files below `root`,
///  each file made when it is missing.
/// Why:
///   The number of watches is what decides the size of the descriptor's record.
fn hold_watches(root: &Path, count: usize) -> i32 {
    std::fs::create_dir_all(root).expect("the watched tree");
    // SAFETY: `inotify_init1` takes only flags and answers a descriptor or an error.
    let descriptor: i32 = unsafe { libc::inotify_init1(IN_CLOEXEC | IN_NONBLOCK) };
    assert!(descriptor >= 0, "inotify_init1: {}", std::io::Error::last_os_error());
    for number in 0..count {
        let path: PathBuf = root.join(format!("w{number:08}"));
        if !path.exists() {
            std::fs::File::create(&path).expect("a watched file");
        }
        let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without a zero byte");
        // SAFETY: the descriptor is this process's own, and the name outlives the call.
        let watch: i32 = unsafe { libc::inotify_add_watch(descriptor, name.as_ptr(), IN_ALL_EVENTS) };
        assert!(watch >= 0, "inotify_add_watch {}: {}", path.display(), std::io::Error::last_os_error());
    }
    return descriptor;
}

/// What:
///  A process of this control's own that holds one inotify descriptor with many watches,
///  ended when this value is dropped.
/// Why:
///   A watch's record is built by the kernel from a live descriptor,
///  so the fixture is a process and never a directory of files.
struct Watches(std::process::Child);

impl Watches {
    /// Start a holder with `count` watches below `root`.
    fn start(root: &Path, count: usize) -> Watches {
        let binary: PathBuf = crate::test_support::this_test_binary().expect("this test binary");
        let mut child: std::process::Child = Command::new(binary)
            .args(["--exact", HOLD_WATCHES_TEST, "--nocapture", "--test-threads=1"])
            .env(HOLD_WATCHES_ROOT, root)
            .env(HOLD_WATCHES_COUNT, count.to_string())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("the holder starts");
        crate::test_support::own_group(&child);
        let line: String = ready_line(&mut child);
        assert!(line.contains(&format!("watches={count}")), "the holder says what it holds: {line}");
        return Watches(child);
    }

    /// The number of the holder process.
    fn pid(&self) -> u32 {
        return self.0.id();
    }

    /// The number of its inotify descriptor, as the text of its links says.
    /// Why:
    ///  A scan is what reads records,
    ///  so this walks the links of the process this control started and never a record of it.
    fn inotify_descriptor(&self) -> u32 {
        let descriptors: PathBuf = PathBuf::from("/proc").join(self.pid().to_string()).join("fd");
        for entry in std::fs::read_dir(descriptors).expect("the holder's descriptors") {
            let Ok(item) = entry else {
                continue;
            };
            let Ok(text) = std::fs::read_link(item.path()) else {
                continue;
            };
            if text.as_os_str().as_encoded_bytes().starts_with(b"anon_inode:") {
                return item.file_name().to_string_lossy().parse().expect("a descriptor number");
            }
        }
        panic!("the holder holds an inotify descriptor");
    }
}

impl Drop for Watches {
    /// End the holder and its watches.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// The line a holder prints when its watches are set, read from its standard output.
/// Why:
///  The test harness prints its own lines around the test's output,
///  so the line this control waits for is the one that says a pid and a descriptor,
///  and the bytes it reads are bounded so a holder that says nothing cannot hold the control here.
fn ready_line(child: &mut std::process::Child) -> String {
    use std::io::Read as _;
    let mut output: std::process::ChildStdout = child.stdout.take().expect("the holder's output");
    let mut read: usize = 0;
    while read < READY_BYTES_AT_MOST {
        let mut line: String = String::new();
        let mut byte: [u8; 1] = [0; 1];
        while let Ok(1) = output.read(&mut byte) {
            read += 1;
            if byte[0] == b'\n' {
                break;
            }
            line.push(char::from(byte[0]));
        }
        if line.contains("ready pid=") {
            return line;
        }
    }
    panic!("the holder printed no ready line within {READY_BYTES_AT_MOST} bytes");
}

/// How much of a holder's output this control reads before it says the holder said nothing.
const READY_BYTES_AT_MOST: usize = 4096;

/// What:
///  A process table of the control's own that names one real process of this host,
/// its descriptors and their records being the ones the kernel holds.
/// Why:
///   The scan walks a table and reads records,
///  and a table that names one process the control started is a walk that never reads another program's record.
fn table_of_one(root: &Path, pid: u32) -> PathBuf {
    let table: PathBuf = root.join("proc");
    let process: PathBuf = table.join(pid.to_string());
    std::fs::create_dir_all(process.as_path()).expect("the process's table directory");
    let real: PathBuf = PathBuf::from("/proc").join(pid.to_string());
    for name in ["fd", "fdinfo", "cwd", "comm"] {
        std::os::unix::fs::symlink(real.join(name), process.join(name)).expect("a link into the kernel's own table");
    }
    return table;
}

/// What:
///  The kernel memory this process's own control group holds now, in bytes.
/// Why:
///   The buffer the kernel builds a record in is charged to the reading process's control group with
///  `GFP_KERNEL_ACCOUNT`,
///  so this is where a read's cost shows.
///  `None` where the group does not account kernel memory, which the control then reports rather than asserts.
fn kernel_charge_now() -> Option<u64> {
    let own: String = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    let path: &str = own.lines().find_map(|line| { return line.strip_prefix("0::"); })?;
    let group: PathBuf = PathBuf::from("/sys/fs/cgroup").join(path.trim_start_matches('/'));
    let stat: String = std::fs::read_to_string(group.join("memory.stat")).ok()?;
    for line in stat.lines() {
        if let Some(value) = line.strip_prefix("kernel ") {
            return value.trim().parse().ok();
        }
    }
    return None;
}

/// What:
///  The largest kernel charge of this process's control group seen while it runs.
/// Why:
///   A record's buffer is released the moment the read returns,
///  so a reader that looked afterwards would see nothing of it.
struct KernelSampler {
    /// Whether the sampler still samples.
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// The largest charge seen.
    most: std::sync::Arc<std::sync::atomic::AtomicU64>,
    /// The thread that samples.
    thread: Option<std::thread::JoinHandle<()>>,
}

/// The pause between two samples of the charge.
/// Why:
///  One read of a record of a few megabytes lasts about a quarter of a second,
///  so a sample this often cannot miss its buffer.
const SAMPLE_PAUSE: std::time::Duration = std::time::Duration::from_millis(2);

impl KernelSampler {
    /// Start sampling.
    fn start() -> KernelSampler {
        let running: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let most: std::sync::Arc<std::sync::atomic::AtomicU64> = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let sampling: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&running);
        let seen: std::sync::Arc<std::sync::atomic::AtomicU64> = std::sync::Arc::clone(&most);
        let thread: std::thread::JoinHandle<()> = std::thread::spawn(move || {
            while sampling.load(std::sync::atomic::Ordering::Relaxed) {
                if let Some(charge) = kernel_charge_now() {
                    seen.fetch_max(charge, std::sync::atomic::Ordering::Relaxed);
                }
                std::thread::sleep(SAMPLE_PAUSE);
            }
        });
        return KernelSampler { running, most, thread: Some(thread) };
    }

    /// Stop sampling and answer the largest charge seen.
    fn stop(mut self) -> u64 {
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("the sampler");
        }
        return self.most.load(std::sync::atomic::Ordering::Relaxed);
    }
}

/// What one count of watches came to.
struct Cost {
    /// The record's length in bytes.
    record_bytes: usize,
    /// The kernel memory one direct read of that record was charged with.
    read_kernel: u64,
    /// The kernel memory the scan of the table was charged with.
    scan_kernel: u64,
    /// How long the scan took.
    scan: std::time::Duration,
}

/// What:
///  Read the holder's record once, then scan the table that names it, and report what each cost.
/// Why:
///   The direct read is the control that the measurement can see a record's cost at all:
///  the scan must not pay it.
fn cost_of_one_scan(root: &Path, watches: &Watches) -> Cost {
    let table: PathBuf = table_of_one(root, watches.pid());
    let record: PathBuf = PathBuf::from("/proc").join(watches.pid().to_string()).join("fdinfo").join(watches.inotify_descriptor().to_string());
    let before: u64 = kernel_charge_now().unwrap_or(0);
    let reading: KernelSampler = KernelSampler::start();
    let text: String = std::fs::read_to_string(record.as_path()).expect("the holder's record, which this control started");
    let read_kernel: u64 = reading.stop().saturating_sub(before);
    let target: UseTarget = UseTarget::File { device: 0, inode: 0 };
    let scanning_base: u64 = kernel_charge_now().unwrap_or(0);
    let scanning: KernelSampler = KernelSampler::start();
    let began: std::time::Instant = std::time::Instant::now();
    let scan: UseScan = processes_using_until(table.as_path(), &target, std::process::id(), Some(std::time::Instant::now() + PROCESS_SCAN_CEILING));
    let scan_time: std::time::Duration = began.elapsed();
    let scan_kernel: u64 = scanning.stop().saturating_sub(scanning_base);
    assert!(scan.unavailable.is_none(), "the control's table is one process and the scan finishes it: {scan:?}");
    assert!(scan.users.is_empty(), "no descriptor holds a file of no device and no inode: {scan:?}");
    return Cost { record_bytes: text.len(), read_kernel, scan_kernel, scan: scan_time };
}

/// The small inotify set this control starts with.
const SMALL_WATCH_COUNT: usize = 2_000;

/// The large one: a record of megabytes, whose read is the cost issue 6287 found.
const LARGE_WATCH_COUNT: usize = 64_000;

/// The most a scan may cost because the record beside it is that much longer.
/// Why:
///  One read of the 64,000-watch record measured 264 ms on 2026-11-12,
///  and a scan that pays nothing for its size stays within a small part of that.
const SCAN_ALLOWANCE: std::time::Duration = std::time::Duration::from_millis(100);

/// What:
///  The scan's time and the kernel memory it is charged with stay flat as the watcher's record grows.
/// Why:
///  This is the defect of issue 6287 as a control:
///  a program of the account that watches a project decided how much memory one Git command's scan cost,
///  and a scope capped at 512 MB was killed for it.
///  The scan of a table that names only the holder walks one process,
///  so nothing here reads a record of a program this control did not start.
#[test]
fn the_scan_costs_nothing_for_a_record_whose_size_another_process_decides() {
    let root: PathBuf = fixture("process-use-watches");
    let small: Cost = {
        let watches: Watches = Watches::start(root.join("small").as_path(), SMALL_WATCH_COUNT);
        cost_of_one_scan(root.join("small-table").as_path(), &watches)
    };
    let large: Cost = {
        let watches: Watches = Watches::start(root.join("large").as_path(), LARGE_WATCH_COUNT);
        cost_of_one_scan(root.join("large-table").as_path(), &watches)
    };
    assert!(
        large.record_bytes > small.record_bytes * (LARGE_WATCH_COUNT / SMALL_WATCH_COUNT) / 2,
        "the fixture's record grows with the watches it holds: {} bytes at {SMALL_WATCH_COUNT} watches, {} at {LARGE_WATCH_COUNT}",
        small.record_bytes,
        large.record_bytes
    );
    assert!(
        large.scan <= small.scan + SCAN_ALLOWANCE,
        "the scan does not read the record: it took {:?} beside a record of {} bytes and {:?} beside one of {} bytes",
        small.scan,
        small.record_bytes,
        large.scan,
        large.record_bytes
    );
    println!(
        "CONTROL the descriptor scan pays nothing for a record another process sized: {SMALL_WATCH_COUNT} watches hold {} bytes of record and the scan took {:?}, charged {} bytes of kernel memory; {LARGE_WATCH_COUNT} watches hold {} bytes and the scan took {:?}, charged {} bytes; one direct read of the large record was charged {} bytes",
        small.record_bytes,
        small.scan,
        small.scan_kernel,
        large.record_bytes,
        large.scan,
        large.scan_kernel,
        large.read_kernel
    );
    if small.read_kernel == 0 || large.read_kernel == 0 {
        println!(
            "SKIPPED the kernel-memory half: this control group charges nothing for a direct read of {} bytes of records",
            large.record_bytes
        );
        remove(root.as_path());
        return;
    }
    assert!(
        large.scan_kernel * 2 < large.read_kernel,
        "the scan is charged {} bytes of kernel memory where one direct read of the same record is charged {} bytes",
        large.scan_kernel,
        large.read_kernel
    );
    remove(root.as_path());
}
