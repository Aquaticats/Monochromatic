//! What:
//!  Bounded,
//!  cycle-aware source fixing with a fresh check after each edit pass.
//! Why:
//!  Every pass uses current byte offsets and no filesystem write occurs before the complete result is accepted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Check, apply atomic fix groups, stop on unchanged/circular output or ten changes, then check once more.
//! ```

/// Import the established diagnostics and grouped edit boundary.
use crate::diagnostic::Diagnostic;
/// Import the fix applier and its error type that each repair pass depends on.
use crate::edits::{Fix, FixError, apply_fixes};

/// Maximum changed-source passes required by the unified-linter contract.
const MAX_PASSES: usize = 10;

/// A language/processor session that reparses the exact supplied snapshot on each call.
pub trait SourceChecker {
    /// Return current findings or an explicit inability to check;
    ///  caller-owned source remains immutable.
    fn check(&mut self, source: &str) -> Result<Vec<Diagnostic>, FixError>;
}

/// Normal reasons to stop applying fixes;
///  processing failures use the error channel instead.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixStop {
    /// No accepted edit changed the source.
    Unchanged,
    /// The newest snapshot repeats an earlier one,
    ///  ending a circular fix sequence.
    Cycle,
    /// The allowed edit-pass count was consumed.
    Limit,
}

/// A complete in-memory result ready for the caller's later atomic write.
#[derive(Debug)]
pub struct FixedSource {
    /// Accepted final bytes;
    ///  no filesystem operation is performed by the loop.
    pub source: String,
    /// Findings from a fresh final check,
    ///  not stale findings from the preceding edit pass.
    pub diagnostics: Vec<Diagnostic>,
    /// Number of passes that actually changed the source.
    pub changed_passes: usize,
    /// Whether the loop reached a stable source,
    ///  a cycle,
    ///  or its fixed budget.
    pub stop: FixStop,
}

/// Refuse to publish provisional edits if a checker explicitly reports incomplete processing.
fn verify_processing(findings: &[Diagnostic]) -> Result<(), FixError> {
    for finding in findings {
        if finding.processing_failure {
            return Err(FixError {
                message: format!(
                    "Cannot safely fix {} because {} reported incomplete processing: {}. Original file bytes have not been changed.",
                    finding.filename, finding.code, finding.message,
                ),
            });
        }
    }
    return Ok(());
}

/// What:
///  Run the agreed bounded fixpoint algorithm against one owned checker session.
/// Why:
///  Exact snapshot equality detects cycles without hashes or collision assumptions;
///  history is capped by ten passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fixSource(source, checker): FixedSource;
/// ```
pub fn fix_source<Checker: SourceChecker>(
    source: &str,
    checker: &mut Checker,
) -> Result<FixedSource, FixError> {
    // Own each provisional snapshot independently of the caller's unchanged source.
    let mut current: String = String::from(source);
    let mut history: Vec<String> = Vec::<String>::new();
    let mut changed_passes: usize = 0;
    let mut stop: FixStop = FixStop::Limit;
    for _ in 0..MAX_PASSES {
        // Previous snapshots stay exact, not normalized text, so the cycle decision is byte-preserving.
        history.push(current.clone());
        let findings: Vec<Diagnostic> = checker.check(current.as_str())?;
        verify_processing(findings.as_slice())?;
        let mut fixes: Vec<Fix> = Vec::<Fix>::new();
        for finding in findings {
            if let Some(fix) = finding.fix {
                fixes.push(fix);
            }
        }
        let next: String = apply_fixes(current.as_str(), fixes.as_slice())?.source;
        if next == current {
            stop = FixStop::Unchanged;
            break;
        }
        changed_passes += 1;
        let circular: bool = history.contains(&next);
        current = next;
        if circular {
            // Keep the repeated snapshot: starting another fix invocation from it returns the same cycle anchor.
            stop = FixStop::Cycle;
            break;
        }
    }
    // Always perform the specified final check, even after an unchanged pass.
    let diagnostics: Vec<Diagnostic> = checker.check(current.as_str())?;
    verify_processing(diagnostics.as_slice())?;
    return Ok(FixedSource {
        source: current,
        diagnostics,
        changed_passes,
        stop,
    });
}

/// Fixture checkers inspect every submitted source version and every normal/error stop path.
#[cfg(test)]
#[path = "fix_loop_tests.rs"]
mod tests;
