//! A new window width in the real window:
//! the window thread breaks the lines near the view again and paints the tile for
//!  the new width before the next frame,
//! within a counted allowance,
//! and what does not fit the allowance goes to the
//!  tile worker.
//! Nothing here waits on the window thread:
//! every wait advances timers and lets the workers answer,
//! and the tests that
//!  look at the first frame hold the worker's measurement still so that only the window thread can have produced it.

/// The production setter and the stamp of the displayed text.
use super::annotate::{displayed, set_annotations};
/// Fixture records of the annotation tests.
use super::annotation_tests::{block, hint, problem};
/// The reader fixture and its key helpers.
use super::find_tests::{Reader, chord, status_for, type_text};
/// Counters of the window thread's work.
use super::overscan_tile_tests::{counts, reset, worker_counts};
/// The paint inputs and the tile description a fresh build needs.
use super::render::{Paint, render, tile_cap, viewport_of};
/// The allowance the window thread may spend on one new width.
use super::rewrap::{REWRAP_LINES, REWRAP_RASTER_BYTES, REWRAP_WORK};
/// The scrolling rule's clock and the check for a map that waits.
use super::rows::{due, hold_clock};
/// Window sizing for exact text widths.
use super::sidebar_tests::{TEXT_LEFT, minimap_beside};
/// The worker's pass gate.
use super::tile_build::{BUILD_GATE, PASS_GATE};
/// The tile's pixels read back from the GPU.
use super::tile_reads::tile_pixels;
/// The wrap width rule.
use super::viewport::wrap_width;
/// Wrapped fixtures,
/// the open reader,
/// and bounded waits.
use super::wrap_tests::{long_text, open, settled, tabbed_text};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Library types that give the expected values.
use ide_app::{
    document::ReadingPosition,
    find_paint::rectangles,
    language::{
        diagnostics::{DiagnosticsSnapshot, Severity, SourceGroup},
        hints::HintsSnapshot,
    },
    row_map::CODE_ROW,
    shaped_text::{ShapedView, TextShaper},
    virtual_row::CONTINUATION_INDENT,
};
/// Toolkit events and models.
use slint::{
    ComponentHandle, LogicalSize, Model,
    platform::{Key, WindowEvent, update_timers_and_animations},
};
/// Snapshots are shared values.
use std::sync::Arc;

/// The line the hint and the message stand on.
const LINE: usize = 3;

/// A message long enough to need rows of its own at any width.
const MESSAGE: &str = "expected `u32`, found `&str` because the argument named first_item is a string slice here, so convert it first";

/// What the window thread did since the last `reset`,
/// read from the debug counters.
#[derive(Debug)]
struct Spent {
    /// Tiles painted on the window thread.
    painted: u64,
    /// Images set on the window.
    images: u64,
    /// Lines shaped on the window thread.
    shaped: u64,
    /// Kept rows the window thread broke again.
    rebroken: u64,
    /// Tiles the worker answered.
    worker_tiles: u64,
    /// Tiles swapped in from the worker.
    applied: u64,
    /// Changes of width taken at once.
    rewrapped: u64,
    /// Changes of width left to the worker.
    deferred: u64,
    /// Lines and work charged to the last immediate change.
    charged: (usize, usize),
}

/// Zero the counters,
/// including those only the width change keeps.
fn start_counting(reader: &Reader) {
    reset(&reader.source);
    let mut current = reader.source.borrow_mut();
    current.rows_rebroken = 0;
    current.rewrapped = 0;
    current.rewrap_deferred = 0;
    current.rewrap_spent = (0, 0);
}

/// Read the counters.
fn spent(reader: &Reader) -> Spent {
    let [_, painted, shaped, images] = counts(&reader.source);
    let [worker_tiles, _, applied, _] = worker_counts(&reader.source);
    let current = reader.source.borrow();
    return Spent {
        painted,
        images,
        shaped,
        rebroken: current.rows_rebroken,
        worker_tiles,
        applied,
        rewrapped: current.rewrapped,
        deferred: current.rewrap_deferred,
        charged: current.rewrap_spent,
    };
}

/// Report the window width that leaves a text area `text_width` wide (the minimap takes its own share) to the toolkit
///  and run the change handlers once:
/// the render the new width causes.
pub(super) fn change_width(reader: &Reader, text_width: f32, height: f32) {
    reader.window.window().dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(TEXT_LEFT + text_width + minimap_beside(text_width), height),
    });
    update_timers_and_animations();
}

/// Hold the worker's measurement still until the returned sender is dropped.
pub(super) fn hold_the_pass() -> std::sync::mpsc::Sender<()> {
    let (release, gate) = std::sync::mpsc::channel();
    *PASS_GATE.lock().expect("pass gate") = Some(gate);
    return release;
}

/// The first character of `line`.
fn start_of(reader: &Reader, line: usize) -> usize {
    return reader.source.borrow().document.text().line_to_char(line);
}

