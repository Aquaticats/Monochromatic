//! The minimap beside the source text in the real window:
//!  the viewport box against the geometry,
//! a press that jumps,
//! a drag of the box,
//! the wheel,
//!  the keyboard after a press,
//! and the worker that builds the picture's scene off the window thread.
//! Nothing here waits on the window thread:
//! every wait advances timers and lets the workers answer.

/// The source reader fixture,
/// its disposable project,
/// and the bounded waits.
use super::find_tests::{Reader, key, memory_project, reader, replace_file};
/// Window metrics shared with the layout tests.
use super::sidebar_tests::{HEADER, TEXT_LEFT};
/// The long wrapped text,
/// the waits,
/// and the window's measured state.
use super::wrap_tests::{long_text, open, settled};
/// The worker's counters,
/// its gate,
/// its reply type,
/// and the accept rule.
use super::{
    AppWindow,
    minimap::{DEBOUNCE, Plan, SLIDE, THROTTLE, apply, plan},
    minimap_worker::{Key as PictureKey, MINIMAP_GATE, OFF_WORKER_BUILDS, Reply},
    overscan_tile_tests::{scroll_to, settle_tile},
    theme_tests::switch,
    tile_reads::{draw_minimap, minimap_pixels},
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Library types that give the expected values.
use ide_app::minimap::{BAND, Geometry, Heading, LEAD, band_count};
/// An empty scene stands for a finished picture in the accept rule's tests.
use ide_app::tile_scene::TileScene;
/// Toolkit events,
/// models,
/// and images.
use slint::{
    ComponentHandle, LogicalPosition, SharedPixelBuffer,
    platform::{Key, PointerEventButton, WindowEvent, update_timers_and_animations},
    private_unstable_api::re_exports::ColorScheme,
};
/// Counters,
/// clocks,
/// and files.
use std::{
    fs,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

/// What:
///  A window position over the minimap,
/// `y` logical pixels from the top of the view and `x` from the minimap's left
///  edge.
/// Why:
///  The minimap stays still on screen while the content scrolls under it,
/// so its place is fixed by the layout:
///  right of the gutter and the text area.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function overMinimap(window: AppWindow, x: number, y: number): Position;
/// ```
pub(super) fn over(window: &super::AppWindow, x: f32, y: f32) -> LogicalPosition {
    return LogicalPosition::new(TEXT_LEFT + window.get_viewport_width() + x, HEADER + y);
}

/// The geometry of the text the reader shows now,
/// from the vertical mapping and the view's height.
pub(super) fn geometry(reader: &Reader) -> Geometry {
    return Geometry::new(
        reader.source.borrow().row_map.height(),
        reader.window.get_viewport_height(),
    );
}

/// The anchor and the head of the selection,
/// without the scroll position the reading position also holds.
pub(super) fn reading(reader: &Reader) -> (usize, usize) {
    let position = reader.source.borrow().document.position();
    return (position.anchor, position.head);
}

/// The scroll offset of the view.
pub(super) fn offset(reader: &Reader) -> f32 {
    return -reader.window.get_scroll_y();
}

/// Press the left button at `at`.
pub(super) fn press(reader: &Reader, at: LogicalPosition) {
    reader
        .window
        .window()
        .dispatch_event(WindowEvent::PointerPressed {
            position: at,
            button: PointerEventButton::Left,
        });
}

/// Move the pointer to `at` with the button as it is.
fn travel(reader: &Reader, at: LogicalPosition) {
    reader
        .window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: at });
}

/// Release the left button at `at`.
pub(super) fn lift(reader: &Reader, at: LogicalPosition) {
    reader
        .window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: at,
            button: PointerEventButton::Left,
        });
}

/// Let every pending render,
/// timer,
/// and worker answer that is already due run.
pub(super) fn calm(reader: &Reader) {
    settle_tile(&reader.source);
    update_timers_and_animations();
}

/// Wait until a picture for the displayed text is on screen and no request is awaited,
/// and the GPU has drawn it.
fn picture_shown(reader: &Reader) {
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = reader.source.borrow();
                if current.minimap.shown.is_some()
                    && current.minimap.posted == current.minimap.shown
                    && current.minimap.images_set > 0
                {
                    break;
                }
            }
            waiting.step("the minimap picture did not arrive");
        }
    }
    draw_minimap(&reader.window);
}

/// The box and the picture's slide match `ide_app::minimap::Geometry` at every scroll offset,
/// and track the scroll live:
/// the first offset puts
///  it at the top,
/// the last at the bottom of the track.
#[test]
fn the_box_matches_the_geometry_at_every_offset() {
    let (_project, opened) = open(&long_text(400));
    settled(&opened);
    let window = &opened.window;
    let shape = geometry(&opened);
    assert!(
        shape.max_offset() > 1000.0,
        "the text is longer than the view"
    );
    assert!(
        shape.slide(shape.max_offset()) > 100.0,
        "the picture of this text is taller than the view and slides"
    );
    for wanted in [
        0.0,
        1.0,
        377.0,
        shape.max_offset() / 2.0,
        shape.max_offset(),
    ] {
        scroll_to(window, wanted);
        calm(&opened);
        let at = offset(&opened);
        assert!(
            (window.get_minimap_box_top() - shape.box_top(at)).abs() < 0.01,
            "box top {} against {} at offset {at}",
            window.get_minimap_box_top(),
            shape.box_top(at)
        );
        assert!((window.get_minimap_box_height() - shape.box_height()).abs() < 0.01);
        assert!((window.get_minimap_track() - shape.track()).abs() < 0.01);
        assert!(
            (window.get_minimap_slide() - shape.slide(at)).abs() < 0.01,
            "the picture's slide {} against {} at offset {at}",
            window.get_minimap_slide(),
            shape.slide(at)
        );
    }
    scroll_to(window, 0.0);
    assert_eq!(
        window.get_minimap_box_top(),
        0.0,
        "the first offset puts the box at the top"
    );
    scroll_to(window, shape.max_offset());
    assert!(
        (window.get_minimap_box_top() + window.get_minimap_box_height()
            - window.get_minimap_track())
        .abs()
            < 0.01,
        "the last offset puts the box at the bottom of the track"
    );
    window.hide().expect("close minimap window");
}

