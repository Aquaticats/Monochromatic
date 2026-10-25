//! A wheel turn faster than the tile worker.
//!  The toolkit eases a wheel distance in a fixed 180 ms,
//!  however long it is,
//!  so a turn of many notches sweeps the view past the displayed tile before any tile built for a step can land.
//!  The window draws the view at the nearest offset the displayed tile holds until the tile for the real offset lands
//!  (the hold),
//!  and asks the worker for one tile at the end of the ease and for none at the steps on the way.
//! The turn is delivered as the application's event hook delivers it
//!  (`wheel::note_turn`,
//!  which the headless platform has no winit window to call)
//!  followed by the toolkit's own wheel event,
//!  so the toolkit eases it for real.

/// The frame the window draws,
///  and the platform the window is drawn on.
use super::gpu_snapshot::snapshot;
/// Settling a tile,
///  scrolling the window,
///  and the counters of the worker.
use super::overscan_tile_tests::{reset, scroll_to, settle_tile, worker_counts};
/// The fixture the window is opened with,
///  and the waiting of every test.
use super::{
    AppWindow, State,
    minimap_tests::{lift, press},
    sidebar_tests::{HEADER, TEXT_LEFT},
    waiting_tests::Waiting,
    wheel::{is_aimed, note_turn},
    wrap_tests::{open, settled},
};
/// The height of one code row.
use ide_app::row_map::CODE_ROW;
/// The window handle and the event types.
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{WindowEvent, update_timers_and_animations},
};
/// Shared state and the clock.
use std::{cell::RefCell, rc::Rc, time::Duration};

/// Lines of the fixture text;
///  the offsets used stay far from both ends.
const LINES: usize = 1500;

/// The offset the view stands at before a turn.
const START: f32 = 12_000.0;

/// The ease of a wheel turn, in the toolkit's own time.
const EASE: Duration = Duration::from_millis(180);

/// Logical pixels of one wheel notch.
const NOTCH: f32 = 60.0;

/// The text:
///  `LINES` numbered lines of one length each.
fn numbered() -> String {
    let mut text = String::new();
    for index in 0..LINES {
        text.push_str(&format!("line {index:04}: value\n"));
    }
    return text;
}

/// A point over the text area of the window,
///  `down` logical pixels below the top of the view.
fn over_text(down: f32) -> LogicalPosition {
    return LogicalPosition::new(TEXT_LEFT + 40.0, HEADER + down);
}

/// The toolkit's ease of a wheel turn of `distance` logical pixels at `elapsed`:
///  the share of the distance covered after `elapsed` of its fixed 180 ms,
///  at a constant deceleration (`ConstantDecelerationParameters::new_with_distance` in Slint 1.18.1).
fn eased(elapsed: Duration) -> f32 {
    let share = (elapsed.as_secs_f32() / 0.18).min(1.0);
    return 1.0 - (1.0 - share) * (1.0 - share);
}

/// Begin a wheel turn of `notches` notches (negative scrolls down) the way the application sees one:
///  the event hook notes the turn,
///  and the scroll reports that it took it.
/// The public window event of the toolkit carries no gesture phase and moves the scroll at once,
///  so the ease itself is stepped here,
///  by `step_ease`,
///  along the curve the toolkit's constant deceleration draws.
fn begin_turn(window: &AppWindow, state: &Rc<RefCell<State>>, notches: f32) -> (f32, f32) {
    let distance = notches * NOTCH;
    let start = state.borrow().offset;
    update_timers_and_animations();
    note_turn(state, distance);
    window.invoke_source_flicked();
    return (start, distance);
}

/// Step the toolkit's ease of a turn that `begin_turn` began from `from` to its end,
///  one step each 12 ms of the ease's own time,
///  however long the host takes for a step:
///  the ease is the toolkit's clock,
///  and a test that stepped it by the host's clock would see a different ease under load.
fn step_ease(window: &AppWindow, turn: (f32, f32), from: Duration) {
    step_ease_each(window, turn, from, || {});
}

/// `step_ease`,
///  calling `after_step` after each step,
///  with the window's frame still to be drawn.
fn step_ease_each(
    window: &AppWindow,
    turn: (f32, f32),
    from: Duration,
    mut after_step: impl FnMut(),
) {
    let (start, distance) = turn;
    let mut elapsed = from;
    while elapsed < EASE {
        elapsed = (elapsed + Duration::from_millis(12)).min(EASE);
        window.set_scroll_y(-(start - distance * eased(elapsed)));
        update_timers_and_animations();
        after_step();
        std::thread::sleep(Duration::from_millis(4));
    }
}

