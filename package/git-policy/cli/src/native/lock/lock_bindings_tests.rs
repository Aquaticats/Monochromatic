//! What:
//!  Controls of the record a command writes about a lock its own Git took:
//!  the text,
//!  the pin that keeps the record from matching another lock,
//!  and the sweep that removes a record whose lock is gone.
//! Why:
//!  The record is the whole proof a later command acts on,
//!  so each part of it is tested where it can be wrong:
//!  a path of another shape must read back as itself,
//!  a record a person edited must decide nothing,
//!  a lock made again at the same path must not match the pin,
//!  and a record whose lock is gone must not stay.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeBinding({...}); expect(bindingOf(commonDir, lock)).toEqual({...});
//! ```

use crate::lock_bindings::{Binding, Written, folder_of, name_of, of, parse, remove, sweep, text_of, write};
use crate::test_support::{fixture, remove as remove_fixture};
use std::path::{Path, PathBuf};

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
    std::fs::write(lock.as_path(), b"a half-written copy of the index\n").expect("the lock");
    return (common_dir, lock);
}

/// What:
///  The device and inode of the file at `path`,
///  which is what a record's name carries.
/// Why:
///  A control names the record it wrote the way the writer did,
/// and a removal that names anything else takes away another record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = statSync(path); [file.dev, file.ino]
/// ```
fn file_of(path: &Path) -> (u64, u64) {
    let seen = crate::index_lock_evidence::read_lock_metadata(path).expect("the file's identity").expect("the file stands");
    return (seen.device, seen.inode);
}

/// What:
///  The identity a record carries for a holder,
///  as this boot and this process table spell one.
/// Why:
///   The parser must read the whole identity and nothing can be guessed of it.
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
///  the holder and the PID file it names.
/// Why:
///  This is the pair a later command acts on,
///  and the pin is what makes the record about one file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(commonDir, lock, [42, identity], [7, 9]); of(commonDir, lock) // { pid: 42, … }
/// ```
#[test]
fn a_record_reads_back_as_the_lock_it_names() {
    let (common_dir, lock) = a_lock("binding-round-trip", "index.lock");
    let holder: (i64, String) = (42, an_identity());
    assert!(write(common_dir.as_path(), lock.as_path(), file_of(lock.as_path()), &holder).is_some(), "a lock that stands can be pinned");
    let read: Binding = of(common_dir.as_path(), lock.as_path()).expect("the record written a moment ago");
    assert_eq!(read.lock, lock);
    assert_eq!(read.pid, 42);
    assert_eq!(read.identity, an_identity());
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
    assert!(write(common_dir.as_path(), lock.as_path(), file_of(lock.as_path()), &holder).is_some());
    std::fs::remove_file(lock.as_path()).expect("the lock goes");
    std::fs::write(lock.as_path(), b"another Git's lock\n").expect("a later lock");
    assert_eq!(of(common_dir.as_path(), lock.as_path()), None, "a record describes the file it pinned and no other");
    remove_fixture(common_dir.as_path());
}

/// What:
///  A record whose pin is gone describes nothing,
///  and the path a record names is the path it was written for.
/// Why:
///   The sweep and the pass both read records the other activity may have removed,
///  and a record of another path must not be read as this one's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(…); unlink(pin); of(commonDir, lock) // undefined
/// ```
#[test]
fn a_record_without_its_pin_describes_nothing() {
    let (common_dir, lock) = a_lock("binding-pin-gone", "index.lock");
    let holder: (i64, String) = (42, an_identity());
    assert!(write(common_dir.as_path(), lock.as_path(), file_of(lock.as_path()), &holder).is_some());
    let name: String = name_of(lock.as_path(), file_of(lock.as_path()));
    let pin: PathBuf = folder_of(common_dir.as_path()).join(format!("{name}.pin"));
    std::fs::remove_file(pin.as_path()).expect("the pin goes");
    assert_eq!(of(common_dir.as_path(), lock.as_path()), None, "a record without its pin proves nothing");
    remove_fixture(common_dir.as_path());
}

/// What:
///  A path of a space and a newline reads back as itself.
/// Why:
///   A record is read as lines,
///  and a file system holds names of any byte;
///  a path written raw would split the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parse(bindingText('a b\nc.lock'))!.lock === 'a b\nc.lock'
/// ```
#[test]
fn a_path_of_a_space_and_a_newline_reads_back_as_itself() {
    let lock: PathBuf = PathBuf::from("/a directory/a file\nwith a newline.lock");
    let text: String = text_of(lock.as_path(), &(42, an_identity()));
    let read: Binding = parse(text.as_str()).expect("the record this module writes");
    assert_eq!(read.lock, lock);
    assert_eq!(read.pid, 42);
}

/// What:
///  A record that is not one decides nothing.
/// Why:
///   A record a person edited,
///  a file cut short,
///  and a text of another shape must all keep the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parse('lock-holder-record v1\nlock 2f\n') // undefined
/// ```
#[test]
fn a_record_that_is_not_one_decides_nothing() {
    let good: String = text_of(Path::new("/repo/.git/index.lock"), &(42, an_identity()));
    assert!(parse(good.as_str()).is_some(), "the record this module writes reads back");
    for tampered in [
        String::new(),
        String::from("monochromatic-git-policy-cli lock-binding v1\n"),
        good.replace("holder 42", "holder forty-two"),
        format!("{good}extra\n"),
        good.replace("monochromatic-git-policy-cli lock-binding v1", "something else"),
    ] {
        assert_eq!(parse(tampered.as_str()), None, "a record that is not one names nobody: {tampered:?}");
    }
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
/// sweep(commonDir); expect(records).toEqual([keptName])
/// ```
#[test]
fn the_sweep_removes_the_record_of_a_lock_that_is_gone() {
    let (common_dir, standing) = a_lock("binding-sweep", "index.lock");
    let gone: PathBuf = common_dir.join("HEAD.lock");
    std::fs::write(gone.as_path(), b"a lock that never reached its rename\n").expect("the lock");
    let holder: (i64, String) = (42, an_identity());
    let standing_record: Written = write(common_dir.as_path(), standing.as_path(), file_of(standing.as_path()), &holder).expect("the standing lock's record");
    assert!(write(common_dir.as_path(), gone.as_path(), file_of(gone.as_path()), &holder).is_some());
    std::fs::remove_file(gone.as_path()).expect("the lock it recorded goes");
    sweep(common_dir.as_path());
    assert!(of(common_dir.as_path(), standing.as_path()).is_some(), "the record of a standing lock stays");
    assert_eq!(of(common_dir.as_path(), gone.as_path()), None, "the record of a lock that is gone goes");
    remove(common_dir.as_path(), &standing_record);
    assert_eq!(of(common_dir.as_path(), standing.as_path()), None, "a record removed by name is gone");
    remove_fixture(common_dir.as_path());
}
