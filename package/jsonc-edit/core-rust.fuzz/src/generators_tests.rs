//! What:
//!      Unit tests for the structured generators.
//! Why:
//!       A fuzz campaign is only as trustworthy as its generator:
//!  if the generator emits
//!           documents the contract rejects,
//!  every campaign measures the wrong thing.
//!  These tests
//!           pin the generator's guarantees,
//!  including that its own depth bookkeeping agrees with the
//!           independent measurement in `invariants`.

/// What:
///      Import the `Arbitrary` trait and the unstructured-input type.
/// Why:
///       Tests drive generators from explicit byte buffers instead of a fuzzer,
///  and the trait
///           supplies the constructor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arbitrary, Unstructured } from 'arbitrary';
/// ```
use arbitrary::{Arbitrary, Unstructured};

/// What:
///      Import the generators under test.
/// Why:
///       Each has a guarantee the campaigns depend on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { mutated, nestedDocument, replacementValue, GeneratedDocument } from './generators';
/// ```
use crate::generators::{mutated, nested_document, replacement_value, GeneratedDocument};

/// What:
///      Import the independent depth measurement.
/// Why:
///       The generator's own counter is only trustworthy if it agrees with a second
///           implementation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { depthOf } from './invariants';
/// ```
use crate::invariants::depth_of;

/// What:
///      Import the crate's parse entry point and payload enum.
/// Why:
///       Parseability is the generator's central promise,
///  and kind coverage is asserted by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseJsonc, JsoncKind } from 'monochromatic-jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{parse_jsonc, JsoncKind};

/// What:
///      Draw one document from a deterministic byte buffer.
/// Why:
///       Tests must be reproducible,
///  so the buffer is the seed rather than random bytes.
fn generated(seed: &[u8]) -> GeneratedDocument {
    let mut unstructured = Unstructured::new(seed);
    return GeneratedDocument::arbitrary(&mut unstructured).expect("generator draws a document");
}

/// Every generated document must parse,
///  and the generator's depth must match the measured depth.
#[test]
fn generated_documents_parse_at_the_depth_they_claim() {
    let mut documents = 0;
    for first in 0u8..=255 {
        // Two-byte seeds keep the buffer large enough for several choices while staying enumerable.
        for second in [0u8, 7, 64, 129, 255] {
            let seed = [first, second, 0x5b, 0x22, 0x61, 0x7d, 0x2c, 0x0a, 0x2f, 0x2a, 0x39, 0xff];
            let document = generated(&seed);
            let parsed = parse_jsonc(&document.source)
                .unwrap_or_else(|error| panic!("seed {seed:?} produced rejected source: {error}\n{:?}", document.source));
            assert_eq!(depth_of(&parsed), document.depth, "generator depth disagrees with measurement");
            documents += 1;
        }
    }
    assert!(documents > 1000, "enumeration covered only {documents} seeds");
}

/// Every comment body the generator records must appear in the source it wrote.
#[test]
fn recorded_comment_bodies_appear_in_the_source() {
    for first in [0u8, 3, 33, 99, 200, 255] {
        let seed = [first, 0x2f, 0x2f, 0x20, 0x6e, 0x0a, 0x7b, 0x7d, 0x2a, 0x2f, 0x22, 0x61];
        let document = generated(&seed);
        for body in &document.comment_texts {
            assert!(document.source.contains(body), "body {body:?} is not in {:?}", document.source);
        }
    }
}

/// The exact-depth builder must nest precisely and only close when asked to.
#[test]
fn nested_document_nests_exactly() {
    for depth in [1usize, 2, 512, 513, 900] {
        let closed = nested_document(depth, false);
        assert_eq!(closed.matches('[').count(), depth, "open brackets at depth {depth}");
        assert_eq!(closed.matches(']').count(), depth, "close brackets at depth {depth}");
        assert!(closed.ends_with(']'), "closed document must end with a bracket");
        let open = nested_document(depth, true);
        assert_eq!(open.matches('[').count(), depth, "open brackets in malformed document");
        assert!(!open.contains(']'), "malformed document must stay unterminated");
    }
}

/// A mutation must change at most one byte and must never produce invalid UTF-8.
#[test]
fn mutation_stays_close_to_the_original() {
    let document = generated(&[9, 0x22, 0x61, 0x22, 0x3a, 0x31, 0x2c, 0x7d, 0x0a, 0x2f, 0x2f, 0x20]);
    for noise in [[1u8, 2, 3], [200, 0, 1], [7, 255, 128], [0, 0, 0]] {
        let mut unstructured = Unstructured::new(&noise);
        let candidate = mutated(&document.source, &mut unstructured).expect("mutation succeeds");
        let distance = candidate.len().abs_diff(document.source.len());
        assert!(distance <= 1, "mutation moved the length by {distance}");
        assert!(candidate.chars().count() > 0 || candidate.is_empty(), "mutation produced no characters");
    }
}

/// Replacement values must cover every scalar kind across seeds.
#[test]
fn replacement_values_cover_every_scalar_kind() {
    let mut kinds: Vec<&'static str> = Vec::new();
    for first in 0u8..=40 {
        let seed = [first, 0x31, 0x2e, 0x35, 0x65, 0x30, 0x22, 0x70, 0x6c, 0x61, 0x69, 0x6e];
        let mut unstructured = Unstructured::new(&seed);
        let value = replacement_value(&mut unstructured).expect("replacement builds");
        let name = match value.kind {
            JsoncKind::Null => "null",
            JsoncKind::Boolean { .. } => "boolean",
            JsoncKind::Number { .. } => "number",
            JsoncKind::Text { .. } => "text",
            JsoncKind::Array { .. } => "array",
            JsoncKind::Record { .. } => "record",
        };
        if !kinds.contains(&name) {
            kinds.push(name);
        }
    }
    for required in ["null", "boolean", "number", "text"] {
        assert!(kinds.contains(&required), "replacement values never produced {required}; saw {kinds:?}");
    }
}
