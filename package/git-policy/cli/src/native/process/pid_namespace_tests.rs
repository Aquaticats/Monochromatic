//! Controls for the PID namespace rule.

use super::*;
use std::fs::File;

/// How far past or short of the patience a fixture record's age sits,
/// so no fixture's age depends on the value of the constant.
const AGE_BESIDE_THE_PATIENCE: Duration = Duration::from_secs(60);

/// The age of a fixture record that has outlived the patience,
/// stated once.
fn forever_past() -> Duration {
    return FOREIGN_OWNER_PATIENCE + AGE_BESIDE_THE_PATIENCE;
}

/// A scratch record file whose modification time is `age` in the past.
fn record_aged(directory: &Path, age: Duration) -> std::path::PathBuf {
    let path: std::path::PathBuf = directory.join("owner.json");
    let file: File = File::create(path.as_path()).expect("create the record");
    file.set_modified(SystemTime::now() - age).expect("age the record");
    return path;
}

/// A scratch record file whose modification time is `age` in the future,
/// which is what a clock that was set back,
/// a record copied with its time or another machine's clock leaves.
fn record_from_the_future(directory: &Path, age: Duration) -> std::path::PathBuf {
    let path: std::path::PathBuf = directory.join("owner-future.json");
    let file: File = File::create(path.as_path()).expect("create the record");
    file.set_modified(SystemTime::now() + age).expect("move the record's time ahead");
    return path;
}

/// The age of a record the clock can measure.
fn measured_age(path: &Path) -> Duration {
    let RecordAge::Measured(age) = foreign_record_age(path) else { panic!("{} has a measurable age: {:?}", path.display(), foreign_record_age(path)) };
    return age;
}

/// This process names its own namespace on Linux,
/// in the kernel's spelling.
#[test]
#[cfg(target_os = "linux")]
fn the_current_namespace_is_named_the_way_the_kernel_names_it() {
    let name: String = current_pid_namespace().expect("a Linux process has a PID namespace");
    assert!(name.starts_with("pid:["), "{name}");
    assert!(name.ends_with(']'), "{name}");
}

/// Only two known and different names make a namespace foreign.
#[test]
fn only_two_known_and_different_names_are_another_namespace() {
    assert!(is_other_namespace(Some("pid:[1]"), Some("pid:[2]")));
    assert!(!is_other_namespace(Some("pid:[1]"), Some("pid:[1]")));
    assert!(!is_other_namespace(None, Some("pid:[2]")), "an old record names no namespace");
    assert!(!is_other_namespace(Some("pid:[1]"), None), "a reader without a namespace cannot tell");
    assert!(!is_other_namespace(None, None));
}

/// A fresh record has not outlived the patience;
/// one past it has.
#[test]
fn a_foreign_record_expires_only_after_the_patience() {
    let directory: std::path::PathBuf = crate::test_support::fixture("pid-namespace-1");
    let fresh: std::path::PathBuf = record_aged(directory.as_path(), Duration::from_secs(60));
    assert!(!age_outlives_patience(measured_age(fresh.as_path())));
    let old: std::path::PathBuf = record_aged(directory.as_path(), forever_past());
    assert!(age_outlives_patience(measured_age(old.as_path())));
    let nearly: std::path::PathBuf = record_aged(directory.as_path(), FOREIGN_OWNER_PATIENCE - AGE_BESIDE_THE_PATIENCE);
    assert!(!age_outlives_patience(measured_age(nearly.as_path())), "just inside the patience stays alive");
}

