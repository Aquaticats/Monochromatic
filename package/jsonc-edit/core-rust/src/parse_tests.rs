//! What:
//!  Parser and emitter conformance probes at the public crate interface.
//! Why:
//!  Compilation alone does not show that comments,
//!  depth limits,
//!  UTF-16 escapes and exact
//!           numbers survive a parse,
//!  emit and reparse cycle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('JSONC parser', () => { /* valid, invalid, comments, UTF-16 */ });
//! ```

/// What:
///  Borrow public parser,
///  emitter,
///  and value tags from this crate.
/// Why:
///  Unit tests should exercise the same small interface a consumer would use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parse, emit, JsoncKind, JsoncCommentKind } from './index';
/// ```
use crate::path::JsoncPathSegment;
use crate::value::{JsoncComment, JsoncCommentKind, JsoncKind, JsoncValue};
use crate::{emit_jsonc_value, jsonc_set, parse_jsonc, parse_jsonc_edit};

/// Check valid JSONC source remains parseable after a canonical emission.
#[test]
fn valid_documents_round_trip() {
    for source in [
        "{}", "[]", "{\"a\":1,}", "[1,2,3,]",
        "[1\n,2,]", "{\"a\":1\n,\"b\":2,}",
        "[1/*before*/,/*after*/2]",
        "{\n  \"a\": 1, // inline\n  \"b\": 2\n}",
        "{\"a\":1}// end", "// start\n{\"a\":1}",
        "{/* key */\"a\":/* value */1}",
        "{\"n\":-1.5e3,\"s\":\"x\\u0041\",\"b\":true,\"z\":null}",
        "{\"nested\":{\"deep\":[true,false,null]}}",
    ] {
        let parsed = parse_jsonc(source).expect("valid JSONC input");
        let output = emit_jsonc_value(&parsed);
        parse_jsonc(&output).expect("canonical JSONC output must reparse");
    }
}

/// Check the JSONC-only root and grammar gates,
///  not JSON5 syntax.
#[test]
fn invalid_documents_fail() {
    for source in [
        "", "42", "\"bare string\"", "{'a':1}", "{a:1}",
        "{\"a\":1", "{\"a\":}", "{\"a\":1 \"b\":2}",
        "[1 2]", "[,]", "[1,,]", "[truefalse]", "[1true]", "[1e+]",
        "{\"a\":0x1F}", "{\"a\":1,,}", "{]", "[}", "{\"a\":1]",
        "{/* open\n\"a\":1}", "{\"a\":1}\u{00a0}",
        "{\"a\":+1}", "{\"a\":01}", "{\"a\":.5}",
        "{\"x\":\"\\uZZZZ\"}", "{\"x\":\"\\q\"}",
        "{\"x\":\"raw\nline\"}", "{\"x\":\"unterminated}",
    ] {
        assert!(parse_jsonc(source).is_err(), "accepted unsupported JSONC: {source}");
    }
}

/// Check arrays and root comments through the public parser and emitter.
#[test]
fn arrays_and_document_comments() {
    let parsed = parse_jsonc("[true,false,null,-1.5e3] // document").expect("valid array with trailing document comment");
    assert_eq!(parsed.comment.as_ref().expect("root comment").text, " document");
    let JsoncKind::Array { elements } = &parsed.kind else { panic!("expected array"); };
    assert!(matches!(&elements[0].kind, JsoncKind::Boolean { value: true }));
    assert!(matches!(&elements[1].kind, JsoncKind::Boolean { value: false }));
    assert!(matches!(&elements[2].kind, JsoncKind::Null));
    let JsoncKind::Number { raw, .. } = &elements[3].kind else { panic!("expected number"); };
    assert_eq!(raw, "-1.5e3");
    let emitted = emit_jsonc_value(&parsed);
    assert!(emitted.starts_with("// document\n["));
    assert!(emitted.contains("  -1.5e3,\n"));
    assert_eq!(parse_jsonc(&emitted).expect("canonical array reparses"), parsed);
}

/// Check that the last admitted structural depth can parse without stack exhaustion.
#[test]
fn depth_512_is_accepted() {
    let nested = format!("{}0{}", "[".repeat(512), "]".repeat(512));
    eprintln!("depth control: before parse");
    let parsed = parse_jsonc(&nested).expect("depth 512 is within the supported grammar");
    eprintln!("depth control: after parse");
    assert!(matches!(&parsed.kind, JsoncKind::Array { elements: _ }));
    eprintln!("depth control: before drop");
    drop(parsed);
    eprintln!("depth control: after drop");
}

