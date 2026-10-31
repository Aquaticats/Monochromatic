//! What:
//!  Rejection target:
//! one deliberate mutation away from a valid document must never panic.
//! Why:
//!  Real callers hand this crate truncated files,
//!  stray quotes and broken references.
//!  The contract is a returned positioned error,
//!  not an abort,
//!  an overflow or an infinite loop,
//!  and mutated valid documents reach those paths far more often than random bytes do.

#![no_main]

/// What:
/// Import the harness macro.
/// Why:
/// Every target in this sidecar is a libFuzzer entry point.
use libfuzzer_sys::fuzz_target;
/// What:
/// Import the byte-budget reader.
/// Why:
/// Both the mutation and the input pair come from fuzzer bytes.
use arbitrary::Unstructured;
/// What:
/// Import the generated document,
/// the mutation helper and the outcome check.
/// Why:
/// Near-valid input reaches
/// rejection paths that unrelated bytes almost never do.
use xml_edit_fuzz::{GeneratedDocument, generators::mutated, invariants::assert_parse_or_reject};

fuzz_target!(|input: (GeneratedDocument, Vec<u8>)| {
    let (document, noise) = input;
    let mut unstructured = Unstructured::new(&noise);
    let Ok(candidate) = mutated(&document.source, &mut unstructured) else {
        // The fuzzer ran out of bytes for a mutation; there is nothing to assert about this input.
        return;
    };
    assert_parse_or_reject(&candidate);
});
