//! What:
//!  The stretches a build keeps,
//!  the Git steps taken out of them,
//!  and their bound and record.
//! Why:
//!  A stretch that keeps the time of a Git child it started would name the same
//!  milliseconds twice;
//! one whose name is lost when a build stops early
//!  would hide the stretch a stopped commit waited in,
//! which is the defect the build's ledger was added for;
//! and an unbounded list would grow with the work.
//!  The ledger is one list for the whole process and other controls of this crate build shadows,
//!  so each control here holds its own lock and empties the list before it starts:
//!  the names it asserts on are the ones at the end of what it kept.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(drained.map((part) => part.name).slice(-2)).toEqual(['branch', 'shadow links']);
//! ```

/// The stretches,
/// their clock and their record.
use super::{BuildPartClock, MAX_BUILD_PARTS, build_parts_json, drain_build_parts, note_build_part};
use crate::commit_prepare_part_durations::PreparePart;
use crate::transaction_step_durations::note_step;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// Held by every control here,
/// so no two of them mark and drain the one list at the same time.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// The lock and an empty list,
/// which every control in this module starts from.
fn exclusive() -> MutexGuard<'static, ()> {
    let held: MutexGuard<'static, ()> = ONE_AT_A_TIME.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let _taken: (Vec<PreparePart>, usize) = drain_build_parts();
    return held;
}

/// The names the list ends with,
/// which are the ones the control under test marked.
fn lasts(parts: &[PreparePart], count: usize) -> Vec<&str> {
    return parts.iter().skip(parts.len().saturating_sub(count)).map(|part| { return part.name.as_str(); }).collect();
}

/// The last stretch kept.
fn last(parts: &[PreparePart]) -> &PreparePart {
    return parts.last().unwrap_or_else(|| { panic!("no stretch was kept: {parts:?}") });
}

/// The clock names each stretch and keeps the one that was running when the build left.
#[test]
fn each_stretch_keeps_its_name_and_the_open_one_is_kept() {
    let _held: MutexGuard<'static, ()> = exclusive();
    let mut clock: BuildPartClock = BuildPartClock::begin("branch");
    std::thread::sleep(Duration::from_millis(3));
    clock.next("shadow links");
    std::thread::sleep(Duration::from_millis(3));
    // The build leaves here, as a stop or a failure does.
    drop(clock);
    let (parts, _dropped) = drain_build_parts();
    assert_eq!(lasts(parts.as_slice(), 2), vec!["branch", "shadow links"]);
    assert!(last(&parts).took >= Duration::from_millis(2), "{parts:?}");
}

/// A Git step that ran inside a stretch is taken out of it,
/// and a stretch with no step keeps its whole span.
#[test]
fn a_git_step_inside_a_stretch_is_taken_out_of_it() {
    let _held: MutexGuard<'static, ()> = exclusive();
    let mut clock: BuildPartClock = BuildPartClock::begin("client build");
    note_step("git commit", Instant::now(), Duration::from_millis(500));
    std::thread::sleep(Duration::from_millis(3));
    clock.next("commit read back");
    std::thread::sleep(Duration::from_millis(3));
    drop(clock);
    let (parts, _dropped) = drain_build_parts();
    assert_eq!(lasts(parts.as_slice(), 2), vec!["client build", "commit read back"]);
    let named: &[PreparePart] = &parts[parts.len() - 2..];
    assert!(named[0].took < Duration::from_millis(1), "the step's half second is taken out: {:?}", named[0]);
    assert!(named[1].took >= Duration::from_millis(2), "no step ran in this stretch: {:?}", named[1]);
}

/// A name marked more than once is added together and says how many stretches it is made of.
#[test]
fn a_name_marked_twice_adds_up_and_says_how_many_calls() {
    let _held: MutexGuard<'static, ()> = exclusive();
    note_build_part("shadow links", Duration::from_millis(11));
    note_build_part("client build", Duration::from_millis(13));
    note_build_part("shadow links", Duration::from_millis(17));
    let (parts, _dropped) = drain_build_parts();
    assert_eq!(build_parts_json(parts.as_slice()), String::from("{\"shadow links\":{\"ms\":28,\"calls\":2},\"client build\":{\"ms\":13,\"calls\":1}}"));
}

/// The record carries at most `MAX_BUILD_PARTS` stretches and says how many it dropped.
#[test]
fn the_list_stops_at_its_bound_and_counts_the_rest() {
    let _held: MutexGuard<'static, ()> = exclusive();
    let over: usize = MAX_BUILD_PARTS + 3;
    for at in 0..over {
        note_build_part("shadow skeleton", Duration::from_millis(u64::try_from(at).expect("fits")));
    }
    let (parts, dropped) = drain_build_parts();
    assert_eq!(parts.len(), MAX_BUILD_PARTS);
    assert_eq!(dropped, 3);
    assert_eq!(drain_build_parts().0.len(), 0, "one record takes the stretches it describes");
}

/// Names are escaped as JSON strings,
/// so a name can never break the record's line.
#[test]
fn a_name_with_delimiters_stays_one_member() {
    let _held: MutexGuard<'static, ()> = exclusive();
    note_build_part("a\"b\\c\nd", Duration::from_millis(1));
    let (parts, _dropped) = drain_build_parts();
    let json: String = build_parts_json(parts.as_slice());
    assert_eq!(json, String::from("{\"a\\\"b\\\\c\\nd\":{\"ms\":1,\"calls\":1}}"));
    assert!(!json.contains('\n'));
}
