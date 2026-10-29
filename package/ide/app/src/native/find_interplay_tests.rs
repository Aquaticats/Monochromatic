//! In-file find beside file switching and the modal search overlay,
//!  through real window key events.

/// The complete reader fixture and key helpers of the find tests.
use super::find_tests::{chord, key, reader, selection, settle, status, status_for, type_text};
/// Bounded waits and tree-row lookup shared with the navigation tests.
use super::navigation_tests::row;
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// Real toolkit key events reach the same capture scopes as seat input.
use slint::{ComponentHandle, Model, platform::Key};
/// What:
///  `fs` writes disposable fixture files;
///  nothing here touches a real project.
/// Why:
///  State-changing verification must run against throwaway directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as fs from 'node:fs';
/// ```
use std::fs;

/// Switching files with the bar open recomputes for the new file and keeps typing in the find input.
#[test]
fn native_find_recomputes_for_a_switched_file_and_keeps_input_focus() {
    let fixture = tempfile::tempdir().expect("disposable switch project");
    fs::write(
        fixture.path().join("alpha.txt"),
        "alpha needle\nand needle\n",
    )
    .expect("first fixture");
    fs::write(
        fixture.path().join("beta.txt"),
        "beta: needle needle needle\n",
    )
    .expect("second fixture");
    let reader = reader(fixture.path(), "alpha.txt");
    let window = &reader.window;
    {
        let mut waiting = Waiting::eventually();
        while !(row(window, "beta.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    chord(window, Key::Control, "f");
    type_text(window, "needle");
    status_for(window, "needle", "1/2");
    let generation = reader.source.borrow().file_generation;
    window.invoke_tree_activate(row(window, "beta.txt").expect("second file row"));
    {
        let mut waiting = Waiting::eventually();
        while !(reader.source.borrow().file_generation > generation) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    {
        let mut waiting = Waiting::eventually();
        while !(window.get_find_status() == "0/3") {
            waiting.step("matches were not recomputed for the new file");
        }
    }
    assert!(
        window.get_find_open(),
        "switching files closed the find bar"
    );
    assert!(
        window.get_find_has_focus(),
        "a file open moved keyboard focus out of the find input"
    );
    assert_eq!(
        window.get_source_matches().row_count(),
        3,
        "the new file's matches were not marked"
    );
    assert_eq!(
        selection(&reader),
        (0, 0),
        "a file switch must not move the selection"
    );
    key(window, Key::Return);
    status(window, "1/3");
    assert_eq!(selection(&reader), (6, 12));
    let switched = reader.source.borrow().file_generation;
    // The recent-file shortcut is pressed while the find input has focus.
    chord(window, Key::Control, "1");
    {
        let mut waiting = Waiting::eventually();
        while !(reader.source.borrow().file_generation > switched) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    status(window, "0/2");
    assert_eq!(window.get_find_query(), "needle");
    assert_eq!(
        window.get_source_text(),
        "alpha needle\nand needle\n",
        "Ctrl+1 in the find input did not switch back"
    );
    assert!(window.get_find_has_focus());
    type_text(window, "s");
    assert_eq!(
        window.get_find_query(),
        "needles",
        "typing after the switch left the find input"
    );
    status(window, "No matches");
    key(window, Key::Escape);
    assert!(!window.get_find_open());
    window.hide().expect("close switch window");
}

/// Escape closes only the topmost surface;
///  Ctrl+F is ignored under the modal search overlay.
#[test]
fn native_find_and_search_overlay_close_topmost_first() {
    let fixture = tempfile::tempdir().expect("disposable overlay project");
    fs::write(fixture.path().join("layers.txt"), "needle and needle\n").expect("overlay fixture");
    let reader = reader(fixture.path(), "layers.txt");
    let window = &reader.window;
    chord(window, Key::Control, "f");
    type_text(window, "needle");
    status_for(window, "needle", "1/2");
    key(window, Key::Shift);
    key(window, Key::Shift);
    assert!(
        window.get_search_open(),
        "double-Shift did not open search above the find bar"
    );
    assert!(window.get_find_open(), "opening search closed the find bar");
    chord(window, Key::Control, "f");
    type_text(window, "q");
    assert_eq!(
        window.get_search_query(),
        "q",
        "Ctrl+F took keyboard focus from the modal search overlay"
    );
    assert_eq!(window.get_find_query(), "needle");
    key(window, Key::Escape);
    assert!(
        !window.get_search_open(),
        "Escape did not close the topmost search overlay"
    );
    assert!(
        window.get_find_open(),
        "one Escape closed both the search overlay and the find bar"
    );
    assert_eq!(window.get_find_status(), "1/2");
    assert_eq!(window.get_source_matches().row_count(), 1);
    key(window, Key::Escape);
    assert!(
        !window.get_find_open(),
        "the second Escape did not close the find bar"
    );
    key(window, Key::Shift);
    key(window, Key::Shift);
    assert!(window.get_search_open());
    chord(window, Key::Control, "f");
    assert!(
        !window.get_find_open(),
        "Ctrl+F opened the find bar under the modal search overlay"
    );
    key(window, Key::Escape);
    assert!(!window.get_search_open());
    assert!(!window.get_find_open());
    // Escape reaches the bar from the tree as well, and focus then moves to the source.
    chord(window, Key::Control, "f");
    assert!(window.get_find_open());
    window.invoke_focus_tree();
    assert!(window.get_tree_has_focus());
    key(window, Key::Escape);
    assert!(
        !window.get_find_open(),
        "Escape from the tree did not close the find bar"
    );
    assert!(
        !window.get_tree_has_focus() && !window.get_find_has_focus(),
        "closing the find bar did not move keyboard focus to the source"
    );
    window.hide().expect("close overlay window");
}

/// Refused find text shows its diagnostic;
///  empty text clears highlights without claiming "No matches".
#[test]
fn native_find_reports_refused_text_and_clears_for_empty_text() {
    let fixture = tempfile::tempdir().expect("disposable diagnostic project");
    fs::write(fixture.path().join("bounds.txt"), "needle\n").expect("diagnostic fixture");
    let reader = reader(fixture.path(), "bounds.txt");
    let window = &reader.window;
    window.invoke_focus_tree();
    window.invoke_find_dismiss();
    assert!(
        window.get_tree_has_focus(),
        "dismissing a closed find bar moved keyboard focus"
    );
    window.set_source_available(false);
    chord(window, Key::Control, "f");
    assert!(
        !window.get_find_open(),
        "Ctrl+F opened the find bar without a displayed file"
    );
    window.set_source_available(true);
    window.invoke_find_edited("needle".into());
    // The same shortcut from the tree is the positive control for the ignored one.
    chord(window, Key::Control, "f");
    assert!(
        window.get_find_open() && window.get_find_has_focus(),
        "Ctrl+F from the tree did not open and focus the find bar"
    );
    settle();
    assert_eq!(
        selection(&reader),
        (0, 0),
        "an edit reached the closed find bar"
    );
    assert_eq!(
        window.get_find_status(),
        "",
        "empty find text must show no count"
    );
    type_text(window, "needle");
    status_for(window, "needle", "1/1");
    window.invoke_find_edited("n".repeat(1001).into());
    {
        let mut waiting = Waiting::eventually();
        while !(window.get_find_error() != "") {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert!(window.get_find_error().contains("at most 1000 characters"));
    assert_eq!(
        window.get_source_matches().row_count(),
        0,
        "refused find text kept old highlights"
    );
    assert!(!window.get_selection_is_match());
    window.invoke_find_edited("needle".into());
    status(window, "1/1");
    assert_eq!(
        window.get_find_error(),
        "",
        "a later valid query kept the diagnostic"
    );
    window.invoke_find_edited("".into());
    assert_eq!(
        window.get_find_status(),
        "",
        "empty find text must show no count"
    );
    assert!(!window.get_find_no_match());
    assert!(!window.get_selection_is_match());
    assert_eq!(
        window.get_selected_text(),
        "needle",
        "clearing find text must keep the selection"
    );
    window.hide().expect("close diagnostic window");
}
