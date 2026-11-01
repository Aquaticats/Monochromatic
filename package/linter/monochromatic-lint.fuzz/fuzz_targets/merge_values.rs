//! What:
//!  Coverage-guided structured inputs for the native settings merge.
//! Why:
//!  The existing JSONC generator supplies valid trees instead of relying on accidental valid byte strings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(generatedDocument, document => checkMerge(parseJsonc(document.source)));
//! ```
#![no_main]

/// Import the existing structured generator,
///  native parser and verified invariants.
use jsonc_edit_fuzz::GeneratedDocument;
use libfuzzer_sys::fuzz_target;
use monochromatic_jsonc_edit::{JsoncValue, parse_jsonc};
use monochromatic_lint_fuzz::check_merge;

// The macro supplies libFuzzer's executable entry and decodes structured inputs.
fuzz_target!(|document: GeneratedDocument| {
    let parsed: JsoncValue =
        parse_jsonc(&document.source).expect("structured generator produces valid JSONC");
    check_merge(&parsed);
});
