//! What:
//! The scope of one run as this process remembers it:
//! the journal file the containment of its children is written to,
//! whether a process it started may still write,
//! whether it could only be contained by a process group,
//! the stack of open scopes of one thread,
//! and the command a timed-out run is told to repeat.
//! Why:
//! A scope is opened before cargo is started and closed when the value is dropped,
//! so a recovery reads the innermost open scope of the thread that runs it;
//! and a bound that passes must name the command to run again.
//! Split out of `bounded_run.rs`;
//! the items are re-exported from there,
//! so their paths are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded_run_scope.ts: interface ScopeState, class RunScope, setRerun(command), rerunText()
//! ```

/// What:
/// `use crate::bounded_run_containment::lose;` imports the mark that a run's writers may remain.
/// Why:
/// The one test-only entry point below reuses it instead of writing the flag itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { lose } from 'crate/bounded_run_containment';
/// ```
#[cfg(test)]
use crate::bounded_run_containment::lose;
/// What:
/// `use std::cell::{Cell, RefCell};` imports a box whose contents change through a shared reference:
/// `Cell` for a value that is simply read and replaced,
/// `RefCell` for one that is borrowed mutably.
/// Why:
/// The scope stack and the command to repeat live in a thread-local,
/// which only ever offers a shared reference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are mutable through any reference; no import is needed.
/// ```
use std::cell::{
    Cell,
    RefCell,
};
/// What:
/// `use std::marker::PhantomData;` imports the type that names a type parameter without holding a value of it.
/// Why:
/// A `RunScope` must not be sent to another thread,
/// and `PhantomData<*const ()>` states that without owning anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TypeScript has no thread-local ownership to state
/// ```
use std::marker::PhantomData;
/// What:
/// `use std::path::PathBuf;` imports the owned path type from the standard library so this file can name it unqualified.
/// Why:
/// A scope keeps the journal file's path for as long as it is open,
/// so it owns it;
/// the sibling `&Path` is a borrowed view.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::PathBuf;
/// What:
/// `use std::rc::Rc;` imports the reference-counted pointer type (its sibling `Arc` is the one that is safe to share between threads).
/// Why:
/// The stack of open scopes and the values handed to the containment both share one state,
/// inside one thread only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // every object reference in TS is shared and counted by the garbage collector
/// ```
use std::rc::Rc;

thread_local! {
    /// The command the user ran,
    /// for the error of a bound that was exceeded.
    static RERUN: RefCell<Option<String>> = const { RefCell::new(None) };

    /// The scopes of the open transactions of this thread,
    /// innermost last.
    static SCOPES: RefCell<Vec<(u64, Rc<ScopeState>)>> = const { RefCell::new(Vec::new()) };

    /// The number the next scope gets.
    static NEXT_SCOPE: Cell<u64> = const { Cell::new(0) };
}

/// What a scope remembers.
pub(crate) struct ScopeState {
    /// The file in which the containment of each child started under this scope is recorded.
    pub(crate) log: PathBuf,
    /// Whether a run under this scope could not show that everything it started was stopped.
    pub(crate) lost: Cell<bool>,
    /// Why it could not,
    /// in the words the refusal of the command carries:
    /// how long the stop waited and what it was waiting for.
    pub(crate) lost_reason: RefCell<Option<String>>,
    /// Whether a run under this scope was contained by a process group only.
    pub(crate) process_group_only: Cell<bool>,
}

/// The records and the verdict of one open transaction.
/// It belongs to the thread that opened it:
/// dropping it,
/// in any order,
/// removes only its own entry.
pub struct RunScope {
    /// The number of this scope in the stack.
    id: u64,
    /// What the scope remembers,
    /// shared with the runs under it.
    state: Rc<ScopeState>,
    /// Keeps the scope on its thread,
    /// because the stack it is in is thread-local.
    _on_one_thread: PhantomData<*const ()>,
}

/// The methods of the type above.
impl RunScope {
    /// What:
    /// `pub fn open(group_log: PathBuf) -> RunScope` makes the runs of this thread record their process groups in `group_log`.
    /// Why:
    /// A recovery that finds a killed run's journal needs to know whether a child of that run may still be running.
    /// The record belongs to the transaction that owns this scope,
    /// not to the thread (round 6,
    /// 5b2 6).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static open(groupLog: string): RunScope
    /// ```
    pub fn open(group_log: PathBuf) -> RunScope {
        let id: u64 = NEXT_SCOPE.get();
        NEXT_SCOPE.set(id + 1);
        let state: Rc<ScopeState> = Rc::new(ScopeState {
            log: group_log,
            lost: Cell::new(false),
            lost_reason: RefCell::new(None),
            process_group_only: Cell::new(false),
        });
        // The stack is taken out of its cell and put back, so no closure holds the borrow; nothing else runs in between.
        let mut stack: Vec<(u64, Rc<ScopeState>)> = SCOPES.take();
        stack.push((id, Rc::clone(&state)));
        SCOPES.set(stack);
        return RunScope {
            id,
            state,
            _on_one_thread: PhantomData,
        };
    }

