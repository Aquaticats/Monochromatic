//! What:
//!  What the probe's status lines add up to:
//!  each arm's outcome,
//!  the first failing step,
//!  the step that was in flight and the last one that finished,
//!  and the verdict the record carries.
//! Why:
//!  The arms are the record's units,
//!  and the verdict is one word a reader gates on
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  Q2,
//!  Q4 and Q5).
//!  A failed arm stops the probe,
//!  so every arm after it reads as not run rather than as a pass,
//!  an arm that could not run blocks the route whatever else passed,
//!  and a child that ended without a line is an arm failure rather than an
//!  absence.
//!  All of that is folding,
//!  which is kept here so the mode holds no second copy of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = new ProbeReport(); report.record(line); report.finishWith({ kind: 'signal', number: 9 });
//! const verdict = report.verdict(); // { kind: 'failed', step: 'move' }
//! ```

use super::probe_arms::{
    ProbeArm,
    ProbeStep,
};
use super::probe_errno_names::ErrnoWord;
use super::probe_status_line::StepAnswer;
use super::probe_status_line::StepLine;

/// What:
///  How a child's pipe ended instead of carrying its next line.
/// Why:
///   A child that died of a signal,
///  a line of another kind and a pipe that ended with no line are three
///  different arm failures,
///  and each must be named rather than folded into the last step that passed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ending = { kind: 'no-status-line' } | { kind: 'unparsed-line'; text: string } | { kind: 'signal'; number: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChildEnding {
    /// The child exited and wrote no further line.
    NoStatusLine,
    /// A line on the pipe was not one this build reads.
    UnparsedLine {
        /// The line as read.
        text: String,
    },
    /// The child died of a signal.
    Signal {
        /// The signal's number.
        number: i32,
    },
}

/// What:
///  One arm's outcome,
///  in the words the record carries.
/// Why:
///   An arm either passed,
///  failed with the errno the first failing step named,
///  or did not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ArmOutcome = { arm: Arm; answer: 'passed' | 'failed' | 'not-exercised'; errno?: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArmAnswer {
    /// Every step of the arm passed.
    Passed,
    /// A step of the arm failed.
    Failed,
    /// The arm did not run,
    /// because a step before it failed or did not run.
    NotExercised,
}

/// What:
///  One arm and what became of it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArmOutcome {
    /// The arm.
    pub arm: ProbeArm,
    /// What became of it.
    pub answer: ArmAnswer,
    /// The errno field the first failing step answered,
    /// where the arm failed.
    pub errno: ErrnoWord,
}

/// What:
///  The verdict the record carries.
/// Why:
///   A route is enabled by a pass and by nothing else,
///  a failed arm refuses new managed attaches,
///  an arm that could not run blocks the route,
///  a missed deadline fails closed and names the step in flight,
///  and a gate refusal skips the probe without a verdict either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verdict = { kind: 'passed' } | { kind: 'failed'; step: Step } | { kind: 'blocked'; arm: Arm } | { kind: 'timed-out'; inFlight?: Step; lastFinished?: Step } | { kind: 'skipped'; step: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeVerdict {
    /// Every arm passed.
    Passed,
    /// A step failed,
    /// or a child ended without answering one.
    Failed {
        /// The step that failed,
        /// or the step that was in flight when the child stopped answering.
        step: ProbeStep,
    },
    /// An arm did not run,
    /// so the route stays blocked.
    Blocked {
        /// The first arm that did not run.
        arm: ProbeArm,
    },
    /// The mode's deadline passed.
    TimedOut {
        /// The step after the last one the mode read.
        in_flight: Option<ProbeStep>,
        /// The last step that wrote a line.
        last_finished: Option<ProbeStep>,
    },
    /// The gate refused before the mode was entered.
    Skipped {
        /// The gate's failing step,
        /// as the gate names it.
        gate_step: &'static str,
    },
}

impl ProbeVerdict {
    /// The verdict in the one word the record carries.
    pub fn word(self) -> &'static str {
        return match self {
            ProbeVerdict::Passed => "passed",
            ProbeVerdict::Failed {
                ..
            } => "failed",
            ProbeVerdict::Blocked {
                ..
            } => "blocked",
            ProbeVerdict::TimedOut {
                ..
            } => "timed-out",
            ProbeVerdict::Skipped {
                ..
            } => "skipped",
        };
    }

    /// The step a failed verdict names,
    /// where it names one.
    pub fn named_step(self) -> Option<ProbeStep> {
        return match self {
            ProbeVerdict::Failed {
                step,
            } => Some(step),
            ProbeVerdict::Passed
            | ProbeVerdict::Blocked {
                ..
            }
            | ProbeVerdict::TimedOut {
                ..
            }
            | ProbeVerdict::Skipped {
                ..
            } => None,
        };
    }
}

/// What:
///  Every line the mode read,
///  and how the child's pipe ended,
///  folded into arm outcomes and one verdict.
/// Why:
///   The mode writes the record from this one value,
///  so no caller re-derives an arm from the raw lines and disagrees with the
///  verdict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ProbeReport { lines: StepLine[]; ending?: Ending; arms(): ArmOutcome[]; verdict(): Verdict }
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProbeReport {
    /// The step lines,
    /// in the order the mode read them.
    lines: Vec<StepLine>,
    /// How a pipe ended,
    /// where one ended without a line for the step in flight.
    ending: Option<ChildEnding>,
}

