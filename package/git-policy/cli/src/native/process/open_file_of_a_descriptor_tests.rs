//! Controls for reading a descriptor's file:
//! the file it has open is found,
//! a process number that no process has is an error rather than a guess,
//! and a table that cannot be asked is never resolved from the text of a link.
#![cfg(target_os = "linux")]

use super::*;
use crate::test_support::{fixture, remove, spawn_owned};
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;
use std::os::unix::fs::MetadataExt as _;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// How long a child is given to open its descriptor.
const PATIENCE: Duration = Duration::from_secs(10);

/// A child of this test that holds a descriptor open,
/// in a group of its own so a failed assertion cannot leave it running.
struct DescriptorHolder {
    /// The `sh` that holds the descriptor.
    child: Child,
}

impl DescriptorHolder {
    /// Start a shell that opens `file` as descriptor 3 and sleeps.
    fn start(file: &Path) -> Self {
        let mut command: Command = Command::new("sh");
        command.arg("-c").arg("exec 3<>\"$1\"; exec sleep 120").arg("sh").arg(file.as_os_str()).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        let child: Child = spawn_owned(&mut command).expect("start the holder");
        let holder: Self = Self { child };
        let started: Instant = Instant::now();
        // The name alone is not enough: a child of this process has a descriptor of its own before the shell starts,
        // so the descriptor is waited for as a link to this very file.
        while !std::fs::read_dir(holder.process().join("fd")).expect("descriptors").flatten().any(|entry| {
            return entry.file_name() == std::ffi::OsStr::new("3") && std::fs::read_link(entry.path()).is_ok_and(|target| { return target == file; });
        }) {
            assert!(started.elapsed() < PATIENCE, "the holder opened descriptor 3");
            std::thread::sleep(Duration::from_millis(10));
        }
        return holder;
    }

    /// The process table directory of this host's view of the child.
    fn process(&self) -> PathBuf {
        return PathBuf::from("/proc").join(self.child.id().to_string());
    }
}

impl Drop for DescriptorHolder {
    /// Kill and reap the holder.
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The device and inode a file has.
fn identity_of_file(file: &Path) -> OpenedFile {
    let metadata: std::fs::Metadata = std::fs::metadata(file).expect("the file's own metadata");
    return OpenedFile { device: metadata.dev(), inode: metadata.ino() };
}

/// A descriptor of another process is read as the file it has open,
/// and another file is another identity.
#[test]
fn a_descriptor_of_another_process_is_the_file_it_has_open() {
    let root: PathBuf = fixture("open-file-of-descriptor");
    let held: PathBuf = root.join("held");
    let other: PathBuf = root.join("other");
    std::fs::write(&held, b"held\n").expect("the file the child holds");
    std::fs::write(&other, b"other\n").expect("another file");
    let holder: DescriptorHolder = DescriptorHolder::start(held.as_path());
    let opened: OpenedFile = opened_file_of(holder.process().as_path(), std::ffi::OsStr::new("3")).expect("the descriptor is read");
    assert_eq!(opened, identity_of_file(held.as_path()), "the descriptor names the file it has open");
    assert_ne!(opened, identity_of_file(other.as_path()), "another file is another identity");
    remove(root.as_path());
}

/// A process number that no process has is the kernel's own refusal,
/// which a caller reads as a process that ended during the scan.
#[test]
fn a_process_number_that_no_process_has_is_an_error() {
    let mut child: Child = Command::new("true").spawn().expect("start true");
    let pid: u32 = child.id();
    child.wait().expect("reap true");
    let error: std::io::Error = opened_file_of(PathBuf::from("/proc").join(pid.to_string()).as_path(), std::ffi::OsStr::new("0")).expect_err("a gone process has no descriptor to copy");
    assert_eq!(error.raw_os_error(), Some(libc::ESRCH), "{error}");
}

/// A directory of a process table that is not a process number is refused rather than given to the kernel.
#[test]
fn a_table_directory_that_is_not_a_process_number_is_refused() {
    let named: std::io::Error = opened_file_of(Path::new("/proc/self"), std::ffi::OsStr::new("0")).expect_err("`self` is not a number");
    assert_eq!(named.kind(), std::io::ErrorKind::InvalidInput, "{named}");
    let unnamed: std::io::Error = opened_file_of(Path::new("/proc"), std::ffi::OsStr::new("0")).expect_err("`/proc` names no process");
    assert_eq!(unnamed.kind(), std::io::ErrorKind::InvalidInput, "{unnamed}");
}

/// A descriptor name that is not a number is refused rather than given to the kernel.
#[test]
fn a_descriptor_name_that_is_not_a_number_is_refused() {
    let error: std::io::Error = opened_file_of(PathBuf::from("/proc").join(std::process::id().to_string()).as_path(), std::ffi::OsStr::new("fd")).expect_err("`fd` is not a number");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput, "{error}");
}

/// A descriptor the process does not hold is the kernel's own refusal.
#[test]
fn a_descriptor_the_process_does_not_hold_is_an_error() {
    let error: std::io::Error = opened_file_of(PathBuf::from("/proc").join(std::process::id().to_string()).as_path(), std::ffi::OsStr::new("4321")).expect_err("no such descriptor");
    assert_eq!(error.raw_os_error(), Some(libc::EBADF), "{error}");
}
