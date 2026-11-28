//! What:
//!  Controls of the words a refusal prints:
//!  each form's phrases,
//!  the cause's own object and field,
//!  the history sentence,
//!  the ways forward,
//!  and the absence of a destructive command.
//! Why:
//!  A refusal is the answer a person reads,
//!  and the project chose that it prints the reads and the decisions rather than a `zfs destroy`
//!  to paste (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  veto V7,
//!  and controls 10,
//!  11 and 25).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(formA(cause)).toContain('nothing is pending and there is nothing to cancel');
//! ```

use super::super::fixtures::{
    CHECKOUT,
    CLONE,
    ORIGIN_SNAPSHOT,
    PLAN_HOME,
    SLOT,
    every_cause,
};
use super::super::origin_refusal::OriginRefusal;
use super::super::origin_refusals::RefusalForm;
use super::super::removal_history::RemovalHistory;
use super::message;

/// The cause one label names.
fn cause(label: &str) -> OriginRefusal {
    let found: Vec<(&'static str, OriginRefusal)> = every_cause();
    let Some((_label, cause)) = found.into_iter().find(|(named, _)| return *named == label) else {
        panic!("the fixture list holds no cause called {label}");
    };
    return cause;
}

/// The sentence one label's cause prints for a plan that destroyed nothing.
fn printed(label: &str) -> String {
    return cause(label).message(&RemovalHistory::NothingDestroyed);
}

#[test]
fn every_cause_prints_its_object_its_way_forward_and_no_destructive_command() {
    for (label, refusal) in every_cause() {
        let sentence: String = message(&refusal, &RemovalHistory::NothingDestroyed);
        assert!(
            sentence.starts_with("refusing to remove the ZFS worktree clone "),
            "{label} opens with the refusal and the clone: {sentence}"
        );
        assert!(
            sentence.contains("Ways forward: "),
            "{label} names the ways forward: {sentence}"
        );
        assert!(
            !sentence.contains("zfs destroy"),
            "{label} prints no destructive command: {sentence}"
        );
        assert!(
            !sentence.contains("zfs promote"),
            "{label} prints no command that moves a lineage: {sentence}"
        );
        assert!(
            sentence.contains(CLONE) || sentence.contains(SLOT),
            "{label} names the clone or the slot: {sentence}"
        );
    }
}

#[test]
fn the_look_form_claims_nothing_taken_down_and_says_there_is_nothing_to_cancel() {
    for label in [
        "marked-before-the-unmount",
        "unreadable-mark-before-the-unmount",
    ] {
        let sentence: String = printed(label);
        assert!(
            sentence.contains("This removal has not destroyed anything yet."),
            "{label} says nothing was destroyed: {sentence}"
        );
        assert!(
            sentence.contains("nothing is pending and there is nothing to cancel"),
            "{label} says nothing is pending: {sentence}"
        );
        assert!(
            !sentence.contains("the clone is unmounted"),
            "{label} reads no mount: {sentence}"
        );
        assert!(
            !sentence.contains("is the way back"),
            "{label} leaves nothing to go back to: {sentence}"
        );
        assert!(
            !sentence.contains("cancel-worktree-removal"),
            "{label} names no cancel command: {sentence}"
        );
    }
}

#[test]
fn the_checks_form_says_the_clone_is_unmounted_and_names_the_way_back() {
    for label in ["marked-at-a-check", "unreadable-mark-at-a-check"] {
        let sentence: String = printed(label);
        assert!(
            sentence.contains("This removal has not destroyed anything yet."),
            "{label} prints the plan's own history: {sentence}"
        );
        assert!(
            sentence.contains("the clone stays unmounted"),
            "{label} says the clone is unmounted: {sentence}"
        );
        assert!(sentence.contains("`cancel-worktree-removal fix-flaky-test` is the way back for a removal that is not forced"), "{label} names the way back: {sentence}");
        assert!(
            !sentence.contains("nothing is pending"),
            "{label} is not a refusal that leaves nothing: {sentence}"
        );
    }
}

#[test]
fn the_comparison_form_says_it_sent_no_request_and_what_it_could_not_compare() {
    for label in [
        "clone-gone",
        "origin-snapshot-gone",
        "read-failed-on-the-origin-snapshot",
    ] {
        let sentence: String = printed(label);
        assert!(
            sentence.contains("This check changed nothing, and it sent no request."),
            "{label} says no request was sent: {sentence}"
        );
        assert!(
            sentence.contains("The removal stays pending"),
            "{label} keeps the removal pending: {sentence}"
        );
        assert!(
            sentence.contains("reports the same mismatch rather than attaching anything"),
            "{label} scopes the cancel command: {sentence}"
        );
        assert!(
            !sentence.contains("is the way back"),
            "{label} does not promise the worktree back: {sentence}"
        );
        assert!(
            !sentence.contains("nothing is pending"),
            "{label} is not a refusal that leaves nothing: {sentence}"
        );
    }
}

#[test]
fn the_marked_cause_names_the_origin_its_value_and_what_the_destroy_does() {
    let sentence: String = printed("marked-at-a-check");
    assert!(
        sentence.contains(&format!(
            "its origin snapshot `{ORIGIN_SNAPSHOT}` is marked for deferred destroy"
        )),
        "{sentence}"
    );
    assert!(
        sentence.contains("`defer_destroy` reads 1"),
        "the value is named: {sentence}"
    );
    assert!(
        sentence.contains("the kernel reports 1 clone of it and 0 user holds"),
        "the two counts are named: {sentence}"
    );
    assert!(
        sentence
            .contains("so destroying this clone destroys that snapshot in the same transaction"),
        "the reap is named: {sentence}"
    );
    assert!(
        !sentence.contains("has not been measured"),
        "the message says what the kernel's code says, not a measurement: {sentence}"
    );
}

#[test]
fn the_identity_causes_name_both_values_and_the_field() {
    let replaced: String = printed("clone-not-the-planned-one");
    assert!(
        replaced.contains("the dataset at the planned clone's name"),
        "the object is named: {replaced}"
    );
    assert!(
        replaced.contains("recorded GUID `12345678901234567890`"),
        "the recorded value is named: {replaced}"
    );
    assert!(
        replaced.contains("the kernel reports `12345678901234567891`"),
        "the live value is named: {replaced}"
    );
    let snapshot: String = printed("origin-snapshot-createtxg-differs");
    assert!(
        snapshot.contains("created in transaction group 4243")
            && snapshot.contains("recorded 4242"),
        "both transaction groups are named: {snapshot}"
    );
    let dataset: String = printed("origin-dataset-not-the-planned-one");
    assert!(
        dataset.contains(&format!("the dataset at the origin's name `{CHECKOUT}`")),
        "the origin dataset is named: {dataset}"
    );
    let missing: String = printed("plan-incomplete");
    assert!(
        missing.contains(&format!(
            "the removal's plan at {PLAN_HOME} holds no line for the clone's dataset GUID"
        )),
        "{missing}"
    );
}

#[test]
fn the_history_sentence_follows_the_plans_outcomes_and_not_the_call_site() {
    let nothing: String = cause("marked-at-a-check").message(&RemovalHistory::NothingDestroyed);
    let lost: String = cause("marked-at-a-check").message(&RemovalHistory::of_destroyed(vec![
        String::from("fix-flaky-test@manual-2026-11-21"),
        String::from("fix-flaky-test@sanoid-hourly-2026-11-22"),
    ]));
    assert!(
        nothing.contains("This removal has not destroyed anything yet."),
        "{nothing}"
    );
    assert!(lost.contains("This clone has already lost `fix-flaky-test@manual-2026-11-21` and `fix-flaky-test@sanoid-hourly-2026-11-22`"), "{lost}");
    assert!(
        lost.contains("in this removal or an earlier one, and they do not come back"),
        "{lost}"
    );
    assert!(
        !lost.contains("This removal has not destroyed anything yet."),
        "a lost history never says nothing was destroyed: {lost}"
    );
    assert_eq!(nothing.replace("This removal has not destroyed anything yet.", "X"), lost.replace("This clone has already lost `fix-flaky-test@manual-2026-11-21` and `fix-flaky-test@sanoid-hourly-2026-11-22`, in this removal or an earlier one, and they do not come back.", "X"), "the two messages differ only in the history sentence");
}

#[test]
fn the_look_never_prints_a_loss() {
    let lost: String =
        cause("marked-before-the-unmount").message(&RemovalHistory::of_destroyed(vec![
            String::from("fix-flaky-test@manual"),
        ]));
    assert!(
        lost.contains("This removal has not destroyed anything yet."),
        "the look runs before anything can be destroyed: {lost}"
    );
    assert!(
        !lost.contains("has already lost"),
        "the look never names a loss: {lost}"
    );
}

#[test]
fn every_form_is_reached_by_a_cause_and_named_in_its_message() {
    let forms: Vec<RefusalForm> = every_cause()
        .into_iter()
        .map(|(_, refusal)| return refusal.form())
        .collect();
    for wanted in [
        RefusalForm::NothingTakenDown,
        RefusalForm::PendingAfterTheUnmount,
        RefusalForm::NothingCompared,
    ] {
        assert!(
            forms.contains(&wanted),
            "at least one cause takes the form {}",
            wanted.name()
        );
    }
    assert!(
        printed("marked-before-the-unmount").contains("This check changed nothing"),
        "a form A message says what the check did"
    );
    assert!(
        printed("marked-at-a-check").contains("This check changed nothing"),
        "a form B message says what the check did"
    );
    assert!(
        printed("clone-gone").contains("This check changed nothing"),
        "a form C message says what the check did"
    );
}
