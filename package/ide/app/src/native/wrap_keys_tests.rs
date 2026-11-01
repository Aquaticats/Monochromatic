//! Keys and pointer on wrapped lines in the real window:
//!  Home and End by visual row,
//! Up and Down with a remembered x,
//! paging by visual rows,
//!  word motion across a wrap offset,
//! the view following the caret's visual row,
//!  drags across rows,
//! and double and triple clicks on rows of one line.
//! Each behavior is checked at several window widths (so at several wrap widths) and at display scales 1 and 2.

/// Caret positions and the in-view check shared with the plain caret tests.
use super::caret_tests::{caret_in_view, position};
/// The complete reader fixture and key helpers.
use super::find_tests::{Reader, chord, key};
/// Window sizes and the text origin of the pointer tests.
use super::sidebar_tests::{HEADER, TEXT_LEFT, resize};
/// Fixture builders and the waits that let the worker measure.
use super::wrap_tests::{long_line, long_text, open, settled};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{Facts, Waiting};
/// Geometry-free word motion,
/// the expected value of Ctrl+Left and Ctrl+Right.
use ide_app::caret_motion::{Motion, moved};
/// The side of a wrap offset a caret stands on.
use ide_app::document::ReadingAffinity;
/// Which row a caret place belongs to.
use ide_app::line_boundary::row_containing;
/// Toolkit events.
use slint::{
    ComponentHandle, LogicalPosition, Model,
    platform::{Key, PointerEventButton, WindowEvent, update_timers_and_animations},
};
/// Pauses longer than the multi-click interval,
/// and half-open ranges.
use std::{ops::Range, thread::sleep, time::Duration};

/// The logical window widths,
/// sidebar included,
/// whose wrap widths the tests walk through at display scale `factor`.
/// The headless window's buffer holds 1100 by 660 physical pixels,
/// so a window at scale 2 is at most 550 logical
/// pixels wide,
/// which still leaves the source column its minimum.
fn widths(factor: f32) -> [f32; 3] {
    if factor > 1.5 {
        return [500.0, 525.0, 550.0];
    }
    return [800.0, 950.0, 1100.0];
}

/// Show the window at display scale `factor` with the logical size `width` by 660 and wait for the measurement.
fn arrange(reader: &Reader, factor: f32, width: f32) {
    let window = &reader.window;
    window
        .window()
        .dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: factor,
        });
    resize(window, width, 660.0 / factor);
    {
        let mut waiting = Waiting::slow();
        while !(reader.source.borrow().width > 0.0) {
            waiting.step("the wrap width did not follow the arrangement");
        }
    }
    settled(reader);
}

/// The ranges of the visual rows of `line`,
/// from the materialized rows.
fn rows_of(reader: &Reader, line: usize) -> Vec<Range<usize>> {
    let current = reader.source.borrow();
    let view = current.shaped.as_ref().expect("shaped view");
    for row in view.rows.iter() {
        if row.row == line {
            return row.source_rows();
        }
    }
    panic!("the line is materialized");
}

/// The side of the wrap offset the caret stands on.
fn affinity(reader: &Reader) -> ReadingAffinity {
    return reader.source.borrow().document.position().affinity;
}

/// The caret's visual row counted from the top of the text,
/// read from where the window draws it.
fn drawn_row(reader: &Reader) -> usize {
    return ((reader.window.get_caret_y() - 2.0) / 24.0).round() as usize;
}

