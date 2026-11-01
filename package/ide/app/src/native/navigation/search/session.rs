//! Overlay transitions cancel obsolete work before accepting another query or navigation target.

/// Native navigation owns the sole project and latest file-open intent.
use super::super::{AppWindow, Navigation, State, open};
/// Bounded row presentation is cleared on every input change,
///  not only after debounce.
use super::{Search, present};
/// Queue and worker shutdown errors stay visible at the native boundary.
use anyhow::Result;
/// Parsed query flags remain independent of native filename and line identities.
use ide_app::{search::SearchKind, search_input::SearchInput};
/// UI-only ownership and monotonic edit timestamps never cross the worker boundary.
use std::{cell::RefCell, rc::Rc, time::Instant};

/// Remember an actual tree interaction,
///  not programmatic ancestor reveal for a recent file.
pub(super) fn remember(navigation: &mut Navigation, index: i32) {
    if index < 0 {
        return;
    }
    let Some(row) = navigation.rows.get(index as usize) else {
        return;
    };
    let mut directory = None;
    if row.entry.is_directory {
        directory = Some(row.entry.path.clone());
    } else if let Some(parent) = row.entry.path.parent() {
        directory = Some(parent.to_path_buf());
    }
    navigation.search.directory = directory;
}

/// Editing invalidates both the result model and subprocess generation immediately,
///  before the debounce delay.
pub(super) fn edit(window: &AppWindow, search: &mut Search, raw: &str) -> Result<()> {
    search.input = SearchInput::parse(raw);
    search.edited = None;
    search.hits.clear();
    window.set_search_query(raw.into());
    window.set_search_error("".into());
    window.set_search_busy(false);
    present::rows(window, &search.hits, &search.scope);
    search.worker.clear()?;
    if search.input.is_some() {
        search.edited = Some(Instant::now());
        window.set_search_busy(true);
    }
    return Ok(());
}

/// Capture the selected directory once;
///  later tree refresh cannot silently broaden the open overlay's search.
pub(super) fn start(window: &AppWindow, navigation: &mut Navigation) -> Result<()> {
    if window.get_tree_has_focus() {
        remember(navigation, window.get_tree_focused_row());
    }
    let search = &mut navigation.search;
    search.scope = match search.directory.clone() {
        Some(directory) => directory,
        None => navigation.workspace.root().to_path_buf(),
    };
    if !window.get_search_open() {
        search.return_tree = window.get_tree_has_focus();
    }
    let relative = search
        .scope
        .strip_prefix(navigation.workspace.root())
        .unwrap_or(&search.scope);
    let label = if relative.as_os_str().is_empty() {
        window.get_project_label().to_string()
    } else {
        relative.display().to_string()
    };
    window.set_search_scope(label.into());
    window.set_search_open(true);
    edit(window, search, "")?;
    window.invoke_focus_search();
    tracing::debug!(scope = %search.scope.display(), "native search opened");
    return Ok(());
}

/// Closing removes pending debounce and invalidates unread replies while retaining the displayed source.
pub(super) fn close(window: &AppWindow, search: &mut Search) -> Result<()> {
    search.edited = None;
    search.input = None;
    search.hits.clear();
    window.set_search_open(false);
    window.set_search_busy(false);
    present::rows(window, &search.hits, &search.scope);
    if search.return_tree || !window.get_source_available() {
        window.invoke_focus_tree();
    } else {
        window.invoke_focus_source();
    }
    tracing::debug!("native search closed and pending work cancelled");
    return search.worker.clear();
}

/// Native hit identity,
///  not its rendered text,
///  supplies the latest source-open intent.
pub(super) fn choose(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &mut Navigation,
    index: i32,
) -> Result<()> {
    if index < 0 || !window.get_search_open() {
        return Ok(());
    }
    let Some(hit) = navigation.search.hits.get(index as usize).cloned() else {
        return Ok(());
    };
    let line = match hit.kind {
        SearchKind::Path => None,
        SearchKind::Content { line, .. } => Some(line),
    };
    close(window, &mut navigation.search)?;
    return open::request_at(window, source, navigation, hit.path, line);
}
