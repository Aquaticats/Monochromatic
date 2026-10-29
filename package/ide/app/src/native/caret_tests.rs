//! Keyboard caret movement and selection through real window key events,
//!  with the view following the caret.

/// The production window whose scroll and caret state is read back.
use super::AppWindow;
/// The complete reader fixture and real key-event helpers shared with the find tests.
use super::find_tests::{Reader, chord, key, reader};
/// What:
///  `Key` names toolkit special keys;
///  `WindowEvent` is what a seat delivers to the window.
/// Why:
///  Tests dispatch the same encoded keys as the nested compositor's seat.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Key, WindowEvent } from 'slint';
/// ```
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
/// Disposable project files back every native window in these tests.
use std::fs;

/// What:
///  Anchor and head of the reading selection;
///  `(usize, usize)` is a pair of character positions
/// (`usize` is address-sized;
///  siblings `u32`,
///  `u64`).
/// Why:
///  Direction matters for Shift movement,
///  so the pair is not sorted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function position(reader: Reader): [anchor: number, head: number];
/// ```
pub(super) fn position(reader: &Reader) -> (usize, usize) {
    let current = reader.source.borrow().document.position();
    return (current.anchor, current.head);
}

/// Hold Control and Shift around one key,
///  releasing both afterwards.
fn control_shift(window: &AppWindow, target: Key) {
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    chord(window, Key::Shift, target);
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
}

/// Press one key `count` times.
fn repeat(window: &AppWindow, target: Key, count: usize) {
    for _ in 0..count {
        key(window, target);
    }
}

/// Left,
///  Right,
///  Home,
///  End,
///  their Control forms,
///  and every Shift variant.
#[test]
fn horizontal_keys_move_by_grapheme_word_line_and_document_and_extend_with_shift() {
    let fixture = tempfile::tempdir().expect("disposable caret project");
    // alpha(0..5) blank beta_2(6..12) blank ==(13..15) blank 猫猫(16..18) LF(18) second(19..25) blank line(26..30) LF.
    let text = "alpha beta_2 == 猫猫\nsecond line\n";
    fs::write(fixture.path().join("keys.txt"), text).expect("caret fixture");
    let reader = reader(fixture.path(), "keys.txt");
    let window = &reader.window;
    let end = text.chars().count();
    assert_eq!(position(&reader), (0, 0));
    key(window, Key::RightArrow);
    assert_eq!(position(&reader), (1, 1), "Right");
    chord(window, Key::Shift, Key::RightArrow);
    assert_eq!(position(&reader), (1, 2), "Shift+Right extends");
    // Two selected characters tell a collapse to the start apart from one step left of the head.
    chord(window, Key::Shift, Key::RightArrow);
    key(window, Key::LeftArrow);
    assert_eq!(
        position(&reader),
        (1, 1),
        "Left collapses a selection to its start"
    );
    chord(window, Key::Shift, Key::RightArrow);
    chord(window, Key::Shift, Key::RightArrow);
    key(window, Key::RightArrow);
    assert_eq!(
        position(&reader),
        (3, 3),
        "Right collapses a selection to its end"
    );
    chord(window, Key::Shift, Key::LeftArrow);
    assert_eq!(position(&reader), (3, 2), "Shift+Left extends backwards");
    chord(window, Key::Control, Key::RightArrow);
    assert_eq!(
        position(&reader),
        (5, 5),
        "Ctrl+Right to the end of the word"
    );
    chord(window, Key::Control, Key::RightArrow);
    assert_eq!(
        position(&reader),
        (12, 12),
        "underscore and digit stay in the word"
    );
    chord(window, Key::Control, Key::RightArrow);
    assert_eq!(position(&reader), (15, 15), "an operator is one stop");
    chord(window, Key::Control, Key::RightArrow);
    assert_eq!(
        position(&reader),
        (18, 18),
        "a run of CJK letters is one word"
    );
    chord(window, Key::Control, Key::RightArrow);
    assert_eq!(
        position(&reader),
        (25, 25),
        "Ctrl+Right continues on the next line"
    );
    chord(window, Key::Control, Key::LeftArrow);
    assert_eq!(
        position(&reader),
        (19, 19),
        "Ctrl+Left to the start of the word"
    );
    chord(window, Key::Control, Key::LeftArrow);
    assert_eq!(
        position(&reader),
        (16, 16),
        "Ctrl+Left continues on the previous line"
    );
    control_shift(window, Key::LeftArrow);
    assert_eq!(
        position(&reader),
        (16, 13),
        "Ctrl+Shift+Left extends by word"
    );
    control_shift(window, Key::RightArrow);
    assert_eq!(
        position(&reader),
        (16, 15),
        "Ctrl+Shift+Right extends by word"
    );
    key(window, Key::Home);
    assert_eq!(position(&reader), (0, 0), "Home");
    key(window, Key::End);
    assert_eq!(
        position(&reader),
        (18, 18),
        "End stops before the terminator"
    );
    chord(window, Key::Shift, Key::Home);
    assert_eq!(position(&reader), (18, 0), "Shift+Home");
    key(window, Key::RightArrow);
    chord(window, Key::Shift, Key::End);
    assert_eq!(position(&reader), (18, 18), "Shift+End from the line end");
    key(window, Key::Home);
    chord(window, Key::Shift, Key::End);
    assert_eq!(position(&reader), (0, 18), "Shift+End");
    chord(window, Key::Control, Key::End);
    assert_eq!(position(&reader), (end, end), "Ctrl+End");
    control_shift(window, Key::Home);
    assert_eq!(position(&reader), (end, 0), "Ctrl+Shift+Home");
    assert_eq!(
        window.get_selected_text(),
        text,
        "the extended selection copies the original source"
    );
    chord(window, Key::Control, Key::Home);
    assert_eq!(position(&reader), (0, 0), "Ctrl+Home");
    control_shift(window, Key::End);
    assert_eq!(position(&reader), (0, end), "Ctrl+Shift+End");
    chord(window, Key::Control, Key::Home);
    chord(window, Key::Alt, Key::RightArrow);
    chord(window, Key::Alt, Key::DownArrow);
    assert_eq!(
        position(&reader),
        (0, 0),
        "Alt combinations are not caret keys"
    );
    chord(window, Key::Control, Key::DownArrow);
    chord(window, Key::Control, Key::PageDown);
    assert_eq!(
        position(&reader),
        (0, 0),
        "Ctrl+Down and Ctrl+PageDown are unbound"
    );
    assert_eq!(window.get_source_text(), text, "caret keys changed source");
    window.hide().expect("close caret window");
}

