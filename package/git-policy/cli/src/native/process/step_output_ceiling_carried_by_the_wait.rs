//! What:
//!  The ceiling a wait for a step's output carries,
//!  why that wait and not a watch ends the step at it,
//!  and the sentence the ending gives.
//! Why:
//!  A watch announces a wait and ends the child at its ceiling,
//!  but the thread it needs can be refused,
//!  and a watch that runs is not allowed to end every step it waits for;
//!  whatever it could not do,
//!  the wait for the output keeps the ceiling,
//!  so no step outlives it.
//!  `rust/max-lines` splits the read of a step's output by role,
//!  and the ceiling is the role that decides which enforcer ends the step.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const carried = carriedFor(ceiling, theWatchOfTheStep);
//! ```

use std::time::Duration;

/// What:
///  The step's own ceiling,
/// carried by the wait for its output,
/// and why no watch ended the step before it.
/// Why:
///   A watch announces a wait and ends the child at its ceiling,
/// but the thread it needs can be refused,
///  and a
/// watch that runs is not allowed to end every step it waits for.
/// Whatever it could not do,
/// the wait for the output keeps the
///  ceiling,
/// so no step outlives it
/// (the third review's finding 2).
///  A watch that can end the step has the ending,
/// and this wait is then the backstop behind it,
/// which is why the
///  moment it fires is its own and not the ceiling's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CarriedCeiling = { ceiling: Duration; backstop: Duration; why?: 'no-watcher' | 'the-watch-could-not-end-it' | 'the-query-is-bounded' };
/// ```
pub struct CarriedCeiling {
    /// The longest the step may run:
    /// the ceiling the watch gives it,
    /// and the number every sentence about the ending states.
    pub ceiling: Duration,
    /// How long this wait waits for the step before it ends the step itself:
    /// the ceiling,
    /// or later by the ending a watch that can end the step needs.
    pub backstop: Duration,
    /// Why this wait,
    /// and not a watch,
    /// ends the step at that ceiling,
    /// or nothing when a watch that can end it has the ending and this wait is only the backstop.
    pub why: Option<CeilingWhy>,
}

/// Why the wait for a step's output carries the step's own ceiling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CeilingWhy {
    /// No watch could run at all:
    /// the system refused the thread it needs.
    NoWatcher,
    /// A watch runs,
    /// and it could not end the step:
    /// nothing proved which process to stop,
    /// so this wait is what ends it at the ceiling.
    TheWatchCouldNotEndIt,
    /// Nothing watches this read and nothing else bounds it:
    /// it is a read-only query,
    /// and its ceiling is the longest one of those waits
    /// (`src/native/push/manual_push_range.rs`),
    /// or the whole bound a caller set on a fact it can do without
    /// (`src/native/repo_read/git_metadata.rs`).
    TheQueryIsBounded,
}

/// What:
///  Whether a watch runs for a step,
/// and whether it can end the step at its ceiling.
/// Why:
///   The watch and the wait for a step's output are given the same ceiling,
/// and which of them ends the step there
///  decides both how much later than the ceiling the wait may fire and what its answer is:
/// a watch whose pin proves the
///  process ends the step itself,
/// one that can prove nothing signals nothing and leaves the ending to the wait
///  at the ceiling itself,
/// and one that never started leaves it there too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheWatchOfAStep = 'not-watched' | 'watched-and-proven' | 'watched-and-unproven';
/// ```
pub enum TheWatchOfAStep {
    /// No watch could run at all:
    /// the system refused the thread one needs.
    NotWatched,
    /// A watch runs,
    /// and the descriptor it took when the child started proves which process to stop.
    WatchedAndProven,
    /// A watch runs,
    /// and nothing proved which process the child's number names,
    /// so it can signal nothing.
    WatchedAndUnproven,
}