/// A text shorter than the view has a box as tall as the track,
/// no room to move it,
/// and a press that scrolls nothing.
#[test]
fn a_short_text_has_a_box_as_tall_as_its_track_and_a_press_scrolls_nothing() {
    let (_project, opened) = open("fn main() {\n    println!(\"hi\");\n}\n");
    settled(&opened);
    let window = &opened.window;
    let shape = geometry(&opened);
    assert_eq!(shape.max_offset(), 0.0);
    assert_eq!(window.get_minimap_box_top(), 0.0);
    assert!((window.get_minimap_box_height() - window.get_minimap_track()).abs() < 0.01);
    assert!(
        window.get_minimap_track() < window.get_viewport_height(),
        "the picture of three lines keeps its natural scale and does not fill the view"
    );
    press(&opened, over(window, 10.0, 5.0));
    lift(&opened, over(window, 10.0, 5.0));
    calm(&opened);
    assert_eq!(offset(&opened), 0.0);
    window.hide().expect("close minimap window");
}

/// A press outside the box moves the caret to the row under the pointer and scrolls that row to the middle of the view;
///  a Shift press extends the selection from the old caret;
/// the keyboard still scrolls afterwards.
#[test]
fn a_press_outside_the_box_jumps_to_the_row_and_the_keyboard_still_works() {
    let (_project, opened) = open(&long_text(400));
    settled(&opened);
    let window = &opened.window;
    let shape = geometry(&opened);
    let y = shape.track() * 0.6;
    let (target, expected) = {
        let current = opened.source.borrow();
        let found = shape.target(&current.row_map, y, offset(&opened));
        let top = current.row_map.code_top(found.line) + found.row as f32 * 24.0;
        (found, shape.centered(top + 12.0).round())
    };
    assert!(
        y > window.get_minimap_box_top() + window.get_minimap_box_height(),
        "the pressed place is outside the box"
    );
    press(&opened, over(window, 10.0, y));
    lift(&opened, over(window, 10.0, y));
    calm(&opened);
    assert_eq!(offset(&opened), expected, "the pressed row is centered");
    {
        let current = opened.source.borrow();
        let line = current
            .document
            .text()
            .char_to_line(current.document.position().head);
        assert_eq!(line, target.line, "the caret is on the pressed row's line");
        assert_eq!(
            current.document.position().anchor,
            current.document.position().head
        );
    }
    let anchor = opened.source.borrow().document.position().head;
    // A place well above the row the first press centered, outside the box that now stands over it.
    let second = {
        let current = opened.source.borrow();
        let row = shape.y_of_row(&current.row_map, target, offset(&opened));
        (row - 150.0).max(2.0)
    };
    assert!(
        second < window.get_minimap_box_top(),
        "the second pressed place is outside the box"
    );
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    press(&opened, over(window, 10.0, second));
    lift(&opened, over(window, 10.0, second));
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    calm(&opened);
    {
        let current = opened.source.borrow();
        let position = current.document.position();
        assert_eq!(position.anchor, anchor, "a Shift press keeps the anchor");
        assert!(
            position.head < anchor,
            "and extends the selection up to the pressed row"
        );
    }
    let before = offset(&opened);
    key(window, Key::PageDown);
    calm(&opened);
    assert!(
        offset(&opened) > before || opened.source.borrow().document.position().head > anchor,
        "PageDown after a press on the minimap moves the view"
    );
    window.hide().expect("close minimap window");
}

/// A press across the minimap puts the caret in the same share of the line's width:
/// the picture is the text's whole width shrunk to the minimap's.
#[test]
fn a_press_across_the_minimap_puts_the_caret_in_the_same_share_of_the_text() {
    let (_project, opened) = open(&long_text(400));
    settled(&opened);
    let window = &opened.window;
    let shape = geometry(&opened);
    let y = shape.track() * 0.2;
    let width = window.get_minimap_width();
    let mut columns = Vec::new();
    for share in [0.05_f32, 0.3, 0.6, 0.95] {
        // A press on the box starts a drag, and a press moves the view,
        // so the view is first put where the box is away from the pressed place and the same row lies under it.
        scroll_to(window, shape.max_offset() / 2.0);
        calm(&opened);
        assert!(
            y < window.get_minimap_box_top()
                || y > window.get_minimap_box_top() + window.get_minimap_box_height(),
            "the pressed place is outside the box"
        );
        press(&opened, over(window, width * share, y));
        lift(&opened, over(window, width * share, y));
        calm(&opened);
        let current = opened.source.borrow();
        let head = current.document.position().head;
        let line = current.document.text().char_to_line(head);
        columns.push(head - current.document.text().line_to_char(line));
    }
    let mut grows = true;
    for pair in columns.windows(2) {
        if pair[0] >= pair[1] {
            grows = false;
        }
    }
    assert!(
        grows,
        "the caret columns {columns:?} do not grow across the minimap"
    );
    window.hide().expect("close minimap window");
}

