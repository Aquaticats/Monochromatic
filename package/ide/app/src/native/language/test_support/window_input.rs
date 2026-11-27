//! Window input for the tests:
//!  the caret,
//!  the three keys language navigation listens to,
//!  window points over annotated rows,
//!  and pointer moves and clicks.

/// The reader whose window and state these helpers drive.
use super::LanguageReader;
/// Where source text starts,
///  and how far down the file label the first row stands.
use super::{
    TEXT_LEFT,
    TEXT_TOP,
};
/// Key events, as the other window tests send them.
use crate::native::find_tests::{
    chord,
    key,
};
/// The window an event is dispatched to,
///  and the render that follows a state change.
use crate::native::{
    AppWindow,
    render,
};
/// The side of a soft-wrap offset a caret position is read on.
use ide_app::document::ReadingAffinity;
/// Window events as a seat delivers them,
///  and the point they are delivered at.
use slint::{
    LogicalPosition,
    platform::{
        Key,
        PointerEventButton,
        WindowEvent,
    },
};

/// Place the caret at `position` through the document,
///  as a click would.
pub(super) fn caret(
    reader: &LanguageReader,
    position: usize,
) {
    let mut current = reader.source.borrow_mut();
    let mut reading = current.document.position();
    reading.anchor = position;
    reading.head = position;
    reading.affinity = ide_app::document::ReadingAffinity::Downstream;
    current.document.select(reading);
    drop(current);
    render(&reader.window, &reader.source);
}

/// Caret head in source characters.
pub(super) fn head(reader: &LanguageReader) -> usize {
    return reader.source.borrow().document.position().head;
}

/// Ctrl+B,
///  as a seat delivers it.
pub(super) fn definition(reader: &LanguageReader) {
    chord(&reader.window, Key::Control, "b");
}

/// Ctrl+Q,
///  as a seat delivers it.
pub(super) fn hover(reader: &LanguageReader) {
    chord(&reader.window, Key::Control, "q");
}

/// Escape,
///  as a seat delivers it.
pub(super) fn escape(reader: &LanguageReader) {
    key(&reader.window, Key::Escape);
}

/// What:
///  Window point over the code row of source `line`,
///  `x` logical pixels into its text,
///  in the middle of
///       the row.
///  The row's place comes from the reader's vertical mapping at the moment of the call.
/// Why:
///  The scripted server's hint and diagnostics take rows above the first line once they arrive,
///  so a
///      line's code row is not at its number times one row height.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function point(reader: LanguageReader, line: number, x: number): LogicalPosition;
/// ```
pub(super) fn point(
    reader: &LanguageReader,
    line: usize,
    x: f32,
) -> LogicalPosition {
    let top = reader.source.borrow().row_map.code_top(line);
    return LogicalPosition::new(
        TEXT_LEFT + x,
        TEXT_TOP + top + 12.0 + reader.window.get_scroll_y(),
    );
}

/// What:
///  Window point `rise` logical pixels above the top of `line`'s code row,
///  `x` logical pixels into the
///       text:
///  inside the line's virtual rows when it has any.
/// Why:
///  Resting and Ctrl+clicking on a hint or message row must ask the server nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function above(reader: LanguageReader, line: number, x: number, rise: number): LogicalPosition;
/// ```
pub(super) fn above(
    reader: &LanguageReader,
    line: usize,
    x: f32,
    rise: f32,
) -> LogicalPosition {
    let top = reader.source.borrow().row_map.code_top(line);
    return LogicalPosition::new(
        TEXT_LEFT + x,
        TEXT_TOP + top - rise + reader.window.get_scroll_y(),
    );
}

/// What:
///  How many hint texts and message-row texts the displayed frame paints;
///  a pair (tuple).
/// Why:
///  Hints and messages are pixels of the source image;
///  the frame's positioned records say what was drawn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rows(reader: LanguageReader): [hints: number, messages: number];
/// ```
pub(super) fn rows(reader: &LanguageReader) -> (usize, usize) {
    let source = reader.source.borrow();
    // `let ... else` answers zeros when no frame or no annotations of it exist.
    let Some(view) = source.shaped.as_ref() else {
        return (0, 0);
    };
    let Some(frame) = view.annotations.as_ref() else {
        return (0, 0);
    };
    let mut hints = 0;
    let mut messages = 0;
    for text in &frame.texts {
        if text.severity.is_some() {
            messages += 1;
        } else {
            hints += 1;
        }
    }
    return (hints, messages);
}

/// Move the pointer to a window point without a button held.
pub(super) fn move_to(
    window: &AppWindow,
    position: LogicalPosition,
) {
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position,
    });
}

/// A left click at a window point.
pub(super) fn click(
    window: &AppWindow,
    position: LogicalPosition,
) {
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
}

/// A Ctrl+click at a window point.
pub(super) fn control_click(
    window: &AppWindow,
    position: LogicalPosition,
) {
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    click(window, position);
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
}

/// Logical x of the caret before character `position` on its shaped row.
pub(super) fn caret_x(
    reader: &LanguageReader,
    position: usize,
) -> f32 {
    let state = reader.source.borrow();
    let line = state.document.text().char_to_line(position);
    let view = state.shaped.as_ref().expect("shaped source");
    for row in &view.rows {
        if row.row == line {
            return row.caret_x(position, view.viewport.scale);
        }
    }
    panic!("line {line} is not materialized");
}

/// The popup text,
///  or empty when no popup is shown.
pub(super) fn popup(reader: &LanguageReader) -> String {
    return reader.window.get_language_popup_text().to_string();
}
