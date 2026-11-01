//! What:
//!  Raw and generated argument vectors through the global-argument layout invariants.
//! Why:
//!  Raw bytes cover arbitrary argument content;
//!  generated tokens reach every option form.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkGlobalLayout(argumentsFromBytes(data)); checkGlobalLayout(generatedArguments(data)); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared generators and checks.
use git_policy_cli_fuzz::arguments::{
    arguments_from_bytes, check_global_layout, generated_arguments,
};
use libfuzzer_sys::fuzz_target;
use std::ffi::OsString;

// Run both views of every input: bytes split at NUL, and bytes mapped to real Git tokens.
// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    let raw: Vec<OsString> = arguments_from_bytes(data);
    check_global_layout(raw.as_slice());
    let generated: Vec<OsString> = generated_arguments(data);
    check_global_layout(generated.as_slice());
});
