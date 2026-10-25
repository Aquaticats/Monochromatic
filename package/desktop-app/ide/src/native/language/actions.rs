//! Window callbacks:
//!  Ctrl+B,
//!  Ctrl+Q,
//!  Ctrl+click,
//!  the resting pointer,
//!  the list,
//!  and Escape.

/// The state,
///  the request record,
///  and the steps the callbacks start.
use super::{Action, Language, Pending, message, outcome, pointer, surface};
/// The window,
///  the source,
///  and the navigation that opens other files.
use crate::native::{AppWindow, State, navigation::Navigation};
/// Weak window handles keep callbacks from holding a closed window alive.
use slint::ComponentHandle;
/// What:
///  `Rc<RefCell<T>>` is the window's shared,
///  borrow-checked state.
/// Why:
///  Every callback changes the same language state the timer polls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

/// What:
///  Start an explicit action at `position`:
///  dismiss what is shown and queue the request.
/// Why:
///  A new action replaces the previous one,
///  whose late replies are then dropped by number.
///      Without a running worker the reason is shown at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function start(window, source, language, action: Action, position: number): void
/// ```
fn start(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    language: &mut Language,
    action: Action,
    position: usize,
) {
    if !window.get_source_available() || source.borrow().file_path.is_none() {
        tracing::debug!(
            ?action,
            "ignored a language action without a displayed file"
        );
        return;
    }
    language.shown = surface::dismiss(window, &language.shown, "a new language action started");
    if language.worker.is_none() {
        // `as_deref` lends the stored reason; `unwrap_or` names an unknown one.
        let reason = language
            .failure
            .as_deref()
            .unwrap_or("Language support is not running");
        let sentence = message::stopped(reason);
        language.shown = surface::popup(window, source, position, &sentence, (true, false));
        return;
    }
    if message::needs_reopen(&language.status) {
        tracing::info!(
            ?action,
            "displaying the file again so its servers are resolved and started anew"
        );
        language.reopen = true;
    }
    let stamp = surface::displayed(&source.borrow());
    tracing::debug!(?action, position, ?stamp, "language action started");
    // `Some(Pending { ... })` replaces any earlier waiting action.
    language.action = Some(Pending {
        action,
        stamp,
        position,
        number: None,
        outcomes: Vec::new(),
        complete: false,
    });
}

/// What:
///  Lend the language state for writing,
///  unless another callback holds it right now.
///  The result is the borrow,
///  or nothing.
/// Why:
///  A window event delivered while the state is held is skipped and logged,
///  never a panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function held(language: Shared<Language>): Language | undefined
/// ```
fn held(language: &Rc<RefCell<Language>>) -> Option<RefMut<'_, Language>> {
    // `try_borrow_mut` fails instead of panicking when the state is already lent.
    let Ok(current) = language.try_borrow_mut() else {
        tracing::debug!("skipped a language callback while the state was held");
        return None;
    };
    return Some(current);
}

/// What:
///  Bind the key callback of an action that starts at the caret:
///  Ctrl+Q for hover,
///       Ctrl+B for definition.
/// Why:
///  The two keys differ only in the action they start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bindCaretAction(owner, source, language, action: Action): void
/// ```
fn bind_caret(
    owner: &AppWindow,
    source: &Rc<RefCell<State>>,
    language: &Rc<RefCell<Language>>,
    action: Action,
) {
    let state = Rc::clone(source);
    let shared = Rc::clone(language);
    let weak = owner.as_weak();
    // `move ||` hands the captured handles to the stored callback.
    // The Slint setters below keep the callback for the life of the window, so it has to own its state.
    // An attribute cannot sit on the initializer of a `let`, so it sits on the statement.
    #[cfg_attr(
        dylint_lib = "monochromatic_lint_closure",
        expect(
            avoidable_closure,
            reason = "the Slint on_hover_request and on_definition_request setters keep the callback after the call, so it must own the weak window handle and the shared state"
        )
    )]
    let callback = move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let head = state.borrow().document.position().head;
        if let Some(mut current) = held(&shared) {
            start(&window, &state, &mut current, action, head);
        }
    };
    if action == Action::Hover {
        owner.on_hover_request(callback);
    } else {
        owner.on_definition_request(callback);
    }
}

