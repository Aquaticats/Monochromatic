//! What:
//!  The controls of the late observers: `attach-watch` with a switch paused longer than the span, a switch that begins after the
//!  install and the leftover test; and T1's registry comparison with a lost baseline, the probe and a non-atomic listing.
//! Why:
//!  Controls 15, 16 and 17 of the twelfth review and 17 and 18 of the thirteenth. Each is red under the earlier revision; the revision 13 residue (a
//!  switch that finishes after the span leaves the merge's files in the new branch) is reached and is stated, not hidden.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a switch paused longer than the span: twelve promised a next check, thirteen states the residue', ...);
//! ```

use crate::model_explore::{Report, replay};
use crate::model_watch_r13::{
    Authority, Policy, Residue, Rules, Span, check_watch_except_concession, Step, check_retire, check_watch, retire_initial, retire_invariant, retire_steps, watch_initial, watch_invariant, watch_steps,
};

const INSTALL: &str = "owner: MG3 installs the merge's files and records attach-watch";
const PAUSED: &str = "environment: a native switch begins before the install and pauses";
const TICK: &str = "timer: one tick of the span";
const SPAN_ENDED: &str = "owner: the end of the authority and the check found HEAD still attached to the target";
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

/// Question 7a's concession and control 18: the person ran `reset --soft`, then a switch that carries the staged merge content and
/// leaves the file alone. Every by-value test passes, so revision 13 undoes the deliberate state (red). Revision 14 with the index-unchanged
/// test (7a option A) does not undo it when any command wrote the index in between, and undoes it when none did: that schedule is the stated
/// concession of option A, shown reachable and not called green. Options B and C never undo.
#[test]
fn a_deliberate_state_is_undone_by_thirteen_and_only_in_the_stated_concession_by_fourteen() {
    const DELIBERATE: &str = "environment: reset --soft, then a switch that carries the staged merge content and leaves the file alone";
    const INDEX_WRITE: &str = "environment: a command writes the index (git add or a refresh)";
    let measured_a = |authority: Authority| -> Rules {
        return Rules::Fourteen(Policy { authority, span: Span::Measured, residue: Residue::PrintOnce });
    };
    let thirteen = replay(watch_initial(Rules::Thirteen), watch_steps, &[INSTALL, INDEX_WRITE, DELIBERATE, CLEANUP]);
    assert!(thirteen.undone_deliberate, "revision 13 undoes it even after the index was written");
    let guarded = replay(watch_initial(measured_a(Authority::ByValueWithIndexTest)), watch_steps, &[INSTALL, INDEX_WRITE, DELIBERATE, CLEANUP]);
    assert!(!guarded.undone_deliberate && guarded.reported, "the index changed: the leftover is named, nothing is restored");
    let concession = replay(watch_initial(measured_a(Authority::ByValueWithIndexTest)), watch_steps, &[INSTALL, DELIBERATE, CLEANUP]);
    assert_eq!(watch_invariant(&concession), Some("MG6c undid a state the person kept on purpose".to_string()), "the stated concession of option A");
    for authority in [Authority::AskFirst, Authority::ReportOnly] {
        let report: Report = check_watch(Rules::Fourteen(Policy { authority, span: Span::Measured, residue: Residue::PrintOnce }));
        assert!(report.violation.is_none() && report.exhausted, "{:?}: {:?}", authority, report.violation.map(|found| { return found.trace; }));
    }
}

/// Revision 12 and revision 13 are red over the whole space.
#[test]
fn the_earlier_watches_are_red_over_every_schedule() {
    assert!(check_watch(Rules::Twelve).violation.is_some());
    assert!(check_watch(Rules::Thirteen).violation.is_some());
}

