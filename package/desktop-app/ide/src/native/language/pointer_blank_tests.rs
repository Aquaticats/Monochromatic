//! A pointer that rests over no character while the view moves under it.
//!  A hint row,
//!  a blank line,
//!  the gutter,
//!  and the space past a line's end are over no character,
//!  and the toolkit sends no pointer event for text that moves under a still pointer,
//!  so only the rest itself can notice that a character arrived under the pointer.
//!  In a view that stands still such a pointer asks nothing and closes a popup.

/// The rest time and the places it is kept for.
use super::pointer::{REST, Rest, Spot, View};
/// The fixture of the scroll tests and its checks of the line a request names.
use super::pointer_scroll_tests::{
    ROW, assert_names_line_drawn_under, line_drawn_under, resting_place, resting_place_with,
    wait_for_popup,
};
/// Fixtures and real window events.
use super::test_support::{escape, idle, move_to, point, popup, rows};
/// Which text the displayed hint answer covers.
use crate::native::annotate::displayed;
/// The scrolls the toolkit reports to the change handler,
///  and the wait for the tile worker.
use crate::native::overscan_tile_tests::{scroll_to, settle_tile};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Timers of the toolkit.
use slint::platform::update_timers_and_animations;
/// Instants for a rest that began long ago.
use std::time::Instant;

/// A line of `long_text` with no character:
///  odd lines are blank.
const BLANK: usize = 5;

/// The scripted server never answers these hint requests,
///  so no hint row moves the text.
const SILENT: &str = "silent,silent,silent,silent,silent,silent,silent,silent";

/// A pointer at rest over a blank line while the view scrolls a little,
///  so that a line with text stands under it:
///  the request is for the row that is under the pointer when the rest is over.
#[test]
fn a_rest_over_no_character_asks_when_a_scroll_puts_a_character_under_it() {
    let (_fixture, reader, x) = resting_place(400);
    let place = point(&reader, BLANK, x);
    move_to(&reader.window, place);
    // An odd number of lines down inside the rest time:
    //  no pointer event follows.
    reader.window.set_scroll_y(-(61.0 * ROW));
    update_timers_and_animations();
    let text = wait_for_popup(
        &reader,
        "no hover for the row under the pointer appeared after the scroll",
    );
    assert_names_line_drawn_under(&reader, place.y, &text);
}

