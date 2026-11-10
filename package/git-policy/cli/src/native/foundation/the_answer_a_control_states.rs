//! What:
//!  An answer a control states for the jobs its own thread asks the blocking pool for,
//! so the two answers a thread refusal gives,
//! and a ceiling that passes,
//!  are exercised without a system that refuses threads at a chosen moment.
//! Why:
//!   `StartedAfterItsThreadWasRefused` needs a pool thread to take a queued job before the caller takes the claim on it,
//! which is a race of microseconds in the pool (`tokio-1.54.1`,
//! `runtime/blocking/pool.rs`,
//! `spawn_task`)
//!  and cannot be arranged from a control.
//! The work still runs,
//! as it does on the pool,
//! so what a control reads is the step's own changes and the caller's
//!  answer.
//! The answer is a thread's own,
//! so a control that drives its future on its own thread states it and no other test reads it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const guard = stateTheAnswer('noAnswer'); try { await applyAwaiting(plan, changes) } finally { guard[Symbol.dispose]() }
//! ```

use std::cell::Cell;

/// The answer a control states.
#[derive(Clone, Copy)]
pub(crate) enum AStatesAnswer {
    /// The system gave no thread,
    /// so the job will do nothing.
    NotRun,
    /// The job started on another thread,
    /// or outlasted its ceiling,
    /// so its value cannot be read.
    NoAnswer,
}

thread_local! {
    /// The answer this thread's next jobs are given,
    /// and nothing for the pool's own.
    static STATED: Cell<Option<AStatesAnswer>> = const { Cell::new(None) };
}

/// The answer this thread states for its next jobs.
pub(crate) fn the_answer_a_control_states() -> Option<AStatesAnswer> {
    return STATED.with(|slot| { return slot.get(); });
}

/// What:
///  States `answer` for every job this thread asks for until the guard drops.
/// Why:
///  A control that asks two of them,
/// a first that started and a second that did not,
/// needs the first answer to pass;
///  the guard keeps the statement to its own block.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using _stated = stateTheAnswer('notRun');
/// ```
pub(crate) fn state_the_answer(answer: AStatesAnswer) -> AStatedAnswer {
    STATED.with(|slot| { slot.set(Some(answer)); });
    return AStatedAnswer;
}

/// Ends the statement it was made with.
pub(crate) struct AStatedAnswer;

impl Drop for AStatedAnswer {
    fn drop(&mut self) {
        STATED.with(|slot| { slot.set(None); });
    }
}
