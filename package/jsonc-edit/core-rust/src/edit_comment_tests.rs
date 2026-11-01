//! What:
//!      Tests for the comment-as-data surface:
//!  reading and replacing value and key comments.
//! Why:
//!       A key's comment and its value's comment are separate data,
//!  and both must survive emission
//!           and reparsing on the same owner.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('jsonc comment api', () => { /* get, set, clear, ownership */ });
//! ```

/// What:
///      Import the failure enum so tests can name the expected variant.
/// Why:
///       A key-comment address can fail as missing or as wrong-shaped,
///  and the two differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncEditError } from './error';
/// ```
use crate::error::JsoncEditError;
/// What:
///      Import the canonical emitter.
/// Why:
///       Comment ownership is only meaningful if it survives writing the document out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { emitJsoncValue } from './emit';
/// ```
use crate::emit::emit_jsonc_value;
/// What:
///      Import the comment types a test builds and compares.
/// Why:
///       Setting a comment takes the same type a query returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncComment, JsoncCommentKind } from './value';
/// ```
use crate::value::{JsoncComment, JsoncCommentKind};
/// What:
///      Import the address helper and the comment surface under test.
/// Why:
///       These tests use the same public functions a consumer uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { jsoncComment, jsoncSetComment, jsoncKeyComment, jsoncSetKeyComment } from './index';
/// ```
use crate::{
    jsonc_comment, jsonc_key_comment, jsonc_set_comment, jsonc_set_key_comment, parse_jsonc,
    parse_jsonc_edit,
};
use crate::path::{jsonc_key_path, JsoncPathSegment};

/// What:
///      Build one block comment with the given body.
/// Why:
///       Tests set comments often,
///  and the style is part of what emission preserves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const block = (text: string) => ({ type: 'block', text });
/// ```
fn block(text: &str) -> JsoncComment {
    return JsoncComment { kind: JsoncCommentKind::Block, text: String::from(text) };
}

/// Check reading value and key comments separately.
#[test]
fn reads_value_and_key_comments() {
    let state = parse_jsonc_edit("{/* key */\"a\":/* value */1}").expect("valid commented document");
    let path = jsonc_key_path(["a"]);
    assert_eq!(jsonc_key_comment(&state.root, &path).expect("key comment query").map(|comment| return comment.text.as_str()), Some(" key "));
    assert_eq!(jsonc_comment(&state.root, &path).expect("value comment query").map(|comment| return comment.text.as_str()), Some(" value "));
    // An address with no comment reports absence rather than an error.
    let plain = parse_jsonc_edit("{\"b\":2}").expect("valid plain document");
    assert_eq!(jsonc_comment(&plain.root, &jsonc_key_path(["b"])).expect("absent comment"), None);
    assert_eq!(jsonc_key_comment(&plain.root, &jsonc_key_path(["b"])).expect("absent key comment"), None);
    // The document root can carry its own comment.
    let rooted = parse_jsonc_edit("{\"c\":3} // tail").expect("valid document with a trailing comment");
    assert_eq!(rooted.root.comment.as_ref().expect("root comment").text, " tail");
}

/// Check replacing and clearing a value comment without disturbing its key comment.
#[test]
fn sets_and_clears_value_comments() {
    let state = parse_jsonc_edit("{/* key */\"a\":1}").expect("valid commented document");
    let path = jsonc_key_path(["a"]);
    let replaced = jsonc_set_comment(&state.root, &path, Some(block(" new "))).expect("set value comment");
    assert_eq!(jsonc_comment(&replaced, &path).expect("value comment").map(|comment| return comment.text.as_str()), Some(" new "));
    // The key comment is untouched by a value-comment edit.
    assert_eq!(jsonc_key_comment(&replaced, &path).expect("key comment").map(|comment| return comment.text.as_str()), Some(" key "));
    // Passing absence clears the comment.
    let cleared = jsonc_set_comment(&replaced, &path, None).expect("clear value comment");
    assert_eq!(jsonc_comment(&cleared, &path).expect("cleared comment"), None);
    // The original state is unchanged.
    assert_eq!(jsonc_comment(&state.root, &path).expect("original comment"), None);
    // A missing address is reported rather than silently ignored.
    assert!(matches!(
        jsonc_set_comment(&state.root, &jsonc_key_path(["nope"]), Some(block(" x "))).expect_err("missing"),
        JsoncEditError::PathNotFound { .. }
    ));
}

/// Check replacing a key comment without disturbing the value comment.
#[test]
fn sets_and_clears_key_comments() {
    let state = parse_jsonc_edit("{\"a\":/* value */1}").expect("valid commented document");
    let path = jsonc_key_path(["a"]);
    let replaced = jsonc_set_key_comment(&state.root, &path, Some(block(" key "))).expect("set key comment");
    assert_eq!(jsonc_key_comment(&replaced, &path).expect("key comment").map(|comment| return comment.text.as_str()), Some(" key "));
    assert_eq!(jsonc_comment(&replaced, &path).expect("value comment").map(|comment| return comment.text.as_str()), Some(" value "));
    let cleared = jsonc_set_key_comment(&replaced, &path, None).expect("clear key comment");
    assert_eq!(jsonc_key_comment(&cleared, &path).expect("cleared key comment"), None);
}