/// What:
///  The ceiling a wait carries,
/// and why it is the wait that ends what it waits for.
/// Why:
///   A step with a ceiling may have no watch (a refused thread),
/// a watch that could not end it,
/// or no watch of
///  its own at all (a read-only query,
/// whose ceiling is the longest one of those waits);
/// or a watch that ends
///  the step itself,
/// which this wait then backs up rather than replaces.
/// The reason is the watch's to tell and is
///  known before the wait begins,
/// except for a watch that runs and can end the step:
/// there the wait is only the backstop,
/// later by
///  `THE_WATCH_HAS_ITS_ENDING`,
/// and the answer it gives is the ceiling's own.
/// `None` in,
///  `None` out:
/// a step with no ceiling is waited for as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// carriedFor(ceiling, theWatchOfTheStep)
/// ```
pub fn carried_for(
    ceiling: Option<Duration>,
    watch: TheWatchOfAStep,
) -> Option<CarriedCeiling> {
    let the_ceiling: Duration = ceiling?;
    return Some(match watch {
        TheWatchOfAStep::NotWatched => CarriedCeiling {
            ceiling: the_ceiling,
            backstop: the_ceiling,
            why: Some(CeilingWhy::NoWatcher),
        },
        // A watch that can prove nothing signals nothing, so this wait is what ends the step, at the ceiling itself.
        TheWatchOfAStep::WatchedAndUnproven => CarriedCeiling {
            ceiling: the_ceiling,
            backstop: the_ceiling,
            why: Some(CeilingWhy::TheWatchCouldNotEndIt),
        },
        // The watch ends the step, so this wait gives it the ending it needs before taking the ending away from it.
        TheWatchOfAStep::WatchedAndProven => CarriedCeiling {
            ceiling: the_ceiling,
            backstop: the_ceiling
                .saturating_add(crate::child_watch_running_task::THE_WATCH_HAS_ITS_ENDING),
            why: None,
        },
    });
}

/// The ceiling `limit` carried for the reason `why`,
/// by a wait that nothing else ends the step for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// carriedBy(limit, why)
/// ```
pub fn carried_by(
    limit: Duration,
    why: CeilingWhy,
) -> CarriedCeiling {
    return CarriedCeiling {
        ceiling: limit,
        backstop: limit,
        why: Some(why),
    };
}

impl CarriedCeiling {
    /// What:
    ///  The error of a step this wait ended at the ceiling it carried.
    /// Why:
    ///   Only this wait knows whether the kernel let the step go,
    /// and only it knows whether the ceiling was the ending at
    ///  all:
    /// a watch that could end the step may have ended it a moment after this wait fired,
    /// and the ceiling's
    ///  answer is then the same one the watch would have given,
    /// while a step no watch could end keeps the sentence that says so.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// (carried) => carried.why ? atTheCeiling(carried, leftRunning) : ceilingReached(carried.ceiling, leftRunning);
    /// ```
    pub(super) fn ending(
        &self,
        waiting_for: &str,
        left_running: bool,
    ) -> std::io::Error {
        let inner: Box<dyn std::error::Error + Send + Sync> = match self.why {
            Some(why) => Box::new(crate::child_waited_for_within_a_ceiling::AtTheCeiling {
                left_running,
                sentence: Some(waiting_carried_the_ceiling(
                    waiting_for,
                    self.ceiling,
                    why,
                    left_running,
                )),
            }),
            None => Box::new(crate::child_watch_step_bounds_and_errors::CeilingReached(
                crate::child_watch_step_bounds_and_errors::step_ended_at_its_ceiling_text(
                    waiting_for,
                    self.ceiling,
                    left_running,
                ),
            )),
        };
        return std::io::Error::new(std::io::ErrorKind::TimedOut, inner);
    }
}

/// What:
///  The sentence for a wait that carried the step's own ceiling because no watch could.
/// Why:
///   The watch is what ends a step,
/// and the system refused the thread that would have run it;
/// this wait then gives the step the
///  ceiling the watch would have given it and says why,
/// so nothing waits for a step for ever without a word
///  (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 8).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} ran for ${limit} seconds and was ${leftRunning ? 'left running' : 'ended'}; no watch could announce it: ${why}`
/// ```
fn waiting_carried_the_ceiling(
    waiting_for: &str,
    ceiling: Duration,
    why: CeilingWhy,
    left_running: bool,
) -> String {
    let reason: &str = match why {
        CeilingWhy::NoWatcher => {
            "Nothing could watch it: the system refused a thread for the watcher. Hint: check the process limit (`ulimit -u`) \
             and the control group's `pids.max`, free some, and run the command again."
        }
        CeilingWhy::TheWatchCouldNotEndIt => {
            "Its watch could not end it: nothing proved which process to stop. Hint: check with `ps` what the step waits for, \
             and run the command again."
        }
        CeilingWhy::TheQueryIsBounded => {
            "Such a query changes nothing of the repository. Hint: check with `ps` what that program waits for, and run the \
             command again."
        }
    };
    let ending: &str = if left_running {
        "and could not be ended, so it was left running, and the system collects it when its call returns"
    } else {
        "and was ended"
    };
    return format!(
        "{waiting_for} ran for {} seconds, the longest the wrapper lets one step run, {ending}. {reason}",
        ceiling.as_secs()
    );
}