/// Check that emission and reparse are also stack-safe at the accepted depth.
#[test]
fn deep_array_emit_and_reparse() {
    let source = format!("{}0{}", "[".repeat(512), "]".repeat(512));
    let parsed = parse_jsonc(&source).expect("valid deep array");
    eprintln!("deep emit: after parse");
    let emitted = emit_jsonc_value(&parsed);
    eprintln!("deep emit: after emit");
    let reparsed = parse_jsonc(&emitted).expect("emitted deep array reparses");
    eprintln!("deep emit: after reparse");
    drop(parsed);
    drop(reparsed);
    eprintln!("deep emit: after drop");
}

/// Check a deep record chain can emit,
///  reparse and clean up without a stack abort.
#[test]
fn deep_record_emit_and_reparse() {
    let source = format!("{}0{}", "{\"k\":".repeat(512), "}".repeat(512));
    let parsed = parse_jsonc(&source).expect("valid deep record");
    eprintln!("deep record: after parse");
    let emitted = emit_jsonc_value(&parsed);
    eprintln!("deep record: after emit");
    let reparsed = parse_jsonc(&emitted).expect("deep record reparses");
    eprintln!("deep record: after reparse");
    drop(parsed);
    drop(reparsed);
    eprintln!("deep record: after drop");
}

/// Check clone and equality also survive the accepted object-member chain.
#[test]
fn deep_record_clone_and_equality() {
    let source = format!("{}0{}", "{\"k\":".repeat(512), "}".repeat(512));
    let parsed = parse_jsonc(&source).expect("valid deep record");
    eprintln!("deep record clone: before clone");
    let cloned = parsed.clone();
    eprintln!("deep record clone: after clone");
    assert_eq!(parsed, cloned);
    eprintln!("deep record clone: after equality");
    drop(parsed);
    drop(cloned);
    eprintln!("deep record clone: after drop");
}

/// Check clone and equality used by immutable state operations at accepted depth.
#[test]
fn deep_clone_and_equality() {
    let source = format!("{}0{}", "[".repeat(512), "]".repeat(512));
    let parsed = parse_jsonc(&source).expect("valid deep array");
    eprintln!("deep clone: before clone");
    let cloned = parsed.clone();
    eprintln!("deep clone: after clone");
    assert_eq!(parsed, cloned);
    eprintln!("deep clone: after equality");
    drop(parsed);
    drop(cloned);
    eprintln!("deep clone: after drop");
}

/// Check an error after a deep completed subtree does not overflow during cleanup.
#[test]
fn deep_error_cleanup() {
    let child = format!("{}0{}", "[".repeat(511), "]".repeat(511));
    let malformed = format!("[{child},?]");
    eprintln!("deep error: before child error");
    assert!(parse_jsonc(&malformed).is_err());
    eprintln!("deep error: after child error");
    let bad_root = format!("{child} trailing");
    assert!(parse_jsonc(&bad_root).is_err());
    eprintln!("deep error: after root trailing error");
}

/// What:
///  Fail before examining malformed input beyond the 512th container opener.
/// Why:
///  The owned parser should bound its frame count even when the tail is invalid.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.throws(() => parse_jsonc('['.repeat(2048) + '{"a":'));
/// ```
#[test]
fn malformed_overdepth_rejects_before_tail() {
    // This source cannot close its containers and exceeds the chosen container budget.
    let source = format!("{}{{\"a\":", "[".repeat(2048));
    // The guard should return at the first excess opener without traversing the malformed tail.
    let error = parse_jsonc(&source).expect_err("513th opener must be rejected");
    assert_eq!(error.message, "JSONC nesting too deep");
    assert_eq!(error.offset, 512);
}

/// Check both scalar and empty-container spines reject the 513th opener.
#[test]
fn excessive_empty_nesting_is_rejected() {
    let rejected = format!("{}{}", "[".repeat(513), "]".repeat(513));
    assert_eq!(parse_jsonc(&rejected).expect_err("513th opener must fail").message, "JSONC nesting too deep");
    let accepted = format!("{}{}", "[".repeat(512), "]".repeat(512));
    parse_jsonc(&accepted).expect("512 empty arrays allowed");
}

/// Check many shallow siblings do not consume the depth budget cumulatively.
#[test]
fn wide_shallow_array_is_accepted() {
    let members = vec!["[]"; 100].join(",");
    let source = format!("[{members}]");
    parse_jsonc(&source).expect("independent sibling containers remain shallow");
}