/// How many pixels of the window's frame,
///  between logical columns `from` and `to` of the code area,
///  differ clearly
/// from the background beside them.
fn ink_between(window: &AppWindow, from: f32, to: f32) -> usize {
    let frame = snapshot(window.window());
    let scale = window.window().scale_factor();
    let width = frame.width() as usize;
    let pixels = frame.as_slice();
    let top = (HEADER * scale) as usize;
    let bottom = top + (window.get_viewport_height() * scale) as usize;
    let background =
        pixels[(top + 2) * width + (TEXT_LEFT * scale) as usize + (240.0 * scale) as usize];
    let mut count = 0;
    for row in top..bottom.min(frame.height() as usize) {
        for column in (from * scale) as usize..(to * scale) as usize {
            let pixel = pixels[row * width + column];
            let distance = pixel.r.abs_diff(background.r) as u32
                + pixel.g.abs_diff(background.g) as u32
                + pixel.b.abs_diff(background.b) as u32;
            if distance > 120 {
                count += 1;
            }
        }
    }
    return count;
}

/// The ink of the text,
///  and nothing for a blank code area.
fn ink(window: &AppWindow) -> usize {
    return ink_between(window, TEXT_LEFT, TEXT_LEFT + 200.0);
}

/// The ink of the line numbers in the gutter,
///  and nothing for a blank gutter.
fn gutter_ink(window: &AppWindow) -> usize {
    return ink_between(window, TEXT_LEFT - 40.0, TEXT_LEFT - 12.0);
}

/// What:
///  The line the caret stands on,
///  from the document.
/// Why:
///  A press puts the caret on the row under the pointer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function caretLine(state: State): number;
/// ```
fn caret_line(state: &Rc<RefCell<State>>) -> usize {
    let current = state.borrow();
    let head = current.document.position().head;
    return current.document.text().char_to_line(head);
}

/// While the tile for the real offset has not landed the view is drawn at the nearest offset the displayed tile holds,
/// showing real rows of it and not a blank,
/// and at the real offset once the tile has landed.
#[test]
fn the_view_is_drawn_at_the_tile_edge_until_the_tile_for_the_real_offset_lands() {
    let (_project, reader) = open(&numbered());
    settled(&reader);
    let window = &reader.window;
    let state = &reader.source;
    let height = window.get_viewport_height();
    scroll_to(window, START);
    settle_tile(state);
    assert_eq!(
        window.get_drawn_offset(),
        START,
        "a tile that holds the view draws it at the real offset (positive control)"
    );
    assert!(!window.get_held(), "a view inside the tile is not held");
    assert!(
        ink(window) > 500,
        "the settled view shows text (positive control)"
    );
    assert!(
        gutter_ink(window) > 100,
        "the settled gutter shows line numbers (positive control)"
    );
    let reach_bottom = window.get_hold_bottom();
    assert!(
        reach_bottom > START + height && reach_bottom < START + 4.0 * height,
        "the reach of the tile ends within three view heights of the view, not at {reach_bottom}"
    );
    // The set offset is far past the tile; the worker has not answered yet, and no timer has run.
    let real = START + 6.0 * height;
    scroll_to(window, real);
    assert_eq!(
        window.get_drawn_offset(),
        reach_bottom - height,
        "the view is drawn at the last offset the displayed tile holds"
    );
    assert!(window.get_held(), "the window says the view is held");
    assert!(
        -window.get_scroll_y() > window.get_drawn_offset() + 2.0 * height,
        "the toolkit's scroll is far past the drawn view"
    );
    assert!(
        ink(window) > 500,
        "the held view shows rows of the displayed tile, not a blank code area"
    );
    assert!(
        gutter_ink(window) > 100,
        "the gutter of a held view shows the line numbers of the displayed tile"
    );
    settle_tile(state);
    assert_eq!(
        window.get_drawn_offset(),
        real,
        "the hold ends when the tile for the real offset lands"
    );
    assert!(!window.get_held(), "a landed tile holds the view again");
    assert!(ink(window) > 500, "the landed tile shows text");
    window.hide().expect("close the window");
}