/// A press on the box starts a drag without a jump;
/// the box follows the pointer one to one,
/// the view scrolls at every
///  step,
/// both ends clamp,
/// and after the button is released the pointer moves nothing.
#[test]
fn dragging_the_box_scrolls_continuously_and_release_ends_it() {
    let (_project, opened) = open(&long_text(400));
    settled(&opened);
    let window = &opened.window;
    let shape = geometry(&opened);
    scroll_to(window, shape.max_offset() / 4.0);
    calm(&opened);
    let start_offset = offset(&opened);
    let caret = reading(&opened);
    let grabbed = window.get_minimap_box_top() + window.get_minimap_box_height() / 2.0;
    press(&opened, over(window, 10.0, grabbed));
    assert_eq!(
        offset(&opened),
        start_offset,
        "a press on the box does not jump"
    );
    assert_eq!(reading(&opened), caret, "nor does it move the caret");
    let mut last = start_offset;
    for step in 1..=10 {
        let y = grabbed + step as f32 * 7.0;
        travel(&opened, over(window, 10.0, y));
        update_timers_and_animations();
        let at = offset(&opened);
        assert_eq!(
            at,
            shape.drag_offset(start_offset, grabbed, y).round(),
            "step {step}"
        );
        assert!(at > last, "the view moved at step {step}");
        assert!(
            (window.get_minimap_box_top() - shape.box_top(at)).abs() < 0.01,
            "the box follows the view at step {step}"
        );
        last = at;
    }
    travel(&opened, over(window, 10.0, shape.track() + 400.0));
    assert_eq!(
        offset(&opened),
        shape.max_offset(),
        "below the track the view stops at its end"
    );
    travel(&opened, over(window, 10.0, -400.0));
    assert_eq!(
        offset(&opened),
        0.0,
        "above the track the view stops at its start"
    );
    lift(&opened, over(window, 10.0, -400.0));
    travel(&opened, over(window, 10.0, 60.0));
    // A drag report that arrives after the release, as a toolkit may deliver one, is ignored by the native side too.
    window.invoke_minimap_drag(shape.track() * 0.9);
    assert_eq!(
        offset(&opened),
        0.0,
        "after the release the pointer moves nothing"
    );
    assert_eq!(reading(&opened), caret, "a drag never moves the caret");
    calm(&opened);
    window.hide().expect("close minimap window");
}

/// The box is an outline without a fill:
///  its border changes in width and opacity when the pointer is on it,
/// the picture inside it never changes,
/// and nothing changes while the pointer is beside it.
#[test]
fn the_box_is_an_outline_whose_border_changes_under_the_pointer() {
    let (release, gate) = std::sync::mpsc::channel();
    *MINIMAP_GATE.lock().expect("minimap gate") = Some(gate);
    let (_project, opened) = open(&long_text(400));
    settled(&opened);
    let window = &opened.window;
    let top = window.get_minimap_box_top();
    let height = window.get_minimap_box_height();

    // The box stands at the top of the track at the first offset, so its border rows are exact.
    assert_eq!(top, 0.0);
    let text_area = LogicalPosition::new(TEXT_LEFT + 100.0, HEADER + 100.0);
    let below = top + height + 60.0;
    travel(&opened, text_area);
    update_timers_and_animations();
    assert!(
        !window.get_minimap_hovered(),
        "the pointer is not over the minimap"
    );
    let rest = window.window().take_snapshot().expect("a frame");
    let plain = pixel_at(window, &rest, 40.0, below);
    let inside_rest = pixel_at(window, &rest, 40.0, height / 2.0);
    assert_ne!(
        pixel_at(window, &rest, 40.0, 0.0),
        pixel_at(window, &rest, 40.0, 1.0),
        "the box has a border"
    );
    assert_eq!(
        pixel_at(window, &rest, 40.0, 1.0),
        pixel_at(window, &rest, 40.0, 4.0),
        "the border at rest is one pixel wide"
    );
    // A pointer over the minimap but outside the box leaves the box as it is at rest: CodeGlance Pro marks the box only
    // while the pointer is on it.
    travel(&opened, over(window, 40.0, below));
    update_timers_and_animations();
    assert!(
        window.get_minimap_hovered(),
        "the pointer is over the minimap"
    );
    assert!(!window.get_minimap_over_box(), "but not over the box");
    let beside = window.window().take_snapshot().expect("a frame");
    assert_eq!(pixel_at(window, &beside, 40.0, below), plain);
    assert_eq!(
        pixel_at(window, &beside, 40.0, height / 2.0),
        inside_rest,
        "the inside does not change while the pointer is beside the box"
    );
    assert_eq!(
        pixel_at(window, &beside, 40.0, 0.0),
        pixel_at(window, &rest, 40.0, 0.0)
    );
    assert_eq!(
        pixel_at(window, &beside, 40.0, 1.0),
        pixel_at(window, &rest, 40.0, 1.0)
    );
    // On the box, the border widens and the inside stays as it is.
    travel(&opened, over(window, 40.0, height / 2.0));
    update_timers_and_animations();
    assert!(window.get_minimap_over_box(), "the pointer is over the box");
    let hovered = window.window().take_snapshot().expect("a frame");
    assert_eq!(
        pixel_at(window, &hovered, 40.0, below),
        plain,
        "the plain minimap does not change under the pointer"
    );
    assert_eq!(
        pixel_at(window, &hovered, 40.0, height / 2.0),
        inside_rest,
        "hover does not fill the box"
    );
    assert_eq!(
        pixel_at(window, &hovered, 40.0, 0.0),
        pixel_at(window, &hovered, 40.0, 1.0),
        "hover makes the border two pixels wide"
    );
    assert_ne!(
        pixel_at(window, &hovered, 40.0, 1.0),
        pixel_at(window, &hovered, 40.0, 4.0)
    );
    drop(release);
    window.hide().expect("close minimap window");
}

/// A wheel turn over the minimap scrolls the view exactly as one over the text.
#[test]
fn the_wheel_over_the_minimap_scrolls_like_the_wheel_over_the_text() {
    let (_project, opened) = open(&long_text(400));
    settled(&opened);
    let window = &opened.window;
    wheel(
        window,
        LogicalPosition::new(TEXT_LEFT + 100.0, HEADER + 100.0),
    );
    let over_text = finish(&opened);
    assert!(over_text > 0.0, "the wheel scrolled the text");
    scroll_to(window, 0.0);
    calm(&opened);
    wheel(window, over(window, 30.0, 100.0));
    let over_minimap = finish(&opened);
    assert_eq!(
        over_minimap, over_text,
        "the wheel over the minimap scrolled by the same amount"
    );
    window.hide().expect("close minimap window");
}

