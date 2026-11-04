//! What:
//!  Controls of the real lane state document in the checked representation (round 23, plan step 1, owner decision 21 "Checked format"):
//!  the adoption order extent, the decision marks, the marker, the logical head and tail, the state of every live request and the
//!  consumption map.
//! Why:
//!  Every field is either a reference the component derives from the lane and checks against the closure (requests, saved content, marks,
//!  the ledger and order extents), or state with no file behind it. Only the first kind is a containment obligation of a reclaiming
//!  checkpoint; the second is superseded by the newer generation. The controls show both halves: a reference cannot be dropped, and a state
//!  change does not stop reclamation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a changed marker still reclaims, a dropped mark does not', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_state_log_tests::{lane_with_envelope, ledger_line_of};
use git_policy_cli::commit_state_log::{
    ClosureEntry, Consumption, ConsumptionKind, EntryKind, LaneDocument, LaneState, Marker, RequestPhase, RequestState, StateWriter, read, state_covers, sync_prefix,
};
use std::path::{Path, PathBuf};

/// The identity of the fixture's envelope: the name of the file.
fn identity_of(envelope: &Path) -> String {
    return envelope.file_name().expect("name").to_string_lossy().to_string();
}

/// Creates the withdrawal mark of `id` (an empty file in `decision/`) and captures it.
fn mark_of(lane: &Path, id: &str, suffix: &str) -> ClosureEntry {
    std::fs::create_dir_all(lane.join("decision")).expect("decision directory");
    let path: PathBuf = lane.join("decision").join(format!("{}.{}", id, suffix));
    std::fs::write(&path, b"").expect("a mark");
    return ClosureEntry::capture(lane, &path, EntryKind::Mark).expect("capture the mark");
}

/// Appends a line to the lane's order file (adoption order) and captures all of it.
fn order_of(lane: &Path, line: &str) -> ClosureEntry {
    let path: PathBuf = lane.join("order");
    let mut text: Vec<u8> = std::fs::read(&path).unwrap_or_default();
    text.extend_from_slice(line.as_bytes());
    std::fs::write(&path, text).expect("an order file");
    return ClosureEntry::capture(lane, &path, EntryKind::AppendPrefix).expect("capture the order file");
}

/// A document that carries every kind of state, for one request `id`.
fn full_state(id: &str, position: u64) -> LaneState {
    return LaneState {
        marker: Marker::Materializing,
        head: Some("a".repeat(40)),
        tail: vec![id.to_string()],
        requests: vec![RequestState { id: id.to_string(), phase: RequestPhase::Publishing, resume: "P1".to_string(), position: Some(position) }],
        consumption: vec![Consumption { path: b"dir/f.txt".to_vec(), kind: ConsumptionKind::Install, request: id.to_string(), version: 1, entry: vec![1, 2, 3] }],
    };
}

/// The references of the fixture and the document that names them with `state`.
fn document_with(entries: &[ClosureEntry], state: LaneState) -> LaneDocument {
    return LaneDocument::from_closure(entries).expect("a closure the representation can express").with_state(state);
}

/// The full real state encodes and parses back to itself, and the encoding is canonical: a reordered, repeated, unknown or dangling line is
/// not a document (never read as a smaller one).
#[test]
fn r23_a_full_real_state_document_round_trips_and_only_the_canonical_bytes_parse() {
    let (lane, _state, envelope) = lane_with_envelope("r23-round-trip");
    let id: String = identity_of(&envelope);
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let mark: ClosureEntry = mark_of(&lane, &id, "withdraw");
    let order: ClosureEntry = order_of(&lane, "adopt a\n");
    let document: LaneDocument = document_with(&[request, mark, order], full_state(&id, 0));
    let bytes: Vec<u8> = document.encode();
    assert_eq!(LaneDocument::parse(&bytes), Some(document.clone()), "the document did not read back as itself");
    let text: String = String::from_utf8(bytes.clone()).expect("text");
    let lines: Vec<&str> = text.lines().collect();
    let mut swapped: Vec<&str> = lines.clone();
    swapped.swap(1, 2);
    assert_ne!(swapped, lines);
    for (label, changed) in [
        ("two lines swapped", format!("{}\n", swapped.join("\n"))),
        ("a repeated line", format!("{}{}\n", text, lines[1])),
        ("an unknown line", format!("{}unknown x\n", text)),
        ("a state for a request the document does not adopt", format!("{}state {} bound B1 -\n", text, "f".repeat(32))),
        ("a tail entry that is not adopted", format!("{}tail {}\n", text, "f".repeat(32))),
        ("a terminal phase", text.replace(" publishing ", " retired ")),
        ("a head that is not an object name", text.replace(&"a".repeat(40), "xyz")),
        ("a marker word that is unknown", text.replace("marker materializing", "marker sideways")),
    ] {
        assert!(LaneDocument::parse(changed.as_bytes()).is_none(), "{} was read as a document", label);
    }
}

