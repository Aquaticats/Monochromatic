//! What:
//!      The shared supported-behavior fixtures,
//!  read from the same JSON file the TypeScript
//!           conformance suite uses.
//! Why:
//!       One corpus keeps both maintained implementations honest:
//!  a case added on either side is a
//!           case both must satisfy,
//!  and the file is compared byte-for-byte by a task in each package.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import fixtures from '../fixtures/jsonc-conformance.json' with { type: 'json' };
//! ```

/// What:
///      Embed the shared fixture document at compile time.
/// Why:
///       A published crate must carry its own copy,
///  and embedding removes any runtime file path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FIXTURES = require('../fixtures/jsonc-conformance.json');
/// ```
const FIXTURE: &str = include_str!("../fixtures/jsonc-conformance.json");

/// What:
///      Import the number identity so fixture equality cases can be checked directly.
/// Why:
///       Mathematical equality is a fixture contract,
///  not an implementation detail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncNumberIdentity } from './number';
/// ```
use crate::number::JsoncNumberIdentity;
/// What:
///      Import the address segments used to walk fixture expectations.
/// Why:
///       A fixture path entry is either an object key or an array index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncPathSegment } from './path';
/// ```
use crate::path::JsoncPathSegment;
/// What:
///      Import the quoted-text conversion.
/// Why:
///       Fixture text is stored as decoded code units and compared as Rust strings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { unitsToString } from './textUnits';
/// ```
use crate::text_units::units_to_string;
/// What:
///      Import the document model plus the public parse,
///  emit and comment surface.
/// Why:
///       The fixture tests exercise the same API a consumer uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseJsonc, emitJsoncValue, jsoncComment, jsoncKeyComment } from './index';
/// ```
use crate::value::{JsoncEntry, JsoncKind, JsoncValue};
use crate::{emit_jsonc_value, jsonc_comment, jsonc_key_comment, jsonc_set, parse_jsonc};

/// What:
///      Parse the embedded fixture document once per test.
/// Why:
///       The fixture is ordinary JSON,
///  so this crate's own parser reads it without a second JSON
///           dependency.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fixtures(): JsoncValue { return parseJsonc(FIXTURES); }
/// ```
fn fixture() -> JsoncValue {
    return parse_jsonc(FIXTURE).expect("shared fixture document parses");
}

/// What:
///      Read one member of a record by key,
///  taking the last match.
/// Why:
///       Fixture sections are addressed by name,
///  and duplicate names would be a fixture bug.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function member(value: JsoncValue, key: string): JsoncValue;
/// ```
fn member<'a>(value: &'a JsoncValue, key: &str) -> &'a JsoncValue {
    let entries = value.entries().expect("fixture section is a record");
    let wanted: Vec<u16> = key.encode_utf16().collect();
    for entry in entries.iter().rev() {
        if entry.key.units == wanted {
            return &entry.value;
        }
    }
    panic!("fixture has no member {key}");
}

/// What:
///      Read a record member's text.
/// Why:
///       Fixture names and sources are stored as JSON strings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function text(value: JsoncValue, key: string): string;
/// ```
fn text(value: &JsoncValue, key: &str) -> String {
    let target = member(value, key);
    let units = target.text_units().expect("fixture field is a string");
    return units_to_string(units).expect("fixture text is valid UTF-16");
}

/// What:
///      Read a record member that holds text or null.
/// Why:
///       Fixture comment expectations use null to mean "no comment here".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function optionalText(value: JsoncValue, key: string): string | undefined;
/// ```
fn optional_text(value: &JsoncValue, key: &str) -> Option<String> {
    let target = member(value, key);
    if matches!(target.kind, JsoncKind::Null) {
        return None;
    }
    let units = target.text_units().expect("fixture field is a string or null");
    return Some(units_to_string(units).expect("fixture text is valid UTF-16"));
}

