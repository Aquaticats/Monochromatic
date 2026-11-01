//! Hints and message rows on wrapped lines in the real window:
//!  a block stands above the first visual row of its line and wraps to the text area,
//!  the map puts the next line under the last visual row,
//!  a press on the block or on a continuation row resolves the right line and row,
//!  and a new window width breaks the blocks again.
//! Nothing here waits on the window thread:
//! every wait advances timers and lets the workers answer.

/// The production setter and the stamp of the displayed text.
use super::annotate::{displayed, set_annotations};
/// Fixture records,
/// the injected pair of snapshots,
/// and the pointer helpers of the annotation tests.
use super::annotation_tests::{TEXT_TOP, above, block, click, hint, painted, problem};
/// The caret position the presses leave behind.
use super::caret_tests::position;
/// The reader fixture.
use super::find_tests::Reader;
/// The counters of the window thread's work.
use super::overscan_tile_tests::{counts, reset};
/// The most lines the window thread brings in for one new width.
use super::rewrap::REWRAP_LINES;
/// Window sizing for exact text widths.
use super::sidebar_tests::{TEXT_LEFT, minimap_beside, resize};
/// The worker's build gate.
use super::tile_build::{BUILD_GATE, PASS_GATE};
/// Wrapped fixtures,
/// the open reader,
/// and bounded waits.
use super::wrap_tests::{long_text, open, settled, tabbed_text};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Snapshot records exactly as the Language module builds them.
use ide_app::{
    document::{ReadingAffinity, ReadingPosition},
    language::{
        diagnostics::{DiagnosticsSnapshot, Severity, SourceGroup},
        hints::HintsSnapshot,
    },
    row_map::{CODE_ROW, Place},
    shaped_text::TextShaper,
    virtual_row::{BLOCK_GAP, CONTINUATION_INDENT, ROW_HEIGHT},
};
/// Toolkit access.
use slint::{ComponentHandle, LogicalPosition, platform::update_timers_and_animations};
/// Snapshots are shared values.
use std::sync::Arc;

/// The line every test annotates;
/// the earlier lines are wrapped too,
/// so its place depends on their heights.
const LINE: usize = 3;

/// A message long enough to need rows of its own at any window width.
const MESSAGE: &str = "expected `u32`, found `&str` because the argument named first_item is a string slice here, so convert it first";

/// Install one hint early on `LINE`,
/// one far along it (on a later visual row),
/// and an error with `MESSAGE`.
fn annotate_line(opened: &Reader) {
    let (stamp, start) = {
        let current = opened.source.borrow();
        let start = current.document.text().line_to_char(LINE);
        (displayed(&current), start)
    };
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
        &opened.window,
        &opened.source,
        Some(Arc::new(hints)),
        Some(Arc::new(diagnostics)),
    );
}

/// The window point on visual row `visual` of `line`,
/// `x` logical pixels into the text.
fn on_row(opened: &Reader, line: usize, visual: usize, x: f32) -> LogicalPosition {
    let top = opened.source.borrow().row_map.code_top(line);
    return LogicalPosition::new(
        TEXT_LEFT + x,
        TEXT_TOP + top + visual as f32 * CODE_ROW + CODE_ROW / 2.0 + opened.window.get_scroll_y(),
    );
}

/// The block stands above the first visual row,
/// the next line starts under the last one,
/// the block is as tall as its
///  rows,
/// and every row of text the frame paints is inside the text area.
#[test]
fn a_block_stands_above_the_first_row_and_wraps_to_the_text_area() {
    let (_project, opened) = open(&long_text(8));
    settled(&opened);
    annotate_line(&opened);
    settled(&opened);
    let width = opened.source.borrow().width;
    let found = block(&opened, LINE).expect("the annotated line has a block");
    {
        let current = opened.source.borrow();
        let map = &current.row_map;
        let rows = map.rows_of(LINE);
        assert!(rows >= 3, "the line wraps to several rows: {rows}");
        assert_eq!(map.block_height(LINE), found.height());
        assert_eq!(map.code_top(LINE), map.block_top(LINE) + found.height());
        assert_eq!(
            map.code_bottom(LINE),
            map.code_top(LINE) + rows as f32 * CODE_ROW
        );
        assert_eq!(
            map.block_top(LINE + 1),
            map.code_bottom(LINE),
            "the next line starts under the last visual row"
        );
        assert_eq!(
            found.height(),
            BLOCK_GAP + (found.hint_rows + found.messages.len()) as f32 * ROW_HEIGHT
        );
    }
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
            "`{}` is {shaped} px wide in a text area of {width}",
            row.text
        );
    }
    let current = opened.source.borrow();
    let painted_view = current
        .shaped
        .as_ref()
        .expect("the frame paints the annotations");
    let frame = painted_view
        .annotations
        .as_ref()
        .expect("the frame paints the annotations");
    assert!(!frame.texts.is_empty());
    for text in &frame.texts {
        assert!(
            text.x + text.width <= width + 0.5,
            "a row of text passes the text area: x {} width {} against {width}",
            text.x,
            text.width
        );
    }
    drop(current);
    assert_eq!(painted(&opened).0, 2, "both hints are painted");
    opened.window.hide().expect("close wrap window");
}

