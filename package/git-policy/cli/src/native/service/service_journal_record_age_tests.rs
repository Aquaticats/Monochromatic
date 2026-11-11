//! Controls for how old a finished batch's record is,
//! as the reading of the clocks it carries says,
//! and as its modification time says where it carries none.

use super::*;
use std::time::Duration;

/// The boot and the machine the readings of a control are taken in,
/// and the ones of another boot of the same machine and of another machine,
/// each 32 hexadecimal digits as the kernel and the host write them.
const BOOT: &str = "0000000000000000000000000000000b";
const ANOTHER_BOOT: &str = "0000000000000000000000000000000c";
const MACHINE: &str = "0000000000000000000000000000000a";
const ANOTHER_MACHINE: &str = "0000000000000000000000000000000d";

/// A reading of the clocks a control compares with,
/// taken at `monotonic` and `wall` in `boot` on `machine`.
fn reading(boot: &str, machine: &str, monotonic: Duration, wall: Duration) -> ClaimTime {
    return ClaimTime::from_parts(boot, monotonic, wall).on_machine(machine);
}

/// A modification time that is `older` than now,
/// which is what a wall clock that stepped forward makes a fresh file's time look like.
fn aged_by(older: Duration) -> std::io::Result<SystemTime> {
    return Ok(SystemTime::now().checked_sub(older).expect("a control's time after the epoch"));
}

/// What:
///  A record written with a reading carries it back,
///  so a sweep can judge the record's age on the clock it was written by.
/// Why:
///   The line is the only place the reading is kept,
/// and a reader that does not know the key skips the line.
#[test]
fn the_line_of_a_record_carries_the_reading_it_was_written_by() {
    let now: ClaimTime = reading(BOOT, MACHINE, Duration::from_secs(1_000), Duration::from_secs(1_700_000_000));
    let text: String = format!("seq=7\n{}", stamp_line(&now));
    assert_eq!(stamp_of(text.as_str()), Some(now));
}

/// A record an earlier release wrote carries no reading,
/// and text that is no record carries none either.
#[test]
fn a_record_of_an_earlier_release_carries_no_reading() {
    assert_eq!(stamp_of("seq=7\nref=refs/heads/main\n"), None);
    assert_eq!(stamp_of("stamp=\n"), None);
    assert_eq!(stamp_of("stamp=not-a-reading\n"), None);
    assert_eq!(stamp_of(""), None);
}

/// What:
///  A wall clock that stepped forward does not age a record of this boot:
///  the monotonic clock the client's own wait is measured on says the record is a moment old,
///  and a client of it may still read it.
/// Why:
///   This is the defect the reading exists for:
/// a machine that resumed from sleep moved the wall clock past the window while the client,
/// whose wait does not count a suspended host's sleep, was still inside it.
/// The same record judged by its modification time alone looks older than the window,
/// which is what the cleanup did before the reading was written.
#[test]
fn a_wall_clock_step_does_not_age_a_record_of_this_boot() {
    let written: ClaimTime = reading(BOOT, MACHINE, Duration::from_secs(1_000), Duration::from_secs(1_700_000_000));
    let now: ClaimTime = written.advanced(Duration::from_secs(30)).wall_stepped_forward(A_FINISHED_RECORD_IS_READ.saturating_mul(2));
    assert!(a_client_may_still_read(Some(&written), aged_by(A_FINISHED_RECORD_IS_READ.saturating_mul(2)), &now), "a record a moment old keeps the journal however far the wall clock moved");
    assert!(
        !a_client_may_still_read(None, aged_by(A_FINISHED_RECORD_IS_READ.saturating_mul(2)), &now),
        "the modification time this control is about is older than the window, so the reading is what keeps the record",
    );
}

/// What:
///  A record written longer ago than a client can look is past,
///  measured on the monotonic clock and not on the wall clock,
///  so a wall clock that stepped back does not keep it for ever.
/// Why:
///   The record must go once no client can read it,
/// and a clock that moved says nothing about the wait a client is inside.
#[test]
fn a_record_older_than_the_window_on_the_monotonic_clock_is_past() {
    let written: ClaimTime = reading(BOOT, MACHINE, Duration::from_secs(1_000), Duration::from_secs(1_700_000_000));
    let now: ClaimTime = written
        .advanced(A_FINISHED_RECORD_IS_READ.saturating_add(Duration::from_secs(1)))
        .wall_stepped_back(A_FINISHED_RECORD_IS_READ.saturating_mul(2));
    assert!(!a_client_may_still_read(Some(&written), Ok(SystemTime::now()), &now), "a record no client can still look for is past");
}

/// What:
///  A record of an earlier boot of the same machine is past at once,
///  because a restart ended every client of that boot.
/// Why:
///   Waiting out the window of a boot that ended delays the cleanup of every journal a reboot left.
#[test]
fn a_record_of_an_earlier_boot_of_this_machine_is_past() {
    let written: ClaimTime = reading(ANOTHER_BOOT, MACHINE, Duration::from_secs(1_000), Duration::from_secs(1_700_000_000));
    let now: ClaimTime = reading(BOOT, MACHINE, Duration::from_secs(1_001), Duration::from_secs(1_700_000_001));
    assert!(!a_client_may_still_read(Some(&written), Ok(SystemTime::now()), &now));
}

/// What:
///  A record of another machine is judged by the wall clock,
///  and one written a moment ago on a host whose clock this one shares keeps the journal.
/// Why:
///   A different boot ID of another machine proves no restart of that machine,
/// so its client may still be running there.
#[test]
fn a_record_of_another_machine_is_judged_by_the_wall_clock() {
    let written: ClaimTime = reading(ANOTHER_BOOT, ANOTHER_MACHINE, Duration::from_secs(1_000), Duration::from_secs(1_700_000_000));
    let now: ClaimTime = reading(BOOT, MACHINE, Duration::from_secs(9_000), Duration::from_secs(1_700_000_030));
    assert!(a_client_may_still_read(Some(&written), Ok(SystemTime::now()), &now), "thirty seconds of wall clock keep the record");
    let later: ClaimTime = reading(
        BOOT,
        MACHINE,
        Duration::from_secs(9_000),
        Duration::from_secs(1_700_000_000).saturating_add(A_FINISHED_RECORD_IS_READ).saturating_add(Duration::from_secs(1)),
    );
    assert!(!a_client_may_still_read(Some(&written), Ok(SystemTime::now()), &later), "past the window of the wall clock the record goes");
}

/// What:
///  A record with no reading and no readable modification time is kept,
///  because no age is proven and an unproven age never discards.
/// Why:
///   The reading of a modification time can fail on a file system that gives none,
/// and a record discarded on an unproven age is the only copy of what it names.
#[test]
fn an_unreadable_modification_time_keeps_a_record_with_no_reading() {
    let now: ClaimTime = reading(BOOT, MACHINE, Duration::from_secs(1_000), Duration::from_secs(1_700_000_000));
    let unreadable: std::io::Result<SystemTime> = Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "a file system that gives no time"));
    assert!(a_client_may_still_read(None, unreadable, &now));
}