/// Inject one hint early on `LINE`,
/// one far along it,
/// and an error with `MESSAGE`.
fn annotate_line(reader: &Reader) {
    let stamp = displayed(&reader.source.borrow());
    let start = start_of(reader, LINE);
    let hints = HintsSnapshot {
        stamp,
        first_line: 0,
        last_line: 8,
        hints: vec![hint(start + 40, "first:"), hint(start + 200, ": u32")],
    };
    let rustc = SourceGroup {
        source: "rustc".to_string(),
        items: vec![problem(
            start + 4,
            start + 9,
            Severity::Error,
            "E0308",
            MESSAGE,
        )],
    };
    let diagnostics = DiagnosticsSnapshot {
        stamp,
        groups: vec![rustc],
    };
    set_annotations(
        &reader.window,
        &reader.source,
        Some(Arc::new(hints)),
        Some(Arc::new(diagnostics)),
    );
}

/// Rows of `view` whose visible advance passes `width`,
/// other than a row of one character (the one-token fallback).
fn overflowing(view: &ShapedView, width: f32) -> Vec<String> {
    let scale = view.viewport.scale;
    let mut found = Vec::new();
    for row in &view.rows {
        for (index, line) in row.layout.lines().enumerate() {
            let metrics = line.metrics();
            let visible = (metrics.advance - metrics.trailing_whitespace) / scale;
            if visible > width + 0.01 && line.text_range().len() > 1 {
                found.push(format!(
                    "line {} row {index} is {visible} px wide in {width}",
                    row.row
                ));
            }
        }
    }
    return found;
}

/// A fresh build of the displayed tile by a shaper that has never seen the text:
/// rows,
/// selection,
/// and matches.
fn fresh_view(reader: &Reader) -> ShapedView {
    let current = reader.source.borrow();
    let paint = Paint::new(&reader.window, &current);
    let viewport = viewport_of(current.shown, paint.scale);
    let mut shaper = TextShaper::new();
    let mut fresh = shaper.prepare_rows(
        &current.document,
        viewport,
        &current.styles,
        &current.row_map,
    );
    fresh.matches = rectangles(&fresh, &paint.ranges, paint.active);
    return fresh;
}

/// The rectangles of a window model as plain numbers.
fn numbers(model: &slint::ModelRc<super::ui::SourceSelection>) -> Vec<[f32; 4]> {
    let mut found: Vec<[f32; 4]> = Vec::new();
    for rect in model.iter() {
        found.push([rect.x, rect.y, rect.width, rect.height]);
    }
    return found;
}

/// The rectangles of a frame as plain numbers.
fn rects(list: &[ide_app::shaped_text::ReadingRect]) -> Vec<[f32; 4]> {
    let mut found: Vec<[f32; 4]> = Vec::new();
    for rect in list.iter() {
        found.push([rect.x, rect.y, rect.width, rect.height]);
    }
    return found;
}

/// Whether any pixel of the shown image has ink to the right of `limit` logical pixels.
fn ink_past(reader: &super::find_tests::Reader, limit: f32) -> Option<usize> {
    let window = &reader.window;
    let scale = window.window().scale_factor();
    let image = tile_pixels(window, &reader.source).expect("source image pixels");
    let width = image.width() as usize;
    let edge = (limit * scale).ceil() as usize;
    for (index, pixel) in image.as_slice().iter().enumerate() {
        if pixel.a > 0 && index % width > edge {
            return Some(index % width);
        }
    }
    return None;
}

