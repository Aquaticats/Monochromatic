//! What:
//!  The judgement of one watched command:
//!  what the last look showed,
//!  since when nothing has moved,
//!  and which of the three limits end it.
//! Why:
//!  A wall-clock bound cannot tell a command that waits on the disk from one that loops or has deadlocked,
//!  so the guard judges a tree by what moved:
//!  its processor time,
//!  the bytes it moved,
//!  the work of the children it waited for,
//!  and the set of processes,
//!  and by whether any task of it is runnable,
//!  waiting for the disk or held from outside.
//!  The judge holds that memory and nothing else,
//!  so a control can judge a look it made itself,
//!  without a command,
//!  a monitor thread or a kill.
//!  The three limits it judges by are in `binary_supervisor_limits.rs`,
//!  and a look at a process is `binary_supervisor_proc.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Judge { observe(elapsed: number, sample: Sample, heldByTest: boolean): Reason | undefined }
//! ```
use super::supervisor_limits::{
    Limits,
    Reason,
};
use super::supervisor_proc::Sample;
use std::time::Duration;

/// What the previous look showed,
/// to compare with.
type Previous = (u64, u64, u64, Vec<(i32, u64)>);

/// The judgement of one watched command:
/// it remembers what the last look showed and since when nothing moved.
#[derive(Clone, Debug)]
pub struct Judge {
    /// The limits applied.
    limits: Limits,
    /// The time of the last look at which anything moved or a task was runnable,
    /// waiting for the disk or held.
    last_change: Duration,
    /// What the previous look showed,
    /// to compare with.
    previous: Option<Previous>,
    /// The most processor time any look showed:
    /// a process that is waited for moves its time from itself to its parent,
    /// so
    ///  the figure is not monotonic by itself.
    cpu_peak_ticks: u64,
    /// Processor time of one clock tick,
    /// in nanoseconds.
    tick: Duration,
}

impl Judge {
    /// A judge with nothing seen yet;
    /// `tick` is the length of a clock tick.
    pub fn new(
        limits: Limits,
        tick: Duration,
    ) -> Judge {
        return Judge {
            limits,
            last_change: Duration::ZERO,
            previous: None,
            cpu_peak_ticks: 0,
            tick,
        };
    }

    /// Judge one look taken `elapsed` after the command started;
    /// `Some` ends the command.
    /// `held_by_test` says the test holds
    ///  the command on purpose (`hold_idle_judgment`),
    /// which suspends the idle rule and nothing else.
    ///  The processor budget and the backstop are checked first,
    /// because they hold whatever the tree is doing.
    pub fn observe(
        &mut self,
        elapsed: Duration,
        sample: &Sample,
        held_by_test: bool,
    ) -> Option<Reason> {
        self.cpu_peak_ticks = self.cpu_peak_ticks.max(sample.cpu_ticks);
        let used: Duration = self
            .tick
            .saturating_mul(u32::try_from(self.cpu_peak_ticks).unwrap_or(u32::MAX));
        if used >= self.limits.cpu_budget {
            return Some(Reason::CpuBudget(used));
        }
        if elapsed >= self.limits.backstop {
            return Some(Reason::Backstop(elapsed));
        }
        let moved: bool =
            self.previous
                .as_ref()
                .is_none_or(|(cpu, activity, faults, processes)| {
                    return *cpu != sample.cpu_ticks
                        || *activity != sample.activity
                        || *faults != sample.child_faults
                        || *processes != sample.processes;
                });
        if moved
            || sample.runnable
            || sample.disk_wait
            || sample.answer_wait
            || sample.held
            || held_by_test
        {
            self.last_change = elapsed;
        }
        self.previous = Some((
            sample.cpu_ticks,
            sample.activity,
            sample.child_faults,
            sample.processes.clone(),
        ));
        let idle: Duration = elapsed.saturating_sub(self.last_change);
        if idle >= self.limits.no_progress {
            return Some(Reason::NoProgress(idle));
        }
        return None;
    }
}
