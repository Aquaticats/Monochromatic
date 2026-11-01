// Unit tests for `crate::model` (the pane-strip spawn/dedup/close state machine). Exempt from
// require-rustdoc/max-lines because the file name ends in `_tests.rs`.

use std::path::PathBuf;

use crate::model::PaneStripState;
use crate::types::PaneLocation;

fn dir(path: &str) -> PaneLocation {
    return PaneLocation::Directory(PathBuf::from(path))
}

fn preview(path: &str) -> PaneLocation {
    return PaneLocation::Preview(PathBuf::from(path))
}

fn panes_in_column(strip: &PaneStripState, column: usize) -> usize {
    return strip.panes().filter(|pane| return pane.column == column).count()
}

#[test]
fn open_root_places_pane_in_column_zero_and_focuses() {
    let mut strip = PaneStripState::new();
    let root = strip.open_root(dir("/home"));
    assert_eq!(strip.len(), 1);
    assert_eq!(strip.active(), Some(root));
    let pane = strip.pane(root).expect("root pane");
    assert_eq!((pane.column, pane.row), (0, 0));
    assert_eq!(strip.first_pane_in_column(0), Some(root));
}

#[test]
fn spawn_child_opens_next_column_aligned_to_parent_row() {
    let mut strip = PaneStripState::new();
    let root = strip.open_root(dir("/home"));
    let child = strip.spawn_child(root, dir("/home/docs"), false);
    assert_ne!(root, child);
    let pane = strip.pane(child).expect("child pane");
    // one column right, same row as the parent (aligned, reads straight across)
    assert_eq!((pane.column, pane.row), (1, 0));
    assert_eq!(strip.active(), Some(child));
    assert_eq!(strip.len(), 2);
}

#[test]
fn child_aligns_with_parent_row_and_siblings_stack_downward() {
    let mut strip = PaneStripState::new();
    let root = strip.open_root(dir("/home"));
    let first = strip.spawn_child(root, dir("/home/a"), false);
    let second = strip.spawn_child(root, dir("/home/b"), false);
    // siblings of the root stack: first at row 0, second pushed to row 1
    assert_eq!((strip.pane(first).unwrap().column, strip.pane(first).unwrap().row), (1, 0));
    assert_eq!((strip.pane(second).unwrap().column, strip.pane(second).unwrap().row), (1, 1));
    // a child of `second` (row 1) aligns with it at row 1, not the top of the next column
    let grandchild = strip.spawn_child(second, dir("/home/b/c"), false);
    assert_eq!((strip.pane(grandchild).unwrap().column, strip.pane(grandchild).unwrap().row), (2, 1));
}

#[test]
fn a_later_sibling_is_pushed_below_a_grown_subtree() {
    let mut strip = PaneStripState::new();
    let root = strip.open_root(dir("/home"));
    let a = strip.spawn_child(root, dir("/home/a"), false);
    let b = strip.spawn_child(root, dir("/home/b"), false);
    let c = strip.spawn_child(root, dir("/home/c"), false);
    // grow b's subtree to two rows
    strip.spawn_child(b, dir("/home/b/x"), false);
    strip.spawn_child(b, dir("/home/b/y"), false);
    // a is a leaf at row 0; b's subtree occupies rows 1..2; c is pushed down to row 3
    assert_eq!(strip.pane(a).unwrap().row, 0);
    assert_eq!(strip.pane(b).unwrap().row, 1);
    assert_eq!(strip.pane(c).unwrap().row, 3);
}

#[test]
fn revisiting_a_location_dedups_and_focuses_existing() {
    let mut strip = PaneStripState::new();
    let root = strip.open_root(dir("/home"));
    let first = strip.spawn_child(root, dir("/home/docs"), false);
    strip.focus(root);
    let again = strip.spawn_child(root, dir("/home/docs"), false);
    assert_eq!(first, again);
    assert_eq!(strip.len(), 2);
    assert_eq!(strip.active(), Some(first));
}

#[test]
fn ctrl_click_forces_a_duplicate_pane() {
    let mut strip = PaneStripState::new();
    let root = strip.open_root(dir("/home"));
    let first = strip.spawn_child(root, dir("/home/docs"), false);
    let dup = strip.spawn_child(root, dir("/home/docs"), true);
    assert_ne!(first, dup);
    assert_eq!(strip.len(), 3);
}

#[test]
fn close_removes_pane_and_clears_dedup() {
    let mut strip = PaneStripState::new();
    let root = strip.open_root(dir("/home"));
    let child = strip.spawn_child(root, dir("/home/docs"), false);
    strip.close(child);
    assert_eq!(strip.len(), 1);
    assert!(strip.pane(child).is_none());
    let respawn = strip.spawn_child(root, dir("/home/docs"), false);
    assert_ne!(child, respawn);
    assert_eq!(strip.len(), 2);
}

#[test]
fn close_column_removes_all_panes_in_it() {
    let mut strip = PaneStripState::new();
    let root = strip.open_root(dir("/home"));
    strip.spawn_child(root, dir("/home/a"), false);
    strip.spawn_child(root, dir("/home/b"), false);
    assert_eq!(panes_in_column(&strip, 1), 2);
    strip.close_column(1);
    assert_eq!(strip.len(), 1);
    assert_eq!(panes_in_column(&strip, 1), 0);
}

#[test]
fn close_right_of_clears_the_tail() {
    let mut strip = PaneStripState::new();
    let root = strip.open_root(dir("/home"));
    let a = strip.spawn_child(root, dir("/home/a"), false);
    strip.spawn_child(a, dir("/home/a/b"), false);
    strip.close_right_of(0);
    assert_eq!(strip.len(), 1);
    assert_eq!(strip.active(), None);
}

#[test]
fn preview_and_directory_locations_are_distinct_keys() {
    let mut strip = PaneStripState::new();
    let root = strip.open_root(dir("/home"));
    let listing = strip.spawn_child(root, dir("/home/x"), false);
    let prev = strip.spawn_child(root, preview("/home/x"), false);
    assert_ne!(listing, prev);
    assert_eq!(strip.len(), 3);
}
