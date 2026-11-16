//! What:
//!  The one allowance of time the wrapper's own steps of one `git worktree add` share,
//!  the clamping of each of those steps into what is
//!  left of it,
//!  and the words an add that ends at it says.
//! Why:
//!  Every wait of the wrapper has a bound of its own
//! (`service/wait_ceilings.rs`),
//!  but a step whose every try is bounded
//!  and whose number of tries is not waits without a bound:
//!  on 2026-11-13 a pipe stood at a repository's
//!  `.git/info/exclude`,
//!  every Git child of an add stalled in the same read,
//!  each was ended at its own window
//!  and another followed,
//!  and the add had not answered after 240 seconds.
//!  A mount that stopped answering is the same shape on
//!  this machine.
//!  The owner's rule is that nothing should ever wait forever or indefinitely,
//!  so the wrapper's own work for
//!  one add has one allowance:
//!  every step it starts is ended by what is left of the allowance,
//!  no step starts once the
//!  allowance is spent,
//!  and an add that has not made its worktree by then stops,
//!  says what it was waiting
//!  for and how long,
//!  and leaves nothing of a half-made worktree behind.
//!  The person's own forwarded command is not part of this:
//!  native Git waits as long as its own work takes,
//!  so the clock stops while that command runs
//!  (`SPEC.md`,
//! "Every Git step the ladder starts itself").
//!  Two things after the worktree is whole keep their own bounds and not this one:
//!  a copy the allowance stops after
//!  real Git made the worktree is a copy failure the next command settles,
//!  as any other failed copy is,
//!  and the
//!  package repair after it keeps one ceiling per tool
//!  (`REPAIR_TOOL_CEILING`);
//!  neither can leave a worktree half made.
//!  The allowance is one deadline and not a count of tries,
//!  because a count of
//!  tries cannot bound the time a copy of a large tree takes,
//!  and the time is what a person waits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! using clock = theAddsOwnWorkBegan();                 // the adding command's own steps
//! using person = thePersonsCommandBegan();             // the clock stops while Git runs
//! const left = whatIsLeft();                           // undefined when no add runs here
//! beforeAStep(subject);                                // throws once the allowance is spent
//! const [terms, bound] = withinTheAllowance(terms, bound, subject); // each step ends by what is left
//! ```

use crate::child_watch::{StepBound, WatchTerms};
use crate::diagnostic_log::debug;
use crate::push_deadline_and_patience_settings::positive_seconds;
use crate::wait_ceilings::WORKTREE_ADD_OWN_WORK_CEILING;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeAddOwnWorkDeadline";

/// The variable that sets the allowance of one add's own steps,
/// in whole seconds,
/// for a person who has to add a worktree over a filesystem slower than any this
///  repository has measured.
pub const ALLOWANCE_VARIABLE: &str = "CLI_GIT_ADD_OWN_WORK_SECONDS";

/// What one add's own work is spending,
/// and the step it is spending it on.
/// Why:
///  The ceiling and the time are read together at each step,
///  and the step that spent the last of it is what the
///  add's ending names,
///  so all three are one value that one lock guards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Spending = { ceiling: Duration; spent: Duration; runningSince?: Instant; waitingFor?: string };
/// ```
struct Spending {
    /// The allowance this clock was armed with,
    /// read once at the start,
    /// so a variable changed in the middle of an add cannot move it.
    ceiling: Duration,
    /// What the add's own work has spent so far,
    /// over every stretch the clock ran.
    spent: Duration,
    /// When the stretch that is running began,
    /// or nothing while the clock is stopped for the person's own command.
    running_since: Option<Instant>,
    /// The step the wrapper was last waiting for,
    /// as the notices and the ending name it.
    waiting_for: Option<String>,
}

/// The one add a process runs,
/// and what its own work has spent.
/// Why:
///  The steps of an add run on the command's own thread,
/// the blocking pool and the runtime,
/// so the state is
///  the process's and not a thread's,
///  and one process runs one add at a time
///  (`add_standing_notice.rs` keeps the
///  same one-add rule for its own record).
static SPENDING: Mutex<Option<Spending>> = Mutex::new(None);