/// The first render after a new width holds a tile wrapped to it,
/// and everything the frame shows agrees with it:
///  the image,
/// the selection,
/// the find rectangles,
/// the caret,
/// the gutter,
/// the hint,
/// and the message.
#[test]
fn the_first_frame_after_a_new_width_is_wrapped_to_it() {
    let (_project, opened) = open(&long_text(80));
    settled(&opened);
    let window = &opened.window;
    annotate_line(&opened);
    settled(&opened);
    // Every line holds `third_item` once, so the bar counts eighty matches; the active one is the reading selection.
    chord(window, Key::Control, "f");
    type_text(window, "third_item");
    status_for(window, "third_item", "1/80");
    settled(&opened);
    // A selection that crosses wrapped rows of two lines.
    let (anchor, head) = (start_of(&opened, 1) + 90, start_of(&opened, 2) + 300);
    {
        let mut current = opened.source.borrow_mut();
        let viewport = current.document.position().viewport;
        current.document.select(ReadingPosition {
            anchor,
            head,
            viewport,
            affinity: ide_app::document::ReadingAffinity::Downstream,
        });
    }
    render(window, &opened.source);
    update_timers_and_animations();
    let wide = opened.source.borrow().width;
    let rows_before = opened.source.borrow().row_map.rows_of(LINE);
    let release = hold_the_pass();
    start_counting(&opened);
    let packed = opened.source.borrow().annotations.work().packed;

    change_width(&opened, 420.0, 660.0);

    let after = spent(&opened);
    let width = wrap_width(window.get_viewport_width()).expect("a width");
    assert!(
        width < wide,
        "the window got narrower: {width} against {wide}"
    );
    {
        let current = opened.source.borrow();
        assert_eq!(current.width, width);
        assert_eq!(
            current.shown.width, width,
            "the displayed tile is not for the new width"
        );
        assert!(current.tile_job.is_none() && current.scroll_wanted.is_none());
        let view = current.shaped.as_ref().expect("shaped view");
        assert_eq!(view.viewport.wrap, Some(width));
        for row in &view.rows {
            assert_eq!(
                row.wrap,
                Some(width),
                "line {} is not broken at it",
                row.row
            );
        }
        let wide_rows = overflowing(view, width);
        assert!(
            wide_rows.is_empty(),
            "rows wider than the width: {wide_rows:?}"
        );
        let repacked = current.annotations.work().packed - packed;
        assert!(
            (1..=3).contains(&repacked),
            "the hinted line was packed {repacked} times"
        );
    }
    // The window thread did it all in this render: one tile, one image, nothing from the worker.
    assert_eq!(after.rewrapped, 1, "{after:?}");
    assert_eq!(after.deferred, 0, "{after:?}");
    assert_eq!(after.painted, 1, "{after:?}");
    assert_eq!(after.images, 1, "{after:?}");
    assert_eq!(after.worker_tiles, 0, "{after:?}");
    assert_eq!(after.applied, 0, "{after:?}");
    // The image is the raster of the new width, and holds no ink past it.
    assert_eq!(
        window.get_image_width(),
        (width + 24.0).ceil(),
        "the shown image is not the raster of the new width"
    );
    assert_eq!(
        ink_past(&opened, width + 24.0),
        None,
        "the shown image has ink past the wrap width"
    );
    // Everything else the frame shows is what a shaper that never saw the text makes of the same tile.
    let fresh = fresh_view(&opened);
    {
        let current = opened.source.borrow();
        let view = current.shaped.as_ref().expect("shaped view");
        assert_eq!(view.rows.len(), fresh.rows.len());
        for (shown, expected) in view.rows.iter().zip(&fresh.rows) {
            assert_eq!(shown.row, expected.row);
            assert_eq!(
                shown.visual_rows(),
                expected.visual_rows(),
                "line {}",
                shown.row
            );
            assert_eq!(shown.top, expected.top, "line {}", shown.row);
            assert_eq!(
                current.row_map.rows_of(shown.row),
                shown.visual_rows(),
                "the map holds the height the tile was painted with"
            );
        }
        assert_eq!(rects(&view.selections), rects(&fresh.selections));
        assert!(!fresh.selections.is_empty());
        assert_eq!(rects(&view.matches), rects(&fresh.matches));
        assert!(!fresh.matches.is_empty());
        let caret = fresh.caret(&current.document);
        assert_eq!(
            (window.get_caret_x(), window.get_caret_y()),
            (caret.x, caret.y)
        );
    }
    assert_eq!(
        numbers(&window.get_source_selections()),
        rects(&fresh.selections),
        "the selection handed to the window"
    );
    assert_eq!(
        numbers(&window.get_source_matches()),
        rects(&fresh.matches),
        "the find rectangles handed to the window"
    );
    // The gutter lists one entry per visual row of the new tile, numbers on first rows only.
    let mut tops = Vec::new();
    let mut labels = Vec::new();
    for row in &fresh.rows {
        for visual in 0..row.visual_rows() {
            tops.push(row.top + row.visual_row_offset(visual, 1.0));
            labels.push(if visual == 0 { row.row as i32 + 1 } else { 0 });
        }
    }
    assert_eq!(window.get_row_tops().iter().collect::<Vec<_>>(), tops);
    assert_eq!(window.get_row_numbers().iter().collect::<Vec<_>>(), labels);
    // The block of the annotated line follows the width: more message rows, and the far hint where the line breaks.
    let found = block(&opened, LINE).expect("block");
    let rows_after = opened.source.borrow().row_map.rows_of(LINE);
    assert!(
        rows_after > rows_before,
        "{rows_after} against {rows_before}"
    );
    {
        let current = opened.source.borrow();
        assert_eq!(current.row_map.block_height(LINE), found.height());
        let mut shaper = TextShaper::new();
        for row in &found.messages {
            let indent = if row.continued {
                CONTINUATION_INDENT
            } else {
                0.0
            };
            let shaped = shaper.row_layout(&row.text, 1.0).full_width();
            assert!(
                indent + shaped <= width + 0.5,
                "`{}` passes the text area of {width}",
                row.text
            );
        }
        let expected = shaper
            .wrapped_row(&current.document, LINE, 1.0, width)
            .caret_x(start_of(&opened, LINE) + 200, 1.0);
        assert_eq!(
            found.hints[1].x, expected,
            "the far hint stands where the line breaks"
        );
    }
    // A frame drawn now keeps the tile: nothing is built again.
    window.window().take_snapshot().expect("the new frame");
    assert_eq!(spent(&opened).painted, 1);
    drop(release);
    window.hide().expect("close wrap window");
}

