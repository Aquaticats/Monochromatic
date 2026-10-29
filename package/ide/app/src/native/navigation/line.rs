//! Search content hits reveal a source line without introducing a go-to-line command or editable source.

/// The existing source renderer and document remain the only geometry and reading-state owners.
use super::{AppWindow, State};
/// The vertical mapping places the revealed line and bounds the scroll offset.
use crate::native::rows;
/// Character positions are canonical even when source lines contain multibyte UTF-8 or ligatures.
use ide_app::document::ReadingPosition;
/// `ComponentHandle` reaches the window's display scale,
///  which hint rows are packed at.
use slint::ComponentHandle;
/// Reading-state changes stay on the native event-loop thread.
use std::{cell::RefCell, rc::Rc};

/// Reveal a one-based search line,
///  clamping a now-stale line number to the current source snapshot.
pub(super) fn reveal(window: &AppWindow, source: &Rc<RefCell<State>>, requested: usize) {
    let mut current = source.borrow_mut();
    let line = requested
        .saturating_sub(1)
        .min(current.document.text().len_lines().saturating_sub(1));
    let position = current.document.text().line_to_char(line);
    current.document.select(ReadingPosition {
        anchor: position,
        head: position,
        viewport: position,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    // The file may have been installed just now; the map must describe it before a line is placed by it.
    rows::refresh(&mut current, window.window().scale_factor());
    // The line starts the view together with its virtual rows, or the view ends with the text.
    let limit = rows::limit(&current.row_map, window.get_viewport_height());
    let target = current.row_map.block_top(line).min(limit);
    let extent = current.row_map.height();
    // The window's next offset report is this reveal, not the reader scrolling.
    current.offset = target;
    // Release the checked mutable borrow before native scroll callbacks can request another source borrow.
    drop(current);
    // Update extent before offset so the old height cannot clamp the new offset.
    window.set_content_extent(extent);
    window.set_scroll_y(-target);
    let mut scrolled = source.borrow_mut();
    let first = scrolled.row_map.line_at(target);
    scrolled.first = first.saturating_sub(1);
    scrolled.frame_stamp = None;
    let mut reading = scrolled.document.position();
    reading.viewport = scrolled
        .document
        .text()
        .line_to_char(first.min(scrolled.document.text().len_lines().saturating_sub(1)));
    scrolled.document.select(reading);
    tracing::debug!(requested, line, "revealed search content line");
}
