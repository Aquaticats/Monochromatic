//! What:
//!  Controls of the disposal of saved content on the component (plan step 2, after the step 1 review's finding 6): the person's explicit
//!  disposal appends a `disposed` line, makes a replacement root that no longer names the item durable with that line in its closure, and
//!  only then unlinks the content.
//! Why:
//!  Recovery validates every retained saved-content entry of a root, so a root that still names an item whose file is gone is ineligible
//!  whatever the ledger says. The order that keeps a root eligible at every power loss is: line durable, replacement root durable, unlink.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // disposeSavedContent(lane, state, authorization): appendDisposed; checkpointWithout(item); unlink(item)
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_state_log_tests::lane_with_envelope;
use git_policy_cli::commit_disposal::{DisposalAuthorization, dispose_saved_content, retire_saved_content, unlink_retired};
use git_policy_cli::commit_lane_lifecycle::capture_lane_closure;
use git_policy_cli::commit_state_log::{ClosureEntry, LaneDocument, StateWriter, create_saved_content, read, select};
use std::path::{Path, PathBuf};

/// A lane with one request and one saved content, and a checked first record that retains both.
fn lane_with_saved(label: &str) -> (PathBuf, PathBuf, String) {
    let (lane, state, _envelope) = lane_with_envelope(label);
    let saved: ClosureEntry = create_saved_content(&lane, b"objects of b").expect("saved content");
    let name: String = saved.path().file_name().expect("name").to_string_lossy().to_string();
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the first record");
    return (lane, state, name);
}

/// The names the newest root's document holds as saved content.
fn saved_in_root(state: &Path) -> Vec<String> {
    let (_generation, bytes) = read(state).expect("readable").expect("a root");
    let document: LaneDocument = LaneDocument::parse(&bytes).expect("a checked document");
    return document.saved().to_vec();
}

/// The whole procedure: the content is gone, the newest root does not name it, the ledger holds its `disposed` line, and recovery selects
/// the replacement root.
#[test]
fn the_disposal_leaves_an_eligible_root_without_the_item_and_a_disposed_line() {
    let (lane, state, name) = lane_with_saved("dispose-whole");
    assert_eq!(saved_in_root(&state), std::slice::from_ref(&name));
    dispose_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name)).expect("the disposal");
    assert!(!lane.join("saved").join(&name).exists(), "the content is removed");
    assert!(saved_in_root(&state).is_empty(), "the root no longer names it");
    let ledger: String = std::fs::read_to_string(lane.join("ledger")).expect("the ledger");
    assert!(ledger.contains(&format!(" {} disposed ", name.chars().take(32).collect::<String>())), "{}", ledger);
    assert!(select(&state).expect("readable").is_some(), "recovery selects a root");
}

/// A crash between the replacement root and the unlink leaves the content on disk and a root that does not name it; the unlink then
/// completes. Nothing is lost at either point.
#[test]
fn a_crash_after_the_replacement_root_and_before_the_unlink_loses_nothing() {
    let (lane, state, name) = lane_with_saved("dispose-crash");
    let retired = retire_saved_content(&lane, &state, &DisposalAuthorization::explicit(&name)).expect("steps up to the replacement root");
    assert!(lane.join("saved").join(&name).exists(), "the content is still there");
    assert!(saved_in_root(&state).is_empty());
    assert!(select(&state).expect("readable").is_some());
    unlink_retired(&lane, &retired).expect("the unlink");
    assert!(!lane.join("saved").join(&name).exists());
    assert!(select(&state).expect("readable").is_some());
}

/// The review's schedule, on the component: the content is unlinked first, with the `disposed` line durable and no replacement root. The
/// old root still names the item, its validation fails on the missing file, and recovery has no root.
#[test]
fn removing_the_content_before_the_replacement_root_leaves_recovery_with_no_root() {
    let (lane, state, name) = lane_with_saved("dispose-wrong-order");
    std::fs::remove_file(lane.join("saved").join(&name)).expect("the unlink that came first");
    assert!(!matches!(select(&state), Ok(Some(_))), "recovery selected a root that names content which is gone");
}

/// Only the person's explicit disposal of a name that the newest root holds is a disposal: another name is refused before anything is
/// written.
#[test]
fn a_disposal_of_content_the_root_does_not_hold_is_refused_before_anything_is_written() {
    let (lane, state, _name) = lane_with_saved("dispose-unknown");
    let other: String = "e".repeat(64);
    assert!(dispose_saved_content(&lane, &state, &DisposalAuthorization::explicit(&other)).is_err());
    assert!(!lane.join("ledger").exists(), "no line was appended for a name the root does not hold");
}
