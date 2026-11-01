//! Window tests of the Licenses row of Settings and the Licenses page it opens.
//!
//! What: each test builds the real window on the testing backend (or the software renderer for the focus ring), sends real
//! pointer and key events and accessibility actions, and reads what the row says to assistive technology, where its parts
//! sit, which page is shown, and what the page holds.
//! Why: the row, the route, Escape and the notice list only exist together in the composed window, and Slint's Royalty-free
//! license is met only if the page really shows the AboutSlint widget.
//! Note: element lookup by id needs `SLINT_EMIT_DEBUG_INFO=1` at build time, which the repo test task sets.

use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use music_player::settings_layout::{
    HEADER_HEIGHT, ICON_DIAMETER, ICON_X_FROM_EDGE, ROW_HEIGHT, SIDE_MARGIN, TITLE_X_FROM_EDGE, list_pane_width, row_width,
};
use slint::{ComponentHandle, Model, ModelRc};

use crate::AppWindow;
use crate::ui_focus_ring_tests::{Case, case, check_focused, run_case};
use crate::ui_licenses::NoticeLines;
use crate::ui_render_support::Rendered;
use crate::ui_screen::{read_state, update_state};
use crate::ui_screen_view::Route;
use crate::ui_settings::open_settings;
use crate::ui_test_support::{chord, fresh_window_at, settle, tap};

/// Height of the section heading above a group: 16 above the label, the label, and 8 below.
const HEADING_HEIGHT: f32 = 44.0;

/// What: `open_settings_at` builds a window of the given width and opens Settings on it.
fn open_settings_at(width: f32) -> AppWindow {
    // Tall enough for the whole list: the Text size group put the Licenses row below 900 px, and a row outside the window
    // is not listed by the element search.
    let (app, _host) = fresh_window_at(width, 1800.0);
    open_settings(&app);
    settle();
    return app;
}

/// What: `elements` lists the elements with a qualified id, in tree order.
fn elements(app: &AppWindow, id: &str) -> Vec<ElementHandle> {
    return ElementHandle::find_by_element_id(app, id).collect();
}

/// What: `only` gives the single element with a qualified id.
fn only(app: &AppWindow, id: &str) -> ElementHandle {
    let mut found = elements(app, id);
    assert_eq!(found.len(), 1, "exactly one {id}, found {}", found.len());
    return found.remove(0);
}

/// What: `box_of` gives an element's window position and size as `(x, y, width, height)`.
fn box_of(element: &ElementHandle) -> (f32, f32, f32, f32) {
    let position = element.absolute_position();
    let size = element.size();
    return (position.x, position.y, size.width, size.height);
}

/// What: `count_of` counts the elements of one component type under the window.
fn count_of(app: &AppWindow, type_name: &str) -> usize {
    return ElementHandle::find_by_element_type_name(app, type_name).count();
}

/// What: `escape` classifies and runs an Escape key press the way the window's key scope does.
fn escape(app: &AppWindow) -> bool {
    let key_scope_used = app.invoke_key_pressed("\u{1b}".into(), false, false, false, false, false, false);
    settle();
    return key_scope_used;
}

/// What: `open_licenses_by_keyboard` moves the keyboard to the Licenses row with End and presses Enter.
fn open_licenses_by_keyboard(app: &AppWindow) {
    tap(app, "End");
    tap(app, "Enter");
}

/// What: `y_of_first_notice_line` gives the window y of the first line the list has built.
fn y_of_first_notice_line(app: &AppWindow) -> f32 {
    return elements(app, "LicensesPage::notice-line").iter().map(|line| return box_of(line).1).fold(f32::MAX, f32::min);
}

//region The row

