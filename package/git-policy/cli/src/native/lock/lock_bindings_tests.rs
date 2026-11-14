//! What:
//!  Controls of the record a command writes about a lock its own Git took:
//!  the one file that is the whole record,
//!  the inode it pins,
//!  the sweep that removes a record whose lock is gone,
//!  and the sweep that runs beside a write.
//! Why:
//!  The record is the whole proof a later command acts on,
//!  so each part of it is tested where it can be wrong:
//!  a path of another shape must read back as itself,
//!  a record a person edited must decide nothing,
//!  a lock made again at the same path must not match the pin,
//!  a record whose lock is gone must not stay,
//!  and a sweep that lands while a record is being written must leave it
//!  (a review of `2ac27d2a29`, finding 1).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeBinding({...}); expect(bindingOf(commonDir, lock)).toEqual({...});
//! ```

use crate::lock_bindings::{A_PIN_IS_REACHABLE, Binding, Written, folder_of, of, pin_holds_the_lock, remove, sweep, write};
use crate::test_support::{fixture, remove as remove_fixture};
use std::path::PathBuf;

/// What:
///  A directory shaped like a Git directory,
///  with a lock file in it and the record folder the wrapper writes.
/// Why:
///   A record is written under the common Git directory about a file beside it,
///  so the two have to be made together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commonDir = fixture('binding'); const lock = join(commonDir, 'index.lock'); writeFileSync(lock, '');
/// ```
fn a_lock(name: &str, lock_name: &str) -> (PathBuf, PathBuf) {
    let common_dir: PathBuf = fixture(name);
    std::fs::create_dir_all(folder_of(common_dir.as_path())).expect("the record folder");
    let lock: PathBuf = common_dir.join(lock_name);
    std::fs::create_dir_all(lock.parent().expect("the lock's folder")).expect("the lock's folder");
    std::fs::write(lock.as_path(), b"a half-written copy of the index\n").expect("the lock");
    return (common_dir, lock);
}

/// What:
///  The identity a record carries for a holder,
///  as this boot and this process table spell one.
/// Why:
///   The record's own file name carries it whole,
///  so the reader must find it again unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// 'linux:8423337#a-boot#4026531836#a-machine'
/// ```
fn an_identity() -> String {
    return String::from("linux:8423337#a-boot#4026531836#a-machine");
}

/// What:
///  A record written for a lock reads back as the lock,
///  the holder and the file it pinned.
/// Why:
///  This is what a later command acts on,
///  and the pin is what makes the record about one file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(commonDir, lock, [42, identity]); of(commonDir, lock) // { pid: 42, … }
/// ```
#[test]
fn a_record_reads_back_as_the_lock_it_names() {
    let (common_dir, lock) = a_lock("binding-round-trip", "index.lock");
    let holder: (i64, String) = (42, an_identity());
    let written: Written = write(common_dir.as_path(), lock.as_path(), &holder).expect("a lock that stands can be pinned");
    let read: Binding = of(common_dir.as_path(), lock.as_path()).expect("the record written a moment ago");
    assert_eq!(read.lock, lock);
    assert_eq!(read.pid, 42);
    assert_eq!(read.identity, an_identity());
    assert_eq!(read.pin, written.pin, "the record the reader found is the one this command wrote");
    assert!(pin_holds_the_lock(&written), "the pin names the file at the path");
    remove_fixture(common_dir.as_path());
}

/// What:
///  A lock that is removed and made again at the same path
///  is not the file the record pinned.
/// Why:
///   This is the sequence the whole pin exists for:
///  a dead holder's record must never describe a later Git's lock,
///  and on a file system that hands a freed inode to a new file a number alone would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(…); rm(lock); writeFileSync(lock, ''); of(commonDir, lock) // undefined
/// ```
#[test]
fn a_lock_made_again_at_the_path_is_not_the_pinned_file() {
    let (common_dir, lock) = a_lock("binding-replaced", "index.lock");
    let holder: (i64, String) = (42, an_identity());
    let written: Written = write(common_dir.as_path(), lock.as_path(), &holder).expect("a lock that stands is pinned");
    std::fs::remove_file(lock.as_path()).expect("the lock goes");
    std::fs::write(lock.as_path(), b"another Git's lock\n").expect("a later lock");
    assert_eq!(of(common_dir.as_path(), lock.as_path()), None, "a record describes the file it pinned and no other");
    assert!(!pin_holds_the_lock(&written), "the pin names a file that is no longer at the path");
    remove_fixture(common_dir.as_path());
}

