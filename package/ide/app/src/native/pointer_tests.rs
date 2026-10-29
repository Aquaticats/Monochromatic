//! Pointer selection through real window pointer events:
//!  caret,
//!  word,
//!  line,
//!  Shift+click,
//!  and drags.

/// The production window that receives the pointer events.
use super::AppWindow;
/// Anchor and head of the reading selection,
///  shared with the keyboard tests.
use super::caret_tests::position;
/// The complete reader fixture shared with the find tests.
use super::find_tests::{Reader, reader};
/// The mark drawn for a selected line terminator.
use ide_app::shaped_text::TERMINATOR_MARK;
/// What:
///  Window events as a seat delivers them,
///  and a point in logical window pixels.
/// Why:
///  Presses,
///  moves,
///  and releases go through the real `TouchArea` and `Flickable` event handling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LogicalPosition, Model, PointerEventButton, WindowEvent, Key } from 'slint';
/// ```
use slint::{
    ComponentHandle, LogicalPosition, Model,
    platform::{Key, PointerEventButton, WindowEvent},
};
/// What:
///  Disposable project files,
///  and a pause longer than the multi-click interval.
/// Why:
///  A slow second press must be a new single click.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { writeFileSync } from 'node:fs'; import { setTimeout } from 'node:timers/promises';
/// ```
use std::{fs, thread::sleep, time::Duration};

/// Where source text starts in the window,
///  derived in `sidebar_tests` from the tree,
///  divider,
///  and gutter widths.
/// The first assertion of every test is a positive control for this origin and `TEXT_TOP`.
use super::sidebar_tests::TEXT_LEFT;

/// Source rows start below the 32 px file label.
const TEXT_TOP: f32 = 32.0;

/// Window point over source row `row`,
///  `x` logical pixels into the text,
///  in the middle of the line.
fn point(window: &AppWindow, row: usize, x: f32) -> LogicalPosition {
    return LogicalPosition::new(
        TEXT_LEFT + x,
        TEXT_TOP + row as f32 * 24.0 + 12.0 + window.get_scroll_y(),
    );
}

/// Press the left button at a window point.
fn press(window: &AppWindow, position: LogicalPosition) {
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
}

/// Release the left button at a window point.
fn release(window: &AppWindow, position: LogicalPosition) {
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
}

/// One complete click at a window point.
fn click(window: &AppWindow, position: LogicalPosition) {
    press(window, position);
    release(window, position);
}

/// Logical x of the caret before source character `head` on its materialized row.
fn caret_x(reader: &Reader, head: usize) -> f32 {
    let reading = reader.source.borrow();
    let row = reading.document.text().char_to_line(head);
    let view = reading.shaped.as_ref().expect("shaped source");
    return view.rows[row - view.viewport.first].caret_x(head, view.viewport.scale);
}

/// One,
///  two,
///  and three quick presses select a caret,
///  a word,
///  and a line;
///  a fourth starts over.
#[test]
fn click_count_selects_caret_word_and_line() {
    let fixture = tempfile::tempdir().expect("disposable pointer project");
    // alpha(0..5) blank beta(6..10) blank gamma(11..16) LF(16); second line starts at 17; empty line at 34; last at 35.
    let text = "alpha beta gamma\nsecond line here\n\nlast";
    fs::write(fixture.path().join("pointer.txt"), text).expect("pointer fixture");
    let reader = reader(fixture.path(), "pointer.txt");
    let window = &reader.window;
    let inside_beta = point(window, 0, caret_x(&reader, 8) + 1.0);
    click(window, inside_beta);
    assert_eq!(
        position(&reader),
        (8, 8),
        "a single click must place the caret at the nearest boundary (coordinate control)"
    );
    click(window, inside_beta);
    assert_eq!(
        position(&reader),
        (6, 10),
        "a double click selects the word"
    );
    assert_eq!(window.get_selected_text(), "beta");
    click(window, inside_beta);
    assert_eq!(
        position(&reader),
        (0, 17),
        "a triple click selects the line with its terminator"
    );
    assert_eq!(window.get_selected_text(), "alpha beta gamma\n");
    click(window, inside_beta);
    assert_eq!(
        position(&reader),
        (8, 8),
        "a fourth click is a single click"
    );
    // A second press after the multi-click interval is a new single click, not a double click.
    sleep(Duration::from_millis(550));
    click(window, inside_beta);
    assert_eq!(
        position(&reader),
        (8, 8),
        "a slow second click selected a word"
    );
    // A double click on another row starts its own count.
    let inside_line = point(window, 1, caret_x(&reader, 17 + 8) + 1.0);
    click(window, inside_line);
    assert_eq!(position(&reader), (25, 25));
    click(window, inside_line);
    assert_eq!(
        position(&reader),
        (24, 28),
        "double click on the second line"
    );
    assert_eq!(window.get_selected_text(), "line");
    // A triple click on the empty line selects its terminator and shows a mark for it.
    let empty = point(window, 2, 1.0);
    click(window, empty);
    click(window, empty);
    click(window, empty);
    assert_eq!(position(&reader), (34, 35));
    assert_eq!(window.get_selected_text(), "\n");
    let marks = window.get_source_selections();
    assert_eq!(
        marks.row_count(),
        1,
        "the selected empty line shows nothing"
    );
    assert_eq!(
        marks.row_data(0).expect("terminator mark").width,
        TERMINATOR_MARK
    );
    assert_eq!(window.get_source_text(), text);
    window.hide().expect("close pointer window");
}

