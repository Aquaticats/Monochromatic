//! What:
//!  The clock of the run:
//! it compares each look with the one before it,
//! and answers when the run has shown nothing for its window.
//! Why:
//!   Nothing is ended here;
//! the caller decides what an ending does,
//! which is what lets the controls drive the judgment with made-up looks and times instead of waiting a window out.
//! A task whose own figures grew is work,
//! a task a look finds where the look before it found none is a task that started,
//! and a look that read nothing is no sign of work either way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // class Judge { constructor(windowMs) {} observe(elapsedMs, look?): Ending | undefined }
//! ```

/// Why an ending says what it does.
use super::ending::Ending;
/// The look the judge compares,
/// and the figures of one task it holds.
use super::look_figures::{
    Look,
    TaskFigures,
};
/// Time spans.
use std::time::Duration;

/// What:
///  The clock of the run:
/// it compares each look with the one before it and says when the run has shown nothing for
///  its window.
/// Why:
///   Nothing is ended here;
/// the caller decides what an ending does,
/// which is what lets the controls drive the
///  judgment with made-up looks and times instead of waiting a window out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Judge { constructor(windowMs) {} observe(elapsedMs, look?: Look): Ending | undefined }
/// ```
#[derive(Debug)]
pub(crate) struct Judge {
    /// How long the run may show nothing before it counts as stuck.
    window: Duration,
    /// When the last look that showed something was taken.
    last_change: Duration,
    /// When the last look that could read this system's counters was taken,
    /// or nothing where no look has.
    last_read: Option<Duration>,
    /// The look before the current one,
    /// to compare with.
    previous: Option<Look>,
}

impl Judge {
    /// A judge for a window,
    /// which has seen nothing yet.
    pub(crate) fn new(window: Duration) -> Judge {
        return Judge {
            window,
            last_change: Duration::ZERO,
            last_read: None,
            previous: None,
        };
    }

    /// Judge one look taken `elapsed` after the run started;
    /// `Some` is the reason the run ends.
    ///  A look that could not be read is no sign of work and no proof that the run stopped:
    /// the clock keeps
    ///  running through it,
    /// so a run whose counters never come back ends,
    /// and a later look that reads the run counts again.
    pub(crate) fn observe(
        &mut self,
        elapsed: Duration,
        look: Option<Look>,
    ) -> Option<Ending> {
        let Some(current): Option<Look> = look else {
            return self.ending_without_a_look(elapsed);
        };
        self.last_read = Some(elapsed);
        // A task's own figures only grow, so a task that shows more than it did before is at work,
        //  where a task that ended only leaves the totals smaller and that is not the run doing anything.
        let moved: bool = self.previous.as_ref().is_none_or(|before: &Look| {
            return current.tasks.iter().any(|now: &TaskFigures| {
                return match before.tasks.iter().find(|then: &&TaskFigures| {
                    return then.task == now.task;
                }) {
                    Some(then) => now.bytes > then.bytes || now.ticks > then.ticks,
                    // A task this look sees and the look before it did not is a task that started.
                    None => true,
                };
            });
        });
        if moved || current.working {
            self.last_change = elapsed;
        }
        self.previous = Some(current);
        return self.ending_after_the_quiet_stretch(elapsed);
    }

    /// What:
    ///  The ending of a look that read nothing.
    /// Why:
    ///   Nothing was read,
    /// so the look is no sign of work and no proof that the run stopped;
    /// the clock keeps
    ///  running,
    /// and a system whose counters never come back ends as unreadable rather than as quiet,
    /// because
    ///  quiet would claim signs this run never showed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const endingWithoutALook = (elapsedMs) => lastRead === undefined
    ///     ? (elapsedMs >= windowMs ? unmeasurable(elapsedMs) : undefined)
    ///     : (elapsedMs - lastRead >= windowMs ? unreadable(elapsedMs - lastRead) : quiet(elapsedMs));
    /// ```
    fn ending_without_a_look(
        &self,
        elapsed: Duration,
    ) -> Option<Ending> {
        let Some(last_read): Option<Duration> = self.last_read else {
            // No look has read this system's counters yet, so until one does the wall clock is the bound.
            return if elapsed >= self.window {
                Some(Ending::Unmeasurable(elapsed))
            } else {
                None
            };
        };
        let unread: Duration = elapsed.saturating_sub(last_read);
        if unread >= self.window {
            return Some(Ending::Unreadable(unread));
        }
        return self.ending_after_the_quiet_stretch(elapsed);
    }

    /// What:
    ///  The ending of a run that has shown nothing since `last_change`,
    /// or nothing before its window
    ///  has passed.
    /// Why:
    ///   Every ending of a run some look could read goes through this one comparison of the clock,
    /// so a look
    ///  that read nothing is measured on the same clock as one that read a still run.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const quiet = elapsedMs - lastChangeMs; return quiet >= windowMs ? { kind: 'quiet', quietMs: quiet } : undefined;
    /// ```
    fn ending_after_the_quiet_stretch(
        &self,
        elapsed: Duration,
    ) -> Option<Ending> {
        let quiet: Duration = elapsed.saturating_sub(self.last_change);
        return if quiet >= self.window {
            Some(Ending::Quiet(quiet))
        } else {
            None
        };
    }
}
