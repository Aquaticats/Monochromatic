//! What:
//!  Raw and generated `rulesFile` option values through the value check.
//! Why:
//!  The value comes from repository configuration;
//!  an accepted one must name a file
//!      inside the repository whatever its words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkRulesFileValue(utf8(data)); checkRulesFileValue(generatedRulesFile(data)); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared generators and checks.
use git_policy_cli_fuzz::content::{check_rules_file_value, generated_rules_file};
use libfuzzer_sys::fuzz_target;

// Text inputs are checked as written; every input also builds a value from path words.
// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        check_rules_file_value(text);
    }
    check_rules_file_value(generated_rules_file(data).as_str());
});