    /// What:
    /// `pub fn writers_may_remain(&self) -> bool` says whether a run under this scope could not show that everything it
    /// started was stopped.
    /// Why:
    /// Then the files of the transaction cannot be rolled back or finished safely,
    /// and the command must stop.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// writersMayRemain(): boolean
    /// ```
    pub fn writers_may_remain(&self) -> bool {
        return self.state.lost.get();
    }

    /// What:
    /// `pub fn writers_may_remain_reason(&self) -> Option<String>` is why the run could not show that everything it started
    /// was stopped,
    /// in the words the refusal carries.
    /// Why:
    /// Issue 6528:
    /// a reader of the refusal has to be able to tell a member that has exited from one that is alive,
    /// and a bound that passed on a loaded host from a member that cannot be killed,
    /// so the message says how long the stop waited and what it was waiting for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// writersMayRemainReason(): string | undefined
    /// ```
    pub fn writers_may_remain_reason(&self) -> Option<String> {
        return self.state.lost_reason.borrow().clone();
    }

    /// What:
    /// `pub fn process_group_only(&self) -> bool` says whether a run under this scope was contained by a process group only,
    /// because no control group could be had.
    /// Why:
    /// Quiescence was not shown for what leaves a process group,
    /// and a caller may say so.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// processGroupOnly(): boolean
    /// ```
    pub fn process_group_only(&self) -> bool {
        return self.state.process_group_only.get();
    }
}

/// How the scope ends.
impl Drop for RunScope {
    /// What:
    /// `fn drop(&mut self)` removes this scope's entry and no other.
    /// Why:
    /// Transactions may nest and end in any order.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        // Taken out of its cell and put back: nothing else runs in between.
        let mut stack: Vec<(u64, Rc<ScopeState>)> = SCOPES.take();
        let mut index: usize = 0;
        while index < stack.len() {
            if stack[index].0 == self.id {
                stack.remove(index);
            } else {
                index += 1;
            }
        }
        SCOPES.set(stack);
    }
}

/// What:
/// `fn scope_state_of(entry: &(u64, Rc<ScopeState>)) -> Rc<ScopeState>` clones the state of one stack entry.
/// Why:
/// `Option::map` takes it by path instead of a closure.
fn scope_state_of(entry: &(u64, Rc<ScopeState>)) -> Rc<ScopeState> {
    return Rc::clone(&entry.1);
}

/// What:
/// `pub(crate) fn current_scope() -> Option<Rc<ScopeState>>` is the innermost open scope of this thread.
/// Why:
/// The run records into,
/// and reports to,
/// the transaction that is open around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentScope(): ScopeState | undefined
/// ```
pub(crate) fn current_scope() -> Option<Rc<ScopeState>> {
    // Taken out of its cell and put back: nothing else runs in between.
    let stack: Vec<(u64, Rc<ScopeState>)> = SCOPES.take();
    let innermost: Option<Rc<ScopeState>> = stack.last().map(scope_state_of);
    SCOPES.set(stack);
    return innermost;
}

/// What:
/// `pub fn mark_scope_lost_for_test()` marks the innermost open scope of this thread as one whose writers may remain.
/// Why:
/// Tests of the callers need the verdict a run gives when a descendant cannot be stopped,
/// which a real run cannot be made to give.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function markScopeLostForTest(): void
/// ```
#[cfg(test)]
pub fn mark_scope_lost_for_test() {
    // What:     `Some("...")` is the present variant of `Option`, wrapping the reason a stand-in stop gives; a real stop's reason
    //           names the time it waited and the members it was left with, and this one says it stands in for such a stop.
    // Why:      A caller of this test entry point reads a reason like every caller of a real stop, so it cannot pass on an empty one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // lose(currentScope(), 'a test marked the scope as one whose writers may remain')
    // ```
    lose(
        &current_scope(),
        "a test marked the scope as one whose writers may remain".to_string(),
    );
}

/// What:
/// `pub fn set_rerun(command: Option<String>)` names the command the user ran,
/// for the errors of runs started by this thread.
/// Why:
/// An error that says "run it again" must say which command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function setRerun(command?: string): void
/// ```
pub fn set_rerun(command: Option<String>) {
    RERUN.set(command);
}

/// What:
/// `pub fn rerun_text() -> String` is the text for the command to run again.
/// Why:
/// Bounds are built far from the shim that knows the arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function rerunText(): string
/// ```
pub fn rerun_text() -> String {
    // Taken out of its cell and put back: nothing else runs in between.
    let held: Option<String> = RERUN.take();
    let text: String = match held.clone() {
        Some(command) => format!("`{command}`"),
        None => "the command you ran".to_string(),
    };
    RERUN.set(held);
    return text;
}