/// Whether an add's own clock runs in this process.
/// Why:
///  Every Git step of every command asks whether the allowance applies,
///  and a commit starts dozens of steps:
///  this flag answers no without taking the lock,
///  so a command that runs no add pays one load per step and nothing else.
static ARMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What:
///  Whether the wrapper's own steps of an add run in this process,
///  and so whether the allowance clamps a step.
/// Why:
///   A caller asks this before it builds the words of a step,
///  so a command that runs no add formats nothing for the allowance.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runs = aClockRunsHere();
/// ```
pub fn a_clock_runs_here() -> bool {
    return ARMED.load(std::sync::atomic::Ordering::Acquire);
}

/// The allowance the add of this process runs under:
/// the variable's value when it names a positive whole number of seconds,
///  and the shipped ceiling otherwise.
/// Why:
///  The shipped number is sized from this repository's adds
///  (`WORKTREE_ADD_OWN_WORK_CEILING`),
///  and a person adding a worktree over a
///  slower filesystem than any measured here raises it for that command rather than for all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const allowance = positiveSeconds(process.env.CLI_GIT_ADD_OWN_WORK_SECONDS) ?? WORKTREE_ADD_OWN_WORK_CEILING;
/// ```
fn the_allowance() -> Duration {
    let named: Option<Duration> = std::env::var_os(ALLOWANCE_VARIABLE)
        .and_then(|value| { return positive_seconds(ALLOWANCE_VARIABLE, value.as_os_str()); });
    return named.unwrap_or(WORKTREE_ADD_OWN_WORK_CEILING);
}

/// What the clock of `spending` reads now:
/// the stretches it ran,
/// and the one it is in.
/// Why:
///  The pause for the person's command is the only stop,
///  so the spent time is what was recorded plus the stretch that is
///  running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spentNow = (s: Spending) => s.spent + (s.runningSince ? now - s.runningSince : 0);
/// ```
fn spent_now(spending: &Spending) -> Duration {
    let running: Duration = spending
        .running_since
        .map_or(Duration::ZERO, |since| { return since.elapsed(); });
    return spending.spent.saturating_add(running);
}

/// Take the state for one look,
/// or nothing when no add is running here.
/// Why:
///  Every face of this module starts at the same lock,
///  and a process that runs no add pays one uncontended lock and no more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spending = trySpending(); // undefined when the lock or the add is absent
/// ```
fn look() -> Option<std::sync::MutexGuard<'static, Option<Spending>>> {
    return SPENDING.lock().ok();
}

/// What:
///  The clock of one add's own work,
///  running from here until the value goes out of scope.
/// Why:
///   An add's own work is the ladder and the copy and nothing else the process does,
///  so one value arms it at the
///  start of a creating command and disarms it when its worktree is settled.
///  A second add in one process would arm over this one:
///  one process runs one forwarded
///  command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using clock = theAddsOwnWorkBegan();
/// ```
#[must_use = "the clock stops when the value is dropped, so dropping it at once bounds nothing"]
pub struct TheAddsOwnClock;

impl Drop for TheAddsOwnClock {
    /// Disarm the clock and forget the step it was on.
    fn drop(&mut self) {
        ARMED.store(false, std::sync::atomic::Ordering::Release);
        let Some(mut held) = look() else {
            return;
        };
        *held = None;
    }
}

/// What:
///  Start the clock of one add's own work.
/// Why:
///   The allowance counts from here,
///  and a variable that names a different one is read once,
///  so a command's ending
///  names the number it was really given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clock = theAddsOwnWorkBegan();
/// ```
pub fn the_adds_own_work_began() -> TheAddsOwnClock {
    return arm(the_allowance());
}

