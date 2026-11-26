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
    FAILING.with(|failing| {
        failing.borrow_mut().push(point.to_string());
    });
}

/// Let every step succeed again.
pub(crate) fn clear_failures() {
    FAILING.with(|failing| {
        failing.borrow_mut().clear();
    });
}

/// Whether the step at `point` fails.
pub(super) fn failing(point: &str) -> bool {
    return FAILING.with(|failing| {
        return failing.borrow().iter().any(|named| {
            return named == point;
        });
    });
}

/// Run nothing at any point again.
pub(crate) fn clear() {
    HOOK.with(|hook| {
        *hook.borrow_mut() = None;
    });
}

/// Run `code` whenever the landing of this thread reaches `point`.
pub(crate) fn set(
    point: &str,
    code: std::rc::Rc<dyn Fn()>,
) {
    HOOK.with(|hook| {
        *hook.borrow_mut() = Some((point.to_string(), code));
    });
}

/// Remember that `event` happened.
pub(super) fn record(event: &str) {
    EVENTS.with(|events| {
        events.borrow_mut().push(event.to_string());
    });
}

/// The steps reported on this thread since the last call,
/// in order.
pub(crate) fn take_events() -> Vec<String> {
    return EVENTS.with(|events| {
        return std::mem::take(&mut *events.borrow_mut());
    });
}

/// Run the hook if it is for `point`.
pub(super) fn run(point: &str) {
    let found: Option<std::rc::Rc<dyn Fn()>> = HOOK.with(|hook| {
        return hook
            .borrow()
            .as_ref()
            .filter(|(named, _)| {
                return named == point;
            })
            .map(|(_, code)| {
                return std::rc::Rc::clone(code);
            });
    });
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

/// What:
///  The holds the controls of one process set,
/// each taken by the thread that reaches its point so a second reach passes.
/// Why:
///  A list and not one slot:
///  `cargo test --lib` runs a crate's tests as threads of one process,
///  so two controls of one module arm their points side by side,
///  and one slot made the later arming drop the earlier control's hold (issue 6497).
///  A point names the instance its own control owns
///  (its own directory, its own request, as `collection_point` and `instance_read_point` state),
///  so two holds never answer for each other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HELD_ON_ANY_THREAD: HoldOnAnyThread[] = [];
/// ```
static HELD_ON_ANY_THREAD: std::sync::Mutex<Vec<HoldOnAnyThread>> = std::sync::Mutex::new(Vec::new());

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
    let point: String = held.point.clone();
    // What:     the arming and the question of whether this point is already held happen under the one lock, so two
    //           controls of one process cannot both add a hold for one point.
    // Why:      the refusal below is raised after the lock is let go, so a run that armed the same point twice reads
    //           that refusal and not a poisoned registry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const armed = withRegistry((holds) => holds.some((h) => h.point === point) ? false : (holds.push(held), true));
    // ```
    let armed: bool = {
        let mut registry = HELD_ON_ANY_THREAD.lock().expect("the hold registry");
        let duplicated: bool = registry.iter().any(|live| return live.point == point);
        if !duplicated {
            registry.push(held);
        }
        !duplicated
    };
    assert!(
        armed,
        "the fault point `{point}` is held twice in one process, and a hold answers the first thread that reaches \
         its point, so the two controls that armed it answer for each other; name the point with the value the \
         control owns (`collection_point`, `instance_read_point`), or run the body in a child of its own \
         (`process_wide_state_child_test_support`), as issue 6497's controls do"
    );
}

/// What:
///  Hold `point` on whichever thread reaches it,
/// giving the end that says the point was reached and the end that lets it go.
/// Why:
///   `hold_on_any_thread` with its two ends made is the one shape every control that holds a point needs,
/// and a control on a thread of its own can wait for the reach where it stands (`finished_within_at`),
/// which an async control does with a timeout instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reached, letGo] = holdThePoint('the-read-at-a-start');
/// ```
pub(crate) fn hold_the_point(
    point: &str
) -> (
    tokio::sync::oneshot::Receiver<()>,
    std::sync::mpsc::Sender<()>,
) {
    let (reached, reached_wait) = tokio::sync::oneshot::channel::<()>();
    let (let_go, held) = std::sync::mpsc::channel::<()>();
    hold_on_any_thread(HoldOnAnyThread {
        point: String::from(point),
        reached,
        let_go: held,
    });
    return (reached_wait, let_go);
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

/// What:
///  The holds the controls of one process wait at,
/// each taken by the task that reaches its point so a second reach passes.
/// Why:
///  A list and not one slot, for the reason `HELD_ON_ANY_THREAD` gives:
///  two controls of one module arm their points in one process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HELD_AWAITING: HoldAwaitingOnAnyThread[] = [];
/// ```
static HELD_AWAITING: std::sync::Mutex<Vec<HoldAwaitingOnAnyThread>> = std::sync::Mutex::new(Vec::new());

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
    let point: String = held.point.clone();
    // What:     the awaiting hold is armed under the same lock and with the same refusal as the thread's hold.
    // Why:      a run reads one answer for a point armed twice, whichever of the two kinds of hold it is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const armed = withRegistry((holds) => holds.some((h) => h.point === point) ? false : (holds.push(held), true));
    // ```
    let armed: bool = {
        let mut registry = HELD_AWAITING.lock().expect("the hold registry");
        let duplicated: bool = registry.iter().any(|live| return live.point == point);
        if !duplicated {
            registry.push(held);
        }
        !duplicated
    };
    assert!(
        armed,
        "the fault point `{point}` is held twice in one process, and a hold answers the first task that reaches \
         its point, so the two controls that armed it answer for each other; name the point with the value the \
         control owns, or run the body in a child of its own (`process_wide_state_child_test_support`), as issue \
         6497's controls do"
    );
}

/// What:
///  Waits at `point` while a control asked for it,
/// and says the point was reached.
/// Why:
///   A hold is taken when it is reached,
/// so a later reach of the same point passes and a control cannot block a second request.
/// A reach of another point leaves the holds where they are:
/// other points are reached by other tasks all through a run,
/// and one that took a hold and found another point would discard the control's hold before the point it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holdFor(point); if (!held) return; held.reached.send(); await held.letGo;
/// ```
pub(crate) async fn await_on_any_thread(point: &str) {
    let taken: Option<HoldAwaitingOnAnyThread> = {
        let mut registry = HELD_AWAITING.lock().expect("the hold registry");
        let found: Option<usize> = registry.iter().position(|held| return held.point == point);
        found.map(|index| return registry.remove(index))
    };
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
/// A reach of another point leaves the holds where they are,
/// for the reason `await_on_any_thread` gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holdFor(point); if (!held) return; held.reached.send(); held.letGo.recv();
/// ```
pub(crate) fn run_on_any_thread(point: &str) {
    let taken: Option<HoldOnAnyThread> = {
        let mut registry = HELD_ON_ANY_THREAD.lock().expect("the hold registry");
        let found: Option<usize> = registry.iter().position(|held| return held.point == point);
        found.map(|index| return registry.remove(index))
    };
    let Some(held) = taken else { return };
    let _said = held.reached.send(());
    let _let_go = held.let_go.recv();
}

/// The controls of the holds themselves: each answers the point its own control names, and one point armed twice in
///  one process is refused.
#[cfg(test)]
#[path = "service_fault_hooks_test_support_tests.rs"]
mod tests;
