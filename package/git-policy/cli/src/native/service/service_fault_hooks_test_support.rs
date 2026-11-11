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
//! export const hooks = { failAt, clearFailures, set, takeEvents, holdOnAnyThread };
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

/// What:
///  A point a control holds on whichever thread reaches it,
/// with the end that says the point was reached and the end that lets it go.
/// Why:
///  The hooks above belong to one thread,
/// which suits a landing that lives on one thread;
///  the check that decides whose a request is runs inside a job of the blocking pool
/// (`is_foreign` in `service_server_connection_handling.rs`),
///  so a control cannot reach it with `set`.
///  A hold from the test's own thread blocks no worker of the runtime,
/// and the connection it holds is watched from the client.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdOnAnyThread({ point: 'in-the-instance-read', reached, letGo })
/// ```
pub(crate) struct HoldOnAnyThread {
    /// The point this hold is for.
    pub(crate) point: String,
    /// Where the thread that reached the point says so.
    pub(crate) reached: tokio::sync::oneshot::Sender<()>,
    /// The end that lets the held thread go on.
    pub(crate) let_go: std::sync::mpsc::Receiver<()>,
}

/// The hold a control set,
/// if any,
/// taken by the thread that reaches it so a second reach passes.
static HELD_ON_ANY_THREAD: std::sync::Mutex<Option<HoldOnAnyThread>> = std::sync::Mutex::new(None);

/// What:
///  Holds `point` on whichever thread reaches it,
/// until the caller sends on the end that lets it go.
/// Why:
///   The caller learns the point was reached from `reached`,
/// so "while the point waits" is a state and not a duration,
///  and the thread that waits is let go by the sender the caller keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reached, reachedWait] = channel(); const [letGo, held] = channel(); holdOnAnyThread({ point, reached, letGo: held });
/// ```
pub(crate) fn hold_on_any_thread(held: HoldOnAnyThread) {
    *HELD_ON_ANY_THREAD.lock().expect("the hold registry") = Some(held);
}

/// What:
///  A point a control holds without holding the thread that reaches it,
/// with the end that says the point was reached and the end that lets it go.
/// Why:
///   A hold that blocks the thread it is reached on holds a worker of the runtime,
/// and a runtime with few workers then runs nothing else,
/// which is no state the shipped code can reach:
/// the steps a hold stands in for wait on a child,
/// on the network or on storage,
/// and each of those frees the worker it waited on.
/// An await frees it too,
/// so the service goes on beside a held step as it does beside a trash that takes minutes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdAwaiting({ point: 'inside-a-slow-step', reached, letGo })
/// ```
pub(crate) struct HoldAwaitingOnAnyThread {
    /// The point this hold is for.
    pub(crate) point: String,
    /// Where the task that reached the point says so.
    pub(crate) reached: tokio::sync::oneshot::Sender<()>,
    /// The end that lets the held task go on.
    pub(crate) let_go: tokio::sync::oneshot::Receiver<()>,
}

/// The hold awaited at a point,
/// if any,
/// taken by the task that reaches it so a second reach passes.
static HELD_AWAITING: std::sync::Mutex<Option<HoldAwaitingOnAnyThread>> = std::sync::Mutex::new(None);

/// What:
///  Holds `point` on whichever task reaches it,
/// until the caller sends on the end that lets it go.
/// Why:
///   The caller learns the point was reached from `reached`,
/// so "while the point waits" is a state and not a duration,
/// and the task that waits is let go by the sender the caller keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reached, reachedWait] = channel(); const [letGo, held] = channel(); holdAwaiting({ point, reached, letGo: held });
/// ```
pub(crate) fn hold_awaiting_on_any_thread(held: HoldAwaitingOnAnyThread) {
    *HELD_AWAITING.lock().expect("the hold registry") = Some(held);
}

/// What:
///  Waits at `point` while a control asked for it,
/// and says the point was reached.
/// Why:
///   A hold is taken when it is reached,
/// so a later reach of the same point passes and a control cannot block a second request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = take(); if (held?.point !== point) return; held.reached.send(); await held.letGo;
/// ```
pub(crate) async fn await_on_any_thread(point: &str) {
    let taken: Option<HoldAwaitingOnAnyThread> = HELD_AWAITING.lock().expect("the hold registry").take().filter(|held| { return held.point == point; });
    let Some(held) = taken else { return };
    let _said = held.reached.send(());
    let _let_go = held.let_go.await;
}

/// What:
///  Holds the thread that reaches `point` while a control asked for it,
/// and says the point was reached.
/// Why:
///   A hold is taken when it is reached,
/// so a later reach of the same point passes and a control cannot block a second request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = take(); if (held?.point !== point) return; held.reached.send(); held.letGo.recv();
/// ```
pub(crate) fn run_on_any_thread(point: &str) {
    let taken: Option<HoldOnAnyThread> = HELD_ON_ANY_THREAD.lock().expect("the hold registry").take().filter(|held| { return held.point == point; });
    let Some(held) = taken else { return };
    let _said = held.reached.send(());
    let _let_go = held.let_go.recv();
}
