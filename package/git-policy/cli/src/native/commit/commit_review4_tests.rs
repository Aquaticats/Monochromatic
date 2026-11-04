//! What:
//!  Controls for the fourth round of review of plan step 2 and step 3 (component tests, code against whole files, models and map): the
//!  disposal revalidation against the whole retained set, the bounded doorbell, and the false-green windows of the earlier tests, each with
//!  its guard removed in the proof of the round.
//! Why:
//!  A retained set may be larger than the document's references (the public append admits an over-approximation), and recovery's eligibility
//!  depends on every retained entry; the doorbell is on the client's path and may not wait for ever.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a newer record that retains the item only in its closure refuses the stale unlink', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_state_log_tests::lane_with_envelope;
use git_policy_cli::commit_disposal::{DisposalAuthorization, retire_saved_content, unlink_retired};
use git_policy_cli::commit_lane_lifecycle::capture_lane_closure;
use git_policy_cli::commit_state_log::{ClosureEntry, LaneDocument, StateWriter, create_saved_content, select};
use std::path::PathBuf;

/// A lane with a request and saved content retained by a first checked record. Returns the lane, the state directory and the content name.
fn lane_with_saved(label: &str) -> (PathBuf, PathBuf, String) {
    let (lane, state, _envelope) = lane_with_envelope(label);
    let saved: ClosureEntry = create_saved_content(&lane, label.as_bytes()).expect("saved content");
    let name: String = saved.path().file_name().expect("name").to_string_lossy().to_string();
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the first record");
    return (lane, state, name);
}

/// Review of step 3 (code, 1): a newer record whose document does not list the retired item but whose retained set does is legal (the
/// retained set may over-approximate the document). The stale token must be refused against the retained set, not the document alone,
/// or the unlink makes that newer record ineligible and recovery falls back, discarding its state.
#[test]
fn a_newer_record_that_retains_the_item_only_in_its_closure_refuses_the_stale_unlink() {
    let (lane, state, name) = lane_with_saved("rev4-closure-only");
    let retired = retire_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name)).expect("the retirement");
    let saved: ClosureEntry = ClosureEntry::capture_sealed_saved(&lane, &lane.join("saved").join(&name)).expect("the item is still on disk");
    let mut items: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&items).expect("a document that does not list the item");
    items.push(saved);
    let generation: u64 = select(&state).expect("readable").expect("root").generation + 1;
    StateWriter::acquire(&state).expect("writer").append_with(generation, &document.encode(), &items).expect("an over-approximating record is admitted");
    let refused = unlink_retired(retired);
    assert!(refused.is_err(), "the stale token deleted content that the newest record retains in its closure");
    assert!(lane.join("saved").join(&name).exists());
    assert_eq!(select(&state).expect("readable").expect("a root").generation, generation, "the newest record is still the root");
}
