//! What:
//!  The state one landing keeps while it runs,
//! carried by the task (or the one thread,
//! in the synchronous service loop) that runs the landing,
//! never by the thread the code happens to be on.
//! Why:
//!  The owner chose "Rewrite per-thread state" (`doc/planning/git-policy-cli-full-tokio.md`,
//! "The owner's answers at 2026-11-07",
//! choice 4):
//! each `thread_local!` of the service's landing becomes a value that travels with its task,
//! because on a multi-thread runtime a task changes worker thread at every `.await`
//! and a thread-local then reads another worker's copy,
//! while two tasks that interleave on one thread would overwrite one operation's state with another's.
//! One owner per piece of state,
//! no lock:
//! the values are `Cell`s and `RefCell`s that only the task that owns the scope touches,
//! and a `RefCell` is `Send`,
//! so a task that holds them can move between workers.
//! Outside a scope (the wrapper's own commands,
//! the service's other threads,
//! a task that was spawned without one)
//! nothing is remembered and every read answers its default,
//! which is what a thread-local gave a thread that never began a landing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const landing = new AsyncLocalStorage<LandingState>();
//! await landing.run(new LandingState(), () => landTheBatch());
//! ```

use crate::service_busy::LandingBoard;
use crate::service_durable::Pending;
use crate::service_index_prestage::Slot;
use crate::service_index_refused_lock::Refused;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::sync::Arc;

tokio::task_local! {
    /// The state of the landing the current task (or the scoped synchronous call) runs.
    static LANDING: LandingState;
}

/// What one landing remembers,
/// each item named for the module that owns its meaning.
pub struct LandingState {
    /// The board of the service whose landing this is (`service_busy_step_record_on_this_thread.rs`).
    pub(crate) board: RefCell<Option<Arc<LandingBoard>>>,
    /// What the landing knows about the next write that is never ended,
    /// for the words that name it.
    pub(crate) detail: RefCell<Option<String>>,
    /// The sentence of the last Git step this landing ended at its ceiling (`service_land_git_steps_and_import_stream.rs`).
    pub(crate) ceiling_step: RefCell<Option<String>>,
    /// The claims this landing placed that could not be read back,
    /// with the marker text each was written from
    /// (`service_index_claim.rs`).
    pub(crate) abandoned_claims: RefCell<Vec<(PathBuf, String)>>,
    /// The lock this landing last gave up on (`service_index_refused_lock.rs`).
    pub(crate) refused_lock: RefCell<Option<Refused>>,
    /// How many locks this landing's holders could not settle (`service_index_ownership.rs`).
    pub(crate) unsettled_locks: Cell<usize>,
    /// How many name comparisons the proofs of this landing made (`service_built_checks.rs`),
    /// counted by controls only.
    #[cfg(test)]
    pub(crate) name_steps: Cell<u64>,
    /// The one marker made ahead for the next batch (`service_index_prestage.rs`).
    pub(crate) prestaged_marker: RefCell<Option<Slot>>,
    /// Whether this landing queues its syncs (`true`) or makes them where it asks (`false`) (`service_durable.rs`).
    pub(crate) deferring_syncs: Cell<bool>,
    /// Whether the index this landing writes is flushed before it is installed.
    pub(crate) index_hardened: Cell<bool>,
    /// The syncs this landing has queued so far.
    pub(crate) queued_syncs: RefCell<Vec<Pending>>,
    /// Every sync this landing made or handed to the background so far,
    /// in the order it asked for them (`service_durable.rs`).
    pub(crate) syncs: RefCell<Vec<crate::service_durable_syncs::SyncFigure>>,
}

impl LandingState {
    /// The state of a landing that has done nothing yet:
    /// no board,
    /// nothing remembered,
    /// syncs made where they are asked for,
    /// and the index hardened.
    pub(crate) fn new() -> Self {
        return Self {
            board: RefCell::new(None),
            detail: RefCell::new(None),
            ceiling_step: RefCell::new(None),
            abandoned_claims: RefCell::new(Vec::new()),
            refused_lock: RefCell::new(None),
            unsettled_locks: Cell::new(0),
            #[cfg(test)]
            name_steps: Cell::new(0),
            prestaged_marker: RefCell::new(None),
            deferring_syncs: Cell::new(false),
            index_hardened: Cell::new(true),
            queued_syncs: RefCell::new(Vec::new()),
            syncs: RefCell::new(Vec::new()),
        };
    }
}

/// What:
///  Runs `body` with a fresh landing state that `on_the_landing_state` reaches from inside it,
/// on the calling thread.
/// Why:
///  The synchronous service loop is the one landing today;
/// it enters the scope once and everything it calls on its thread shares the state.
/// A scope inside a scope shadows the outer one,
/// so two operations on one thread never share an item.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landing.run(new LandingState(), body)
/// ```
pub fn in_a_landing<Output>(body: impl FnOnce() -> Output) -> Output {
    return LANDING.sync_scope(LandingState::new(), body);
}

