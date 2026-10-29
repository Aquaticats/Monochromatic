//! A line wrapped to more rows than one raster holds,
//! in the real window:
//!  the tile holds a clip of the line around the view,
//! the gutter lists only the rows of the clip,
//!  the clip follows the view through the line with re-centers on the worker,
//!  and scrolling inside an already built tile of a wrapped file does no shaping,
//! painting,
//! or image work.
//! Nothing here waits on the window thread:
//! every wait advances timers and lets the workers answer.

/// Counters and scrolling helpers of the overscan tests.
use super::overscan_tile_tests::{counts, reset, scroll_to, worker_counts};
/// The raster limit and the size rule of a tile that is derived from it.
use super::render::tile_cap;
/// The clip rule,
/// which does not depend on a window.
use super::viewport::band_clip;
/// The reader fixture,
/// bounded waits,
/// and the pixels of the shown image.
use super::{
    tile_reads::shown,
    wrap_tests::{long_text, open, settled},
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Library types that give the expected values.
use ide_app::{row_map::CODE_ROW, shaped_text::stack_fits};
/// Toolkit models and events.
use slint::{ComponentHandle, Model, platform::update_timers_and_animations};

/// Words of the shape `word12 ` until the text holds `length` characters.
fn words(length: usize) -> String {
    let mut text = String::new();
    let mut number = 0;
    while text.len() < length {
        text.push_str(&format!("word{number} "));
        number += 1;
    }
    text.truncate(length);
    return text;
}

/// A file with one line far past the raster limit between two short ones.
fn tall_file() -> String {
    return format!("first line\n{}\nlast line\n", words(100_000));
}

/// Wait until no re-center is queued,
/// shown late,
/// or on the worker.
fn settled_tile(opened: &super::find_tests::Reader) {
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                if current.tile_job.is_none() && current.scroll_wanted.is_none() {
                    break;
                }
            }
            waiting.step("the tile worker did not finish the re-center");
        }
    }
}

/// Whether any pixel of the visible band of the image has ink;
/// a window with no image yet has none.
fn has_ink(
    window: &super::AppWindow,
    source: &std::rc::Rc<std::cell::RefCell<super::State>>,
) -> bool {
    if source.borrow().gpu.tile_size().is_none() && !source.borrow().gpu.has_pending() {
        return false;
    }
    for pixel in shown(window, source) {
        if pixel[3] > 0 {
            return true;
        }
    }
    return false;
}

/// Wait until the first tile is painted or the window names a failure,
/// then fail on the failure.
fn painted_without_error(opened: &super::find_tests::Reader) {
    let window = &opened.window;
    {
        let mut waiting = Waiting::slow();
        while !(has_ink(window, &opened.source) || !window.get_error_message().is_empty()) {
            waiting.step("the window neither painted nor named a failure");
        }
    }
    assert_eq!(
        window.get_error_message().to_string(),
        "",
        "the window shows an error"
    );
}

/// A line past the raster limit is painted in a clip;
/// the clip follows the view through the line from its top to
///  the lines after it,
/// and the gutter lists only the rows of the clip.
#[test]
fn a_line_past_the_raster_limit_is_painted_in_a_clip_that_follows_the_view() {
    let (_project, opened) = open(&tall_file());
    painted_without_error(&opened);
    settled(&opened);
    let window = &opened.window;
    let scale = window.window().scale_factor();
    let (width, line_rows, height) = {
        let current = opened.source.borrow();
        (
            current.width,
            current.row_map.rows_of(1),
            window.get_viewport_height(),
        )
    };
    let cap = tile_cap(width, scale, height);
    assert!(
        line_rows as f32 * CODE_ROW > cap,
        "the fixture line takes {line_rows} rows, which one raster of {cap} px holds"
    );
    // Four places to stand: the top of the text, the middle of the line, near its end, and the text after it.
    let line_top = opened.source.borrow().row_map.code_top(1);
    let line_bottom = opened.source.borrow().row_map.code_bottom(1);
    let end = opened.source.borrow().row_map.height();
    for target in [
        0.0,
        line_top + (line_bottom - line_top) / 2.0,
        line_bottom - 3.0 * height,
        end - height,
        line_top + 300.0 * CODE_ROW,
    ] {
        scroll_to(window, target);
        settled_tile(&opened);
        assert_eq!(
            window.get_error_message().to_string(),
            "",
            "offset {target}: the window shows an error"
        );
        let current = opened.source.borrow();
        let view = current.shaped.as_ref().expect("shaped view");
        assert!(
            stack_fits(u64::from(view.width), u64::from(view.height)),
            "offset {target}: the raster passes the limit"
        );
        let top = view.origin;
        let bottom = view.origin + view.height as f32 / scale;
        let offset = -window.get_scroll_y();
        assert!(
            top <= offset + 0.5 && offset + height <= bottom + 0.5,
            "offset {target}: the raster [{top}, {bottom}] does not hold the view [{offset}, {}]",
            offset + height
        );
        let entries = window.get_row_tops().row_count();
        assert!(
            entries as f32 * CODE_ROW <= bottom - top + 2.0 * CODE_ROW,
            "offset {target}: the gutter lists {entries} rows for a raster {} px tall",
            bottom - top
        );
        assert!(
            entries < line_rows,
            "offset {target}: the gutter lists {entries} of {line_rows} rows"
        );
        drop(current);
        assert!(
            has_ink(window, &opened.source),
            "offset {target}: nothing is painted"
        );
    }
    // The first row of the tall line carries its number once the view has gone back to the top.
    scroll_to(window, 0.0);
    settled_tile(&opened);
    let numbers: Vec<i32> = window.get_row_numbers().iter().collect();
    assert_eq!(numbers.first(), Some(&1));
    assert!(
        numbers.contains(&2),
        "the tall line's first row carries its number"
    );
    window.hide().expect("close wrap window");
}