/// A press on a held view lands on the row that is drawn under the pointer,
/// not on the row the toolkit's scroll would put there.
#[test]
fn a_press_on_a_held_view_lands_on_the_row_drawn_under_it() {
    let (_project, reader) = open(&numbered());
    settled(&reader);
    let window = &reader.window;
    let state = &reader.source;
    let height = window.get_viewport_height();
    scroll_to(window, START);
    settle_tile(state);
    // A press on the settled view puts the caret on the row under the pointer (positive control).
    let down = 100.0;
    press(&reader, over_text(down));
    lift(&reader, over_text(down));
    let at_rest = state.borrow().row_map.line_at(START + down);
    assert_eq!(
        caret_line(state),
        at_rest,
        "a press lands on the row under it"
    );
    scroll_to(window, START + 6.0 * height);
    assert!(window.get_held());
    let drawn = window.get_drawn_offset();
    let real = -window.get_scroll_y();
    let drawn_line = state.borrow().row_map.line_at(drawn + down);
    let real_line = state.borrow().row_map.line_at(real + down);
    assert_ne!(
        drawn_line, real_line,
        "the two readings of the press differ"
    );
    press(&reader, over_text(down));
    lift(&reader, over_text(down));
    assert_eq!(
        caret_line(state),
        drawn_line,
        "a press on a held view lands on the row drawn under the pointer"
    );
    window.hide().expect("close the window");
}

/// A drag that begins on a held view extends the selection to the row drawn under the pointer,
/// the way a drag of the text does on a settled view.
#[test]
fn a_drag_on_a_held_view_reaches_the_row_drawn_under_the_pointer() {
    let (_project, reader) = open(&numbered());
    settled(&reader);
    let window = &reader.window;
    let state = &reader.source;
    let height = window.get_viewport_height();
    scroll_to(window, START);
    settle_tile(state);
    scroll_to(window, START + 6.0 * height);
    assert!(window.get_held());
    let drawn = window.get_drawn_offset();
    let down = 100.0;
    let first = state.borrow().row_map.line_at(drawn + down);
    press(&reader, over_text(down));
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: over_text(down + 4.0 * CODE_ROW),
    });
    lift(&reader, over_text(down + 4.0 * CODE_ROW));
    let text = state.borrow().document.text().clone();
    let position = state.borrow().document.position();
    assert_eq!(
        (
            text.char_to_line(position.anchor.min(position.head)),
            text.char_to_line(position.anchor.max(position.head))
        ),
        (first, first + 4),
        "a drag on a held view spans the rows drawn under its two ends"
    );
    window.hide().expect("close the window");
}

/// New rows above the end of a followed ease move that end,
/// so the tile asked for is asked for again under the new map and the ease stays followed.
#[test]
fn rows_that_arrive_during_a_followed_ease_move_the_tile_asked_for_with_them() {
    let (_project, reader) = open(&numbered());
    settled(&reader);
    let window = &reader.window;
    let state = &reader.source;
    let width = window.get_viewport_width();
    let height = window.get_viewport_height();
    let scale = window.window().scale_factor();
    scroll_to(window, START);
    settle_tile(state);
    let _begun = begin_turn(window, state, -100.0);
    let end = START + 100.0 * NOTCH;
    let before = state.borrow().first;
    {
        let mut current = state.borrow_mut();
        // Two more rows in a line far above the view: everything below it moves down by 48 px.
        assert!(super::rows::absorb(&mut current, [(100, 3)]));
        super::rows::settle(&mut current, scale, (START, width, height), true);
    }
    let current = state.borrow();
    assert!(
        is_aimed(&current),
        "the ease is still followed after the rows arrived"
    );
    let wanted = current
        .scroll_wanted
        .expect("the tile of the end is still wanted");
    assert_eq!(
        wanted.first,
        current.row_map.line_at(end - height),
        "the tile asked for starts where the new map puts the line one view height above the end"
    );
    assert_ne!(wanted.first, before, "the window did ask again");
    drop(current);
    window.hide().expect("close the window");
}

/// A far wheel turn asks the worker for one tile,
///  at the end of the ease,
///  and for none at the steps on the way;
/// the tile lands while the ease runs and is applied,
///  and nothing is built on the window thread.
#[test]
fn a_far_wheel_turn_asks_for_one_tile_at_its_end() {
    let (_project, reader) = open(&numbered());
    settled(&reader);
    let window = &reader.window;
    let state = &reader.source;
    scroll_to(window, START);
    settle_tile(state);
    reset(state);
    let end = START + 100.0 * NOTCH;
    let begun = begin_turn(window, state, -100.0);
    assert!(
        is_aimed(&state.borrow()),
        "the window follows the ease to its end"
    );
    assert!(
        state.borrow().scroll_wanted.is_some(),
        "the tile of the end is wanted at once"
    );
    // The ease is stepped to its middle and held there until the worker has answered: the tile of the end must land
    // while the ease still runs, and the clock of the test does not decide whether it does.
    let (first, distance) = begun;
    for tenth in 1..=5 {
        let elapsed = Duration::from_millis(18 * tenth);
        window.set_scroll_y(-(first - distance * eased(elapsed)));
        update_timers_and_animations();
    }
    let mut waiting = Waiting::eventually();
    while worker_counts(state)[2] == 0 {
        waiting.step("the tile of the end did not land while the ease was still running");
    }
    assert!(
        is_aimed(&state.borrow()),
        "the tile of the end landed while the ease still ran"
    );
    assert!(
        window.get_held() || window.get_drawn_offset() == -window.get_scroll_y(),
        "the view is drawn at the landed tile's edge or inside it"
    );
    step_ease(window, begun, Duration::from_millis(90));
    settle_tile(state);
    let [worker_tiles, _rows, applied, discarded] = worker_counts(state);
    assert_eq!(
        worker_tiles, 1,
        "the worker builds the one tile of the end and none for the steps"
    );
    assert_eq!(applied, 1, "the tile of the end is applied once");
    assert_eq!(discarded, 0, "no tile is discarded");
    assert_eq!(
        state.borrow().tiles_painted,
        0,
        "the window thread paints no tile for the ease"
    );
    assert_eq!(-window.get_scroll_y(), end, "the ease ends at its offset");
    assert_eq!(
        window.get_drawn_offset(),
        end,
        "the view is drawn at its end"
    );
    assert!(!window.get_held());
    assert!(ink(window) > 500, "the end of the ease shows text");
    window.hide().expect("close the window");
}

