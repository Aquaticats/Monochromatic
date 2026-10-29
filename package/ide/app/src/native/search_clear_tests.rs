//! The search box's clear control through real key and pointer events:
//!  its 48px cell,
//!  focus,
//!  and results.

/// Real typing into whatever has keyboard focus,
///  shared with the find tests.
use super::find_tests::type_text;
/// Bounded waits and tree-row lookup shared with the navigation tests.
use super::navigation_tests::row;
/// The source-empty reader and the double-Shift opener of the search tests.
use super::search_tests::{open, reader};
/// Pointer helpers that dispatch real window events and then lay the window out again.
use super::sidebar_tests::{click, resize, settle};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// Window ownership and the model interface that counts result rows.
use slint::{ComponentHandle, Model};
/// Fixture files are written into a disposable project directory.
use std::fs;

/// The cell measures at least 48px by 48px;
///  the pixels beside it clear nothing;
///  a click on it empties the
/// query,
///  removes the results,
///  and leaves keyboard focus in the box.
#[test]
fn search_clear_cell_is_48px_clears_the_query_and_results_and_keeps_focus() {
    // What: `expect` returns the successful value or fails the test with this message.
    // Why: A fixture that cannot be written is a broken test, not behavior under test.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fixture = mkdtempSync(...); writeFileSync(join(fixture, 'needle-file.txt'), 'needle here');
    // ```
    let fixture = tempfile::tempdir().expect("disposable search-clear project");
    fs::write(fixture.path().join("needle-file.txt"), "needle here").expect("search fixture");
    let reader = reader(fixture.path());
    // What: `&reader.window` borrows the window out of the reader record.
    // Why: The helpers take a borrowed window, and the reader keeps owning it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = reader.window;
    // ```
    let window = &reader.window;
    resize(window, 1100.0, 660.0);
    // What: `|| return ...` is a zero-argument arrow function that `wait_until` calls until it holds;
    // `is_some()` asks whether the optional row index is present.
    // Why: The tree lists the project asynchronously, and the search scope comes from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // waitUntil(() => row(window, 'needle-file.txt') !== undefined);
    // ```
    {
        let mut waiting = Waiting::eventually();
        while !(row(window, "needle-file.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    open(window);
    assert!(
        !window.get_search_clear_shown(),
        "the clear control is shown for an empty query"
    );
    type_text(window, "needle");
    assert_eq!(
        window.get_search_query(),
        "needle",
        "typing did not reach the search box"
    );
    {
        let mut waiting = Waiting::eventually();
        while !(!window.get_search_busy() && window.get_search_entries().row_count() > 0) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    settle(window);
    assert!(
        window.get_search_clear_shown(),
        "the clear control is hidden although the focused box has text"
    );
    let x = window.get_search_clear_x();
    let y = window.get_search_clear_y();
    let width = window.get_search_clear_width();
    let height = window.get_search_clear_height();
    assert!(
        width >= 48.0 && height >= 48.0,
        "the clear cell measures {width} by {height}, below 48 by 48"
    );
    let middle = y + height / 2.0;
    // The last text pixel left of the cell and the panel's padding right of it.
    click(window, x - 0.5, middle);
    click(window, x + width + 0.5, middle);
    assert_eq!(
        window.get_search_query(),
        "needle",
        "a click beside the clear cell cleared the query"
    );
    assert!(
        window.get_search_open(),
        "a click on the panel's padding dismissed the search overlay"
    );
    click(window, x + width / 2.0, middle);
    assert_eq!(
        window.get_search_query(),
        "",
        "a click on the clear cell did not clear the query"
    );
    assert!(
        !window.get_search_busy() && window.get_search_entries().row_count() == 0,
        "clearing the query left search results"
    );
    type_text(window, "n");
    assert_eq!(
        window.get_search_query(),
        "n",
        "clearing moved keyboard focus out of the search box"
    );
    window.invoke_search_dismiss();
    window.hide().expect("close search-clear reader");
}
