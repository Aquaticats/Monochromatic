//! What:
//! The error of finishing a transaction, told by whether the outputs were already on disk.
//! Why:
//! Round 6 (fifth review, 5c N1): a failed durability barrier must stop the command, while a journal that could not be removed
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
    /// The run's outputs may not survive a crash, so the command must not rely on them.
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
    /// The journal stays and the next run settles it; nothing the run produced is at risk.
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
