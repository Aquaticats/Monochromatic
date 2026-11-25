//! The watch rebinding with fixture directories:
//! when a watch from before still sees the tree,
//! what a watch on the covered directory really sees,
//! and the sentence a person reads.
//!
//! A mount cannot be made without privilege, so a control stands one in with the
//! move that has the same shape for a watch:
//! the watched directory is moved aside and another one takes its path,
//! which is what a mount over the slot does to the directory object the path
//! names.

use super::*;
use crate::test_support::{fixture, remove};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How long a control waits for one inotify event before it calls the wait a silence.
///  Short, because the write it waits for follows at once and the wait only has to
/// outlast the kernel's own delivery.
const PATIENCE: Duration = Duration::from_millis(500);

/// The name a control writes through the slot's path,
/// so an event for it is the event the control is looking for.
const WRITTEN: &str = "written-through-the-slot";

/// What:
///  An inotify watch on one directory's new entries, closed however the control ends.
/// Why:
///  The controls need the one answer `landing/service_index_watch.rs` does not give
///  (whether the event arrived, rather than whether to look again),
///  and a descriptor held across an assertion that fails must still be closed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const probe = Probe.on(directory); // closed in the finalizer
/// ```
struct Probe {
    /// The inotify descriptor this probe owns.
    watch: std::ffi::c_int,
}

impl Probe {
    /// What:
    ///  A watch for new entries in `directory`.
    /// Why:
    ///  Only creations are asked for:
    /// the control's own write is the one event it wants,
    /// and the end of a watch on a directory moved aside reports nothing here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function on(directory: string): Probe;
    /// ```
    fn on(directory: &Path) -> Probe {
        let named: std::ffi::CString = std::ffi::CString::new(directory.as_os_str().as_bytes()).expect("a path without NUL");
        // SAFETY: the descriptor is created here and closed by `Drop`, whatever the waits answered.
        let watch: std::ffi::c_int = unsafe { libc::inotify_init1(libc::IN_CLOEXEC) };
        assert!(watch >= 0, "inotify_init1 refused: {}", std::io::Error::last_os_error());
        // SAFETY: `named` is a NUL-terminated name of a directory this fixture holds.
        let added: std::ffi::c_int = unsafe { libc::inotify_add_watch(watch, named.as_ptr(), libc::IN_CREATE) };
        let probe: Probe = Probe { watch };
        assert!(added >= 0, "inotify_add_watch refused for {}: {}", directory.display(), std::io::Error::last_os_error());
        return probe;
    }

    /// What:
    ///  Whether an event naming the control's file arrives inside `PATIENCE`.
    /// Why:
    ///  The wait is bounded, and a silence is one answer rather than a failure of
    ///  this process:
    /// the control that expects a silence proves its watch works by the same call
    /// answering true on the tree the path names now.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// probe.sees(writtenName, patience): boolean
    /// ```
    fn sees(&self) -> bool {
        let deadline: Instant = Instant::now() + PATIENCE;
        loop {
            let left: Duration = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            let mut entry: libc::pollfd = libc::pollfd { fd: self.watch, events: libc::POLLIN, revents: 0 };
            let milliseconds: std::ffi::c_int = std::ffi::c_int::try_from(left.as_millis()).unwrap_or(std::ffi::c_int::MAX).max(1);
            // SAFETY: `entry` names this probe's one descriptor and lives for the call.
            if unsafe { libc::poll(&raw mut entry, 1, milliseconds) } <= 0 {
                return false;
            }
            let mut buffer: [u8; 4096] = [0; 4096];
            // SAFETY: the buffer is valid for its length.
            let read: isize = unsafe { libc::read(self.watch, buffer.as_mut_ptr().cast(), buffer.len()) };
            let Ok(count) = usize::try_from(read) else { return false };
            if count == 0 {
                return false;
            }
            if names_the_file(&buffer[..count], OsStr::new(WRITTEN).as_bytes()) {
                return true;
            }
        }
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        // SAFETY: the descriptor belongs to this probe and is closed once.
        let closed: std::ffi::c_int = unsafe { libc::close(self.watch) };
        assert_eq!(closed, 0, "the watch closes: {}", std::io::Error::last_os_error());
    }
}

/// What:
///  Whether `buffer` holds an inotify event naming `wanted`.
/// Why:
///  The controls must tell an event for their own file from the end of a watch,
///  and `events_name` answers both as one,
///  so the name is compared here and nothing else is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = (buffer) => events(buffer).map((event) => event.name);
/// ```
fn names_the_file(buffer: &[u8], wanted: &[u8]) -> bool {
    let mut rest: &[u8] = buffer;
    while rest.len() >= 16 {
        let length: usize = usize::try_from(u32::from_ne_bytes([rest[12], rest[13], rest[14], rest[15]])).expect("a length");
        let payload: &[u8] = &rest[16..];
        if payload.len() < length {
            return false;
        }
        let name: &[u8] = payload[..length].split(|byte| return *byte == 0).next().unwrap_or(&[]);
        if name == wanted {
            return true;
        }
        rest = &rest[16 + length..];
    }
    return false;
}

