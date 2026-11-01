//! What:
//!      Tests for the immutable read and structural edit surface.
//! Why:
//!       An edit must return a new document,
//!  leave the input usable,
//!  create only a missing final
//!           segment,
//!  and report a missing address separately from a wrong-shaped one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('jsonc edit api', () => { /* lookup, keys, set, delete, immutability */ });
//! ```

/// What:
///      Import the failure enum so tests can name the expected variant.
/// Why:
///       A missing address and a wrong shape are different outcomes and must not be conflated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncEditError } from './error';
/// ```
use crate::error::JsoncEditError;
/// What:
///      Import the canonical emitter for edited values.
/// Why:
///       An edit returns a value,
///  and its text is what a consumer would write out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { emitJsoncValue } from './emit';
/// ```
use crate::emit::emit_jsonc_value;
/// What:
///      Import the address segment type and the key-path helper.
/// Why:
///       Addresses are built from key and index segments rather than from a string syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { jsoncKeyPath, type JsoncPathSegment } from './path';
/// ```
use crate::path::{jsonc_key_path, JsoncPathSegment};
/// What:
///      Import the quoted-text conversion used to read key names.
/// Why:
///       Keys are stored as code units,
///  so a test comparing names converts them explicitly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { unitsToString } from './textUnits';
/// ```
use crate::text_units::units_to_string;
/// What:
///      Import the value model and the state lifecycle functions under test.
/// Why:
///       These tests use the same public surface a consumer uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { jsoncLookup, jsoncSet, jsoncDelete, parseJsoncEdit } from './index';
/// ```
use crate::value::{JsoncKind, JsoncValue};
use crate::{
    jsonc_delete, jsonc_has, jsonc_keys, jsonc_lookup, jsonc_set, jsonc_state_from_value, jsonc_stringify,
    parse_jsonc_edit,
};

/// What:
///      Build one index segment.
/// Why:
///       Array addresses are positions,
///  and the helper keeps test lines readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const at = (index: number) => ({ kind: 'index', index });
/// ```
fn at(index: usize) -> JsoncPathSegment {
    return JsoncPathSegment::Index { index };
}

/// Check that parsing into a state and serializing it round-trips through the public surface.
#[test]
fn state_parses_and_serializes() {
    let source = "{\n  // note\n  \"a\": 1,\n}";
    let state = parse_jsonc_edit(source).expect("valid JSONC parses into a state");
    let text = jsonc_stringify(&state);
    let reparsed = parse_jsonc_edit(&text).expect("canonical output reparses");
    assert_eq!(reparsed, state);
    // A state built from an already-parsed value serializes identically.
    let wrapped = jsonc_state_from_value(state.root.clone());
    assert_eq!(jsonc_stringify(&wrapped), text);
}

/// Check reads at the root,
///  through keys,
///  through indexes,
///  and against scalar targets.
#[test]
fn lookup_walks_addresses() {
    let state = parse_jsonc_edit("{\"a\":{\"b\":[10,20]},\"c\":true}").expect("valid document");
    // An empty address names the root.
    assert!(jsonc_lookup(&state.root, &[]).is_ok());
    let nested = jsonc_lookup(&state.root, &jsonc_key_path(["a", "b"])).expect("nested array");
    assert_eq!(nested.elements().expect("array payload").len(), 2);
    // A mixed address reaches one element and keeps its token spelling.
    let mut mixed = jsonc_key_path(["a", "b"]);
    mixed.push(at(1));
    let twenty = jsonc_lookup(&state.root, &mixed).expect("array element");
    assert_eq!(twenty.number_token(), Some("20"));
    assert!(jsonc_has(&state.root, &jsonc_key_path(["c"])));
    assert!(!jsonc_has(&state.root, &jsonc_key_path(["missing"])));
}

