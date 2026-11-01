//! What:
//!      Tests for the remaining public surface:
//!  address helpers,
//!  identity accessors,
//!  comment merging
//!           and every error rendering.
//! Why:
//!       A published crate is not complete until each exported item has been exercised,
//!  including the
//!           diagnostics a consumer prints.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('public api', () => { /* paths, identities, comments, errors */ });
//! ```

/// What:
///      Import the error types whose rendering is part of the public contract.
/// Why:
///       Shared fixtures compare messages across the Rust and TypeScript implementations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncParseError, JsoncNumberError } from './error';
/// ```
use crate::error::{
    JsoncEditError, JsoncNumberError, JsoncParseError, JsoncPathNotFoundError, JsoncTypeError,
};
/// What:
///      Import the comment merge function and the comment types.
/// Why:
///       Stacked comments collapse into one queryable comment per owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { mergeComments } from './mergeComments';
/// ```
use crate::merge_comments;
/// What:
///      Import the exact-number identity.
/// Why:
///       Its accessors are how a consumer inspects a normalized value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncNumberIdentity } from './number';
/// ```
use crate::number::JsoncNumberIdentity;
/// What:
///      Import the address helpers.
/// Why:
///       Callers build addresses from key text and ask which kind a segment is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { jsoncKeyPath, JsoncPathSegment } from './path';
/// ```
use crate::path::{jsonc_key_path, JsoncPathSegment};
/// What:
///      Import the quoted-text conversion and the document model.
/// Why:
///       Key and value constructors are part of the edit surface a consumer uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { decodeQuoted, JsoncComment, JsoncKey } from './index';
/// ```
use crate::text_units::decode_quoted;
use crate::value::{JsoncComment, JsoncCommentKind, JsoncKey, JsoncValue};
use crate::{jsonc_has, parse_jsonc};

/// Check the address helpers build and classify segments.
#[test]
fn path_helpers_build_and_classify_segments() {
    // An empty key list yields an empty address, which names the document root.
    let empty = jsonc_key_path([]);
    assert!(empty.is_empty());
    let path = jsonc_key_path(["a", "b"]);
    assert_eq!(path.len(), 2);
    assert!(path[0].is_key());
    assert!(!path[0].is_index());
    let index = JsoncPathSegment::Index { index: 3 };
    assert!(index.is_index());
    assert!(!index.is_key());
    // A built address works against a parsed document.
    let state = parse_jsonc("{\"a\":{\"b\":1}}").expect("valid document");
    assert!(jsonc_has(&state, &path));
}

/// Check the identity accessors report the normalized parts.
#[test]
fn identity_accessors_expose_normalized_parts() {
    let scaled = JsoncNumberIdentity::from_token("1.500").expect("valid token");
    assert_eq!(scaled.digits(), "15");
    assert_eq!(scaled.exponent(), "-1");
    assert!(!scaled.is_zero());
    assert!(!scaled.is_negative());
    let shifted = JsoncNumberIdentity::from_token("100").expect("valid token");
    assert_eq!(shifted.digits(), "1");
    assert_eq!(shifted.exponent(), "2");
    // Every zero spelling collapses to one canonical zero, including a negative one.
    for token in ["0", "-0", "0.0", "-0e1000"] {
        let zero = JsoncNumberIdentity::from_token(token).expect("valid zero token");
        assert!(zero.is_zero(), "not zero: {token}");
        assert!(!zero.is_negative(), "negative zero kept its sign: {token}");
        assert_eq!(zero.digits(), "0");
        assert_eq!(zero.exponent(), "0");
    }
    let negative = JsoncNumberIdentity::from_token("-2.5").expect("valid token");
    assert!(negative.is_negative());
    assert_eq!(negative.digits(), "25");
    assert_eq!(negative.exponent(), "-1");
    // A huge exponent stays text rather than widening into a machine integer.
    let huge_digits = "1".to_string() + "0".repeat(40).as_str();
    let huge_token = format!("1e{huge_digits}");
    let huge = JsoncNumberIdentity::from_token(&huge_token).expect("valid huge exponent");
    assert_eq!(huge.exponent(), huge_digits.as_str());
    assert_eq!(huge.digits(), "1");
}

/// Check each number grammar rejection names the part it failed on.
#[test]
fn number_errors_name_their_grammar_part() {
    let cases: [(&str, JsoncNumberError, &str); 8] = [
        ("", JsoncNumberError::Empty, "number token is empty"),
        ("+1", JsoncNumberError::LeadingPlus, "number token starts with +"),
        ("01", JsoncNumberError::LeadingZero, "number token has a leading zero"),
        (".5", JsoncNumberError::MissingIntegerDigit, "number token has no integer digit"),
        ("1.", JsoncNumberError::MissingFractionDigit, "number token has no fraction digit"),
        ("1e", JsoncNumberError::MissingExponentDigit, "number token has no exponent digit"),
        ("1e+", JsoncNumberError::MissingExponentDigit, "number token has no exponent digit"),
        ("1.2.3", JsoncNumberError::UnexpectedSuffix, "number token has an unexpected trailing byte"),
    ];
    for (token, expected, message) in cases {
        let error = JsoncNumberIdentity::from_token(token).expect_err("invalid number token");
        assert_eq!(error, expected, "wrong variant for {token:?}");
        assert_eq!(error.to_string(), message, "wrong message for {token:?}");
    }
}