/// A long fixture:
///  a full first line,
///  a short and an empty line,
///  then `lines` full lines.
fn tall(lines: usize) -> String {
    // What: `String::from` copies a literal into an owned, growable string (sibling: borrowed `&str`).
    // Why: The fixture is assembled line by line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let text = '0123456789\nab\n\n';
    // ```
    let mut text = String::from("0123456789\nab\n\n");
    for _ in 0..lines {
        text.push_str("0123456789\n");
    }
    return text;
}

/// Up and Down return to their column across a short and an empty line;
///  a horizontal move forgets it.
#[test]
fn vertical_keys_keep_a_preferred_column_and_extend_with_shift() {
    let fixture = tempfile::tempdir().expect("disposable caret project");
    // Line starts: 0 full, 11 "ab", 14 empty, 15 full, 26 full.
    let text = tall(200);
    fs::write(fixture.path().join("tall.txt"), &text).expect("caret fixture");
    let reader = reader(fixture.path(), "tall.txt");
    let window = &reader.window;
    repeat(window, Key::RightArrow, 6);
    assert_eq!(position(&reader), (6, 6));
    key(window, Key::DownArrow);
    assert_eq!(position(&reader), (13, 13), "Down clamps to the short line");
    key(window, Key::DownArrow);
    assert_eq!(position(&reader), (14, 14), "Down onto the empty line");
    key(window, Key::DownArrow);
    assert_eq!(position(&reader), (21, 21), "Down returns to column six");
    repeat(window, Key::UpArrow, 3);
    assert_eq!(position(&reader), (6, 6), "Up returns to column six");
    key(window, Key::UpArrow);
    assert_eq!(
        position(&reader),
        (0, 0),
        "Up on the first line goes to the start of text"
    );
    repeat(window, Key::RightArrow, 6);
    chord(window, Key::Shift, Key::DownArrow);
    chord(window, Key::Shift, Key::DownArrow);
    chord(window, Key::Shift, Key::DownArrow);
    assert_eq!(
        position(&reader),
        (6, 21),
        "Shift+Down extends and keeps the column"
    );
    chord(window, Key::Shift, Key::UpArrow);
    assert_eq!(position(&reader), (6, 14), "Shift+Up shrinks the selection");
    // Home on the empty line does not move the caret, yet it names column zero as the new aim.
    key(window, Key::Home);
    assert_eq!(position(&reader), (14, 14));
    key(window, Key::DownArrow);
    key(window, Key::DownArrow);
    assert_eq!(
        position(&reader),
        (26, 26),
        "Down after Home must stay in column zero"
    );
    window.hide().expect("close caret window");
}