#[test]
fn the_licenses_row_is_a_labelled_button_under_an_about_heading_in_both_forms() {
    for width in [360.0, 443.0, 850.0] {
        let app = open_settings_at(width);
        let touch = only(&app, "SettingsPage::licenses-touch");
        assert_eq!(touch.accessible_role(), Some(AccessibleRole::Button), "width {width}");
        assert_eq!(touch.accessible_label().as_deref(), Some("Licenses"), "width {width}");
        assert_eq!(
            touch.accessible_description().as_deref(),
            Some("Third-party notices"),
            "width {width}"
        );
        // The row sits one 44 px heading below the Reset text sizes row that ends the group before About, 16 px in from the
        // pane edges, and is as tall as every row.
        let switch = only(&app, "SettingsPage::switch-touch");
        let reset = only(&app, "SettingsPage::reset-touch");
        let (switch_x, _switch_y, switch_width, _switch_height) = box_of(&switch);
        let (_reset_x, reset_y, _reset_width, reset_height) = box_of(&reset);
        let (x, y, row_wide, row_high) = box_of(&touch);
        assert_eq!((x, row_high), (SIDE_MARGIN, ROW_HEIGHT), "width {width}");
        assert_eq!(x, switch_x, "width {width}");
        assert_eq!(row_wide, switch_width.min(row_width(list_pane_width(width))), "width {width}");
        assert_eq!(y, reset_y + reset_height + HEADING_HEIGHT, "width {width}");
    }
}

#[test]
fn the_licenses_row_icon_and_title_sit_where_a_template_row_puts_them() {
    let app = open_settings_at(443.0);
    let (row_x, row_y, _width, _height) = box_of(&only(&app, "SettingsPage::licenses-touch"));
    let (icon_x, icon_y, icon_width, icon_height) = box_of(&only(&app, "SettingsPage::licenses-icon"));
    assert_eq!((icon_x, icon_width, icon_height), (ICON_X_FROM_EDGE, ICON_DIAMETER, ICON_DIAMETER));
    assert_eq!(icon_y, row_y + (ROW_HEIGHT - ICON_DIAMETER) / 2.0, "the circle is centered on the row");
    assert_eq!(box_of(&only(&app, "SettingsPage::licenses-title")).0, TITLE_X_FROM_EDGE);
    assert_eq!(row_x, SIDE_MARGIN);
}

#[test]
fn the_accessibility_action_of_the_licenses_row_opens_the_page_and_keeps_the_editor() {
    for width in [443.0, 850.0] {
        let app = open_settings_at(width);
        let kind_before = read_state(|state| return state.editor.as_ref().map(|editor| return editor.kind()));
        only(&app, "SettingsPage::licenses-touch").invoke_accessible_default_action();
        settle();
        assert_eq!(app.get_route(), Route::Licenses.to_int(), "width {width}");
        assert_eq!(count_of(&app, "LicensesPage"), 1, "width {width}");
        assert_eq!(count_of(&app, "SettingsPage"), 0, "the Settings page is replaced at width {width}");
        assert_eq!(count_of(&app, "TemplateEditor"), 0, "so is the editor pane at width {width}");
        let kind_now = read_state(|state| return state.editor.as_ref().map(|editor| return editor.kind()));
        assert_eq!(kind_now, kind_before, "the open template is kept at width {width}");
    }
}

#[test]
fn a_click_on_the_licenses_row_opens_the_page() {
    let app = open_settings_at(443.0);
    let touch = only(&app, "SettingsPage::licenses-touch");
    let (x, y, width, height) = box_of(&touch);
    let position = slint::LogicalPosition::new(x + width / 2.0, y + height / 2.0);
    app.window().dispatch_event(slint::platform::WindowEvent::PointerMoved { position });
    app.window().dispatch_event(slint::platform::WindowEvent::PointerPressed {
        position,
        button: slint::platform::PointerEventButton::Left,
    });
    app.window().dispatch_event(slint::platform::WindowEvent::PointerReleased {
        position,
        button: slint::platform::PointerEventButton::Left,
    });
    settle();
    assert_eq!(app.get_route(), Route::Licenses.to_int());
}

//endregion

//region The page