/// What:
///  Start the clock with a ceiling of the caller's choosing.
/// Why:
///   A control arms a clock of milliseconds instead of the twenty shipped minutes,
///  so a wait is measured in a
///  moment and the shipped number stays what the code says it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clock = arm(Duration.fromMillis(200));
/// ```
fn arm(ceiling: Duration) -> TheAddsOwnClock {
    let Some(mut held) = look() else {
        debug(TAG, "the add's own clock could not be started");
        return TheAddsOwnClock;
    };
    *held = Some(Spending { ceiling, spent: Duration::ZERO, running_since: Some(Instant::now()), waiting_for: None });
    ARMED.store(true, std::sync::atomic::Ordering::Release);
    return TheAddsOwnClock;
}

/// What:
///  The clock stopped while the person's own forwarded command runs,
///  and running again when the value goes out of scope.
/// Why:
///   Native Git waits as long as its own work takes,
///  and the wrapper adds no bound to what the person typed:
///  a checkout
///  of twenty minutes is Git's own time and must not be counted against the wrapper's allowance,
///  or a healthy add would
///  fail its copy for a slow checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using person = thePersonsCommandBegan();
/// ```
#[must_use = "the clock runs again when the value is dropped, so dropping it at once counts the person's command"]
pub struct WhileGitRunsForThePerson;

impl Drop for WhileGitRunsForThePerson {
    /// Add the stretch that ran to what was spent,
    /// and stop.
    fn drop(&mut self) {
        let Some(mut held) = look() else {
            return;
        };
        let Some(spending) = held.as_mut() else {
            return;
        };
        let ran: Duration = spending.running_since.map_or(Duration::ZERO, |since| { return since.elapsed(); });
        spending.spent = spending.spent.saturating_add(ran);
        spending.running_since = Some(Instant::now());
    }
}

/// What:
///  Stop the clock while the person's own forwarded command runs.
/// Why:
///   The command is the person's,
///  so its time is not the wrapper's own and the allowance does not spend it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const person = thePersonsCommandBegan();
/// ```
pub fn the_persons_command_began() -> WhileGitRunsForThePerson {
    let Some(mut held) = look() else {
        return WhileGitRunsForThePerson;
    };
    let Some(spending) = held.as_mut() else {
        return WhileGitRunsForThePerson;
    };
    let ran: Duration = spending.running_since.map_or(Duration::ZERO, |since| { return since.elapsed(); });
    spending.spent = spending.spent.saturating_add(ran);
    spending.running_since = None;
    return WhileGitRunsForThePerson;
}

/// What:
///  How much of its allowance the add of this process has left,
///  or nothing when no add is running here.
/// Why:
///   Every step is clamped into what is left,
///  and a process that runs no add answers nothing instead of a ceiling it never counted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = whatIsLeft(); // undefined: no add here
/// ```
pub fn what_is_left() -> Option<Duration> {
    let held = look()?;
    let spending: &Spending = held.as_ref()?;
    return Some(spending.ceiling.saturating_sub(spent_now(spending)));
}

/// What:
///  What one add's own work spent of its allowance,
///  and the step it was waiting for when the last of it went.
/// Why:
///   The add's ending names a number and a step,
///  and both are read once so the sentence cannot pair one
///  add's ceiling with another's step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Spent = { ceiling: Duration; spent: Duration; waitingFor?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Spent {
    /// The allowance the add was armed with.
    ceiling: Duration,
    /// What its own work spent of it.
    spent: Duration,
    /// The step it was waiting for,
    /// as the step named itself.
    waiting_for: Option<String>,
}

impl Spent {
    /// The allowance the add was armed with.
    pub fn ceiling(&self) -> Duration {
        return self.ceiling;
    }

    /// What the add's own work spent of it.
    pub fn spent(&self) -> Duration {
        return self.spent;
    }

    /// The step the add was waiting for when the allowance went,
    /// as the step named itself,
    /// or nothing when no step was recorded.
    pub fn waiting_for(&self) -> Option<&str> {
        return self.waiting_for.as_deref();
    }

