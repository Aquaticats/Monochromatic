//! What:
//!  JSONL stream routing and exit status for completed lint findings.
//! Why:
//!  Quiet/silent output must not change failure accounting,
//!  and stdin fixing has its established source-output exception.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Count all findings before filtering display, then route fixed source and JSONL to their designated streams.
//! ```

/// Import the existing JSONL encoder and typed severity.
use crate::diagnostic::{Diagnostic, Severity, render};

/// Output controls independent of file discovery and rule execution.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OutputOptions {
    /// Hide warnings in output without removing them from warning-limit accounting.
    pub quiet: bool,
    /// Hide all diagnostic records without changing exit status.
    pub silent: bool,
    /// Optional limit on warnings,
    ///  including ones hidden by quiet/silent.
    pub max_warnings: Option<usize>,
}

/// Exact output bytes and process status returned to the executable boundary.
#[derive(Debug, Eq, PartialEq)]
pub struct RunOutput {
    /// Ordinary JSONL,
    ///  or fixed input text for stdin-fix mode.
    pub stdout: String,
    /// JSONL findings in stdin-fix mode;
    ///  ordinary finding output leaves this empty.
    pub stderr: String,
    /// 0 for an accepted run,
    ///  1 for failed policy findings,
    ///  2 for incomplete processing.
    pub exit_code: u8,
}

/// What:
///  The exit status a completed run earns:
///  1 for an error finding or more warnings than the limit,
///  2 for a processing failure,
///  otherwise 0.
/// Why:
///  The executable and an embedding caller classify a run by this one function,
///  so they cannot disagree about whether a run succeeded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exitStatus(findings: Diagnostic[], maxWarnings?: number): 0 | 1 | 2;
/// ```
pub fn exit_status(findings: &[Diagnostic], max_warnings: Option<usize>) -> u8 {
    let mut warnings: usize = 0;
    let mut errors: bool = false;
    let mut processing_failure: bool = false;
    for finding in findings {
        if finding.severity == Severity::Warn {
            warnings += 1;
        } else {
            errors = true;
        }
        processing_failure |= finding.processing_failure;
    }
    let mut exit_code: u8 = 0;
    if errors {
        exit_code = 1;
    }
    if let Some(limit) = max_warnings
        && warnings > limit
    {
        exit_code = 1;
    }
    if processing_failure {
        exit_code = 2;
    }
    return exit_code;
}

/// Route a completed run without manually escaping JSON or mixing source bytes with diagnostic records.
pub fn run_output(
    findings: &[Diagnostic],
    fixed_stdin: Option<&str>,
    options: OutputOptions,
) -> Result<RunOutput, serde_json::Error> {
    let mut visible: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for finding in findings {
        if options.silent || (options.quiet && finding.severity == Severity::Warn) {
            continue;
        }
        visible.push(finding.clone());
    }
    let exit_code: u8 = exit_status(findings, options.max_warnings);
    let diagnostics: String = render(&visible)?;
    if let Some(source) = fixed_stdin {
        return Ok(RunOutput {
            stdout: String::from(source),
            stderr: diagnostics,
            exit_code,
        });
    }
    return Ok(RunOutput {
        stdout: diagnostics,
        stderr: String::new(),
        exit_code,
    });
}

/// Stream and status controls are not release code.
#[cfg(test)]
#[path = "run_output_tests.rs"]
mod tests;