/// A wheel turn the displayed tile holds,
///  in the middle of its reach,
///  asks for no tile and starts no follow;
/// a turn just past what it holds asks for one tile at its end.
#[test]
fn a_near_wheel_turn_asks_for_no_tile_and_a_turn_past_the_tile_for_one() {
    let (_project, reader) = open(&numbered());
    settled(&reader);
    let window = &reader.window;
    let state = &reader.source;
    scroll_to(window, START);
    settle_tile(state);
    reset(state);
    // Five notches, 300 px, stay inside the 0.75 view heights the displayed tile lets the view travel.
    let begun = begin_turn(window, state, -5.0);
    assert!(
        !is_aimed(&state.borrow()),
        "a turn the tile holds is not followed"
    );
    step_ease(window, begun, Duration::ZERO);
    assert!(state.borrow().scroll_wanted.is_none());
    settle_tile(state);
    assert_eq!(
        worker_counts(state)[0],
        0,
        "a turn inside the tile asks the worker for nothing"
    );
    assert!(!window.get_held());
    // 40 notches, 2,400 px, take the view past the re-center edge of the tile.
    let past_begun = begin_turn(window, state, -40.0);
    assert!(
        is_aimed(&state.borrow()),
        "a turn past the tile is followed"
    );
    step_ease(window, past_begun, Duration::ZERO);
    settle_tile(state);
    assert_eq!(
        worker_counts(state)[0],
        1,
        "a turn past the tile asks for one tile"
    );
    window.hide().expect("close the window");
}

/// A placement ends the follow:
///  the view is set,
///  not eased,
///  so the next steps ask for their own tiles again.
#[test]
fn a_placement_ends_the_followed_ease() {
    let (_project, reader) = open(&numbered());
    settled(&reader);
    let window = &reader.window;
    let state = &reader.source;
    let width = window.get_viewport_width();
    let height = window.get_viewport_height();
    scroll_to(window, START);
    settle_tile(state);
    let _begun = begin_turn(window, state, -100.0);
    assert!(is_aimed(&state.borrow()));
    super::viewport::place(&mut state.borrow_mut(), START + 300.0, width, height);
    assert!(
        !is_aimed(&state.borrow()),
        "a placement releases the ease the window followed"
    );
    window.hide().expect("close the window");
}

/// Every frame of a far ease shows text:
/// the frames before the tile of the end lands show the displayed tile at its edge,
/// the frames after show the tile of the end at its near edge and then the ease arriving in it.
#[test]
fn no_frame_of_a_far_ease_is_blank() {
    let (_project, reader) = open(&numbered());
    settled(&reader);
    let window = &reader.window;
    let state = &reader.source;
    scroll_to(window, START);
    settle_tile(state);
    let begun = begin_turn(window, state, -300.0);
    let mut frames = 0;
    let mut held = 0;
    let mut blank = Vec::new();
    step_ease_each(window, begun, Duration::ZERO, || {
        frames += 1;
        if window.get_held() {
            held += 1;
        }
        if ink(window) < 500 {
            blank.push((frames, -window.get_scroll_y(), window.get_drawn_offset()));
        }
    });
    assert!(held > 0, "the ease outran the tile, so frames were held");
    assert!(
        blank.is_empty(),
        "frames of the ease showed no text: {blank:?}"
    );
    settle_tile(state);
    assert!(
        !window.get_held(),
        "the ease ends on a tile that holds the view"
    );
    window.hide().expect("close the window");
}
