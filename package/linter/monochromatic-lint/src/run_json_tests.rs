//! What:
//!  Controls for the strict JSON writer.
//! Why:
//!  The output must be accepted by a strict JSON parser,
//!  keep every value exactly,
//!  and drop
//! the comments and trailing commas JSONC allows.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(JSON.parse(strictJson(parseJsonc(source)))).toEqual(expected);
//! ```

/// Import the writer under test and both parsers used to check it.
use super::strict_json;
use monochromatic_jsonc_edit::{JsoncValue, parse_jsonc};

/// Parse JSONC and render it strictly.
fn rendered(source: &str) -> String {
    let value: JsoncValue = parse_jsonc(source).expect("JSONC fixture");
    return strict_json(&value);
}

/// Every value kind keeps its data;
///  comments and trailing commas are gone and the layout is fixed.
#[test]
fn every_kind_is_rendered_exactly_and_strictly() {
    let source: &str = r#"// leading comment
{
  "text": "plain", // trailing comment
  "number": 0e100,
  "exact": 300,
  "flags": [true, false, null,],
  "empty": { },
  "none": [ ],
  "nested": { "list": ["a/", "b/",], },
}"#;
    let output: String = rendered(source);
    assert_eq!(
        output,
        "{\n  \"text\": \"plain\",\n  \"number\": 0e100,\n  \"exact\": 300,\n  \"flags\": [\n    true,\n    false,\n    null\n  ],\n  \"empty\": {},\n  \"none\": [],\n  \"nested\": {\n    \"list\": [\n      \"a/\",\n      \"b/\"\n    ]\n  }\n}\n"
    );
    // A strict parser accepts the output and sees the same data.
    let strict: serde_json::Value =
        serde_json::from_str::<serde_json::Value>(output.as_str()).expect("strict JSON");
    assert_eq!(strict["nested"]["list"][1], "b/");
    assert_eq!(strict["exact"], 300);
    assert!(!output.contains("comment"));
    assert!(!output.contains(",\n  }"));
    assert!(!output.contains(",\n]"));
}

/// Text that needs escaping is re-encoded,
///  in keys and in values,
///  so the output stays one valid document.
#[test]
fn text_is_escaped_in_keys_and_values() {
    let source: &str = r#"{ "quote\" and \\ slash": "line\nbreak \u0001 tab\t 🚀 /tmp/a\"b" }"#;
    let output: String = rendered(source);
    let strict: serde_json::Value =
        serde_json::from_str::<serde_json::Value>(output.as_str()).expect("strict JSON");
    assert_eq!(
        strict["quote\" and \\ slash"],
        "line\nbreak \u{1} tab\t 🚀 /tmp/a\"b"
    );
    // The control character and the line break never appear raw inside the string literal.
    assert_eq!(output.lines().count(), 3);
    assert!(!output.contains('\u{1}'));
}

/// Empty and nested containers at the root are complete documents with one final newline.
/// The JSONC parser accepts only an object or an array at the root,
///  so scalars are checked as
/// the single element of an array,
///  where a number keeps its exact source token.
#[test]
fn root_containers_are_documents_and_numbers_keep_their_token() {
    assert_eq!(rendered("[]"), "[]\n");
    assert_eq!(rendered("{}"), "{}\n");
    assert_eq!(rendered("[\"x\"]"), "[\n  \"x\"\n]\n");
    assert_eq!(rendered("[-1.50]"), "[\n  -1.50\n]\n");
    assert_eq!(rendered("[true]"), "[\n  true\n]\n");
    assert_eq!(rendered("[[1]]"), "[\n  [\n    1\n  ]\n]\n");
    assert!(parse_jsonc("\"x\"").is_err());
}