/// What:
///  Wait until the line measurements are done and the picture has stood still,
///  with the rows,
///  the picture shown,
///  and the picture posted unchanged for longer than the slowest pace.
/// Why:
///  The measurements of a long file keep changing the rows for a while,
///  and the picture follows them a pace later,
///  so a picture read earlier may be one built for rows that are gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function pictureStill(reader: Reader): Promise<void>;
/// ```
pub(super) fn picture_still(reader: &Reader) {
    settled(reader);
    let mut waiting = Waiting::slow();
    let mut seen = None;
    let mut since = Instant::now();
    loop {
        let now = {
            let current = reader.source.borrow();
            (
                current.row_map.height().to_bits(),
                current.minimap.shown,
                current.minimap.posted,
            )
        };
        if seen != Some(now) {
            seen = Some(now);
            since = Instant::now();
        } else if since.elapsed() > Duration::from_millis(800) && now.1 == now.2 {
            return;
        }
        waiting.step("the picture did not stand still");
    }
}

/// How many bands the whole text of the reader's window is cut into.
pub(super) fn bands_of(reader: &Reader) -> usize {
    return band_count(reader.source.borrow().row_map.height(), BAND);
}

/// The picture is built on the worker thread,
/// its texture is as wide as the minimap,
/// it holds the bands near the visible part of the text,
/// and what it cost stays inside its bounds.
#[test]
fn the_picture_is_built_on_the_worker_and_stays_inside_its_bounds() {
    let (_project, opened) = open(&long_text(2000));
    settled(&opened);
    picture_shown(&opened);
    let window = &opened.window;
    assert_eq!(
        OFF_WORKER_BUILDS.load(Ordering::SeqCst),
        0,
        "a picture was built off the worker"
    );
    let image = minimap_pixels(window, &opened.source).expect("picture pixels");
    let stats = opened
        .source
        .borrow()
        .minimap
        .last_stats
        .expect("build statistics");
    assert_eq!(image.width() as f32, window.get_minimap_width());
    assert_eq!(image.width() as f32, window.get_minimap_picture_width());
    assert_eq!(image.height() as f32, window.get_minimap_picture_height());
    let scale = window.window().scale_factor();
    assert!(
        image.height() as f32 + 1.0 >= window.get_minimap_track() * scale,
        "the picture covers the track"
    );
    assert!(
        stats.units <= 12 && stats.units * 3 < bands_of(&opened),
        "{} of {} bands were built for one view",
        stats.units,
        bands_of(&opened)
    );
    assert_eq!(
        stats.shaped + stats.reused + stats.skipped,
        stats.lines,
        "every line of the picture was shaped, taken from the row cache, or left blank"
    );
    assert_eq!(stats.skipped, 0, "no line was left out");
    assert!(
        stats.lines <= stats.units * 40,
        "{} lines for {} bands",
        stats.lines,
        stats.units
    );
    let mut inked = false;
    for pixel in image.as_slice().iter() {
        if pixel.a > 0 {
            inked = true;
        }
    }
    assert!(inked, "the picture holds ink");
    window.hide().expect("close minimap window");
}

/// The bands a scroll to `offset` shows,
/// in the reader's window,
/// with the heading the window has now.
fn bands_at(reader: &Reader, offset: f32) -> std::ops::Range<usize> {
    let shape = geometry(reader);
    let heading = reader.source.borrow().minimap.heading;
    // A track taller than one texture is stacked from several, so no band the track shows is left out.
    return shape.bands_toward(offset, BAND, usize::MAX, heading);
}

/// The scroll offset at which the top of the track is `top` logical pixels down the text.
fn offset_of_top(shape: &Geometry, top: f32) -> f32 {
    let overhang = shape.slide(shape.max_offset());
    return shape.max_offset() * (top * shape.scale() / overhang);
}

/// The numbers of the bands a picture holds.
fn bands_of_key(key: PictureKey) -> std::ops::Range<usize> {
    return key.window.0..key.window.1;
}

/// Wait until the picture on screen is the one for `bands` and no newer request is posted.
fn picture_of_bands(reader: &Reader, bands: &std::ops::Range<usize>) {
    let mut waiting = Waiting::slow();
    loop {
        {
            let current = reader.source.borrow();
            let shown = current.minimap.shown.map(bands_of_key);
            if shown.as_ref() == Some(bands) && current.minimap.posted == current.minimap.shown {
                break;
            }
        }
        waiting.step("the picture of the scrolled bands did not arrive");
    }
    draw_minimap(&reader.window);
}

/// A scroll that keeps the bands the track shows posts no request and sets no picture:
/// only the box and the picture's slide move.
/// A scroll that names other bands posts one request and sets one picture,
/// which starts further down the whole picture.
#[test]
fn a_scroll_posts_a_request_only_when_it_names_other_bands() {
    let (_project, opened) = open(&long_text(2000));
    settled(&opened);
    picture_shown(&opened);
    let window = &opened.window;
    let shape = geometry(&opened);
    // The stretch of offsets from the middle on that shows the same bands as the middle.
    let start = (shape.max_offset() / 2.0).round();
    scroll_to(window, start);
    let named = bands_at(&opened, start);
    picture_of_bands(&opened, &named);
    calm(&opened);
    let mut edge = start;
    while bands_at(&opened, edge + 4.0) == named {
        edge += 4.0;
    }
    assert!(
        edge > start + 8.0,
        "the stretch of the same bands is only {} px",
        edge - start
    );
    let (posts, images, top, drawn) = {
        let current = opened.source.borrow();
        (
            current.minimap.posts,
            current.minimap.images_set,
            window.get_minimap_picture_top() + window.get_minimap_picture_height(),
            current.gpu.minimaps_rendered(),
        )
    };
    let slide = window.get_minimap_slide();
    let first = window.get_minimap_picture_top();
    let mut at = start;
    while at < edge {
        at += 4.0;
        scroll_to(window, at.min(edge));
        calm(&opened);
    }
    for _ in 0..30 {
        update_timers_and_animations();
        std::thread::sleep(Duration::from_millis(10));
    }
    {
        let current = opened.source.borrow();
        assert_eq!(
            current.minimap.posts, posts,
            "a scroll within the bands posted a minimap request"
        );
        assert_eq!(
            current.minimap.images_set, images,
            "a scroll within the bands set a minimap picture"
        );
    }
    draw_minimap(window);
    assert_eq!(
        opened.source.borrow().gpu.minimaps_rendered(),
        drawn,
        "a scroll within the bands rendered a slice on the GPU"
    );
    assert!(
        window.get_minimap_slide() > slide,
        "the picture slid with the scroll"
    );
    assert_eq!(
        window.get_minimap_picture_top() + window.get_minimap_picture_height(),
        top,
        "the picture is as it was"
    );
    // Four pixels more name other bands: one request, one picture.
    scroll_to(window, edge + 4.0);
    let other = bands_at(&opened, edge + 4.0);
    assert_ne!(other, named);
    picture_of_bands(&opened, &other);
    calm(&opened);
    let current = opened.source.borrow();
    assert_eq!(
        current.minimap.posts,
        posts + 1,
        "one request for the new bands"
    );
    assert_eq!(
        current.minimap.images_set,
        images + 1,
        "one picture for the new bands"
    );
    assert_eq!(
        current.gpu.minimaps_rendered(),
        drawn + 1,
        "one slice rendered on the GPU for the new bands"
    );
    drop(current);
    // The slice moves down by a band at one end or at both, so it starts or ends further down the text.
    assert!(
        window.get_minimap_picture_top() > first
            || window.get_minimap_picture_top() + window.get_minimap_picture_height() > top,
        "the new picture reaches further down the text"
    );
    window.hide().expect("close minimap window");
}

