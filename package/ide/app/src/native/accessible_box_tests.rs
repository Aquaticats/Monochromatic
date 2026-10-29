//! The find box,
//!  the search box,
//!  and their clear controls as an assistive tool reaches them:
//!  through
//! Slint's element handles,
//!  which read the accessible role,
//!  label,
//!  value,
//!  placeholder,
//!  and description,
//! and invoke the accessible set-value and default actions,
//!  the same properties and actions the
//! platform accessibility bridge exposes.

/// The window type generated from the shipped markup.
use super::AppWindow;
/// The find fixture with the bar open and focused,
///  and the wait for the find count.
use super::find_clear_tests::opened;
/// Typing into whatever has keyboard focus,
///  one key press,
///  and the bounded wait for a condition.
use super::find_tests::{key, type_text};
/// The bounded wait for native navigation state and the tree-row lookup.
use super::navigation_tests::row;
/// The search reader and the double-Shift opening of the search overlay.
use super::search_tests::{open, reader};
/// An exact window size and settled layout.
use super::sidebar_tests::{resize, settle};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// What:
///  `ElementHandle` is Slint's test-only handle on one element of a live window;
///  `AccessibleRole`
/// is the toolkit's enum of accessible roles (`TextInput`,
///  `Button`,
///  `Slider`,
///  `List`,
///  and so on).
/// Why:
///  Searching by accessible label and reading roles and values is what an assistive tool does,
/// so these tests check what such a tool would find,
///  not the window's private properties.
/// Gotcha:
///  The crate is internal to Slint and must have exactly the resolved Slint version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ElementHandle, AccessibleRole } from 'slint-testing';
/// ```
use i_slint_backend_testing::{AccessibleRole, ElementHandle};
/// Window ownership,
///  row counts of the result model,
///  and the toolkit's key names.
use slint::{ComponentHandle, Model, platform::Key};
/// Fixture files for the disposable search project.
use std::fs;

/// What:
///  `window: &AppWindow` lends the window;
///  `label: &str` borrows the label text;
///  the answer is an
/// owned `ElementHandle`.
/// Why:
///  Every check starts by finding exactly one element with a given accessible label;
///  a second
/// element with the same label would leave an assistive tool with two indistinguishable targets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function only(window: AppWindow, label: string): ElementHandle;
/// ```
pub(super) fn only(window: &AppWindow, label: &str) -> ElementHandle {
    // What: `find_by_accessible_label` returns an iterator, a lazy sequence that hands out one match per
    // `next()` call; `mut` lets the calls advance it.
    // Why: The first `next()` takes the match and the second proves there is no other.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found = ElementHandle.findByAccessibleLabel(window, label)[Symbol.iterator]();
    // ```
    let mut found = ElementHandle::find_by_accessible_label(window, label);
    // What: `next()` returns an `Option`: `Some(element)` or `None` when the sequence is empty;
    // `let Some(first) = ... else { ... }` keeps the element or runs the `else` block, which must leave;
    // `panic!` fails the test with a formatted message.
    // Why: A missing element is the failure under test, so the message names the label.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const first = found.next().value;
    // if (first === undefined) throw new Error(`assistive tools find no element labelled ${label}`);
    // ```
    let Some(first) = found.next() else {
        panic!("assistive tools find no element labelled {label:?}");
    };
    // What: `is_none()` is true when the `Option` is `None`.
    // Why: Exactly one element may carry the label.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!found.next().done) fail(...);
    // ```
    assert!(
        found.next().is_none(),
        "assistive tools find more than one element labelled {label:?}"
    );
    return first;
}

/// The number of visible elements with this accessible label;
///  hidden elements are not offered.
pub(super) fn count(window: &AppWindow, label: &str) -> usize {
    return ElementHandle::find_by_accessible_label(window, label).count();
}

/// What:
///  `element: &ElementHandle` lends the handle;
///  `name` labels messages.
/// Why:
///  A clear control is a button an assistive tool can find,
///  of at least the 48px by 48px target size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertClearButton(element: ElementHandle, name: string): void;
/// ```
fn assert_clear_button(element: &ElementHandle, name: &str) {
    // What: `Some(AccessibleRole::Button)` is the present variant of `Option` holding the button role;
    // `assert_eq!` compares both sides and prints them on failure.
    // Why: The role tells an assistive tool that the element is activated, not edited.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert.equal(element.accessibleRole, AccessibleRole.Button);
    // ```
    assert_eq!(
        element.accessible_role(),
        Some(AccessibleRole::Button),
        "the {name} is not a button"
    );
    let size = element.size();
    assert!(
        size.width >= 48.0 && size.height >= 48.0,
        "the {name} measures {} by {}, below 48 by 48",
        size.width,
        size.height
    );
}

