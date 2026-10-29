//! What:
//!      Round-trip target:
//!  every generated document must parse,
//!  emit canonically and stay stable.
//! Why:
//!       This is the crate's core promise,
//!  so it is the first property fuzzing should attack:
//!           canonical output must be a fixed point,
//!  comments must survive it,
//!  and the parsed tree
//!           must respect the published nesting bound.

#![no_main]

/// What:
///      Import the harness macro.
/// Why:
///       Every target in this sidecar is a libFuzzer entry point.
use libfuzzer_sys::fuzz_target;
/// What:
///      Import the shared invariants and the generated document type.
/// Why:
///       This target asserts the round-trip contract and nothing else.
use jsonc_edit_fuzz::{assert_canonical_stability, assert_comments_preserved, assert_depth_bound, GeneratedDocument};
/// What:
///      Import the parse and emit entry points.
/// Why:
///       Round-trip stability is a statement about those two functions together.
use monochromatic_jsonc_edit::{emit_jsonc_value, parse_jsonc};

fuzz_target!(|document: GeneratedDocument| {
    // A generated document is valid JSONC by construction, so a rejection here is a parser bug.
    let parsed = parse_jsonc(&document.source)
        .unwrap_or_else(|error| panic!("generated document rejected: {error}\nsource: {:?}", document.source));

    assert_depth_bound(&parsed);
    assert_canonical_stability(&document.source);

    let emitted = emit_jsonc_value(&parsed);
    assert_comments_preserved(&parsed, &emitted);

    // Every line of every body the generator wrote must still be present, which catches comments
    // dropped during merging as well as during emission. Whole bodies are not substrings of the
    // output by design: canonical emission renders a merged multi-line body as one indented `//`
    // line per body line.
    for body in &document.comment_texts {
        for line in body.replace("\r\n", "\n").split(['\n', '\r']) {
            assert!(emitted.contains(line), "generated comment body {body:?} lost {line:?} in emission:\n{emitted}");
        }
    }

    // Emission must itself be a document the parser accepts, with the same depth.
    let reparsed = parse_jsonc(&emitted).unwrap_or_else(|error| panic!("emission rejected: {error}\n{emitted}"));
    assert_depth_bound(&reparsed);
});
