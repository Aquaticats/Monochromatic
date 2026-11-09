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
