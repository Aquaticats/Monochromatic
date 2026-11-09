//! Controls for reading a descriptor's file:
//! the file it has open is found on a mount whose attributes are in memory,
//! and a mount that this kernel cannot answer for is refused rather than waited on.
#![cfg(target_os = "linux")]

use super::*;
use crate::test_support::{fixture, remove, spawn_owned};
/// `MetadataExt` gives the device and inode a file has.
use std::os::unix::fs::MetadataExt as _;
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;
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

/// A table of a process that the kernel does not know:
/// its record names a mount,
/// and its mount table says what that mount is.
fn table_with_mount(root: &Path, mount: Option<u64>, kind: Option<&str>) -> PathBuf {
    let process: PathBuf = root.join("proc").join("4242");
    std::fs::create_dir_all(process.join("fdinfo")).expect("the record directory");
    std::fs::create_dir_all(process.join("fd")).expect("the descriptor directory");
    match mount {
        Some(number) => std::fs::write(process.join("fdinfo").join("3"), format!("pos:\t0\nflags:\t0100000\nmnt_id:\t{number}\nino:\t1\n")).expect("a record naming a mount"),
        None => std::fs::write(process.join("fdinfo").join("3"), "pos:\t0\nflags:\t0100000\nino:\t1\n").expect("a record naming no mount"),
    }
    match kind {
        Some(name) => std::fs::write(process.join("mountinfo"), format!("7 1 0:43 / / rw - {name} none rw\n")).expect("a mount table"),
        None => std::fs::write(process.join("mountinfo"), "8 1 0:44 / /other rw - btrfs none rw\n").expect("a mount table that names another mount"),
    }
    return process;
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

/// A descriptor on a file system served by another process or machine is refused:
/// its attributes are not in the memory of this kernel,
/// and walking its path would wait on a server that has stopped.
#[test]
fn a_descriptor_on_a_mount_that_is_not_local_is_refused() {
    let root: PathBuf = fixture("open-file-not-local");
    let process: PathBuf = table_with_mount(root.as_path(), Some(7), Some("fuse"));
    let error: std::io::Error = opened_file_of(process.as_path(), std::ffi::OsStr::new("3")).expect_err("a fuse mount is not read");
    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported, "{error}");
    assert!(error.to_string().contains("`fuse`"), "the reason names the file system: {error}");
    remove(root.as_path());
}

/// A descriptor whose mount the process's own table does not name is refused:
/// an unknown mount is not known to be local.
#[test]
fn a_descriptor_on_a_mount_the_table_does_not_name_is_refused() {
    let root: PathBuf = fixture("open-file-unknown-mount");
    let process: PathBuf = table_with_mount(root.as_path(), Some(7), None);
    let error: std::io::Error = opened_file_of(process.as_path(), std::ffi::OsStr::new("3")).expect_err("an unknown mount is not read");
    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported, "{error}");
    assert!(error.to_string().contains("not in the process's mount table"), "{error}");
    remove(root.as_path());
}

/// A record that names no mount is refused:
/// there is nothing to ask about its file system.
#[test]
fn a_descriptor_whose_record_names_no_mount_is_refused() {
    let root: PathBuf = fixture("open-file-no-mount");
    let process: PathBuf = table_with_mount(root.as_path(), None, Some("btrfs"));
    let error: std::io::Error = opened_file_of(process.as_path(), std::ffi::OsStr::new("3")).expect_err("no mount is not read");
    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported, "{error}");
    remove(root.as_path());
}

/// A descriptor of a process that has ended is the kernel's own refusal,
/// which a caller reads as a process that ended during the scan.
#[test]
fn a_process_that_has_ended_is_an_error() {
    let mut child: Child = Command::new("true").spawn().expect("start true");
    let pid: u32 = child.id();
    child.wait().expect("reap true");
    let error: std::io::Error = opened_file_of(PathBuf::from("/proc").join(pid.to_string()).as_path(), std::ffi::OsStr::new("0")).expect_err("a gone process has no record");
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound, "{error}");
}
