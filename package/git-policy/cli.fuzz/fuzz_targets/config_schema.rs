//! What:
//!  Generated and raw configuration documents through the JSONC schema invariants.
//! Why:
//!  Generated documents reach accepted settings with a known expectation;
//!  raw text exercises rejection.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkGeneratedConfig(generatedConfig(data)); checkConfigSource(utf8(data)); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared generators and checks.
use git_policy_cli_fuzz::configuration::{
    GeneratedConfig, check_config_source, check_generated_config, generated_config,
};
use libfuzzer_sys::fuzz_target;

// Every input yields one valid generated document; inputs that are UTF-8 are also parsed as written.
// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    let generated: GeneratedConfig = generated_config(data);
    check_generated_config(&generated);
    if let Ok(source) = std::str::from_utf8(data) {
        check_config_source(source);
    }
});
