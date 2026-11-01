//! What: Controls proving the journal and capture generators reach every record kind and edit.
//! Why: An invariant that is never reached proves nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const data of inputs()) checkGeneratedJournal(data);
//! ```

/// Import the generators and invariants under control.
use super::{
    check_capture_records, check_generated_journal, check_journal_bytes, generated_landing,
    generated_preparing,
};
use crate::owners::Cursor;
use git_policy_cli::transaction_journal::{Conclusion, LandingOperation, SymbolicHead};
use git_policy_cli::transaction_journal_encode::{encode_landing, encode_preparing};

/// Inputs shared with the owner controls.
fn inputs() -> Vec<Vec<u8>> {
    return crate::owners::tests::inputs();
}

/// Every record kind round-trips and every refusing edit is applied.
#[test]
fn journal_records_reach_every_kind_and_edit() {
    let mut normalize: usize = 0;
    let mut detached: usize = 0;
    let mut merges: usize = 0;
    for data in inputs() {
        check_generated_journal(data.as_slice());
        check_capture_records(data.as_slice());
        check_journal_bytes(data.as_slice());
        let mut cursor: Cursor<'_> = Cursor::new(data.as_slice());
        let preparing = generated_preparing(&mut cursor);
        if preparing.symbolic_head == SymbolicHead::Detached {
            detached += 1;
        }
        if preparing.conclusion == Conclusion::Merge {
            merges += 1;
        }
        let landing = generated_landing(&mut cursor);
        if landing.operation == LandingOperation::NormalizeOnly {
            normalize += 1;
        }
        // The raw-bytes invariant also runs on real encoder output.
        check_journal_bytes(encode_preparing(&preparing).as_bytes());
        check_journal_bytes(encode_landing(&landing).as_bytes());
    }
    assert!(
        normalize > 100 && detached > 100 && merges > 50,
        "{normalize} {detached} {merges}"
    );
}

/// Hard cases: an empty input and records with a byte-order mark.
#[test]
fn hard_cases_hold() {
    check_generated_journal(b"");
    check_capture_records(b"");
    check_journal_bytes(
        b"\xef\xbb\xbf{\"schemaVersion\":2,\"state\":\"ref-updated\",\"landedOid\":\"x\"}\n",
    );
    check_capture_records(b"0\n");
}