/// Home and End on a line of several rows,
/// as the window shows them,
/// at several widths and scales.
#[test]
fn home_and_end_follow_the_visual_rows() {
    for factor in [1.0, 2.0] {
        for width in widths(factor) {
            let label = format!("scale {factor} width {width}");
            let (_project, reader) = open(&long_text(8));
            arrange(&reader, factor, width);
            let window = &reader.window;
            let rows = rows_of(&reader, 0);
            assert!(
                rows.len() >= 3,
                "{label}: the first line wraps to {} rows",
                rows.len()
            );
            let text = reader.source.borrow().document.text().to_string();
            let last = rows.len() - 1;
            // End leaves the trailing blank of the first row out, then goes to its end, then to the line end.
            key(window, Key::End);
            let first_stop = position(&reader).1;
            let trimmed = text.chars().take(rows[0].end).collect::<String>();
            assert_eq!(
                first_stop,
                trimmed.trim_end_matches([' ', '\t']).chars().count(),
                "{label}: End goes before the blanks that end the first row"
            );
            assert_eq!(drawn_row(&reader), 0, "{label}: and is drawn on it");
            if first_stop != rows[0].end {
                key(window, Key::End);
            }
            assert_eq!(
                position(&reader),
                (rows[0].end, rows[0].end),
                "{label}: the end of the first row"
            );
            assert_eq!(affinity(&reader), ReadingAffinity::Upstream, "{label}");
            assert_eq!(
                drawn_row(&reader),
                0,
                "{label}: the end of an upper row is drawn on it"
            );
            // The line ends in a blank, so End from the end of a row goes before it first, and then to the true end.
            key(window, Key::End);
            let line_end = rows[last].end;
            let before_blanks = text
                .chars()
                .take(line_end)
                .collect::<String>()
                .trim_end_matches([' ', '\t'])
                .chars()
                .count();
            assert!(
                before_blanks < line_end,
                "{label}: the fixture line ends in a blank"
            );
            assert_eq!(
                position(&reader).1,
                before_blanks,
                "{label}: End from the end of a row goes to the line end"
            );
            assert_eq!(
                drawn_row(&reader),
                last,
                "{label}: rows {rows:?} affinity {:?}",
                affinity(&reader)
            );
            key(window, Key::End);
            assert_eq!(
                position(&reader).1,
                line_end,
                "{label}: and then after the blanks"
            );
            assert_eq!(affinity(&reader), ReadingAffinity::Downstream, "{label}");
            // Home goes to the start of the last row, then to the line's first non-blank character.
            key(window, Key::Home);
            assert_eq!(
                position(&reader).1,
                rows[last].start,
                "{label}: Home goes to the start of the row"
            );
            assert_eq!(drawn_row(&reader), last, "{label}");
            key(window, Key::Home);
            assert_eq!(
                position(&reader),
                (0, 0),
                "{label}: Home at a row start goes to the line start"
            );
            assert_eq!(drawn_row(&reader), 0, "{label}");
            // Shift extends from the anchor and the selection covers the row.
            chord(window, Key::Shift, Key::End);
            if position(&reader).1 != rows[0].end {
                chord(window, Key::Shift, Key::End);
            }
            assert_eq!(
                position(&reader),
                (0, rows[0].end),
                "{label}: Shift+End extends"
            );
            assert_eq!(window.get_source_selections().row_count(), 1, "{label}");
            chord(window, Key::Shift, Key::End);
            assert_eq!(
                position(&reader),
                (0, before_blanks),
                "{label}: Shift+End again extends to the line end, before its trailing blank"
            );
            assert_eq!(
                window.get_source_selections().row_count(),
                rows.len(),
                "{label}: one rectangle per row"
            );
            chord(window, Key::Shift, Key::Home);
            assert_eq!(
                position(&reader),
                (0, rows[last].start),
                "{label}: Shift+Home shrinks to the last row's start"
            );
            window.hide().expect("close wrap window");
        }
    }
}

/// Home on an indented line goes to the first non-blank column from the start of a continuation row,
///  and End skips the trailing blanks of a row that breaks after a blank.
#[test]
fn home_reaches_the_indent_of_a_wrapped_line() {
    let line = format!("        {}", long_line(0));
    let (_project, reader) = open(&format!("{line}\nshort\n"));
    arrange(&reader, 1.0, 1100.0);
    let window = &reader.window;
    let rows = rows_of(&reader, 0);
    assert!(rows.len() >= 3);
    let last = rows.len() - 1;
    // End as often as it takes to reach the end of the line.
    for _ in 0..6 {
        if position(&reader).1 != rows[last].end {
            key(window, Key::End);
        }
    }
    assert_eq!(position(&reader).1, rows[last].end);
    assert_eq!(drawn_row(&reader), last);
    key(window, Key::Home);
    assert_eq!(position(&reader).1, rows[last].start);
    key(window, Key::Home);
    assert_eq!(
        position(&reader),
        (8, 8),
        "from a continuation row start to the first non-blank column"
    );
    assert_eq!(drawn_row(&reader), 0);
    key(window, Key::Home);
    assert_eq!(
        position(&reader),
        (0, 0),
        "from the first non-blank column to the line start"
    );
    key(window, Key::Home);
    assert_eq!(position(&reader), (8, 8), "and back");
    window.hide().expect("close wrap window");
}

