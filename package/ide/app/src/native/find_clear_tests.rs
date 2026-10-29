//! The find box's clear control through real pointer events:
//!  its 48px cell,
//!  its edges,
//!  focus,
//!  and find results.

/// The generated window type from the shipped markup.
use super::AppWindow;
/// Real key events,
///  the full production reader,
///  and bounded waits shared with the find tests.
use super::find_tests::{chord, reader, status_for, type_text};
/// Pointer helpers that dispatch real window events and then lay the window out again.
use super::sidebar_tests::{click, resize, settle};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Toolkit key names,
///  window ownership,
///  and the model interface that counts match rectangles.
use slint::{ComponentHandle, Model, platform::Key};
/// Fixture files are written into a disposable project directory.
use std::fs;

/// What:
///  A record of four `f32` values,
///  the 32-bit float Slint uses for logical pixels (sibling `f64`
/// has twice the precision):
///  the clear cell's left edge,
///  top edge,
///  width,
///  and height in the window.
/// Why:
///  Every click in these tests is placed relative to the cell the layout actually produced,
/// so the tests measure the cell instead of assuming it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cell = { x: number; y: number; width: number; height: number };
/// ```
struct Cell {
    /// Left edge in window pixels.
    x: f32,
    /// Top edge in window pixels.
    y: f32,
    /// Width of the layout cell.
    width: f32,
    /// Height of the layout cell.
    height: f32,
}

/// What:
///  `window: &AppWindow` lends the window without giving it away;
///  the answer is an owned `Cell`.
/// Why:
///  The cell is read after a layout pass,
///  from the properties the box reports about itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cell(window: AppWindow): Cell;
/// ```
fn cell(window: &AppWindow) -> Cell {
    settle(window);
    return Cell {
        x: window.get_find_clear_x(),
        y: window.get_find_clear_y(),
        width: window.get_find_clear_width(),
        height: window.get_find_clear_height(),
    };
}

/// Type find text with two matches into the focused find box and wait for its count.
fn found(window: &AppWindow) {
    type_text(window, "needle");
    status_for(window, "needle", "1/2");
    settle(window);
}

/// What:
///  The answer is a pair:
///  the disposable directory (`TempDir`,
///  deleted when dropped) and the reader.
/// Why:
///  The directory must outlive the reader's file workers,
///  so the caller keeps both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function opened(): [TempDir, Reader];
/// ```
pub(super) fn opened() -> (tempfile::TempDir, super::find_tests::Reader) {
    // What: `expect` returns the successful value or fails the test with this message.
    // Why: A fixture that cannot be written is a broken test, not behavior under test.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fixture = mkdtempSync(...); writeFileSync(join(fixture, 'clear.txt'), text);
    // ```
    let fixture = tempfile::tempdir().expect("disposable clear project");
    fs::write(fixture.path().join("clear.txt"), "needle one\nneedle two\n").expect("clear fixture");
    let opened = reader(fixture.path(), "clear.txt");
    resize(&opened.window, 1100.0, 660.0);
    chord(&opened.window, Key::Control, "f");
    assert!(
        opened.window.get_find_has_focus(),
        "Ctrl+F did not focus the find box"
    );
    return (fixture, opened);
}

/// The cell measures at least 48px by 48px.
///  A click on each of its corners and on its center clears the
/// find text,
///  keeps keyboard focus in the box,
///  and removes the count and the highlights.
#[test]
fn find_clear_cell_is_48px_and_every_part_of_it_clears() {
    // `_fixture` keeps the directory alive until the test ends.
    let (_fixture, reader) = opened();
    // What: `&reader.window` borrows the window out of the reader record.
    // Why: The helpers take a borrowed window, and the reader keeps owning it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = reader.window;
    // ```
    let window = &reader.window;
    found(window);
    assert!(
        window.get_find_clear_shown(),
        "the clear control is hidden although the focused box has text"
    );
    let target = cell(window);
    assert!(
        target.width >= 48.0 && target.height >= 48.0,
        "the clear cell measures {} by {}, below 48 by 48",
        target.width,
        target.height
    );
    // Half a pixel inside each edge is the middle of the cell's first or last pixel on that side.
    let right = target.width - 0.5;
    let bottom = target.height - 0.5;
    // What: Each `(across, down)` pair is a two-value tuple taken apart by the loop header.
    // Why: The four corner pixels and the center are the points a 48px by 48px target must cover.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const [across, down] of [[0.5, 0.5], [right, 0.5], [0.5, bottom], [right, bottom], center]) { ... }
    // ```
    for (across, down) in [
        (0.5, 0.5),
        (right, 0.5),
        (0.5, bottom),
        (right, bottom),
        (target.width / 2.0, target.height / 2.0),
    ] {
        click(window, target.x + across, target.y + down);
        assert_eq!(
            window.get_find_query(),
            "",
            "a click {across}px across and {down}px down the clear cell did not clear the find text"
        );
        assert!(
            window.get_find_has_focus(),
            "clearing moved keyboard focus out of the find box"
        );
        // What: `|| { ... }` is a zero-argument arrow function that `eventually` calls until it holds.
        // Why: The count and the highlights are removed by native code when the box reports the edit.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // eventually(message, () => window.findStatus === '' && window.sourceMatches.length === 0);
        // ```
        {
            let mut waiting = Waiting::eventually();
            while !(window.get_find_status() == "" && window.get_source_matches().row_count() == 0)
            {
                waiting.step("clearing did not update the find results");
            }
        }
        // Typing again without a click proves that the keyboard still reaches the box.
        found(window);
    }
    window.hide().expect("close clear window");
}

/// A click one pixel outside each edge of the cell clears nothing.
///  The control exists only while the box
/// has keyboard focus and text,
///  which is the toolkit's rule;
///  hidden,
///  its cell is the 12px trailing padding.
#[test]
fn find_clear_cell_ends_at_its_edges_and_hides_without_focus_or_text() {
    let (_fixture, reader) = opened();
    let window = &reader.window;
    assert!(
        !window.get_find_clear_shown(),
        "the clear control is shown for empty find text"
    );
    assert_eq!(
        cell(window).width,
        12.0,
        "the hidden clear control's cell is not the 12px trailing padding"
    );
    found(window);
    let target = cell(window);
    assert_eq!(target.width, 48.0, "shown clear cell positive control");
    let middle = target.y + target.height / 2.0;
    let center = target.x + target.width / 2.0;
    // The last text pixel, the spacing right of the box, and the bar's padding above and below the box.
    for (x, y) in [
        (target.x - 0.5, middle),
        (target.x + target.width + 0.5, middle),
        (center, target.y - 0.5),
        (center, target.y + target.height + 0.5),
    ] {
        click(window, x, y);
        assert_eq!(
            window.get_find_query(),
            "needle",
            "a click at {x}, {y}, outside the clear cell, cleared the find text"
        );
    }
    assert!(
        window.get_find_has_focus(),
        "a click beside the clear cell took keyboard focus from the find box"
    );
    window.invoke_focus_source();
    assert!(
        !window.get_find_clear_shown(),
        "the clear control is shown without keyboard focus in the box"
    );
    assert_eq!(
        cell(window).width,
        12.0,
        "the unfocused box did not return the clear cell to trailing padding"
    );
    // Where the control was is text area again: a click there focuses the box and clears nothing.
    click(window, center, middle);
    assert_eq!(
        window.get_find_query(),
        "needle",
        "a click where the hidden clear control was cleared the find text"
    );
    assert!(
        window.get_find_has_focus() && window.get_find_clear_shown(),
        "a click on the text area did not focus the box and show the clear control again"
    );
    window.hide().expect("close clear window");
}