/// Line `n` from three onwards starts at 15 + (n - 3) * 11.
fn start_of_line(line: usize) -> usize {
    return 15 + (line - 3) * 11;
}

/// PageDown and PageUp move caret and view by the whole lines in view;
///  Shift extends.
#[test]
fn page_keys_move_by_the_visible_height_and_keep_the_caret_in_view() {
    let fixture = tempfile::tempdir().expect("disposable caret project");
    let text = tall(200);
    fs::write(fixture.path().join("tall.txt"), &text).expect("caret fixture");
    let reader = reader(fixture.path(), "tall.txt");
    let window = &reader.window;
    let height = window.get_viewport_height();
    let page = (height / 24.0).floor() as usize;
    assert!(
        page >= 4,
        "the fixture window must show several lines, found {page}"
    );
    repeat(window, Key::DownArrow, 3);
    repeat(window, Key::RightArrow, 4);
    assert_eq!(position(&reader), (19, 19));
    key(window, Key::PageDown);
    let landed = start_of_line(3 + page) + 4;
    assert_eq!(
        position(&reader),
        (landed, landed),
        "PageDown by {page} lines, same column"
    );
    assert_eq!(
        -window.get_scroll_y(),
        page as f32 * 24.0,
        "PageDown must move the view by the same whole lines"
    );
    key(window, Key::PageDown);
    assert_eq!(position(&reader).1, start_of_line(3 + page * 2) + 4);
    assert_eq!(-window.get_scroll_y(), page as f32 * 48.0);
    key(window, Key::PageUp);
    key(window, Key::PageUp);
    assert_eq!(position(&reader), (19, 19), "PageUp returns to the start");
    assert_eq!(window.get_scroll_y(), 0.0, "PageUp returns the view");
    chord(window, Key::Shift, Key::PageDown);
    assert_eq!(position(&reader), (19, landed), "Shift+PageDown extends");
    chord(window, Key::Shift, Key::PageUp);
    assert_eq!(position(&reader), (19, 19), "Shift+PageUp shrinks");
    // Near the top a page up lands on the first line and keeps the column.
    key(window, Key::PageUp);
    assert_eq!(
        position(&reader),
        (4, 4),
        "PageUp near the top lands on the first line"
    );
    key(window, Key::PageUp);
    assert_eq!(
        position(&reader),
        (0, 0),
        "PageUp on the first line goes to the start of text"
    );
    let end = text.chars().count();
    chord(window, Key::Control, Key::End);
    key(window, Key::PageDown);
    assert_eq!(position(&reader), (end, end), "PageDown at the end stays");
    window.hide().expect("close caret window");
}

/// Whether the caret's line lies completely inside the vertical viewport.
pub(super) fn caret_in_view(reader: &Reader) -> bool {
    let window = &reader.window;
    let reading = reader.source.borrow();
    let head = reading.document.position().head;
    // The caret bar stands 2 px inside its visual row, so its row is the 24 px around it.
    let top = window.get_caret_y() - 2.0;
    let offset = -window.get_scroll_y();
    assert_eq!(
        reading.document.text().char_to_line(head),
        reading.row_map.line_at(top),
        "the caret is drawn on its own line"
    );
    return top >= offset && top + 24.0 <= offset + window.get_viewport_height();
}

