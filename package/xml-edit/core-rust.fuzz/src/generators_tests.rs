//! What:
//!  Tests for the generator and the mutation helper.
//! Why:
//!  The fuzz targets assume every generated document is well-formed,
//!  so that assumption is checked here over many deterministic byte patterns instead of trusted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('generators', () => { /* validity sweep, mutation */ });
//! ```

use arbitrary::{Arbitrary, Unstructured};
use monochromatic_xml_edit::parse_xml;

use crate::generators::mutated;
use crate::GeneratedDocument;

/// Build a deterministic byte pattern of the given length from a seed.
fn pattern(seed: u32, length: usize) -> Vec<u8> {
    let mut state: u32 = seed.wrapping_mul(2_654_435_761).wrapping_add(12_345);
    let mut bytes: Vec<u8> = Vec::with_capacity(length);
    for _ in 0..length {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        bytes.push((state >> 24) as u8);
    }
    return bytes;
}

/// Check every generated document is accepted by the parser.
#[test]
fn generated_documents_are_well_formed() {
    let mut generated: usize = 0;
    for seed in 0..2_000_u32 {
        let bytes: Vec<u8> = pattern(seed, 200);
        let mut unstructured: Unstructured<'_> = Unstructured::new(&bytes);
        if let Ok(document) = GeneratedDocument::arbitrary(&mut unstructured) {
            generated += 1;
            assert!(parse_xml(&document.source).is_ok(), "seed {seed} produced {:?}", document.source);
        }
    }
    assert!(generated > 1_500, "only {generated} documents were generated");
}

/// Check the sweep reaches the constructs the grammar cares about.
#[test]
fn the_sweep_reaches_every_construct() {
    let mut combined: String = String::new();
    for seed in 0..2_000_u32 {
        let bytes: Vec<u8> = pattern(seed, 200);
        let mut unstructured: Unstructured<'_> = Unstructured::new(&bytes);
        if let Ok(document) = GeneratedDocument::arbitrary(&mut unstructured) {
            combined.push_str(&document.source);
        }
    }
    for needle in ["\u{FEFF}", "<?xml", "<![CDATA[", "<!--", "&#10;", "\r\n", "\t", "'single'", " />", "ns:item"] {
        assert!(combined.contains(needle), "generator never produced {needle:?}");
    }
}

/// Check a mutation changes a document and keeps the result valid UTF-8 on a character boundary.
#[test]
fn mutations_change_the_source() {
    let source: &str = "<a k=\"\u{e9}\"><b/></a>";
    let mut changed: usize = 0;
    for seed in 0..500_u32 {
        let bytes: Vec<u8> = pattern(seed, 64);
        let mut unstructured: Unstructured<'_> = Unstructured::new(&bytes);
        if let Ok(candidate) = mutated(source, &mut unstructured)
            && candidate != source
        {
            changed += 1;
        }
    }
    assert!(changed > 300, "only {changed} mutations changed the source");
}

/// Check an empty byte budget falls back to the first choices instead of panicking,
/// even on an empty source.
#[test]
fn an_empty_budget_falls_back_to_defaults() {
    let mut unstructured: Unstructured<'_> = Unstructured::new(&[]);
    assert_eq!(mutated("<a/>", &mut unstructured).expect("defaults"), "a/>");
    let mut again: Unstructured<'_> = Unstructured::new(&[]);
    assert_eq!(mutated("", &mut again).expect("defaults"), "");
}