/// The find box is a text input labelled "Find text" with its placeholder,
///  value,
///  and the match count as
/// its description;
///  setting its value runs find.
///  The clear control is a 48px button labelled
/// "Clear find text" that exists only while there is text,
///  and its default action clears and keeps focus.
#[test]
fn find_box_and_clear_control_expose_role_label_value_and_actions() {
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
    let field = only(window, "Find text");
    assert_eq!(
        field.accessible_role(),
        Some(AccessibleRole::TextInput),
        "the find box is not a text input"
    );
    // What: `.as_deref()` turns an `Option` holding an owned string into an `Option` holding a borrowed
    // `&str`, so it compares with a string literal.
    // Why: The handle returns Slint's owned string type; the expected values are literals.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert.equal(field.accessiblePlaceholderText, 'Find in file');
    // ```
    assert_eq!(
        field.accessible_placeholder_text().as_deref(),
        Some("Find in file"),
        "the find box does not report its placeholder"
    );
    assert_eq!(
        field.accessible_value().as_deref(),
        Some(""),
        "the empty find box reports a value"
    );
    assert_eq!(
        field.accessible_enabled(),
        Some(true),
        "the open find box is reported as disabled"
    );
    assert_eq!(
        count(window, "Clear find text"),
        0,
        "assistive tools are offered a clear control for empty find text"
    );
    // Setting the value is how an assistive tool types; it must run find like typing does.
    field.set_accessible_value("needle");
    {
        let mut waiting = Waiting::eventually();
        while !(field.accessible_description().as_deref() == Some("Match 1 of 2")) {
            waiting.step("setting the find box's value did not run find");
        }
    }
    assert_eq!(
        window.get_find_query(),
        "needle",
        "setting the find box's value did not change the find text"
    );
    assert_eq!(
        field.accessible_value().as_deref(),
        Some("needle"),
        "the find box does not report its new value"
    );
    let clear = only(window, "Clear find text");
    assert_clear_button(&clear, "find box's clear control");
    clear.invoke_accessible_default_action();
    settle(window);
    assert_eq!(
        field.accessible_value().as_deref(),
        Some(""),
        "the clear control's default action did not clear the find box"
    );
    {
        let mut waiting = Waiting::eventually();
        while !(field.accessible_description().as_deref() == Some("")) {
            waiting.step("clearing did not remove the find count");
        }
    }
    assert!(
        window.get_find_has_focus(),
        "the clear control's default action moved keyboard focus out of the find box"
    );
    assert_eq!(
        count(window, "Clear find text"),
        0,
        "the clear control is still offered after clearing"
    );
    window.hide().expect("close accessible find window");
}