/// A press right after a new width resolves against the new rows,
/// before the worker has answered anything.
#[test]
fn a_press_right_after_a_new_width_lands_on_the_new_rows() {
    let (_project, opened) = open(&long_text(40));
    settled(&opened);
    let window = &opened.window;
    let release = hold_the_pass();
    change_width(&opened, 420.0, 660.0);
    assert_eq!(
        opened.source.borrow().shown.width,
        opened.source.borrow().width
    );
    let (row, head_on_row) = {
        let current = opened.source.borrow();
        let view = current.shaped.as_ref().expect("shaped view");
        let shaped = &view.rows[1];
        let rows = shaped.visual_rows();
        assert!(
            rows >= 4,
            "the second line wraps to {rows} rows at this width"
        );
        (shaped.source_rows()[3].clone(), shaped.row)
    };
    let top = opened.source.borrow().row_map.code_top(head_on_row);
    super::annotation_tests::click(
        window,
        slint::LogicalPosition::new(
            TEXT_LEFT + 30.0,
            super::annotation_tests::TEXT_TOP
                + top
                + 3.0 * CODE_ROW
                + CODE_ROW / 2.0
                + window.get_scroll_y(),
        ),
    );
    let head = opened.source.borrow().document.position().head;
    assert!(
        row.contains(&head),
        "a press on the fourth row of the line put the caret at {head}, outside {row:?}"
    );
    drop(release);
    window.hide().expect("close wrap window");
}

/// A file of fifty thousand lines costs the window thread what a tile costs,
/// and no more.
#[test]
fn a_new_width_on_fifty_thousand_lines_costs_a_bounded_amount() {
    let mut text = String::new();
    for number in 0..50_000 {
        let tail = if number % 7 == 0 {
            " // trailing words words words words words words words words"
        } else {
            ""
        };
        text.push_str(&format!(
            "fn item_{number}(value: u32) -> u32 {{ value + {number} }}{tail}\n"
        ));
    }
    let release = hold_the_pass();
    let (_project, opened) = open(&text);
    let window = &opened.window;
    start_counting(&opened);
    change_width(&opened, 500.0, 660.0);
    let first = spent(&opened);
    assert_eq!(first.rewrapped, 1, "{first:?}");
    assert_eq!(first.painted, 1, "{first:?}");
    assert_eq!(first.images, 1, "{first:?}");
    assert!(
        first.charged.0 <= REWRAP_LINES && first.charged.1 <= REWRAP_WORK,
        "{first:?}"
    );
    assert!(
        first.shaped + first.rebroken <= first.charged.0 as u64,
        "more lines were brought in than were charged: {first:?}"
    );
    assert!(
        first.shaped + first.rebroken <= REWRAP_LINES as u64,
        "{first:?}"
    );
    // A second width finds the first tile's lines in the cache.
    start_counting(&opened);
    change_width(&opened, 700.0, 660.0);
    let second = spent(&opened);
    assert_eq!(second.rewrapped, 1, "{second:?}");
    assert_eq!(second.painted, 1, "{second:?}");
    assert!(second.charged.1 <= REWRAP_WORK, "{second:?}");
    assert_eq!(second.worker_tiles, 0, "{second:?}");
    drop(release);
    window.hide().expect("close wrap window");
}

/// Lines of six hundred characters,
/// two thousand of them:
/// a tile holds few lines,
/// each paid for by its length.
#[test]
fn a_new_width_on_two_thousand_long_lines_costs_a_bounded_amount() {
    let mut text = String::new();
    for number in 0..2000 {
        let mut line = String::new();
        let mut word = 0;
        while line.len() < 600 {
            line.push_str(&format!("w{number}x{word} "));
            word += 1;
        }
        line.truncate(600);
        text.push_str(&line);
        text.push('\n');
    }
    let release = hold_the_pass();
    let (_project, opened) = open(&text);
    let window = &opened.window;
    for text_width in [500.0, 800.0, 400.0] {
        start_counting(&opened);
        change_width(&opened, text_width, 660.0);
        let step = spent(&opened);
        assert_eq!(step.rewrapped, 1, "{text_width}: {step:?}");
        assert_eq!(step.painted, 1, "{text_width}: {step:?}");
        assert!(
            step.charged.0 <= REWRAP_LINES && step.charged.1 <= REWRAP_WORK,
            "{text_width}: {step:?}"
        );
        assert!(
            step.shaped + step.rebroken <= step.charged.0 as u64,
            "{text_width}: {step:?}"
        );
        let width = wrap_width(window.get_viewport_width()).expect("a width");
        let current = opened.source.borrow();
        let view = current.shaped.as_ref().expect("shaped view");
        assert_eq!(view.viewport.wrap, Some(width));
        assert!(overflowing(view, width).is_empty(), "{text_width}");
    }
    drop(release);
    window.hide().expect("close wrap window");
}

