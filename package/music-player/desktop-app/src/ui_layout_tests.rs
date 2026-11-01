//! Headless layout tests for the composed `AppWindow`.
//!
//! What: builds the real window on the testing backend and checks the key elements at the cover width (443 px)
//! and the unfolded width (850 px), in light and dark schemes.
//! Why: the layout branch is chosen by window width, so a width-specific regression shows only at that width.
//! Note: element lookup by type name needs `SLINT_EMIT_DEBUG_INFO=1` at build time, which the repo test task sets.

use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;

use crate::AppWindow;
use crate::ui_screen_view::Route;

/// What: `init_platform` installs the testing backend once per process.
/// Why: Slint's platform is process-wide, so a second install from another test would fail.
fn init_platform() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(i_slint_backend_testing::init_no_event_loop);
}

/// What: `window_at` builds the window at `width` with the given scheme (1 light, 2 dark).
/// Why: every test starts from the same window construction, so only the width and scheme differ.
fn window_at(width: f32, scheme: i32) -> AppWindow {
    init_platform();
    let app = AppWindow::new().expect("the window should build on the testing backend");
    app.window().set_size(slint::LogicalSize::new(width, 900.0));
    app.global::<crate::Theme>().set_scheme_override(scheme);
    app.set_route(Route::Player.to_int());
    return app;
}

/// What: `count_of` counts the elements of one component type under the window.
fn count_of(app: &AppWindow, type_name: &str) -> usize {
    return ElementHandle::find_by_element_type_name(app, type_name).count();
}

fn check_cover_width_elements() {
    for scheme in [1, 2] {
        let app = window_at(443.0, scheme);
        assert_eq!(count_of(&app, "CoverTopRow"), 1, "scheme {scheme}: cover top row");
        assert_eq!(count_of(&app, "TrackListPane"), 1, "scheme {scheme}: track list");
        assert_eq!(count_of(&app, "TransportDeck"), 1, "scheme {scheme}: transport deck");
    }
}

fn check_unfolded_width_elements() {
    for scheme in [1, 2] {
        let app = window_at(850.0, scheme);
        assert_eq!(count_of(&app, "UnfoldedFoldersHeader"), 1, "scheme {scheme}: left header");
        assert_eq!(count_of(&app, "UnfoldedTrackHeader"), 1, "scheme {scheme}: right header");
        assert_eq!(count_of(&app, "FolderPickerPane"), 1, "scheme {scheme}: folder picker");
        assert_eq!(count_of(&app, "TrackListPane"), 1, "scheme {scheme}: track list");
    }
}

fn check_track_list_widths() {
    let cover = window_at(443.0, 2);
    let cover_list = ElementHandle::find_by_element_type_name(&cover, "TrackListPane")
        .next()
        .expect("cover track list should exist");
    assert!((cover_list.size().width - 443.0).abs() < 1.0, "cover list width {}", cover_list.size().width);
    let unfolded = window_at(850.0, 2);
    let right_list = ElementHandle::find_by_element_type_name(&unfolded, "TrackListPane")
        .next()
        .expect("unfolded track list should exist");
    assert!(right_list.size().width < 850.0 * 0.6, "unfolded right pane width {}", right_list.size().width);
}

fn check_search_route() {
    let app = window_at(443.0, 1);
    assert_eq!(count_of(&app, "SearchPage"), 0, "search hidden on the player route");
    app.set_route(Route::Search.to_int());
    assert_eq!(count_of(&app, "SearchPage"), 1, "search shown on the search route");
}

/// What: `layout_matches_the_design_at_cover_and_unfolded_widths` runs every layout check in order on one thread.
/// Why: the testing platform and its windows are per process and per thread, so the checks share one test.
#[test]
fn layout_matches_the_design_at_cover_and_unfolded_widths() {
    check_cover_width_elements();
    check_unfolded_width_elements();
    check_track_list_widths();
    check_search_route();
}
