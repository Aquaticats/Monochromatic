//! What:
//!  Round-trip target:
//! every generated document must parse and be self-consistent.
//! Why:
//!  The generator only builds well-formed XML,
//!  so a rejection here is a parser bug,
//!  and the span bookkeeping that edits rely on is checked for every element.

#![no_main]

/// What:
/// Import the harness macro.
/// Why:
/// Every target in this sidecar is a libFuzzer entry point.
use libfuzzer_sys::fuzz_target;
/// What:
/// Import the parser and the shared invariants.
/// Why:
/// The property under test is parse then consistency.
use monochromatic_xml_edit::parse_xml;
/// What:
/// Import the generated document type and the consistency check.
/// Why:
/// One definition of "consistent".
use xml_edit_fuzz::{GeneratedDocument, invariants::assert_self_consistent};

fuzz_target!(|input: GeneratedDocument| {
    let document = parse_xml(&input.source).expect("a generated document is well-formed");
    assert_self_consistent(&document);
    assert_eq!(document.source(), input.source, "parsing must keep the source byte-for-byte");
});
