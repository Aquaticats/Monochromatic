//! What:
//!      Rejection target:
//!  one deliberate mutation away from valid input must never panic.
//! Why:
//!       Real callers hand this crate truncated files,
//!  stray commas and broken escapes.
//!  The
//!           contract is a returned `JsoncParseError`,
//!  not an abort,
//!  an overflow or an infinite loop,
//!           and mutated valid documents reach those paths far more often than random bytes do.

#![no_main]

/// What:
///      Import the harness macro.
/// Why:
///       Every target in this sidecar is a libFuzzer entry point.
use libfuzzer_sys::fuzz_target;
/// What:
///      Import the byte-budget reader.
/// Why:
///       Both the mutation and the input pair come from fuzzer bytes.
use arbitrary::Unstructured;
/// What:
///      Import the single-mutation generator.
/// Why:
///       Near-valid input reaches rejection paths that unrelated bytes almost never do.
use jsonc_edit_fuzz::generators::mutated;
/// What:
///      Import the shared invariants and the generated document type.
/// Why:
///       A mutation that still parses must satisfy every property an untouched document does.
use jsonc_edit_fuzz::{assert_canonical_stability, assert_depth_bound, GeneratedDocument};
/// What:
///      Import the parser entry point.
/// Why:
///       Acceptance versus rejection is the property under test.
use monochromatic_jsonc_edit::parse_jsonc;

fuzz_target!(|input: (GeneratedDocument, Vec<u8>)| {
    let (document, noise) = input;
    let mut unstructured = Unstructured::new(&noise);
    let Ok(candidate) = mutated(&document.source, &mut unstructured) else {
        // The fuzzer ran out of bytes for a mutation; there is nothing to assert about this input.
        return;
    };

    match parse_jsonc(&candidate) {
        Ok(parsed) => {
            // A mutation that still parses must satisfy every property an untouched document does.
            assert_depth_bound(&parsed);
            assert_canonical_stability(&candidate);
        }
        Err(error) => {
            // Rejection must carry a message; an empty one would be undiagnosable at the caller.
            assert!(!error.message.is_empty(), "rejection of {candidate:?} had no message");
        }
    }
});
