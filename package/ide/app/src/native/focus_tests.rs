//! Tab and Shift+Tab move between the tree,
//!  the source view,
//!  and the open find bar;
//!  typing never edits source.

/// The production window whose focus state is read back.
use super::AppWindow;
/// The complete reader fixture and real key-event helpers shared with the find tests.
use super::find_tests::{chord, key, reader, type_text};
/// What:
///  `Key` names toolkit special keys;
///  converting one into text yields the encoded key string.
/// Why:
///  Tests dispatch the same encoded keys that the seat delivers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Key } from 'slint';
/// ```
use slint::{ComponentHandle, platform::Key};
/// Disposable project files back every native window in these tests.
use std::fs;

/// What:
///  `&'static str` is a borrowed string that lives for the whole program (sibling:
///  owned `String`).
/// Why:
///  One comparable word per focus owner makes a wrong traversal order readable in the failure text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function focused(window: AppWindow): 'tree' | 'source' | 'find' | 'none';
/// ```
fn focused(window: &AppWindow) -> &'static str {
    if window.get_tree_has_focus() {
        return "tree";
    }
    if window.get_source_has_focus() {
        return "source";
    }
    if window.get_find_has_focus() {
        return "find";
    }
    if window.get_sidebar_divider_has_focus() {
        return "divider";
    }
    return "none";
}

/// Press Tab `count` times and collect the focus owner after each press.
fn tab(window: &AppWindow, count: usize) -> Vec<&'static str> {
    // What: `Vec::new()` creates an empty growable list (siblings: fixed array `[T; N]`, borrowed slice `&[T]`).
    // Why: The number of presses is a parameter, so the list grows per press.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const seen: string[] = [];
    // ```
    let mut seen = Vec::new();
    for _ in 0..count {
        key(window, Key::Tab);
        seen.push(focused(window));
    }
    return seen;
}

/// Press Shift+Tab `count` times,
///  as the seat delivers it:
///  the Tab key while Shift is held.
fn shift_tab(window: &AppWindow, count: usize) -> Vec<&'static str> {
    let mut seen = Vec::new();
    for _ in 0..count {
        chord(window, Key::Shift, Key::Tab);
        seen.push(focused(window));
    }
    return seen;
}

/// Without a find bar the only stops are the tree,
///  the sidebar divider,
///  and the source view,
///  in both directions.
#[test]
fn tab_cycles_tree_divider_and_source_while_find_is_closed() {
    let fixture = tempfile::tempdir().expect("disposable focus project");
    fs::write(fixture.path().join("focus.txt"), "one\ntwo\n").expect("focus fixture");
    let reader = reader(fixture.path(), "focus.txt");
    let window = &reader.window;
    assert_eq!(focused(window), "source");
    assert_eq!(
        tab(window, 4),
        ["tree", "divider", "source", "tree"],
        "Tab must cycle tree, divider, and source only"
    );
    assert_eq!(
        shift_tab(window, 4),
        ["source", "divider", "tree", "source"],
        "Shift+Tab must cycle source, divider, and tree only"
    );
    // Some platforms deliver Shift+Tab as one dedicated key instead of Tab with a modifier.
    key(window, Key::Backtab);
    assert_eq!(
        focused(window),
        "divider",
        "the dedicated back-tab key did not leave the source view"
    );
    key(window, Key::Backtab);
    assert_eq!(focused(window), "tree");
    window.hide().expect("close focus window");
}

/// With the find bar open the order is tree,
///  source,
///  find input,
///  then the tree again.
#[test]
fn tab_visits_tree_source_and_open_find_bar_in_reading_order() {
    let fixture = tempfile::tempdir().expect("disposable focus project");
    fs::write(fixture.path().join("focus.txt"), "one\ntwo\n").expect("focus fixture");
    let reader = reader(fixture.path(), "focus.txt");
    let window = &reader.window;
    chord(window, Key::Control, "f");
    assert_eq!(focused(window), "find");
    assert_eq!(
        tab(window, 6),
        ["tree", "divider", "source", "find", "tree", "divider"],
        "Tab order with an open find bar"
    );
    assert_eq!(
        shift_tab(window, 6),
        ["tree", "find", "source", "divider", "tree", "find"],
        "Shift+Tab order with an open find bar"
    );
    assert_eq!(
        window.get_find_query(),
        "",
        "Tab must not be typed into the find input"
    );
    key(window, Key::Escape);
    assert_eq!(
        focused(window),
        "source",
        "closing the bar returns to source"
    );
    assert_eq!(
        tab(window, 3),
        ["tree", "divider", "source"],
        "the closed find bar must not remain a stop"
    );
    window.hide().expect("close focus window");
}

/// Typing,
///  deleting,
///  pasting,
///  and line breaks in the focused source view change neither text nor file.
#[test]
fn typing_in_the_source_view_is_ignored() {
    let fixture = tempfile::tempdir().expect("disposable focus project");
    let path = fixture.path().join("focus.txt");
    fs::write(&path, "one\ntwo\n").expect("focus fixture");
    let reader = reader(fixture.path(), "focus.txt");
    let window = &reader.window;
    assert_eq!(focused(window), "source");
    type_text(window, "typed 123");
    for special in [Key::Return, Key::Backspace, Key::Delete, Key::Space] {
        key(window, special);
    }
    for letter in ["v", "x", "z", "y"] {
        chord(window, Key::Control, letter);
    }
    assert_eq!(
        window.get_source_text(),
        "one\ntwo\n",
        "typing changed displayed source"
    );
    assert_eq!(
        fs::read_to_string(&path).expect("fixture after typing"),
        "one\ntwo\n",
        "typing changed the project file"
    );
    assert_eq!(focused(window), "source", "typing moved keyboard focus");
    window.hide().expect("close focus window");
}
