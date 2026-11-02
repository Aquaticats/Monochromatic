//! Controls for the claim clock: which clock judges which pair of readings, and that a reading survives a file name.

use super::*;

const BOOT: &str = "0123456789abcdef0123456789abcdef";
const OTHER_BOOT: &str = "fedcba9876543210fedcba9876543210";

fn reading() -> ClaimTime {
    return ClaimTime::from_parts(BOOT, Duration::from_secs(5000), Duration::from_secs(1_700_000_000));
}

/// Within one boot the monotonic clock decides, so a wall-clock step in either direction changes nothing.
#[test]
fn within_one_boot_a_wall_clock_step_changes_no_elapsed_time() {
    let then: ClaimTime = reading();
    let later: ClaimTime = then.advanced(Duration::from_secs(90));
    assert_eq!(later.elapsed_since(&then), Duration::from_secs(90));
    assert_eq!(later.wall_stepped_forward(Duration::from_secs(7200)).elapsed_since(&then), Duration::from_secs(90), "a step forward");
    assert_eq!(later.wall_stepped_back(Duration::from_secs(7200)).elapsed_since(&then), Duration::from_secs(90), "a step back");
}

/// Across boots the monotonic readings mean nothing to each other, so the wall clock decides.
#[test]
fn across_boots_the_wall_clock_decides() {
    let then: ClaimTime = reading();
    let later: ClaimTime = then.advanced(Duration::from_secs(90)).in_boot(OTHER_BOOT);
    assert_eq!(later.elapsed_since(&then), Duration::from_secs(90));
    assert_eq!(later.wall_stepped_back(Duration::from_secs(3600)).elapsed_since(&then), Duration::ZERO, "a reading from the future is no time");
}

/// A reading with no boot ID has only its wall clock, even beside an identical reading.
#[test]
fn a_reading_without_a_boot_falls_back_to_the_wall_clock() {
    let then: ClaimTime = ClaimTime::from_parts("", Duration::from_secs(10), Duration::from_secs(1000));
    let later: ClaimTime = ClaimTime::from_parts("", Duration::from_secs(10), Duration::from_secs(1500));
    assert_eq!(later.elapsed_since(&then), Duration::from_secs(500));
    assert_eq!(ClaimTime::from_wall(UNIX_EPOCH + Duration::from_secs(1000)).elapsed_since(&then), Duration::ZERO);
}

/// A reading written into a file name reads back whole, and text that is not one is refused.
#[test]
fn a_reading_round_trips_through_its_text() {
    let original: ClaimTime = reading();
    assert_eq!(ClaimTime::decode(original.encode().as_str()), Some(original));
    let unbooted: ClaimTime = ClaimTime::from_parts("", Duration::ZERO, Duration::from_secs(9));
    assert_eq!(ClaimTime::decode(unbooted.encode().as_str()), Some(unbooted));
    let broken: Vec<String> = vec![
        String::new(),
        String::from("x"),
        String::from("a~b~c"),
        String::from("~1"),
        String::from("~1~2~3"),
        format!("{BOOT}~1"),
        format!("{BOOT}x~1~2"),
        format!("{BOOT}~-1~2"),
    ];
    for text in broken {
        assert_eq!(ClaimTime::decode(text.as_str()), None, "{text:?}");
    }
}

/// The reading of this host now is in one boot with itself, so elapsed time between two of them is the monotonic clock's and
/// never negative.
#[test]
fn two_readings_of_this_host_agree_on_the_boot() {
    let first: ClaimTime = ClaimTime::read();
    let second: ClaimTime = ClaimTime::read();
    assert_eq!(first.boot_id, second.boot_id);
    assert!(second.monotonic_millis >= first.monotonic_millis);
    #[cfg(target_os = "linux")]
    assert_eq!(first.boot_id.len(), BOOT_ID_LENGTH, "Linux gives every boot an ID");
}