/// What:
///  A record whose pin is gone describes nothing,
///  and the path a record names is the path it was written for.
/// Why:
///   The record is one file,
///  so removing it is removing the proof:
///  nothing may read a name without it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(…); unlink(pin); of(commonDir, lock) // undefined
/// ```
#[test]
fn a_record_without_its_pin_describes_nothing() {
    let (common_dir, lock) = a_lock("binding-pin-gone", "index.lock");
    let holder: (i64, String) = (42, an_identity());
    let written: Written = write(common_dir.as_path(), lock.as_path(), &holder).expect("a lock that stands is pinned");
    std::fs::remove_file(written.pin.as_path()).expect("the record goes");
    assert_eq!(of(common_dir.as_path(), lock.as_path()), None, "a record without its file proves nothing");
    remove(&written);
    assert_eq!(of(common_dir.as_path(), lock.as_path()), None, "a record removed by name is gone");
    remove_fixture(common_dir.as_path());
}

/// What:
///  A path of a space and a newline reads back as itself.
/// Why:
///   The path's own text is what the sweep judges a record by,
///  and a file system holds names of any byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = join(commonDir, 'a file\nwith a newline.lock'); write(…); of(…)!.lock === lock
/// ```
#[test]
fn a_path_of_a_space_and_a_newline_reads_back_as_itself() {
    let (common_dir, lock) = a_lock("binding-odd-path", "a file\nwith a newline.lock");
    let holder: (i64, String) = (42, an_identity());
    let written: Written = write(common_dir.as_path(), lock.as_path(), &holder).expect("a lock of any name can be pinned");
    let read: Binding = of(common_dir.as_path(), lock.as_path()).expect("the record of that path");
    assert_eq!(read.lock, lock);
    assert!(pin_holds_the_lock(&written));
    // A sweep must keep it: its lock stands.
    sweep(common_dir.as_path());
    assert!(of(common_dir.as_path(), lock.as_path()).is_some(), "a sweep keeps the record of a lock that stands");
    remove_fixture(common_dir.as_path());
}

/// What:
///  A file that is not a record of this release decides nothing,
///  and the sweep removes it.
/// Why:
///   A record a person edited,
///  a file cut short,
///  a name of another shape,
///  and the two files of the earlier format of this folder
///  are all litter no reader reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(join(pinned, '3f9a….pin'), ''); sweep(commonDir); readdirSync(pinned) // []
/// ```
#[test]
fn a_file_that_is_not_a_record_decides_nothing_and_goes() {
    let (common_dir, lock) = a_lock("binding-not-a-record", "index.lock");
    let folder: PathBuf = folder_of(common_dir.as_path());
    for name in ["a-record-of-another-shape.holder", "3f9a.pin", "4154.not-a-take", "no-suffix-here"] {
        std::fs::write(folder.join(name), b"not a record\n").expect("a planted file");
    }
    assert_eq!(of(common_dir.as_path(), lock.as_path()), None, "none of them names this lock");
    sweep(common_dir.as_path());
    let left: Vec<String> = std::fs::read_dir(folder.as_path())
        .expect("the record folder")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return !name.ends_with(".lock-path"); })
        .collect();
    remove_fixture(common_dir.as_path());
    assert!(left.is_empty(), "the sweep removes every file that is not a record of a path: {left:?}");
}

/// What:
///  The sweep removes the record of a lock that is gone,
///  and keeps the record of the lock that still stands.
/// Why:
///  A record holds the inode of the file it pinned,
///  so one that is kept after its lock went holds a file nothing names,
///  and one that is removed early takes the proof of a standing lock with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sweep(commonDir); expect(readdirSync(pinned)).toEqual([kept])
/// ```
#[test]
fn the_sweep_removes_the_record_of_a_lock_that_is_gone() {
    let (common_dir, standing) = a_lock("binding-sweep", "index.lock");
    let gone: PathBuf = common_dir.join("HEAD.lock");
    std::fs::write(gone.as_path(), b"a lock that never reached its rename\n").expect("the lock");
    let holder: (i64, String) = (42, an_identity());
    let kept: Written = write(common_dir.as_path(), standing.as_path(), &holder).expect("the standing lock's record");
    let litter: Written = write(common_dir.as_path(), gone.as_path(), &holder).expect("the other lock's record");
    std::fs::remove_file(gone.as_path()).expect("the lock it recorded goes");
    sweep(common_dir.as_path());
    assert!(of(common_dir.as_path(), standing.as_path()).is_some(), "the record of a standing lock stays");
    assert_eq!(of(common_dir.as_path(), gone.as_path()), None, "the record of a lock that is gone goes");
    assert!(!litter.pin.exists(), "and its own file is unlinked");
    assert!(kept.pin.exists(), "the standing lock's record file is still there");
    remove_fixture(common_dir.as_path());
}

