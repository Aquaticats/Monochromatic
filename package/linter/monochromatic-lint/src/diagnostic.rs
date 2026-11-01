//! What:
//!  The unified linter's JSONL finding model.
//! Why:
//!  Rules,
//!  processors and output share one owned representation,
//!  with fixes kept off the wire.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // JSONL carries diagnostic fields; Fix stays internal to the fixing pipeline.
//! ```

/// Import the fix type so each diagnostic can carry the automatic repair it proposes.
use crate::edits::Fix;
/// What:
///  Import Serde's field encoder and the internal atomic-fix model.
/// Why:
///  The existing JSON encoder handles quotes,
///  controls and Unicode at the output boundary.
use serde::Serialize;

/// What:
///  Severities that can appear in a reported finding.
/// Why:
///  Off is configuration,
///  not a third kind of diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Severity = 'warn' | 'error';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Reported without failing unless the warning limit is exceeded.
    Warn,
    /// Causes the lint run to fail.
    Error,
}

/// What:
///  One source range in the established diagnostic wire shape.
/// Why:
///  Byte addressing remains separate from human-facing line and column units.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Span = { offset: number; length: number; line: number; column: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Span {
    /// Zero-based UTF-8 byte offset in the host source.
    pub offset: usize,
    /// Length in UTF-8 bytes.
    pub length: usize,
    /// One-based host line number.
    pub line: usize,
    /// One-based column,
    ///  preserving the originating language's established units.
    pub column: usize,
}

/// What:
///  A labelled source range in the JSONL record.
/// Why:
///  Keeping the wrapper preserves the current consumer's labels[].span structure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Label = { span: Span };
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Label {
    /// Range highlighted by this label.
    pub span: Span,
}

/// What:
///  A finding plus its optional internal fix.
/// Why:
///  All strings are owned so findings can outlive borrowed parser nodes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Diagnostic = { message: string; code: string; severity: Severity;
///   filename: string; labels: Label[]; url?: string; help?: string; fix?: Fix };
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Diagnostic {
    /// Human-readable explanation,
    ///  encoded by Serde at rendering time.
    pub message: String,
    /// Descriptive rule ID,
    ///  or a core processing-failure code.
    pub code: String,
    /// Effective severity for this finding.
    pub severity: Severity,
    /// The established wire format requires an empty causes array.
    pub causes: [(); 0],
    /// Display path of the real host file,
    ///  never a virtual snippet name.
    pub filename: String,
    /// Mapped host-source ranges.
    pub labels: Vec<Label>,
    /// The established wire format requires an empty related array.
    pub related: [(); 0],
    /// Optional rule documentation link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Optional remediation detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    /// The fixing engine consumes this;
    ///  it is not part of the accepted JSONL shape.
    #[serde(skip)]
    pub fix: Option<Fix>,
    /// The driver distinguishes inability to check from an ordinary rule violation;
    ///  omitted from JSONL.
    #[serde(skip)]
    pub processing_failure: bool,
}

/// What:
///  Construct ordinary single-span rule findings.
/// Why:
///  Rules supply their behavior while shared wire defaults stay in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Diagnostic({ code, severity, message, filename, span });
/// ```
impl Diagnostic {
    /// Create a finding with no optional hints or fix.
    pub fn new(
        code: &str,
        severity: Severity,
        message: String,
        filename: String,
        span: Span,
    ) -> Diagnostic {
        return Diagnostic {
            message,
            code: String::from(code),
            severity,
            causes: [],
            filename,
            labels: vec![Label { span }],
            related: [],
            url: None,
            help: None,
            fix: None,
            processing_failure: false,
        };
    }
}

/// What:
///  Serialize one compact object per finding with an LF terminator.
/// Why:
///  Encoding failure is propagated;
///  it must not silently become an empty output line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function render(diagnostics: Diagnostic[]): string;
/// ```
pub fn render(diagnostics: &[Diagnostic]) -> Result<String, serde_json::Error> {
    let mut output = String::new();
    for diagnostic in diagnostics {
        let line = serde_json::to_string(diagnostic)?;
        output.push_str(line.as_str());
        output.push('\n');
    }
    return Ok(output);
}

/// Keep serialization boundary tests out of release artifacts.
#[cfg(test)]
#[path = "diagnostic_tests.rs"]
mod tests;