/// Check the address shapes a key-comment operation must reject.
#[test]
fn key_comment_address_guards() {
    let state = parse_jsonc_edit("{\"a\":[1,2],\"s\":\"text\"}").expect("valid document");
    // An empty address has no final key.
    assert!(matches!(
        jsonc_key_comment(&state.root, &[]).expect_err("empty path"),
        JsoncEditError::Type { .. }
    ));
    assert!(matches!(
        jsonc_set_key_comment(&state.root, &[], Some(block(" x "))).expect_err("empty path"),
        JsoncEditError::Type { .. }
    ));
    // An index as the final segment cannot name a key.
    let indexed = [JsoncPathSegment::Index { index: 0 }];
    assert!(matches!(
        jsonc_key_comment(&state.root, &indexed).expect_err("index final segment"),
        JsoncEditError::Type { .. }
    ));
    // A parent that is not a record names no member.
    let mut into_array = jsonc_key_path(["a"]);
    into_array.push(JsoncPathSegment::Key { key: String::from("b") });
    assert!(matches!(
        jsonc_key_comment(&state.root, &into_array).expect_err("array parent"),
        JsoncEditError::PathNotFound { .. }
    ));
    assert!(matches!(
        jsonc_set_key_comment(&state.root, &into_array, Some(block(" x "))).expect_err("array parent"),
        JsoncEditError::PathNotFound { .. }
    ));
    // A missing member is a missing address.
    let missing = jsonc_key_path(["nope"]);
    assert!(matches!(
        jsonc_key_comment(&state.root, &missing).expect_err("missing member"),
        JsoncEditError::PathNotFound { .. }
    ));
    assert!(matches!(
        jsonc_set_key_comment(&state.root, &missing, Some(block(" x "))).expect_err("missing member"),
        JsoncEditError::PathNotFound { .. }
    ));
    // A scalar parent cannot hold members either.
    let into_scalar = jsonc_key_path(["s", "x"]);
    assert!(jsonc_key_comment(&state.root, &into_scalar).is_err());
}

/// Check that a multi-line value comment stays on its value through emission and reparsing.
#[test]
fn multiline_value_comment_keeps_its_owner() {
    let state = parse_jsonc_edit("{\"k\":/*value\ncomment*/1}").expect("valid multi-line comment document");
    let path = jsonc_key_path(["k"]);
    // Before emission the comment belongs to the value, not the key.
    assert_eq!(jsonc_key_comment(&state.root, &path).expect("key comment before emit"), None);
    assert_eq!(jsonc_comment(&state.root, &path).expect("value comment before emit").map(|comment| return comment.text.as_str()), Some("value\ncomment"));
    let emitted = emit_jsonc_value(&state.root);
    let reparsed = parse_jsonc_edit(&emitted).expect("canonical output reparses");
    // After the round trip the same owner still holds the same body.
    assert_eq!(jsonc_key_comment(&reparsed.root, &path).expect("key comment after emit"), None);
    assert_eq!(jsonc_comment(&reparsed.root, &path).expect("value comment after emit").map(|comment| return comment.text.as_str()), Some("value\ncomment"));
}

/// Check that a replaced comment survives emission on the owner it was set on.
#[test]
fn replaced_comments_survive_round_trip() {
    let state = parse_jsonc_edit("{\"a\":1,\"b\":[2]}").expect("valid document");
    let with_value = jsonc_set_comment(&state.root, &jsonc_key_path(["a"]), Some(block(" note "))).expect("set value comment");
    let mut element = jsonc_key_path(["b"]);
    element.push(JsoncPathSegment::Index { index: 0 });
    let with_element = jsonc_set_comment(&with_value, &element, Some(block(" item "))).expect("set element comment");
    let with_key = jsonc_set_key_comment(&with_element, &jsonc_key_path(["b"]), Some(block(" list "))).expect("set key comment");
    let emitted = emit_jsonc_value(&with_key);
    let reparsed = parse_jsonc_edit(&emitted).expect("canonical output reparses");
    assert_eq!(jsonc_comment(&reparsed.root, &jsonc_key_path(["a"])).expect("value comment").map(|comment| return comment.text.as_str()), Some(" note "));
    assert_eq!(jsonc_key_comment(&reparsed.root, &jsonc_key_path(["b"])).expect("key comment").map(|comment| return comment.text.as_str()), Some(" list "));
    assert_eq!(jsonc_comment(&reparsed.root, &element).expect("element comment").map(|comment| return comment.text.as_str()), Some(" item "));
}

/// A caller may attach a line-style comment whose body carries a bare CR.
///  Emission must not render
/// it as a `//` line,
///  because that comment would end at the CR and leave the rest as code.
#[test]
fn line_kind_body_with_cr_round_trips() {
    let state = parse_jsonc_edit("{\"a\":1}").expect("document parses");
    let path = [JsoncPathSegment::Key { key: "a".to_string() }];
    let comment = JsoncComment { kind: JsoncCommentKind::Line, text: " x\ry ".to_string() };
    let edited = jsonc_set_comment(&state.root, &path, Some(comment)).expect("comment attaches");
    let emitted = emit_jsonc_value(&edited);
    let reparsed = parse_jsonc(&emitted).unwrap_or_else(|error| panic!("emission must reparse: {error}\n{emitted:?}"));
    let body = jsonc_comment(&reparsed, &path).expect("query").map(|found| return found.text.clone());
    assert_eq!(body.as_deref(), Some(" x\ry "), "CR body did not survive: {emitted:?}");
}
