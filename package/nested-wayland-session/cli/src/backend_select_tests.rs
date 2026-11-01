//! Backend selection regressions independent of a display.

use std::ffi::OsStr;

use super::*;

/// Every combination of the three parent display variables,
/// present or absent.
fn every_parent_combination() -> Vec<ParentEnvironment> {
    let mut combinations = Vec::new();
    for bits in 0..8u8 {
        combinations.push(ParentEnvironment {
            wayland_display: bits & 1 != 0,
            wayland_socket: bits & 2 != 0,
            x11_display: bits & 4 != 0,
        });
    }
    return combinations
}

/// Without the flag the nested window is chosen whether or not a parent display exists.
#[test]
fn default_backend_is_nested_winit_for_every_parent_combination() {
    let combinations = every_parent_combination();
    assert_eq!(combinations.len(), 8);
    for parent in combinations {
        let choice = select_backend(false, parent);
        assert_eq!(choice.kind, BackendKind::NestedWinit, "{parent:?}");
        assert!(!choice.ignored_parent_display, "{parent:?}");
    }
}

/// A missing parent display never turns headless on by itself.
#[test]
fn absent_parent_display_does_not_select_headless() {
    let absent = ParentEnvironment::default();
    assert!(!absent.any_display());
    assert_eq!(select_backend(false, absent).kind, BackendKind::NestedWinit);
}

/// The flag selects headless with or without a parent display,
/// and reports an ignored parent.
#[test]
fn headless_flag_selects_headless_for_every_parent_combination() {
    for parent in every_parent_combination() {
        let choice = select_backend(true, parent);
        assert_eq!(choice.kind, BackendKind::Headless, "{parent:?}");
        assert_eq!(choice.ignored_parent_display, parent.any_display(), "{parent:?}");
    }
}

/// Empty values count as absent,
/// like libwayland and Xlib treat them.
#[test]
fn empty_values_count_as_absent() {
    let parent = ParentEnvironment::from_values(Some(OsStr::new("")), None, Some(OsStr::new("")));
    assert_eq!(parent, ParentEnvironment::default());
    let present = ParentEnvironment::from_values(Some(OsStr::new("wayland-0")), None, Some(OsStr::new(":0")));
    assert!(present.wayland_display && present.x11_display && !present.wayland_socket);
}

/// With no parent display the failure names the flag and keeps the original cause.
#[test]
fn failure_without_parent_display_names_the_headless_flag() {
    let message = nested_failure_message("NoCompositor", ParentEnvironment::default());
    assert!(message.contains("--headless"), "{message}");
    assert!(message.contains("NoCompositor"), "{message}");
    assert!(message.contains("WAYLAND_DISPLAY"), "{message}");
}

/// With a parent display the failure keeps the original message unchanged.
#[test]
fn failure_with_parent_display_keeps_the_original_message() {
    for parent in every_parent_combination().into_iter().filter(|parent| return parent.any_display()) {
        let message = nested_failure_message("NoCompositor", parent);
        assert_eq!(message, "winit backend init failed: NoCompositor", "{parent:?}");
    }
}