    /// What:
    ///  The line an add prints when its own work has spent the allowance and it stops before making a worktree.
    /// Why:
    ///   The person learns what the seconds went on,
    ///  which number was the limit and where to change it,
    ///  and what
    ///  the command left:
    ///  no worktree,
    ///  and nothing else of the add.
    ///  The remedy names the person's own Git,
    ///  because
    ///  Git's own command has no such limit and is what to run when the wrapper's own steps are what stalled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// `monochromatic-git-policy-cli: this \`git worktree add\` stopped at its allowance ...`
    /// ```
    pub fn the_add_stopped_line(&self, destination: Option<&Path>) -> String {
        let waited_for: String = match self.waiting_for.as_deref() {
            Some(step) => step.to_string(),
            None => String::from("a step that did not name itself"),
        };
        let left: String = match destination {
            Some(path) => format!(
                "The worktree at {} was not made, the empty folder it would have filled is gone, and nothing else of the add is left behind.",
                crate::shell_quote::shell_quote(path.to_string_lossy().as_bytes())
            ),
            None => String::from("No worktree was made, and nothing of the add is left behind."),
        };
        return format!(
            "monochromatic-git-policy-cli: this `git worktree add` used its whole allowance for its own steps ({} seconds of {} seconds; {ALLOWANCE_VARIABLE} sets it). The last step it waited for was {waited_for}, and the allowance is what ends a step still running when it is spent. {left} \
             Nothing of yours was changed. Run the command again to try once more, or raise the allowance with {ALLOWANCE_VARIABLE}=<seconds> if this repository's own steps really need longer here.",
            self.spent.as_secs(),
            self.ceiling.as_secs()
        );
    }

    /// What:
    ///  The line a step that is not started because the allowance is spent answers with.
    /// Why:
    ///   The step's own caller words its failure,
    ///  so this says what happened to this step and not what became of the whole add,
    ///  which
    ///  the add's own ending says.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// `${step} was not started: the add's allowance of ${n} seconds is spent ...`
    /// ```
    fn the_step_was_not_started_line(&self, step: &str) -> String {
        return format!(
            "{step} was not started: this `git worktree add` has spent its whole allowance for its own steps ({} seconds of {} seconds; {ALLOWANCE_VARIABLE} sets it)",
            self.spent.as_secs(),
            self.ceiling.as_secs()
        );
    }
}

/// What:
///  A spent allowance a control words without arming a clock.
/// Why:
///   The ending's words are checked against the numbers a person would see,
///  and a control that waited the shipped
///  twelve minutes to spend them would be a control nobody runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spent = Spent.forAControl(ceiling, spent, step);
/// ```
#[cfg(test)]
impl Spent {
    /// A spent allowance of the caller's numbers.
    pub(crate) fn for_a_control(ceiling: Duration, spent: Duration, waiting_for: Option<String>) -> Spent {
        return Spent { ceiling, spent, waiting_for };
    }
}

/// What:
///  What the add of this process has spent of its allowance,
///  when it has spent all of it.
/// Why:
///   The add's own ending asks this once,
///  before it starts its Git,
///  and the answer carries the step and the numbers the sentence needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spent = theAllowanceIsSpent(); // undefined: no add, or time left
/// ```
pub fn the_allowance_is_spent() -> Option<Spent> {
    let held = look()?;
    let spending: &Spending = held.as_ref()?;
    let spent: Duration = spent_now(spending);
    if spent < spending.ceiling {
        return None;
    }
    return Some(Spent { ceiling: spending.ceiling, spent, waiting_for: spending.waiting_for.clone() });
}

