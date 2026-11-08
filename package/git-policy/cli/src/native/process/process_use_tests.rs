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
    let target: UseTarget = UseTarget::File { mount: mount_of(root.join("index.lock").as_path()), inode: metadata.ino() };
    assert!(processes_using(&target).users.is_empty());
    let holder: Sleeper = Sleeper::start(root.as_path(), Some(root.join("alias").as_path()));
    let scan: UseScan = processes_using(&target);
    assert!(scan.users.iter().any(|user| { return user.pid == holder.pid(); }), "{scan:?}");
    drop(holder);
    remove(root.as_path());
}

/// A file system is found in the mount table by the device number of its files,
/// and a descriptor is recognised by the mount number and the inode the kernel writes for it,
/// so no descriptor of another process is ever resolved on its own disk.
#[test]
fn a_descriptor_is_named_by_its_mount_and_inode_rather_than_by_resolving_it() {
    let root: PathBuf = fixture("process-use-fdinfo");
    std::fs::write(root.join("fdinfo"), "pos:\t0\nflags:\t0100000\nmnt_id:\t40\nino:\t4242\n").expect("one descriptor");
    let described: PathBuf = root.join("fdinfo");
    assert!(descriptor_is_the_file(described.as_path(), 40, 4242).expect("a descriptor of this mount"), "the descriptor on the mount is the file");
    assert!(!descriptor_is_the_file(described.as_path(), 40, 4243).expect("a descriptor of this mount"), "another inode on the same mount is another file");
    assert!(!descriptor_is_the_file(described.as_path(), 41, 4242).expect("a descriptor of another mount"), "the same inode number on another mount is another file");
    assert!(!descriptor_is_the_file(described.as_path(), 0, 4242).expect("a descriptor without a mount"), "a file whose mount is not known matches no descriptor");
    remove(root.as_path());
}

/// The scanning process is never its own user.
#[cfg(target_os = "linux")]
#[test]
fn the_scanning_process_is_not_reported() {
    let root: PathBuf = fixture("process-use-self").canonicalize().expect("canonical root");
    let held: std::fs::File = std::fs::File::create(root.join("held")).expect("held");
    let metadata: std::fs::Metadata = held.metadata().expect("metadata");
    let scan: UseScan = processes_using(&UseTarget::File { mount: mount_of(root.join("held").as_path()), inode: metadata.ino() });
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