/// What:
///      Read one array element by position.
/// Why:
///       Fixture cases are lists,
///  and a missing position is a fixture bug worth naming.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function element(value: JsoncValue, index: number): JsoncValue;
/// ```
fn element(value: &JsoncValue, index: usize) -> &JsoncValue {
    let elements = value.elements().expect("fixture section is an array");
    return elements.get(index).unwrap_or_else(|| panic!("fixture array has no element {index}"));
}

/// What:
///      Convert one fixture path entry list into address segments.
/// Why:
///       Fixture paths mix object keys and array positions,
///  exactly like a caller's address.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function toPath(entries: JsoncValue[]): JsoncPathSegment[];
/// ```
fn to_path(entries: &[JsoncValue]) -> Vec<JsoncPathSegment> {
    let mut path: Vec<JsoncPathSegment> = Vec::new();
    for entry in entries {
        if let Some(units) = entry.text_units() {
            let key = units_to_string(units).expect("fixture path key is valid UTF-16");
            path.push(JsoncPathSegment::Key { key });
            continue;
        }
        let JsoncKind::Number { raw, .. } = &entry.kind else {
            panic!("fixture path entry is neither a key nor an index");
        };
        let index: usize = raw.parse().expect("fixture path index is a non-negative integer");
        path.push(JsoncPathSegment::Index { index });
    }
    return path;
}

/// Check every fixture source that must parse,
///  including its root shape and member count.
#[test]
fn valid_sources_parse_with_the_expected_shape() {
    let document = fixture();
    let cases = member(&document, "valid");
    for index in 0..cases.elements().expect("valid cases").len() {
        let case = element(cases, index);
        let name = text(case, "name");
        let source = text(case, "source");
        let parsed = parse_jsonc(&source).unwrap_or_else(|error| panic!("{name} must parse: {error}"));
        let expected_kind = text(case, "rootKind");
        let entries = parsed.entries();
        let elements = parsed.elements();
        let actual_kind = if entries.is_some() { "record" } else { "array" };
        assert_eq!(actual_kind, expected_kind, "{name} root kind");
        let members = member(case, "members");
        let JsoncKind::Number { raw, .. } = &members.kind else { panic!("{name} member count"); };
        let expected: usize = raw.parse().expect("fixture member count");
        let actual = entries.map(|list| return list.len()).unwrap_or_else(|| return elements.expect("array payload").len());
        assert_eq!(actual, expected, "{name} member count");
    }
}

/// Check every fixture source that must be rejected.
#[test]
fn invalid_sources_are_rejected() {
    let document = fixture();
    let cases = member(&document, "invalid");
    for index in 0..cases.elements().expect("invalid cases").len() {
        let case = element(cases, index);
        let name = text(case, "name");
        let source = text(case, "source");
        assert!(parse_jsonc(&source).is_err(), "{name} must be rejected: {source:?}");
    }
}

/// Check comment ownership for every fixture case,
///  on the root and at each address.
#[test]
fn comment_ownership_matches_the_fixture() {
    let document = fixture();
    let cases = member(&document, "commentOwnership");
    for index in 0..cases.elements().expect("ownership cases").len() {
        let case = element(cases, index);
        let name = text(case, "name");
        let source = text(case, "source");
        let parsed = parse_jsonc(&source).unwrap_or_else(|error| panic!("{name} must parse: {error}"));
        let root_comment = parsed.comment.as_ref().map(|comment| return comment.text.clone());
        assert_eq!(root_comment, optional_text(case, "root"), "{name} root comment");
        let expectations = member(case, "at");
        for expectation_index in 0..expectations.elements().expect("expectations").len() {
            let expectation = element(expectations, expectation_index);
            let path_entries = member(expectation, "path");
            let path = to_path(path_entries.elements().expect("path entries"));
            let value_comment = jsonc_comment(&parsed, &path)
                .unwrap_or_else(|error| panic!("{name} value comment query: {error}"))
                .map(|comment| return comment.text.clone());
            assert_eq!(value_comment, optional_text(expectation, "value"), "{name} value comment");
            let key_comment = jsonc_key_comment(&parsed, &path)
                .map(|comment| return comment.map(|found| return found.text.clone()))
                .unwrap_or(None);
            assert_eq!(key_comment, optional_text(expectation, "key"), "{name} key comment");
        }
    }
}