/// What:
///  `in_a_landing` for a body that is a named function and the state it reads.
/// Why:
///  A closure that captures state is reported by the closure lint unless its callee needs it past the call;
/// `LocalKey::sync_scope` accepts only a callable,
/// so this one function carries the closure
/// (`package/git-policy/cli/doc/closure-lint-suppressions.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landing.run(new LandingState(), () => body(arguments))
/// ```
pub fn in_a_landing_with<Arguments, Output>(arguments: Arguments, body: fn(Arguments) -> Output) -> Output {
    #[cfg_attr(
        dylint_lib = "monochromatic_lint_closure",
        expect(
            avoidable_closure,
            reason = "`tokio::task::LocalKey::sync_scope` accepts only a callable with no arguments, and it must carry the arguments and the function; see package/git-policy/cli/doc/closure-lint-suppressions.md"
        )
    )]
    let output: Output = LANDING.sync_scope(LandingState::new(), move || { return body(arguments); });
    return output;
}

/// What:
///  Awaits `future` with a fresh landing state that follows the future wherever the runtime moves it.
/// Why:
///  The state belongs to the task,
/// not to the worker thread it runs on at the moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await landing.run(new LandingState(), () => future)
/// ```
pub async fn in_a_landing_task<Output>(future: impl std::future::Future<Output = Output>) -> Output {
    return LANDING.scope(LandingState::new(), future).await;
}

/// A state that belongs to no landing:
/// what a read outside a landing is given.
fn state_of_no_landing() -> LandingState {
    return LandingState::new();
}

/// What:
///  Applies `apply` to the state of the landing this code runs in and `arguments`;
/// outside any landing it applies it to a fresh state that is dropped afterwards.
/// Why:
///  A read outside a landing answers the default and a write is not kept:
/// the wrapper's own commands and the service's other threads never began a landing,
/// as a thread that never wrote a thread-local read its initial value.
/// A named function and its arguments,
/// not a closure,
/// because the closure lint asks for that;
/// `LocalKey::try_with` accepts only a callable,
/// so this one function carries the closure
/// (`package/git-policy/cli/doc/closure-lint-suppressions.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// apply(landing.getStore() ?? new LandingState(), arguments)
/// ```
pub fn on_the_landing_state<Arguments, Output>(arguments: Arguments, apply: fn(&LandingState, Arguments) -> Output) -> Output {
    // The arguments wait in an `Option`: whichever of the two paths runs takes them.
    let mut waiting: Option<Arguments> = Some(arguments);
    #[cfg_attr(
        dylint_lib = "monochromatic_lint_closure",
        expect(
            avoidable_closure,
            reason = "`tokio::task::LocalKey::try_with` accepts only a callable taking the value, and it must carry the arguments and the function; see package/git-policy/cli/doc/closure-lint-suppressions.md"
        )
    )]
    let inside: Option<Output> = LANDING
        .try_with(|state| {
            let Some(carried) = waiting.take() else { unreachable!("the closure runs once") };
            return apply(state, carried);
        })
        .ok();
    return match inside {
        Some(output) => output,
        None => {
            let Some(carried) = waiting.take() else { unreachable!("a missed scope never ran the closure") };
            apply(&state_of_no_landing(), carried)
        }
    };
}

/// The controls' way to the state of the landing:
/// a closure over it,
/// where the production code passes a named function.
#[cfg(test)]
pub fn with_the_landing_state<Output, Use: FnOnce(&LandingState) -> Output>(use_state: Use) -> Output {
    // The closure waits in an `Option` for whichever of the two paths runs.
    let mut waiting: Option<Use> = Some(use_state);
    let inside: Option<Output> = LANDING
        .try_with(|state| {
            let Some(use_it) = waiting.take() else { unreachable!("the closure runs once") };
            return use_it(state);
        })
        .ok();
    return match inside {
        Some(output) => output,
        None => {
            let Some(use_it) = waiting.take() else { unreachable!("a missed scope never ran the closure") };
            use_it(&state_of_no_landing())
        }
    };
}

/// Whether the caller runs inside a landing's scope.
pub fn inside_a_landing() -> bool {
    return LANDING.try_with(ignore_the_state).is_ok();
}

/// Does nothing with a landing's state;
/// `inside_a_landing` only asks whether there is one.
fn ignore_the_state(_state: &LandingState) {}

/// The controls of the landing's scope.
#[cfg(test)]
#[path = "landing_state_carried_by_its_task_tests.rs"]
mod tests;