#[test]
fn the_page_shows_the_made_with_slint_widget_the_royalty_free_license_asks_for() {
    for width in [360.0, 443.0, 850.0] {
        let app = open_settings_at(width);
        open_licenses_by_keyboard(&app);
        assert_eq!(app.get_route(), Route::Licenses.to_int(), "width {width}");
        let widgets = ElementHandle::find_by_element_type_name(&app, "AboutSlint").collect::<Vec<_>>();
        assert_eq!(widgets.len(), 1, "the page holds exactly one AboutSlint at width {width}");
        let (x, y, widget_width, widget_height) = box_of(&widgets[0]);
        assert!(widget_width > 0.0 && widget_height >= 60.0, "the widget has room: {widget_width} by {widget_height}");
        assert!(x >= 0.0 && x + widget_width <= width + 0.5, "the widget is inside the window at width {width}");
        assert!(y >= HEADER_HEIGHT, "the widget is below the header at width {width}");
        let mut labels = Vec::new();
        for image in ElementHandle::find_by_element_type_name(&app, "Image") {
            labels.extend(image.accessible_label());
        }
        assert!(labels.iter().any(|label| return label == "#MadeWithSlint"), "the widget's logo is announced: {labels:?}");
    }
}

#[test]
fn the_page_gives_the_list_the_notice_lines_and_releases_them_on_close() {
    let app = open_settings_at(443.0);
    assert_eq!(app.get_licenses_lines().row_count(), 0, "Settings holds no notice text");
    open_licenses_by_keyboard(&app);
    let expected = NoticeLines::new(music_player::licenses::NOTICES).row_count();
    assert!(expected > 1);
    assert_eq!(app.get_licenses_lines().row_count(), expected, "every line of the embedded notices");
    assert!(escape(&app));
    assert_eq!(app.get_licenses_lines().row_count(), 0, "the text is released when the page closes");
}

#[test]
fn the_list_builds_only_the_lines_in_view_of_a_40000_line_text() {
    let app = open_settings_at(443.0);
    open_licenses_by_keyboard(&app);
    let text: &'static str = Box::leak((0..40_000).map(|line| format!("Line {line} of a long license text\n")).collect::<String>().into_boxed_str());
    app.set_licenses_lines(ModelRc::new(NoticeLines::new(text)));
    settle();
    let built = elements(&app, "LicensesPage::notice-line").len();
    assert!(built > 5, "some lines are drawn: {built}");
    assert!(built < 150, "far fewer than 40000 lines are built: {built}");
}

#[test]
fn the_page_announces_its_list_and_its_back_button() {
    let app = open_settings_at(443.0);
    open_licenses_by_keyboard(&app);
    let list = only(&app, "LicensesPage::notice-list");
    assert_eq!(list.accessible_label().as_deref(), Some("License notices"));
    assert_eq!(list.accessible_role(), Some(AccessibleRole::List));
    let back = only(&app, "LicensesPage::back-touch");
    assert_eq!(back.accessible_role(), Some(AccessibleRole::Button));
    assert_eq!(back.accessible_label().as_deref(), Some("Back"));
}

#[test]
fn the_arrow_and_page_keys_scroll_the_notices_and_home_returns_to_the_top() {
    let app = open_settings_at(443.0);
    open_licenses_by_keyboard(&app);
    let text: &'static str = Box::leak((0..400).map(|line| format!("Line {line} of a long license text\n")).collect::<String>().into_boxed_str());
    app.set_licenses_lines(ModelRc::new(NoticeLines::new(text)));
    settle();
    let top = y_of_first_notice_line(&app);
    tap(&app, "Down");
    let after_down = y_of_first_notice_line(&app);
    assert!(after_down < top, "Down scrolls the text up: {top} to {after_down}");
    tap(&app, "PageDown");
    let after_page = y_of_first_notice_line(&app);
    assert!(after_page < after_down, "Page Down scrolls further: {after_down} to {after_page}");
    tap(&app, "Up");
    assert!(y_of_first_notice_line(&app) > after_page, "Up scrolls back");
    tap(&app, "End");
    tap(&app, "Home");
    assert_eq!(y_of_first_notice_line(&app), top, "Home returns to the first line");
}

//endregion

//region Closing

