//! What:
//!  Test-only hooks of the service's fault points:
//! a control sets code to run,
//! or a step to fail,
//! at a named point of a landing on its own thread.
//! Why:
//!  Kept in a file of its own so the shipped `service_fault.rs` holds no inline test module (`rust/tests-in-sidecars`);
//!  it is declared there under `#[cfg(test)]`,
//! so the release executable never contains it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a module that exists only in test builds
//! export const hooks = { failAt, clearFailures, set, takeEvents };
//! ```
use std::cell::RefCell;

thread_local! {
    /// The steps of a landing on this thread,
    /// in the order they happened.
    static EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// The point and the code to run at it.
    static HOOK: RefCell<Option<(String, std::rc::Rc<dyn Fn()>)>> = const { RefCell::new(None) };
    /// The points whose steps fail on this thread.
    static FAILING: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Make the step at `point` fail on this thread's landings until `clear_failures`.
pub(crate) fn fail_at(point: &str) {
    FAILING.with(|failing| { failing.borrow_mut().push(point.to_string()); });
}

/// Let every step succeed again.
pub(crate) fn clear_failures() {
    FAILING.with(|failing| { failing.borrow_mut().clear(); });
}

/// Whether the step at `point` fails.
pub(super) fn failing(point: &str) -> bool {
    return FAILING.with(|failing| { return failing.borrow().iter().any(|named| { return named == point; }); });
}

/// Run nothing at any point again.
pub(crate) fn clear() {
    HOOK.with(|hook| { *hook.borrow_mut() = None; });
}

/// Run `code` whenever the landing of this thread reaches `point`.
pub(crate) fn set(point: &str, code: std::rc::Rc<dyn Fn()>) {
    HOOK.with(|hook| { *hook.borrow_mut() = Some((point.to_string(), code)); });
}

/// Remember that `event` happened.
pub(super) fn record(event: &str) {
    EVENTS.with(|events| { events.borrow_mut().push(event.to_string()); });
}

/// The steps reported on this thread since the last call,
/// in order.
pub(crate) fn take_events() -> Vec<String> {
    return EVENTS.with(|events| { return std::mem::take(&mut *events.borrow_mut()); });
}

/// Run the hook if it is for `point`.
pub(super) fn run(point: &str) {
    let found: Option<std::rc::Rc<dyn Fn()>> = HOOK.with(|hook| { return hook.borrow().as_ref().filter(|(named, _)| { return named == point; }).map(|(_, code)| { return std::rc::Rc::clone(code); }); });
    if let Some(hook) = found {
        hook();
    }
}
