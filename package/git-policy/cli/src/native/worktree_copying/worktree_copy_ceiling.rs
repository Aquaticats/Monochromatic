//! What:
//!  The longest one command spends finishing an ignored-state copy another command started,
//! and the words a copy stopped at that bound is reported with.
//! Why:
//!  A `git worktree add` killed while it copied leaves a worktree that still owes its ignored state
//! (`worktree_copy_owed_copy.rs`) or a journal that still holds it (`worktree_copy_recovery.rs`).
//! That work is finished only by a command run in the worktree it fills,
//! so a command never pays for another worktree's copy
//! (`doc/planning/cli-git-owed-copy-off-other-commands-path.md`).
//! The bound keeps even that command from spending an unbounded time on a checkout whose ignored
//! state is huge or whose filesystem stopped answering, because the owner's rule is that
//! "nothing should ever wait forever or indefinitely".
//! A copy is a transaction that can be cut at any instruction,
//! so a stop at the bound is safe and loses nothing it had written:
//! it ends as a copy that could not be finished,
//! which names the worktree and the two commands that settle it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ceiling = CopyCeiling.lasting(OWED_COPY_CEILING); if (ceiling.reached()) return stopped(ceiling);
//! ```

use std::time::{Duration, Instant};

/// What:
///  How long a command that finishes a copy it did not start may spend on it.
/// Why:
///  Six minutes is five times the longest copy measured at the scale the audit that found this wait used
///  (15,151 ignored entries, 150 MB: 73 s for the whole copy on this host at a load average of about 100),
///  rounded up to a whole minute,
///  which is the margin the wrapper's measured limits use.
/// A copy the destination's own command may not finish ends as a worktree named for removal and a fresh
/// `git worktree add`,
/// which copies the same state itself and holds no ceiling.
pub const OWED_COPY_CEILING: Duration = Duration::from_secs(360);

/// What:
///  How long a copy may run,
/// counted from the moment the command began it.
/// Why:
///   A copy the destination's own command finishes is bounded;
/// the copy a creating command makes for the worktree it just registered is not,
/// because the owner allows that add its own time ("60s is fine for adding worktrees").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopyCeiling = { deadline?: number; longest?: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CopyCeiling {
    /// The instant the copy must stop at,
    /// absent for a copy no ceiling holds.
    deadline: Option<Instant>,
    /// How long the copy may run,
    /// absent for a copy no ceiling holds.
    longest: Option<Duration>,
}

impl CopyCeiling {
    /// A copy no bound holds: the creating command's own.
    pub const fn unbounded() -> CopyCeiling {
        return CopyCeiling { deadline: None, longest: None };
    }

    /// A copy that must stop `longest` from now.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static lasting(longest: Duration): CopyCeiling { return { deadline: performance.now() + longest, longest }; }
    /// ```
    pub fn lasting(longest: Duration) -> CopyCeiling {
        #[cfg(test)]
        let longest: Duration = SHORTER_FOR_THIS_THREAD.with(std::cell::Cell::get).unwrap_or(longest);
        return CopyCeiling { deadline: Some(Instant::now() + longest), longest: Some(longest) };
    }

    /// Whether the copy has run past its bound,
    /// asked before each entry the copy writes and between its phases.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reached(): boolean { return this.deadline !== undefined && performance.now() >= this.deadline; }
    /// ```
    pub fn reached(&self) -> bool {
        return self.deadline.is_some_and(|deadline| { return Instant::now() >= deadline; });
    }

    /// What a stopped copy's failure says.
    /// Why:
    ///  The person reads why a worktree they did not touch is still missing files,
    /// and that the bound,
    /// not a fault in their checkout,
    /// ended it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stoppedSentence(): string { return `... did not finish within the ${seconds}-second ceiling ...`; }
    /// ```
    pub fn stopped_sentence(&self) -> String {
        let seconds: u64 = self.longest.map_or(0, |longest| { return longest.as_secs(); });
        return format!(
            "monochromatic-git-policy-cli: the ignored-state copy did not finish within the {seconds}-second ceiling one command holds for a copy it did not start, so it was stopped and its worktree's ignored files are still missing."
        );
    }

    /// The failure of a copy that reached its bound.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stoppedError(): WorktreeCopyError { return new WorktreeCopyError(this.stoppedSentence()); }
    /// ```
    pub fn stopped_error(&self) -> crate::worktree_copy_error::WorktreeCopyError {
        return crate::worktree_copy_error::failure_with_cause(self.stopped_sentence().as_str(), None);
    }
}

// A bound a control shortens for the thread that runs it,
// so the stop is reached inside one edit rather than after six minutes.
#[cfg(test)]
thread_local! {
    /// The shortened bound of the copy this thread runs.
    static SHORTER_FOR_THIS_THREAD: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}

/// What:
///  Runs the returned value's life with every copy this thread starts bounded by `longest` instead of
/// `OWED_COPY_CEILING`.
/// Why:
///   The stop has to be reached inside a unit test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using shortened = aShorterCeilingForThisThread(1); // every copy of this thread stops at once
/// ```
#[cfg(test)]
pub fn a_shorter_ceiling_for_this_thread(longest: Duration) -> ShorterCeilingForTheControl {
    SHORTER_FOR_THIS_THREAD.with(|cell| { return cell.set(Some(longest)); });
    return ShorterCeilingForTheControl;
}

/// Clears the control's shortened bound when it is dropped.
#[cfg(test)]
pub struct ShorterCeilingForTheControl;

/// Clears the shortened bound this thread's copies were given.
#[cfg(test)]
impl Drop for ShorterCeilingForTheControl {
    fn drop(&mut self) {
        SHORTER_FOR_THIS_THREAD.with(|cell| { return cell.set(None); });
    }
}

/// Controls of the bound stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_ceiling_tests.rs"]
mod tests;