/// Up and Down move one visual row at a time,
/// keep the pixel x through short rows,
/// and Left clears it.
#[test]
fn up_and_down_move_by_visual_rows_and_keep_their_x() {
    for factor in [1.0, 2.0] {
        for width in widths(factor) {
            let label = format!("scale {factor} width {width}");
            let text = format!("{}\nshort\n{}\n", long_line(0), long_line(2));
            let (_project, reader) = open(&text);
            arrange(&reader, factor, width);
            let window = &reader.window;
            let first = rows_of(&reader, 0);
            let third = rows_of(&reader, 2);
            // Half a row in, which is past the end of the short line: the aim.
            for _ in 0..first[0].len() / 2 {
                key(window, Key::RightArrow);
            }
            let aim = window.get_caret_x();
            let mut previous = 0;
            for expected in 1..first.len() {
                key(window, Key::DownArrow);
                assert_eq!(drawn_row(&reader), expected, "{label}: Down {expected}");
                // The last row may be shorter than the aim and then ends at its text.
                assert!(
                    (window.get_caret_x() - aim).abs() <= 9.0 || expected == first.len() - 1,
                    "{label}: Down {expected} moved x from {aim} to {}",
                    window.get_caret_x()
                );
                assert!(
                    position(&reader).1 > previous,
                    "{label}: the caret moves forward"
                );
                previous = position(&reader).1;
            }
            key(window, Key::DownArrow);
            let short_start = reader.source.borrow().document.text().line_to_char(1);
            assert_eq!(
                position(&reader).1,
                short_start + 5,
                "{label}: the short line takes its whole text"
            );
            assert_eq!(drawn_row(&reader), first.len(), "{label}");
            key(window, Key::DownArrow);
            let third_start = reader.source.borrow().document.text().line_to_char(2);
            assert_eq!(
                drawn_row(&reader),
                first.len() + 1,
                "{label}: the first row of the third line"
            );
            assert!(
                third[0].contains(&position(&reader).1) && position(&reader).1 > third_start + 5,
                "{label}: the aim returns after the short line, head {}",
                position(&reader).1
            );
            assert!(
                (window.get_caret_x() - aim).abs() <= 9.0,
                "{label}: x {} against the aim {aim}",
                window.get_caret_x()
            );
            // Up retraces the rows.
            for _ in 0..2 {
                key(window, Key::UpArrow);
            }
            assert_eq!(
                drawn_row(&reader),
                first.len() - 1,
                "{label}: Up from the third line's first row twice"
            );
            // Left clears the aim: Down then takes the x of the new caret.
            key(window, Key::LeftArrow);
            let moved_x = window.get_caret_x();
            key(window, Key::UpArrow);
            key(window, Key::DownArrow);
            assert!(
                (window.get_caret_x() - moved_x).abs() <= 9.0,
                "{label}: after Left the aim is the caret's own x"
            );
            // Shift+Down extends by a row.
            let anchor = position(&reader).1;
            chord(window, Key::Shift, Key::DownArrow);
            let (from, to) = position(&reader);
            assert_eq!(from, anchor, "{label}: the anchor stays");
            assert!(to > from, "{label}");
            window.hide().expect("close wrap window");
        }
    }
}

