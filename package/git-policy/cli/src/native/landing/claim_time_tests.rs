//! Controls for the claim clock: which clock judges which pair of readings, and that a reading survives a file name.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

const BOOT: &str = "0123456789abcdef0123456789abcdef";
const OTHER_BOOT: &str = "fedcba9876543210fedcba9876543210";
const MACHINE: &str = "00112233445566778899aabbccddeeff";
const OTHER_MACHINE: &str = "ffeeddccbbaa99887766554433221100";

/// A reading of one boot of one machine.
fn reading() -> ClaimTime {
    return ClaimTime::from_parts(BOOT, Duration::from_secs(5000), Duration::from_secs(1_700_000_000)).on_machine(MACHINE);
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

/// A rewound reading is as old as it was rewound, on both clocks.
#[test]
fn a_rewound_reading_is_that_much_older() {
    let now: ClaimTime = reading();
    let then: ClaimTime = now.rewound(Duration::from_secs(45));
    assert_eq!(now.elapsed_since(&then), Duration::from_secs(45));
    assert_eq!(then.advanced(Duration::from_secs(45)), now);
    assert_eq!(now.in_boot("").elapsed_since(&then), Duration::from_secs(45), "and by the wall clock without a boot ID");
}

/// Across two different boots of one machine the earlier boot's holder is dead, whatever either wall clock shows.
#[test]
fn a_reading_of_an_earlier_boot_is_past() {
    let then: ClaimTime = reading();
    let later: ClaimTime = then.advanced(Duration::from_secs(90)).in_boot(OTHER_BOOT);
    assert!(later.is_past_the_boot_of(&then));
    assert!(later.wall_stepped_back(Duration::from_secs(3600)).is_past_the_boot_of(&then), "a wall clock set back at boot");
    assert!(!then.is_past_the_boot_of(&then), "one boot is not past itself");
    assert!(!then.advanced(Duration::from_secs(90)).is_past_the_boot_of(&then));
}

/// Different boot IDs of two machines are no restart: the holder may be running on the other machine, so the lease decides.
#[test]
fn a_reading_of_another_machine_is_never_past_a_boot() {
    let then: ClaimTime = reading();
    let elsewhere: ClaimTime = then.advanced(Duration::from_secs(90)).in_boot(OTHER_BOOT).on_machine(OTHER_MACHINE);
    assert!(!elsewhere.is_past_the_boot_of(&then));
    assert!(!then.is_past_the_boot_of(&elsewhere));
    assert_eq!(elsewhere.elapsed_since(&then), Duration::from_secs(90), "the wall clock is the one they share");
}

/// A machine ID missing on either side is not an equal machine ID: two unknown machines are not one machine.
#[test]
fn a_reading_without_a_machine_is_never_past_a_boot() {
    let then: ClaimTime = reading();
    let later: ClaimTime = then.advanced(Duration::from_secs(90)).in_boot(OTHER_BOOT);
    assert!(later.is_past_the_boot_of(&then), "control: with the machine known the boot differs");
    assert!(!later.on_machine("").is_past_the_boot_of(&then), "the judge does not know its machine");
    assert!(!later.is_past_the_boot_of(&then.on_machine("")), "the claim does not know its machine");
    assert!(!later.on_machine("").is_past_the_boot_of(&then.on_machine("")), "neither knows");
}

/// An empty boot ID on either side leaves nothing to compare boots by, so no reading is past another's boot.
#[test]
fn a_reading_without_a_boot_is_never_past_a_boot() {
    let booted: ClaimTime = reading();
    let unbooted: ClaimTime = booted.in_boot("");
    assert!(!unbooted.is_past_the_boot_of(&booted));
    assert!(!booted.is_past_the_boot_of(&unbooted));
    assert!(!unbooted.is_past_the_boot_of(&unbooted));
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
    let machineless: ClaimTime = ClaimTime::from_parts(BOOT, Duration::from_secs(1), Duration::from_secs(2));
    assert_eq!(ClaimTime::decode(machineless.encode().as_str()), Some(machineless));
    let broken: Vec<String> = vec![
        String::new(),
        String::from("x"),
        String::from("a~b~c"),
        String::from("~1"),
        String::from("~1~2~3"),
        format!("{BOOT}~1"),
        format!("{BOOT}x~1~2"),
        format!("{BOOT}~-1~2"),
        format!("{BOOT}~1~2~"),
        format!("{BOOT}~1~2~{}", &MACHINE[1..]),
        format!("{BOOT}~1~2~{MACHINE}x"),
        format!("{BOOT}~1~2~{}z", &MACHINE[1..]),
        format!("{BOOT}~1~2~{MACHINE}~3"),
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
    assert_eq!(first.machine_id, second.machine_id, "and on the machine");
    assert!(second.monotonic_millis >= first.monotonic_millis);
    #[cfg(target_os = "linux")]
    assert_eq!(first.boot_id.len(), BOOT_ID_LENGTH, "Linux gives every boot an ID");
}

/// A reading written by a release before machine IDs (three parts) still decodes, as a reading with no machine, and is
/// judged by its lease: a different boot of the judge's does not end it.
#[test]
fn a_reading_in_the_old_three_part_form_decodes_and_is_judged_by_its_lease() {
    let old_text: String = format!("{BOOT}~5000000~1700000000");
    let old: ClaimTime = ClaimTime::decode(old_text.as_str()).expect("the old form decodes");
    assert_eq!(old, ClaimTime::from_parts(BOOT, Duration::from_secs(5000), Duration::from_secs(1_700_000_000)));
    assert_eq!(old.encode(), old_text, "and it is written back the same way");
    let judge: ClaimTime = old.advanced(Duration::from_secs(90)).in_boot(OTHER_BOOT).on_machine(MACHINE);
    assert!(!judge.is_past_the_boot_of(&old));
    assert_eq!(judge.elapsed_since(&old), Duration::from_secs(90), "so the wall clock measures it");
}

/// The machine ID is the first of the given files that holds 32 hexadecimal digits; anything else counts as unknown.
#[test]
fn the_machine_id_is_the_first_file_that_holds_one() {
    let directory: PathBuf = fixture("claim-time-machine-id");
    let files: Vec<(&str, String)> = vec![
        ("missing", String::new()),
        ("empty", String::new()),
        ("uninitialized", String::from("uninitialized\n")),
        ("short", format!("{}\n", &MACHINE[1..])),
        ("long", format!("{MACHINE}0\n")),
        ("not-hex", format!("{}g\n", &MACHINE[1..])),
        ("good", format!("{MACHINE}\n")),
        ("other-good", format!("{OTHER_MACHINE}\n")),
    ];
    for (name, text) in &files {
        if *name != "missing" {
            std::fs::write(directory.join(name), text).expect("write");
        }
    }
    let path = |name: &str| { return directory.join(name); };
    for bad in ["missing", "empty", "uninitialized", "short", "long", "not-hex"] {
        assert_eq!(machine_id_from(&[path(bad).as_path()]), "", "{bad}");
    }
    assert_eq!(machine_id_from(&[path("good").as_path()]), MACHINE);
    assert_eq!(machine_id_from(&[path("uninitialized").as_path(), path("good").as_path()]), MACHINE, "the next place is tried");
    assert_eq!(machine_id_from(&[path("good").as_path(), path("other-good").as_path()]), MACHINE, "the first wins");
    assert_eq!(machine_id_from(&[]), "");
    remove(directory.as_path());
}

/// A machine ID is 32 lowercase hexadecimal digits that are not all zero: anything else is no evidence of a machine, and taking it
///  as one would make every such host look like the same machine.
#[test]
fn only_lowercase_hexadecimal_that_is_not_all_zero_is_a_machine_id() {
    assert!(is_machine_id("0123456789abcdef0123456789abcdef"));
    assert!(!is_machine_id("0123456789ABCDEF0123456789ABCDEF"), "uppercase");
    assert!(!is_machine_id("00000000000000000000000000000000"), "all zero");
    assert!(!is_machine_id("0123456789abcdef0123456789abcde"), "short");
    assert!(!is_machine_id("0123456789abcdef0123456789abcdefa"), "long");
    assert!(!is_machine_id("0123456789abcdef0123456789abcdeg"), "not hexadecimal");
    assert!(!is_machine_id("uninitialized"), "a fresh image's text");
}

/// A stamp an earlier release wrote with an uppercase or an all-zero machine ID (it accepted any 32 hexadecimal digits) still
///  decodes, through the code that reads a claim's file name, and is judged by the wall clock beside this host's own reading
///  (which is stricter and so may have none), never read as another host's restart and never dropped.
#[test]
fn a_stamp_an_earlier_release_wrote_with_a_machine_id_of_any_case_still_decodes_and_is_judged_by_the_wall_clock() {
    for stored in ["0123456789ABCDEF0123456789ABCDEF", "00000000000000000000000000000000"] {
        let name: String = format!("item~0123456789abcdef0123456789abcdef~00112233445566778899aabbccddeeff~5000~1700000000~{stored}");
        let (_, stamp) = crate::work_item::parse_claim_name(name.as_str()).expect("the name still parses");
        let now: ClaimTime = ClaimTime::from_parts("ffeeddccbbaa99887766554433221100", Duration::from_secs(9), Duration::from_secs(1_700_000_100)).on_machine("0123456789abcdef0123456789abcdef");
        assert!(!now.is_past_the_boot_of(&stamp), "{stored}: another machine's boot is no restart");
        assert_eq!(now.elapsed_since(&stamp), Duration::from_secs(100), "{stored}: the wall clock decides");
    }
}
