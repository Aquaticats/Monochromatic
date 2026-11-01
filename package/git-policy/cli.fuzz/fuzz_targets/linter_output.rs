//! What: Raw bytes and generated records through the reader of one linter run.
//! Why: The linter is a child process; its exit status, fixed source and JSON Lines findings
//!      are untrusted, and only a run the contract accepts may reach a candidate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkLinterRun(data); checkGeneratedRecords(data); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared checks.
use git_policy_cli_fuzz::markdown::{check_generated_records, check_linter_run};
use libfuzzer_sys::fuzz_target;

// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    check_linter_run(data);
    check_generated_records(data);
});
