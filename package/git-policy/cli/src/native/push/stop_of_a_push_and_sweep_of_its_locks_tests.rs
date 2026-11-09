//! Controls for the sweep after a stopped push:
//! its wait for the rest of the tree ends at the ceiling of a stop.

use super::*;

/// The sweep's wait for the rest of the tree ends at the ceiling of a stop,
/// so a descendant that never ends leaves no task waking for the life of the service.
#[test]
fn the_sweep_waits_for_the_tree_no_longer_than_the_ceiling_of_a_stop() {
    let never: bool = crate::lazy_runtime::run_on_runtime(wait_until_gone(&|| return true, Duration::ZERO)).expect("the runtime runs the wait");
    assert!(!never, "a tree that keeps running is left at the ceiling instead of being waited for for ever");
    // The control: the same wait answers at once for a tree that is gone, so the ceiling above is what ended it.
    let at_once: bool = crate::lazy_runtime::run_on_runtime(wait_until_gone(&|| return false, Duration::ZERO)).expect("the runtime runs the wait");
    assert!(at_once, "a tree that is already gone ends the wait");
}