/// Two lines of twelve thousand characters,
/// one with a hint,
/// are more work than the allowance covers:
/// the window
///  thread brings in at most the first before it gives up,
/// paints nothing,
/// and the worker's tile is swapped in
///  later;
/// the heights the whole-file pass ends at are those of a fresh shaping.
#[test]
fn lines_over_the_work_allowance_are_left_to_the_worker() {
    let mut text = String::from("first line\n");
    for number in 0..2 {
        let mut line = String::new();
        let mut word = 0;
        while line.len() < 12_000 {
            line.push_str(&format!("v{number}w{word} "));
            word += 1;
        }
        line.truncate(12_000);
        text.push_str(&line);
        text.push('\n');
    }
    text.push_str("last line\n");
    let (_project, opened) = open(&text);
    settled(&opened);
    let window = &opened.window;
    let scale = window.window().scale_factor();
    // A hint on the first long line: the blocks differ after the change, and the displayed tile must still stay.
    let stamp = displayed(&opened.source.borrow());
    set_annotations(
        window,
        &opened.source,
        Some(Arc::new(HintsSnapshot {
            stamp,
            first_line: 0,
            last_line: 4,
            hints: vec![hint(start_of(&opened, 1) + 50, ": u32")],
        })),
        None,
    );
    settled(&opened);
    start_counting(&opened);
    change_width(&opened, 500.0, 660.0);
    let step = spent(&opened);
    assert_eq!(
        step.deferred, 1,
        "the change of width was not left to the worker: {step:?}"
    );
    assert_eq!(step.rewrapped, 0, "{step:?}");
    assert_eq!(
        step.painted, 0,
        "the window thread painted a tile for a change over the allowance: {step:?}"
    );
    assert!(
        step.shaped + step.rebroken < 2,
        "the window thread brought in both lines: {step:?}"
    );
    {
        let current = opened.source.borrow();
        assert!(
            current.scroll_wanted.is_some(),
            "the change was queued for the worker"
        );
        assert!(
            current.shown.width > current.width,
            "the old tile stays until the worker's arrives"
        );
    }
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                if current.tile_job.is_none()
                    && current.scroll_wanted.is_none()
                    && current.shown.width == current.width
                {
                    break;
                }
            }
            waiting.step("the worker's tile did not arrive");
        }
    }
    settled(&opened);
    let after = spent(&opened);
    assert_eq!(after.applied, 1, "{after:?}");
    assert_eq!(after.painted, 0, "{after:?}");
    let current = opened.source.borrow();
    let lines = current.document.text().len_lines();
    let mut shaper = TextShaper::new();
    let fresh = shaper.measure_lines(&current.document, 0..lines, scale, Some(current.width), &[]);
    let mut stale = Vec::new();
    for (line, rows) in fresh.lines {
        if current.row_map.rows_of(line) != rows {
            stale.push((line, current.row_map.rows_of(line), rows));
        }
    }
    assert!(
        stale.is_empty(),
        "{} lines keep a stale height, first: {:?}",
        stale.len(),
        &stale[..stale.len().min(5)]
    );
    drop(current);
    window.hide().expect("close wrap window");
}

/// Lines with a tab are charged for the corrective passes of their tabs:
/// a view of ordinary short tabbed lines is more
///  work than the allowance covers,
/// goes to the worker,
/// and the lines the old tile held and the new one does not are
///  measured at the new width by the pass that follows.
#[test]
fn tabbed_lines_are_charged_for_their_corrective_passes_and_left_to_the_worker() {
    let mut text = String::new();
    for number in 0..400 {
        text.push_str(&format!(
            "\tlet value_{number} = compute_total(first_item, second_item, third_item);\n"
        ));
    }
    let (_project, opened) = open(&text);
    settled(&opened);
    let window = &opened.window;
    let scale = window.window().scale_factor();
    start_counting(&opened);
    change_width(&opened, 500.0, 660.0);
    let step = spent(&opened);
    assert_eq!(
        step.deferred, 1,
        "the change of width was not left to the worker: {step:?}"
    );
    assert_eq!(
        step.painted, 0,
        "the window thread painted a tile for a change over the allowance: {step:?}"
    );
    assert!(
        step.shaped + step.rebroken <= 9,
        "the window thread brought in more lines than the allowance pays for: {step:?}"
    );
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                if current.tile_job.is_none()
                    && current.scroll_wanted.is_none()
                    && current.shown.width == current.width
                {
                    break;
                }
            }
            waiting.step("the worker's tile did not arrive");
        }
    }
    settled(&opened);
    let current = opened.source.borrow();
    let lines = current.document.text().len_lines();
    let mut shaper = TextShaper::new();
    let fresh = shaper.measure_lines(&current.document, 0..lines, scale, Some(current.width), &[]);
    let mut stale = Vec::new();
    for (line, rows) in fresh.lines {
        if current.row_map.rows_of(line) != rows {
            stale.push((line, current.row_map.rows_of(line), rows));
        }
    }
    assert!(
        stale.is_empty(),
        "{} lines keep a stale height, first: {:?}",
        stale.len(),
        &stale[..stale.len().min(5)]
    );
    drop(current);
    window.hide().expect("close wrap window");
}

