//! What:
//!  The instant between a pool sweep's liveness check and its removal,
//!  which a control fills with the claim of a live fill.
//! Why:
//!  Whether a fill that claims its staging tree in that instant keeps it
//!  is what tells a sweep that removes under the claim from one that looks and lets go
//!  (`tree_held_by_a_live_process.rs`,
//!  `remove_as_a_sweep`).
//!  Kept in a file of its own so the shipped `shadow_pool.rs` holds no state a release would carry
//!  (`rust/tests-in-sidecars`);
//!  it is declared there under `#[cfg(test)]`,
//!  so the release executable never contains it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a module that exists only in test builds
//! export const removalPoint = { set, reached };
//! ```
use std::cell::RefCell;
use std::path::Path;

thread_local! {
    static BEFORE_THE_REMOVAL: RefCell<Option<Box<dyn FnMut(&Path)>>> = const { RefCell::new(None) };
}

/// What:
///  Sets what a control does at the instant before a sweep removes a stale staging tree.
/// Why:
///  A fill that takes its claim there is a live holder of a tree the sweep has already decided against,
///  and a control has to place that holder at that instant and nowhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beforeTheRemoval = hook
/// ```
pub(crate) fn set_before_the_removal<F: FnMut(&Path) + 'static>(hook: F) {
    BEFORE_THE_REMOVAL.with(|slot| { *slot.borrow_mut() = Some(Box::new(hook)); });
}

/// What:
///  Does what the control set, with the tree the sweep means to remove.
/// Why:
///  The call site is the only moment this thread has the entry and has not yet removed it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beforeTheRemoval?.(entry)
/// ```
pub(crate) fn before_the_removal(entry: &Path) {
    BEFORE_THE_REMOVAL.with(|slot| {
        if let Some(hook) = slot.borrow_mut().as_deref_mut() {
            hook(entry);
        }
    });
}