/// Check that a missing address and a wrong-shaped target are reported differently.
#[test]
fn lookup_reports_missing_and_mismatched_addresses() {
    let state = parse_jsonc_edit("{\"a\":[1],\"s\":\"text\"}").expect("valid document");
    let missing = jsonc_lookup(&state.root, &jsonc_key_path(["nope"])).expect_err("missing key");
    assert!(matches!(missing, JsoncEditError::PathNotFound { .. }), "expected missing address: {missing:?}");
    // An index cannot address an object member.
    let mismatched = jsonc_lookup(&state.root, &[at(0)]).expect_err("index against record");
    assert!(matches!(mismatched, JsoncEditError::Type { .. }), "expected shape failure: {mismatched:?}");
    // A key cannot address an array element.
    let mismatched_key = jsonc_lookup(&state.root, &jsonc_key_path(["a", "b"])).expect_err("key against array");
    assert!(matches!(mismatched_key, JsoncEditError::Type { .. }), "expected shape failure");
    // An out-of-range index is a missing address.
    let mut out_of_range = jsonc_key_path(["a"]);
    out_of_range.push(at(5));
    assert!(matches!(
        jsonc_lookup(&state.root, &out_of_range).expect_err("range"),
        JsoncEditError::PathNotFound { .. }
    ));
    // A scalar cannot be indexed at all.
    let scalar = jsonc_lookup(&state.root, &jsonc_key_path(["s", "x"])).expect_err("index into scalar");
    assert!(matches!(scalar, JsoncEditError::Type { .. }), "expected shape failure");
}

/// Check that keys come back in source order,
///  duplicates included,
///  and only for records.
#[test]
fn keys_list_members_in_order() {
    let state = parse_jsonc_edit("{\"b\":1,\"a\":2,\"b\":3}").expect("valid document with a duplicate key");
    let keys = jsonc_keys(&state.root, &[]).expect("root keys");
    let names: Vec<String> = keys
        .iter()
        .map(|key| return units_to_string(&key.units).expect("ascii key text"))
        .collect();
    assert_eq!(names, ["b", "a", "b"]);
    // A duplicate key resolves to its last member.
    let duplicate = jsonc_lookup(&state.root, &jsonc_key_path(["b"])).expect("duplicate key lookup");
    assert_eq!(duplicate.number_token(), Some("3"));
    // An array has no keys.
    let array_state = parse_jsonc_edit("[1]").expect("valid array");
    assert!(matches!(
        jsonc_keys(&array_state.root, &[]).expect_err("array keys"),
        JsoncEditError::Type { .. }
    ));
}

/// Check replacement,
///  insertion and the addresses that must not invent structure.
#[test]
fn set_replaces_and_appends_only_a_final_segment() {
    let state = parse_jsonc_edit("{\"a\":1,\"list\":[1,2]} // doc").expect("valid document");
    // Replacing a value keeps the document comment.
    let replaced = jsonc_set(&state.root, &jsonc_key_path(["a"]), JsoncValue::boolean(true))
        .expect("replace scalar");
    assert!(emit_jsonc_value(&replaced).contains("\"a\": true"));
    assert!(emit_jsonc_value(&replaced).contains("// doc"));
    // The original state is unchanged, which is the immutability contract.
    assert!(jsonc_stringify(&state).contains("\"a\": 1"));
    // A missing final key is appended.
    let appended = jsonc_set(&state.root, &jsonc_key_path(["b"]), JsoncValue::null()).expect("append key");
    assert!(emit_jsonc_value(&appended).contains("\"b\": null"));
    // A missing final index at the array length appends an element.
    let mut tail = jsonc_key_path(["list"]);
    tail.push(at(2));
    let pushed = jsonc_set(&state.root, &tail, JsoncValue::number_from_token("3").expect("valid number"))
        .expect("append element");
    assert!(emit_jsonc_value(&pushed).contains('3'));
    // An index beyond the length is a missing address, not an insertion.
    let mut beyond = jsonc_key_path(["list"]);
    beyond.push(at(9));
    assert!(matches!(
        jsonc_set(&state.root, &beyond, JsoncValue::null()).expect_err("beyond length"),
        JsoncEditError::PathNotFound { .. }
    ));
    // A missing intermediate key is a missing address, not invented structure.
    let intermediate = jsonc_key_path(["deep", "deeper"]);
    assert!(matches!(
        jsonc_set(&state.root, &intermediate, JsoncValue::null()).expect_err("intermediate"),
        JsoncEditError::PathNotFound { .. }
    ));
    // Replacing the whole document keeps the root comment.
    let whole = jsonc_set(&state.root, &[], JsoncValue::array(Vec::new())).expect("replace root");
    assert_eq!(whole.comment, state.root.comment);
    // A wrong-shaped step is a shape failure.
    assert!(matches!(
        jsonc_set(&state.root, &[at(0)], JsoncValue::null()).expect_err("index against record"),
        JsoncEditError::Type { .. }
    ));
}

