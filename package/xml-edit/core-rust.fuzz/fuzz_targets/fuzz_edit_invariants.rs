//! What:
//!  Edit target:
//! a random element plus a random edit,
//!  checked against the preservation,
//! read-back and consistency promises.
//! Why:
//!  The edit surface is the crate's reason to exist,
//!  and bytes outside the edited element changing would silently corrupt a user's settings file.

#![no_main]

/// What:
/// Import the harness macro.
/// Why:
/// Every target in this sidecar is a libFuzzer entry point.
use libfuzzer_sys::fuzz_target;
/// What:
/// Import the byte-budget reader.
/// Why:
/// The element choice,
/// edit kind and value come from fuzzer bytes.
use arbitrary::Unstructured;
/// What:
/// Import the parser.
/// Why:
/// The generated source is parsed before it is edited.
use monochromatic_xml_edit::parse_xml;
/// What:
/// Import the generated document and the edit invariants.
/// Why:
/// Each edit kind has its own promise.
use xml_edit_fuzz::{
    GeneratedDocument,
    invariants::{assert_append_child, assert_remove_element, assert_set_attribute},
};

fuzz_target!(|input: (GeneratedDocument, Vec<u8>)| {
    let (generated, noise) = input;
    let document = parse_xml(&generated.source).expect("a generated document is well-formed");
    let mut unstructured = Unstructured::new(&noise);
    let ids = document.element_ids();
    let Ok(choice) = unstructured.choose(&ids) else {
        return;
    };
    let id = *choice;
    let Ok(kind) = unstructured.int_in_range(0..=3_u8) else {
        return;
    };
    let Ok(value) = unstructured.arbitrary::<String>() else {
        return;
    };
    if kind == 0 {
        assert_set_attribute(&document, id, "k", &value);
    } else if kind == 1 {
        assert_set_attribute(&document, id, "fresh", &value);
    } else if kind == 2 {
        assert_append_child(&document, id);
    } else {
        assert_remove_element(&document, id);
    }
});
