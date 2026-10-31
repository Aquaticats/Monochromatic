//! What:
//!  Probes for `parse_jsonc_with_spans`,
//! `jsonc_position_at` and the recorder's ordering rule.
//! Why:
//!  A position is only useful if it points at the node it claims to describe,
//!  so every test slices the text between a span's offsets and compares it to the node.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('JSONC spans', () => { it('slices back to each token', () => { /* walk tree */ }); });
//! ```

/// What:
///  Import the span API and the tree types the walkers inspect.
/// Why:
///  Tests use the same surface a consumer would,
/// plus the recorder to check its ordering guard.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseJsonc, parseJsoncWithSpans, jsoncPositionAt } from './index';
/// ```
use crate::span::SpanRecorder;
use crate::value::{JsoncKind, JsoncValue};
use crate::{JsoncSpans, jsonc_position_at, parse_jsonc, parse_jsonc_with_spans};

/// What:
///  Walk a tree depth-first and check each node's span slices back to that node's own token.
///  `&mut usize` is a borrowed counter this function may change (TS:
/// a `{ value: number }` box passed in).
/// Why:
///  This is the exact consumer contract:
/// one counter per span list,
/// advanced in document order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkSlices(node: JsoncValue, text: string, spans: JsoncSpans, counters: { v: number; k: number }): void
/// ```
fn check_slices(node: &JsoncValue, text: &str, spans: &JsoncSpans, value_index: &mut usize, key_index: &mut usize) {
    let span = spans.values[*value_index];
    *value_index += 1;
    let slice = &text[span.start.offset..span.end.offset];
    match &node.kind {
        JsoncKind::Text { raw, .. } | JsoncKind::Number { raw, .. } => assert_eq!(slice, raw),
        JsoncKind::Boolean { value } => assert_eq!(slice, if *value { "true" } else { "false" }),
        JsoncKind::Null => assert_eq!(slice, "null"),
        JsoncKind::Array { elements } => {
            assert!(slice.starts_with('[') && slice.ends_with(']'), "{slice:?}");
            for element in elements {
                check_slices(element, text, spans, value_index, key_index);
            }
        }
        JsoncKind::Record { entries } => {
            assert!(slice.starts_with('{') && slice.ends_with('}'), "{slice:?}");
            for entry in entries {
                let key_span = spans.keys[*key_index];
                *key_index += 1;
                assert_eq!(&text[key_span.start.offset..key_span.end.offset], entry.key.raw);
                check_slices(&entry.value, text, spans, value_index, key_index);
            }
        }
    }
}

/// Check every node of a document with every kind of value,
/// comment placement and line ending.
#[test]
fn every_span_slices_back_to_its_own_token() {
    for text in [
        "{}",
        "[]",
        "{\"a\":1,}",
        "[1,2,3,]",
        "// lead\n{\r\n  \"a\" /* k */ : /* v */ 1.50e3, // t\r\n  \"b\": [true,\n false, null, \"x\\u0041\\uD800\"],\n  \"c\": {\"d\": {\"e\": []}}\n} // end",
        "{\"\u{00e9}\u{1f600}\": \"\u{00e9}\"}",
        "[1,\r2]",
    ] {
        let spanned = parse_jsonc_with_spans(text).expect("valid");
        let mut value_index = 0;
        let mut key_index = 0;
        check_slices(&spanned.value, text, &spanned.spans, &mut value_index, &mut key_index);
        assert_eq!(value_index, spanned.spans.values.len(), "{text:?}: unused value spans");
        assert_eq!(key_index, spanned.spans.keys.len(), "{text:?}: unused key spans");
    }
}

/// Check the span-reporting parse yields the same tree and the same errors as the plain parse.
#[test]
fn the_spanned_parse_agrees_with_the_plain_parse() {
    for text in ["{\"a\":[1,{\"b\":null}],\"c\":\"d\"}", "[]", "// c\n{ /* k */ \"a\": 1 }"] {
        let spanned = parse_jsonc_with_spans(text).expect("valid");
        assert_eq!(spanned.value, parse_jsonc(text).expect("valid"), "{text:?}");
    }
    for text in ["", "42", "{\"a\":1", "[1 2]", "{\"a\":}", "{/* open\n\"a\":1}", "[1]x"] {
        assert_eq!(parse_jsonc_with_spans(text).map(|parsed| return parsed.value), parse_jsonc(text), "{text:?}");
    }
}