/// An Up from the lower row of a wrap offset stays on the upper row when its end is nearer than the aim.
#[test]
fn a_move_to_a_shorter_upper_row_keeps_the_caret_on_it() {
    // A word too long for the rest of the first row breaks to the second row, so the first row ends early.
    let filler = "x ".repeat(25);
    let text = format!("{filler}{} tail\n", "w".repeat(60));
    let (_project, reader) = open(&text);
    // A window 110 px wider than 900 px, because the minimap takes its widest 110 px: the text area, and so the rows,
    //  are the ones a 900 px window gave before the minimap.
    arrange(&reader, 1.0, 900.0 + 110.0);
    let window = &reader.window;
    let rows = rows_of(&reader, 0);
    assert!(rows.len() >= 2, "the line wraps");
    key(window, Key::End);
    key(window, Key::End);
    assert_eq!(position(&reader).1, rows[0].end);
    assert_eq!(affinity(&reader), ReadingAffinity::Upstream);
    let end_x = window.get_caret_x();
    // Down to the second row and ten glyphs to the right of the end of the first, then Up again.
    key(window, Key::DownArrow);
    assert_eq!(drawn_row(&reader), 1);
    for _ in 0..10 {
        key(window, Key::RightArrow);
    }
    assert!(
        window.get_caret_x() > end_x + 80.0,
        "the aim is right of the end of the first row"
    );
    key(window, Key::UpArrow);
    assert_eq!(
        drawn_row(&reader),
        0,
        "Up lands on the upper row, where its text ends"
    );
    assert_eq!(position(&reader).1, rows[0].end);
    assert_eq!(affinity(&reader), ReadingAffinity::Upstream);
    assert_eq!(window.get_caret_x(), end_x);
    window.hide().expect("close wrap window");
}

/// PageDown and PageUp move the caret by the visual rows a page holds,
/// scroll the view by those rows,
///  and keep the caret's visual row in view.
#[test]
fn page_keys_move_by_visual_rows_and_keep_the_caret_in_view() {
    for factor in [1.0, 2.0] {
        for width in widths(factor) {
            let label = format!("scale {factor} width {width}");
            let (_project, reader) = open(&long_text(80));
            arrange(&reader, factor, width);
            let window = &reader.window;
            let page = (window.get_viewport_height() / 24.0).floor() as usize;
            assert!(page >= 4, "{label}: the window shows {page} rows");
            let mut row = 0;
            for step in 1..=4 {
                key(window, Key::PageDown);
                row += page;
                assert_eq!(
                    drawn_row(&reader),
                    row,
                    "{label}: PageDown {step} moves the caret by {page} visual rows"
                );
                assert!(
                    caret_in_view(&reader),
                    "{label}: the caret row is in view after PageDown {step}"
                );
                assert_eq!(
                    -window.get_scroll_y(),
                    (step * page) as f32 * 24.0,
                    "{label}: the view moves by those rows"
                );
            }
            chord(window, Key::Shift, Key::PageDown);
            assert!(
                position(&reader).0 < position(&reader).1,
                "{label}: Shift+PageDown extends"
            );
            row += page;
            for step in 0..4 {
                key(window, Key::PageUp);
                row -= page;
                assert_eq!(drawn_row(&reader), row, "{label}: PageUp {step}");
                assert!(
                    caret_in_view(&reader),
                    "{label}: the caret row is in view after PageUp {step}"
                );
            }
            assert_eq!(drawn_row(&reader), page, "{label}");
            window.hide().expect("close wrap window");
        }
    }
}

