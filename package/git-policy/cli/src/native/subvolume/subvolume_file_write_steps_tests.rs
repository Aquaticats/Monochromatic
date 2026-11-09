//! Controls for the shared write steps and the switch a control uses to keep
//! its writes in the page cache,
//! for the extent that switch covers inside the command and the conversion,
//! and for the writer that replaces a link at a name and never writes through it.
//! The refusal of a directory flush is shown with `procfs`,
//! which answers `EINVAL` to it,
//! the way `private_storage_tests` shows a directory sync that a filesystem cannot do.

use super::*;
use crate::subvolume_test_support::{Flushing, PlainPlatform, checkout_with_worktree, migrate_arguments, run_command};
use crate::subvolume_arguments::MigrateArguments;
use crate::subvolume_error::Interrupt;
use crate::test_support::{fixture, remove};
use crate::worktree_convert::{ConvertOptions, convert_worktree};
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

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

/// A write that keeps its bytes in memory writes all of them,
/// and so does one that flushes,
/// which is the write step's own promise.
/// Which of the two asked the kernel to flush is what the control below tells.
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

/// What:
///  A handle that takes every byte and answers `EINVAL` to a flush,
/// or nothing where this system has no such handle.
/// Why:
///   A regular file reads back from the page cache whether or not its flush ran,
/// so the bytes alone never say which
///  branch of the write step was taken;
/// this handle answers differently to the two,
/// which is what tells them apart.
fn handle_that_cannot_be_flushed() -> Option<std::fs::File> {
    let handle: std::fs::File = std::fs::OpenOptions::new().write(true).open("/dev/null").ok()?;
    // Where a flush is allowed there is nothing to tell the two branches apart.
    return match handle.sync_all() {
        Ok(()) => None,
        Err(_) => Some(handle),
    };
}

/// The switch decides whether a write is flushed:
/// inside it the kernel is never asked,
/// and outside it the write meets the refusal,
/// so the control fails when either branch goes away.
#[test]
fn the_switch_decides_whether_a_write_is_flushed() {
    let Some(mut handle): Option<std::fs::File> = handle_that_cannot_be_flushed() else {
        eprintln!("skipped: this system has no handle that refuses a flush");
        return;
    };
    let bytes: &[u8] = b"a record that must arrive whole\n";
    let kept: std::io::Result<()> = {
        let _kept = WritesKeptInMemory::for_this_thread();
        write_all_then_sync(&mut handle, bytes)
    };
    assert!(kept.is_ok(), "the switch skipped the flush, so the refusal was never met: {kept:?}");
    let flushed: std::io::Result<()> = write_all_then_sync(&mut handle, bytes);
    assert!(flushed.is_err(), "the write made the flush and the kernel refused it: {flushed:?}");
}

/// What:
///  A platform that flushes,
/// with the flush policy inside the call written to `flushed`.
/// Why:
///   The switch belongs to the calling thread,
/// so a nested call on a platform that flushes has to turn the flushes back on;
/// the
///  platform records what a flush of `procfs`,
/// which refuses one,
/// answered there.
fn flushing(procfs: &'static Path, flushed: Rc<Cell<Option<bool>>>) -> Flushing {
    return Flushing {
        inner: PlainPlatform::new(),
        on_question: Box::new(move || {
            // A refused flush is the sign that the flush ran, so `is_err` says the policy in force was to flush.
            flushed.set(Some(sync_directory_to_disk(procfs).is_err()));
        }),
    };
}