/// Check deletion of members and elements,
///  including duplicates and the root guard.
#[test]
fn delete_removes_members_and_elements() {
    let state = parse_jsonc_edit("{\"a\":1,\"dup\":2,\"dup\":3,\"list\":[10,20,30]}").expect("valid document");
    // Deleting a key removes every member with that name.
    let deduped = jsonc_delete(&state.root, &jsonc_key_path(["dup"])).expect("delete duplicates");
    assert_eq!(jsonc_keys(&deduped, &[]).expect("keys after delete").len(), 2);
    // Deleting an element shifts the rest.
    let mut middle = jsonc_key_path(["list"]);
    middle.push(at(1));
    let shortened = jsonc_delete(&state.root, &middle).expect("delete element");
    let list = jsonc_lookup(&shortened, &jsonc_key_path(["list"])).expect("list after delete");
    assert_eq!(list.elements().expect("array payload").len(), 2);
    // Deleting a member removes it from the document.
    let without_a = jsonc_delete(&state.root, &jsonc_key_path(["a"])).expect("delete member");
    assert!(!jsonc_has(&without_a, &jsonc_key_path(["a"])));
    // The document root cannot be deleted.
    assert!(matches!(
        jsonc_delete(&state.root, &[]).expect_err("delete root"),
        JsoncEditError::Type { .. }
    ));
    // A missing intermediate address is reported, while a missing final key deletes nothing.
    let intermediate = jsonc_key_path(["nope", "deeper"]);
    assert!(matches!(
        jsonc_delete(&state.root, &intermediate).expect_err("intermediate"),
        JsoncEditError::PathNotFound { .. }
    ));
    let unchanged = jsonc_delete(&state.root, &jsonc_key_path(["nope"])).expect("absent final key");
    assert_eq!(unchanged, state.root);
    // An out-of-range index is a missing address.
    let mut beyond = jsonc_key_path(["list"]);
    beyond.push(at(9));
    assert!(matches!(
        jsonc_delete(&state.root, &beyond).expect_err("range"),
        JsoncEditError::PathNotFound { .. }
    ));
}

/// Check that an edit and a deletion at the accepted depth complete without exhausting the stack.
#[test]
fn set_and_delete_at_accepted_depth() {
    let source = format!("{}0{}", "{\"k\":".repeat(512), "}".repeat(512));
    let state = parse_jsonc_edit(&source).expect("depth 512 document");
    let mut path: Vec<JsoncPathSegment> = Vec::new();
    for _ in 0..512 {
        path.push(JsoncPathSegment::Key { key: String::from("k") });
    }
    let replaced = jsonc_set(&state.root, &path, JsoncValue::boolean(true)).expect("deep set");
    let deep = jsonc_lookup(&replaced, &path).expect("deep lookup");
    assert!(matches!(deep.kind, JsoncKind::Boolean { value: true }));
    // Deleting the leaf empties the record that held it.
    let removed = jsonc_delete(&replaced, &path).expect("deep delete");
    let parent = jsonc_lookup(&removed, &path[..511]).expect("parent record");
    assert_eq!(parent.entries().expect("record payload").len(), 0);
}

