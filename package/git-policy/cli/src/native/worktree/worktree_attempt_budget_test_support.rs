//! What:
//!  The steps a command can be killed at between proving a leftover of a removal and queueing the delete it owes,
//!  for a control that kills a command at each of them and proves the delete is still owed afterwards.
//! Why:
//!  A process can be killed at any instruction,
//!  and the one thing that must not follow is a command that dies after spending what the attempts a record may cost
//!  before it queues anything:
//!  the next command would find every attempt spent, queue no delete, and claim attempts no service ever made.
//!  The control states the step it dies at, the look calls this at each step it reaches, and the release executable
//!  carries none of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! actAtEachStep((step) => { if (step === watching) process.exit(KILLED_EXIT) });
//! const answer = queueTheLeavingsARecordNoLongerNames(common);
//! ```

use std::cell::RefCell;

/// The step right after the look has proved the subvolume a leaving names still stands where the rename put it.
pub(crate) const AFTER_THE_PROOF: &str = "after-the-proof";

/// The step immediately before the delete of that leaving is queued.
pub(crate) const BEFORE_THE_DELETE_IS_QUEUED: &str = "before-the-delete-is-queued";

/// What:
///  Ask every look this thread runs to call `act` with each step it reaches.
/// Why:
///   The act arrives with the control that needs it,
///  the step arrives with the moment,
///  and the act decides which one it dies at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function actAtEachStep(act: (step: string) => void): void; // replaces the act this thread stated before
/// ```
pub(crate) fn act_at_each_step(act: impl FnMut(&str) + 'static) {
    THE_ACT.with(|cell| {
        *cell.borrow_mut() = Some(Box::new(act));
    });
}

/// What:
///  Call the act this thread stated, with the step the look has just reached.
/// Why:
///   The look calls this and knows nothing of what the control does;
/// a thread that stated no act pays one thread-local lookup per step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reached(step: string): void; // nothing when no act is stated
/// ```
pub(crate) fn reached(step: &str) {
    THE_ACT.with(|cell| {
        if let Some(act) = cell.borrow_mut().as_mut() {
            act(step);
        }
    });
}

// The act this thread stated, and nothing when it stated none.
thread_local! {
    // The act, kept per thread so two controls running beside each other cannot call each other's.
    static THE_ACT: RefCell<Option<Box<dyn FnMut(&str)>>> = const { RefCell::new(None) };
}
