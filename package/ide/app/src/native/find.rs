//! Native in-file find:
//!  bar lifecycle,
//!  worker polling,
//!  and selection-based match navigation.

/// One window and one source owner;
///  the find worker never receives either.
use super::{AppWindow, State};
/// What:
///  `Result` carries either the started binding or a startup error.
/// Why:
///  A window without a find worker must report that at startup,
///  not offer a dead shortcut.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // bind() throws when the worker thread cannot start
/// ```
use anyhow::Result;
/// Requests and replies are tagged by file generation,
///  content revision,
///  and query generation.
use ide_app::find_worker::{FindIdentity, FindWorker};
/// Weak window handles and a retained timer connect worker replies to the native event loop.
use slint::{ComponentHandle, Timer, TimerMode};
/// What:
///  `Rc` shares one owner on this thread;
///  `RefCell` checks mutable borrows at run time.
/// Why:
///  Four callbacks and one timer change the same find state without cross-thread locking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const find = { current: createFind() };
/// ```
use std::{cell::RefCell, rc::Rc, time::Duration};

/// Count text and active match derived from accepted results and the selection.
pub(super) mod present;
/// Open,
///  edit,
///  navigate,
///  and close transitions.
mod session;
/// Identity synchronization with the displayed document and reply application.
mod tick;

/// What:
///  Window-local find state;
///  `u64` is a fixed 64-bit counter (sibling `usize` is pointer-sized).
/// Why:
///  Matches live in the source state for painting;
///  this record owns only the request lifecycle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Find = { worker: FindWorker; open: boolean; query: string; queryGeneration: number;
///   requested?: FindIdentity; seek: boolean; available: boolean };
/// ```
pub(super) struct Find {
    /// One background job and one waiting request;
    ///  joined when this state is dropped.
    worker: FindWorker,
    /// True between Ctrl+F and Escape.
    open: bool,
    /// Latest find text,
    ///  kept across closing so reopening offers it selected.
    query: String,
    /// Incremented on every edit;
    ///  part of every request tag.
    query_generation: u64,
    /// Identity last handed to the worker;
    ///  a different displayed identity triggers a new request.
    requested: Option<FindIdentity>,
    /// An edit asks the next accepted reply to select the first match at or after the selection start.
    seek: bool,
    /// False after the worker stopped unexpectedly,
    ///  so its diagnostic is reported once.
    available: bool,
}

/// Bind the find callbacks and poll the worker every 20 ms.
/// The returned timer must stay alive until the window closes;
///  dropping all owners joins the worker.
pub(super) fn bind(owner: &AppWindow, source: &Rc<RefCell<State>>) -> Result<Timer> {
    // What: `?` returns a worker-start failure; `Rc::new(RefCell::new(...))` creates the shared owner.
    // Why: Every callback below needs the same find state after this function returns.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const find = { current: { worker: new FindWorker(), open: false, query: '', ... } };
    // ```
    let find = Rc::new(RefCell::new(Find {
        worker: FindWorker::new()?,
        open: false,
        query: String::new(),
        query_generation: 0,
        requested: None,
        seek: false,
        available: true,
    }));

    // What: `Rc::clone` copies the pointer; `as_weak` gives a handle that does not keep the window alive.
    // Why: The stored callback outlives this function but must not prevent window shutdown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const openFind = find; const openWindow = new WeakRef(owner);
    // ```
    let open_find = Rc::clone(&find);
    let open_source = Rc::clone(source);
    let open_window = owner.as_weak();
    // What: `move ||` transfers the captured handles into the stored callback.
    // Why: Borrowed locals would not outlive this binding function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.onFindRequest(() => session.open(window, source, find.current));
    // ```
    owner.on_find_request(move || {
        // What: `upgrade` returns `Some(window)` only while the window still exists.
        // Why: A late callback during shutdown must do nothing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const window = openWindow.deref(); if (!window) return;
        // ```
        let Some(window) = open_window.upgrade() else {
            return;
        };
        // What: `borrow_mut` lends the find state mutably for this call; `&mut` passes that loan on.
        // Why: Only one transition changes find state at a time.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // session.open(window, source, find.current);
        // ```
        session::open(&window, &open_source, &mut open_find.borrow_mut());
    });

    let edit_find = Rc::clone(&find);
    let edit_source = Rc::clone(source);
    let edit_window = owner.as_weak();
    owner.on_find_edited(move |raw| {
        let Some(window) = edit_window.upgrade() else {
            return;
        };
        session::edit(&window, &edit_source, &mut edit_find.borrow_mut(), &raw);
    });

    let step_find = Rc::clone(&find);
    let step_source = Rc::clone(source);
    let step_window = owner.as_weak();
    owner.on_find_navigate(move |delta| {
        let Some(window) = step_window.upgrade() else {
            return;
        };
        session::navigate(&window, &step_source, &step_find.borrow(), delta);
    });

    let close_find = Rc::clone(&find);
    let close_source = Rc::clone(source);
    let close_window = owner.as_weak();
    owner.on_find_dismiss(move || {
        let Some(window) = close_window.upgrade() else {
            return;
        };
        session::close(&window, &close_source, &mut close_find.borrow_mut());
    });

    let timer = Timer::default();
    let tick_source = Rc::clone(source);
    let tick_window = owner.as_weak();
    timer.start(TimerMode::Repeated, Duration::from_millis(20), move || {
        if let Some(window) = tick_window.upgrade() {
            tick::update(&window, &tick_source, &mut find.borrow_mut());
        }
    });
    // What: `Ok(timer)` hands the running timer to the caller as the success value.
    // Why: The caller keeps it alive for the lifetime of the window.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return timer;
    // ```
    return Ok(timer);
}
