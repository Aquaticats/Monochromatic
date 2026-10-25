//! A resting pointer while the view scrolls under it.
//!  The toolkit sends no pointer event for text that moves under a still pointer,
//!  so what a hover request names must be found again from the pointer's place in the view
//!  when the rest is over,
//!  and a view that is still moving is not a rest.

/// The rest time and the places it is kept for.
use super::pointer::{REST, Rest, Spot};
/// Fixtures and real window events.
use super::test_support::{
    LanguageReader, TEXT_TOP, caret_x, definitions, idle, move_to, point, popup, project, reader,
    ready,
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Timers of the toolkit.
use slint::platform::update_timers_and_animations;
/// Instants for a rest that began long ago.
use std::time::Instant;

/// Logical pixels of one code row.
const ROW: f32 = 24.0;

/// `lines` lines:
///  `alphaNNN beta` on even lines,
///  blank odd lines.
fn long_text(lines: usize) -> String {
    let mut text = String::new();
    for index in 0..lines {
        if index % 2 == 0 {
            text.push_str(&format!("alpha{index:03} beta\n"));
        } else {
            text.push('\n');
        }
    }
    return text;
}

/// Character offset of the start of `line` in `long_text`:
///  even lines have 14 characters with their line break,
///  odd lines one.
fn start(line: usize) -> usize {
    return (line / 2) * 15 + (line % 2) * 14;
}

/// A ready reader of `long_text(lines)`,
///  and the window point of the `b` of `beta` on line 4.
fn resting_place(lines: usize) -> (super::test_support::Project, LanguageReader, f32) {
    let fixture = project(&[
        ("main.scripted", &long_text(lines)),
        ("other.scripted", "o\n"),
    ]);
    let opened = reader(&fixture, "main.scripted", definitions(&[], None));
    ready(&opened);
    let x = caret_x(&opened, start(4) + 9) + 2.0;
    return (fixture, opened, x);
}

/// The line drawn under the window point `y`:
///  the point in the text is the distance below the top of the view
///  plus the offset that is drawn.
fn line_drawn_under(reader: &LanguageReader, y: f32) -> usize {
    let text_y = y - TEXT_TOP + reader.window.get_drawn_offset();
    return reader.source.borrow().row_map.locate(text_y).line;
}

/// Wait for the hover popup and give its text.
fn wait_for_popup(reader: &LanguageReader, what: &str) -> String {
    let mut waiting = Waiting::language();
    while popup(reader).is_empty() {
        waiting.step(what);
    }
    return popup(reader);
}

/// The hover popup of a request names the line under the pointer in the view that is drawn,
///  not the line the pointer was over when it last moved.
fn assert_names_line_drawn_under(reader: &LanguageReader, y: f32, text: &str) {
    let line = line_drawn_under(reader, y);
    assert!(
        line > 4,
        "the view did not move: the pointer is over line {line}"
    );
    assert!(
        text.starts_with(&format!("line=alpha{line:03} beta")),
        "the popup `{text}` is no hover for the row under the pointer, which is line {line}"
    );
}

/// A pointer at rest while the view scrolls a little:
///  the request is for the row that is under the pointer when the rest is over.
#[test]
fn resting_pointer_names_the_row_under_it_after_a_scroll() {
    let (_fixture, reader, x) = resting_place(400);
    let place = point(&reader, 4, x);
    move_to(&reader.window, place);
    // Sixty lines down inside the rest time:
    //  no pointer event follows.
    reader.window.set_scroll_y(-(60.0 * ROW));
    update_timers_and_animations();
    let text = wait_for_popup(
        &reader,
        "no hover for the row under the pointer appeared after the scroll",
    );
    assert_names_line_drawn_under(&reader, place.y, &text);
}

/// A pointer at rest while the view jumps far past the tile:
///  the view is held at the tile's edge until the tile lands,
///  and the request is for the row of the frame drawn when it is built.
#[test]
fn resting_pointer_names_the_row_under_it_after_a_far_jump() {
    let (_fixture, reader, x) = resting_place(900);
    let place = point(&reader, 4, x);
    move_to(&reader.window, place);
    reader.window.set_scroll_y(-(500.0 * ROW));
    update_timers_and_animations();
    assert!(
        reader.window.get_held(),
        "the jump did not outrun the displayed tile"
    );
    let text = wait_for_popup(
        &reader,
        "no hover for the row under the pointer appeared after the jump",
    );
    assert_names_line_drawn_under(&reader, place.y, &text);
    assert!(!reader.window.get_held(), "the request came while held");
}

/// A pointer that moves while the view is held at the tile's edge:
///  the character it is over then is another one than the settled view puts under it.
#[test]
fn pointer_moved_during_the_hold_names_the_settled_row() {
    let (_fixture, reader, x) = resting_place(900);
    let first = point(&reader, 4, x);
    move_to(&reader.window, first);
    reader.window.set_scroll_y(-(500.0 * ROW));
    update_timers_and_animations();
    assert!(
        reader.window.get_held(),
        "the jump did not outrun the displayed tile"
    );
    let second = slint::LogicalPosition::new(first.x + 14.0, first.y);
    move_to(&reader.window, second);
    let text = wait_for_popup(
        &reader,
        "no hover for the row under the pointer appeared after the jump",
    );
    assert_names_line_drawn_under(&reader, second.y, &text);
}

/// A view that keeps moving is no rest:
///  no request goes out while it moves,
///  and the one after it is for the row it stopped on.
#[test]
fn a_view_still_scrolling_asks_nothing_until_it_stops() {
    let (_fixture, reader, x) = resting_place(400);
    let place = point(&reader, 4, x);
    move_to(&reader.window, place);
    // A step of two lines each 100 ms for 1.2 s:
    //  more than three rest times.
    for step in 1..=12 {
        reader.window.set_scroll_y(-(2.0 * ROW * step as f32));
        for _ in 0..10 {
            idle(10);
            assert_eq!(
                popup(&reader),
                "",
                "a hover was asked for while the view moved"
            );
        }
    }
    let text = wait_for_popup(
        &reader,
        "no hover for the row under the pointer appeared after the scroll",
    );
    assert_names_line_drawn_under(&reader, place.y, &text);
}

/// A rest that began long ago is due,
///  and a view that moves makes it begin again;
///  one already asked for stays asked.
#[test]
fn a_moving_view_restarts_a_rest_that_was_not_asked_for() {
    let spot = Spot {
        below_top: 100.0,
        x: 20.0,
        offset: 500.0,
    };
    let mut rest = Rest::default();
    rest.moved(Some(7), spot);
    assert_eq!(rest.due(), None, "a rest is due before the rest time");
    rest.at = Some((Some(7), Instant::now() - REST * 2));
    assert_eq!(rest.due(), Some(spot));
    // Within half a pixel is rounding.
    rest.follow(500.4);
    assert_eq!(rest.due(), Some(spot), "rounding restarted the rest");
    rest.follow(620.0);
    assert_eq!(rest.due(), None, "a moved view did not restart the rest");
    rest.at = Some((Some(7), Instant::now() - REST * 2));
    rest.asked = true;
    rest.follow(900.0);
    assert!(rest.asked, "a request already made was forgotten");
    assert_eq!(rest.due(), None);
}
