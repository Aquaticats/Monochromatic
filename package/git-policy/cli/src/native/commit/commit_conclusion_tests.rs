//! Controls for the fix summary the conclusion appends.

use super::*;
use crate::event_path::EventPath;
use crate::policy_trigger::Trigger;

/// Paths of a summary event, as text.
fn summary_paths(events: &[PolicyEvent]) -> Vec<String> {
    match events.last() {
        Some(PolicyEvent::FixSummary { changed_paths, trigger, .. }) => {
            assert_eq!(*trigger, Trigger::PreForward);
            return changed_paths.iter().map(|path: &EventPath| { return String::from(path.text()); }).collect();
        }
        other => panic!("expected a fix summary, got {other:?}"),
    }
}

/// No pass changed anything, so the events stay as they were.
#[test]
fn no_changed_pass_adds_no_summary() {
    assert_eq!(events_with_changed_summary(Vec::new(), 0, &[b"a.txt".to_vec()]), Vec::<PolicyEvent>::new());
}

/// A changed pass adds one summary naming each path in byte order.
#[test]
fn a_changed_pass_adds_a_sorted_summary() {
    let events: Vec<PolicyEvent> =
        events_with_changed_summary(Vec::new(), 2, &[b"b.txt".to_vec(), b"B.txt".to_vec(), b"a.txt".to_vec()]);
    assert_eq!(events.len(), 1);
    assert_eq!(summary_paths(&events), vec!["B.txt", "a.txt", "b.txt"]);
}

/// The unconditional summary also appears for zero passes and keeps the events before it.
#[test]
fn the_unconditional_summary_follows_the_settled_events() {
    let first: PolicyEvent = PolicyEvent::CommitLanded { oid: String::from("x") };
    let events: Vec<PolicyEvent> = events_with_summary(vec![first.clone()], 0, &[]);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], first);
    assert_eq!(summary_paths(&events[..]).len(), 0);
    match events.last() {
        Some(PolicyEvent::FixSummary { passes, .. }) => assert_eq!(*passes, 0),
        other => panic!("expected a fix summary, got {other:?}"),
    }
}

/// A name that is not UTF-8 keeps its exact bytes in the summary,
///  and the order is the order of the bytes, not of the readable text:
///  the byte 0x80 sorts before the first byte of "é" although its replacement character sorts after it.
#[test]
fn a_summary_keeps_the_exact_bytes_of_a_name_that_is_not_utf8() {
    let events: Vec<PolicyEvent> =
        events_with_summary(Vec::new(), 1, &["aé".as_bytes().to_vec(), b"a\x80".to_vec(), b"a.txt".to_vec()]);
    match events.last() {
        Some(PolicyEvent::FixSummary { changed_paths, .. }) => {
            let exact: Vec<Option<&[u8]>> = changed_paths.iter().map(EventPath::exact).collect();
            assert_eq!(exact, vec![None, Some(b"a\x80".as_slice()), None]);
            assert_eq!(summary_paths(&events), vec!["a.txt", "a\u{fffd}", "aé"]);
        }
        other => panic!("expected a fix summary, got {other:?}"),
    }
}
