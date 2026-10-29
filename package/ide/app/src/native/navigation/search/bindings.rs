//! Global shortcut capture and modal callbacks share the existing native navigation owner.

/// One navigation owner supplies project scope and safe source opening.
use super::super::{AppWindow, Navigation, State, open};
/// Session transitions invalidate worker generations before changing presentation.
use super::session;
/// Weak windows avoid retaining an invisible native window after shutdown.
use slint::{ComponentHandle, SharedString};
/// Each callback clones UI-thread ownership rather than sharing mutable source with workers.
use std::{cell::RefCell, rc::Rc};

/// Bind once;
///  native input continues to its normal widget unless a completed double-Shift opens search.
pub(in crate::native::navigation) fn bind(
    owner: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &Rc<RefCell<Navigation>>,
) {
    let focus_navigation = Rc::clone(navigation);
    owner.on_tree_user_focus(move |index| {
        session::remember(&mut focus_navigation.borrow_mut(), index);
    });

    let shortcut_navigation = Rc::clone(navigation);
    let shortcut_window = owner.as_weak();
    owner.on_shortcut_observed(move |key, pressed| {
        let Some(window) = shortcut_window.upgrade() else {
            return false;
        };
        let mut active_navigation = shortcut_navigation.borrow_mut();
        let is_shift = SharedString::from(slint::platform::Key::Shift) == key;
        if pressed {
            active_navigation.search.gesture.press(is_shift);
            return false;
        }
        let elapsed = active_navigation.search.clock.elapsed();
        if !active_navigation.search.gesture.release(is_shift, elapsed) {
            return false;
        }
        if let Err(error) = session::start(&window, &mut active_navigation) {
            tracing::warn!(%error, "cannot open native search");
            window.set_search_error(format!("{error:#}").into());
        }
        return true;
    });

    let edit_navigation = Rc::clone(navigation);
    let edit_window = owner.as_weak();
    owner.on_search_edited(move |raw| {
        let Some(window) = edit_window.upgrade() else {
            return;
        };
        if !window.get_search_open() {
            return;
        }
        if let Err(error) = session::edit(&window, &mut edit_navigation.borrow_mut().search, &raw) {
            tracing::warn!(%error, "cannot edit native search request");
            window.set_search_error(format!("{error:#}").into());
        }
    });

    let close_navigation = Rc::clone(navigation);
    let close_source = Rc::clone(source);
    let close_window = owner.as_weak();
    owner.on_search_dismiss(move || {
        let Some(window) = close_window.upgrade() else {
            return;
        };
        if let Err(error) = session::close(&window, &mut close_navigation.borrow_mut().search) {
            open::failed(&window, &close_source, format!("{error:#}"));
        }
    });

    let choose_navigation = Rc::clone(navigation);
    let choose_source = Rc::clone(source);
    let choose_window = owner.as_weak();
    owner.on_search_choose(move |index| {
        let Some(window) = choose_window.upgrade() else {
            return;
        };
        if let Err(error) = session::choose(
            &window,
            &choose_source,
            &mut choose_navigation.borrow_mut(),
            index,
        ) {
            open::failed(&window, &choose_source, format!("{error:#}"));
        }
    });
}
