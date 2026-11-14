//! What:
//!  The moment a removal of a worktree's subvolume has renamed it out of the person's way
//!  and has not yet asked the queue whether a round of the commit service owes its delete,
//!  for a control that settles that record there.
//! Why:
//!  A round of the service may claim the record the hand-over queued and look at the directory while it still
//!  stands at the worktree's own name,
//!  so the round settles a delete it found nothing for,
//!  and the rename that follows leaves a directory no queue record names.
//!  The command's own look at the queue is what repairs that,
//!  and the two steps are not one:
//! a control that has to settle the record in that window cannot time a race,
//!  so it states the act here and the hand-over calls it at the one moment the state can be changed.
//!  The release executable carries none of this.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! actAfterTheRename((renamed) => { settleTheQueuedDelete(renamed); subvolumeCreate(`${renamed}/inner`); });
//! const answer = handOverWith({ ... }); // its own delete is refused, and the leaving names the directory
//! stopActing();
//! ```

use std::cell::RefCell;
use std::path::Path;

/// What:
///  Ask every hand-over this thread runs to call `act` once its rename is made,
///  before it looks at what the queue owes.
/// Why:
///   The act arrives with the control that needs it,
///  the renamed directory arrives with the moment,
///  and the act decides what it does there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function actAfterTheRename(act: (renamed: string) => void): void; // replaces the act this thread stated before
/// ```
pub(crate) fn act_after_the_rename(act: impl FnMut(&Path) + 'static) {
    THE_ACT.with(|cell| {
        *cell.borrow_mut() = Some(Box::new(act));
    });
}

/// What:
///  Call the act this thread stated, with the directory the rename made.
/// Why:
///   The hand-over calls this and knows nothing of what the control does;
/// a thread that stated no act pays one thread-local lookup per rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function afterTheRename(renamed: string): void; // nothing when no act is stated
/// ```
pub(crate) fn after_the_rename(renamed: &Path) {
    THE_ACT.with(|cell| {
        if let Some(act) = cell.borrow_mut().as_mut() {
            act(renamed);
        }
    });
}

/// What:
///  Forget the act this thread stated,
/// so a later hand-over on it runs untouched.
/// Why:
///   A control that leaves its act behind would act on every hand-over the rest of the test makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopActing(): void;
/// ```
pub(crate) fn stop_acting() {
    THE_ACT.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

// The act this thread stated, and nothing when it stated none.
thread_local! {
    // The act, kept per thread so two controls running beside each other cannot call each other's.
    static THE_ACT: RefCell<Option<Box<dyn FnMut(&Path)>>> = const { RefCell::new(None) };
}