/// Check that unedited number spellings survive canonical emission.
#[test]
fn number_spellings_survive_emission() {
    let document = fixture();
    let cases = member(&document, "numberSpelling");
    for index in 0..cases.elements().expect("spelling cases").len() {
        let case = element(cases, index);
        let name = text(case, "name");
        let parsed = parse_jsonc(&text(case, "source")).expect("spelling fixture parses");
        let emitted = emit_jsonc_value(&parsed);
        let expected = member(case, "emittedContains");
        for expectation_index in 0..expected.elements().expect("expectations").len() {
            let needle = units_to_string(
                element(expected, expectation_index).text_units().expect("fixture expectation text"),
            )
            .expect("fixture expectation is valid UTF-16");
            assert!(emitted.contains(&needle), "{name} lost {needle} in {emitted:?}");
        }
    }
}

/// Check mathematical equality and inequality for every fixture pair.
#[test]
fn number_equality_matches_the_fixture() {
    let document = fixture();
    let cases = member(&document, "numberEquality");
    for index in 0..cases.elements().expect("equality cases").len() {
        let case = element(cases, index);
        let name = text(case, "name");
        let left = JsoncNumberIdentity::from_token(&text(case, "left")).expect("left token is valid");
        let right = JsoncNumberIdentity::from_token(&text(case, "right")).expect("right token is valid");
        let expected = matches!(member(case, "equal").kind, JsoncKind::Boolean { value: true });
        assert_eq!(left == right, expected, "{name} equality");
    }
}

/// Check the fixture's nesting boundary on both the accepted and the rejected depth.
#[test]
fn nesting_boundary_matches_the_fixture() {
    let document = fixture();
    let depth = member(&document, "depth");
    let JsoncKind::Number { raw: accepted_raw, .. } = &member(depth, "accepted").kind else {
        panic!("accepted depth must be a number");
    };
    let JsoncKind::Number { raw: rejected_raw, .. } = &member(depth, "rejected").kind else {
        panic!("rejected depth must be a number");
    };
    let accepted: usize = accepted_raw.parse().expect("accepted depth");
    let rejected: usize = rejected_raw.parse().expect("rejected depth");
    assert_eq!(rejected, accepted + 1, "the fixture boundary must be one container apart");
    let accepted_source = format!("{}0{}", "[".repeat(accepted), "]".repeat(accepted));
    assert!(parse_jsonc(&accepted_source).is_ok(), "depth {accepted} must parse");
    let rejected_source = format!("{}0{}", "[".repeat(rejected), "]".repeat(rejected));
    let error = parse_jsonc(&rejected_source).expect_err("one container past the limit must fail");
    assert_eq!(error.message, "JSONC nesting too deep");
}

/// Check that the fixture's own document shape stays usable by both implementations.
#[test]
fn fixture_document_has_every_section() {
    let document = fixture();
    let entries: &[JsoncEntry] = document.entries().expect("fixture root is a record");
    let names: Vec<String> = entries
        .iter()
        .map(|entry| return units_to_string(&entry.key.units).expect("ascii section name"))
        .collect();
    for required in ["valid", "invalid", "commentOwnership", "numberSpelling", "numberEquality", "depth"] {
        assert!(names.iter().any(|name| return name == required), "fixture is missing {required}");
    }
}

/// What:
///      Collect every comment body in a tree,
///  from the node itself,
///  its keys and its values.
/// Why:
///       The round-trip contract is stated as bodies that must still be attached afterwards,
///           which needs one walk rather than a query per address.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commentBodies(value: JsoncValue): string[];
/// ```
fn comment_bodies(value: &JsoncValue) -> Vec<String> {
    let mut bodies: Vec<String> = Vec::new();
    let mut stack: Vec<&JsoncValue> = vec![value];
    while let Some(current) = stack.pop() {
        if let Some(comment) = &current.comment {
            bodies.push(comment.text.clone());
        }
        if let JsoncKind::Record { entries } = &current.kind {
            for entry in entries.iter().rev() {
                if let Some(comment) = &entry.key.comment {
                    bodies.push(comment.text.clone());
                }
                stack.push(&entry.value);
            }
        }
        if let JsoncKind::Array { elements } = &current.kind {
            for element in elements.iter().rev() {
                stack.push(element);
            }
        }
    }
    return bodies;
}