/// What:
///  A record whose path cannot be read is kept by the sweep.
/// Why:
///   Nothing shows which file such a pin stands for,
///  and a record kept is a clearance that may still be made;
///  removing it would take away a proof no command could then make again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(…); unlink(lockPathFile); sweep(commonDir); expect(pinExists()).toBe(true)
/// ```
#[test]
fn a_record_whose_path_cannot_be_read_is_kept() {
    let (common_dir, lock) = a_lock("binding-unreadable-path", "index.lock");
    let holder: (i64, String) = (42, an_identity());
    let written: Written = write(common_dir.as_path(), lock.as_path(), &holder).expect("a lock that stands is pinned");
    let folder: PathBuf = folder_of(common_dir.as_path());
    for entry in std::fs::read_dir(folder.as_path()).expect("the record folder").flatten() {
        if entry.file_name().to_string_lossy().ends_with(".lock-path") {
            std::fs::remove_file(entry.path()).expect("the path's text goes");
        }
    }
    sweep(common_dir.as_path());
    assert!(written.pin.exists(), "a pin whose path's text cannot be read is kept");
    remove_fixture(common_dir.as_path());
}

/// What:
///  Another command's sweep, run at the instant a record's pin becomes reachable,
///  leaves that record where it is:
///  the lock it pins stands,
///  so the record is the proof that clears it if the Git holding it is killed.
/// Why:
///  This is the interleaving a review of `2ac27d2a29` reproduced with a preload that widened only the
///  window between the pin and the record
///  (`sweep-race-reproduction.ts`):
///  the sweep listed the folder while the pin was there and the record was not,
///  unlinked the pin it could not explain,
///  and the writer then published a record with no pin.
///  A Git killed holding that lock then left a lock no command on this volume could clear
///  (its birth and the PID file's carry one stamp),
///  so the next `git add` and `git commit` failed.
///  The order is forced here by the code's own fault point `A_PIN_IS_REACHABLE`
///  and by nothing else, and the sweep is the production one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// at(APinIsReachable, () => sweep(common)); write(common, lock, holder);
/// expect(bindingOf(common, lock)).toBeDefined();
/// ```
#[test]
fn a_sweep_beside_a_write_leaves_the_record_of_a_standing_lock() {
    let (common_dir, lock) = a_lock("binding-sweep-beside-a-write", "index.lock");
    let holder: (i64, String) = (42, an_identity());
    let swept_at: std::rc::Rc<std::cell::Cell<bool>> = std::rc::Rc::new(std::cell::Cell::new(false));
    let hook_common: PathBuf = common_dir.clone();
    let ran: std::rc::Rc<std::cell::Cell<bool>> = swept_at.clone();
    crate::service_fault::hooks::set(
        A_PIN_IS_REACHABLE,
        std::rc::Rc::new(move || {
            // The other command's pass, at the instant this record's pin is reachable.
            sweep(hook_common.as_path());
            ran.set(true);
        }),
    );
    let written: Option<Written> = write(common_dir.as_path(), lock.as_path(), &holder);
    crate::service_fault::hooks::clear();
    assert!(swept_at.get(), "the control reached the instant the pin became reachable, and the sweep ran there");
    assert!(written.is_some(), "the record is written");
    let found: Option<Binding> = of(common_dir.as_path(), lock.as_path());
    // A later command's pass runs the sweep again over the same state.
    sweep(common_dir.as_path());
    let after: Option<Binding> = of(common_dir.as_path(), lock.as_path());
    remove_fixture(common_dir.as_path());
    assert!(
        found.is_some(),
        "a sweep that ran while the record was being published leaves the record of a standing lock: {found:?}"
    );
    assert!(after.is_some(), "and so does the next one: {after:?}");
}

/// What:
///  A Git that renames its lock over its target and takes the path again
///  leaves a record that names the file standing there,
///  and one sweep keeps it.
/// Why:
///   One take follows another at one path,
///  and the record of the first names a file that is now the target,
///  so the second take has to be written again or a killed Git's second lock would have no proof
///  (the control in `lock_watch_tests.rs` reaches the same through the run that writes records).
///  The name of a record is the take's, so the second write takes the first one's place:
///  one file per take, and the sweeping judgement is about the file that stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(…); rename(lock, index); writeFileSync(lock); write(…); expect(of(…).pin).toBeDefined()
/// ```
#[test]
fn a_second_take_of_one_path_leaves_a_record_of_the_file_that_stands() {
    let (common_dir, lock) = a_lock("binding-second-take", "index.lock");
    let holder: (i64, String) = (42, an_identity());
    let _first: Written = write(common_dir.as_path(), lock.as_path(), &holder).expect("the first take is recorded");
    std::fs::rename(lock.as_path(), common_dir.join("index")).expect("the child renames its lock over its target");
    std::fs::write(lock.as_path(), b"the second take\n").expect("the path is taken again");
    let second: Written = write(common_dir.as_path(), lock.as_path(), &holder).expect("the second take is recorded");
    assert!(pin_holds_the_lock(&second), "the record names the file standing at the path");
    let read: Binding = of(common_dir.as_path(), lock.as_path()).expect("the standing lock's record");
    assert_eq!(read.pin, second.pin, "the record read is the one that names the standing file");
    sweep(common_dir.as_path());
    assert!(of(common_dir.as_path(), lock.as_path()).is_some(), "a sweep keeps the standing take's record");
    remove_fixture(common_dir.as_path());
}