/// The same pointer while the view jumps far past the tile:
///  the view is held at the tile's edge until the tile lands,
///  and the request is for the row of the frame drawn when it is built.
#[test]
fn a_rest_over_no_character_asks_after_a_far_jump() {
    let (_fixture, reader, x) = resting_place(900);
    let place = point(&reader, BLANK, x);
    move_to(&reader.window, place);
    reader.window.set_scroll_y(-(501.0 * ROW));
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

/// A pointer over no character in a view that stands still asks nothing,
///  however long it rests,
///  and marks nothing asked.
#[test]
fn a_rest_over_no_character_in_a_settled_view_asks_nothing() {
    // No hint rows arrive in this test, so nothing but the pointer's own move changes what stands under it.
    let (_fixture, reader, x) = resting_place_with(400, &[("HINT_STEPS", SILENT)]);
    move_to(&reader.window, point(&reader, BLANK, x));
    let binding = reader.binding.as_ref().expect("the language binding");
    let arrived = binding.language.borrow().rest.at;
    assert!(arrived.is_some(), "the move did not start a rest");
    idle(3 * REST.as_millis() as u64);
    assert_eq!(popup(&reader), "", "a pointer over no character asked");
    let language = binding.language.borrow();
    // A rest that was resolved again would have taken a new moment.
    assert_eq!(
        language.rest.at, arrived,
        "a settled rest over nothing was resolved again"
    );
    assert!(
        !language.rest.asked,
        "a rest over no character was marked asked"
    );
    assert_eq!(
        language.rest.due(),
        None,
        "a settled rest over nothing is due"
    );
}

/// A scroll that leaves no character under the pointer asks nothing,
///  and the rest is over no character again:
///  the next scroll that puts a character under it is asked about.
#[test]
fn a_rest_that_finds_no_character_after_a_scroll_asks_after_the_next_scroll() {
    // No hint rows arrive in this test, so the lines stand where the offsets put them.
    let (_fixture, reader, x) = resting_place_with(400, &[("HINT_STEPS", SILENT)]);
    let place = point(&reader, BLANK, x);
    move_to(&reader.window, place);
    // An even number of lines leaves another blank line under the pointer.
    reader.window.set_scroll_y(-(60.0 * ROW));
    update_timers_and_animations();
    idle(3 * REST.as_millis() as u64);
    assert_eq!(popup(&reader), "", "a blank line under the pointer asked");
    {
        let binding = reader.binding.as_ref().expect("the language binding");
        let language = binding.language.borrow();
        assert!(
            !language.rest.asked,
            "a rest that asked nothing is marked asked"
        );
        assert_eq!(
            language.rest.due(),
            None,
            "a settled rest over nothing is due"
        );
    }
    reader.window.set_scroll_y(-(61.0 * ROW));
    update_timers_and_animations();
    let text = wait_for_popup(
        &reader,
        "no hover appeared after the scroll that put a character under the pointer",
    );
    assert_names_line_drawn_under(&reader, place.y, &text);
}

/// A popup that is open over a character closes when the view moves and no character is under the pointer,
///  and nothing opens again while the pointer rests over no character.
#[test]
fn a_popup_closes_when_a_scroll_leaves_no_character_under_the_pointer() {
    let (_fixture, reader, x) = resting_place_with(400, &[("HINT_STEPS", SILENT)]);
    move_to(&reader.window, point(&reader, 4, x));
    let shown = wait_for_popup(&reader, "no hover appeared for the resting pointer");
    assert!(
        shown.starts_with("line=alpha004 beta"),
        "the popup is `{shown}`"
    );
    // One line down puts the blank line 5 under the pointer.
    reader.window.set_scroll_y(-ROW);
    update_timers_and_animations();
    let mut waiting = Waiting::language();
    while !popup(&reader).is_empty() {
        waiting.step("the popup stayed after the view moved");
    }
    idle(3 * REST.as_millis() as u64);
    assert_eq!(
        popup(&reader),
        "",
        "a pointer over no character opened a popup"
    );
}

/// Hints that arrive after a jump put other text under a pointer that rests over no character:
///  a blank line first,
///  a line with text once the hint rows stand above it.
#[test]
fn a_rest_over_no_character_asks_when_hints_arrive_under_it() {
    // The start's request is never answered, so the hints come with the request of the jump.
    let (_fixture, reader, x) = resting_place_with(400, &[("HINT_STEPS", "silent")]);
    let place = point(&reader, 15, x);
    move_to(&reader.window, place);
    idle(REST.as_millis() as u64);
    assert_eq!(rows(&reader).0, 0, "hints stood before the jump");
    // The first visible line is 90; the answer to the request of that view puts a hint on line 99.
    let top = reader.source.borrow().row_map.block_top(90);
    scroll_to(&reader.window, top);
    settle_tile(&reader.source);
    let before = line_drawn_under(&reader, place.y);
    assert_eq!(
        before % 2,
        1,
        "a character stood under the pointer before the hints"
    );
    let mut waiting = Waiting::language();
    loop {
        {
            let current = reader.source.borrow();
            if current
                .annotations
                .hints(displayed(&current))
                .is_some_and(|held| return held.first_line > 0)
            {
                break;
            }
        }
        waiting.step("the hint answer of the jump did not arrive");
    }
    let text = wait_for_popup(
        &reader,
        "no hover appeared for the line the hint rows put under the pointer",
    );
    assert_ne!(
        line_drawn_under(&reader, place.y),
        before,
        "the hints did not move the text under the pointer"
    );
    assert_names_line_drawn_under(&reader, place.y, &text);
}

/// A rest over no character is due only after the view moved under it,
///  and the rest time after that move;
///  a pointer that moves again is the start of a new rest.
#[test]
fn a_rest_over_no_character_is_due_only_after_the_view_moved() {
    let spot = Spot {
        below_top: 100.0,
        x: 20.0,
        offset: 500.0,
        extent: 9000.0,
    };
    let mut rest = Rest::default();
    rest.moved(None, spot);
    rest.at = Some((None, Instant::now() - REST * 2));
    assert_eq!(rest.due(), None, "a settled rest over nothing is due");
    assert!(rest.idle(), "a settled rest over nothing is not idle");
    rest.follow(View {
        offset: 620.0,
        extent: 9000.0,
    });
    assert_eq!(rest.due(), None, "the move did not start the rest again");
    assert!(!rest.idle(), "a moved view left the rest over nothing idle");
    rest.at = Some((None, Instant::now() - REST * 2));
    assert_eq!(
        rest.due(),
        Some(spot),
        "a moved view over nothing is not due"
    );
    // Nothing found: the rest is over nothing in a settled view again.
    rest.found_nothing();
    rest.at = Some((None, Instant::now() - REST * 2));
    assert_eq!(rest.due(), None, "a rest that found nothing is due again");
    assert!(!rest.asked, "a rest that found nothing is marked asked");
    // A move of the pointer to another place over nothing keeps the rest.
    rest.follow(View {
        offset: 700.0,
        extent: 9000.0,
    });
    rest.moved(None, spot);
    rest.at = Some((None, Instant::now() - REST * 2));
    assert_eq!(
        rest.due(),
        Some(spot),
        "a pointer move over nothing forgot the moved view"
    );
    // A pointer that arrives on a character starts a new rest on it.
    rest.moved(Some(3), spot);
    assert_eq!(rest.due(), None, "a new rest is due at once");
    // A pointer that arrives over nothing after a moved view starts a rest in a settled view.
    rest.follow(View {
        offset: 800.0,
        extent: 9000.0,
    });
    rest.moved(None, spot);
    rest.at = Some((None, Instant::now() - REST * 2));
    assert_eq!(
        rest.due(),
        None,
        "a new rest over nothing was due as if the view had moved"
    );
}

/// A view that stands still does not start the rest of a pointer again:
///  the first tick after the move keeps the moment the pointer arrived.
#[test]
fn a_view_that_stands_still_keeps_the_moment_the_pointer_arrived() {
    let (_fixture, reader, x) = resting_place_with(400, &[("HINT_STEPS", SILENT)]);
    move_to(&reader.window, point(&reader, 4, x));
    let binding = reader.binding.as_ref().expect("the language binding");
    let arrived = binding.language.borrow().rest.at;
    assert!(arrived.is_some(), "the move did not start a rest");
    idle(100);
    assert_eq!(
        binding.language.borrow().rest.at,
        arrived,
        "a tick of a view that stood still started the rest again"
    );
}

/// A rest is asked about once:
///  after its popup is dismissed nothing opens again until the pointer moves to another character.
#[test]
fn a_rest_that_was_asked_about_asks_once() {
    let (_fixture, reader, x) = resting_place_with(400, &[("HINT_STEPS", SILENT)]);
    move_to(&reader.window, point(&reader, 4, x));
    let shown = wait_for_popup(&reader, "no hover appeared for the resting pointer");
    assert!(
        shown.starts_with("line=alpha004 beta"),
        "the popup is `{shown}`"
    );
    escape(&reader);
    idle(3 * REST.as_millis() as u64);
    assert_eq!(popup(&reader), "", "the same rest asked again");
}

/// A text that grows without a move of the offset (hints or messages arriving between the top of the view and the pointer)
///  puts other text under the pointer,
///  so it starts a rest again like a move of the offset.
#[test]
fn a_growing_text_restarts_a_rest_that_was_not_asked_for() {
    let spot = Spot {
        below_top: 100.0,
        x: 20.0,
        offset: 500.0,
        extent: 9000.0,
    };
    let mut rest = Rest::default();
    rest.moved(Some(7), spot);
    rest.at = Some((Some(7), Instant::now() - REST * 2));
    assert_eq!(rest.due(), Some(spot));
    // Within half a pixel is rounding.
    rest.follow(View {
        offset: 500.0,
        extent: 9000.3,
    });
    assert_eq!(rest.due(), Some(spot), "rounding restarted the rest");
    rest.follow(View {
        offset: 500.0,
        extent: 9052.0,
    });
    assert_eq!(rest.due(), None, "a taller text did not restart the rest");
    let mut blank = Rest::default();
    blank.moved(None, spot);
    blank.follow(View {
        offset: 500.0,
        extent: 9052.0,
    });
    blank.at = Some((None, Instant::now() - REST * 2));
    assert_eq!(
        blank.due(),
        Some(spot),
        "a taller text did not make a rest over nothing due"
    );
}
