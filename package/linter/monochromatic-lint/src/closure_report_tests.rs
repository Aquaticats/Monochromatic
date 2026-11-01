//! What:
//!  Controls of the reading of cargo's JSON records.
//! Why:
//!  The records are the only evidence the check has,
//! so a wrong code,
//! a missing span,
//! a duplicate from a second target
//!  and a compiler error must each be handled on purpose.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('reads records', () => expect(parseRecords(lines).reports).toEqual([...]));
//! ```

use super::{REPORTED_CODES, parse_records};
use serde_json::Value;

/// Parse JSON text that the test wrote itself.
fn value(text: &str) -> Value {
    return serde_json::from_str::<Value>(text).expect("the fixture is JSON");
}

/// One compiler-message record with a code,
/// level,
/// message and one primary span.
fn record(code: Option<&str>, level: &str, message: &str, file: &str, start: u64) -> Value {
    let code_text: String = match code {
        Some(text) => format!("{{\"code\": {text:?}, \"explanation\": null}}"),
        None => String::from("null"),
    };
    return value(
        format!(
            r#"{{"reason": "compiler-message", "message": {{
                "code": {code_text}, "level": {level:?}, "message": {message:?}, "rendered": "{level}: {message}\n",
                "spans": [
                  {{"file_name": "ignored.rs", "is_primary": false, "byte_start": 1, "byte_end": 2, "line_start": 1, "column_start": 1}},
                  {{"file_name": {file:?}, "is_primary": true, "byte_start": {start}, "byte_end": {end}, "line_start": 3, "column_start": 9}}
                ],
                "children": [
                  {{"level": "note", "message": "a note", "spans": [], "children": [], "code": null}},
                  {{"level": "help", "message": "a help", "spans": [], "children": [], "code": null}},
                  {{"level": "help", "message": "a second help", "spans": [], "children": [], "code": null}}
                ]}}}}"#,
            end = start + 5
        )
        .as_str(),
    );
}

/// Join records as the lines of standard output.
fn lines(records: &[Value]) -> Vec<u8> {
    let mut text: String = String::new();
    for value in records {
        text.push_str(value.to_string().as_str());
        text.push('\n');
    }
    return text.into_bytes();
}

/// Both codes are findings;
/// the primary span and the first help are used.
#[test]
fn reads_both_reported_codes_with_primary_span_and_first_help() {
    let parsed = parse_records(&lines(&[
        record(
            Some("avoidable_closure"),
            "warning",
            "a closure",
            "src/a.rs",
            10,
        ),
        record(
            Some("unfulfilled_lint_expectations"),
            "warning",
            "stale expect",
            "src/b.rs",
            20,
        ),
    ]));
    assert_eq!(parsed.reports.len(), 2);
    assert_eq!(parsed.reports[0].file, "src/a.rs");
    assert_eq!(
        (parsed.reports[0].byte_start, parsed.reports[0].byte_end),
        (10, 15)
    );
    assert_eq!((parsed.reports[0].line, parsed.reports[0].column), (3, 9));
    assert_eq!(parsed.reports[0].help.as_deref(), Some("a help"));
    assert_eq!(parsed.reports[1].message, "stale expect");
    assert!(REPORTED_CODES.contains(&"avoidable_closure"));
    assert_eq!(parsed.compiler_error, None);
}

/// Other lints,
/// compiler messages without a code and records of other reasons are not findings.
#[test]
fn ignores_other_codes_and_other_records() {
    let mut other: Value = record(Some("avoidable_closure"), "warning", "x", "src/a.rs", 1);
    other["reason"] = Value::from("compiler-artifact");
    let parsed = parse_records(&lines(&[
        record(Some("dead_code"), "warning", "unused", "src/a.rs", 1),
        record(None, "warning", "no code", "src/a.rs", 2),
        other,
        value(r#"{"reason": "build-finished", "success": true}"#),
    ]));
    assert!(parsed.reports.is_empty());
    assert_eq!(parsed.compiler_error, None);
}

/// A second target that compiles the same line adds nothing;
/// a different message at the same place does.
#[test]
fn deduplicates_a_report_seen_for_two_targets() {
    let first: Value = record(Some("avoidable_closure"), "warning", "m", "src/a.rs", 10);
    let parsed = parse_records(&lines(&[
        first.clone(),
        first,
        record(
            Some("avoidable_closure"),
            "warning",
            "other message",
            "src/a.rs",
            10,
        ),
    ]));
    assert_eq!(parsed.reports.len(), 2);
}

/// Text that is not JSON,
/// blank lines and non-object JSON are skipped without losing later records.
#[test]
fn skips_lines_that_are_not_records() {
    let mut bytes: Vec<u8> = b"   Compiling something\n\nnot json\n[1,2]\n42\n".to_vec();
    bytes.extend(lines(&[record(
        Some("avoidable_closure"),
        "warning",
        "m",
        "src/a.rs",
        1,
    )]));
    assert_eq!(parse_records(&bytes).reports.len(), 1);
}

/// Invalid UTF-8 in a line does not stop the reading.
#[test]
fn survives_invalid_utf8() {
    let mut bytes: Vec<u8> = vec![0xff, 0xfe, b'\n'];
    bytes.extend(lines(&[record(
        Some("avoidable_closure"),
        "warning",
        "m",
        "src/a.rs",
        1,
    )]));
    assert_eq!(parse_records(&bytes).reports.len(), 1);
}

/// The first error-level message is kept as the explanation,
/// and a later one does not replace it.
#[test]
fn keeps_the_first_compiler_error() {
    let parsed = parse_records(&lines(&[
        record(Some("E0425"), "error", "first failure", "src/a.rs", 1),
        record(
            None,
            "error",
            "aborting due to 1 previous error",
            "src/a.rs",
            2,
        ),
    ]));
    assert_eq!(
        parsed.compiler_error.as_deref(),
        Some("error: first failure")
    );
}

/// A report without a primary span,
/// with a negative or fractional offset,
/// or without a file cannot be placed.
#[test]
fn drops_reports_that_cannot_be_placed() {
    let mut no_primary: Value = record(Some("avoidable_closure"), "warning", "m", "src/a.rs", 1);
    no_primary["message"]["spans"] = value(r#"[{"file_name": "src/a.rs", "is_primary": false}]"#);
    let mut negative: Value = record(Some("avoidable_closure"), "warning", "m", "src/a.rs", 1);
    negative["message"]["spans"][1]["byte_start"] = Value::from(-4);
    let mut fractional: Value = record(Some("avoidable_closure"), "warning", "m", "src/a.rs", 1);
    fractional["message"]["spans"][1]["byte_end"] = Value::from(1.5);
    let mut no_file: Value = record(Some("avoidable_closure"), "warning", "m", "src/a.rs", 1);
    no_file["message"]["spans"][1]["file_name"] = Value::Null;
    let parsed = parse_records(&lines(&[no_primary, negative, fractional, no_file]));
    assert!(parsed.reports.is_empty());
}

/// Empty output has no reports and no error.
#[test]
fn empty_output_is_empty() {
    assert_eq!(parse_records(b""), super::Parsed::default());
}