/// Check the value constructors a caller uses to build replacement values.
#[test]
fn constructors_build_consistent_values() {
    // A quoted token keeps its spelling and decodes its escapes.
    let text = JsoncValue::text_from_token(r#""x\u0041""#).expect("valid quoted token");
    assert_eq!(text.text_units(), Some([120, 65].as_slice()));
    // Units derive their own legal quoted spelling, including a lone surrogate.
    let lone = JsoncValue::text_from_units(vec![0xD800]);
    assert_eq!(lone.text_units(), Some([0xD800].as_slice()));
    let emitted = emit_jsonc_value(&lone);
    assert!(emitted.contains("\\ud800"), "lone surrogate must stay escaped: {emitted}");
    // A number token is validated and normalized.
    let number = JsoncValue::number_from_token("1.500").expect("valid number token");
    assert_eq!(number.number_token(), Some("1.500"));
    assert!(JsoncValue::number_from_token("01").is_err(), "leading zero must be rejected");
    assert!(JsoncValue::number_from_token("+1").is_err(), "leading plus must be rejected");
    // Containers and scalars build without comments.
    assert!(JsoncValue::null().comment.is_none());
    assert!(JsoncValue::boolean(false).comment.is_none());
    assert_eq!(JsoncValue::array(Vec::new()).elements().expect("array payload").len(), 0);
    assert_eq!(JsoncValue::record(Vec::new()).entries().expect("record payload").len(), 0);
}

/// The document root must stay a container.
///  A root set to a scalar emits text the parser then
/// rejects,
///  which fuzzing found as an emission of `null`.
#[test]
fn root_set_to_a_scalar_is_refused() {
    let document = parse_jsonc_edit("{\"a\":1}").expect("source parses").root;
    let scalar = JsoncValue { kind: JsoncKind::Null, comment: None };
    let error = jsonc_set(&document, &[], scalar).expect_err("a scalar root must be refused");
    assert!(format!("{error}").contains("root"), "refusal did not name the root: {error}");

    // A container replacement at the root stays allowed, so the guard is about shape, not address.
    let replacement = JsoncValue { kind: JsoncKind::Array { elements: Vec::new() }, comment: None };
    assert!(jsonc_set(&document, &[], replacement).is_ok(), "a container root must be accepted");
}

/// A shape mismatch must name both the operation and the mismatch.
///  The generic scalar complaint is
/// a different arm,
///  so asserting the exact message is what distinguishes them.
#[test]
fn mismatched_segments_name_operation_and_shape() {
    let state = parse_jsonc_edit("{\"list\":[10],\"rec\":{\"a\":1}}").expect("document parses");
    let list_by_key = [JsoncPathSegment::Key { key: "list".to_string() }, JsoncPathSegment::Key { key: "a".to_string() }];
    let rec_by_position = [JsoncPathSegment::Key { key: "rec".to_string() }, at(0)];
    let replacement = JsoncValue { kind: JsoncKind::Null, comment: None };

    let lookup_error = jsonc_lookup(&state.root, &list_by_key).expect_err("an array takes no key");
    assert_eq!(format!("{lookup_error}"), "jsonc lookup: cannot index an array with key \"a\"");
    let lookup_position = jsonc_lookup(&state.root, &rec_by_position).expect_err("a record takes no position");
    assert_eq!(format!("{lookup_position}"), "jsonc lookup: cannot index an object with position 0");

    let set_error = jsonc_set(&state.root, &list_by_key, replacement.clone()).expect_err("set into an array by key");
    assert_eq!(format!("{set_error}"), "jsonc edit: cannot index an array with key \"a\"");
    let set_position = jsonc_set(&state.root, &rec_by_position, replacement).expect_err("set into a record by position");
    assert_eq!(format!("{set_position}"), "jsonc edit: cannot index an object with position 0");

    let delete_error = jsonc_delete(&state.root, &list_by_key).expect_err("delete from an array by key");
    assert_eq!(format!("{delete_error}"), "jsonc delete: cannot index an array with key \"a\"");
    let delete_position = jsonc_delete(&state.root, &rec_by_position).expect_err("delete from a record by position");
    assert_eq!(format!("{delete_position}"), "jsonc delete: cannot index an object with position 0");
}
