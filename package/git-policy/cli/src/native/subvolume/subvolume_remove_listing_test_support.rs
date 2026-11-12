//! What:
//!  The moment a removal has read a directory's entries and has not yet removed them,
//!  for a control that has to change the tree there.
//! Why:
//!   A removal reads a directory once and removes what it read
//! (`subvolume_remove_walk.rs`,
//! `worktree_trash_remove.rs`),
//! so the only way a control can put a boundary or an entry in that directory after
//!  the read is to act inside the removal,
//! and a control that raced a live removal would pass or fail by
//!  what the host happened to schedule.
//! The control states the act it wants at each directory the removal has just read,
//!  the removal calls it on its own thread,
//! and the release executable carries none of this.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! actAfterEachListing((listed) => { if (listed.endsWith('/locked')) createSubvolume(`${listed}/boundary`); });
//! const reason = removeTreeBeneath({ ... }); // refuses the boundary it met there
//! stopActing();
//! ```

use std::cell::RefCell;
use std::path::Path;

/// The act this thread stated,
/// and nothing when it stated none.
thread_local! {
    // The act, kept per thread so two controls running beside each other cannot call each other's.
    static THE_ACT: RefCell<Option<Box<dyn FnMut(&Path)>>> = const { RefCell::new(None) };
}

/// What:
///  Ask every removal this thread runs to call `act` at each directory it has just read,
///  before the first entry of that directory is removed.
/// Why:
///   The act arrives with the control that needs it,
/// the directory arrives with the moment,
/// and the act decides which directory it cares about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function actAfterEachListing(act: (listed: string) => void): void; // replaces the act this thread stated before
/// ```
pub(crate) fn act_after_each_listing(act: impl FnMut(&Path) + 'static) {
    THE_ACT.with(|cell| {
        *cell.borrow_mut() = Some(Box::new(act));
    });
}

/// What:
///  Call the act this thread stated, with the directory a removal has just read.
/// Why:
///   The removal calls this and knows nothing of what the control does;
/// a thread that stated no act pays one thread-local lookup per directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function afterListing(listed: string): void; // nothing when no act is stated
/// ```
pub(crate) fn after_listing(listed: &Path) {
    THE_ACT.with(|cell| {
        if let Some(act) = cell.borrow_mut().as_mut() {
            act(listed);
        }
    });
}

/// What:
///  Forget the act this thread stated,
/// so a later removal on it runs untouched.
/// Why:
///   A control that leaves its act behind would act on every removal the rest of the test makes.
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