/// The clip never passes the raster limit,
/// for the sizes that matter:
/// ordinary windows at both scales,
/// a window so tall
///  that three views pass the limit,
/// and a very wide one;
/// its edges are whole code rows inside the lines.
#[test]
fn a_clip_stays_within_the_raster_limit_for_every_window_size() {
    for (width, scale, height) in [
        (782.0_f32, 1.0_f32, 660.0_f32),
        (782.0, 2.0, 660.0),
        (1918.0, 2.0, 1080.0),
        (3800.0, 1.0, 2000.0),
        (782.0, 3.0, 1000.0),
        (1300.0, 1.5, 4000.0),
    ] {
        let cap = tile_cap(width, scale, height * 3.0);
        for view in [0.0_f32, 12_345.0, 400_000.0] {
            let band = ((view - height).max(0.0), view + 2.0 * height);
            let Some(clip) = band_clip(0.0, 1_000_000.0, band, cap) else {
                panic!("a million pixels of lines fit one raster at {width}x{height}");
            };
            let columns = ((width + 24.0).ceil() * scale).ceil() as u64;
            let rows = ((clip.bottom - clip.top) * scale).ceil() as u64;
            assert!(
                stack_fits(columns, rows),
                "width {width} scale {scale} height {height}: a raster of {columns}x{rows} passes the limit"
            );
            assert!(
                clip.bottom - clip.top <= cap,
                "width {width} scale {scale} height {height}: a clip of {} rows passes the limit of {cap}",
                clip.bottom - clip.top
            );
            assert_eq!(clip.top % CODE_ROW, 0.0);
            assert_eq!(clip.bottom % CODE_ROW, 0.0);
            assert!(clip.top >= 0.0 && clip.bottom <= 1_000_000.0 && clip.top < clip.bottom);
        }
    }
    // A band as tall as the cap that starts inside a code row is rounded out by up to two rows, and is cut back to
    // the rows the cap allows.
    let cap = 100.0 * CODE_ROW;
    let Some(clip) = band_clip(0.0, 1_000_000.0, (13.0, 13.0 + cap - 1.0), cap) else {
        panic!("a million pixels of lines fit one raster");
    };
    assert!(
        clip.bottom - clip.top <= cap,
        "a clip of {} rows passes the limit of {cap}",
        clip.bottom - clip.top
    );
    // Lines that fit are not clipped.
    assert!(band_clip(0.0, 1000.0, (0.0, 3000.0), tile_cap(782.0, 1.0, 660.0)).is_none());
}

/// Opening the file shapes each of its lines once on the window thread,
/// and the clip,
/// not the whole line,
///  is what the first raster holds.
#[test]
fn opening_a_hundred_thousand_character_line_shapes_it_once_and_paints_a_clip() {
    let (_project, opened) = open(&tall_file());
    let shaped = opened.source.borrow().rows_shaped;
    assert!(
        shaped <= 4,
        "the window thread shaped {shaped} lines to open a file of three"
    );
    settled(&opened);
    let current = opened.source.borrow();
    assert!(current.shown.clip.is_some(), "the shown tile is clipped");
    let view = current.shaped.as_ref().expect("shaped view");
    assert!(view.height as f32 / view.viewport.scale < current.row_map.code_height(1));
    drop(current);
    opened.window.hide().expect("close wrap window");
}

