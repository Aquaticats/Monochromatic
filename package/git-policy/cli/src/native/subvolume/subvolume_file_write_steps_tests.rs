//! Controls for the two shared write steps and the switch a control uses to keep
//! its writes in the page cache.
//! The refusal of a directory flush is shown with `procfs`,
//! which answers `EINVAL` to it,
//! the way `private_storage_tests` shows a directory sync that a filesystem cannot do.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// A directory whose flush the kernel refuses,
/// or nothing where this system has no `procfs`.
fn directory_that_cannot_be_flushed() -> Option<&'static Path> {
    let procfs: &Path = Path::new("/proc/self");
    let refused: bool = std::fs::File::open(procfs).is_ok_and(|handle| { return handle.sync_all().is_err(); });
    return if refused { Some(procfs) } else { None };
}

/// A directory flush that the kernel refuses is reported,
/// which is what makes the refusal the sign that a flush ran at all.
#[test]
fn a_directory_that_cannot_be_flushed_is_reported() {
    let Some(procfs): Option<&Path> = directory_that_cannot_be_flushed() else {
        eprintln!("skipped: this system has no procfs, so no directory flush is refused");
        return;
    };
    assert!(sync_directory_to_disk(procfs).is_err(), "the kernel refuses to flush {}", procfs.display());
}

/// A thread that keeps its writes in memory does not ask the kernel to flush,
/// so the flush it would otherwise be refused never runs.
#[test]
fn a_thread_that_keeps_its_writes_in_memory_does_not_flush() {
    let Some(procfs): Option<&Path> = directory_that_cannot_be_flushed() else {
        eprintln!("skipped: this system has no procfs, so no directory flush is refused");
        return;
    };
    let outcome: std::io::Result<()> = {
        let _kept = WritesKeptInMemory::for_this_thread();
        sync_directory_to_disk(procfs)
    };
    assert!(outcome.is_ok(), "the flush was skipped: {outcome:?}");
    assert!(sync_directory_to_disk(procfs).is_err(), "the switch is back when the body ended");
}

/// The switch goes back when the body panics,
/// so a control that fails does not leave its thread keeping every later write in memory.
#[test]
fn the_switch_goes_back_when_the_body_panics() {
    let Some(procfs): Option<&Path> = directory_that_cannot_be_flushed() else {
        eprintln!("skipped: this system has no procfs, so no directory flush is refused");
        return;
    };
    let caught: std::thread::Result<()> = std::panic::catch_unwind(|| {
        let _kept = WritesKeptInMemory::for_this_thread();
        panic!("the body fails on purpose");
    });
    assert!(caught.is_err(), "the body panics on purpose");
    assert!(sync_directory_to_disk(procfs).is_err(), "the switch is back after a body that panicked");
}

/// A write that keeps its bytes in memory writes all of them and no flush runs,
/// and the same write on a thread that does not keep them flushes.
#[test]
fn a_write_that_keeps_its_bytes_in_memory_writes_all_of_them() {
    let root: PathBuf = fixture("write-steps");
    let kept: PathBuf = root.join("kept");
    let flushed: PathBuf = root.join("flushed");
    let bytes: &[u8] = b"a record that must arrive whole\n";
    {
        let _kept = WritesKeptInMemory::for_this_thread();
        let mut file: std::fs::File = std::fs::File::create(kept.as_path()).expect("kept file");
        write_all_then_sync(&mut file, bytes).expect("the write and its skipped flush");
    }
    let mut file: std::fs::File = std::fs::File::create(flushed.as_path()).expect("flushed file");
    write_all_then_sync(&mut file, bytes).expect("the write and its flush");
    assert_eq!(std::fs::read(kept.as_path()).expect("kept bytes"), bytes);
    assert_eq!(std::fs::read(flushed.as_path()).expect("flushed bytes"), bytes);
    remove(root.as_path());
}