/// A press on the block of a wrapped line acts on that line's first row;
/// a press on a continuation row of the same
///  line lands in that row;
/// hover treats the block as over no character,
/// The shaped row of source line `line` among the materialized rows;
///  `what` names it in the failure.
fn row_of_line<'a>(
    view: &'a ide_app::shaped_text::ShapedView,
    line: usize,
    what: &str,
) -> &'a ide_app::shaped_text::ShapedRow {
    for candidate in view.rows.iter() {
        if candidate.row == line {
            return candidate;
        }
    }
    panic!("{what}");
}

/// How many message rows,
///  how many hint rows,
///  and what height the block of `LINE` has.
fn rows_at(reader: &Reader) -> (usize, usize, f32) {
    let found = block(reader, LINE).expect("block");
    return (found.messages.len(), found.hint_rows, found.height());
}

/// and the line below keeps its rows.
#[test]
fn presses_on_a_wrapped_block_and_its_rows_resolve_the_right_line_and_row() {
    let (_project, opened) = open(&long_text(8));
    settled(&opened);
    annotate_line(&opened);
    settled(&opened);
    let window = &opened.window;
    let (first_row, third_row, below_second) = {
        let current = opened.source.borrow();
        let view = current.shaped.as_ref().expect("shaped view");
        let shaped = row_of_line(view, LINE, "line");
        let below = row_of_line(view, LINE + 1, "next");
        let map = &current.row_map;
        let y = map.code_top(LINE) - ROW_HEIGHT;
        assert_eq!(
            map.locate(y),
            Place {
                line: LINE,
                in_block: true
            },
            "a point in the block belongs to the block's line"
        );
        assert_eq!(
            map.locate(map.code_top(LINE) + 2.0 * CODE_ROW + 3.0),
            Place {
                line: LINE,
                in_block: false
            },
            "a point on a continuation row is not in the block"
        );
        (
            shaped.source_rows()[0].clone(),
            shaped.source_rows()[2].clone(),
            below.source_rows()[1].clone(),
        )
    };
    // A press on the block places the caret on the line's first visual row.
    click(window, above(&opened, LINE, 60.0, ROW_HEIGHT));
    let on_block = position(&opened).1;
    assert!(
        first_row.contains(&on_block) || on_block == first_row.end,
        "a press on the block put the caret at {on_block}, outside the first row {first_row:?}"
    );
    // A press on the third visual row of the same line lands in that row.
    click(window, on_row(&opened, LINE, 2, 60.0));
    let on_third = position(&opened).1;
    assert!(
        third_row.contains(&on_third),
        "a press on the third row put the caret at {on_third}, outside {third_row:?}"
    );
    // A press on the second visual row of the next line, whose rows moved down by the block above this line.
    click(window, on_row(&opened, LINE + 1, 1, 60.0));
    let on_below = position(&opened).1;
    assert!(
        below_second.contains(&on_below),
        "a press on the next line's second row put the caret at {on_below}, outside {below_second:?}"
    );
    opened.window.hide().expect("close wrap window");
}