/// A fling outruns a picture that holds one band beyond the track:
/// the view is a few bands further on before the next picture is built and drawn,
/// and the rows beyond the picture are blank.
/// The picture asked for during a fling holds `LEAD` bands on the side the view is heading toward,
/// so the track a few bands later is still inside it,
/// and one asked for after the view turns holds them on the other side.
#[test]
fn a_fling_is_given_a_picture_that_reaches_the_bands_it_is_heading_toward() {
    let (_project, opened) = open(&long_text(2000));
    settled(&opened);
    picture_shown(&opened);
    let window = &opened.window;
    let shape = geometry(&opened);
    let track_text = shape.track() / shape.scale();
    // The text the shown picture holds,
    // as the first and one past the last logical pixel of the text.
    let held = |reader: &Reader| {
        let shown = reader.source.borrow().minimap.shown.expect("a picture");
        return (shown.window.0 as f32 * BAND, shown.window.1 as f32 * BAND);
    };
    // Three steps of a band and a half each, as a fling that no picture keeps up with.
    let start = offset_of_top(&shape, 20.0 * BAND);
    scroll_to(window, start);
    for step in 1..=3 {
        scroll_to(
            window,
            offset_of_top(&shape, (20.0 + 1.5 * step as f32) * BAND),
        );
    }
    let down_top = (20.0 + 4.5) * BAND;
    let Heading::Down { at } = opened.source.borrow().minimap.heading else {
        panic!("a fling down is not heading down");
    };
    assert!(
        (at - down_top).abs() < 1.0,
        "heading down confirmed at {at}, not {down_top}"
    );
    picture_of_bands(&opened, &bands_at(&opened, offset_of_top(&shape, down_top)));
    calm(&opened);
    let (first, end) = held(&opened);
    let reached = down_top + (LEAD as f32 - 1.0) * BAND;
    assert!(
        first <= down_top && reached + track_text <= end,
        "the picture holds the text from {first} to {end}; the track {} bands further on needs {reached} to {}",
        LEAD - 1,
        reached + track_text
    );
    // The view turns and flings up: the lead is before the track now.
    for step in 1..=3 {
        scroll_to(
            window,
            offset_of_top(&shape, down_top - 1.5 * step as f32 * BAND),
        );
    }
    let up_top = down_top - 4.5 * BAND;
    let Heading::Up { at: turned_at } = opened.source.borrow().minimap.heading else {
        panic!("a fling up is not heading up");
    };
    assert!(
        (turned_at - up_top).abs() < 1.0,
        "heading up confirmed at {turned_at}, not {up_top}"
    );
    picture_of_bands(&opened, &bands_at(&opened, offset_of_top(&shape, up_top)));
    calm(&opened);
    let (first_up, end_up) = held(&opened);
    let reached_back = up_top - (LEAD as f32 - 1.0) * BAND;
    assert!(
        reached_back >= first_up && up_top + track_text <= end_up,
        "the picture holds the text from {first_up} to {end_up}; the track {} bands further back needs {reached_back} from",
        LEAD - 1
    );
    window.hide().expect("close minimap window");
}

/// A worker held at its gate never blocks the view:
/// the window opens,
/// scrolls,
/// and takes a press while no picture
///  exists,
/// and the picture appears once the worker is released.
#[test]
fn a_held_worker_never_blocks_the_view() {
    let (release, gate) = std::sync::mpsc::channel();
    *MINIMAP_GATE.lock().expect("minimap gate") = Some(gate);
    let (_project, opened) = open(&long_text(600));
    settled(&opened);
    let window = &opened.window;
    {
        let mut waiting = Waiting::slow();
        while !(opened.source.borrow().minimap.posts > 0) {
            waiting.step("the held worker was not asked for a picture");
        }
    }
    let shape = geometry(&opened);
    scroll_to(window, shape.max_offset() / 2.0);
    calm(&opened);
    assert_eq!(
        opened.source.borrow().minimap.images_set,
        0,
        "no picture while the worker is held"
    );
    let y = shape.track() * 0.8;
    press(&opened, over(window, 10.0, y));
    lift(&opened, over(window, 10.0, y));
    calm(&opened);
    assert!(
        offset(&opened) > shape.max_offset() / 2.0,
        "the press scrolled the view while the worker was held"
    );
    let _ = release.send(());
    picture_shown(&opened);
    assert_eq!(OFF_WORKER_BUILDS.load(Ordering::SeqCst), 0);
    drop(release);
    window.hide().expect("close minimap window");
}