/// Moving through every visual row of a text keeps the caret's row in the view,
/// upwards and downwards.
#[test]
fn the_view_follows_the_caret_through_every_visual_row() {
    for width in widths(1.0) {
        let (_project, reader) = open(&long_text(30));
        arrange(&reader, 1.0, width);
        let window = &reader.window;
        let mut rows: usize = 0;
        for line in 0..30 {
            rows += reader.source.borrow().row_map.rows_of(line);
        }
        // What: The wrap width and the facts at the moment the rows were counted, kept for the failure message.
        // Why: A count taken at another wrap width than the Down presses ran at (the failure recorded in full runs
        //      of the minimap branch: 162 rows counted, 121 in the map afterwards) is told from a caret that
        //      stopped early only by these.
        let counted_at = (
            reader.source.borrow().width,
            window.get_viewport_width(),
            Facts {
                window,
                source: &reader.source,
            }
            .to_string(),
        );
        for step in 0..rows - 1 {
            key(window, Key::DownArrow);
            assert!(
                caret_in_view(&reader),
                "width {width}: Down {step} left the caret row out of view"
            );
        }
        let mut rows_after: usize = 0;
        for line in 0..30 {
            rows_after += reader.source.borrow().row_map.rows_of(line);
        }
        assert_eq!(
            drawn_row(&reader),
            rows - 1,
            "width {width}: after {} Down presses; rows counted {rows} at wrap width {} (viewport {}), rows in the map now \
             {rows_after} at wrap width {} (viewport {}); facts when counted {}; facts now {}",
            rows - 1,
            counted_at.0,
            counted_at.1,
            reader.source.borrow().width,
            window.get_viewport_width(),
            counted_at.2,
            Facts {
                window,
                source: &reader.source
            }
        );
        for step in 0..rows - 1 {
            key(window, Key::UpArrow);
            assert!(
                caret_in_view(&reader),
                "width {width}: Up {step} left the caret row out of view"
            );
        }
        assert_eq!(drawn_row(&reader), 0);
        window.hide().expect("close wrap window");
    }
}

/// Ctrl+Left and Ctrl+Right move by word in the document,
/// whatever the rows are,
/// also across a wrap offset.
#[test]
fn word_motion_crosses_a_wrap_offset_by_document_offset() {
    for width in widths(1.0) {
        let (_project, reader) = open(&long_text(2));
        arrange(&reader, 1.0, width);
        let window = &reader.window;
        let rows = rows_of(&reader, 0);
        let wrap = rows[0].end;
        let text = reader.source.borrow().document.text().clone();
        let mut head = 0;
        let mut reached = false;
        while head < rows[1].end {
            chord(window, Key::Control, Key::RightArrow);
            let expected = moved(text.slice(..), head, Motion::WordRight);
            assert_eq!(
                position(&reader),
                (expected, expected),
                "width {width}: Ctrl+Right from {head}"
            );
            assert_eq!(
                drawn_row(&reader),
                row_containing(&rows, expected, ReadingAffinity::Downstream),
                "width {width}: Ctrl+Right to {expected} is drawn on the row that holds it"
            );
            reached |= head < wrap && expected >= wrap;
            head = expected;
        }
        assert!(
            reached,
            "width {width}: a word stop reached or passed the wrap offset {wrap}"
        );
        while head > 0 {
            chord(window, Key::Control, Key::LeftArrow);
            let expected = moved(text.slice(..), head, Motion::WordLeft);
            assert_eq!(
                position(&reader),
                (expected, expected),
                "width {width}: Ctrl+Left from {head}"
            );
            assert_eq!(
                drawn_row(&reader),
                row_containing(
                    &rows,
                    expected.min(rows[rows.len() - 1].end),
                    ReadingAffinity::Downstream
                ),
                "width {width}: Ctrl+Left to {expected} is drawn on the row that holds it"
            );
            head = expected;
        }
        window.hide().expect("close wrap window");
    }
}

/// The window point over global visual row `row`,
/// `x` logical pixels into the text.
fn over(reader: &Reader, row: usize, x: f32) -> LogicalPosition {
    return LogicalPosition::new(
        TEXT_LEFT + x,
        HEADER + row as f32 * 24.0 + 12.0 + reader.window.get_scroll_y(),
    );
}

/// Press the left button.
fn press(reader: &Reader, position: LogicalPosition) {
    reader
        .window
        .window()
        .dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
}

/// Move the pointer.
fn drag(reader: &Reader, position: LogicalPosition) {
    reader
        .window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    update_timers_and_animations();
}

/// Release the left button.
fn release(reader: &Reader, position: LogicalPosition) {
    reader
        .window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
}

