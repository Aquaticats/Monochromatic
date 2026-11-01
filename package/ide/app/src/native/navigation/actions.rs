//! Native tree activation and session-local Ctrl+digit navigation.

/// Shared source and project state remain independent of the background filesystem workers.
use super::{AppWindow, Navigation, State, open, present};
/// Decode exactly the approved editord-compatible shortcut grammar.
use ide_app::recent::shortcut_slot;
/// Weak native window handles avoid retaining a hidden window after shutdown.
use slint::{ComponentHandle, SharedString};
/// UI callbacks share state without any cross-thread mutable ownership.
use std::{cell::RefCell, rc::Rc};

/// Activate the current native row identity,
///  never reconstructing a path from its display label.
fn activate(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &mut Navigation,
    index: i32,
) {
    if index < 0 {
        return;
    }
    // Clone metadata before mutating the tree; stale indices remain a logged no-op.
    let Some(row) = navigation.rows.get(index as usize).cloned() else {
        tracing::debug!(index, "ignored obsolete tree row activation");
        return;
    };
    if row.entry.is_directory {
        navigation.reveal = None;
        match navigation.tree.toggle(&row.entry.path) {
            Ok(expanded) => {
                let errored_here = match navigation.directory_error.as_ref() {
                    Some((path, _message)) => path == &row.entry.path,
                    None => false,
                };
                if !expanded && errored_here {
                    navigation.directory_error = None;
                }
            }
            Err(error) => {
                tracing::warn!(%error, "tree directory activation failed");
                navigation.directory_error = Some((row.entry.path, format!("{error:#}")));
            }
        }
        present::update(window, source, navigation);
        return;
    }
    if let Err(error) = open::request(window, source, navigation, row.entry.path) {
        open::failed(window, source, format!("{error:#}"));
    }
}

/// Right expands or enters a directory;
///  left collapses or focuses its visible parent.
fn navigate(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &mut Navigation,
    key: &str,
    index: i32,
) {
    if index < 0 {
        return;
    }
    let Some(row) = navigation.rows.get(index as usize).cloned() else {
        return;
    };
    if SharedString::from(slint::platform::Key::RightArrow) == key {
        if row.entry.is_directory && !row.expanded {
            activate(window, source, navigation, index);
        } else if let Some(next) = navigation.rows.get(index as usize + 1)
            && next.depth > row.depth
        {
            window.invoke_reveal_tree(index + 1);
        }
    } else if SharedString::from(slint::platform::Key::LeftArrow) == key {
        if row.entry.is_directory && row.expanded {
            activate(window, source, navigation, index);
        } else {
            let mut parent = None;
            for (candidate_index, candidate) in
                navigation.rows[..index as usize].iter().enumerate().rev()
            {
                if candidate.depth < row.depth {
                    parent = Some(candidate_index);
                    break;
                }
            }
            if let Some(parent_index) = parent {
                window.invoke_reveal_tree(parent_index as i32);
            }
        }
    }
}

/// Bind callbacks once;
///  history remains session-local and promotes only after successful source installation.
pub(super) fn bind(
    owner: &AppWindow,
    shared_source: &Rc<RefCell<State>>,
    shared_navigation: &Rc<RefCell<Navigation>>,
) {
    let click_source = Rc::clone(shared_source);
    let click_navigation = Rc::clone(shared_navigation);
    let click_window = owner.as_weak();
    owner.on_tree_activate(move |index| {
        if let Some(window) = click_window.upgrade() {
            activate(
                &window,
                &click_source,
                &mut click_navigation.borrow_mut(),
                index,
            );
        }
    });
    let key_source = Rc::clone(shared_source);
    let key_navigation = Rc::clone(shared_navigation);
    let key_window = owner.as_weak();
    owner.on_tree_navigate(move |key, index| {
        if let Some(window) = key_window.upgrade() {
            navigate(
                &window,
                &key_source,
                &mut key_navigation.borrow_mut(),
                &key,
                index,
            );
        }
    });
    let recent_source = Rc::clone(shared_source);
    let recent_navigation = Rc::clone(shared_navigation);
    let recent_window = owner.as_weak();
    owner.on_tree_shortcut(move |key, control, shift, alt| {
        let Some(slot) = shortcut_slot(&key, control, shift, alt) else {
            return;
        };
        let Some(window) = recent_window.upgrade() else {
            return;
        };
        let mut navigation = recent_navigation.borrow_mut();
        let Some(path) = navigation.recent.at(slot) else {
            tracing::debug!(slot, "ignored empty recent-file slot");
            return;
        };
        // Own the slot before a later successful open can promote and reorder history.
        let target = path.to_path_buf();
        if let Err(error) = open::request(&window, &recent_source, &mut navigation, target) {
            open::failed(&window, &recent_source, format!("{error:#}"));
        }
    });
}
