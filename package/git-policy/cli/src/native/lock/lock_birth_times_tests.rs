//! What:
//!  Controls of the birth-time arm and of the gate that reads a PID file's birth:
//!  the times this file system records,
//!  none at all,
//!  and one tick shared by both files.
//! Why:
//!  A file system may record no birth time,
//!  or record it in ticks of a second,
//!  and the review of `d189162e3e` found the equal case removing a live Git's editor-phase lock
//!  at this volume's one-tick resolution.
//!  The gate must read an equal or missing time as "the order is not proven",
//!  which a caller that would remove a lock takes as "keep it",
//!  and only a PID file born strictly later as this lock's holder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(pidFileStanding(equal, equal, 7)).toEqual({ kind: 'unproven', pid: 7 });
//! withBirthTimes({ kind: 'coarse', tickMs: 2000 }, () => expect(birthTimeOf(path)).toEqual(floored));
//! ```

use crate::lock_birth_times::{BirthTimes, birth_time_of, birth_times_of, with_birth_times};
use crate::stale_ref_lock_found::PidFile;
use crate::stale_ref_lock_judge::pid_file_standing;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A time a whole number of seconds after the epoch,
/// so two of them differ by exactly what a control means.
fn at(seconds: u64) -> SystemTime {
    return UNIX_EPOCH + Duration::from_secs(seconds);
}

/// What:
///  An equal birth time is unproven,
/// never "later" and never "earlier".
/// Why:
///  This is the state a file system with coarse times gives two files written moments apart,
///  and the state the review met on this volume when it planted a PID file beside a live lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(pidFileStanding(at(10), at(10), 7)).toEqual({ kind: 'unproven' });
/// ```
#[test]
fn an_equal_birth_time_proves_no_order() {
    let standing: PidFile = pid_file_standing(Some(at(10)), Some(at(10)), 7);
    assert_eq!(standing, PidFile::Unproven(7), "equal times prove neither order");
}

/// What:
///  A birth time the file system does not record on either file is unproven.
/// Why:
///  A file system may answer none at all,
///  and then nothing shows which file came first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(pidFileStanding(undefined, undefined, 7)).toEqual({ kind: 'unproven' });
/// ```
#[test]
fn a_missing_birth_time_proves_no_order() {
    assert_eq!(pid_file_standing(None, None, 7), PidFile::Unproven(7), "no times prove no order");
    assert_eq!(pid_file_standing(Some(at(10)), None, 7), PidFile::Unproven(7), "a PID file without a time proves no order");
    assert_eq!(pid_file_standing(None, Some(at(10)), 7), PidFile::Unproven(7), "a lock without a time proves no order");
}

/// What:
///  Only a strictly later PID file names the lock's holder,
///  and only a strictly earlier one is an earlier holder's.
/// Why:
///  Git creates the PID file after the lock,
///  so these two orders are the ones that do mean something.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(pidFileStanding(at(10), at(11), 7)).toEqual({ kind: 'names' });
/// expect(pidFileStanding(at(10), at(9), 7)).toEqual({ kind: 'older' });
/// ```
#[test]
fn a_proven_order_names_the_holder_or_an_earlier_one() {
    assert_eq!(pid_file_standing(Some(at(10)), Some(at(11)), 7), PidFile::Names(7), "a later PID file is this lock's");
    assert_eq!(pid_file_standing(Some(at(10)), Some(at(9)), 7), PidFile::Older(7), "an earlier PID file is an earlier holder's");
}

/// What:
///  A setting the control's variable carries names the arm it means,
///  and a setting that means nothing leaves the recorded times.
/// Why:
///  A misspelt variable must not change what production reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(birthTimesOf('none')).toEqual({ kind: 'hidden' });
/// expect(birthTimesOf('nonsense')).toBeUndefined();
/// ```
#[test]
fn a_setting_names_the_arm_or_nothing() {
    assert_eq!(birth_times_of("none"), Some(BirthTimes::Hidden));
    assert_eq!(birth_times_of(" 2000 "), Some(BirthTimes::Coarse(Duration::from_millis(2000))));
    assert_eq!(birth_times_of("0"), None, "a tick of zero is not a tick");
    assert_eq!(birth_times_of("nonsense"), None);
    assert_eq!(birth_times_of(""), None);
}