/// A drag from one visual row to another selects across the rows between,
/// also over a line boundary,
///  and a release ends the drag.
#[test]
fn a_drag_selects_across_visual_rows() {
    for width in widths(1.0) {
        let label = format!("width {width}");
        let (_project, reader) = open(&long_text(6));
        arrange(&reader, 1.0, width);
        let window = &reader.window;
        let first = rows_of(&reader, 0);
        assert!(first.len() >= 3, "{label}");
        press(&reader, over(&reader, 0, 120.0));
        let (anchor, _) = position(&reader);
        assert!(
            first[0].contains(&anchor),
            "{label}: the press is on the first row"
        );
        drag(&reader, over(&reader, 2, 300.0));
        let (from, to) = position(&reader);
        assert_eq!(from, anchor, "{label}: the anchor stays");
        assert!(
            first[2].contains(&to),
            "{label}: the drag reaches the third row, head {to}"
        );
        assert_eq!(
            window.get_source_selections().row_count(),
            3,
            "{label}: one rectangle per row the selection crosses"
        );
        // Over the next line.
        drag(&reader, over(&reader, first.len() + 1, 200.0));
        let second = rows_of(&reader, 1);
        let (_, onto) = position(&reader);
        assert!(
            second[1].contains(&onto),
            "{label}: the drag reaches the second row of the next line, head {onto}"
        );
        // Back up, and then release; a move after the release changes nothing.
        drag(&reader, over(&reader, 1, 50.0));
        release(&reader, over(&reader, 1, 50.0));
        let released = position(&reader);
        assert!(
            first[1].contains(&released.1),
            "{label}: dragged back to the second row"
        );
        drag(&reader, over(&reader, 3, 50.0));
        assert_eq!(
            position(&reader),
            released,
            "{label}: no drag after the release"
        );
        window.hide().expect("close wrap window");
    }
}

/// A double click selects the word under the pointer even when the word spans rows,
///  and a triple click selects the whole logical line,
/// all its rows.
#[test]
fn double_and_triple_clicks_work_on_wrapped_lines() {
    for width in widths(1.0) {
        let label = format!("width {width}");
        let token = "m".repeat(500);
        let text = format!("{token} end of line\nsecond\n");
        let (_project, reader) = open(&text);
        arrange(&reader, 1.0, width);
        let window = &reader.window;
        let rows = rows_of(&reader, 0);
        assert!(rows.len() >= 4, "{label}: {} rows", rows.len());
        let on_second_row = over(&reader, 1, 90.0);
        press(&reader, on_second_row);
        release(&reader, on_second_row);
        press(&reader, on_second_row);
        release(&reader, on_second_row);
        assert_eq!(
            position(&reader),
            (0, 500),
            "{label}: the word spans every row it is on"
        );
        assert_eq!(window.get_selected_text(), token, "{label}");
        press(&reader, on_second_row);
        release(&reader, on_second_row);
        let line_length = text.find('\n').expect("a line end") + 1;
        assert_eq!(
            position(&reader),
            (0, line_length),
            "{label}: the triple click takes the whole line"
        );
        assert_eq!(
            window.get_source_selections().row_count(),
            rows.len() + 1,
            "{label}: one rectangle per row and the mark of the selected line end"
        );
        window.hide().expect("close wrap window");
    }
}

/// Two quick presses at one x on two rows of the same line are two single clicks.
#[test]
fn quick_presses_on_two_rows_of_one_line_are_not_a_double_click() {
    let (_project, reader) = open(&long_text(3));
    arrange(&reader, 1.0, 1100.0);
    let one = over(&reader, 0, 100.0);
    let two = over(&reader, 1, 100.0);
    // The control: the same row twice is a double click.
    press(&reader, one);
    release(&reader, one);
    press(&reader, one);
    release(&reader, one);
    let (from, to) = position(&reader);
    assert!(to > from, "the double click on one row selects a word");
    sleep(Duration::from_millis(550));
    press(&reader, one);
    release(&reader, one);
    press(&reader, two);
    release(&reader, two);
    let (second_from, second_to) = position(&reader);
    assert_eq!(
        second_from, second_to,
        "a second press on the next row of the line is a single click"
    );
    reader.window.hide().expect("close wrap window");
}
