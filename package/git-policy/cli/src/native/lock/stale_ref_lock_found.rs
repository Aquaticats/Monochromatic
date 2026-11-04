//! What:
//!  The record of one judged reference lock, and the outcome of one clearing pass.
//! Why:
//!  The callers decide what to do from the outcome (run the command again only when a lock was cleared) and the notice is
//!  worded from the same record, so what was printed is what was decided.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Outcome = { cleared: Found[]; left: Found[]; displaced: string[] };
//! ```

use crate::stale_ref_lock_judge::Verdict;
use std::path::PathBuf;
use std::time::SystemTime;

/// What the pid file beside a lock says.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PidFile {
    /// No pid file, or one that holds no readable number yet.
    Absent,
    /// A pid file born before the lock: written by an earlier holder, so it names nobody who holds this lock.
    Older(i64),
    /// A pid file born at or after the lock, naming this process.
    Names(i64),
}

/// One judged reference lock.
#[derive(Clone, Debug)]
pub struct Found {
    /// The lock file.
    pub lock: PathBuf,
    /// What the judgement concluded.
    pub verdict: Verdict,
    /// The size of the lock when it was judged.
    pub size: u64,
    /// When the lock was born, where the file system keeps it.
    pub born: Option<SystemTime>,
    /// The pid file beside the lock.
    pub pid_file: PidFile,
    /// The files that go with the lock and are removed or named with it: its pid file and, for `packed-refs.lock`, the
    ///  staging file `packed-refs.new`. Only those that exist.
    pub companions: Vec<PathBuf>,
}

/// The result of one clearing pass.
#[derive(Clone, Debug, Default)]
pub struct Outcome {
    /// Locks that were removed.
    pub cleared: Vec<Found>,
    /// Locks that were left alone, with the reason each was not removed.
    pub left: Vec<Found>,
    /// Files that were set aside and could not be put back or removed because the path had been taken again.
    pub displaced: Vec<PathBuf>,
}

impl Outcome {
    /// Whether a lock was removed, which is the one reason to run the blocked command again.
    pub fn cleared_any(&self) -> bool {
        return !self.cleared.is_empty();
    }
}