/// A position names a line of the adoption order the document relies on: it must lie inside the extent, so a document cannot name a request's
/// adopt line past the order prefix it retains.
#[test]
fn r23_a_request_position_beyond_the_order_extent_is_not_a_document() {
    let (lane, _state, envelope) = lane_with_envelope("r23-position");
    let id: String = identity_of(&envelope);
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let order: ClosureEntry = order_of(&lane, "adopt a\n");
    let inside: LaneDocument = document_with(&[request.clone(), order.clone()], full_state(&id, 0));
    assert!(LaneDocument::parse(&inside.encode()).is_some());
    let beyond: LaneDocument = document_with(&[request.clone(), order.clone()], full_state(&id, order.length()));
    assert!(LaneDocument::parse(&beyond.encode()).is_none(), "a position past the end of the order extent was accepted");
    let without_order: LaneDocument = document_with(&[request], full_state(&id, 0));
    assert!(LaneDocument::parse(&without_order.encode()).is_none(), "a position with no order extent was accepted");
}

/// Marks are references: a reclaiming checkpoint whose closure lacks a decision mark the lane holds is refused, because the predecessor it
/// removes is the only record that still retains the stop disposition of that request.
#[test]
fn r23_a_closure_lacking_a_decision_mark_the_lane_holds_is_refused() {
    let (lane, state, envelope) = lane_with_envelope("r23-mark-obligation");
    let id: String = identity_of(&envelope);
    StateWriter::acquire(&state).expect("writer").append(1, &LaneDocument::default().encode()).expect("a checked first record");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let mark: ClosureEntry = mark_of(&lane, &id, "withdraw");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    let without: LaneDocument = document_with(std::slice::from_ref(&request), LaneState::default());
    assert!(writer.begin_checkpoint_reclaiming(2, &without, std::slice::from_ref(&request)).is_err(), "a mark the lane holds was left out");
    assert!(state.join("state.log.1").exists());
    let both: Vec<ClosureEntry> = vec![request, mark];
    writer.checkpoint_reclaiming(2, &document_with(&both, LaneState::default()), &both).expect("with the mark it is accepted");
    assert!(!state.join("state.log.1").exists());
}

/// A mark must be the empty file of a live request: a mark with content is not a mark, and a document naming a mark in its closure that its
/// set lacks is neither admitted nor selected.
#[test]
fn r23_a_mark_with_content_is_not_a_mark_and_a_named_mark_must_be_in_the_set() {
    let (lane, state, envelope) = lane_with_envelope("r23-mark-validity");
    let id: String = identity_of(&envelope);
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    std::fs::create_dir_all(lane.join("decision")).expect("decision directory");
    let path: PathBuf = lane.join("decision").join(format!("{}.withdraw", id));
    std::fs::write(&path, b"not empty").expect("a file with content at the mark name");
    assert!(ClosureEntry::capture(&lane, &path, EntryKind::Mark).is_err(), "a mark with content was captured");
    let other: PathBuf = lane.join("decision").join("notes.withdraw");
    std::fs::write(&other, b"").expect("a file with another name");
    assert!(ClosureEntry::capture(&lane, &other, EntryKind::Mark).is_err(), "a name that is not an identity plus a mark suffix was captured");
    std::fs::write(&path, b"").expect("empty again");
    let mark: ClosureEntry = ClosureEntry::capture(&lane, &path, EntryKind::Mark).expect("capture");
    let document: Vec<u8> = document_with(&[request.clone(), mark], LaneState::default()).encode();
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    assert!(writer.append_with(1, &document, std::slice::from_ref(&request)).is_err(), "a document naming a mark was admitted without it in the set");
}

/// A mark is retained until its request is discharged: a checkpoint that drops it without a ledger line for the request inside its own
/// closure is refused and the segment that promised it stays; with the line, the checkpoint reclaims the segment and covers the promise.
#[test]
fn r23_a_mark_is_retained_until_its_request_is_discharged_by_the_ledger() {
    let (lane, state, envelope) = lane_with_envelope("r23-mark-dropped");
    let id: String = identity_of(&envelope);
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let mark: ClosureEntry = mark_of(&lane, &id, "withdraw");
    let both: Vec<ClosureEntry> = vec![request.clone(), mark.clone()];
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    writer.append_with(1, &document_with(&both, LaneState::default()).encode(), &both).expect("a record that promises the mark");
    let promised = sync_prefix(&state).expect("acknowledged");
    let only: Vec<ClosureEntry> = vec![request.clone()];
    assert!(writer.begin_checkpoint_reclaiming(2, &document_with(&only, LaneState::default()), &only).is_err(), "a checkpoint dropped a mark that no ledger line discharges");
    assert!(state.join("state.log.1").exists(), "the segment whose record promised the mark was reclaimed");
    assert!(state_covers(&state, &promised).expect("readable"), "the acknowledgment is still covered by the log that holds it");
    std::fs::write(lane.join("ledger"), ledger_line_of(&id, "withdrawn")).expect("a ledger line that discharges the request");
    let ledger: ClosureEntry = ClosureEntry::capture(&lane, &lane.join("ledger"), EntryKind::AppendPrefix).expect("capture the ledger");
    let discharged: Vec<ClosureEntry> = vec![ledger];
    assert!(writer.begin_checkpoint_reclaiming(3, &document_with(&discharged, LaneState::default()), &discharged).is_err(), "a retirement the writer did not name was accepted");
    let candidate = writer
        .begin_checkpoint_reclaiming_retiring(3, &document_with(&discharged, LaneState::default()), &discharged, std::slice::from_ref(&id))
        .expect("a checkpoint that names and proves the discharge of the request");
    writer.complete_checkpoint(&candidate).expect("completion");
    assert!(!state.join("state.log.1").exists(), "a discharged request still held its segment");
    assert!(state_covers(&state, &promised).expect("readable"), "the checkpoint that discharged the request did not cover the acknowledgment");
}

