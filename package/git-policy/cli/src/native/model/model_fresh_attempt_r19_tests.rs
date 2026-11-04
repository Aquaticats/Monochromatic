//! What:
//!  The controls of decision 19's fresh attempt with a stored notice: durable positive authority prevents the attempt; without it the
//!  attempt runs only after a durable notice; the notice follows the outcome, is printed by the next command, and is kept until the outcome
//!  is delivered; the accepted risk is a reachable state; each clause has a single-defect control.
//! Why:
//!  Question 8 is answered, so the stale owner-question matrix is replaced by the behaviour the owner approved, modelled with the actual
//!  observations (visible and durable facts, a notice file that is itself lost with the power).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('fresh attempt', () => { expect(check(Nineteen).violation).toBeUndefined(); expect(check(NoticeAfterAttempt).violation).toBeDefined(); });
//! ```

use crate::model_explore::{Report, replay};
use crate::model_fresh_attempt_r19::{Notice, Phase, Rules, State, check, initial, invariant, steps, terminal};

const LOSS: &str = "environment: power is lost";
const NO_AUTHORITY: &str = "owner: recovery sees no decisive positive authority and chooses a fresh attempt";
const WRITES_NOTICE: &str = "owner: recovery writes the notice that an attempt is scheduled";
const FORCES_NOTICE: &str = "owner: recovery forces the notice";
const PROCEEDS: &str = "owner: recovery has forced the notice and proceeds";
const STARTS: &str = "owner: the fresh attempt starts";
const PUBLISHES: &str = "owner: the attempt publishes a commit";
const PRINTS: &str = "environment: the next wrapper command prints the notice";
const REMOVES: &str = "environment: the command removes the printed notice";
const FORCES_REMOVAL: &str = "owner: the removal of the delivered notice is forced";
const UPDATES_OUTCOME: &str = "owner: the notice is updated to the attempt's outcome";
const FORCES_OUTCOME: &str = "owner: the outcome notice is forced";
const ENDS_FAILED: &str = "owner: the attempt ends conflicted, refused or signing failed";
const RECLAIMS: &str = "owner: the request's recovery data is reclaimed";
const PERSISTS_RECEIPT: &str = "environment: write-back persists the receipt";
const SETTLES: &str = "owner: recovery sees durable positive authority and settles the request";
const RECONCILES: &str = "owner: recovery replaces the scheduled notice by the published outcome";

/// The message of the first violation, or a panic when the search finds none.
fn red(rules: Rules) -> String {
    return check(rules).violation.unwrap_or_else(|| { panic!("{:?} must be red", rules) }).message;
}

/// Decision 19 over every schedule of write-back, power loss (twice), attempt outcome and the person's next command is green and exhaustive.
#[test]
fn decision_nineteen_is_green_over_every_schedule() {
    let report: Report = check(Rules::Nineteen);
    assert!(report.violation.is_none() && report.exhausted, "{:?}", report.violation.map(|found| { return (found.message, found.trace); }));
}

/// Durable positive authority prevents the attempt: with the receipt persisted, the loss leaves recovery settled; the control that ignores
/// the authority makes a second commit.
#[test]
fn a_durable_receipt_settles_the_request_and_the_control_that_ignores_it_runs_a_second_attempt() {
    let settled: State = replay(initial(Rules::Nineteen), steps, &["environment: write-back persists the receipt", LOSS, "owner: recovery sees durable positive authority and settles the request"]);
    assert_eq!(settled.phase, Phase::Settled);
    assert_eq!(settled.notice_v, Notice::None);
    let doubled: State = replay(
        initial(Rules::FreshDespitePositive),
        steps,
        &["environment: write-back persists the receipt", LOSS, NO_AUTHORITY, WRITES_NOTICE, FORCES_NOTICE, PROCEEDS, STARTS],
    );
    assert_eq!(invariant(&doubled), Some("a fresh attempt ran although durable positive authority survived".to_string()));
    assert!(red(Rules::FreshDespitePositive).contains("durable positive authority"));
}