/// What:
///  Under the hidden arm no file has a birth time,
///  and under a coarse one two files written moments apart share theirs while
///  two that lie in different ticks keep their order.
/// Why:
///  The arm is what lets a control drive the gate as a file system of another granularity would,
///  and it must not change anything else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withBirthTimes({ kind: 'hidden' }, () => expect(birthTimeOf(path)).toBeUndefined());
/// ```
#[test]
fn the_hidden_and_coarse_arms_report_what_other_file_systems_do() {
    let root: PathBuf = fixture("birth-time-arm");
    let first: PathBuf = root.join("first");
    std::fs::write(first.as_path(), b"one\n").expect("a file");
    assert!(birth_time_of(first.as_path()).is_some(), "this file system records a birth time, so the recorded arm has one");
    let hidden: Option<SystemTime> = with_birth_times(BirthTimes::Hidden, || { return birth_time_of(first.as_path()); });
    assert_eq!(hidden, None, "the hidden arm reports no birth time at all");
    let coarse: Option<SystemTime> = with_birth_times(BirthTimes::Coarse(Duration::from_secs(3600)), || { return birth_time_of(first.as_path()); });
    let floored: u64 = coarse.expect("the coarse arm still reports a time").duration_since(UNIX_EPOCH).expect("after the epoch").as_secs();
    assert_eq!(floored % 3600, 0, "the coarse arm floors the time to its tick");
    // A second file written in the same tick as the first carries the same time, which is what the coarse file system gives.
    let second: PathBuf = root.join("second");
    std::fs::write(second.as_path(), b"two\n").expect("a second file");
    let both: (Option<SystemTime>, Option<SystemTime>) = with_birth_times(BirthTimes::Coarse(Duration::from_secs(3600)), || {
        return (birth_time_of(first.as_path()), birth_time_of(second.as_path()));
    });
    assert_eq!(both.0, both.1, "two files in one tick carry one time");
    remove(root.as_path());
}
/// What:
///  Write the PID file beside `lock`,
/// again if need be,
///  until its birth is strictly after the lock's.
/// Why:
///  A control that means to exercise the removal must give the gate a proven order,
///  and this volume's birth times give two files written microseconds apart one tick most of the time
///  (measured:
/// `index.lock` and `index~pid.lock` of one Git carry the same stamp).
///  Writing the file is the step that has to wait,
///  because a birth time is the file's and never moves:
///  the file is made again after a pause until the two stamps differ.
///  A file system that records no birth time never proves it,
///  and the control then fails naming what the file system said rather than passing without testing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writePidFileProvablyLater(lock, 'pid 42\n') // throws when the file system cannot show the order
/// ```
pub(crate) fn write_after_the_lock(lock: &Path, content: &str) {
    let pid_file: std::path::PathBuf = crate::stale_ref_lock_walk::pid_file_beside(lock);
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        let _ = std::fs::remove_file(pid_file.as_path());
        std::thread::sleep(std::time::Duration::from_millis(2));
        std::fs::write(pid_file.as_path(), content.as_bytes()).expect("the PID file");
        let lock_born: Option<SystemTime> = birth_time_of(lock);
        let pid_born: Option<SystemTime> = birth_time_of(pid_file.as_path());
        if let (Some(lock_at), Some(pid_at)) = (lock_born, pid_born)
            && pid_at > lock_at
        {
            return;
        }
        assert!(
            began.elapsed() < std::time::Duration::from_secs(5),
            "this file system cannot show that {} was born after {}: the lock's birth is {:?} and the PID file's {:?}, so the control cannot test a removal here",
            pid_file.display(),
            lock.display(),
            lock_born,
            pid_born
        );
    }
}
