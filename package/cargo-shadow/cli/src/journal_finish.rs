//! What:
//! The error of finishing a transaction,
//! told by whether the outputs were already on disk.
//! Why:
//! Round 6 (fifth review,
//! 5c N1):
//! a failed durability barrier must stop the command,
//! while a journal that could not be removed
//! after its completion marker was made durable is only a warning.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class FinishError extends Error { outputsDurable: boolean }
//! ```

/// Imports `std::fmt` for this file.
use std::fmt;

/// Imports `std::io` for this file.
use std::io;

/// Why a transaction could not be finished.
#[derive(Debug)]
pub struct FinishError {
    /// Whether the lockfiles and manifests the run leaves were forced to disk before this failure.
    outputs_durable: bool,
    /// The failure.
    source: io::Error,
}

/// The methods of the type above.
impl FinishError {
    /// What:
    /// `pub fn before_durable(source: io::Error) -> FinishError` is a failure while forcing the outputs to disk.
    /// Why:
    /// The run's outputs may not survive a crash,
    /// so the command must not rely on them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static beforeDurable(source: Error): FinishError
    /// ```
    pub fn before_durable(source: io::Error) -> FinishError {
        return FinishError { outputs_durable: false, source };
    }

    /// What:
    /// `pub fn after_durable(source: io::Error) -> FinishError` is a failure after the outputs were forced to disk.
    /// Why:
    /// The journal stays and the next run settles it;
    /// nothing the run produced is at risk.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static afterDurable(source: Error): FinishError
    /// ```
    pub fn after_durable(source: io::Error) -> FinishError {
        return FinishError { outputs_durable: true, source };
    }

    /// What:
    /// `pub fn outputs_durable(&self) -> bool` says whether the outputs were forced to disk before the failure.
    /// Why:
    /// It decides between stopping the command and warning.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// outputsDurable(): boolean
    /// ```
    pub fn outputs_durable(&self) -> bool {
        return self.outputs_durable;
    }
}

/// How the error reads to a person.
impl fmt::Display for FinishError {
    /// What:
    /// `fn fmt(&self, formatter)` writes the failure with what it means for the run's outputs.
    /// Why:
    /// A person must be able to tell a lockfile that may be lost from a leftover journal.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.outputs_durable {
            return write!(formatter, "the transaction journal could not be completed ({}); the next run settles it", self.source);
        }
        return write!(
            formatter,
            "the lockfiles and manifests this run left could not be forced to disk ({}); the journal was kept and the next run settles it",
            self.source
        );
    }
}

/// Marks the error as a standard error.
impl std::error::Error for FinishError {
    /// What:
    /// `fn source(&self)` is the underlying I/O error.
    /// Why:
    /// Callers that walk the error chain reach the cause.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get cause(): Error
    /// ```
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        return Some(&self.source);
    }
}

/// What:
/// `pub(crate) fn settle_finish(outcome, finished, recovered, restore_warnings) -> Result<UpdateReport, UpdateError>` joins the
/// result of an update with the result of finishing its transaction.
/// Why:
/// A refusal stands and names a failed durability barrier too;
/// an accepted run whose outputs could not be forced to disk is a
/// stopped run,
/// never a clean pass;
/// only a journal that could not be removed after its durable marker is a warning (round 6,
/// 5c N1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function settleFinish(outcome: UpdateReport, finished: FinishError | undefined, recovered: string[], restoreWarnings: string[]): UpdateReport  // throws
/// ```
pub(crate) fn settle_finish(
    outcome: Result<crate::update::UpdateReport, crate::update::UpdateError>,
    finished: Result<(), FinishError>,
    recovered: Vec<String>,
    restore_warnings: Vec<String>,
) -> Result<crate::update::UpdateReport, crate::update::UpdateError> {
    return outcome.and_then(|mut report| {
        report.warnings.extend(recovered);
        report.warnings.extend(restore_warnings);
        return match finished {
            Ok(()) => Ok(report),
            Err(error) if !report.problems.is_empty() && !error.outputs_durable() => {
                report.problems.push(error.to_string());
                Ok(report)
            }
            Err(error) if !error.outputs_durable() => Err(crate::update::other_text(format!("{error}; run the command again"))),
            Err(error) => {
                report.warnings.push(error.to_string());
                Ok(report)
            }
        };
    });
}