/// One line of two hundred thousand characters is not shaped or broken on the window thread:
/// the change goes to the
///  worker,
/// and its tile is swapped in when it arrives.
#[test]
fn a_line_of_two_hundred_thousand_characters_is_left_to_the_worker() {
    let mut line = String::new();
    let mut word = 0;
    while line.len() < 200_000 {
        line.push_str(&format!("word{word} "));
        word += 1;
    }
    line.truncate(200_000);
    let text = format!("first line\n{line}\nlast line\n");
    let (_project, opened) = open(&text);
    settled(&opened);
    let window = &opened.window;
    start_counting(&opened);
    change_width(&opened, 500.0, 660.0);
    let step = spent(&opened);
    assert_eq!(
        step.deferred, 1,
        "the change of width was not left to the worker: {step:?}"
    );
    assert_eq!(step.rewrapped, 0, "{step:?}");
    assert_eq!(
        step.painted, 0,
        "the window thread painted a tile for a change over the allowance: {step:?}"
    );
    assert_eq!(step.shaped, 0, "the window thread shaped a line: {step:?}");
    assert_eq!(
        step.rebroken, 0,
        "the window thread broke a line again: {step:?}"
    );
    assert_eq!(step.images, 0, "{step:?}");
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                if current.tile_job.is_none()
                    && current.scroll_wanted.is_none()
                    && current.shown.width == current.width
                {
                    break;
                }
            }
            waiting.step("the worker's tile did not arrive");
        }
    }
    let after = spent(&opened);
    assert_eq!(after.applied, 1, "{after:?}");
    assert_eq!(after.painted, 0, "{after:?}");
    assert_eq!(after.shaped, 0, "{after:?}");
    window.hide().expect("close wrap window");
}

/// A view so tall that its tile holds more lines than the allowance covers is left to the worker without any
///  shaping on the window thread,
/// and a view taller than one raster is left to it by the allowance of lines or by
///  the bound on the raster.
#[test]
fn a_view_with_more_lines_than_the_allowance_is_left_to_the_worker() {
    // A tile of the limit is far past the bound, or holds more lines than the allowance, at every size
    // where the texture limit of the device does not cut it below the allowance:
    // a 300 px wide tile on a device limited to 8192 px holds 340 lines, which the allowance covers.
    for (width, scale) in [(1100.0_f32, 1.0_f32), (1100.0, 2.0), (3800.0, 1.0)] {
        let cap = tile_cap(width, scale, 1.0e9);
        let columns = ((width + 24.0).ceil() * scale).ceil() as u64;
        let bytes = columns * (cap * scale).floor() as u64 * 4;
        assert!(
            bytes > REWRAP_RASTER_BYTES || cap / CODE_ROW > REWRAP_LINES as f32,
            "a raster at the cap of {cap} px is {bytes} bytes at {width} wide and scale {scale}"
        );
    }
    let (_project, opened) = open(&"a short line\n".repeat(5000));
    settled(&opened);
    let window = &opened.window;
    // A window 3,300 pixels tall shows about 137 rows, so its tile holds about 412 lines. The headless renderer
    // cannot draw a window that large, so the size is reported without a frame.
    change_width(&opened, 700.0, 3300.0);
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                if current.tile_job.is_none()
                    && current.scroll_wanted.is_none()
                    && current.shown.width == current.width
                {
                    break;
                }
            }
            waiting.step("the tall window's tile did not arrive");
        }
    }
    // Away from the top of the text the tile has a view of overscan on both sides.
    window.set_scroll_y(-24_000.0);
    update_timers_and_animations();
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                if current.tile_job.is_none()
                    && current.scroll_wanted.is_none()
                    && current.shown.first > 500
                {
                    break;
                }
            }
            waiting.step("the tile for the scrolled view did not arrive");
        }
    }
    settled(&opened);
    start_counting(&opened);
    change_width(&opened, 500.0, 3300.0);
    let step = spent(&opened);
    assert!(
        opened.source.borrow().count > REWRAP_LINES,
        "the tile holds {} lines in a view {} px tall",
        opened.source.borrow().count,
        window.get_viewport_height()
    );
    assert_eq!(
        step.deferred, 1,
        "the change of width was not left to the worker: {step:?}"
    );
    assert_eq!(
        step.painted, 0,
        "the window thread painted a tile for a view over the allowance: {step:?}"
    );
    assert_eq!(
        step.shaped + step.rebroken,
        0,
        "the window thread brought in lines for a view over the allowance: {step:?}"
    );
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                if current.tile_job.is_none()
                    && current.scroll_wanted.is_none()
                    && current.shown.width == current.width
                {
                    break;
                }
            }
            waiting.step("the worker's tile did not arrive");
        }
    }
    window.hide().expect("close wrap window");
}

/// The bytes of a raster of three views of 1124 columns at display scale `scale`.
fn common_raster_bytes(scale: f32) -> u64 {
    return (1124.0 * scale).ceil() as u64 * (1980.0 * scale).ceil() as u64 * 4;
}