/// A record whose time the clock cannot measure says so,
/// instead of reading as a record of no age at all,
/// and a record that is gone says it is gone:
/// its owner finished and removed it,
/// which is no unreadable record and no refusal.
/// Why:
///  An owner that completes its merge removes its intent between a reader's two reads,
///  and reading that absence as an unreadable record stopped the command with an instruction to remove a
///  file that no longer exists (cross-vendor review of 2026-11-09,
/// finding 3).
#[test]
fn a_record_with_no_measurable_time_says_which_case_it_is_and_a_gone_record_is_gone() {
    let directory: std::path::PathBuf = crate::test_support::fixture("pid-namespace-2");
    assert_eq!(
        foreign_record_age(directory.join("absent.json").as_path()),
        RecordAge::Gone,
        "a record that is not there was finished, not left unreadable"
    );
    assert_eq!(
        foreign_record_age(directory.join("absent.json").join("under-a-file.json").as_path()),
        RecordAge::Gone,
        "a path under a file that is not a directory names no record either"
    );
    // A link that names itself is a record the platform will not look at, and no permission is needed to make one.
    let looping: std::path::PathBuf = directory.join("looping.json");
    std::os::unix::fs::symlink(&looping, &looping).expect("a link that names itself");
    let age: RecordAge = foreign_record_age(looping.as_path());
    assert!(matches!(age, RecordAge::Unreadable(_)), "a record that cannot be looked at is unreadable, not gone: {age:?}");
    let future: std::path::PathBuf = record_from_the_future(directory.as_path(), Duration::from_secs(7200));
    assert_eq!(foreign_record_age(future.as_path()), RecordAge::Ahead, "a record from the future has no age");
}

/// A record of another PID namespace that is gone is finished:
/// its owner did the work and removed the record,
/// so nothing is refused and nothing is in the way.
/// Why:
///  A merge that completes removes its intent while a reader holds the parsed copy,
///  and a refusal there failed a command for a record that no longer exists
///  (cross-vendor review of 2026-11-09,
/// finding 3).
#[test]
#[cfg(target_os = "linux")]
fn a_foreign_record_that_is_gone_is_finished_and_not_refused() {
    let directory: std::path::PathBuf = crate::test_support::fixture("pid-namespace-finished");
    let absent: std::path::PathBuf = directory.join("owner.json");
    assert_eq!(foreign_owner_standing(Some("pid:[1]"), absent.as_path()), Some(ForeignStanding::Finished));
    let gone: std::path::PathBuf = record_aged(directory.as_path(), Duration::from_secs(60));
    std::fs::remove_file(gone.as_path()).expect("the owner removes its record");
    assert_eq!(foreign_owner_standing(Some("pid:[1]"), gone.as_path()), Some(ForeignStanding::Finished));
}

/// The shared rule:
/// unknown or same namespace leaves the decision to the PID;
/// another one answers by age.
#[test]
#[cfg(target_os = "linux")]
fn a_foreign_owner_is_alive_until_its_record_expires() {
    let directory: std::path::PathBuf = crate::test_support::fixture("pid-namespace-3");
    let fresh: std::path::PathBuf = record_aged(directory.as_path(), Duration::from_secs(60));
    let own: String = current_pid_namespace().expect("namespace");
    assert_eq!(foreign_owner_standing(Some(own.as_str()), fresh.as_path()), None);
    assert_eq!(foreign_owner_standing(None, fresh.as_path()), None);
    assert_eq!(foreign_owner_standing(Some("pid:[1]"), fresh.as_path()), Some(ForeignStanding::Standing));
    let old: std::path::PathBuf = record_aged(directory.as_path(), forever_past());
    // Neither alive nor dead (owner, 2026-11-04, "Refuse and name it (Recommended)"): the record is refused, with what a person needs.
    let Some(ForeignStanding::Refused(refused)) = foreign_owner_standing(Some("pid:[1]"), old.as_path()) else { panic!("an expired foreign record is refused") };
    let text: String = refused.describe(31337);
    let age_said: String = format!("{} minutes old", forever_past().as_secs() / 60);
    let patience_said: String = format!("{} minutes after which", FOREIGN_OWNER_PATIENCE.as_secs() / 60);
    assert!(text.contains("31337") && text.contains("pid:[1]") && text.contains(age_said.as_str()) && text.contains(patience_said.as_str()) && text.contains("rm -- "), "{text}");
    assert!(text.contains("another container") && text.contains("left running") && text.contains(old.to_string_lossy().as_ref()), "{text}");
    assert!(matches!(refused.because, ForeignRefusal::PastThePatience(_)), "the refusal names the measured age: {:?}", refused.because);
}