/// Scrolling inside an already built tile of a wrapped file enters no render,
/// shapes no row,
/// paints no tile,
/// and
///  sets no image,
/// whether the tile holds many short lines or a clip of one tall line.
#[test]
fn scrolling_inside_a_built_tile_of_a_wrapped_file_does_no_source_work() {
    for text in [long_text(400), tall_file()] {
        let (_project, opened) = open(&text);
        settled(&opened);
        let window = &opened.window;
        let height = window.get_viewport_height();
        let start = opened.source.borrow().row_map.code_top(1) + 40.0 * CODE_ROW;
        scroll_to(window, start);
        settled_tile(&opened);
        settled(&opened);
        update_timers_and_animations();
        reset(&opened.source);
        // Half a view height each way stays inside the tile, which holds the view until about 0.75 view heights.
        let steps = (height * 0.5 / 3.0) as usize;
        for step in 1..=steps {
            scroll_to(window, start + step as f32 * 3.0);
        }
        for step in (0..steps).rev() {
            scroll_to(window, start + step as f32 * 3.0);
        }
        let [renders, tiles, rows, images] = counts(&opened.source);
        let facts = crate::native::waiting_tests::Facts {
            window,
            source: &opened.source,
        };
        assert_eq!(
            renders, 0,
            "scrolling inside the tile entered render {facts}"
        );
        assert_eq!(tiles, 0, "scrolling inside the tile painted a tile {facts}");
        assert_eq!(rows, 0, "scrolling inside the tile shaped a row {facts}");
        assert_eq!(images, 0, "scrolling inside the tile set an image {facts}");
        let [worker_tiles, worker_rows, applied, _] = worker_counts(&opened.source);
        assert_eq!(
            [worker_tiles, worker_rows, applied],
            [0, 0, 0],
            "scrolling inside the tile asked the worker for work {facts}"
        );
        window.hide().expect("close wrap window");
    }
}

/// A range on a late row of a wrapped last line is revealed in full even when the background measurement has not
///  reached that line:
/// the map counts such a line as one row,
/// and the offset must not be limited by that count.
#[test]
fn revealing_a_late_row_of_an_unmeasured_last_line_puts_it_in_view() {
    let text = format!("{}{}", "short line\n".repeat(200), "word ".repeat(600));
    let (_project, opened) = open(&text);
    settled(&opened);
    let window = &opened.window;
    let scale = window.window().scale_factor();
    let last = 200;
    // The window opens at a first width and takes a second one a moment later, so two passes are posted;
    // `settled` returns when the worker reports no open pass, which the first pass's finishing output used to cause
    // while the second was still running. Only a map that counts the last line past one row shows that the
    // measurement at the current width has reached it and been applied.
    {
        let mut waiting = Waiting::slow();
        while !(opened.source.borrow().row_map.rows_of(last) > 4) {
            waiting.step("the measurement did not reach the last line");
        }
    }
    // Forget what the measurement found for the last line, as it is before the measurement reaches it.
    {
        let mut current = opened.source.borrow_mut();
        let known = current.row_map.rows_of(last);
        assert!(known > 4, "the last line wraps to {known} rows");
        assert!(super::rows::absorb(&mut current, [(last, 1)]));
        let view = (
            -window.get_scroll_y(),
            window.get_viewport_width(),
            window.get_viewport_height(),
        );
        super::rows::settle(&mut current, scale, view, true);
        assert_eq!(current.row_map.rows_of(last), 1, "the line is unmeasured");
    }
    let start = text.chars().count() - 10;
    super::viewport::reveal(window, &opened.source, start, start + 4);
    let current = opened.source.borrow();
    let view = current.shaped.as_ref().expect("shaped view");
    let rectangle = view.range(start, start + 4)[0];
    let offset = -window.get_scroll_y();
    assert!(
        rectangle.y >= offset
            && rectangle.y + rectangle.height <= offset + window.get_viewport_height(),
        "the revealed range is outside the view: row at {}, view {offset} to {}",
        rectangle.y,
        offset + window.get_viewport_height()
    );
    drop(current);
    window.hide().expect("close wrap window");
}