/// Review 4b: the end of the authority is observed, not assumed. After a paused switch completed and HEAD left the target, the span's
/// end must not drop the watch silently (the earlier transition did, leaving the merge's file untouched); the cleanup is what is enabled.
#[test]
fn the_end_of_the_authority_does_not_abandon_a_leftover_after_head_moved() {
    let policy = Rules::Fourteen(Policy { authority: Authority::ByValueWithIndexTest, span: Span::Measured, residue: Residue::PrintOnce });
    let moved = replay(
        watch_initial(policy),
        watch_steps,
        &[PAUSED, INSTALL, TICK, TICK, TICK, "environment: the paused switch completes; HEAD leaves the target and the merge's files stay"],
    );
    let labels: Vec<String> = watch_steps(&moved).into_iter().map(|step| { return step.0; }).collect();
    assert!(!labels.contains(&SPAN_ENDED.to_string()), "{:?}", labels);
    assert!(labels.contains(&CLEANUP.to_string()));
    // The earlier revisions could take the dropping transition only because it did not check HEAD; the guard is the head test.
    assert_eq!(moved.head, crate::model_watch_r13::Head::Elsewhere);
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

/// Revision 15 with the owner's answers (7a automatic cleanup with the index fingerprint test, 7b the measured span from MG3's installation,
/// 7c a reminder): the one violation the whole space reaches is the accepted concession of 7a (a deliberate state the fingerprint cannot
/// tell from a leftover), reported as such; with that concession set aside, every other property holds over every schedule, including no
/// new cleanup after the span, a cleanup already begun finishing, a derived reminder obligation on every no-restore ending, and dismissal
/// keeping the saved copies.
#[test]
fn fifteen_has_the_accepted_concession_and_nothing_else() {
    let all: Report = check_watch(Rules::Fifteen);
    assert_eq!(all.violation.expect("the accepted concession of 7a").message, "MG6c undid a state the person kept on purpose");
    let rest: Report = check_watch_except_concession(Rules::Fifteen);
    assert!(rest.violation.is_none() && rest.exhausted, "{:?}", rest.violation.map(|found| { return (found.message, found.trace); }));
}

/// Control 14 (expiry during cleanup): MG6c has taken the lock and begun before the span ended; the clock passes the span; the cleanup
/// finishes (one already begun finishes), and no new cleanup starts after the span.
#[test]
fn a_cleanup_begun_before_the_span_ends_finishes_after_it_and_no_new_one_starts() {
    const BEGIN: &str = "owner: MG6c takes index.lock and begins the cleanup";
    const FINISH: &str = "owner: MG6c finishes the cleanup";
    let begun = replay(
        watch_initial(Rules::Fifteen),
        watch_steps,
        &[PAUSED, INSTALL, "environment: the paused switch completes; HEAD leaves the target and the merge's files stay", BEGIN, TICK, TICK, TICK],
    );
    assert!(begun.clock > crate::model_watch_r13::SPAN && begun.cleanup_started);
    let finished = replay(begun, watch_steps, &[FINISH]);
    assert_eq!(finished.file, crate::model_watch_r13::File::Original);
    // A new cleanup after the span: HEAD moves after the span, and the only step is to name the leftover and keep a reminder.
    let late = replay(
        watch_initial(Rules::Fifteen),
        watch_steps,
        &[PAUSED, INSTALL, TICK, TICK, TICK, "environment: the paused switch completes; HEAD leaves the target and the merge's files stay"],
    );
    let labels: Vec<String> = watch_steps(&late).into_iter().map(|step| { return step.0; }).collect();
    assert!(!labels.contains(&BEGIN.to_string()));
    assert!(labels.contains(&"owner: no cleanup starts; the leftover is named and a reminder is kept".to_string()));
}

/// Control 15 (fingerprint restoration): the index changes away from the recorded bytes and back, or is rewritten with identical bytes,
/// before the check. The fingerprint is equal, so the cleanup proceeds; this is part of the concession of 7a and is shown reachable.
#[test]
fn an_index_that_returns_to_the_recorded_bytes_passes_the_fingerprint_test() {
    let reached = replay(
        watch_initial(Rules::Fifteen),
        watch_steps,
        &[
            INSTALL,
            "environment: a command rewrites the index with identical bytes, or changes it away and back",
            "environment: reset --soft, then a switch that carries the staged merge content and leaves the file alone",
            "owner: MG6c takes index.lock and begins the cleanup",
            "owner: MG6c finishes the cleanup",
        ],
    );
    assert!(reached.index_rewritten && !reached.fingerprint_changed && reached.undone_deliberate);
}

/// Control 18 (dismissal versus disposal): the reminder is dismissed, and the saved copies remain, so the person can still restore.
#[test]
fn dismissing_the_reminder_deletes_no_saved_copy_and_the_restore_command_still_works() {
    let reached = replay(
        watch_initial(Rules::Fifteen),
        watch_steps,
        &[
            INSTALL,
            TICK,
            TICK,
            TICK,
            "environment: a switch that began after the install rewrites the file itself",
        ],
    );
    assert_eq!(reached.file, crate::model_watch_r13::File::Git);
    let leftover = replay(
        watch_initial(Rules::Fifteen),
        watch_steps,
        &[PAUSED, INSTALL, TICK, TICK, TICK, "environment: the paused switch completes; HEAD leaves the target and the merge's files stay"],
    );
    let named = replay(leftover, watch_steps, &["owner: no cleanup starts; the leftover is named and a reminder is kept"]);
    assert!(named.reminder);
    let dismissed = replay(named, watch_steps, &["person: dismisses the reminder"]);
    assert!(!dismissed.reminder && dismissed.saved_copy, "dismissing the notification disposes of nothing");
    let restored = replay(dismissed, watch_steps, &["person: runs the printed restore command"]);
    assert_eq!(restored.file, crate::model_watch_r13::File::Original);
}