/// A new wrap width rebuilds the picture without blocking the view,
/// and the picture then covers the new text.
#[test]
fn a_new_wrap_width_rebuilds_the_picture() {
    let (_project, opened) = open(&long_text(400));
    settled(&opened);
    picture_shown(&opened);
    let window = &opened.window;
    let before = opened.source.borrow().minimap.shown.expect("a picture key");
    let height = opened.source.borrow().row_map.height();
    super::sidebar_tests::resize(window, 760.0, 660.0);
    settled(&opened);
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                // The picture is for the new text and for the minimap's size as it now stands.
                let size = current.minimap.size;
                let followed = match current.minimap.shown {
                    Some(shown) => {
                        shown.rows != before.rows
                            && shown.size == (size.0.to_bits(), size.1.to_bits())
                    }
                    None => false,
                };
                if followed && current.minimap.shown == current.minimap.posted {
                    break;
                }
            }
            waiting.step("the picture did not follow the new width");
        }
    }
    let after = opened.source.borrow().minimap.shown.expect("a picture key");
    assert_ne!(
        after.wrap, before.wrap,
        "the picture is for the new wrap width"
    );
    assert_ne!(
        opened.source.borrow().row_map.height(),
        height,
        "the longer text is another height"
    );
    let image = minimap_pixels(window, &opened.source).expect("picture pixels");
    assert_eq!(image.width() as f32, window.get_minimap_width());
    assert!(
        image.height() as f32 + 1.0 >= window.get_minimap_track() * window.window().scale_factor()
    );
    assert_eq!(OFF_WORKER_BUILDS.load(Ordering::SeqCst), 0);
    window.hide().expect("close minimap window");
}

/// The pixel of a frame at the minimap's column `x` and row `y` below the header.
fn pixel_at(
    window: &AppWindow,
    frame: &SharedPixelBuffer<slint::Rgba8Pixel>,
    x: f32,
    y: f32,
) -> slint::Rgba8Pixel {
    let index = (HEADER + y) as usize * frame.width() as usize
        + (TEXT_LEFT + window.get_viewport_width() + x) as usize;
    return frame.as_slice()[index];
}

/// Turn the wheel over the window at `at` by one notch of 360 logical pixels.
fn wheel(window: &AppWindow, at: LogicalPosition) {
    window
        .window()
        .dispatch_event(WindowEvent::PointerScrolled {
            position: at,
            delta_x: 0.0,
            delta_y: -360.0,
        });
}

/// Let the scroll animation run out and return the offset it stopped at.
fn finish(reader: &Reader) -> f32 {
    let mut previous = f32::MAX;
    for _ in 0..200 {
        update_timers_and_animations();
        std::thread::sleep(Duration::from_millis(10));
        let at = offset(reader);
        if at > 0.0 && at == previous {
            break;
        }
        previous = at;
    }
    return offset(reader);
}

/// A final reply for request `number`,
///  valid for `picture` with its text revision moved by `stamp_shift`.
fn reply(
    picture: PictureKey,
    stats: ide_app::minimap::Stats,
    number: u64,
    stamp_shift: u64,
) -> Reply {
    let mut moved = picture;
    moved.stamp.revision += stamp_shift;
    return Reply {
        id: number,
        key: moved,
        top: 0,
        chunks: vec![TileScene {
            scene: vello::Scene::new(),
            width: 4,
            height: 4,
        }],
        stats,
        elapsed: Duration::ZERO,
    };
}

/// Whether the picture on screen is drawn from the current highlighting and no newer request is posted.
fn styled(reader: &Reader) -> bool {
    let current = reader.source.borrow();
    let styles = std::sync::Arc::as_ptr(&current.styles).cast::<()>() as usize;
    let drawn = match current.minimap.shown {
        Some(shown) => shown.styles == styles,
        None => false,
    };
    return drawn && current.minimap.posted == current.minimap.shown;
}

/// The mean brightness of the inked pixels of the picture on screen.
fn brightness(reader: &Reader) -> f32 {
    let image = minimap_pixels(&reader.window, &reader.source).expect("picture pixels");
    let mut total = 0.0_f32;
    let mut count = 0.0_f32;
    for pixel in image.as_slice() {
        if pixel.a > 0 {
            let alpha = f32::from(pixel.a);
            total += (0.299 * f32::from(pixel.r)
                + 0.587 * f32::from(pixel.g)
                + 0.114 * f32::from(pixel.b))
                / alpha;
            count += 1.0;
        }
    }
    assert!(count > 100.0, "the picture holds ink");
    return total / count;
}

/// `key` for a text with one more row in its last part of the row measure.
fn another_row(key: PictureKey) -> PictureKey {
    let mut moved = key;
    if let Some(mut rows) = moved.rows {
        rows.3 += 1;
        moved.rows = Some(rows);
    }
    return moved;
}

/// A picture of an older text,
/// of an older request,
/// or of another file is dropped,
/// and a newer one is shown.
#[test]
fn a_stale_picture_is_dropped_and_a_current_one_is_shown() {
    let (_project, opened) = open(&long_text(400));
    settled(&opened);
    picture_shown(&opened);
    let window = &opened.window;
    let (picture, id, stats) = {
        let current = opened.source.borrow();
        (
            current.minimap.shown.expect("a picture key"),
            current.minimap.shown_id,
            current.minimap.last_stats.expect("statistics"),
        )
    };
    let shown_before = opened.source.borrow().minimap.images_set;
    apply(window, &opened.source, reply(picture, stats, id + 5, 1));
    apply(
        window,
        &opened.source,
        reply(picture, stats, id.saturating_sub(1), 0),
    );
    {
        let current = opened.source.borrow();
        assert_eq!(
            current.minimap.images_set, shown_before,
            "a stale picture was set"
        );
        assert_eq!(current.minimap.dropped, 2);
    }
    apply(window, &opened.source, reply(picture, stats, id + 6, 0));
    let current = opened.source.borrow();
    assert_eq!(current.minimap.images_set, shown_before + 1);
    assert_eq!(current.minimap.shown_id, id + 6);
    drop(current);
    window.hide().expect("close minimap window");
}

