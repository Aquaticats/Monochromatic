//! What:
//!  Controls for core processing findings and host positions.
//! Why:
//!  These findings decide exit status 2 and tell a person where processing stopped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('run failure', () => { /* codes, flags, positions per language, panic text */ });
//! ```

/// Import the constructors and position helper under test.
use super::{
    FIX_REFUSED, PROCESSING_FAILURE, file_start, fix_refused, host_span, panic_text,
    processing_failure, processor_failure,
};
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::processors::ProcessorError;
use crate::run_paths::Language;

/// Both core findings are errors that mark incomplete processing and carry no fix.
#[test]
fn core_findings_are_error_severity_processing_failures() {
    let span: Span = Span {
        offset: 4,
        length: 0,
        line: 2,
        column: 1,
    };
    let failure: Diagnostic = processing_failure("a.md", span.clone(), String::from("reason"));
    assert_eq!(failure.code, PROCESSING_FAILURE);
    assert_eq!(failure.code, "core/processing-failure");
    assert_eq!(failure.severity, Severity::Error);
    assert_eq!(failure.filename, "a.md");
    assert_eq!(failure.message, "reason");
    assert_eq!(failure.labels[0].span, span);
    assert!(failure.processing_failure);
    assert!(failure.fix.is_none());
    let refused: Diagnostic = fix_refused("b.rs", String::from("why"));
    assert_eq!(refused.code, FIX_REFUSED);
    assert_eq!(refused.code, "core/fix-refused");
    assert_eq!(refused.severity, Severity::Error);
    assert_eq!(refused.filename, "b.rs");
    assert_eq!(refused.labels[0].span, file_start());
    assert!(refused.processing_failure);
    assert_eq!(
        file_start(),
        Span {
            offset: 0,
            length: 0,
            line: 1,
            column: 1
        }
    );
}

/// Rust positions count LF-delimited lines and byte columns;
///  a carriage return is an ordinary byte.
#[test]
fn rust_positions_use_lf_lines_and_byte_columns() {
    let source: &str = "ab\r\né x\n\nlast";
    assert_eq!(host_span(Language::Rust, source, 0), file_start());
    // Offset 4 is the first byte of the second line.
    let second: Span = host_span(Language::Rust, source, 4);
    assert_eq!((second.line, second.column, second.offset), (2, 1, 4));
    // The two-byte letter makes the following space byte column 3.
    let after_letter: Span = host_span(Language::Rust, source, 6);
    assert_eq!((after_letter.line, after_letter.column), (2, 3));
    // The carriage return at offset 2 is still on line 1, at byte column 3.
    let carriage: Span = host_span(Language::Rust, source, 2);
    assert_eq!((carriage.line, carriage.column), (1, 3));
    let empty_line: Span = host_span(Language::Rust, source, 9);
    assert_eq!((empty_line.line, empty_line.column), (3, 1));
    // An offset past the end is clamped to the end of the source.
    let clamped: Span = host_span(Language::Rust, source, 999);
    assert_eq!((clamped.offset, clamped.line, clamped.column), (14, 4, 5));
    assert_eq!(clamped.length, 0);
}

/// Markdown positions treat CR,
///  LF and CRLF as line ends and count UTF-16 columns.
#[test]
fn markdown_positions_use_markdown_line_ends_and_utf16_columns() {
    let source: &str = "a\rb\r\n🚀 x\n";
    let second: Span = host_span(Language::Markdown, source, 2);
    assert_eq!((second.line, second.column), (2, 1));
    // The rocket is four bytes and two UTF-16 units; the space after it is column 3.
    let after_rocket: Span = host_span(Language::Mdx, source, 9);
    assert_eq!((after_rocket.line, after_rocket.column), (3, 3));
    let clamped: Span = host_span(Language::Markdown, source, 999);
    assert_eq!(clamped.offset, source.len());
}

/// A processor refusal keeps its host filename,
///  message and offset,
///  positioned for the host language.
#[test]
fn processor_refusals_become_positioned_failures() {
    let error: ProcessorError = ProcessorError {
        filename: String::from("src/a.rs"),
        offset: 3,
        message: String::from("refused"),
    };
    let finding: Diagnostic = processor_failure(Language::Rust, "x\ny z\n", &error);
    assert_eq!(finding.filename, "src/a.rs");
    assert_eq!(finding.message, "refused");
    assert_eq!(finding.labels[0].span.offset, 3);
    assert_eq!(finding.labels[0].span.line, 2);
    assert_eq!(finding.labels[0].span.column, 2);
    assert!(finding.processing_failure);
}

/// Text payloads are read back;
///  any other payload is described without being dropped.
#[test]
fn panic_payload_text_is_recovered() {
    let owned: Box<dyn std::any::Any + Send> = Box::new(String::from("owned text"));
    assert_eq!(panic_text(owned.as_ref()), "owned text");
    let borrowed: Box<dyn std::any::Any + Send> = Box::new("static text");
    assert_eq!(panic_text(borrowed.as_ref()), "static text");
    let other: Box<dyn std::any::Any + Send> = Box::new(7_u8);
    assert_eq!(
        panic_text(other.as_ref()),
        "a panic payload that is not text"
    );
}
