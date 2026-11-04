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
    let printed: State = replay(ended, steps, &["owner: the outcome notice is forced", PRINTS]);
    assert!(printed.delivered && printed.notice_v == Notice::None && printed.printed == Notice::Published);
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
    assert!(!kept.delivered);
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