#[test]
fn escape_returns_from_licenses_to_settings_and_a_second_escape_closes_settings() {
    for width in [443.0, 850.0] {
        let app = open_settings_at(width);
        open_licenses_by_keyboard(&app);
        assert_eq!(app.get_route(), Route::Licenses.to_int(), "width {width}");
        assert!(escape(&app), "the key scope uses Escape on the Licenses page at width {width}");
        assert_eq!(app.get_route(), Route::Settings.to_int(), "Escape returns to Settings at width {width}");
        assert_eq!(count_of(&app, "SettingsPage"), 1, "width {width}");
        assert!(read_state(|state| return state.editor.is_some()), "the editor value is still held at width {width}");
        assert!(escape(&app));
        assert_eq!(app.get_route(), Route::Player.to_int(), "a second Escape closes Settings at width {width}");
    }
}

#[test]
fn the_back_arrow_returns_to_settings_and_the_keyboard_is_on_the_licenses_row() {
    let app = open_settings_at(443.0);
    open_licenses_by_keyboard(&app);
    only(&app, "LicensesPage::back-touch").invoke_accessible_default_action();
    settle();
    assert_eq!(app.get_route(), Route::Settings.to_int());
    // Enter acts on the focused row: it is the Licenses row again, so the page opens a second time.
    tap(&app, "Enter");
    assert_eq!(app.get_route(), Route::Licenses.to_int(), "focus came back on the Licenses row");
}

#[test]
fn opening_licenses_from_the_wide_settings_leaves_the_template_selection_alone() {
    let app = open_settings_at(850.0);
    let selected = app.get_settings_selected();
    open_licenses_by_keyboard(&app);
    assert!(escape(&app));
    assert_eq!(app.get_settings_selected(), selected, "the selected template row is unchanged");
    assert_eq!(app.get_route(), Route::Settings.to_int());
}

#[test]
fn settings_opened_afresh_starts_on_the_first_row_not_on_licenses() {
    let app = open_settings_at(443.0);
    open_licenses_by_keyboard(&app);
    assert!(escape(&app));
    assert!(escape(&app), "closes Settings");
    assert_eq!(app.get_route(), Route::Player.to_int());
    update_state(|state| state.route = Route::Player);
    chord(&app, &["Ctrl", "Alt"], "s");
    settle();
    assert_eq!(app.get_route(), Route::Settings.to_int());
    tap(&app, "Enter");
    assert_eq!(app.get_route(), Route::TemplateEditor.to_int(), "Enter acts on the first row after a fresh open");
}

#[test]
fn a_template_editor_visited_after_licenses_returns_to_the_first_row_not_to_licenses() {
    let app = open_settings_at(443.0);
    open_licenses_by_keyboard(&app);
    assert!(escape(&app));
    assert_eq!(app.get_route(), Route::Settings.to_int());
    tap(&app, "Home");
    tap(&app, "Enter");
    assert_eq!(app.get_route(), Route::TemplateEditor.to_int(), "Home then Enter opens the first template");
    app.invoke_editor_back();
    settle();
    assert_eq!(app.get_route(), Route::Settings.to_int());
    tap(&app, "Enter");
    assert_eq!(app.get_route(), Route::TemplateEditor.to_int(), "the keyboard is on the first row again, not on Licenses");
}

//endregion

//region The ring

#[test]
fn the_licenses_row_ring_follows_the_14_px_group_corners_and_the_page_back_ring_is_a_circle() {
    let rendered = Rendered::new(443, 900);
    chord(&rendered.app, &["Ctrl", "Alt"], "s");
    rendered.wait(300);
    assert_eq!(rendered.app.get_route(), Route::Settings.to_int());
    tap(&rendered.app, "End");
    let row = Case { radii: [14.0; 4], ..case("settings-list", "SettingsPage::licenses-row", (0.0, 0.0), 0.0) };
    check_focused(&rendered, &row);
    tap(&rendered.app, "Enter");
    rendered.wait(300);
    assert_eq!(rendered.app.get_route(), Route::Licenses.to_int());
    run_case(&rendered, &case("licenses-back", "LicensesPage::back-stop", (0.0, 0.0), 28.0));
}

//endregion
