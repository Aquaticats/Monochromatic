//! What:
//!  Regression tests for the linter's ordered JSONC settings merge.
//! Why:
//!  Fixed examples and the independent public corpus distinguish all-input merging from a pairwise fold.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('configuration merge', () => { /* semantic and ownership controls */ });
//! ```

/// What:
///  Import the production merge and the actual JSONC parser.
/// Why:
///  Fixtures cross the same value representation as runtime configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { mergeValues } from './config-merge';
/// import { parseJsonc, emitJsoncValue } from 'jsonc-edit';
/// ```
use super::merge_values;
use monochromatic_jsonc_edit::{JsoncKind, JsoncValue, emit_jsonc_value, parse_jsonc};

/// What:
///  Read a required corpus field as an independent owned value.
/// Why:
///  Missing fixture fields must fail the test rather than silently remove a case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function field(value: JsoncValue, name: string): JsoncValue;
/// ```
fn field(value: &JsoncValue, name: &str) -> JsoncValue {
    // Decode the fixture field name to the parser's UTF-16 key representation.
    let units: Vec<u16> = name.encode_utf16().collect();
    // Borrow the members; every corpus case is required to be a record.
    for entry in value.entries().expect("corpus cases are records") {
        if entry.key.units == units {
            // Clone the selected field so the helper need not expose a borrowed lifetime.
            return entry.value.clone();
        }
    }
    // An invalid checked-in fixture is a test failure, not a merge outcome.
    panic!("missing corpus field {name}");
}

/// What:
///  Merge an input-array document through the production implementation.
/// Why:
///  Every test exercises the real parser and keeps scalar inputs possible inside the outer array.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeText(source: string): JsoncValue { return mergeValues(parseJsonc(source)); }
/// ```
fn merge_text(source: &str) -> JsoncValue {
    // Parse a known-valid test document, making syntax errors visible as test failures.
    let document = parse_jsonc(source).expect("merge fixture parses");
    // Borrow, rather than move, each element so the production signature matches runtime use.
    let inputs: Vec<&JsoncValue> = document
        .elements()
        .expect("merge fixture is an array")
        .iter()
        .collect();
    // Lend the reference vector while the parsed fixture remains alive.
    return merge_values(&inputs);
}

/// What:
///  Check the independent public JSON corpus from the existing merge sidecar.
/// Why:
///  The Rust implementation must not define its own expected outcomes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const test of jsonCases) expect(mergeValues(test.inputs)).toEqual(test.expected);
/// ```
#[test]
fn independent_json_corpus_matches() {
    // Embed a generated copy of public, non-embargoed cases; no package manager runs in this test.
    let source = include_str!("../fixtures/merge-cases.json");
    let corpus = parse_jsonc(source).expect("public corpus parses");
    for case in corpus.elements().expect("public corpus is an array") {
        // Read both sides before borrowing the input elements.
        let input_document = field(case, "inputs");
        let expected = field(case, "expected");
        let inputs: Vec<&JsoncValue> = input_document
            .elements()
            .expect("inputs are an array")
            .iter()
            .collect();
        let actual = merge_values(&inputs);
        // Rust's assertion macro borrows both trees and displays their structural difference.
        assert_eq!(
            actual,
            expected,
            "{}",
            emit_jsonc_value(&field(case, "name"))
        );
    }
}

/// Empty configuration matching produces an empty settings object.
#[test]
fn empty_inputs_produce_empty_record() {
    let actual = merge_text("[]");
    let expected = parse_jsonc("{}").expect("expected record parses");
    assert_eq!(actual, expected);
}

/// A type mismatch at any point selects the last value instead of folding later containers.
#[test]
fn all_input_mismatch_is_not_pairwise_folding() {
    let input = parse_jsonc("[{\"k\":[1]}, {\"k\":false}, {\"k\":[2]}, {\"k\":[3]}]")
        .expect("fixture parses");
    let values: Vec<&JsoncValue> = input.elements().expect("array").iter().collect();
    let actual = merge_values(&values);
    let expected = parse_jsonc("{\"k\":[3]}").expect("expected record parses");
    assert_eq!(actual, expected);
    // Positive control: the same case distinguishes the explicitly wrong pairwise strategy.
    let mut pairwise = values[0].clone();
    for value in &values[1..] {
        pairwise = merge_values(&[&pairwise, value]);
    }
    assert_ne!(actual, pairwise);
}

/// Decoded key identity joins different escape spellings without changing first-seen key order.
#[test]
fn decoded_keys_and_first_key_spelling_survive() {
    let actual = merge_text(r#"[{"\u0061":{"first":true},"z":1},{"a":{"second":true},"b":2}]"#);
    let expected = parse_jsonc(r#"{"\u0061":{"first":true,"second":true},"z":1,"b":2}"#)
        .expect("expected record parses");
    assert_eq!(actual, expected);
}

/// Null,
///  booleans,
///  strings and numbers use the final value without treating null as absence.
#[test]
fn scalar_values_and_mixed_kinds_choose_last() {
    for source in [
        "[null,null]",
        "[true,false]",
        "[1,2]",
        "[\"a\",\"b\"]",
        "[{},[],false]",
    ] {
        let document = parse_jsonc(source).expect("fixture parses");
        let elements = document.elements().expect("array");
        let actual = merge_text(source);
        assert_eq!(actual, elements.last().expect("nonempty fixture").clone());
    }
}

/// Merge results own their values and preserve the final container comment.
#[test]
fn input_trees_are_unchanged_and_output_is_owned() {
    // JSONC comments following a comma on the same line attach to the preceding value.
    // Put each leading comment on its own line so this fixture actually annotates both inputs.
    let document = parse_jsonc("[\n/* first */ {\"a\":[1]},\n/* last */ {\"a\":[2]}\n]")
        .expect("fixture parses");
    let snapshot = document.clone();
    let elements = document.elements().expect("array");
    assert!(
        elements[1].comment.is_some(),
        "ownership control needs an actual final comment"
    );
    let inputs: Vec<&JsoncValue> = elements.iter().collect();
    let mut actual = merge_values(&inputs);
    assert_eq!(actual.comment, elements[1].comment);
    actual.comment = None;
    // What: Borrow the output record and then its array mutably, without borrowing any input.
    // Why: This proves independence of nested payloads as well as the outer comment.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // actual.a.push(true);
    // ```
    if let JsoncKind::Record { entries } = &mut actual.kind {
        if let JsoncKind::Array {
            elements: merged_elements,
        } = &mut entries[0].value.kind
        {
            merged_elements.push(JsoncValue::boolean(true));
        } else {
            panic!("ownership fixture must produce an array member");
        }
    } else {
        panic!("ownership fixture must produce a record");
    }
    assert_eq!(document, snapshot);
    assert_ne!(actual.comment, elements[1].comment);
}

/// The parser's maximum supported record depth also merges without losing its leaf members.
#[test]
fn maximum_parser_depth_merges() {
    // A leaf record plus 511 enclosing records reaches the parser's 512-container limit.
    let prefix = "{\"x\":".repeat(511);
    let suffix = "}".repeat(511);
    let left = parse_jsonc(&format!("{prefix}{{\"a\":1}}{suffix}"))
        .expect("left boundary document parses");
    let right = parse_jsonc(&format!("{prefix}{{\"b\":2}}{suffix}"))
        .expect("right boundary document parses");
    let expected = parse_jsonc(&format!("{prefix}{{\"a\":1,\"b\":2}}{suffix}"))
        .expect("expected boundary document parses");
    let actual = merge_values(&[&left, &right]);
    assert_eq!(actual, expected);
}