/// Syntax classes arriving rebuild the picture with the new colors,
/// and the picture follows the scheme:
/// light ink is
///  darker than dark ink.
#[test]
fn highlights_and_the_scheme_rebuild_the_picture_in_the_palette_inks() {
    let project = memory_project("minimap-theme-");
    let mut text = String::new();
    for number in 0..300 {
        text.push_str(&format!(
            "const value{number} = compute({number}, \"text\"); // note\n"
        ));
    }
    fs::write(project.path().join("a.ts"), &text).expect("theme fixture");
    let opened = reader(project.path(), "a.ts");
    {
        let mut waiting = Waiting::eventually();
        while !(!opened.source.borrow().styles.is_empty()) {
            waiting.step("TypeScript highlighting did not arrive");
        }
    }
    switch(&opened.window, ColorScheme::Dark);
    {
        let mut waiting = Waiting::slow();
        while !(styled(&opened)) {
            waiting.step("the picture did not follow the highlights");
        }
    }
    let dark_key = opened.source.borrow().minimap.shown.expect("a picture key");
    let dark = brightness(&opened);
    switch(&opened.window, ColorScheme::Light);
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                let followed = match current.minimap.shown {
                    Some(shown) => shown.colors != dark_key.colors,
                    None => false,
                };
                if followed && current.minimap.shown == current.minimap.posted {
                    break;
                }
            }
            waiting.step("the picture did not follow the scheme");
        }
    }
    let light = brightness(&opened);
    assert!(
        light < dark - 0.15,
        "light-scheme ink ({light}) is darker than dark-scheme ink ({dark})"
    );
    opened.window.hide().expect("close minimap window");
}

/// Opening fifty thousand long lines builds a bounded picture off the window thread:
/// no window-thread callback is slow,
///  and the picture's cost follows the view,
/// not the length of the text.
#[test]
fn fifty_thousand_lines_give_a_bounded_picture_without_blocking_the_window() {
    let mut text = String::new();
    for number in 0..50_000 {
        text.push_str(&format!(
            "fn item_{number}(value: u32) -> u32 {{ value + {number} }} // trailing words words words words words words\n"
        ));
    }
    let started = Instant::now();
    let (_project, opened) = open(&text);
    let window = &opened.window;
    let mut slowest = Duration::ZERO;
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let tick = Instant::now();
                update_timers_and_animations();
                slowest = slowest.max(tick.elapsed());
                let current = opened.source.borrow();
                if current.minimap.images_set > 0 {
                    break;
                }
            }
            waiting.step("the picture of fifty thousand lines did not arrive");
        }
    }
    let first = started.elapsed();
    assert!(
        slowest < Duration::from_millis(1500),
        "a window-thread callback took {slowest:?} (first picture after {first:?})"
    );
    let stats = opened
        .source
        .borrow()
        .minimap
        .last_stats
        .expect("statistics");
    assert!(
        stats.units <= 12 && stats.units * 100 < bands_of(&opened),
        "{} of {} bands were built",
        stats.units,
        bands_of(&opened)
    );
    assert!(
        stats.lines <= 40 * stats.units,
        "{} lines were drawn for {} bands",
        stats.lines,
        stats.units
    );
    // What: Every line of the last picture was shaped by its build or taken from the row cache.
    // Why: The window thread applies every picture the worker has finished in one poll, so a loaded host hands this
    //      test the third picture of the open (a new highlighting reply or a settled extent asks for another
    //      build), whose lines an earlier build, applied or dropped, left in the cache. Whether a cold build
    //      shapes every line is not visible here; `tests/minimap_scene.rs` asserts it on a build with an empty
    //      cache.
    assert_eq!(
        stats.shaped + stats.reused,
        stats.lines,
        "every line of the picture is shaped or reused; pictures applied {}, last build {stats:?}",
        opened.source.borrow().minimap.images_set
    );
    assert_eq!(stats.skipped, 0, "no line was left out");
    assert_eq!(OFF_WORKER_BUILDS.load(Ordering::SeqCst), 0);
    window.hide().expect("close minimap window");
}

/// A change of one line of a file of fifty thousand lines that reaches the window through its watch
/// shapes the one line that changed and takes the rest of the picture's lines from the row cache:
/// the cost of an edit follows the lines it touches,
/// not the length of the file nor the size of the picture.
#[test]
fn an_external_edit_of_fifty_thousand_lines_shapes_only_the_line_it_touches() {
    // The lines are short enough never to wrap:
    // a reload briefly maps every line to one row until it is measured again,
    // and a line that wraps would change its rows and so the lines the picture holds,
    // which brings lines into the picture that no edit touched.
    let mut text = String::new();
    for number in 0..50_000 {
        text.push_str(&format!(
            "fn item_{number}(value: u32) -> u32 {{ value + {number} }} // x\n"
        ));
    }
    let (project, opened) = open(&text);
    let window = &opened.window;
    settled(&opened);
    picture_shown(&opened);
    {
        let mut waiting = Waiting::eventually();
        while !(opened.source.borrow().refresh.is_watched()) {
            waiting.step("the file was not watched");
        }
    }
    // The picture the edit is compared with is the one for the final rows,
    // or its window would differ from the picture after the edit by lines that are new to it.
    picture_still(&opened);
    let revision = opened.source.borrow().document.revision();
    let (cold, shaped_before, window_before) = {
        let current = opened.source.borrow();
        (
            current.minimap.last_stats.expect("statistics"),
            current.minimap.shaped,
            current
                .minimap
                .shown
                .map(|key| return (key.window, key.size)),
        )
    };
    // Positive control: the first picture shaped every line it drew, so a count of one is a count that can move.
    assert!(cold.lines > 50, "the picture draws {} lines", cold.lines);
    assert!(
        shaped_before >= cold.lines as u64,
        "the first pictures shaped {shaped_before} of {} lines",
        cold.lines
    );
    // One word of line 20 changes to another of the same length: the line count and every row stay.
    let edited = text.replacen("item_20(", "itom_20(", 1);
    assert_ne!(edited, text);
    replace_file(&project.path().join("a.txt"), &edited);
    {
        let mut waiting = Waiting::slow();
        while !(opened.source.borrow().document.revision() > revision) {
            waiting.step("the edit did not reach the window");
        }
    }
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                let shown = current.minimap.shown;
                let followed = match shown {
                    Some(key) => key.stamp.revision == current.document.revision(),
                    None => false,
                };
                if followed && current.minimap.posted == shown {
                    break;
                }
            }
            waiting.step("the picture did not follow the edit");
        }
    }
    let (stats, shaped_after, window_after) = {
        let current = opened.source.borrow();
        (
            current.minimap.last_stats.expect("statistics"),
            current.minimap.shaped,
            current
                .minimap
                .shown
                .map(|key| return (key.window, key.size)),
        )
    };
    eprintln!(
        "EDIT before {cold:?} {window_before:?} after {stats:?} {window_after:?} shaped {shaped_before} to {shaped_after}"
    );
    assert!(
        shaped_after > shaped_before,
        "the edit shaped no line of the picture, so the picture shows the old text"
    );
    assert!(
        shaped_after - shaped_before <= 2,
        "the edit shaped {} lines of the picture's {}",
        shaped_after - shaped_before,
        stats.lines
    );
    assert!(
        stats.reused + 2 >= stats.lines,
        "the build after the edit took {} of {} lines from the row cache",
        stats.reused,
        stats.lines
    );
    assert_eq!(stats.skipped, 0);
    window.hide().expect("close minimap window");
}