impl ProbeReport {
    /// An empty report.
    pub fn new() -> Self {
        return ProbeReport {
            lines: Vec::new(),
            ending: None,
        };
    }

    /// Keep one step's line.
    pub fn record(
        &mut self,
        line: StepLine,
    ) {
        self.lines.push(line);
    }

    /// Keep how a child's pipe ended.
    pub fn finish_with(
        &mut self,
        ending: ChildEnding,
    ) {
        self.ending = Some(ending);
    }

    /// The lines the mode read,
    /// in order.
    pub fn lines(&self) -> &[StepLine] {
        return self.lines.as_slice();
    }

    /// How a pipe ended,
    /// where one did.
    pub fn ending(&self) -> Option<&ChildEnding> {
        return self.ending.as_ref();
    }

    /// The step that was running when the report ends:
    /// the step after the last one the worker answered,
    /// or the worker's first step when it answered none.
    pub fn in_flight(&self) -> Option<ProbeStep> {
        let last: Option<usize> = ProbeStep::WORKER_ORDER
            .iter()
            .rposition(|step| return self.line_for(*step).is_some());
        return match last {
            None => ProbeStep::WORKER_ORDER.first().copied(),
            Some(index) => ProbeStep::WORKER_ORDER.get(index + 1).copied(),
        };
    }

    /// The last worker step that answered,
    /// where one did.
    pub fn last_finished(&self) -> Option<ProbeStep> {
        return ProbeStep::WORKER_ORDER
            .iter()
            .rev()
            .copied()
            .find(|step| return self.line_for(*step).is_some());
    }

    /// The line one step wrote,
    /// where it wrote one.
    fn line_for(
        &self,
        step: ProbeStep,
    ) -> Option<&StepLine> {
        return self.lines.iter().find(|line| return line.step == step);
    }

    /// The first step that failed,
    /// in the worker's own order.
    pub fn first_failed_step(&self) -> Option<ProbeStep> {
        return ProbeStep::WORKER_ORDER.iter().copied().find(|step| {
            return matches!(
                self.line_for(*step).map(|line| return &line.answer),
                Some(StepAnswer::Failed { .. })
            );
        });
    }

    /// Whether the report already says why the worker stopped:
    /// a step failed,
    /// or a step could not run.
    /// Why:
    ///  A worker that wrote either ended on purpose,
    ///  and a missing teardown line is then its own account rather than a
    ///  child that died without answering.
    pub fn stopped_on_purpose(&self) -> bool {
        return self.lines().iter().any(|line| {
            return matches!(
                line.answer,
                StepAnswer::Failed { .. } | StepAnswer::NotExercised { .. }
            );
        });
    }

    /// What became of one arm.
    pub fn arm_outcome(
        &self,
        arm: ProbeArm,
    ) -> ArmOutcome {
        let mut every_step_passed: bool = true;
        for step in ProbeStep::ALL
            .iter()
            .copied()
            .filter(|step| return step.arm() == arm)
        {
            match self.line_for(step).map(|line| return &line.answer) {
                Some(StepAnswer::Passed) => {}
                Some(StepAnswer::Failed {
                    errno,
                }) => {
                    return ArmOutcome {
                        arm,
                        answer: ArmAnswer::Failed,
                        errno: *errno,
                    };
                }
                Some(StepAnswer::NotExercised {
                    ..
                })
                | None => every_step_passed = false,
            }
        }
        return ArmOutcome {
            arm,
            answer: if every_step_passed {
                ArmAnswer::Passed
            } else {
                ArmAnswer::NotExercised
            },
            errno: ErrnoWord::None,
        };
    }

    /// Every arm's outcome,
    /// in the design's order.
    pub fn arms(&self) -> Vec<ArmOutcome> {
        return ProbeArm::ALL
            .iter()
            .map(|arm| return self.arm_outcome(*arm))
            .collect();
    }

    /// Whether every arm passed.
    pub fn passed(&self) -> bool {
        return self
            .arms()
            .iter()
            .all(|outcome| return outcome.answer == ArmAnswer::Passed);
    }

    /// The verdict this report's lines and ending add up to.
    pub fn verdict(&self) -> ProbeVerdict {
        if self.ending.is_some() {
            return ProbeVerdict::Failed {
                step: self.in_flight().unwrap_or(ProbeStep::Teardown),
            };
        }
        if let Some(step) = self.first_failed_step() {
            return ProbeVerdict::Failed {
                step,
            };
        }
        let arms: Vec<ArmOutcome> = self.arms();
        if let Some(outcome) = arms
            .iter()
            .find(|outcome| return outcome.answer != ArmAnswer::Passed)
        {
            return ProbeVerdict::Blocked {
                arm: outcome.arm,
            };
        }
        return ProbeVerdict::Passed;
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_report_tests.rs"]
mod tests;
