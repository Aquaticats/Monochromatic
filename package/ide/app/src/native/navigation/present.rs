//! Tree labels are presentation only;
//!  row actions retain original native path identities.

/// Tree binding state is UI-local and remains separate from source reading state.
use super::{AppWindow, Navigation, State, watch};
/// Generated row values feed the toolkit's virtualized ListView.
use crate::native::ui::TreeEntry;
/// Owned row models survive beyond the presentation call without borrowing native state.
use slint::{ModelRc, SharedString, VecModel};
/// Native path ownership survives UI-label conversion;
///  Rc/RefCell remain on the event-loop thread.
use std::{cell::RefCell, rc::Rc};

/// Expand only known ancestors;
///  later directory replies expose the next missing level.
fn expand_reveal(navigation: &mut Navigation) {
    // Clone the desired identity so directory expansion can mutate the tree independently.
    let Some(target) = navigation.reveal.clone() else {
        return;
    };
    let mut ancestors = Vec::new();
    for ancestor in target.ancestors().skip(1) {
        if ancestor == navigation.workspace.root() {
            break;
        }
        ancestors.push(ancestor);
    }
    // Reverse the bounded ancestor walk so parents become visible before their children.
    for ancestor in ancestors.into_iter().rev() {
        let rows = navigation.tree.rows();
        let mut found_row = None;
        for candidate in rows.iter() {
            if candidate.entry.path == ancestor {
                found_row = Some(candidate);
                break;
            }
        }
        let Some(row) = found_row else {
            break;
        };
        if !row.entry.is_directory {
            break;
        }
        // A let-chain enters the error branch only when expansion was needed and that operation failed.
        if !row.expanded
            && let Err(error) = navigation.tree.toggle(ancestor)
        {
            tracing::warn!(%error, path = %ancestor.display(), "cannot expand reveal ancestor");
            break;
        }
    }
}

/// Publish rows after releasing the source borrow;
///  reveal only after the complete target row is present.
pub(super) fn update(window: &AppWindow, source: &Rc<RefCell<State>>, navigation: &mut Navigation) {
    expand_reveal(navigation);
    navigation.rows = navigation.tree.rows();
    // Expansion, collapse, and file switches all pass here, so the watch set follows what is shown.
    watch::show(source, navigation);
    let selected = source.borrow().file_path.clone();
    let mut model = Vec::new();
    let mut reveal_index = None;
    for (index, row) in navigation.rows.iter().enumerate() {
        // Native filenames may not be UTF-8; this lossy string is never used to reopen their paths.
        let label = SharedString::from(row.entry.name.to_string_lossy().into_owned());
        let mut slot = None;
        for (slot_position, path) in navigation.recent.paths().iter().enumerate() {
            if path == &row.entry.path {
                slot = Some(slot_position);
                break;
            }
        }
        let recency = if let Some(slot_index) = slot {
            slot_index.to_string()
        } else {
            String::new()
        };
        model.push(TreeEntry {
            label,
            depth: row.depth as i32,
            directory: row.entry.is_directory,
            expanded: row.expanded,
            selected: selected.as_ref() == Some(&row.entry.path),
            recency: SharedString::from(recency),
            ignored: row.ignored,
        });
        if navigation.reveal.as_ref() == Some(&row.entry.path) {
            reveal_index = Some(index as i32);
        }
    }
    // Move owned rows into the toolkit model; the native row vector retains the corresponding exact paths.
    window.set_tree_entries(ModelRc::from(Rc::new(VecModel::from(model))));
    let message = if let Some((_path, message)) = &navigation.directory_error {
        message.as_str()
    } else {
        ""
    };
    window.set_tree_error(SharedString::from(message));
    if let Some(index) = reveal_index {
        window.invoke_reveal_tree(index);
        navigation.reveal = None;
    }
}