/// What:
///  A fixture root with an empty slot and the directory that will take its path.
/// Why:
///  A directory can be renamed over only an empty one,
///  so the slot starts empty and the standing-in tree holds what the mount would
///  bring.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = fixture(name); mkdir(join(root, 'slot')); mkdir(join(root, 'mounted'));
/// ```
fn scene(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let slot: PathBuf = root.join("slot");
    let mounted: PathBuf = root.join("mounted");
    std::fs::create_dir(slot.as_path()).expect("the slot");
    std::fs::create_dir(mounted.as_path()).expect("the tree that takes the slot's path");
    return (root, slot, mounted);
}

/// The write a control makes through the slot's path.
fn write_through_slot(slot: &Path) {
    std::fs::write(slot.join(WRITTEN), b"a write in the tree the path names\n").expect("the write");
}

/// What:
///  A slot whose tree was not replaced leaves the watches as they were,
///  and no sentence is written.
/// Why:
///   An add that mounted nothing is not a reason to ask a person to reopen an
///  editor,
/// and a line that appeared anyway would teach readers to ignore it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = theObjectThePathShows(slot); const after = afterTheMount(slot, before);
/// assert.equal(after.watches, 'still-the-same-tree'); assert.equal(after.line, undefined);
/// ```
#[test]
fn a_path_that_still_names_its_object_keeps_every_watch() {
    let (root, slot, _mounted) = scene("watch-rebinding-same");
    let before: ObjectId = the_object_the_path_shows(slot.as_path()).expect("the slot's identity");
    let after: AfterTheMount = after_the_mount(slot.as_path(), before).expect("the answer");
    assert_eq!(after.watches, Watches::StillTheSameTree);
    assert_eq!(after.line, None);
    remove(&root);
}

/// What:
///  A slot whose tree was replaced names another object,
///  and the sentence names the slot and the two programs to restart.
/// Why:
///   The comparison is by object:
/// an inode number alone is reused,
/// and a mount over the slot is one path naming a second object.
/// The sentence is what a person acts on,
/// so it must name the path and both watchers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = theObjectThePathShows(slot); rename(mounted, slot);
/// const after = afterTheMount(slot, before); assert.equal(after.watches, 'blind'); assert.match(after.line, /fsmonitor--daemon/);
/// ```
#[test]
fn a_path_that_names_another_object_makes_every_watch_blind() {
    let (root, slot, mounted) = scene("watch-rebinding-blind-line");
    let before: ObjectId = the_object_the_path_shows(slot.as_path()).expect("the slot's identity");
    std::fs::rename(mounted.as_path(), slot.as_path()).expect("the tree takes the slot's path");
    let after: AfterTheMount = after_the_mount(slot.as_path(), before).expect("the answer");
    assert_eq!(after.watches, Watches::Blind);
    let line: String = after.line.expect("a sentence for the person");
    assert!(line.contains(slot.to_str().expect("utf8")), "{line}");
    assert!(line.contains("editor"), "{line}");
    assert!(line.contains("git fsmonitor--daemon status") && line.contains("git fsmonitor--daemon stop"), "{line}");
    remove(&root);
}

/// What:
///  A watch set on the slot before its tree was replaced sees nothing written
///  through the slot afterwards,
///  and one set after it sees the same write.
/// Why:
///   This is the premise the sentence rests on,
/// and the second arm is the positive control:
/// a watch on the tree the path names now does see a file written there,
/// so the first arm's silence is the covered directory and not a probe that
/// cannot see anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const covered = Probe.on(slot); rename(mounted, slot); writeThroughSlot(slot);
/// assert.equal(covered.sees(), false); assert.equal(Probe.on(slot).seesAfter(write), true);
/// ```
#[test]
fn a_watch_from_before_the_replacement_is_blind_and_one_set_after_it_sees_the_write() {
    let (root, slot, mounted) = scene("watch-rebinding-premise");
    // An editor that opened the slot holds this watch, and the mount covers the directory it was set on.
    let covered: Probe = Probe::on(slot.as_path());
    std::fs::rename(mounted.as_path(), slot.as_path()).expect("the tree takes the slot's path");
    write_through_slot(slot.as_path());
    let covered_sees: bool = covered.sees();
    assert!(!covered_sees, "a watch set on the covered directory saw a write in the tree the path names");
    drop(covered);
    // The positive control: the same probe, the same write, on the tree the path names now.
    let _gone: std::io::Result<()> = std::fs::remove_file(slot.join(WRITTEN));
    let fresh: Probe = Probe::on(slot.as_path());
    write_through_slot(slot.as_path());
    assert!(fresh.sees(), "a watch set on the tree the path names must see a file written there");
    remove(&root);
}

/// What:
///  A slot that is not a directory is refused.
/// Why:
///   The wiring step runs after a mount it believes in,
/// and a path that is not a directory there is a fact to report and not a
/// panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFile(join(root, 'a-file'), ''); assert(theObjectThePathShows(join(root, 'a-file')) throws);
/// ```
#[test]
fn a_slot_that_is_not_a_directory_is_an_error() {
    let (root, _slot, _mounted) = scene("watch-rebinding-not-a-directory");
    let file: PathBuf = root.join("a-file");
    std::fs::write(file.as_path(), b"not a directory\n").expect("a file at the path");
    assert!(the_object_the_path_shows(file.as_path()).is_err(), "a plain file is not a slot");
    assert!(the_object_the_path_shows(root.join("gone").as_path()).is_err(), "a missing path is not a slot");
    remove(&root);
}
