//! What:
//!  The failure type of the subvolume migration and the test hook that simulates a kill.
//! Why:
//!  Every step names what it did to which path and why it stopped,
//! so the report can tell the person the cause,
//! the
//!  state the journal holds,
//! and the way forward.
//! The interrupt hook stops a run at a named point exactly as a kill
//!  would (all state is on disk or in the journal;
//! nothing else survives a kill),
//! so the resume controls can stop at
//!  every journal step without killing the test process.
//! Release code passes `Interrupt::never()`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class MigrateError extends Error {}
//! const interrupt = { point: undefined as string | undefined, after: 0 }; // tests only
//! ```

use crate::fs_primitive_error::PrimitiveError;
use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// What:
///  Why a migration step stopped.
///  `String` is owned text;
/// `PathBuf` is an owned path.
/// Why:
///   Each variant has its own wording and its own remedy in the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MigrateError = { kind: 'io'; ... } | { kind: 'primitive'; ... } | { kind: 'unreadable-directory'; ... } | ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MigrateError {
    /// An ordinary file operation failed.
    Io {
        /// What was being done,
        /// in words.
        action: String,
        /// The path involved.
        path: PathBuf,
        /// The operating system's words.
        reason: String,
    },
    /// A kernel primitive failed (ioctl,
    /// clone,
    /// exchange).
    Primitive(PrimitiveError),
    /// A directory this user cannot list or search;
    /// the migration stops instead of leaving its subtree out.
    UnreadableDirectory {
        /// The directory.
        path: PathBuf,
        /// The operating system's words.
        reason: String,
    },
    /// The journal file is unreadable or has an unknown shape.
    Journal(String),
    /// The run stopped at the named test point.
    Interrupted(String),
}

impl MigrateError {
    /// What:
    ///  An I/O failure with the action and path,
    /// from a `std::io::Error`.
    /// Why:
    ///   One constructor keeps every message in the same shape.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const io = (action: string, path: string, error: Error) => new MigrateError(...);
    /// ```
    pub fn io(action: &str, path: &std::path::Path, error: &std::io::Error) -> MigrateError {
        return MigrateError::Io { action: action.to_string(), path: path.to_path_buf(), reason: error.to_string() };
    }

    /// The success of an input and output result as it is,
    /// its failure as `MigrateError::io` for `action` on `path`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const value = ioResult(await attempt(), action, path); // throws MigrateError(io) on failure
    /// ```
    pub fn from_io_result<T>(result: std::io::Result<T>, action: &str, path: &std::path::Path) -> Result<T, MigrateError> {
        return match result {
            Ok(value) => Ok(value),
            Err(error) => Err(MigrateError::io(action, path, &error)),
        };
    }
}

/// Prints the cause in plain words.
impl fmt::Display for MigrateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrateError::Io { action, path, reason } => return write!(formatter, "cannot {action} {}: {reason}", path.display()),
            MigrateError::Primitive(error) => return write!(formatter, "{error}"),
            MigrateError::UnreadableDirectory { path, reason } => {
                return write!(
                    formatter,
                    "the directory {} cannot be listed or searched by this user ({reason}); copying around it would leave its content out of the new checkout",
                    path.display()
                );
            }
            MigrateError::Journal(message) => return write!(formatter, "{message}"),
            MigrateError::Interrupted(point) => return write!(formatter, "stopped at the test point {point}"),
        }
    }
}

/// A migration error is an error value.
impl std::error::Error for MigrateError {}

/// A primitive failure is a migration failure.
impl From<PrimitiveError> for MigrateError {
    fn from(error: PrimitiveError) -> MigrateError {
        return MigrateError::Primitive(error);
    }
}

/// What:
///  The simulated kill:
/// stop the run after a named point has been reached a number of times.
///  `AtomicU64` is a number that can change behind a shared borrow from several threads at once (TS has no distinction;
///  everything mutates).
/// Why:
///   A control sets the point,
/// runs,
/// sees the `Interrupted` error,
/// and runs again with `Interrupt::never()`.
/// Worktrees converted
///  side by side share one hook,
/// so the hits are counted across all of them and exactly one hit is the chosen one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Interrupt { check(point: string): void { if (point === this.point && ++this.hits > this.after) throw ...; } }
/// ```
#[derive(Debug)]
pub struct Interrupt {
    /// The point name and how many earlier hits to let pass.
    target: Option<(String, u64)>,
    /// Hits seen so far.
    hits: AtomicU64,
}

impl Interrupt {
    /// A run that is never stopped:
    /// what the release code uses.
    pub fn never() -> Interrupt {
        return Interrupt { target: None, hits: AtomicU64::new(0) };
    }

    /// A run that stops at the `after`-th (counting from 0) hit of `point`.
    pub fn at(point: &str, after: u64) -> Interrupt {
        return Interrupt { target: Some((point.to_string(), after)), hits: AtomicU64::new(0) };
    }

    /// What:
    ///  Pass the named point;
    /// fail with `Interrupted` when it is the chosen hit.
    /// Why:
    ///   The steps call it between their durable actions.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// interrupt.check('after-copy');
    /// ```
    pub fn check(&self, point: &str) -> Result<(), MigrateError> {
        let Some((name, after)) = &self.target else { return Ok(()) };
        if name != point {
            return Ok(());
        }
        let seen: u64 = self.hits.fetch_add(1, Ordering::SeqCst);
        if seen == *after {
            return Err(MigrateError::Interrupted(point.to_string()));
        }
        return Ok(());
    }
}