/// What the window plans for each kind of change of what the picture is valid for.
#[test]
fn each_kind_of_change_has_its_own_pace() {
    let (_project, opened) = open(&long_text(400));
    settled(&opened);
    picture_shown(&opened);
    // A picture keyed before the rows were laid out has no mapping to move, so wait for the one that has.
    {
        let mut waiting = Waiting::slow();
        while opened
            .source
            .borrow()
            .minimap
            .shown
            .and_then(|shown| return shown.rows)
            .is_none()
        {
            waiting.step("no picture was made for a laid out mapping");
        }
    }
    picture_shown(&opened);
    let base = opened.source.borrow().minimap.shown.expect("a picture key");
    let long_ago = Some(Duration::from_secs(5));
    let just_now = Some(Duration::from_millis(40));
    assert_eq!(
        plan(None, None, base, None, false),
        Plan::Post,
        "the first picture is not delayed"
    );
    let mut other_file = base;
    other_file.stamp.file += 1;
    assert_eq!(
        plan(Some(base), Some(base), other_file, just_now, true),
        Plan::Post,
        "another file"
    );
    let mut other_scale = base;
    other_scale.scale ^= 1;
    assert_eq!(
        plan(Some(base), Some(base), other_scale, just_now, true),
        Plan::Post,
        "another scale"
    );
    let mut other_size = base;
    other_size.size.1 ^= 1;
    assert_eq!(
        plan(Some(base), Some(base), other_size, long_ago, false),
        Plan::Wait(DEBOUNCE),
        "a new size waits for the changes to stop"
    );
    let mut other_wrap = base;
    other_wrap.wrap ^= 1;
    assert_eq!(
        plan(Some(base), Some(base), other_wrap, long_ago, false),
        Plan::Wait(DEBOUNCE)
    );
    let mut new_text = base;
    new_text.stamp.revision += 1;
    assert_eq!(
        plan(Some(base), Some(base), new_text, long_ago, false),
        Plan::Wait(DEBOUNCE)
    );
    let mut new_styles = base;
    new_styles.styles ^= 1;
    assert_eq!(
        plan(Some(base), Some(base), new_styles, long_ago, false),
        Plan::Wait(DEBOUNCE)
    );
    let mut other_bands = base;
    other_bands.window.0 += 1;
    other_bands.window.1 += 1;
    assert_eq!(
        plan(Some(base), Some(base), other_bands, long_ago, false),
        Plan::Post,
        "other bands after the slide period are posted at once"
    );
    assert_eq!(
        plan(Some(base), Some(base), other_bands, just_now, false),
        Plan::Wait(SLIDE - Duration::from_millis(40)),
        "other bands inside the period wait for the rest of it, a shorter one than the mapping's"
    );
    assert_eq!(
        plan(Some(base), Some(base), other_bands, just_now, true),
        Plan::Keep,
        "a wait that is already running is kept"
    );
    // The measurement of a long text moves the bands too: it keeps the mapping's slower pace.
    let mut measured = another_row(base);
    measured.window.1 += 1;
    assert_eq!(
        plan(Some(base), Some(base), measured, just_now, false),
        Plan::Wait(THROTTLE - Duration::from_millis(40)),
        "a mapping change that moves the bands is paced as a mapping change"
    );
    assert_eq!(
        plan(Some(base), Some(base), another_row(base), long_ago, false),
        Plan::Post,
        "a mapping change after the throttle's period is posted at once"
    );
    assert_eq!(
        plan(Some(base), Some(base), another_row(base), just_now, false),
        Plan::Wait(THROTTLE - Duration::from_millis(40)),
        "a mapping change inside the period waits for the rest of it, not for quiet"
    );
    assert_eq!(
        plan(Some(base), Some(base), another_row(base), just_now, true),
        Plan::Keep,
        "a wait that is already running is kept"
    );
    // A resize drag changes the size and, with it, the mapping: the wait the size started is kept.
    assert_eq!(
        plan(
            Some(base),
            Some(other_size),
            another_row(other_size),
            long_ago,
            true
        ),
        Plan::Keep,
        "the mapping change that follows a size change does not end the wait"
    );
    // The measurement of a long text after a size change that was already posted: the picture follows at the pace.
    assert_eq!(
        plan(
            Some(base),
            Some(other_size),
            another_row(other_size),
            long_ago,
            false
        ),
        Plan::Post,
        "a mapping change that follows a size change is not held back once no wait is running"
    );
    opened.window.hide().expect("close minimap window");
}