/// What:
///      Build one replacement value for a fixture `valueKind` name.
/// Why:
///       The root-shape contract is about the replacement's shape,
///  so the fixture names a kind
///           rather than spelling a literal both suites would have to agree on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function valueForKind(kind: string): JsonValue;
/// ```
fn value_for_kind(kind: &str) -> JsoncValue {
    let payload = if kind == "null" {
        JsoncKind::Null
    } else if kind == "number" {
        JsoncKind::Number {
            raw: "7".to_string(),
            identity: JsoncNumberIdentity::from_token("7").expect("seven is a valid token"),
        }
    } else if kind == "string" {
        JsoncKind::Text { units: Vec::new(), raw: "\"\"".to_string() }
    } else if kind == "array" {
        JsoncKind::Array { elements: Vec::new() }
    } else if kind == "record" {
        JsoncKind::Record { entries: Vec::new() }
    } else {
        panic!("fixture names an unknown value kind {kind}");
    };
    return JsoncValue { kind: payload, comment: None };
}

/// Emission must reparse,
///  and every comment body the fixture names must still be attached.
#[test]
fn round_trip_cases_keep_their_comment_bodies() {
    let document = fixture();
    let cases = member(&document, "roundTrip");
    for index in 0..cases.elements().expect("roundTrip cases").len() {
        let case = element(cases, index);
        let name = text(case, "name");
        let source = text(case, "source");
        let parsed = parse_jsonc(&source).unwrap_or_else(|error| panic!("{name} must parse: {error}"));
        let emitted = emit_jsonc_value(&parsed);
        let reparsed = parse_jsonc(&emitted).unwrap_or_else(|error| panic!("{name} emission must reparse: {error}"));
        let bodies = comment_bodies(&reparsed);
        let expected = member(case, "commentBodies");
        for expectation in 0..expected.elements().expect("expected bodies").len() {
            let wanted = units_to_string(element(expected, expectation).text_units().expect("body text"))
                .expect("fixture body is valid UTF-16");
            assert!(
                bodies.iter().any(|body| return *body == wanted),
                "{name} lost comment body {wanted:?}; found {bodies:?}"
            );
        }
    }
}

/// A root replacement that is not a container must be refused,
///  and every other case accepted.
#[test]
fn root_shape_cases_match_the_fixture() {
    let document = fixture();
    let cases = member(&document, "rootShape");
    let base = parse_jsonc("{\"a\":1}").expect("base document parses");
    for index in 0..cases.elements().expect("rootShape cases").len() {
        let case = element(cases, index);
        let name = text(case, "name");
        let path = to_path(member(case, "path").elements().expect("path entries"));
        let replacement = value_for_kind(&text(case, "valueKind"));
        let refused = matches!(member(case, "refused").kind, JsoncKind::Boolean { value: true });
        match jsonc_set(&base, &path, replacement) {
            Ok(_) => assert!(!refused, "{name} should have been refused"),
            Err(error) => assert!(refused, "{name} should have been accepted: {error}"),
        }
    }
}

/// Canonical emission must equal the recorded text exactly,
///  byte for byte.
#[test]
fn canonical_layout_matches_the_fixture() {
    let document = fixture();
    let cases = member(&document, "canonicalLayout");
    for index in 0..cases.elements().expect("layout cases").len() {
        let case = element(cases, index);
        let name = text(case, "name");
        let source = text(case, "source");
        let parsed = parse_jsonc(&source).unwrap_or_else(|error| panic!("{name} must parse: {error}"));
        let expected = text(case, "emitted");
        assert_eq!(emit_jsonc_value(&parsed), expected, "{name} canonical layout differs");
    }
}
