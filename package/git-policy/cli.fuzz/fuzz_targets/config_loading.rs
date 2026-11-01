//! What:
//!  Raw and generated argument vectors through the configuration-loading invariants.
//! Why:
//!  Skipping configuration skips policy,
//!  so the classification is checked over byte-valued arguments.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkConfigLoading(argumentsFromBytes(data)); checkConfigLoading(generatedArguments(data)); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared generators and checks.
use git_policy_cli_fuzz::arguments::{
    arguments_from_bytes, check_config_loading, generated_arguments,
};
use libfuzzer_sys::fuzz_target;
use std::ffi::OsString;

// Run both views of every input: bytes split at NUL, and bytes mapped to real Git tokens.
// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    let raw: Vec<OsString> = arguments_from_bytes(data);
    check_config_loading(raw.as_slice());
    let generated: Vec<OsString> = generated_arguments(data);
    check_config_loading(generated.as_slice());
});
