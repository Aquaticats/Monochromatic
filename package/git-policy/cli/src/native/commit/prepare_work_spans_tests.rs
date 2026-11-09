//! What:
//!  The preparation's named steps:
//! their aggregation by name,
//! their order,
//! their bound,
//! what the timing record writes,
//! and that taking them empties the ledger.
//! Why:
//!   A record that summed every run under one name would hide a single long policy behind many short ones,
//! and a list that grew with the work would grow the timing file without end;
//! so the count,
//! the longest run,
//! the order and the bound are each asked here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // noteSpan('index copy', 12); noteSpan('index copy', 30); expect(drainSpans()).toEqual([[{ name: 'index copy', calls: 2, total: 42, longest: 30 }], 0]);
//! ```

use super::*;

/// The spans this process noted,
/// which a control starts with none of.
fn noted() -> (Vec<PrepareSpan>, usize) {
    return drain_spans();
}

/// Runs under one name are added to one entry,
/// with how often,
/// the whole of their time and the longest of them,
/// and the entries keep the order their names were first noted in,
/// so a reader sees the work in the order the preparation did it.
#[test]
fn runs_under_one_name_are_added_to_one_entry_in_the_order_first_noted() {
    assert_eq!(noted(), (Vec::new(), 0), "a control starts with no step");
    note_span("index copy", Duration::from_millis(12));
    note_span("index stat check", Duration::from_millis(31));
    note_span("index copy", Duration::from_millis(30));
    note_span("index copy", Duration::from_millis(4));
    let taken: (Vec<PrepareSpan>, usize) = noted();
    assert_eq!(
        taken.0,
        vec![
            PrepareSpan { name: String::from("index copy"), calls: 3, total: Duration::from_millis(46), longest: Duration::from_millis(30) },
            PrepareSpan { name: String::from("index stat check"), calls: 1, total: Duration::from_millis(31), longest: Duration::from_millis(31) },
        ]
    );
    assert_eq!(taken.1, 0);
    assert_eq!(noted(), (Vec::new(), 0), "the record takes the steps of its own preparation");
}

/// A step that took no measurable time is still a step that ran,
/// so a reader never tells "it did not run" from "it was fast".
#[test]
fn a_step_of_no_measurable_time_is_stated_as_one_call() {
    note_span("index read", Duration::ZERO);
    let taken: (Vec<PrepareSpan>, usize) = noted();
    assert_eq!(
        taken.0,
        vec![PrepareSpan { name: String::from("index read"), calls: 1, total: Duration::ZERO, longest: Duration::ZERO }]
    );
}

/// The list is bounded by its number of names:
/// a name noted after the list is full is counted,
/// not kept,
/// and the count is what the record states,
/// so a record never grows with the work.
#[test]
fn names_after_the_bound_are_counted_and_not_kept() {
    for index in 0..MAX_SPAN_NAMES {
        note_span(format!("step {index}").as_str(), Duration::from_millis(1));
    }
    note_span("step past the bound", Duration::from_millis(1));
    note_span("another step past the bound", Duration::from_millis(1));
    // A name already listed keeps counting its runs: the bound is on names, not on the work.
    note_span("step 0", Duration::from_millis(2));
    let taken: (Vec<PrepareSpan>, usize) = noted();
    assert_eq!(taken.0.len(), MAX_SPAN_NAMES);
    assert_eq!(taken.1, 2);
    assert_eq!(taken.0.first().map(|span| { return (span.calls, span.total); }), Some((2, Duration::from_millis(3))));
    assert!(!taken.0.iter().any(|span| { return span.name == "step past the bound"; }));
}

/// The record states an empty list for a preparation that noted nothing,
/// and a name holding a quote or a backslash stays inside its JSON string.
#[test]
fn the_record_states_the_list_and_escapes_a_name() {
    assert_eq!(spans_json(&[]), "[]");
    let spans: Vec<PrepareSpan> = vec![PrepareSpan {
        name: String::from("policy a\"b\\c"),
        calls: 2,
        total: Duration::from_millis(42),
        longest: Duration::from_millis(30),
    }];
    assert_eq!(spans_json(spans.as_slice()), "[{\"name\":\"policy a\\\"b\\\\c\",\"calls\":2,\"totalMs\":42,\"longestMs\":30}]");
    let parsed: serde_json::Value = serde_json::from_str(spans_json(spans.as_slice()).as_str()).expect("JSON");
    assert_eq!(parsed[0]["name"], "policy a\"b\\c");
}

/// A policy's span names its registry name and the lifecycle point whose pass ran it,
/// so a preparation's own pass reads apart from the post-commit gate's,
/// which spends the gate's milliseconds and not the preparation's.
#[test]
fn a_policy_span_names_the_policy_and_the_lifecycle_point() {
    use crate::policy_trigger::Trigger;
    assert_eq!(policy_span_name("security/forbidden-strings", Trigger::PreForward), "policy security/forbidden-strings at pre-forward");
    assert_eq!(policy_span_name("final-newline", Trigger::PostCommit), "policy final-newline at post-commit");
}
