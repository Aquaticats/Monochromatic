//! Unit tests for the cursor answer.

use smithay::input::pointer::{CursorIcon, CursorImageStatus};

use super::describe_cursor;

/// Named cursors use the cursor-shape protocol's names.
#[test]
fn named_cursors_use_shape_names() {
    assert_eq!(describe_cursor(&CursorImageStatus::Named(CursorIcon::Default)), "default");
    assert_eq!(describe_cursor(&CursorImageStatus::Named(CursorIcon::Pointer)), "pointer");
    assert_eq!(describe_cursor(&CursorImageStatus::Named(CursorIcon::Text)), "text");
    assert_eq!(describe_cursor(&CursorImageStatus::Named(CursorIcon::EwResize)), "ew-resize");
}

/// A hidden cursor is reported as hidden.
#[test]
fn a_hidden_cursor_is_hidden() {
    assert_eq!(describe_cursor(&CursorImageStatus::Hidden), "hidden");
}