/// Check structural recursion is rejected at the declared 512-depth ceiling.
#[test]
fn excessive_nesting_is_rejected() {
    let nested = format!("{}0{}", "[".repeat(513), "]".repeat(513));
    let error = parse_jsonc(&nested).expect_err("nesting beyond bound must fail");
    assert_eq!(error.message, "JSONC nesting too deep");
}

/// Check that object key comments remain separate from the value's own comment.
#[test]
fn comments_are_addressable() {
    let parsed = parse_jsonc("{/* key */\"a\":/* value */1}").expect("valid commented record");
    let JsoncKind::Record { entries } = &parsed.kind else { panic!("expected record"); };
    let entry = &entries[0];
    assert_eq!(entry.key.comment.as_ref().expect("key comment").text, " key ");
    assert_eq!(entry.value.comment.as_ref().expect("value comment").text, " value ");
    assert_eq!(entry.key.comment.as_ref().expect("key comment").kind, JsoncCommentKind::Block);
    assert_eq!(entry.value.comment.as_ref().expect("value comment").kind, JsoncCommentKind::Block);

    let stacked = parse_jsonc("{\n// l1\n// l2\n\"x\":1\n}").expect("stacked line comments");
    let JsoncKind::Record { entries: stacked_entries } = &stacked.kind else { panic!("expected record"); };
    let merged = stacked_entries[0].key.comment.as_ref().expect("merged key comment");
    assert_eq!(merged.kind, JsoncCommentKind::Line);
    assert_eq!(merged.text, " l1\n l2");
    assert!(emit_jsonc_value(&stacked).contains("// l1\n  // l2"));
}

/// Check the outer comment precedes an empty container's inner dangling comment.
#[test]
fn empty_container_comment_order() {
    let parsed = parse_jsonc("/*outer*/[/*inner*/]").expect("valid nested comment ownership");
    let comment = parsed.comment.as_ref().expect("merged container comment");
    assert_eq!(comment.kind, JsoncCommentKind::Block);
    assert_eq!(comment.text, "outer\ninner");
}

/// Check a multi-line value comment does not move onto its key after emission.
#[test]
fn multiline_value_comment_stays_on_value() {
    let parsed = parse_jsonc("{\"k\":/*value\ncomment*/1}").expect("valid value comment");
    let output = emit_jsonc_value(&parsed);
    let again = parse_jsonc(&output).expect("canonical comment output reparses");
    let JsoncKind::Record { entries } = &again.kind else { panic!("expected record"); };
    assert!(entries[0].key.comment.is_none(), "value comment migrated to key: {output}");
    assert_eq!(entries[0].value.comment.as_ref().expect("value comment").text, "value\ncomment");
}

/// Check the following object member survives a line-ending comment and keeps its body text.
fn assert_line_comment_boundary(source: &str) {
    let parsed = parse_jsonc(source).expect("JSONC line comment ends at carriage return");
    let JsoncKind::Record { entries } = &parsed.kind else { panic!("expected record"); };
    assert_eq!(entries.len(), 2);
    let comment = entries[0].value.comment.as_ref().expect("first value comment");
    assert_eq!(comment.kind, JsoncCommentKind::Line);
    assert_eq!(comment.text, " x");
}

/// Check CRLF does not include its carriage return in comment text.
#[test]
fn crlf_line_comment_body() {
    assert_line_comment_boundary("{\"a\":1,// x\r\n\"b\":2}");
}

/// Check CR alone terminates a line comment before the next member.
#[test]
fn cr_only_line_comment_boundary() {
    assert_line_comment_boundary("{\"a\":1,// x\r\"b\":2}");
}

/// Check an inline comment after a comma stays on the preceding value.
#[test]
fn inline_comment_after_comma_stays_on_value() {
    let source = "{\"a\":1, // inline\n\"b\":2}";
    let parsed = parse_jsonc(source).expect("valid inline comment after comma");
    let JsoncKind::Record { entries } = &parsed.kind else { panic!("expected record"); };
    assert_eq!(entries[0].value.comment.as_ref().expect("first value comment").text, " inline");
    assert!(entries[1].key.comment.is_none());
    let again = parse_jsonc(&emit_jsonc_value(&parsed)).expect("canonical record reparses");
    let JsoncKind::Record { entries: reparsed_entries } = &again.kind else { panic!("expected record"); };
    assert_eq!(reparsed_entries[0].value.comment.as_ref().expect("retained value comment").text, " inline");
    assert!(reparsed_entries[1].key.comment.is_none());
}

