//! What:
//!  The controls of the late observers: `attach-watch` with a switch paused longer than the span, a switch that begins after the
//!  install and the leftover test; and T1's registry comparison with a lost baseline, the probe and a non-atomic listing.
//! Why:
//!  Controls 15, 16 and 17 of the twelfth review. Each is red under revision 12 and green under 13; the revision 13 residue (a
//!  switch that finishes after the span leaves the merge's files in the new branch) is reached and is stated, not hidden.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a switch paused longer than the span: twelve promised a next check, thirteen states the residue', ...);
//! ```

use crate::model_explore::{Report, replay};
use crate::model_watch_r13::{
    Rules, Step, check_retire, check_watch, retire_initial, retire_invariant, retire_steps, watch_initial, watch_invariant, watch_steps,
};

const INSTALL: &str = "owner: MG3 installs the merge's files and records attach-watch";
const PAUSED: &str = "environment: a native switch begins before the install and pauses";
const TICK: &str = "timer: one tick of the span";
const SPAN_ENDED: &str = "owner: the span ended and the check found HEAD still attached to the target";
const LATE_COMPLETE: &str = "environment: the paused switch completes after the watch ended";
const AFTER_INSTALL_SWITCH: &str = "environment: a switch that began after the install rewrites the file itself";
const CLEANUP: &str = "owner: MG6c cleans up after the late switch";

/// Control 15: a switch paused longer than the span, then completing. Revision 12 dropped the obligation with a timer and told the
/// person a next check would come (red); revision 13 retains the obligation for the span, reaches the same leftover, and states
/// it as a residue without promising a check (green, and the residue is shown reachable).
#[test]
fn a_switch_paused_longer_than_the_span_is_a_stated_residue_not_a_promise() {
    let schedule = &[PAUSED, INSTALL, TICK, TICK, TICK, SPAN_ENDED, LATE_COMPLETE];
    let twelve = replay(watch_initial(Rules::Twelve), watch_steps, schedule);
    assert!(twelve.residue && twelve.promises_next_check);
    assert_eq!(watch_invariant(&twelve), Some("a leftover remained after the watch ended and the message promised a next check".to_string()));
    let thirteen = replay(watch_initial(Rules::Thirteen), watch_steps, schedule);
    assert!(thirteen.residue, "the residue exists and is reachable");
    assert_eq!(watch_invariant(&thirteen), None, "and it is stated, not promised away");
}

/// Control 16: a switch that begins after the install rewrites the file itself; MG6c must leave Git's result alone (revision 12
/// restored the pinned original over it), while a leftover (the merge's file still there) is restored.
#[test]
fn the_leftover_test_leaves_what_a_later_switch_wrote() {
    let twelve = replay(watch_initial(Rules::Twelve), watch_steps, &[INSTALL, AFTER_INSTALL_SWITCH, CLEANUP]);
    assert!(twelve.git_result_overwritten);
    assert_eq!(watch_invariant(&twelve), Some("MG6c restored the original over the file a native switch had written".to_string()));
    let thirteen = replay(watch_initial(Rules::Thirteen), watch_steps, &[INSTALL, AFTER_INSTALL_SWITCH, CLEANUP]);
    assert!(!thirteen.git_result_overwritten);
    let leftover = replay(watch_initial(Rules::Thirteen), watch_steps, &[PAUSED, INSTALL, "environment: the paused switch completes; HEAD leaves the target and the merge's files stay", CLEANUP]);
    assert_eq!(leftover.file, crate::model_watch_r13::File::Original, "a true leftover is restored from the pinned original");
}

/// The whole space of the watch: red under 12, green and exhausted under 13.
#[test]
fn the_watch_is_red_under_twelve_and_green_under_thirteen() {
    assert!(check_watch(Rules::Twelve).violation.is_some());
    let thirteen: Report = check_watch(Rules::Thirteen);
    assert!(thirteen.violation.is_none() && thirteen.exhausted, "{:?}", thirteen.violation.map(|found| { return (found.message, found.trace); }));
}

/// Control 17: T1 crashes after the check and restarts. Revision 12 compared with a baseline lost in the crash, and a run admitted
/// between the original baseline and the crash was never seen (red); revision 13 restarts from its Pre with a fresh `L0` (green).
#[test]
fn t1_recovery_without_its_original_baseline_releases_over_a_late_run_under_twelve_only() {
    let twelve_schedule = &[
        "owner: T1 writes the intent and lists the registry as L0",
        "environment: a maintenance run is admitted and its reservation linked",
        "owner: T1 runs the connectivity check",
        "environment: the owner dies after the check and restarts",
        "owner: T1 lists L1 and compares it with L0",
    ];
    let twelve = replay(retire_initial(Rules::Twelve), retire_steps, twelve_schedule);
    assert_eq!(twelve.step, Step::Released);
    assert_eq!(retire_invariant(&twelve), Some("T1 released the spool although a run was admitted after its baseline".to_string()));
    assert!(check_retire(Rules::Twelve).violation.is_some());
    let thirteen: Report = check_retire(Rules::Thirteen);
    assert!(thirteen.violation.is_none() && thirteen.exhausted, "{:?}", thirteen.violation.map(|found| { return found.trace; }));
}

/// The registry probe: a reservation linked between the baseline and the comparison is found by the probe even when a listing
/// misses it (revision 12's listings are not atomic); revision 12 releases on the missed link.
#[test]
fn a_link_that_falls_between_the_reads_of_a_listing_is_found_by_the_probe() {
    let missed = replay(
        retire_initial(Rules::Twelve),
        retire_steps,
        &[
            "owner: T1 writes the intent and lists the registry as L0",
            "owner: T1 runs the connectivity check",
            "environment: a maintenance run is admitted and its reservation linked",
            "owner: T1 lists L1 and a link that falls between the reads of the listing is missed",
        ],
    );
    assert_eq!(missed.step, Step::Released);
    assert!(retire_invariant(&missed).is_some());
    let probed = replay(
        retire_initial(Rules::Thirteen),
        retire_steps,
        &[
            "owner: T1 writes the intent and lists the registry as L0",
            "owner: T1 runs the connectivity check",
            "environment: a maintenance run is admitted and its reservation linked",
            "owner: T1 probes the next number, lists L1, compares with L0, and releases or withdraws",
        ],
    );
    assert_eq!(probed.step, Step::Withdrawn);
}