/// A record whose own time is later than this reader's clock has no age to be alive by:
/// taking it for a live owner's keeps it in the way for ever,
/// so it is refused like an expired one,
/// with the record's name,
/// why its time cannot be trusted,
/// and the `rm` (audit of 2026-11-09,
/// finding 2).
#[test]
#[cfg(target_os = "linux")]
fn a_foreign_record_whose_time_is_later_than_the_clock_is_refused_and_named() {
    let directory: std::path::PathBuf = crate::test_support::fixture("pid-namespace-ahead");
    let future: std::path::PathBuf = record_from_the_future(directory.as_path(), FOREIGN_OWNER_PATIENCE * 3);
    let Some(ForeignStanding::Refused(refused)) = foreign_owner_standing(Some("pid:[1]"), future.as_path()) else {
        panic!("a foreign record whose time is ahead of the clock is refused, not taken for a live owner's")
    };
    assert_eq!(refused.because, ForeignRefusal::TimeAhead);
    let text: String = refused.describe(31337);
    assert!(text.contains("31337") && text.contains("pid:[1]") && text.contains(future.to_string_lossy().as_ref()), "{text}");
    assert!(text.contains("later than this reader's clock") && text.contains("cannot be measured"), "{text}");
    assert!(text.contains("another container") && text.contains("left running") && text.contains("rm -- "), "{text}");
}

/// A record whose time cannot be read is refused the same way,
/// with the platform's reason,
/// and a record that is not there is no unreadable record:
/// its owner finished and removed it,
/// so nothing stops over it
/// (cross-vendor review of 2026-11-09,
/// finding 3).
#[test]
#[cfg(target_os = "linux")]
fn a_foreign_record_whose_time_cannot_be_read_is_refused_and_a_gone_one_is_finished() {
    let directory: std::path::PathBuf = crate::test_support::fixture("pid-namespace-unreadable");
    let looping: std::path::PathBuf = directory.join("looping.json");
    std::os::unix::fs::symlink(&looping, &looping).expect("a link that names itself");
    let Some(ForeignStanding::Refused(refused)) = foreign_owner_standing(Some("pid:[1]"), looping.as_path()) else {
        panic!("a record whose time cannot be read is refused")
    };
    assert!(matches!(refused.because, ForeignRefusal::TimeUnreadable(_)), "{:?}", refused.because);
    let text: String = refused.describe(1);
    assert!(text.contains("its time cannot be read") && text.contains("rm -- "), "{text}");
    let absent: std::path::PathBuf = directory.join("absent.json");
    assert_eq!(
        foreign_owner_standing(Some("pid:[1]"), absent.as_path()),
        Some(ForeignStanding::Finished),
        "a record that is not there is finished, not unreadable"
    );
}

/// Exactly the patience is still alive;
/// one nanosecond more is expired.
#[test]
fn the_patience_boundary_is_exclusive() {
    assert!(!age_outlives_patience(FOREIGN_OWNER_PATIENCE));
    assert!(age_outlives_patience(FOREIGN_OWNER_PATIENCE + Duration::from_nanos(1)));
    assert!(!age_outlives_patience(FOREIGN_OWNER_PATIENCE - Duration::from_nanos(1)));
}

/// The patience is derived from the merge's ceiling,
/// so the two cannot drift apart again:
/// a hold of another namespace's owner lasts
/// at most one wait for a foreign `index.lock` plus the work around it,
/// and the patience is twice that wait,
/// which is the two minutes the owner chose (2026-11-10,
/// "Two minutes (Recommended)").
#[test]
fn the_patience_is_twice_the_merge_ceiling() {
    assert_eq!(FOREIGN_OWNER_PATIENCE, 2 * crate::wait_ceilings::MERGE_LOCK_CEILING);
    assert_eq!(FOREIGN_OWNER_PATIENCE, Duration::from_secs(2 * 60), "the owner's two minutes");
}

/// The patience outlasts the longest hold of this population that was ever measured
/// (119,721 ms,
/// the commit service's hold of an index in its journal),
/// so a live owner's record is still not judged past the patience while its hold runs.
/// The longest whole command in the commit timing log (1,228,797 ms) is a command's length,
/// not the age of a record.
#[test]
fn the_patience_outlasts_the_longest_measured_hold() {
    const LONGEST_HOLD_MEASURED: Duration = Duration::from_millis(119_721);
    assert!(FOREIGN_OWNER_PATIENCE > LONGEST_HOLD_MEASURED);
}