/// What:
///  Refuse to start a step when the add's allowance is spent,
///  and record which step is being waited for when it is not.
/// Why:
///   A step started after the allowance is gone would be ended at once,
///  so starting it only costs a process and a
///  misleading notice;
///  the refusal is a timeout,
///  which every caller of a Git step already words as a step that did not answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beforeAStep(subject); // throws a timeout once the allowance is spent
/// ```
pub fn before_a_step(waiting_for: &str) -> Result<(), std::io::Error> {
    let Some(mut held) = look() else {
        return Ok(());
    };
    let Some(spending) = held.as_mut() else {
        return Ok(());
    };
    let spent: Duration = spent_now(spending);
    if spent < spending.ceiling {
        // The last step that starts is the one an add that spends the allowance is said to have waited for, so a step that
        // is refused here does not take that place from the step that really spent the time.
        spending.waiting_for = Some(waiting_for.to_string());
        return Ok(());
    }
    let ended: Spent = Spent { ceiling: spending.ceiling, spent, waiting_for: Some(waiting_for.to_string()) };
    return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, ended.the_step_was_not_started_line(waiting_for)));
}

/// What:
///  The bound of one step of the ladder,
///  clamped into what is left of the add's allowance,
///  which is also recorded as the step the add is waiting for.
/// Why:
///   The ladder starts its own Git through the forwarded runner and not through a watched step,
///  so its ceiling arrives as a
///  plain span of time and the allowance clamps it here.
///  A step the allowance cannot hold is not started at all,
///  because a
///  ceiling of no seconds would start Git only to kill it (`before_a_step`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = aBoundWithinTheAllowance(ceilingOfTheStep, subject);
/// ```
pub(crate) fn a_bound_within_the_allowance(bound: Option<Duration>, waiting_for: &str) -> Option<Duration> {
    let Some(mut held) = look() else {
        return bound;
    };
    let Some(spending) = held.as_mut() else {
        return bound;
    };
    spending.waiting_for = Some(waiting_for.to_string());
    let left: Duration = spending.ceiling.saturating_sub(spent_now(spending));
    return bound.map(|ceiling| { return ceiling.min(left); });
}

/// What:
///  The terms and the bound of one step,
///  clamped into what is left of the add's allowance.
/// Why:
///   A bound a step keeps is a bound that belongs to that step alone;
///  with an allowance the add's own end is the
///  earlier of the two,
///  so the window,
///  the notice grace and the bound all shrink to what is left.
///  A step that may not be run twice is never ended,
///  so it keeps that rule and the command stops waiting for it at the allowance instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [terms, bound] = withinTheAllowance(terms, bound, subject); // both clamped to whatIsLeft()
/// ```
pub(crate) fn within_the_allowance(terms: WatchTerms, bound: StepBound, waiting_for: &str) -> (WatchTerms, StepBound) {
    if !a_clock_runs_here() {
        return (terms, bound);
    }
    let Some(mut held) = look() else {
        return (terms, bound);
    };
    let Some(spending) = held.as_mut() else {
        return (terms, bound);
    };
    spending.waiting_for = Some(waiting_for.to_string());
    let left: Duration = spending.ceiling.saturating_sub(spent_now(spending));
    let mut clamped_terms: WatchTerms = terms;
    clamped_terms.limit = clamped_terms.limit.min(left);
    // The grace is the class's, the interval's and half the window, whichever is shortest (`grace_for`); the window is
    // now the allowance's, so the half of it is taken here as well.
    clamped_terms.cadence.grace = clamped_terms.cadence.grace.min(clamped_terms.limit / 2);
    let clamped_bound: StepBound = match bound {
        StepBound::EndAt(ceiling) => StepBound::EndAt(ceiling.min(left)),
        StepBound::LeaveRunning { after, extra } => {
            let whole: Duration = after.saturating_add(extra).min(left);
            StepBound::LeaveRunning { after: whole.saturating_sub(extra.min(whole)), extra: extra.min(whole) }
        }
        // A step never ended is never ended here either; the command stops waiting for it when the allowance is gone.
        StepBound::Unbounded => StepBound::LeaveRunning { after: left, extra: Duration::ZERO },
    };
    return (clamped_terms, clamped_bound);
}

/// The controls of the allowance:
/// what one step is clamped to,
/// what a spent allowance refuses,
/// and the words of the add's ending.
#[cfg(test)]
#[path = "worktree_add_own_work_deadline_tests.rs"]
mod tests;