/// A narrower window breaks the block's message into more rows and a wider one returns it to what it was.
#[test]
fn a_new_window_width_breaks_the_block_again() {
    let (_project, opened) = open(&long_text(8));
    settled(&opened);
    annotate_line(&opened);
    settled(&opened);
    let window = &opened.window;
    let wide = opened.source.borrow().width;
    let before = rows_at(&opened);
    reset(&opened.source);
    let packed = opened.source.borrow().annotations.work().packed;
    resize(window, TEXT_LEFT + 300.0 + minimap_beside(300.0), 660.0);
    {
        let mut waiting = Waiting::slow();
        while !(opened.source.borrow().width < wide) {
            waiting.step("the narrow width did not reach the state");
        }
    }
    settled(&opened);
    // The blocks follow the width at once, and so does the tile: the window thread paints it once, within its allowance.
    let [_, tiles, rows, images] = counts(&opened.source);
    assert_eq!(tiles, 1, "the window thread paints the tile for the resize");
    assert!(
        rows <= REWRAP_LINES as u64,
        "the window thread shaped {rows} lines for the resize"
    );
    assert_eq!(images, 1, "the tile for the new width is set once");
    let repacked = opened.source.borrow().annotations.work().packed - packed;
    assert!(
        (1..=3).contains(&repacked),
        "the resize packed {repacked} lines: the one line with hints, once for each width it passed"
    );
    let narrow_width = opened.source.borrow().width;
    let narrow = rows_at(&opened);
    assert!(
        narrow.0 > before.0,
        "a text area of {narrow_width} px breaks the message into more rows than {wide} px: {} against {}",
        narrow.0,
        before.0
    );
    let mut shaper = TextShaper::new();
    for row in &block(&opened, LINE).expect("block").messages {
        let indent = if row.continued {
            CONTINUATION_INDENT
        } else {
            0.0
        };
        let shaped = shaper.row_layout(&row.text, 1.0).full_width();
        assert!(
            indent + shaped <= narrow_width + 0.5,
            "`{}` passes a text area of {narrow_width}",
            row.text
        );
    }
    let current = opened.source.borrow();
    assert_eq!(current.row_map.block_height(LINE), narrow.2);
    drop(current);
    resize(
        window,
        TEXT_LEFT + wide + 2.0 + minimap_beside(wide + 2.0),
        660.0,
    );
    {
        let mut waiting = Waiting::slow();
        while !(opened.source.borrow().width > narrow_width) {
            waiting.step("the wide width did not return");
        }
    }
    settled(&opened);
    assert_eq!(
        rows_at(&opened).0,
        before.0,
        "a wide area restores the rows"
    );
    window.hide().expect("close wrap window");
}

