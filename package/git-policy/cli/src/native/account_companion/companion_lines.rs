//! What:
//!  What one companion run means for the add,
//!  and the lines a person reads:
//!  the skip,
//!  the failed arm,
//!  the deadline,
//!  and the two failures that happen before any child exists.
//! Why:
//!  A skip lets the add go ahead,
//!  because a check that cannot speak about the root attach must not turn the
//!  managed route off
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//! Q-a);
//!  every other failure refuses that add's managed attach and lets the slot take
//!  the reflink or ordinary Git route.
//!  The wording is the design's own
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "Every ending,
//! and the line it prints"),
//!  and the line goes to standard error under the reporting rules.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = runTheCompanion(); // { kind: 'passed' } | { kind: 'skipped', skip } | { kind: 'refused', refusal }
//! for (const line of verdict.lines()) console.error(line);
//! ```

use super::companion_calls::{CompanionCall, Failure};
use super::companion_settings::Settings;
use super::companion_steps::CompanionStep;
use std::time::Duration;

/// What the slot takes once the managed attach is refused.
pub const THE_FALLBACK_ROUTE: &str = "the reflink or ordinary Git route";

/// What:
///  One run's answer.
/// Why:
///  Three answers are all the add needs,
///  plus the control arm that runs nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verdict = { kind: 'passed' } | { kind: 'not-run' } | { kind: 'skipped'; skip: Skip } | { kind: 'refused'; refusal: Refusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompanionVerdict {
    /// Every step passed.
    Passed,
    /// The test-only control arm ran nothing,
    /// so no check happened and the add goes ahead as before.
    NotRun,
    /// A host refusal on the skip's list:
    /// the add goes ahead.
    Skipped {
        /// The call the host refused and the values it read.
        skip: Skip,
    },
    /// A failure the managed attach is refused for.
    Refused {
        /// What failed.
        refusal: Refusal,
    },
}

/// What:
///  The call a host refused and the settings it read,
///  as the skip line prints them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Skip = { call: Call; errno: number; settings: Settings };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Skip {
    /// The call that answered.
    pub call: CompanionCall,
    /// The number it answered.
    pub errno: i32,
    /// The three settings.
    pub settings: Settings,
}

/// What:
///  Why the managed attach is refused.
/// Why:
///  Each cause has its own line,
///  and each names what a person needs:
///  the step and call,
///  the signal,
///  the missing record,
///  the deadline,
///  the scratch directory,
///  or the fork.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { kind: 'arm'; step: Step; failure: Failure } | { kind: 'deadline'; deadline: Duration } | ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// One arm ran and failed.
    ArmFailed {
        /// The step.
        step: CompanionStep,
        /// The call and its errno.
        failure: Failure,
    },
    /// The child did not answer within the deadline.
    Deadline {
        /// The bound that passed.
        deadline: Duration,
    },
    /// The child died of a signal,
    /// naming the step that was in flight.
    ChildSignaled {
        /// The step after the last record the child wrote.
        step_in_flight: CompanionStep,
        /// The signal's number.
        signal: i32,
    },
    /// The child ended without a whole set of records.
    NoStatusLine {
        /// The step after the last whole record.
        step_in_flight: CompanionStep,
    },
    /// The system refused the fork for longer than the wrapper's budget.
    ForkRefused {
        /// How many refusals the budget spent.
        refusals: u32,
        /// The number the last one answered.
        errno: i32,
    },
    /// The status pipe could not be made.
    PipeRefused {
        /// The number the call answered.
        errno: i32,
    },
    /// The scratch directory could not be made,
    /// with the sentence that names the variable and what to do.
    Scratch {
        /// The sentence.
        message: String,
    },
}

impl CompanionVerdict {
    /// What:
    ///  Whether the add goes ahead with the managed attach.
    /// Why:
    ///  A skip and a control arm both let it go ahead,
    ///  and every refusal does not.
    pub fn the_managed_attach_goes_ahead(&self) -> bool {
        return !matches!(self, CompanionVerdict::Refused { .. });
    }

    /// What:
    ///  The lines this verdict prints,
    ///  which are none for a pass.
    pub fn lines(&self) -> Vec<String> {
        return match self {
            CompanionVerdict::Passed | CompanionVerdict::NotRun => Vec::new(),
            CompanionVerdict::Skipped { skip } => vec![
                format!(
                    "account companion skipped: {} answered {} (errno {}); user namespaces are unavailable here",
                    skip.call.name(),
                    CompanionCall::errno_word(skip.errno),
                    skip.errno
                ),
                skip.settings.line(),
            ],
            CompanionVerdict::Refused { refusal } => vec![refusal.line()],
        };
    }

    /// What:
    ///  Write every line this verdict carries to standard error,
    ///  which is where a person reads it.
    /// Why:
    ///  One writer for every ending,
    ///  so no call site decides what a person reads
    ///  and a refusal the add resolves before the check prints the same way as
    ///  one the check answers.
    pub fn report(&self) {
        use std::io::Write as _;
        for line in self.lines() {
            let _written: std::io::Result<()> = writeln!(std::io::stderr(), "{line}");
        }
    }
}

/// What:
///  The words one refusal prints,
///  naming the step and the call where there is one.
fn answered(failure: Failure) -> String {
    if failure.errno == super::companion_calls::NO_ERRNO {
        return format!("{} (no errno)", failure.call.name());
    }
    return format!("{} ({}, errno {})", failure.call.name(), CompanionCall::errno_word(failure.errno), failure.errno);
}

impl Refusal {
    /// The line this refusal prints.
    pub fn line(&self) -> String {
        return match self {
            Refusal::ArmFailed { step, failure } => format!(
                "managed add refused: the account companion's {} failed: {}; this slot takes {THE_FALLBACK_ROUTE}.",
                step.name(),
                answered(*failure)
            ),
            Refusal::Deadline { deadline } => format!(
                "managed add refused: the account companion did not answer within its deadline ({} ms); this slot takes {THE_FALLBACK_ROUTE}.",
                deadline.as_millis()
            ),
            Refusal::ChildSignaled { step_in_flight, signal } => format!(
                "managed add refused: the account companion's {} failed: the child (signal {signal}); this slot takes {THE_FALLBACK_ROUTE}.",
                step_in_flight.name()
            ),
            Refusal::NoStatusLine { step_in_flight } => format!(
                "managed add refused: the account companion's {} failed: the status pipe (no status line); this slot takes {THE_FALLBACK_ROUTE}.",
                step_in_flight.name()
            ),
            Refusal::ForkRefused { refusals, errno } => format!(
                "managed add refused: the account companion's start failed: fork (errno {errno}) after {refusals} refusals in the wrapper's start budget; this slot takes {THE_FALLBACK_ROUTE}."
            ),
            Refusal::PipeRefused { errno } => format!(
                "managed add refused: the account companion's start failed: pipe2 (errno {errno}); this slot takes {THE_FALLBACK_ROUTE}."
            ),
            Refusal::Scratch { message } => {
                format!("managed add refused: the account companion's scratch failed: {message}")
            }
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "companion_lines_tests.rs"]
mod tests;