/// Shift+click extends from the anchor in both directions and is never counted as a double click.
#[test]
fn shift_click_extends_the_selection_from_its_anchor() {
    let fixture = tempfile::tempdir().expect("disposable pointer project");
    let text = "alpha beta gamma\nsecond line here\n\nlast";
    fs::write(fixture.path().join("pointer.txt"), text).expect("pointer fixture");
    let reader = reader(fixture.path(), "pointer.txt");
    let window = &reader.window;
    click(window, point(window, 0, caret_x(&reader, 8) + 1.0));
    assert_eq!(position(&reader), (8, 8));
    let later = point(window, 1, caret_x(&reader, 17 + 3) + 1.0);
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    click(window, later);
    assert_eq!(position(&reader), (8, 20), "Shift+click extends forwards");
    click(window, later);
    assert_eq!(
        position(&reader),
        (8, 20),
        "a second Shift+click at the same place must not select a word"
    );
    click(window, point(window, 0, caret_x(&reader, 2) + 1.0));
    assert_eq!(
        position(&reader),
        (8, 2),
        "Shift+click before the anchor extends backwards"
    );
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    assert_eq!(window.get_selected_text(), "pha be");
    // The first plain click was at this very point moments ago; the Shift+clicks in between ended that count.
    click(window, point(window, 0, caret_x(&reader, 8) + 1.0));
    assert_eq!(
        position(&reader),
        (8, 8),
        "a plain click after Shift+click must be a single click"
    );
    window.hide().expect("close pointer window");
}

/// A mouse drag selects text instead of panning the view,
///  and after a double click it extends by words.
#[test]
fn drag_selects_text_by_characters_and_by_words_without_panning() {
    let fixture = tempfile::tempdir().expect("disposable pointer project");
    // Two hundred lines make the view scrollable, so a pan would be possible if drags were not selections.
    let text = "alpha beta gamma\n".repeat(200);
    fs::write(fixture.path().join("drag.txt"), &text).expect("drag fixture");
    let reader = reader(fixture.path(), "drag.txt");
    let window = &reader.window;
    let start = point(window, 1, caret_x(&reader, 17 + 2) + 1.0);
    let target = point(window, 5, caret_x(&reader, 17 + 12) + 1.0);
    press(window, start);
    assert_eq!(position(&reader), (19, 19), "coordinate control");
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: target });
    assert_eq!(
        position(&reader),
        (19, 5 * 17 + 12),
        "a quick drag of four lines must extend the selection, not be swallowed"
    );
    assert_eq!(
        window.get_scroll_y(),
        0.0,
        "a mouse drag panned the source view"
    );
    release(window, target);
    assert_eq!(position(&reader), (19, 5 * 17 + 12));
    // A double click selects "beta"; dragging right takes whole words, dragging left keeps "beta".
    let beta = point(window, 0, caret_x(&reader, 8) + 1.0);
    click(window, beta);
    press(window, beta);
    assert_eq!(position(&reader), (6, 10));
    let gamma = point(window, 0, caret_x(&reader, 13) + 1.0);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: gamma });
    assert_eq!(
        position(&reader),
        (6, 16),
        "a word drag takes the whole word"
    );
    let alpha = point(window, 0, caret_x(&reader, 2) + 1.0);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: alpha });
    assert_eq!(
        position(&reader),
        (10, 0),
        "a word drag to the left keeps the pressed word and takes the earlier word"
    );
    release(window, alpha);
    assert_eq!(window.get_selected_text(), "alpha beta");
    // Only the left button selects: a drag with the middle button leaves the selection alone.
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: start,
        button: PointerEventButton::Middle,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: target });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: target,
            button: PointerEventButton::Middle,
        });
    assert_eq!(
        position(&reader),
        (10, 0),
        "a drag with the middle button changed the selection"
    );
    assert_eq!(window.get_source_text(), text.as_str());
    window.hide().expect("close pointer window");
}
