//! What:
//!  The core's own findings:
//!  inability to process an input,
//!  and refused fixes.
//! Why:
//!  A file that could not be read,
//!  parsed,
//!  mapped or safely fixed must appear in the JSONL
//! output and fail the run with status 2;
//!  it must never look like a clean file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // processingFailure(filename, span, message); fixRefused(filename, message)
//! ```

/// Import the finding model and both languages' position conventions.
use crate::{
    diagnostic::{Diagnostic, Severity, Span},
    markdown_positions::MarkdownPositions,
    processors::ProcessorError,
    run_paths::Language,
};

/// What:
///  The code of a finding that reports incomplete processing.
/// Why:
///  Parse failures,
///  MDX errors,
///  caught panics,
///  unreadable inputs and unavailable semantic
/// coverage share one code;
///  the message names the operation that failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PROCESSING_FAILURE = 'core/processing-failure';
/// ```
pub const PROCESSING_FAILURE: &str = "core/processing-failure";

/// What:
///  The code of a finding that reports a fix the engine declined to apply.
/// Why:
///  A consumer can tell "this file was not rewritten" from an ordinary rule finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FIX_REFUSED = 'core/fix-refused';
/// ```
pub const FIX_REFUSED: &str = "core/fix-refused";

/// What:
///  The zero-width position at the start of a file.
/// Why:
///  A failure that concerns the whole file still needs a position in the JSONL record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fileStart = { offset: 0, length: 0, line: 1, column: 1 };
/// ```
pub fn file_start() -> Span {
    return Span {
        offset: 0,
        length: 0,
        line: 1,
        column: 1,
    };
}

/// What:
///  Resolve a host byte offset to a zero-width span in that language's column convention.
/// Why:
///  Rust findings use LF-delimited lines with byte columns;
///  Markdown findings use CR,
///  LF or
/// CRLF lines with UTF-16 columns.
///  No parser runs here,
///  so a failure caused by a parser can still be placed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hostSpan(language: Language, source: string, offset: number): Span;
/// ```
pub fn host_span(language: Language, source: &str, offset: usize) -> Span {
    let clamped: usize = offset.min(source.len());
    if language != Language::Rust {
        return MarkdownPositions::new(source).span(clamped, 0);
    }
    let mut line: usize = 1;
    let mut line_start: usize = 0;
    for (index, byte) in source.as_bytes()[..clamped].iter().enumerate() {
        if *byte == b'\n' {
            line += 1;
            line_start = index + 1;
        }
    }
    return Span {
        offset: clamped,
        length: 0,
        line,
        column: clamped - line_start + 1,
    };
}

/// What:
///  Build an error-severity finding that marks incomplete processing.
/// Why:
///  `processing_failure` makes the run exit with status 2 and stops the fixer from publishing edits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function processingFailure(filename: string, span: Span, message: string): Diagnostic;
/// ```
pub fn processing_failure(filename: &str, span: Span, message: String) -> Diagnostic {
    let mut finding: Diagnostic = Diagnostic::new(
        PROCESSING_FAILURE,
        Severity::Error,
        message,
        String::from(filename),
        span,
    );
    finding.processing_failure = true;
    return finding;
}

/// What:
///  Build the finding for a fix the engine refused,
///  such as one that would empty a non-empty file.
/// Why:
///  The file's bytes are unchanged,
///  and the run must say so instead of reporting only what remained.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fixRefused(filename: string, message: string): Diagnostic;
/// ```
pub fn fix_refused(filename: &str, message: String) -> Diagnostic {
    let mut finding: Diagnostic = Diagnostic::new(
        FIX_REFUSED,
        Severity::Error,
        message,
        String::from(filename),
        file_start(),
    );
    finding.processing_failure = true;
    return finding;
}

/// What:
///  Convert a processor refusal into a host-positioned processing failure.
/// Why:
///  The processor reports an original-host byte offset;
///  the finding needs line and column too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function processorFailure(language: Language, source: string, error: ProcessorError): Diagnostic;
/// ```
pub fn processor_failure(language: Language, source: &str, error: &ProcessorError) -> Diagnostic {
    return processing_failure(
        error.filename.as_str(),
        host_span(language, source, error.offset),
        error.message.clone(),
    );
}

/// What:
///  Read the text of a caught panic payload.
/// Why:
///  A panic's message is the only explanation available once unwinding has been stopped.
/// `&(dyn Any + Send)` is a borrowed value of unknown type;
///  `downcast_ref` asks whether it is text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function panicText(payload: unknown): string { return payload instanceof Error ? payload.message : String(payload); }
/// ```
pub fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&str>() {
        return String::from(*message);
    }
    return String::from("a panic payload that is not text");
}

/// Position and construction controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_failure_tests.rs"]
mod tests;
