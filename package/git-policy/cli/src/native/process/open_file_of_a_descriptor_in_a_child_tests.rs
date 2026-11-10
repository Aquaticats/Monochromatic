//! Controls for reading a descriptor's file in a bounded child:
//! a mount this process cannot classify still answers,
//! and a child that does not answer is abandoned rather than waited for.
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
struct Holder {
    /// The `sh` that holds the descriptor.
    child: Child,
}

impl Holder {
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

impl Drop for Holder {
    /// Kill and reap the holder.
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// An `ask` that answers as a read of a file that is not there does,
/// so the parent's own error is seen.
fn not_there(_name: &CStr) -> std::io::Result<OpenedFile> {
    return Err(std::io::Error::from_raw_os_error(libc::ENOENT));
}

/// An `ask` that has not answered by any ceiling this control gives it:
/// a child asleep in a system call it cannot be ended inside.
fn asleep(_name: &CStr) -> std::io::Result<OpenedFile> {
    let how_long: libc::timespec = libc::timespec { tv_sec: 2, tv_nsec: 0 };
    // SAFETY: the interval is a live value of the type the call takes, and no signal handler of this child runs.
    let _slept = unsafe { libc::nanosleep(&raw const how_long, std::ptr::null_mut()) };
    return Err(std::io::Error::from_raw_os_error(libc::EIO));
}

/// A descriptor of another process is answered with the file it has open,
/// by a child,
/// within the ceiling.
#[test]
fn a_child_answers_a_descriptor_of_another_process() {
    let root: PathBuf = fixture("child-descriptor-answer");
    let held: PathBuf = root.join("held");
    std::fs::write(&held, b"held\n").expect("the file the child holds");
    let holder: Holder = Holder::start(held.as_path());
    let opened: OpenedFile = opened_file_of_in_a_child(holder.process().as_path(), std::ffi::OsStr::new("3")).expect("the child answers");
    let metadata: std::fs::Metadata = std::fs::metadata(held.as_path()).expect("the file's own metadata");
    assert_eq!(opened, OpenedFile { device: metadata.dev(), inode: metadata.ino() }, "the child names the file the descriptor has open");
    remove(root.as_path());
}

/// The answer is a refusal the kernel gave,
/// with its own number,
/// rather than an answer invented for it.
#[test]
fn a_refusal_is_the_kernel_number_the_child_read() {
    let root: PathBuf = fixture("child-descriptor-refusal");
    let process: PathBuf = root.join("proc").join("4242");
    std::fs::create_dir_all(process.as_path()).expect("a process directory");
    let error: std::io::Error = read_in_a_child_with(process.as_path(), std::ffi::OsStr::new("3"), CHILD_READ_CEILING, not_there).expect_err("a file that is not there is refused");
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound, "{error}");
    remove(root.as_path());
}

/// A child that has not answered within its ceiling is abandoned,
/// and the caller learns the ceiling rather than the answer,
/// so nothing waits for a server that has stopped.
#[test]
fn a_child_that_does_not_answer_is_abandoned_at_its_ceiling() {
    let root: PathBuf = fixture("child-descriptor-ceiling");
    let process: PathBuf = root.join("proc").join("4242");
    std::fs::create_dir_all(process.as_path()).expect("a process directory");
    let ceiling: Duration = Duration::from_millis(50);
    let began: Instant = Instant::now();
    let error: std::io::Error = read_in_a_child_with(process.as_path(), std::ffi::OsStr::new("3"), ceiling, asleep).expect_err("a child that does not answer is abandoned");
    let waited: Duration = began.elapsed();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut, "{error}");
    assert!(error.to_string().contains("50 ms"), "the refusal names the ceiling: {error}");
    assert!(waited < Duration::from_secs(1), "the caller waited {waited:?} for a child that never answered");
    remove(root.as_path());
}
