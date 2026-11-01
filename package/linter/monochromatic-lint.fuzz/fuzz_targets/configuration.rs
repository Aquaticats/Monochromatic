//! What:
//!  Raw and structured configuration inputs in the same coverage-guided target.
//! Why:
//!  Every byte input reaches valid merging as well as attempting the malformed-text boundary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkGeneratedConfig(data); checkRawConfig(data); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared checks.
use libfuzzer_sys::fuzz_target;
use monochromatic_jsonc_edit::{JsoncValue, emit_jsonc_value};
use monochromatic_lint_fuzz::{check_configuration, generated_configuration};

// Run valid generated settings for every draw, including byte strings that are not UTF-8.
fuzz_target!(|data: &[u8]| {
    let generated: JsoncValue = generated_configuration(data);
    let encoded: String = emit_jsonc_value(&generated);
    check_configuration(encoded.as_str());
    if let Ok(raw_source) = std::str::from_utf8(data) {
        check_configuration(raw_source);
    }
});
