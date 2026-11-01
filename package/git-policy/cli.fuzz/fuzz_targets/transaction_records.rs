//! What: Raw bytes and generated records through the journal, capture-order and sequence readers.
//! Why: Recovery acts on these files after a crash; what it accepts must mean one thing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkGeneratedJournal(data); checkJournalBytes(data); checkCaptureRecords(data); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared generators and checks.
use git_policy_cli_fuzz::journal::{
    check_capture_records, check_generated_journal, check_journal_bytes,
};
use libfuzzer_sys::fuzz_target;

// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    check_generated_journal(data);
    check_journal_bytes(data);
    check_capture_records(data);
});