/// A command on a platform that flushes keeps its flushes inside a call on a stand-in that skips them,
/// so a real-platform command run from a simulated one still flushes.
#[test]
fn a_command_on_a_platform_that_flushes_flushes_inside_one_that_skips() {
    let Some(procfs): Option<&'static Path> = directory_that_cannot_be_flushed() else {
        eprintln!("skipped: this system has no procfs, so no directory flush is refused");
        return;
    };
    let flushed: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));
    let platform: Flushing = flushing(procfs, Rc::clone(&flushed));
    let parent: PathBuf = fixture("flushing-command-inside-skipping");
    let root: PathBuf = parent.join("checkout");
    std::fs::create_dir(root.as_path()).expect("the checkout directory");
    let arguments: MigrateArguments = MigrateArguments { dry_run: true, ..migrate_arguments() };
    let (code, lines) = {
        // The extent of the stand-in that skips the flushes, as a control's simulated run has it.
        let _kept = WritesKeptInMemory::for_this_thread();
        run_command(&platform, &arguments, root.as_path(), &Interrupt::never())
    };
    assert_eq!(flushed.get(), Some(true), "the command on a platform that flushes made its flush, though it ran inside a stand-in that skips them (exit {code}, {lines:?})");
    remove(parent.as_path());
}

/// A conversion on a platform that flushes keeps its flushes inside a call on a stand-in that skips them,
/// which is the same extent as the command's.
#[test]
fn a_conversion_on_a_platform_that_flushes_flushes_inside_one_that_skips() {
    let Some(procfs): Option<&'static Path> = directory_that_cannot_be_flushed() else {
        eprintln!("skipped: this system has no procfs, so no directory flush is refused");
        return;
    };
    let flushed: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));
    let platform: Flushing = flushing(procfs, Rc::clone(&flushed));
    let (parent, root) = checkout_with_worktree("flushing-conversion-inside-skipping");
    let worktree: PathBuf = root.join("wt");
    let options: ConvertOptions = ConvertOptions { dry_run: true, abandon: false, command_line_attributes: false };
    let mut lines: Vec<String> = Vec::new();
    {
        // The extent of the stand-in that skips the flushes, as a control's simulated run has it.
        let _kept = WritesKeptInMemory::for_this_thread();
        let verdict = convert_worktree(&platform, worktree.as_path(), options, &Interrupt::never(), &mut |text: &str| { lines.push(String::from(text)); });
        assert!(verdict.is_ok(), "the conversion was previewed, so it reached the platform: {verdict:?} {lines:?}");
    }
    assert_eq!(flushed.get(), Some(true), "the conversion on a platform that flushes made its flush, though it ran inside a stand-in that skips them ({lines:?})");
    remove(parent.as_path());
}

/// What:
///  A symbolic link at the target's name is replaced,
///  and the file it points at keeps its bytes.
/// Why:
///   A plain write follows the link and truncates the target,
/// so a link planted at a list's name would let the command
///  overwrite a file of the person's own.
#[test]
fn a_link_at_the_targets_name_is_replaced_and_its_target_is_untouched() {
    let root: PathBuf = fixture("file-write-link");
    let notes: PathBuf = root.join("notes.txt");
    std::fs::write(notes.as_path(), b"a person's notes\n").expect("their file");
    let target: PathBuf = root.join("list.txt");
    std::os::unix::fs::symlink(notes.as_path(), target.as_path()).expect("a link at the target's name");
    write_beside_then_rename(target.as_path(), b"ignored/restored.txt\n").expect("written");
    assert_eq!(std::fs::read(notes.as_path()).expect("their file"), b"a person's notes\n", "the target of the link is untouched");
    assert!(!std::fs::symlink_metadata(target.as_path()).expect("the name").file_type().is_symlink(), "the name holds the bytes, not the link");
    assert_eq!(std::fs::read(target.as_path()).expect("the list"), b"ignored/restored.txt\n");
    assert_eq!(std::fs::read_dir(root.as_path()).expect("listing").flatten().count(), 2, "no temporary file stays");
    remove(root.as_path());
}

/// A write into a directory that cannot be listed as a parent is reported,
/// and nothing this call made stays.
#[test]
fn a_write_that_cannot_be_put_in_place_leaves_nothing_behind() {
    let root: PathBuf = fixture("file-write-missing-parent");
    let target: PathBuf = root.join("gone").join("list.txt");
    let error: std::io::Error = write_beside_then_rename(target.as_path(), b"x").expect_err("no such directory");
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound, "the failure is the missing directory");
    assert_eq!(std::fs::read_dir(root.as_path()).expect("listing").flatten().count(), 0, "nothing is left in the directory that exists");
    remove(root.as_path());
}