/// A tile whose raster passes the bound is left to the worker although its lines are few and cheap,
/// and the bound
///  admits the tile of a common window.
#[test]
fn a_tile_with_a_huge_raster_is_left_to_the_worker() {
    // 1100 x 660 at scale 1: three views of 1124 columns.
    assert!(common_raster_bytes(1.0) <= REWRAP_RASTER_BYTES);
    const { assert!(REWRAP_RASTER_BYTES < ide_app::shaped_text::MAX_RASTER_BYTES) };
    let (_project, opened) = open(&"a short line\n".repeat(6000));
    settled(&opened);
    let window = &opened.window;
    // At scale 3 the tile of 1,000 columns by 6,000 rows is about 70 MB, past the bound and under the worker's limit.
    let original = window.window().scale_factor();
    window
        .window()
        .dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: original * 3.0,
        });
    update_timers_and_animations();
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                if current.tile_job.is_none() && current.scroll_wanted.is_none() {
                    break;
                }
            }
            waiting.step("the tile for the new scale did not arrive");
        }
    }
    settled(&opened);
    start_counting(&opened);
    change_width(&opened, 700.0, 660.0);
    let step = spent(&opened);
    assert!(opened.source.borrow().count <= REWRAP_LINES);
    assert_eq!(
        step.deferred, 1,
        "the change of width was not left to the worker: {step:?}"
    );
    assert_eq!(
        step.painted, 0,
        "the window thread painted a tile whose raster passes the bound: {step:?}"
    );
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                if current.tile_job.is_none()
                    && current.scroll_wanted.is_none()
                    && current.shown.width == current.width
                {
                    break;
                }
            }
            waiting.step("the worker's tile did not arrive");
        }
    }
    window.hide().expect("close wrap window");
}

/// After a width taken at once,
/// the worker's whole-file re-break ends at the heights a fresh shaping gives.
#[test]
fn the_workers_re_break_converges_after_a_width_taken_at_once() {
    let (_project, opened) = open(&long_text(300));
    settled(&opened);
    let window = &opened.window;
    let scale = window.window().scale_factor();
    start_counting(&opened);
    change_width(&opened, 420.0, 660.0);
    assert_eq!(opened.source.borrow().rewrapped, 1);
    settled(&opened);
    let current = opened.source.borrow();
    let lines = current.document.text().len_lines();
    let mut shaper = TextShaper::new();
    let fresh = shaper.measure_lines(&current.document, 0..lines, scale, Some(current.width), &[]);
    let mut stale = Vec::new();
    for (line, rows) in fresh.lines {
        if current.row_map.rows_of(line) != rows {
            stale.push((line, current.row_map.rows_of(line), rows));
        }
    }
    assert!(
        stale.is_empty(),
        "{} lines keep a stale height, first: {:?}",
        stale.len(),
        &stale[..stale.len().min(5)]
    );
    drop(current);
    window.hide().expect("close wrap window");
}

/// The worker's measurement of a new width is applied before the tile for it is swapped in:
/// the lines the old tile
///  held and the new tile does not are skipped by that application,
/// so the measurement must be asked for again once
///  the tile is shown.
/// The view jumps away while the first tile build is held,
/// so the build for the view the
///  window ends at starts only after the measurement output waits at the window.
#[test]
fn a_measurement_applied_before_the_tile_of_a_new_width_leaves_no_stale_heights() {
    let release = hold_the_pass();
    for _ in 0..400 {
        let _ = release.send(());
    }
    let (_project, opened) = open(&tabbed_text(1300));
    settled(&opened);
    let window = &opened.window;
    let scale = window.window().scale_factor();
    let height = window.get_viewport_height();
    let (build_release, build_gate) = std::sync::mpsc::channel();
    *BUILD_GATE.lock().expect("build gate") = Some(build_gate);
    window.set_scroll_y(-(40.0 * height));
    update_timers_and_animations();
    {
        let mut waiting = Waiting::slow();
        while !(BUILD_GATE.lock().expect("build gate").is_none()) {
            waiting.step("the worker did not start the held build");
        }
    }
    start_counting(&opened);
    change_width(&opened, 500.0, 660.0);
    assert_eq!(
        spent(&opened).deferred,
        1,
        "the change of width was not left to the worker"
    );
    let lines = opened.source.borrow().document.text().len_lines();
    build_release
        .send(())
        .expect("the held build is waiting for its release");
    // Nothing advances timers here, so the window takes neither the tile nor the measurement until both wait.
    let start = std::time::Instant::now();
    loop {
        let waiting = match opened.source.borrow().tile_worker.as_ref() {
            Some(worker) => worker.pending_pass_lines(),
            None => 0,
        };
        if waiting >= lines {
            break;
        }
        assert!(
            start.elapsed() < std::time::Duration::from_secs(90),
            "the worker measured only {waiting} of {lines} lines"
        );
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                if current.tile_job.is_none()
                    && current.scroll_wanted.is_none()
                    && current.shown.width == current.width
                {
                    break;
                }
            }
            waiting.step("the tile for the new width did not arrive");
        }
    }
    settled(&opened);
    // What: The map follows the table of heights only once the reader has stopped scrolling for the quiet time, and
    //       the final tile and measurement of this test come within it of the scroll that started it, so the clock
    //       of that rule is moved past it and the window is asked until the map has caught up.
    // Why: The heights this test compares are those of the map; a fast host reaches the comparison before the quiet
    //      time has passed, which is the rule working and not a stale height.
    hold_clock(Some(
        std::time::Instant::now() + std::time::Duration::from_secs(10),
    ));
    {
        let mut waiting = Waiting::slow();
        while !(!due(&mut opened.source.borrow_mut(), scale)) {
            waiting.step("the map did not follow the table of heights");
        }
    }
    hold_clock(None);
    let current = opened.source.borrow();
    let mut shaper = TextShaper::new();
    let fresh = shaper.measure_lines(&current.document, 0..lines, scale, Some(current.width), &[]);
    let mut stale = Vec::new();
    for (line, rows) in fresh.lines {
        if current.row_map.rows_of(line) != rows {
            stale.push((line, current.row_map.rows_of(line), rows));
        }
    }
    assert!(
        stale.is_empty(),
        "{} lines keep a stale height, first: {:?}",
        stale.len(),
        &stale[..stale.len().min(5)]
    );
    drop(current);
    drop(release);
    window.hide().expect("close wrap window");
}