/// The search box is a text input labelled "Search query" with its placeholder and value;
///  setting its
/// value searches.
///  The clear control is a 48px button labelled "Clear search query";
///  its default action
/// clears the query and the results and keeps keyboard focus in the box.
#[test]
fn search_box_and_clear_control_expose_role_label_value_and_actions() {
    // What: `tempfile::tempdir()` makes a disposable directory removed when the value is dropped.
    // Why: Search runs over a real project, which must never be the repository.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fixture = mkdtempSync(join(tmpdir(), 'search-'));
    // ```
    let fixture = tempfile::tempdir().expect("disposable accessible search project");
    // What: `fixture.path().join(..)` builds a path inside the directory; `fs::write` creates the file.
    // Why: One file whose name and contents match the query.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeFileSync(join(fixture, 'needle-file.txt'), 'needle here');
    // ```
    fs::write(fixture.path().join("needle-file.txt"), "needle here").expect("search fixture");
    let reader = reader(fixture.path());
    let window = &reader.window;
    resize(window, 1100.0, 660.0);
    // What: `|| return ...` is a zero-argument arrow function; `is_some()` is true for `Some`.
    // Why: The search waits for the tree, so the project is loaded before the overlay opens.
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
    let field = only(window, "Search query");
    assert_eq!(
        field.accessible_role(),
        Some(AccessibleRole::TextInput),
        "the search box is not a text input"
    );
    assert_eq!(
        field.accessible_placeholder_text().as_deref(),
        Some("Files and contents · % for contents only"),
        "the search box does not report its placeholder"
    );
    assert_eq!(
        field.accessible_value().as_deref(),
        Some(""),
        "the empty search box reports a value"
    );
    assert_eq!(
        count(window, "Clear search query"),
        0,
        "assistive tools are offered a clear control for an empty query"
    );
    field.set_accessible_value("needle");
    {
        let mut waiting = Waiting::eventually();
        while !(!window.get_search_busy() && window.get_search_entries().row_count() > 0) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    settle(window);
    assert_eq!(
        window.get_search_query(),
        "needle",
        "setting the search box's value did not change the query"
    );
    assert_eq!(
        field.accessible_value().as_deref(),
        Some("needle"),
        "the search box does not report its new value"
    );
    let clear = only(window, "Clear search query");
    assert_clear_button(&clear, "search box's clear control");
    clear.invoke_accessible_default_action();
    settle(window);
    assert_eq!(
        field.accessible_value().as_deref(),
        Some(""),
        "the clear control's default action did not clear the search box"
    );
    assert!(
        !window.get_search_busy() && window.get_search_entries().row_count() == 0,
        "the clear control's default action left search results"
    );
    // Typed text reaches the box only while it has keyboard focus.
    type_text(window, "n");
    assert_eq!(
        window.get_search_query(),
        "n",
        "the clear control's default action moved keyboard focus out of the search box"
    );
    window.invoke_search_dismiss();
    window.hide().expect("close accessible search window");
}

/// What:
///  `field` lends the search box's handle;
///  `expected` is the description it must report.
/// Why:
///  Every step of the description test reads the same property and names it in the failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function described(field: ElementHandle, expected: string, after: string): void;
/// ```
fn described(field: &ElementHandle, expected: &str, after: &str) {
    // What: `.unwrap_or_default()` takes the text out of the `Option`, or an empty text when there is none.
    // Why: An absent description and an empty one read the same to an assistive tool.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const shown = field.accessibleDescription ?? '';
    // ```
    let shown = field.accessible_description().unwrap_or_default();
    assert_eq!(
        shown.as_str(),
        expected,
        "after {after} the search box's description is not the expected count text"
    );
}

/// The search box's description counts the results for assistive tools,
///  worded like the find box's match
/// count:
///  nothing without a query,
///  "Searching" while results are pending,
///  "Result 1 of N" for the selected
/// row and following it,
///  and "No results" when nothing matches.
#[test]
fn search_box_description_counts_the_results() {
    let fixture = tempfile::tempdir().expect("disposable search-count project");
    // Two files whose names match, so the selected row can move.
    fs::write(fixture.path().join("needle-one.txt"), "first\n").expect("search fixture");
    fs::write(fixture.path().join("needle-two.txt"), "second\n").expect("search fixture");
    let reader = reader(fixture.path());
    let window = &reader.window;
    resize(window, 1100.0, 660.0);
    {
        let mut waiting = Waiting::eventually();
        while !(row(window, "needle-two.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    open(window);
    let field = only(window, "Search query");
    described(&field, "", "opening the overlay");
    field.set_accessible_value("needle");
    assert!(
        window.get_search_busy(),
        "positive control: setting the query did not start a search"
    );
    described(&field, "Searching", "setting the query");
    {
        let mut waiting = Waiting::eventually();
        while !(!window.get_search_busy()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    settle(window);
    let total = window.get_search_entries().row_count();
    assert_eq!(total, 2, "positive control: the fixture gives two results");
    described(&field, "Result 1 of 2", "the results arrived");
    key(window, Key::DownArrow);
    settle(window);
    described(&field, "Result 2 of 2", "Down");
    field.set_accessible_value("zzzz");
    {
        let mut waiting = Waiting::eventually();
        while !(!window.get_search_busy()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    settle(window);
    described(&field, "No results", "a query that matches nothing");
    window.invoke_search_dismiss();
    window.hide().expect("close search-count window");
}
