//! What:
//!  Raw bytes and generated text through the final-newline rule.
//! Why:
//!  File contents are arbitrary;
//!  the rule decides the bytes a direct fix writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkFinalNewline(data); checkFinalNewline(generatedText(data)); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared generators and checks.
use git_policy_cli_fuzz::content::{check_final_newline, generated_text};
use libfuzzer_sys::fuzz_target;

// Raw bytes are mostly not text; generated text reaches every correction.
// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    check_final_newline(data);
    check_final_newline(generated_text(data).as_slice());
});