/// A frame drawn right after the toolkit reports a new size,
/// with no timer or handler run between them,
/// already holds
///  the tile for the new width:
/// the viewport callback runs inside the draw,
/// before the frame is composed,
/// as it does
///  under a real windowing backend.
#[test]
fn a_frame_drawn_right_after_a_resize_event_is_wrapped_to_the_new_width() {
    let (_project, opened) = open(&long_text(80));
    settled(&opened);
    let window = &opened.window;
    let release = hold_the_pass();
    start_counting(&opened);
    window.window().dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(TEXT_LEFT + 420.0 + minimap_beside(420.0), 660.0),
    });
    // The first thing that happens after the event is the draw: no `update_timers_and_animations` before it.
    let frame = window.window().take_snapshot().expect("a drawn frame");
    // The headless adapter keeps its own buffer size; the layout has the size of the event, and its right edge is
    //  the column the code area ends at.
    let drawn = frame.width() as usize;
    let edge = (TEXT_LEFT + 420.0) as usize;
    let pixels = frame.as_slice();
    let mut common = std::collections::HashMap::new();
    for row in 40..frame.height() as usize {
        *common
            .entry(pixels[row * drawn + edge - 1])
            .or_insert(0_usize) += 1;
    }
    let mut most_common = None;
    let mut most_rows = 0;
    for (pixel, count) in common {
        if most_common.is_none() || count > most_rows {
            most_common = Some(pixel);
            most_rows = count;
        }
    }
    let background = most_common.expect("a code area background");
    let mut at_edge = 0;
    for row in 40..frame.height() as usize {
        let last_column = pixels[row * drawn + edge - 1] != background;
        let before_last = pixels[row * drawn + edge - 2] != background;
        if last_column || before_last {
            at_edge += 1;
        }
    }
    let step = spent(&opened);
    assert_eq!(
        at_edge, 0,
        "{at_edge} rows of the drawn frame run to its right edge: the text is wrapped for the old width"
    );
    {
        let current = opened.source.borrow();
        assert_eq!(
            current.shown.width, current.width,
            "the frame was drawn with a tile for another width"
        );
    }
    assert_eq!(step.rewrapped, 1, "{step:?}");
    drop(release);
    window.hide().expect("close wrap window");
}

/// A view taller than the tallest stack of textures a tile may be (`tile_cap` of a view that asks for everything) is
/// left to the worker too:
/// nothing of it is
///  shaped or painted on the window thread.
/// No raster can hold such a view whole,
/// so the test ends with the step and
///  does not wait for a tile.
/// The headless renderer cannot draw a window that large,
/// so the size is reported without a
///  frame.
/// The lines are long,
/// so each of them is broken again at the new width,
/// and the work is read from the shaper's own counters:
/// the step's counters only record a measurement that ended within its allowance,
/// and one that ran out of it has broken lines by then and records none.
#[test]
fn a_view_taller_than_the_clip_cap_is_left_to_the_worker() {
    let (_project, opened) = open(&long_text(5000));
    settled(&opened);
    let window = &opened.window;
    let text_width = 3600.0;
    let cap = tile_cap(wrap_width(text_width).expect("a width"), 1.0, 1.0e9);
    let height = cap + 400.0;
    start_counting(&opened);
    let brought_in = |reader: &Reader| {
        let current = reader.source.borrow();
        return current.shaper.shaped_rows() + current.shaper.rebroken_rows();
    };
    let before = brought_in(&opened);
    change_width(&opened, text_width, height);
    let step = spent(&opened);
    assert!(
        window.get_viewport_height() > cap,
        "the view is {} px tall, not past the cap of {cap} px",
        window.get_viewport_height()
    );
    // The width and the height are reported one after the other, so the worker may be asked twice.
    assert!(
        step.deferred >= 1 && step.rewrapped == 0,
        "the change of width was not left to the worker: {step:?}"
    );
    assert_eq!(
        step.painted, 0,
        "the window thread painted a tile for a view past the cap: {step:?}"
    );
    assert_eq!(
        brought_in(&opened) - before,
        0,
        "the window thread brought in lines for a view past the cap: {step:?}"
    );
    window.hide().expect("close wrap window");
}
