//! What:
//!  Read cargo's JSON records into the reports of the closure check.
//! Why:
//!  `cargo dylint -- --message-format json` prints one record per line.
//!  Only compiler messages of the closure lint and of unfulfilled lint expectations are findings;
//!  a compiler error is kept as the explanation of a failed run,
//! and everything else is ignored.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Report = { file: string; byteStart: number; byteEnd: number; line: number; message: string; help?: string };
//! function parseRecords(stdout: Uint8Array): { reports: Report[]; compilerError?: string };
//! ```

/// Import the JSON value model.
use serde_json::Value;
/// Import an ordered set that recognizes a report already seen.
use std::collections::BTreeSet;

/// What:
///  The lint codes that count as findings of the check.
/// Why:
///  The closure lint is the rule itself;
///  an expectation that suppressed nothing is a stale `expect` attribute of the same suppression scheme.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REPORTED_CODES = ['avoidable_closure', 'unfulfilled_lint_expectations'];
/// ```
pub const REPORTED_CODES: &[&str] = &["avoidable_closure", "unfulfilled_lint_expectations"];

/// What:
///  One finding of the compiler before it is mapped to a file of the run.
/// Why:
///  Positions stay as the compiler gave them until the source file is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Report = { file: string; byteStart: number; byteEnd: number; line: number; column: number; message: string; help?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    /// The file as the compiler printed it:
    ///  relative to the package directory for the package's own files.
    pub file: String,
    /// First byte of the primary span in the file.
    pub byte_start: usize,
    /// One past the last byte of the primary span.
    pub byte_end: usize,
    /// One-based line of the span's start,
    ///  used when the file cannot be read back.
    pub line: usize,
    /// One-based column of the span's start in characters,
    ///  used when the file cannot be read back.
    pub column: usize,
    /// The compiler's primary message.
    pub message: String,
    /// The first `help` child of the message,
    ///  when there is one.
    pub help: Option<String>,
}

/// What:
///  Everything read from one run's standard output.
/// Why:
///  A failed run is explained by its first compiler error,
///  so a package that does not compile is not mistaken for a clean one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Parsed = { reports: Report[]; compilerError?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Parsed {
    /// The findings,
    ///  once each even when several targets of the package compile the same file.
    pub reports: Vec<Report>,
    /// The rendered text of the first compiler error,
    ///  when a record had level `error`.
    pub compiler_error: Option<String>,
}

/// What:
///  Read an unsigned number from a JSON field.
/// Why:
///  A missing or negative field makes the whole span unusable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function count(value: unknown): number | undefined;
/// ```
fn count(value: &Value) -> Option<usize> {
    return match value.as_u64() {
        Some(number) => usize::try_from(number).ok(),
        None => None,
    };
}

/// What:
///  Build a report from the compiler message of one record,
///  or nothing when the code is not one of the check's.
/// Why:
///  The primary span carries the position;
///  a message without one cannot be placed and is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportOf(message: unknown): Report | undefined;
/// ```
fn report_of(message: &Value) -> Option<Report> {
    let code: &str = message["code"]["code"].as_str()?;
    if !REPORTED_CODES.contains(&code) {
        return None;
    }
    let spans: &Vec<Value> = message["spans"].as_array()?;
    let mut primary: Option<&Value> = None;
    for span in spans {
        if span["is_primary"].as_bool() == Some(true) {
            primary = Some(span);
            break;
        }
    }
    let span: &Value = primary?;
    let mut help: Option<String> = None;
    if let Some(children) = message["children"].as_array() {
        for child in children {
            if child["level"].as_str() == Some("help") {
                help = child["message"].as_str().map(String::from);
                break;
            }
        }
    }
    return Some(Report {
        file: String::from(span["file_name"].as_str()?),
        byte_start: count(&span["byte_start"])?,
        byte_end: count(&span["byte_end"])?,
        line: count(&span["line_start"])?,
        column: count(&span["column_start"])?,
        message: String::from(message["message"].as_str()?),
        help,
    });
}

/// What:
///  Read every record of a run's standard output.
/// Why:
///  A line that is not a JSON object is ignored,
///  because cargo may print other text;
///  the first error-level compiler message is kept for the failure explanation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRecords(stdout: Uint8Array): Parsed;
/// ```
pub fn parse_records(stdout: &[u8]) -> Parsed {
    let mut parsed: Parsed = Parsed::default();
    let mut seen: BTreeSet<(String, usize, usize, String)> = BTreeSet::new();
    for line in String::from_utf8_lossy(stdout).lines() {
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if record["reason"].as_str() != Some("compiler-message") {
            continue;
        }
        let message: &Value = &record["message"];
        if message["level"].as_str() == Some("error") && parsed.compiler_error.is_none() {
            let text: &str = match message["rendered"].as_str() {
                Some(rendered) => rendered,
                None => message["message"]
                    .as_str()
                    .unwrap_or("an error without text"),
            };
            parsed.compiler_error = Some(String::from(text.trim()));
        }
        let Some(report) = report_of(message) else {
            continue;
        };
        // One source line is reported once per target (library, binary) that compiles it.
        let identity: (String, usize, usize, String) = (
            report.file.clone(),
            report.byte_start,
            report.byte_end,
            report.message.clone(),
        );
        if seen.insert(identity) {
            parsed.reports.push(report);
        }
    }
    return parsed;
}

/// Record parsing controls stay outside release artifacts.
#[cfg(test)]
#[path = "closure_report_tests.rs"]
mod tests;
