//! What:
//! Raw bytes and generated findings through the reader of one in-process linter run.
//! Why:
//! The linter's failure status,
//! fixed source and findings are untrusted values,
//! and only a
//!      run the contract accepts may reach a candidate.
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
