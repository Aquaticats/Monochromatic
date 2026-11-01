//! What: Exclude patterns through the one-rule linter configuration.
//! Why: The repository chooses the patterns; none may change the document's structure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => checkLinterConfig(data));
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared checks.
use git_policy_cli_fuzz::markdown::check_linter_config;
use libfuzzer_sys::fuzz_target;

// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    check_linter_config(data);
});