/// The accepted risk: nothing was durable, so the loss removes the landing and recovery makes a fresh attempt, which publishes again. This is
/// a reachable state of the approved rule (the first commit's facts were lost with the power, so there is no second commit to the person's
/// knowledge) and the notice says so.
#[test]
fn with_no_durable_authority_a_fresh_attempt_publishes_and_the_notice_says_so() {
    let ended: State = replay(
        initial(Rules::Nineteen),
        steps,
        &[LOSS, NO_AUTHORITY, WRITES_NOTICE, FORCES_NOTICE, PROCEEDS, STARTS, PUBLISHES, "owner: the notice is updated to the attempt's outcome"],
    );
    assert_eq!(ended.commits, 2);
    assert_eq!(ended.notice_v, Notice::Published);
    assert_eq!(terminal(&ended), None);
    let printed: State = replay(ended, steps, &[FORCES_OUTCOME, PRINTS]);
    assert!(printed.delivered.is_none() && printed.notice_v == Notice::Published, "a print alone delivers nothing: the removal is not forced yet");
    let removed: State = replay(printed, steps, &[REMOVES]);
    assert!(removed.delivered.is_none() && removed.notice_v == Notice::None, "a removal that is not forced delivers nothing");
    let delivered: State = replay(removed, steps, &[FORCES_REMOVAL]);
    assert_eq!(delivered.delivered, Some(delivered.attempt));
    assert!(delivered.notice_d == Notice::None && delivered.printed == Notice::Published);
}

/// Another crash during the attempt repeats the ambiguity and the notice survives it: the notice was forced before the attempt started.
#[test]
fn a_second_crash_during_the_attempt_leaves_the_notice_and_repeats_the_attempt() {
    let again: State = replay(initial(Rules::Nineteen), steps, &[LOSS, NO_AUTHORITY, WRITES_NOTICE, FORCES_NOTICE, PROCEEDS, STARTS, LOSS]);
    assert_eq!(again.notice_v, Notice::Scheduled, "the forced notice survived the second loss");
    let repeated: State = replay(again, steps, &[NO_AUTHORITY, PROCEEDS, STARTS]);
    assert_eq!(repeated.phase, Phase::Running);
}

/// The notice is durable before the attempt: the control that writes it afterwards runs an attempt with no notice.
#[test]
fn the_notice_is_forced_before_the_attempt_starts() {
    let unannounced: State = replay(initial(Rules::NoticeAfterAttempt), steps, &[LOSS, NO_AUTHORITY, "owner: recovery goes straight to the attempt", STARTS]);
    assert_eq!(invariant(&unannounced), Some("a fresh attempt runs and no durable notice names it".to_string()));
    assert!(red(Rules::NoticeAfterAttempt).contains("no durable notice"));
}

/// The notice is outcome-specific: the control that keeps saying "scheduled" after the attempt published is red.
#[test]
fn the_notice_follows_the_outcome() {
    let stale: State = replay(initial(Rules::StaleNotice), steps, &[LOSS, NO_AUTHORITY, WRITES_NOTICE, FORCES_NOTICE, PROCEEDS, STARTS, PUBLISHES]);
    assert_eq!(terminal(&stale), Some("the stored notice does not say what the attempt did".to_string()));
    assert!(red(Rules::StaleNotice).contains("does not say what the attempt did"));
}

/// The notice is kept until the outcome is delivered: printing "scheduled" removes nothing, and the control that discards on the first print
/// leaves a later fresh attempt with no notice.
#[test]
fn a_printed_scheduled_notice_stays_and_the_early_discard_is_red() {
    let kept: State = replay(initial(Rules::Nineteen), steps, &[LOSS, NO_AUTHORITY, WRITES_NOTICE, FORCES_NOTICE, PRINTS]);
    assert_eq!(kept.notice_v, Notice::Scheduled);
    assert!(kept.delivered.is_none());
    assert!(red(Rules::DiscardedOnFirstPrint).contains("no durable notice"));
}

