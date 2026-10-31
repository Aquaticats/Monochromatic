//! What:
//!  The property check for source spans, shared by the fuzz targets that parse documents.
//! Why:
//!  `parse_jsonc_with_spans` promises that each value and key span slices back to its own token,
//!  that spans come in document order, and that the tree equals the plain parse;
//!  a campaign finds inputs that break that promise where fixed tests cannot.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function assertSpansConsistent(source: string): void;
//! ```

/// What:
///  Import the crate's span-reporting parse, its result type and the plain parse.
/// Why:
///  The check is stated over the public API a consumer uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseJsonc, parseJsoncWithSpans, jsoncPositionAt, JsoncKey, JsoncKind, JsoncSpanned, JsoncValue } from 'monochromatic-jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{
    jsonc_position_at, parse_jsonc, parse_jsonc_with_spans, JsoncKey, JsoncKind, JsoncSpanned, JsoncValue,
};

/// What:
///  One pending step of the depth-first walk: a value to check or a key to check.
///  An `enum` is a type that is exactly one of several named shapes (TS: a discriminated union).
/// Why:
///  Pre-order puts a member's key before its value, and the walk keeps its own stack so a 512-deep
///  document cannot exhaust the call stack.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Step = { kind: 'value'; value: JsoncValue } | { kind: 'key'; key: JsoncKey };
/// ```
enum Step<'a> {
    /// A value whose span is next in the value list.
    Value(&'a JsoncValue),
    /// A key whose span is next in the key list.
    Key(&'a JsoncKey),
}

/// What:
///  Check a span-reporting parse result against the text it came from.
/// Why:
///  Taking the result as a parameter lets a unit test tamper with it and prove each rule can fail.
///  Rules: every span slices to its own token (containers to matching brackets), the lists are fully used,
///  spans nest inside their parent, and positions agree with the standalone position lookup.
///
/// # Arguments
///
/// * `source` - The text that was parsed.
/// * `spanned` - The result of `parse_jsonc_with_spans(source)`, possibly tampered with by a test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function assertSpansMatch(source: string, spanned: JsoncSpanned): void;
/// ```
pub fn assert_spans_match(source: &str, spanned: &JsoncSpanned) {
    let spans = &spanned.spans;
    let mut value_index = 0;
    let mut key_index = 0;
    // Each entry is a step plus the end offset of the enclosing container (the whole text for the root).
    let mut stack: Vec<(Step<'_>, usize)> = vec![(Step::Value(&spanned.value), source.len())];
    while let Some((step, parent_end)) = stack.pop() {
        match step {
            Step::Key(key) => {
                let span = *spans.keys.get(key_index).unwrap_or_else(|| panic!("a key has no span"));
                key_index += 1;
                assert_eq!(&source[span.start.offset..span.end.offset], key.raw, "key span does not slice to its token");
                assert!(span.end.offset <= parent_end, "key span leaves its container");
                assert_eq!(jsonc_position_at(source, span.start.offset), span.start, "key start position disagrees");
            }
            Step::Value(value) => {
                let span = *spans.values.get(value_index).unwrap_or_else(|| panic!("a value has no span"));
                value_index += 1;
                let slice = &source[span.start.offset..span.end.offset];
                assert!(span.end.offset <= parent_end, "value span leaves its container");
                assert_eq!(jsonc_position_at(source, span.start.offset), span.start, "value start position disagrees");
                assert_eq!(jsonc_position_at(source, span.end.offset), span.end, "value end position disagrees");
                match &value.kind {
                    JsoncKind::Text { raw, .. } | JsoncKind::Number { raw, .. } => assert_eq!(slice, raw, "scalar span"),
                    JsoncKind::Boolean { value: flag } => assert_eq!(slice, if *flag { "true" } else { "false" }),
                    JsoncKind::Null => assert_eq!(slice, "null"),
                    JsoncKind::Array { elements } => {
                        assert!(slice.starts_with('[') && slice.ends_with(']'), "array span {slice:?}");
                        for element in elements.iter().rev() {
                            stack.push((Step::Value(element), span.end.offset));
                        }
                    }
                    JsoncKind::Record { entries } => {
                        assert!(slice.starts_with('{') && slice.ends_with('}'), "record span {slice:?}");
                        for entry in entries.iter().rev() {
                            stack.push((Step::Value(&entry.value), span.end.offset));
                            stack.push((Step::Key(&entry.key), span.end.offset));
                        }
                    }
                }
            }
        }
    }
    assert_eq!(value_index, spans.values.len(), "unused value spans");
    assert_eq!(key_index, spans.keys.len(), "unused key spans");
}

/// What:
///  Parse a document that must parse, then check its spans and that the spanned parse equals the plain parse.
/// Why:
///  A fuzz target calls this on every accepted input.
///
/// # Arguments
///
/// * `source` - Document text that must already parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function assertSpansConsistent(source: string): void;
/// ```
pub fn assert_spans_consistent(source: &str) {
    let spanned = parse_jsonc_with_spans(source).unwrap_or_else(|error| panic!("spanned parse must accept: {error}"));
    let plain = parse_jsonc(source).unwrap_or_else(|error| panic!("plain parse must accept: {error}"));
    assert!(spanned.value == plain, "the spanned parse produced a different tree");
    assert_spans_match(source, &spanned);
}
