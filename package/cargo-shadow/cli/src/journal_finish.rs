//! What:
//! The error of finishing a transaction,
//! told by the phase that failed.
//! Why:
//! Round 7 (sixth review 6b2 5, 6c R1):
//! finishing has three phases that fail differently.
//! Forcing the outputs to disk and publishing the durable completion marker stop the command
//! (the next recovery would otherwise find an active journal and judge the run again);
//! only the removal of a journal whose completion marker is durable is a warning.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class FinishError extends Error { phase: 'outputs' | 'completion' | 'cleanup' }
//! ```

/// Imports `std::fmt` for this file.
use std::fmt;

/// Imports `std::io` for this file.
use std::io;

/// The phase of finishing in which the failure happened.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    /// Forcing the lockfiles and manifests the run leaves to disk.
    Outputs,
    /// Publishing the durable completion marker (the rename to the `.done` name and the sync of the directory that lists it).
    Completion,
    /// Removing a journal whose completion marker is durable.
    Cleanup,
}

/// Why a transaction could not be finished.
#[derive(Debug)]
pub struct FinishError {
    /// The phase that failed.
    phase: Phase,
    /// The failure.
    source: io::Error,
}

/// The methods of the type above.
impl FinishError {
    /// What:
    /// `pub fn outputs(source: io::Error) -> FinishError` is a failure while forcing the outputs to disk.
    /// Why:
    /// The run's outputs may not survive a crash,
    /// so the command must not rely on them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static outputs(source: Error): FinishError
    /// ```
    pub fn outputs(source: io::Error) -> FinishError {
        return FinishError { phase: Phase::Outputs, source };
    }

    /// What:
    /// `pub fn completion(source: io::Error) -> FinishError` is a failure to publish the durable completion marker.
    /// Why:
    /// The outputs are on disk, but nothing durable says this run is finished:
    /// a later recovery would see an active journal and could judge the run again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static completion(source: Error): FinishError
    /// ```
    pub fn completion(source: io::Error) -> FinishError {
        return FinishError { phase: Phase::Completion, source };
    }

    /// What:
    /// `pub fn cleanup(source: io::Error) -> FinishError` is a failure to remove a journal whose completion marker is durable.
    /// Why:
    /// The journal is disposable and the next run removes it;
    /// nothing the run produced is at risk.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static cleanup(source: Error): FinishError
    /// ```
    pub fn cleanup(source: io::Error) -> FinishError {
        return FinishError { phase: Phase::Cleanup, source };
    }

    /// What:
    /// `pub fn cleanup_only(&self) -> bool` says whether only the removal of a disposable journal failed.
    /// Why:
    /// It decides between stopping the command and warning.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// cleanupOnly(): boolean
    /// ```
    pub fn cleanup_only(&self) -> bool {
        return self.phase == Phase::Cleanup;
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
        return match self.phase {
            Phase::Cleanup => write!(formatter, "the transaction journal could not be removed ({}); the next run removes it", self.source),
            Phase::Completion => write!(
                formatter,
                "the completion of the transaction could not be made durable ({}); the journal was kept and the next run settles it",
                self.source
            ),
            Phase::Outputs => write!(
                formatter,
                "the lockfiles and manifests this run left could not be forced to disk ({}); the journal was kept and the next run settles it",
                self.source
            ),
        };
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
    // A failure of a mandatory phase never lets an earlier error stand alone: both are reported (round 7, 6c R1).
    let standing = match (outcome, &finished) {
        (Err(original), Err(error)) if !error.cleanup_only() => {
            return Err(crate::update::other_text(format!("{original}\n{error}; run the command again")));
        }
        (kept, _) => kept,
    };
    return standing.and_then(|mut report| {
        report.warnings.extend(recovered);
        report.warnings.extend(restore_warnings);
        return match finished {
            Ok(()) => Ok(report),
            Err(error) if !report.problems.is_empty() && !error.cleanup_only() => {
                report.problems.push(error.to_string());
                Ok(report)
            }
            Err(error) if !error.cleanup_only() => Err(crate::update::other_text(format!("{error}; run the command again"))),
            Err(error) => {
                report.warnings.push(error.to_string());
                Ok(report)
            }
        };
    });
}