/// The request's recovery data is not reclaimed before the outcome is delivered.
#[test]
fn recovery_data_is_kept_until_the_outcome_notice_is_delivered() {
    let early: State = replay(
        initial(Rules::ReclaimBeforeDelivery),
        steps,
        &[LOSS, NO_AUTHORITY, WRITES_NOTICE, FORCES_NOTICE, PROCEEDS, STARTS, PUBLISHES, "owner: the request's recovery data is reclaimed"],
    );
    assert_eq!(invariant(&early), Some("the request's recovery data was reclaimed before the outcome notice was delivered".to_string()));
    assert!(red(Rules::ReclaimBeforeDelivery).contains("reclaimed before"));
    let labels: Vec<String> = steps(&replay(initial(Rules::Nineteen), steps, &[LOSS, NO_AUTHORITY, WRITES_NOTICE, FORCES_NOTICE, PROCEEDS, STARTS, PUBLISHES])).into_iter().map(|step| { return step.0; }).collect();
    assert!(labels.iter().all(|label| { return !label.contains("reclaimed"); }), "{:?}", labels);
}

/// Review 9b item 2A: the fresh attempt published, background write-back persisted the receipt, and a second power loss came before the
/// outcome notice was updated. Recovery correctly sees durable positive authority and settles without another attempt, and the stored
/// notice still says "scheduled"; nothing reconciled it. Under the rule recovery replaces it by the published outcome; the control that
/// leaves it is red at quiescence.
#[test]
fn a_settled_recovery_reconciles_a_notice_that_still_says_scheduled() {
    let before_settle = [LOSS, NO_AUTHORITY, WRITES_NOTICE, FORCES_NOTICE, PROCEEDS, STARTS, PUBLISHES, PERSISTS_RECEIPT, LOSS, SETTLES];
    let settled: State = replay(initial(Rules::Nineteen), steps, &before_settle);
    assert_eq!(settled.phase, Phase::Settled);
    assert_eq!(settled.notice_v, Notice::Scheduled, "the durable notice from before the attempt");
    let reconciled: State = replay(settled, steps, &[RECONCILES]);
    assert_eq!(reconciled.notice_v, Notice::Published);
    let kept: State = replay(initial(Rules::SettledKeepsNotice), steps, &[before_settle.as_slice(), &[PRINTS]].concat());
    assert!(terminal(&kept).is_some_and(|message| { return message.contains("settled the request but the stored notice still says"); }), "{:?}", terminal(&kept));
    assert!(red(Rules::SettledKeepsNotice).contains("settled the request but"));
}

/// Review 9b item 2B: attempt 1 published and its outcome notice was delivered (printed, removed, removal forced); a second power loss lost
/// the still-unforced landing facts; recovery scheduled attempt 2, which ended before its notice was delivered. Delivery of attempt 1 is not
/// delivery of attempt 2, so its recovery data is not reclaimed; the control with one request-wide Boolean reclaims it.
#[test]
fn an_earlier_attempts_delivery_does_not_authorize_reclaiming_a_later_attempt() {
    let first_delivered = [LOSS, NO_AUTHORITY, WRITES_NOTICE, FORCES_NOTICE, PROCEEDS, STARTS, PUBLISHES, UPDATES_OUTCOME, FORCES_OUTCOME, PRINTS, REMOVES, FORCES_REMOVAL];
    let second_ended = [LOSS, NO_AUTHORITY, WRITES_NOTICE, FORCES_NOTICE, PROCEEDS, STARTS, ENDS_FAILED];
    let schedule: Vec<&str> = [first_delivered.as_slice(), second_ended.as_slice()].concat();
    let bound: State = replay(initial(Rules::Nineteen), steps, &schedule);
    assert_eq!(bound.attempt, 2);
    assert_eq!(bound.delivered, Some(1));
    let labels: Vec<String> = steps(&bound).into_iter().map(|step| { return step.0; }).collect();
    assert!(!labels.iter().any(|label| { return label == RECLAIMS; }), "attempt 2 was reclaimable on attempt 1's delivery: {:?}", labels);
    let wide: State = replay(initial(Rules::DeliveryIsRequestWide), steps, &schedule);
    let reclaimed: State = replay(wide, steps, &[RECLAIMS]);
    assert_eq!(invariant(&reclaimed), Some("the request's recovery data was reclaimed before the outcome notice was delivered".to_string()));
    assert!(red(Rules::DeliveryIsRequestWide).contains("reclaimed before"));
}