/// Moving past an edge scrolls by the smallest amount;
///  document keys reach both ends.
#[test]
fn view_follows_the_caret_with_the_smallest_scroll() {
    let fixture = tempfile::tempdir().expect("disposable caret project");
    let text = format!("a short first line\n{}", tall(200));
    fs::write(fixture.path().join("follow.txt"), &text).expect("caret fixture");
    let reader = reader(fixture.path(), "follow.txt");
    let window = &reader.window;
    let height = window.get_viewport_height();
    let visible = (height / 24.0).floor() as usize;
    // The last fully visible line is `visible - 1`; one more Down must scroll, and only by what is missing.
    repeat(window, Key::DownArrow, visible - 1);
    assert_eq!(
        window.get_scroll_y(),
        0.0,
        "moving inside the view must not scroll"
    );
    key(window, Key::DownArrow);
    let needed = ((visible + 1) as f32 * 24.0 - height).round();
    assert_eq!(
        -window.get_scroll_y(),
        needed,
        "one line past the bottom edge must scroll by the missing part of that line only"
    );
    assert!(caret_in_view(&reader));
    // Two pages further the caret is still the bottom line; `visible` lines up it crosses the top edge.
    key(window, Key::PageDown);
    key(window, Key::PageDown);
    repeat(window, Key::UpArrow, visible);
    assert!(caret_in_view(&reader), "Up past the top edge must scroll");
    assert_eq!(
        -window.get_scroll_y(),
        (visible * 2) as f32 * 24.0,
        "one line past the top edge must put exactly that line at the top"
    );
    chord(window, Key::Control, Key::End);
    assert!(
        caret_in_view(&reader),
        "Ctrl+End must scroll to the end of text"
    );
    assert!(-window.get_scroll_y() > 200.0 * 24.0 - height);
    chord(window, Key::Control, Key::Home);
    assert_eq!(
        window.get_scroll_y(),
        0.0,
        "Ctrl+Home must scroll to the start"
    );
    assert_eq!(window.get_source_text(), text.as_str());
    window.hide().expect("close caret window");
}

/// A caret on a continuation row of a long wrapped line scrolls its own visual row into view,
/// not only the line.
#[test]
fn view_follows_the_caret_across_the_visual_rows_of_a_wrapped_line() {
    let fixture = tempfile::tempdir().expect("disposable caret project");
    // A token without break opportunities breaks at the width, so this line is taller than the view.
    let wide = "x".repeat(6000);
    let text = format!("{}{wide}\nlast\n", "short\n".repeat(3));
    fs::write(fixture.path().join("wrapped.txt"), &text).expect("caret fixture");
    let reader = reader(fixture.path(), "wrapped.txt");
    let window = &reader.window;
    let height = window.get_viewport_height();
    repeat(window, Key::DownArrow, 3);
    assert_eq!(
        window.get_caret_y(),
        3.0 * 24.0 + 2.0,
        "the caret is on the wide line's first row"
    );
    // The first End stops at the end of the first visual row; the second goes to the end of the line.
    key(window, Key::End);
    assert_eq!(
        window.get_caret_y(),
        3.0 * 24.0 + 2.0,
        "End stops at the end of the visual row it is on"
    );
    key(window, Key::End);
    let caret = window.get_caret_y() - 2.0;
    assert!(
        caret > height,
        "the end of the line is on a row below the first view"
    );
    let offset = -window.get_scroll_y();
    assert!(
        caret >= offset && caret + 24.0 <= offset + height,
        "End must scroll the caret's visual row into view: row at {caret}, view {offset} to {}",
        offset + height
    );
    assert_eq!(
        offset,
        (caret + 24.0 - height).round(),
        "the view moves only by what is missing"
    );
    // Home goes to the start of the last row first, which is already in view.
    key(window, Key::Home);
    assert_eq!(
        window.get_caret_y() - 2.0,
        caret,
        "Home stays on the visual row it is on"
    );
    assert_eq!(-window.get_scroll_y(), offset, "the view stays");
    key(window, Key::Home);
    let home_offset = -window.get_scroll_y();
    let home_caret = window.get_caret_y() - 2.0;
    assert!(
        home_caret >= home_offset && home_caret + 24.0 <= home_offset + height,
        "Home brings the first row back"
    );
    assert_eq!(
        home_caret,
        3.0 * 24.0,
        "and puts the caret on the first row"
    );
    window.hide().expect("close caret window");
}