/// Check line and column:
/// 1-based,
/// LF ends a line,
/// CRLF is one LF after a column,
/// a bare CR is a column,
///  and a multi-byte character is one column.
#[test]
fn line_and_column_follow_the_documented_rule() {
    let text = "{\n  \"a\": 1,\r\n  \"\u{00e9}\u{1f600}\": [2]\n}";
    let spanned = parse_jsonc_with_spans(text).expect("valid");
    let first_key = spanned.spans.keys[0];
    assert_eq!((first_key.start.line, first_key.start.column), (2, 3));
    assert_eq!((first_key.end.line, first_key.end.column), (2, 6));
    let first_value = spanned.spans.values[1];
    assert_eq!((first_value.start.line, first_value.start.column), (2, 8));
    let second_key = spanned.spans.keys[1];
    assert_eq!((second_key.start.line, second_key.start.column), (3, 3));
    assert_eq!((second_key.end.line, second_key.end.column), (3, 7));
    let array = spanned.spans.values[2];
    assert_eq!((array.start.line, array.start.column), (3, 9));
    assert_eq!((array.end.line, array.end.column), (3, 12));
    let root = spanned.spans.values[0];
    assert_eq!((root.start.line, root.start.column, root.start.offset), (1, 1, 0));
    assert_eq!((root.end.line, root.end.column, root.end.offset), (4, 2, text.len()));
    let bare_cr = parse_jsonc_with_spans("[\r1]").expect("a bare CR is whitespace");
    assert_eq!((bare_cr.spans.values[1].start.line, bare_cr.spans.values[1].start.column), (1, 3));
}

/// Check a nested value lies inside its parent and spans come in non-decreasing start order.
#[test]
fn spans_nest_and_run_in_document_order() {
    let spanned = parse_jsonc_with_spans("{\"a\": [1, {\"b\": 2}], \"c\": 3}").expect("valid");
    let values = &spanned.spans.values;
    for pair in values.windows(2) {
        assert!(pair[0].start.offset <= pair[1].start.offset);
    }
    let root = values[0];
    for child in &values[1..] {
        assert!(root.start.offset <= child.start.offset && child.end.offset <= root.end.offset);
    }
    let keys = &spanned.spans.keys;
    for pair in keys.windows(2) {
        assert!(pair[0].end.offset <= pair[1].start.offset);
    }
}

/// Check the nesting limit is unchanged and a document at the limit still reports its spans.
#[test]
fn the_nesting_limit_and_spans_coexist() {
    let at_limit = format!("{}{}", "[".repeat(512), "]".repeat(512));
    let spanned = parse_jsonc_with_spans(&at_limit).expect("512 containers are accepted");
    assert_eq!(spanned.spans.values.len(), 512);
    assert_eq!(spanned.spans.values[0].end.offset, 1024);
    assert_eq!(spanned.spans.values[511].start.offset, 511);
    let over = format!("{}{}", "[".repeat(513), "]".repeat(513));
    assert_eq!(parse_jsonc_with_spans(&over).map(|parsed| return parsed.value), parse_jsonc(&over));
    assert!(parse_jsonc_with_spans(&over).is_err());
}

/// Check a large flat document reports one span per node,
/// so the recorder grows with the text and not faster.
#[test]
fn a_large_document_has_exactly_one_span_per_node() {
    let count = 50_000;
    let text = format!("[{}]", vec!["1"; count].join(","));
    let spanned = parse_jsonc_with_spans(&text).expect("valid");
    assert_eq!(spanned.spans.values.len(), count + 1);
    assert!(spanned.spans.keys.is_empty());
    assert_eq!(spanned.spans.values[count].end.column, text.len());
}

/// Check the standalone position lookup agrees with the spans and clamps offsets past the end.
#[test]
fn position_lookup_matches_the_spans_and_clamps() {
    let text = "{\n  \"a\": \u{00e9}1\n}";
    let end = jsonc_position_at(text, usize::MAX);
    assert_eq!(end.offset, text.len());
    assert_eq!((end.line, end.column), (3, 2));
    let spanned = parse_jsonc_with_spans("{\n  \"a\": 12,\n  \"b\": [3]\n}").expect("valid");
    for span in spanned.spans.keys.iter().chain(spanned.spans.values.iter()) {
        let source = "{\n  \"a\": 12,\n  \"b\": [3]\n}";
        assert_eq!(jsonc_position_at(source, span.start.offset), span.start);
        assert_eq!(jsonc_position_at(source, span.end.offset), span.end);
    }
}

/// Check a recorder asked for an earlier offset than it has reached stops loudly instead of reporting a wrong position.
#[test]
#[should_panic(expected = "non-decreasing")]
fn a_decreasing_offset_is_a_parser_bug_and_panics() {
    let mut recorder = SpanRecorder::new("[1,2]", true);
    recorder.scalar(3, 4);
    recorder.scalar(1, 2);
}

/// Check a disabled recorder records nothing,
/// which is what keeps a plain parse free of position work.
#[test]
fn a_disabled_recorder_records_nothing() {
    let mut recorder = SpanRecorder::new("[1]", false);
    recorder.container_start(0);
    recorder.scalar(1, 2);
    recorder.key(0, 1);
    recorder.container_end(3);
    assert_eq!(recorder.finish(), JsoncSpans::default());
}
