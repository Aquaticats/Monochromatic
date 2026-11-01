//! Unit tests for the window answers.

use super::{format_focus, format_windows, WindowInfo};

/// Builds an info for the tests.
fn info(app_id: &str, title: &str, focused: bool) -> WindowInfo {
    return WindowInfo { app_id: app_id.to_string(), title: title.to_string(), focused }
}

/// No windows list as zero and have no focus.
#[test]
fn an_empty_list_is_zero_and_none() {
    assert_eq!(format_windows(&[]), "0");
    assert_eq!(format_focus(&[]), "none");
}

/// Records carry the index,
///  encoded names,
///  and the focus flag.
#[test]
fn records_are_indexed_and_encoded() {
    let windows = [info("player", "Music; 1=a b", false), info("dialog", "Pick a folder", true)];
    assert_eq!(
        format_windows(&windows),
        "2 index=0 app-id=player title=Music%3B%201%3Da%20b focused=false; \
         index=1 app-id=dialog title=Pick%20a%20folder focused=true",
    );
}

/// The focus answer is the focused window's record.
#[test]
fn focus_names_the_focused_window() {
    let windows = [info("player", "Music", false), info("dialog", "Pick", true)];
    assert_eq!(format_focus(&windows), "index=1 app-id=dialog title=Pick focused=true");
    assert_eq!(format_focus(&[info("player", "Music", false)]), "none");
}

/// A line break in a title cannot end the answer.
#[test]
fn a_line_break_in_a_title_is_encoded() {
    assert_eq!(format_windows(&[info("a", "x\ny", false)]), "1 index=0 app-id=a title=x%0Ay focused=false");
}
