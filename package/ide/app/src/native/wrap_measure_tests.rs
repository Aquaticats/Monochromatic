//! The tile the window thread takes for a new width agrees with the map about every one of its lines.
//!  The heights the table held for the old width are stale inside the tile,
//! so the window thread measures the tile's lines again until no height changes.
//! The worker's measurement is held still,
//! so only the window thread can have corrected a height.

/// The reader fixture's waits.
use super::find_tests::Reader;
/// The table of measured rows the background pass fills.
use super::rows::absorb;
/// A new width and the held measurement.
use super::wrap_resize_tests::{change_width, hold_the_pass};
/// The open reader.
use super::wrap_tests::open;
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Library types that give the expected values.
use ide_app::shaped_text::TextShaper;
/// The toolkit's own handle.
use slint::ComponentHandle;

/// Lines that never wrap at the widths the tests use:
/// the constants of a source file.
fn short_text(count: usize) -> String {
    let mut text = String::new();
    for number in 0..count {
        text.push_str(&format!("export const alpha{number} = {number};\n"));
    }
    return text;
}

/// Lines of the displayed tile whose rows in the map differ from a fresh measurement at the wrap width.
fn stale_in_tile(reader: &Reader, scale: f32) -> Vec<(usize, usize, usize)> {
    let current = reader.source.borrow();
    let first = current.shown.first;
    let end = first + current.shown.count;
    let mut shaper = TextShaper::new();
    let fresh = shaper.measure_lines(
        &current.document,
        first..end,
        scale,
        Some(current.width),
        &[],
    );
    let mut stale = Vec::new();
    for (line, rows) in fresh.lines {
        if current.row_map.rows_of(line) != rows {
            stale.push((line, current.row_map.rows_of(line), rows));
        }
    }
    return stale;
}

/// A pass chunk measured at a narrow width leaves two rows for lines that one row holds at the new width.
/// The tile for the new width reaches further down the text at every round the window thread measures,
/// because the lines it has corrected are shorter:
/// the last lines of the tile are corrected by the last round,
/// and the pass never sends the lines of a displayed tile again.
#[test]
fn a_tile_taken_at_once_for_a_new_width_holds_only_measured_lines() {
    let release = hold_the_pass();
    let (_project, opened) = open(&short_text(1500));
    let window = &opened.window;
    let scale = window.window().scale_factor();
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                if current.tile_job.is_none() && current.scroll_wanted.is_none() {
                    break;
                }
            }
            waiting.step("the first tile did not settle");
        }
    }
    {
        let mut current = opened.source.borrow_mut();
        let narrow = (0..600).map(|line| return (line, 2));
        assert!(
            absorb(&mut current, narrow),
            "the stale heights changed nothing"
        );
    }
    change_width(&opened, 500.0, 660.0);
    {
        let current = opened.source.borrow();
        assert_eq!(
            current.rewrap_deferred, 0,
            "the width was left to the worker"
        );
        assert!(
            current.shown.count > 40,
            "the tile holds only {} lines",
            current.shown.count
        );
    }
    let stale = stale_in_tile(&opened, scale);
    assert!(
        stale.is_empty(),
        "{} lines of the tile keep a stale height, first: {:?}",
        stale.len(),
        &stale[..stale.len().min(5)]
    );
    drop(release);
    window.hide().expect("close wrap window");
}
