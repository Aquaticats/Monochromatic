//! What:
//!  Test-only machine ID:
//! a control gives its thread a machine ID in place of the host's,
//! so the control does not depend on the host having one.
//! Why:
//!  Kept in a file of its own so the shipped `claim_time.rs` holds no inline test module (`rust/tests-in-sidecars`);
//!  it is declared there under `#[cfg(test)]`,
//! so the release executable never contains it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a module that exists only in test builds
//! export const testMachine = { injected, inject };
//! ```
use std::cell::Cell;

thread_local! {
    static INJECTED: Cell<Option<&'static str>> = const { Cell::new(None) };
}

/// What this thread's machine ID is:
/// the injected text (empty for a host with none),
/// or nothing injected.
pub(crate) fn injected() -> Option<&'static str> {
    return INJECTED.with(Cell::get);
}

/// Gives this thread the machine ID `id` (empty for a host with none) until the thread ends.
pub(crate) fn inject(id: &'static str) {
    INJECTED.with(|cell| { cell.set(Some(id)); });
}
