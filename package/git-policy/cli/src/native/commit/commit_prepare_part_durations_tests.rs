//! What:
//!  The stretches a preparation keeps,
//!  the Git steps taken out of them,
//!  and their bound and record.
//! Why:
//!  A stretch that keeps the time of a Git child it started would name the same
//!  milliseconds twice;
//! one whose name is lost when a preparation stops early
//!  would hide the stretch a stopped commit waited in;
//! and an unbounded list would grow with the work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(drained.map((part) => part.name)).toEqual(['layout', 'capture']);
//! ```

/// The stretches,
/// their clock and their record.
use super::{MAX_PREPARE_PARTS, PreparePart, PreparePartClock, drain_prepare_parts, note_prepare_part, prepare_parts_json};
use crate::transaction_step_durations::note_step;
use std::time::{Duration, Instant};

/// A stretch's figure as the record would state it,
/// in whole milliseconds.
fn milliseconds(part: &PreparePart) -> u128 {
    return part.took.as_millis();
}

/// The clock names each stretch and keeps the one that was running when the preparation left.
#[test]
fn each_stretch_keeps_its_name_and_the_open_one_is_kept() {
    let mut clock: PreparePartClock = PreparePartClock::begin("layout");
    std::thread::sleep(Duration::from_millis(3));
    clock.next("capture");
    std::thread::sleep(Duration::from_millis(3));
    // The preparation leaves here, as a stop or a failure does.
    drop(clock);
    let (parts, dropped) = drain_prepare_parts();
    assert_eq!(parts.iter().map(|part| { return part.name.as_str(); }).collect::<Vec<&str>>(), vec!["layout", "capture"]);
    assert!(milliseconds(&parts[0]) >= 2, "{parts:?}");
    assert!(milliseconds(&parts[1]) >= 2, "{parts:?}");
    assert_eq!(dropped, 0);
}

/// A Git step that ran inside a stretch is taken out of it,
/// and a stretch with no step keeps its whole span.
#[test]
fn a_git_step_inside_a_stretch_is_taken_out_of_it() {
    let mut clock: PreparePartClock = PreparePartClock::begin("policies");
    // The step ledger holds a step that began inside the stretch and ran far longer than it, which is what a step under host
    //  pressure looks like.
    note_step("git add", Instant::now(), Duration::from_millis(500));
    std::thread::sleep(Duration::from_millis(3));
    clock.next("changes");
    std::thread::sleep(Duration::from_millis(3));
    drop(clock);
    let (parts, _dropped) = drain_prepare_parts();
    assert_eq!(parts.len(), 2);
    assert!(parts[0].took < Duration::from_millis(1), "the step's half second is taken out: {:?}", parts[0]);
    assert!(milliseconds(&parts[1]) >= 2, "no step ran in this stretch: {:?}", parts[1]);
}

/// A name marked more than once is added together and says how many stretches it is made of.
#[test]
fn a_name_marked_twice_adds_up_and_says_how_many_calls() {
    note_prepare_part("layout", Duration::from_millis(11));
    note_prepare_part("capture", Duration::from_millis(13));
    note_prepare_part("layout", Duration::from_millis(17));
    let (parts, _dropped) = drain_prepare_parts();
    assert_eq!(prepare_parts_json(parts.as_slice()), String::from("{\"layout\":{\"ms\":28,\"calls\":2},\"capture\":{\"ms\":13,\"calls\":1}}"));
}

/// The record carries at most `MAX_PREPARE_PARTS` stretches and says how many it dropped.
#[test]
fn the_list_stops_at_its_bound_and_counts_the_rest() {
    let over: usize = MAX_PREPARE_PARTS + 3;
    for at in 0..over {
        note_prepare_part("layout", Duration::from_millis(u64::try_from(at).expect("fits")));
    }
    let (parts, dropped) = drain_prepare_parts();
    assert_eq!(parts.len(), MAX_PREPARE_PARTS);
    assert_eq!(dropped, 3);
    assert_eq!(drain_prepare_parts().0.len(), 0, "one record takes the stretches it describes");
}

/// Names are escaped as JSON strings,
/// so a name can never break the record's line.
#[test]
fn a_name_with_delimiters_stays_one_member() {
    note_prepare_part("a\"b\\c\nd", Duration::from_millis(1));
    let (parts, _dropped) = drain_prepare_parts();
    let json: String = prepare_parts_json(parts.as_slice());
    assert_eq!(json, String::from("{\"a\\\"b\\\\c\\nd\":{\"ms\":1,\"calls\":1}}"));
    assert!(!json.contains('\n'));
}