/// Bind Ctrl+click and the resting pointer.
fn bind_pointer(owner: &AppWindow, source: &Rc<RefCell<State>>, language: &Rc<RefCell<Language>>) {
    let click_state = Rc::clone(source);
    let click_language = Rc::clone(language);
    let click_window = owner.as_weak();
    owner.on_definition_at_pointer(move |y, x| {
        let Some(window) = click_window.upgrade() else {
            return;
        };
        // A Ctrl+click past the end of a line or on the line numbers asks nothing, as in editord.
        let Some(character) = pointer::character_at(&click_state.borrow(), y, x) else {
            tracing::debug!(y, x, "Ctrl+click over no source character");
            return;
        };
        if let Some(mut current) = held(&click_language) {
            start(
                &window,
                &click_state,
                &mut current,
                Action::PointerDefinition,
                character,
            );
        }
    });
    let move_state = Rc::clone(source);
    let move_language = Rc::clone(language);
    let move_window = owner.as_weak();
    owner.on_pointer_moved(move |y, x| {
        let Some(window) = move_window.upgrade() else {
            return;
        };
        let character = pointer::character_at(&move_state.borrow(), y, x);
        // The point is kept as a distance from the top of the view,
        //  which stays the pointer's own while the view scrolls under it.
        let offset = window.get_drawn_offset();
        let spot = pointer::Spot {
            below_top: y - offset,
            x,
            offset,
            extent: window.get_content_extent(),
            text: surface::displayed(&move_state.borrow()),
        };
        if let Some(mut current) = held(&move_language) {
            current.rest.moved(character, spot);
        }
    });
    let exit_language = Rc::clone(language);
    owner.on_pointer_exited(move || {
        if let Some(mut current) = held(&exit_language) {
            current.rest.left();
        }
    });
}

/// Bind Escape,
///  closing the list,
///  and choosing from it.
fn bind_surfaces(
    owner: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &Rc<RefCell<Navigation>>,
    language: &Rc<RefCell<Language>>,
) {
    let escape_language = Rc::clone(language);
    let escape_window = owner.as_weak();
    owner.on_language_dismiss(move || {
        let Some(window) = escape_window.upgrade() else {
            return;
        };
        if let Some(mut current) = held(&escape_language) {
            current.shown = surface::dismiss(&window, &current.shown, "Escape");
        }
    });
    let close_language = Rc::clone(language);
    let close_window = owner.as_weak();
    owner.on_references_dismiss(move || {
        let Some(window) = close_window.upgrade() else {
            return;
        };
        if let Some(mut current) = held(&close_language) {
            current.shown = surface::dismiss(&window, &current.shown, "the list was closed");
        }
    });
    let choose_state = Rc::clone(source);
    let choose_navigation = Rc::clone(navigation);
    let choose_language = Rc::clone(language);
    let choose_window = owner.as_weak();
    owner.on_references_choose(move |index| {
        let Some(window) = choose_window.upgrade() else {
            return;
        };
        if let Some(mut current) = held(&choose_language) {
            outcome::choose(
                &window,
                &choose_state,
                &choose_navigation,
                &mut current,
                index,
            );
        }
    });
}

/// Bind the key,
///  pointer,
///  and list callbacks of the language features.
pub(super) fn bind(
    owner: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &Rc<RefCell<Navigation>>,
    language: &Rc<RefCell<Language>>,
) {
    bind_caret(owner, source, language, Action::Definition);
    bind_caret(owner, source, language, Action::Hover);
    bind_pointer(owner, source, language);
    bind_surfaces(owner, source, navigation, language);
}