/// Check comment merging keeps one comment per owner and marks mixed styles.
#[test]
fn merge_comments_collapses_stacks_in_order() {
    let line = |text: &str| return JsoncComment { kind: JsoncCommentKind::Line, text: String::from(text) };
    let block = |text: &str| return JsoncComment { kind: JsoncCommentKind::Block, text: String::from(text) };
    // Merging nothing into absence stays absent.
    assert_eq!(merge_comments(None, Vec::new()), None);
    // A single addition is stored unchanged, keeping its own style.
    let single = merge_comments(None, vec![block(" b ")]).expect("one comment");
    assert_eq!(single.kind, JsoncCommentKind::Block);
    assert_eq!(single.text, " b ");
    // Two of the same style keep that style and join bodies in source order.
    let same = merge_comments(Some(line("one")), vec![line("two")]).expect("merged lines");
    assert_eq!(same.kind, JsoncCommentKind::Line);
    assert_eq!(same.text, "one\ntwo");
    // Different styles become mixed while the bodies stay ordered.
    let mixed = merge_comments(Some(line("one")), vec![block("two")]).expect("mixed merge");
    assert_eq!(mixed.kind, JsoncCommentKind::Mixed);
    assert_eq!(mixed.text, "one\ntwo");
    // Several additions fold left to right.
    let folded = merge_comments(None, vec![line("a"), line("b"), block("c")]).expect("folded merge");
    assert_eq!(folded.kind, JsoncCommentKind::Mixed);
    assert_eq!(folded.text, "a\nb\nc");
}

/// Check each error type renders the documented text and behaves as a Rust error.
#[test]
fn errors_render_documented_text() {
    let parse = JsoncParseError { offset: 7, message: String::from("unterminated object") };
    assert_eq!(parse.to_string(), "unterminated object (at offset 7)");
    let path = JsoncPathNotFoundError { path: jsonc_key_path(["a", "b"]) };
    assert!(path.to_string().starts_with("no JSONC node at path "), "unexpected text: {path}");
    let shape = JsoncTypeError { message: String::from("target is not an object") };
    assert_eq!(shape.to_string(), "target is not an object");
    // The edit enum delegates to whichever failure it carries.
    let missing = JsoncEditError::PathNotFound { error: path.clone() };
    assert_eq!(missing.to_string(), path.to_string());
    let wrong = JsoncEditError::Type { error: shape.clone() };
    assert_eq!(wrong.to_string(), shape.to_string());
    // Every type works as a standard-library error, so `?` and error chains can carry it.
    let boxed: Vec<Box<dyn std::error::Error>> = vec![Box::new(parse), Box::new(path), Box::new(shape)];
    assert_eq!(boxed.len(), 3);
    // A parse failure from the public parser names the operation and renders its byte offset.
    let source = "{\"a\":";
    let failure = parse_jsonc(source).expect_err("incomplete document");
    let rendered = failure.to_string();
    assert!(!failure.message.is_empty(), "a parse failure must explain itself");
    assert!(rendered.contains(&format!("(at offset {})", failure.offset)), "offset not rendered: {rendered}");
    assert!(failure.offset <= source.len(), "offset must point inside the rejected source");
    // Every error type is usable as a boxed standard-library error.
    let boxed_parse: Box<dyn std::error::Error> = Box::new(failure);
    assert!(boxed_parse.to_string().contains("at offset"), "boxed error lost its text");
}

/// Check key constructors keep spelling and decode units consistently.
#[test]
fn key_constructors_agree_on_spelling_and_units() {
    let from_text = JsoncKey::from_text("a\"b");
    assert_eq!(from_text.units, [97, 34, 98]);
    assert_eq!(from_text.raw, r#""a\"b""#);
    assert!(from_text.comment.is_none());
    // A key built from a token round-trips through the decoder.
    let from_token = JsoncKey::from_token(r#""a\"b""#).expect("valid quoted key");
    assert_eq!(from_token.units, from_text.units);
    assert_eq!(from_token.raw, from_text.raw);
    // A lone surrogate survives in a key.
    let lone = JsoncKey::from_token(r#""\uD800""#).expect("valid quoted key");
    assert_eq!(lone.units, [0xD800]);
    assert_eq!(decode_quoted(&lone.raw).expect("decodable key"), lone.units);
    // A malformed token is rejected rather than half-decoded.
    assert!(JsoncKey::from_token(r#""abc"#).is_err());
    // A value constructor rejects a malformed number token.
    assert!(JsoncValue::number_from_token("1e").is_err());
}
