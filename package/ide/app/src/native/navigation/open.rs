//! File-open success is the only boundary that replaces displayed source or promotes history.

/// Native source state and tree presentation share one event-loop thread.
use super::{AppWindow, Navigation, State, line, present};
/// A language target names a file and a place in it;
///  the caret goes there once the file is shown.
use crate::native::language::{Jump, place};
/// Reuse the same revision-aware classification application as external reloads;
///  `rows` maps the new text.
use crate::native::{reload::apply_syntax, render, rows};
/// Identity exhaustion reports an error rather than reusing an obsolete file generation.
use anyhow::{Context, Result};
/// New source documents arrive only after successful project-boundary resolution and reading.
use ide_app::{file_open::OpenedFile, source_style::SourceStyles};
/// File context remains a display-only native-path label;
///  `ComponentHandle` reaches the window's display scale.
use slint::{ComponentHandle, SharedString};
/// Shared source state never crosses the background-reader thread.
use std::{cell::RefCell, path::PathBuf, rc::Rc};

/// Focus the source after an open,
///  unless the find input is being typed into.
/// A recent-file shortcut pressed in the find bar then keeps stepping through matches of the new file.
fn focus_source(window: &AppWindow) {
    if window.get_find_has_focus() {
        tracing::debug!("kept keyboard focus in the find input after a source open");
        return;
    }
    window.invoke_focus_source();
}

/// Keep the current document and viewport when reopening its existing canonical target.
pub(super) fn request(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &mut Navigation,
    path: PathBuf,
) -> Result<()> {
    return request_at(window, source, navigation, path, None);
}

/// A content result's line stays attached to the newest request,
///  including contained canonical aliases.
pub(super) fn request_at(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &mut Navigation,
    path: PathBuf,
    target_line: Option<usize>,
) -> Result<()> {
    navigation.pending_line = None;
    // Every new open intent replaces a language target that was still waiting for its file.
    source.borrow_mut().pending_jump = None;
    if source.borrow().file_path.as_ref() == Some(&path) {
        navigation.opener.cancel()?;
        navigation.reveal = Some(path);
        source.borrow_mut().navigation_error = None;
        present::update(window, source, navigation);
        if let Some(target) = target_line {
            line::reveal(window, source, target);
        }
        render(window, source);
        if !window.get_search_open() {
            focus_source(window);
        }
        return Ok(());
    }
    navigation.opener.request(path)?;
    navigation.pending_line = target_line;
    // A new intent clears the previous open diagnostic, not the old document or its own refresh errors.
    source.borrow_mut().navigation_error = None;
    render(window, source);
    return Ok(());
}

/// What:
///  Open the file of a language target and put the caret at the target once it is shown.
///       `Jump` names the canonical path,
///  whether it lies outside the project,
///  and the place.
/// Why:
///  Project files go through the ordinary open (history,
///  tree reveal,
///  badges);
///  a file outside
///      the project is read without project resolution and gets neither history nor reveal.
///  The
///      displayed file itself is not reopened:
///  the caret moves at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function requestJump(window, source, navigation, jump: Jump): void
/// ```
pub(in crate::native) fn request_jump(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &mut Navigation,
    jump: Jump,
) -> Result<()> {
    if source.borrow().file_path.as_ref() == Some(&jump.path) {
        // A waiting open of another file must not replace the file the target is in.
        navigation.opener.cancel()?;
        navigation.pending_line = None;
        source.borrow_mut().pending_jump = None;
        place(window, source, &jump);
        focus_source(window);
        return Ok(());
    }
    if jump.outside {
        navigation.pending_line = None;
        navigation.opener.request_outside(jump.path.clone())?;
        source.borrow_mut().navigation_error = None;
    } else {
        request_at(window, source, navigation, jump.path.clone(), None)?;
    }
    tracing::debug!(path = %jump.path.display(), outside = jump.outside, "requested a language target's file");
    // `Some(jump)` keeps the target until its file is installed; any later open intent clears it.
    source.borrow_mut().pending_jump = Some(jump);
    render(window, source);
    return Ok(());
}

/// Read failures do not replace readable source or promote a failed path into history.
pub(super) fn failed(window: &AppWindow, source: &Rc<RefCell<State>>, message: String) {
    tracing::warn!(%message, "project source open failed; retaining displayed source");
    let mut current = source.borrow_mut();
    current.navigation_error = Some(message);
    // The target belonged to the open that failed.
    current.pending_jump = None;
    drop(current);
    render(window, source);
}

/// Install a successful different file,
///  invalidating all source-specific frame and accessibility caches.
pub(super) fn apply(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &mut Navigation,
    opened: OpenedFile,
) -> Result<()> {
    // Own the resolved identity for history/reveal after the document moves into its native owner.
    let path = opened.path;
    let outside = opened.outside_project;
    let mut current = source.borrow_mut();
    let changed = current.file_path.as_ref() != Some(&path);
    if changed {
        let generation = current
            .file_generation
            .checked_add(1)
            .context("Displayed-file identity exhausted; restart the application")?;
        current.document = opened.document;
        current.file_path = Some(path.clone());
        current.file_generation = generation;
        current.file_error = None;
        current.syntax_error = None;
        current.syntax_revision = None;
        current.styles = SourceStyles::from([]);
        current.first = 0;
        // A new file starts at its top; the window's next offset report is that reset, not scrolling.
        current.offset = 0.0;
        current.shaped = None;
        current.frame_stamp = None;
        // A tile queued for the old file must not be swapped in for the new one.
        crate::native::tile::cancel(&mut current);
        current.presented_revision = None;
        if let Some(syntax) = opened.syntax {
            apply_syntax(&mut current, syntax);
        }
    }
    current.navigation_error = None;
    current.outside_project = outside;
    // The scroll extent of the text now displayed comes from its own vertical mapping.
    rows::refresh(&mut current, window.window().scale_factor());
    let extent = current.row_map.height();
    // `take` moves a waiting language target out; it applies only to the file it names.
    let jump = current.pending_jump.take();
    drop(current);
    if outside {
        // A file outside the project has no tree row and is not a Ctrl+digit history entry.
        navigation.reveal = None;
    } else {
        // Promote only successful opens; canonical aliases share the existing history slot.
        navigation.recent.opened(path.clone());
        navigation.reveal = Some(path.clone());
    }
    window.set_file_label(SharedString::from(path.display().to_string()));
    window.set_file_outside_project(outside);
    window.set_source_available(true);
    if changed {
        window.set_content_extent(extent);
        window.invoke_reset_source_scroll();
    }
    if let Some(target) = navigation.pending_line.take() {
        line::reveal(window, source, target);
    }
    // Every other open intent cleared the target, so a waiting one belongs to this file.
    if let Some(target) = jump {
        place(window, source, &target);
    }
    render(window, source);
    present::update(window, source, navigation);
    if !window.get_search_open() {
        focus_source(window);
    }
    tracing::info!(path = %path.display(), changed, outside, "project source open presented");
    return Ok(());
}
