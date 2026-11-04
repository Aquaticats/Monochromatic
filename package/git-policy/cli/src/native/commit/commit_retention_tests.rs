//! What:
//!  Controls of what recomputation needs and where it is retained (plan step 2, item "spool and attempt data"): the inbox envelope is one
//!  checksummed file holding the request record and both sides of every change, so it is the retained input of a recomputation; a checked
//!  document keeps it until the ledger discharges it, and an outcome that promises saved inputs (`withdrawn`, `conflicted`, `unresolved`,
//!  `rewritten-away`) does not discharge it until a `disposed` line does.
//! Why:
//!  The design's retention rule: "an outcome that promises saved inputs leaves the dependency live until a `disposed` line in the ledger
//!  discharges it". Until now any ledger line for the request discharged its envelope, so a seal after a conflict could drop the only copy of
//!  the bytes that a retry or a recomputation needs. The spool directories and the attempt directories are not inputs of a recomputation:
//!  the spool is the revision 11 form that the envelope replaced, and an attempt directory holds the effects of one attempt (a lost one makes
//!  recovery start a fresh attempt, decision 19).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a conflicted request keeps its envelope until it is disposed', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_state_log_tests::{lane_with_envelope, ledger_line_of};
use git_policy_cli::commit_inbox::decode_inbox_file;
use git_policy_cli::commit_integrity::sha256_hex;
use git_policy_cli::commit_state_log::{ClosureEntry, EntryKind, LaneDocument, LaneState, StateWriter};
use std::path::{Path, PathBuf};

/// The identity of the fixture's envelope.
fn identity_of(envelope: &Path) -> String {
    return envelope.file_name().expect("name").to_string_lossy().to_string();
}

/// A lane with an envelope retained by a checked first record. Returns the lane, the state directory, the identity and the writer.
fn lane_retaining_a_request(label: &str) -> (PathBuf, PathBuf, String) {
    let (lane, state, envelope) = lane_with_envelope(label);
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(std::slice::from_ref(&request)).expect("document");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), std::slice::from_ref(&request)).expect("the record that retains the request");
    return (lane.clone(), state, identity_of(&envelope));
}

/// Tries to drop the request from a reclaiming checkpoint whose ledger holds `lines`; whether it was accepted.
fn drop_with_ledger(label: &str, outcomes: &[&str]) -> bool {
    let (lane, state, id) = lane_retaining_a_request(label);
    let mut ledger: Vec<u8> = Vec::new();
    for outcome in outcomes {
        ledger.extend_from_slice(&ledger_line_of(&id, outcome));
    }
    std::fs::write(lane.join("ledger"), ledger).expect("the ledger");
    let held: ClosureEntry = ClosureEntry::capture(&lane, &lane.join("ledger"), EntryKind::AppendPrefix).expect("capture the ledger");
    let closure: Vec<ClosureEntry> = vec![held];
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("document").with_state(LaneState::default());
    let mut writer: StateWriter = StateWriter::acquire(&state).expect("writer");
    return writer.begin_checkpoint_reclaiming_retiring(2, &document, &closure, std::slice::from_ref(&id)).is_ok();
}

/// An outcome that promises nothing saved discharges the envelope; one that promises saved inputs does not, until a `disposed` line does.
#[test]
fn an_envelope_is_discharged_by_a_saved_free_outcome_and_kept_by_one_that_promises_saved_inputs() {
    for outcome in ["published", "refused", "already", "superseded"] {
        assert!(drop_with_ledger("retain-free", &[outcome]), "{} promises nothing saved, so it discharges the envelope", outcome);
    }
    for outcome in ["withdrawn", "conflicted", "unresolved", "rewritten-away"] {
        assert!(!drop_with_ledger("retain-saved", &[outcome]), "{} promises saved inputs, and the envelope was dropped", outcome);
        assert!(drop_with_ledger("retain-disposed", &[outcome, "disposed"]), "{} then disposed discharges it", outcome);
    }
}

/// What a recomputation needs is the retained envelope and nothing else: with no spool directory and no attempt directory in the lane, the
/// envelope decodes to the request and both sides of every change, each matching the hash the record promises.
#[test]
fn the_retained_envelope_alone_holds_the_bytes_a_recomputation_needs() {
    let (lane, _state, envelope) = lane_with_envelope("retain-recompute");
    assert!(!lane.join("spool").exists() && !lane.join("p2").exists(), "the lane has neither a spool nor an attempt directory");
    let (request, news, olds) = decode_inbox_file(&std::fs::read(&envelope).expect("the envelope")).expect("decodes");
    for (number, change) in request.changes.iter().enumerate() {
        if change.mode != "000000" {
            assert_eq!(sha256_hex(&news[number]), change.sha256, "new side of change {}", number);
        }
        if change.old_mode != "000000" {
            assert_eq!(sha256_hex(&olds[number]), change.old_sha256, "old side of change {}", number);
        }
    }
}
