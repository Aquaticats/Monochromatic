//! What:
//! The trace of a run and the hooks a test drives inside one:
//! the ordered events of this thread,
//! the flag that lets the weaker path print its notice once per process,
//! the hooks that run inside a start,
//! after the launcher was spawned,
//! and after the program started,
//! and the route override that makes the host look like one without a capability.
//! Why:
//! The order record write,
//! durability barrier,
//! launch is a property of the real boundaries,
//! so the tests read an ordered trace of them;
//! and every hook exists in a test build only,
//! so a production build never computes what one would do.
//! Split out of `bounded_run.rs`;
//! the items are re-exported from there,
//! so their paths are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded_run_trace_and_hooks.ts: trace(event), takeTraceForTest(), onStartForTest(hook), onStartupForTest(hook)
//! ```

/// What:
/// `use crate::control_group::RouteSettings;` imports the settings the route is discovered with.
/// Why:
/// The override holds a settings value,
/// so a test can make the host look like one that lacks a capability.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RouteSettings } from 'crate/control_group';
/// ```
#[cfg(test)]
use crate::control_group::RouteSettings;
/// What:
/// `use std::cell::RefCell;` imports a box whose contents can be changed through a shared reference (sibling:
/// `Cell`,
/// for values that are simply read and replaced).
/// Why:
/// The hooks and the trace live in thread-locals,
/// which only ever offer a shared reference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are mutable through any reference; no import is needed.
/// ```
#[cfg(test)]
use std::cell::RefCell;
/// What:
/// `use std::rc::Rc;` imports the reference-counted pointer type (its sibling `Arc` is the one that is safe to share between threads).
/// Why:
/// A hook is a callback the test and the run both hold,
/// inside one thread only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // every object reference in TS is shared and counted by the garbage collector
/// ```
#[cfg(test)]
use std::rc::Rc;

/// Makes the one-line notice of the weaker path appear once per process.
/// It is a flag that one caller wins with a swap,
/// not a `Once`:
/// a caller that loses returns at once,
/// where `Once::call_once` would make
/// it wait,
/// with no deadline,
/// for the winner's write to standard error (fifth certification pass,
/// V6:
/// no wait the runner controls may
/// depend on another run's slow write).
pub(crate) static WEAKER_NOTICE_GIVEN: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// A test holds this lock to keep the winner of the notice inside its emission,
/// so that another run can show it does not wait for it.
#[cfg(test)]
pub(crate) static NOTICE_GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Set when the winner of the notice reached its emission (a test build only).
#[cfg(test)]
pub(crate) static NOTICE_ENTERED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
thread_local! {
    /// The settings the route is discovered with instead of the ordinary ones,
    /// so a test can make the host look like one that lacks a capability.
    pub(crate) static ROUTE_OVERRIDE: RefCell<Option<RouteSettings>> = const { RefCell::new(None) };

    /// A hook that runs inside the start of a run,
    /// after the scope's record is on disk and before the program starts,
    /// so a test can overlap other work with the start at a boundary of its own choosing.
    pub(crate) static START_HOOK: RefCell<Option<Rc<dyn Fn()>>> = const { RefCell::new(None) };

    /// A hook that runs right after the scope's launcher was spawned and before the launcher record is appended,
    /// with the launcher's process id,
    /// so a test can kill the wrapper inside that window.
    pub(crate) static AFTER_LAUNCH_HOOK: RefCell<Option<Rc<dyn Fn(u32)>>> = const { RefCell::new(None) };

    /// A hook that runs right after the program has started and decides whether the completion of the startup fails:
    /// it returns
    /// `true` to make it fail,
    /// after the test has waited for whatever it needs to be ready (a writer,
    /// for one).
    pub(crate) static STARTUP_HOOK: RefCell<Option<Rc<dyn Fn() -> bool>>> = const { RefCell::new(None) };

    /// The ordered events of the recording and the launching of the runs of this thread,
    /// for the tests of their order.
    static TRACE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// What:
/// `fn trace(event: fmt::Arguments<'_>)` appends an event to this thread's trace in a test build and does nothing otherwise.
/// Why:
/// The order record write,
/// durability barrier,
/// launch is a property of the real boundaries,
/// so the tests read an ordered trace of
/// them (certification group 2).
/// The event is a `format_args!` value:
/// nothing is formatted unless a test build turns it into text,
/// and a production build drops it unread.
/// An event whose arguments would compute something (read a file,
/// call a function with a cost) is not passed here;
/// its statement carries `#[cfg(test)]` and calls `trace_text`,
/// so a production build never evaluates it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trace(event: string): void  // the argument is a template that is only rendered in a test build
/// ```
#[cfg(test)]
pub(crate) fn trace(event: std::fmt::Arguments<'_>) {
    trace_text(event.to_string());
}

/// What:
/// `fn trace_text(text: String)` appends an already formatted event to this thread's trace.
/// Why:
/// The statements that compute their text are compiled in a test build only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function traceText(text: string): void
/// ```
#[cfg(test)]
pub(crate) fn trace_text(text: String) {
    TRACE.with(|slot| {
        slot.borrow_mut().push(text);
    });
}

/// The production build keeps no trace.
#[cfg(not(test))]
pub(crate) fn trace(_event: std::fmt::Arguments<'_>) {}

/// What:
/// `pub(crate) fn take_trace_for_test() -> Vec<String>` returns and clears this thread's trace.
/// Why:
/// A test reads the events of the run it just made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function takeTraceForTest(): string[]
/// ```
#[cfg(test)]
pub(crate) fn take_trace_for_test() -> Vec<String> {
    return TRACE.with(|slot| {
        return std::mem::take(&mut *slot.borrow_mut());
    });
}

/// What:
/// `pub(crate) fn on_startup_for_test(hook: Option<Rc<dyn Fn() -> bool>>)` sets the hook that runs after the program started and may
/// make the completion of the startup fail.
/// Why:
/// The row A2/R4 requires a failure after the child and a writer are running to be as conservative as a failed stop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function onStartupForTest(hook?: () => boolean): void
/// ```
#[cfg(test)]
pub(crate) fn on_startup_for_test(hook: Option<Rc<dyn Fn() -> bool>>) {
    STARTUP_HOOK.with(|slot| {
        *slot.borrow_mut() = hook;
    });
}

/// What:
/// `pub fn on_start_for_test(hook: Option<Rc<dyn Fn()>>)` sets the hook that runs inside the start of each run of this thread.
/// Why:
/// The spawn-isolation tests overlap an unrelated spawn and a second run with the start of the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function onStartForTest(hook?: () => void): void
/// ```
#[cfg(test)]
pub(crate) fn on_start_for_test(hook: Option<Rc<dyn Fn()>>) {
    START_HOOK.with(|slot| {
        *slot.borrow_mut() = hook;
    });
}

/// What:
/// `pub fn after_launch_for_test(hook: Option<Rc<dyn Fn(u32)>>)` sets the hook that runs after the scope's launcher was spawned and
/// before its record is appended,
/// with the launcher's process id.
/// Why:
/// The wrapper's pending-launcher window is real,
/// and a test kills a real wrapper inside it (sixth pass,
/// group 12).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function afterLaunchForTest(hook?: (pid: number) => void): void
/// ```
#[cfg(test)]
pub(crate) fn after_launch_for_test(hook: Option<Rc<dyn Fn(u32)>>) {
    AFTER_LAUNCH_HOOK.with(|slot| {
        *slot.borrow_mut() = hook;
    });
}

/// The guard that a production build never computes a trace event.
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "bounded_run_trace_guard_tests.rs"]
mod bounded_run_trace_guard_tests;