/// A width the window thread's allowance does not cover is left to the worker although the blocks changed with it:
/// the
///  window thread paints no tile for it,
/// and the tile that arrives holds the blocks broken for the new width.
#[test]
fn a_new_width_left_to_the_worker_keeps_the_displayed_tile_until_the_workers_tile_arrives() {
    let (_project, opened) = open(&tabbed_text(60));
    settled(&opened);
    annotate_line(&opened);
    settled(&opened);
    let window = &opened.window;
    let wide = opened.source.borrow().width;
    let before = block(&opened, LINE).expect("block").messages.len();
    reset(&opened.source);
    opened.source.borrow_mut().rewrap_deferred = 0;
    resize(window, TEXT_LEFT + 300.0 + minimap_beside(300.0), 660.0);
    {
        let mut waiting = Waiting::slow();
        while !(opened.source.borrow().width < wide) {
            waiting.step("the narrow width did not reach the state");
        }
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
    let [_, tiles, _, _] = counts(&opened.source);
    assert_eq!(tiles, 0, "the window thread painted a tile for the resize");
    assert!(
        opened.source.borrow().rewrap_deferred >= 1,
        "the change of width was not left to the worker"
    );
    let narrow = block(&opened, LINE).expect("block").messages.len();
    assert!(
        narrow > before,
        "the message is {narrow} rows at the narrow width and {before} at the wide one"
    );
    window.hide().expect("close wrap window");
}

/// Wait until the worker has taken the held build from the gate slot.
fn until_build_started(message: &str) {
    {
        let mut waiting = Waiting::slow();
        while !(BUILD_GATE.lock().expect("build gate").is_none()) {
            waiting.step(message);
        }
    }
}

/// Short lines with a tab,
/// so many that a tile of them is more work than the window thread's allowance covers,
/// and
///  short enough that no line changes its rows when the width changes a little.
fn short_tabbed_text(count: usize) -> String {
    let mut text = String::new();
    for number in 0..count {
        text.push_str(&format!(
            "\tlet value_{number} = compute_total(first_item, second_item, third_item);\n"
        ));
    }
    return text;
}

/// A tile the worker builds for a new width goes stale when the text on screen changes during its build,
/// and the window
///  asks for another tile at once,
/// because the displayed tile stays on screen meanwhile and no later event is certain
///  to ask.
/// No line changes its rows at the new width,
/// so the discard moves nothing and only the wanted tile calls for
///  a render;
/// the change is made in the state directly so that nothing else renders.
#[test]
fn a_tile_discarded_while_the_window_waits_for_one_is_asked_for_again() {
    let (_project, opened) = open(&short_tabbed_text(300));
    settled(&opened);
    let window = &opened.window;
    let wide = opened.source.borrow().width;
    // The measurement of the new width is held back: its output would render the window and so ask for the tile.
    let (hold_the_pass, pass_gate) = std::sync::mpsc::channel();
    *PASS_GATE.lock().expect("pass gate") = Some(pass_gate);
    let (release, gate) = std::sync::mpsc::channel();
    *BUILD_GATE.lock().expect("build gate") = Some(gate);
    resize(window, TEXT_LEFT + 650.0 + minimap_beside(650.0), 660.0);
    until_build_started("the worker did not start the held build");
    assert!(
        opened.source.borrow().width < wide,
        "the window did not get narrower"
    );
    {
        let mut current = opened.source.borrow_mut();
        assert!(
            current.scroll_wanted.is_some(),
            "the window waits for a tile"
        );
        let viewport = current.document.position().viewport;
        current.document.select(ReadingPosition {
            anchor: 1,
            head: 5,
            viewport,
            affinity: ReadingAffinity::Downstream,
        });
    }
    release
        .send(())
        .expect("the build is waiting for its release");
    {
        let mut waiting = Waiting::slow();
        while !(opened.source.borrow().tiles_discarded >= 1) {
            waiting.step("the stale tile was not discarded");
        }
    }
    let start = std::time::Instant::now();
    loop {
        update_timers_and_animations();
        {
            let current = opened.source.borrow();
            if current.shown.width == current.width
                && current.tile_job.is_none()
                && current.scroll_wanted.is_none()
            {
                break;
            }
        }
        assert!(
            start.elapsed() < std::time::Duration::from_secs(15),
            "the discarded tile was not asked for again"
        );
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    drop(hold_the_pass);
    window.hide().expect("close wrap window");
}

/// Hints far from the view keep the packing of the width they were made for after a resize,
/// so the window thread packs
///  only the lines near the view,
/// and they are packed again for the new width once the view comes near them.
#[test]
fn hints_far_from_the_view_are_packed_again_when_the_view_comes_near() {
    const FAR: usize = 250;
    let (_project, opened) = open(&long_text(300));
    settled(&opened);
    // A hint near the end of its line stands on the last visual row, whose start moves with the width.
    let (stamp, near, far) = {
        let current = opened.source.borrow();
        let text = current.document.text();
        let end = text.line(LINE).len_chars() - 4;
        (
            displayed(&current),
            text.line_to_char(LINE) + end,
            text.line_to_char(FAR) + end,
        )
    };
    let snapshot = HintsSnapshot {
        stamp,
        first_line: 0,
        last_line: 300,
        hints: vec![hint(near, ": u32"), hint(far, ": u32")],
    };
    set_annotations(
        &opened.window,
        &opened.source,
        Some(Arc::new(snapshot)),
        None,
    );
    settled(&opened);
    let wide_far = block(&opened, FAR).expect("far block").hints[0].x;
    let wide_near = block(&opened, LINE).expect("near block").hints[0].x;
    let wide = opened.source.borrow().width;
    let packed = opened.source.borrow().annotations.work().packed;
    resize(
        &opened.window,
        TEXT_LEFT + 300.0 + minimap_beside(300.0),
        660.0,
    );
    {
        let mut waiting = Waiting::slow();
        while !(opened.source.borrow().width < wide) {
            waiting.step("the narrow width did not reach the state");
        }
    }
    settled(&opened);
    let narrow = opened.source.borrow().width;
    let repacked = opened.source.borrow().annotations.work().packed - packed;
    assert!(
        (1..=3).contains(&repacked),
        "the resize packed {repacked} lines; only the line near the view may be packed"
    );
    assert_ne!(
        block(&opened, LINE).expect("near block").hints[0].x,
        wide_near,
        "the line near the view was not packed for the new width"
    );
    assert_eq!(
        block(&opened, FAR).expect("far block").hints[0].x,
        wide_far,
        "a line far from the view was packed for the new width"
    );
    // The view comes near the far line: its hints are packed for the width it has now.
    let top = opened.source.borrow().row_map.code_top(FAR);
    opened.window.set_scroll_y(-top);
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                update_timers_and_animations();
                let current = opened.source.borrow();
                if current.tile_job.is_none()
                    && current.scroll_wanted.is_none()
                    && current.shown.first <= FAR
                    && FAR < current.shown.first + current.shown.count
                {
                    break;
                }
            }
            waiting.step("the tile for the far line did not arrive");
        }
    }
    let mut shaper = TextShaper::new();
    let expected = {
        let current = opened.source.borrow();
        shaper
            .wrapped_row(&current.document, FAR, 1.0, narrow)
            .caret_x(far, 1.0)
    };
    {
        let mut waiting = Waiting::slow();
        loop {
            update_timers_and_animations();
            let far_packed = match block(&opened, FAR) {
                Some(found) => found.hints[0].x == expected,
                None => false,
            };
            if far_packed {
                break;
            }
            waiting.step("the far line was not packed for the new width");
        }
    }
    opened.window.hide().expect("close wrap window");
}