/// The order extent is a reference: a closure shorter than the order file the lane holds is refused, a document naming a longer extent than its
/// set holds is not admitted, and a newer document that relies on a shorter extent does not contain the older one.
#[test]
fn r23_the_order_extent_is_a_reference_the_closure_and_the_newer_document_must_hold() {
    let (lane, state, envelope) = lane_with_envelope("r23-order-extent");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let short: ClosureEntry = order_of(&lane, "adopt a\n");
    let long: ClosureEntry = order_of(&lane, "adopt b\n");
    assert!(long.length() > short.length());
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    let with_long: Vec<ClosureEntry> = vec![request.clone(), long];
    writer.append_with(1, &document_with(&with_long, LaneState::default()).encode(), &with_long).expect("a record that relies on the long extent");
    let with_short: Vec<ClosureEntry> = vec![request.clone(), short];
    assert!(writer.begin_checkpoint_reclaiming(2, &document_with(&with_short, LaneState::default()), &with_short).is_err(), "a closure shorter than the order file the lane holds was accepted");
    let lengthened: Vec<ClosureEntry> = vec![request.clone(), order_of(&lane, "adopt c\n")];
    let admission = writer.append_with(2, &document_with(&lengthened, LaneState::default()).encode(), std::slice::from_ref(&request));
    assert!(admission.is_err(), "a document naming an order extent its set lacks was admitted");
    writer.checkpoint_reclaiming(2, &document_with(&lengthened, LaneState::default()), &lengthened).expect("with the extent in its set it is accepted");
    assert!(!state.join("state.log.1").exists(), "the contained predecessor was not reclaimed");
}

/// State fields are not containment obligations: a newer document with another marker, head, tail, request phase and consumption map still
/// reclaims the predecessor and covers its acknowledgment, because the newer generation supersedes them and no file lies behind them. This is
/// the control that shows the representation does not stop reclamation for every state change.
#[test]
fn r23_a_changed_state_still_reclaims_and_covers_because_state_has_no_file_behind_it() {
    let (lane, state, envelope) = lane_with_envelope("r23-state-changes");
    let id: String = identity_of(&envelope);
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let order: ClosureEntry = order_of(&lane, "adopt a\n");
    let entries: Vec<ClosureEntry> = vec![request, order];
    let before: LaneDocument = document_with(&entries, full_state(&id, 0));
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    writer.append_with(1, &before.encode(), &entries).expect("the state before");
    let promised = sync_prefix(&state).expect("acknowledged");
    let after_state: LaneState = LaneState {
        marker: Marker::None,
        head: Some("b".repeat(40)),
        tail: Vec::new(),
        requests: vec![RequestState { id: id.clone(), phase: RequestPhase::Published, resume: "T1".to_string(), position: Some(0) }],
        consumption: vec![Consumption { path: b"dir/f.txt".to_vec(), kind: ConsumptionKind::Suppress, request: id, version: 1, entry: vec![9] }],
    };
    let after: LaneDocument = document_with(&entries, after_state);
    assert_ne!(before, after);
    writer.checkpoint_reclaiming(2, &after, &entries).expect("a changed state is a reclaiming checkpoint");
    assert!(!state.join("state.log.1").exists(), "a changed marker, head or consumption map kept the predecessor");
    assert!(state_covers(&state, &promised).expect("readable"), "the checkpoint with the changed state did not cover the acknowledgment");
    assert_eq!(LaneDocument::parse(&read(&state).expect("readable").expect("state").1), Some(after));
}

/// The equality that reclaiming demands is about references: a document whose state fields name more than its closure holds (a state for a
/// request the closure lacks) is not a document, and a document with the same references and any state matches its closure.
#[test]
fn r23_a_reclaiming_checkpoint_compares_references_not_state() {
    let (lane, state, envelope) = lane_with_envelope("r23-projection");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    let stateful: LaneDocument = document_with(std::slice::from_ref(&request), LaneState { marker: Marker::Recovering, ..LaneState::default() });
    writer.checkpoint_reclaiming(1, &stateful, std::slice::from_ref(&request)).expect("any state over the same references");
    let dangling: LaneDocument = document_with(std::slice::from_ref(&request), full_state(&"f".repeat(32), 0));
    assert!(LaneDocument::parse(&dangling.encode()).is_none());
    assert!(writer.begin_checkpoint_reclaiming(2, &dangling, std::slice::from_ref(&request)).is_err(), "a document with a state for a request it does not adopt was reclaimed");
}