/// Check a raw-token adapter can reuse strict UTF-16 decoding and comment merging.
#[test]
fn adapter_scalar_helpers() {
    assert_eq!(crate::decode_quoted(r#""\uD800""#).expect("lone surrogate token"), vec![0xD800]);
    assert!(crate::decode_quoted(r#""\uZZZZ""#).is_err());
    assert!(crate::decode_quoted(r#""x" trailing"#).is_err());
    let left = Some(JsoncComment { kind: JsoncCommentKind::Line, text: "one".to_string() });
    let right = JsoncComment { kind: JsoncCommentKind::Block, text: "two".to_string() };
    let merged = crate::merge_comments(left, vec![right]).expect("merged comment");
    assert_eq!(merged.kind, JsoncCommentKind::Mixed);
    assert_eq!(merged.text, "one\ntwo");
}

/// Check UTF-16 string values keep escaped unpaired halves and valid pairs.
#[test]
fn escaped_utf16_is_not_lost() {
    for (literal, expected) in [
        (r#"\uD800"#, vec![0xD800]),
        (r#"\uDFFF"#, vec![0xDFFF]),
        (r#"\uD83D\uDE00"#, vec![0xD83D, 0xDE00]),
        (r#"\u0041"#, vec![0x0041]),
        (r#"\""#, vec![34]),
        (r#"\/"#, vec![47]),
        (r#"\b\f\n\r\t"#, vec![8, 12, 10, 13, 9]),
        ("😀", vec![0xD83D, 0xDE00]),
    ] {
        let source = format!("{{\"s\":\"{literal}\"}}");
        let state = parse_jsonc(&source).expect("valid escaped code-unit string");
        let JsoncKind::Record { entries } = &state.kind else { panic!("expected record"); };
        let JsoncKind::Text { units, raw } = &entries[0].value.kind else { panic!("expected string"); };
        assert_eq!(units, &expected);
        assert_eq!(raw, &format!("\"{literal}\""));
        let output = emit_jsonc_value(&state);
        let reparsed = parse_jsonc(&output).expect("emitted UTF-16 escapes parse");
        assert_eq!(state, reparsed);
    }
    let key = parse_jsonc(r#"{"\uD800":1}"#).expect("lone surrogate key");
    let JsoncKind::Record { entries } = &key.kind else { panic!("expected record"); };
    assert_eq!(entries[0].key.units, vec![0xD800]);
}

/// Check numbers compare exactly and preserve their raw spelling even on clean input.
#[test]
fn number_identity_and_raw_spelling() {
    let source = "{\"a\":1e0,\"b\":1,\"large\":9007199254740993}";
    let parsed = parse_jsonc(source).expect("valid numbers");
    let JsoncKind::Record { entries } = &parsed.kind else { panic!("expected record"); };
    let JsoncKind::Number { raw: first, identity: first_id } = &entries[0].value.kind else { panic!("expected number"); };
    let JsoncKind::Number { identity: second_id, .. } = &entries[1].value.kind else { panic!("expected number"); };
    assert_eq!(first_id, second_id);
    assert_eq!(first, "1e0");
    assert!(emit_jsonc_value(&parsed).contains("\"a\": 1e0,"));
    assert!(emit_jsonc_value(&parsed).contains("9007199254740993"));
}

/// Check adversarial edited comment text stays on its owner and reaches a fixpoint.
#[test]
fn comment_text_cannot_break_jsonc_syntax() {
    for body in [
        " x */ y ", "\"quoted\"", "\\escaped", "line\nnext", "line\n\nlast",
        "line\r\nnext", "/* nested */", "\0", "😀*/\nnext", "*/\n",
    ] {
        for on_key in [true, false] {
            let mut state = parse_jsonc("{\"a\":1}").expect("valid record");
            let JsoncKind::Record { entries } = &mut state.kind else { panic!("expected record"); };
            let inserted = Some(JsoncComment { kind: JsoncCommentKind::Block, text: body.to_string() });
            if on_key {
                entries[0].key.comment = inserted;
            } else {
                entries[0].value.comment = inserted;
            }
            let once = emit_jsonc_value(&state);
            let again = parse_jsonc(&once).expect("edited comment remains parseable");
            let JsoncKind::Record { entries: reparsed_entries } = &again.kind else { panic!("expected record"); };
            let retained = if on_key { reparsed_entries[0].key.comment.as_ref() }
                else { reparsed_entries[0].value.comment.as_ref() };
            assert_eq!(retained.expect("comment owner survives").text, body, "{once}");
            let twice = emit_jsonc_value(&again);
            let thrice = emit_jsonc_value(&parse_jsonc(&twice).expect("canonical output reparses"));
            assert_eq!(twice, thrice, "comment emission is not stable for {body:?}");
        }
    }
}

/// A block comment body may contain a bare CR,
///  and a `//` comment ends at CR,
///  so a body with one
/// must never be emitted in trailing form.
///  Fuzzing found this:
///  the emitted document no longer
/// parsed,
///  because the comment terminated early and the rest of the body became code.
#[test]
fn block_comment_body_with_cr_survives_round_trip() {
    let source = "[1 /* a\rb */]";
    let document = parse_jsonc(source).expect("source parses");
    let emitted = emit_jsonc_value(&document);
    let reparsed = parse_jsonc(&emitted).unwrap_or_else(|error| panic!("emission must reparse: {error}\nemitted: {emitted:?}"));
    let elements = reparsed.elements().expect("array root");
    let body = elements[0].comment.as_ref().map(|comment| return comment.text.clone());
    assert_eq!(body.as_deref(), Some(" a\rb "), "comment body did not survive the round trip");
}

/// A merged body carrying a bare CR reaches the leading `//` fallback rather than the block branch,
/// so the fallback must not emit a line containing CR either.
#[test]
fn merged_comment_body_with_cr_survives_round_trip() {
    let source = "[1, // one\r/* a\rb */]";
    let document = parse_jsonc(source).expect("source parses");
    let emitted = emit_jsonc_value(&document);
    let reparsed = parse_jsonc(&emitted).unwrap_or_else(|error| panic!("emission must reparse: {error}\nemitted: {emitted:?}"));
    let elements = reparsed.elements().expect("array root");
    let body = elements[0].comment.as_ref().map(|comment| return comment.text.clone()).unwrap_or_default();
    assert!(body.contains('a') && body.contains('b'), "merged body lost content: {body:?} from {emitted:?}");
    assert!(!body.contains('\r') || emitted.contains("/*"), "a CR body was emitted as a // line: {emitted:?}");
}

/// A block comment at offset zero exercises the scan loop's first iteration,
///  where an arithmetic
/// mistake on the offset underflows rather than merely miscounting.
#[test]
fn leading_block_comment_at_offset_zero_parses() {
    let document = parse_jsonc("/* lead */\n{\"a\":1}").expect("a leading block comment parses");
    let body = document.comment.as_ref().map(|comment| return comment.text.clone());
    assert_eq!(body.as_deref(), Some(" lead "), "leading block comment body");
}

/// A block comment body may contain a lone star or slash;
///  only the pair closes it.
#[test]
fn block_comment_body_may_contain_lone_star_and_slash() {
    let document = parse_jsonc("{\"a\":/* a * b / c */1}").expect("lone star and slash parse");
    let value = &document.entries().expect("record")[0].value;
    let body = value.comment.as_ref().map(|comment| return comment.text.clone());
    assert_eq!(body.as_deref(), Some(" a * b / c "), "block body was cut short");
}

/// A block comment that ends exactly at end of input must still be recognized as closed.
#[test]
fn block_comment_at_end_of_input_is_attached() {
    let document = parse_jsonc("{\"a\":1}/* tail */").expect("trailing block comment parses");
    let body = document.comment.as_ref().map(|comment| return comment.text.clone());
    assert_eq!(body.as_deref(), Some(" tail "), "end-of-input block comment body");
}

/// A space inside a string is emitted literally;
///  only control characters and surrogates are escaped.
#[test]
fn string_body_keeps_spaces_literal() {
    let document = parse_jsonc("{\"s\":\"a b\"}").expect("string with a space parses");
    let emitted = emit_jsonc_value(&document);
    assert!(emitted.contains("\"a b\""), "space was escaped in {emitted:?}");
    assert!(!emitted.contains("\\u0020"), "space was escaped in {emitted:?}");
}

/// A lone slash is not a comment opener,
///  and the refusal must name the character rather than
/// reporting the end of input that a mistaken comment scan would run into.
#[test]
fn lone_slash_is_rejected_naming_the_character() {
    let error = parse_jsonc("{\"a\": 1/2}").expect_err("a lone slash must be rejected");
    // The parser reads `1`, then meets `/` where a separator or close was required. A scanner that
    // mistook the lone slash for a comment opener would instead run to end of input.
    assert_eq!(error.message, "expected comma or container close", "refusal named the wrong failure");
}

/// Canonical layout indents two spaces per level,
///  so a nesting arithmetic mistake is visible.
#[test]
fn canonical_layout_indents_each_nesting_level() {
    let document = parse_jsonc("{\"a\":{\"b\":{\"c\":1}}}").expect("nested document parses");
    let emitted = emit_jsonc_value(&document);
    let pads: Vec<usize> = emitted
        .lines()
        .map(|line| return line.len() - line.trim_start().len())
        .filter(|pad| return *pad > 0)
        .collect();
    assert!(pads.contains(&2), "no line indented one level in {emitted:?}");
    assert!(pads.contains(&4), "no line indented two levels in {emitted:?}");
    assert!(pads.contains(&6), "no line indented three levels in {emitted:?}");
}

/// A single-line value comment is emitted after the value on the same line,
///  which is what
/// distinguishes trailing placement from leading placement.
#[test]
fn single_line_value_comment_emits_trailing() {
    let document = parse_jsonc("{\"a\": 1 /* note */}").expect("document parses");
    let emitted = emit_jsonc_value(&document);
    assert!(emitted.contains("1, // note"), "comment did not trail the value in {emitted:?}");
}

/// What:
///      Report the indentation of the first emitted line carrying a needle.
/// Why:
///       A substring search cannot pin indentation,
///  because a deeper pad contains every shallower
///           one;
///  the indent has to be measured on its own line.
fn indent_of_line_containing(emitted: &str, needle: &str) -> usize {
    for line in emitted.lines() {
        if line.contains(needle) {
            return line.len() - line.trim_start().len();
        }
    }
    panic!("no emitted line contains {needle} in {emitted:?}");
}

/// A lone slash after the root is trailing content,
///  not a comment opener.
#[test]
fn trailing_lone_slash_is_rejected() {
    let error = parse_jsonc("{\"a\":1} /").expect_err("a lone slash must be rejected");
    assert_eq!(error.message, "unexpected content after JSONC root");
}

/// Text built through the public constructor keeps a space literal rather than escaping it.
#[test]
fn constructed_text_keeps_spaces_literal() {
    let state = parse_jsonc_edit("{\"s\":1}").expect("document parses");
    let path = [JsoncPathSegment::Key { key: "s".to_string() }];
    let replacement = JsoncValue::text_from_units(vec![0x61, 0x20, 0x62]);
    let edited = jsonc_set(&state.root, &path, replacement).expect("set succeeds");
    let emitted = emit_jsonc_value(&edited);
    assert!(emitted.contains("\"a b\""), "space was escaped in {emitted:?}");
    assert!(!emitted.contains("\\u0020"), "space was escaped in {emitted:?}");
}

/// Array nesting indents two spaces per level,
///  the same way record nesting does.
#[test]
fn array_nesting_indents_each_level() {
    let document = parse_jsonc("[[[1]]]").expect("nested arrays parse");
    let emitted = emit_jsonc_value(&document);
    let pads: Vec<usize> = emitted
        .lines()
        .map(|line| return line.len() - line.trim_start().len())
        .filter(|pad| return *pad > 0)
        .collect();
    assert!(pads.contains(&2), "no line indented one level in {emitted:?}");
    assert!(pads.contains(&4), "no line indented two levels in {emitted:?}");
}

/// A multi-line value comment inside a nested record is indented to the value's own level.
#[test]
fn nested_multi_line_value_comment_is_indented_to_its_level() {
    let document = parse_jsonc("{\"a\":{\"b\":/*x\ny*/1}}").expect("document parses");
    let emitted = emit_jsonc_value(&document);
    assert_eq!(indent_of_line_containing(&emitted, "/*x"), 4, "wrong indent in {emitted:?}");
}

/// A multi-line comment on a nested array element is indented to the element's own level.
#[test]
fn nested_array_element_comment_is_indented_to_its_level() {
    let document = parse_jsonc("{\"a\":[/*x\ny*/1]}").expect("document parses");
    let emitted = emit_jsonc_value(&document);
    assert_eq!(indent_of_line_containing(&emitted, "/*x"), 4, "wrong indent in {emitted:?}");
}
