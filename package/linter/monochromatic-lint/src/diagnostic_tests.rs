//! What:
//!  JSONL wire-shape and escaping regressions.
//! Why:
//!  Messages and filenames may contain syntax delimiters;
//!  only the JSON encoder should interpret them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('JSONL diagnostic output', () => { /* exact keys and hostile strings */ });
//! ```

/// Import the production model,
///  renderer and internal fix type.
use super::{Diagnostic, Severity, Span, render};
use crate::edits::{Edit, Fix};

/// One finding with a source span whose fields are deliberately distinct.
fn fixture() -> Diagnostic {
    return Diagnostic::new(
        "rust/max-lines",
        Severity::Error,
        String::from("message"),
        String::from("file.rs"),
        Span {
            offset: 7,
            length: 3,
            line: 2,
            column: 4,
        },
    );
}

/// Wire fields retain their shape while optional values and internal fixes remain absent.
#[test]
fn stable_shape_omits_absent_hints_and_internal_fix() {
    let mut finding = fixture();
    finding.fix = Some(Fix {
        edits: vec![Edit {
            start: 0,
            end: 1,
            replacement: String::from("x"),
        }],
    });
    let output = render(&[finding]).expect("serialize finding");
    let value: serde_json::Value = serde_json::from_str(output.as_str()).expect("strict JSON");
    assert_eq!(value["code"], "rust/max-lines");
    assert_eq!(value["severity"], "error");
    assert_eq!(value["filename"], "file.rs");
    assert_eq!(value["causes"], serde_json::Value::Array(Vec::new()));
    assert_eq!(value["related"], serde_json::Value::Array(Vec::new()));
    assert_eq!(value["labels"][0]["span"]["offset"], 7);
    assert_eq!(value["labels"][0]["span"]["length"], 3);
    assert_eq!(value["labels"][0]["span"]["line"], 2);
    assert_eq!(value["labels"][0]["span"]["column"], 4);
    assert!(value.get("fix").is_none());
    assert!(value.get("url").is_none());
    assert!(value.get("help").is_none());
    assert!(output.ends_with('\n'));
}

/// Controls and delimiters cannot inject an additional diagnostic record.
#[test]
fn strings_are_encoded_at_the_json_boundary() {
    let mut finding = fixture();
    finding.message = String::from("quote \" backslash \\ newline\n NUL\0 emoji 🚀");
    finding.filename = String::from("odd\n\"name.rs");
    finding.severity = Severity::Warn;
    finding.url = Some(String::from("https://example.invalid/rule"));
    finding.help = Some(String::from("preserve \"quoted\" help"));
    let expected = finding.clone();
    let output = render(&[finding]).expect("encode hostile fields");
    assert_eq!(output.lines().count(), 1);
    let value: serde_json::Value = serde_json::from_str(output.as_str()).expect("strict JSON");
    assert_eq!(value["message"], expected.message);
    assert_eq!(value["filename"], expected.filename);
    assert_eq!(value["severity"], "warn");
    assert_eq!(value["url"], expected.url.expect("present URL"));
    assert_eq!(value["help"], expected.help.expect("present help"));
}

/// Processing status informs exit handling without changing the accepted JSONL schema.
#[test]
fn processing_failure_state_is_internal() {
    let mut finding: Diagnostic = fixture();
    assert!(!finding.processing_failure);
    finding.processing_failure = true;
    let output: String = render(&[finding]).expect("encode processing diagnostic");
    let record: serde_json::Value = serde_json::from_str(output.as_str()).expect("diagnostic JSON");
    assert!(record.get("processing_failure").is_none());
}

/// A clean run prints no records,
///  and multiple findings retain order without pretty-printing.
#[test]
fn clean_and_multiple_outputs_have_exact_line_cardinality() {
    assert_eq!(render(&[]).expect("empty findings"), "");
    let first = fixture();
    let mut second = fixture();
    second.code = String::from("markdown/no-bare-urls");
    let output = render(&[first, second]).expect("two findings");
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2);
    let first_record: serde_json::Value = serde_json::from_str(lines[0]).expect("first line");
    let second_record: serde_json::Value = serde_json::from_str(lines[1]).expect("second line");
    assert_eq!(first_record["code"], "rust/max-lines");
    assert_eq!(second_record["code"], "markdown/no-bare-urls");
}
