//! Headless tests for the Settings page on the testing backend: row geometry in both forms, selection without
//! layout shift, the fade, the keyboard, and Escape and Back closing in both forms.
//!
//! What: builds the real `AppWindow`, registers the same Settings callbacks the app registers, and drives it with
//! window events (pointer, keys) and accessibility actions.
//! Why: the layout rules of D109 and D110 and the glue between the page and the editor state only show up in the
//! composed window; the numbers come from `music_player::settings_layout`, so the markup is compared with the rule.
//! Note: element lookup by id needs `SLINT_EMIT_DEBUG_INFO=1` at build time, which the repo test task sets.

use i_slint_backend_testing::ElementHandle;
use music_player::settings_layout::{
    PILL_RADIUS, HEADER_HEIGHT, ICON_DIAMETER, ROW_HEIGHT, SIDE_MARGIN, editor_pane_width, icon_x_in_row, icon_y_in_row, list_pane_width,
    row_top, row_width, title_x_in_row,
};
use music_player::template_model::TemplateKind;
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, SharedString};

use crate::AppWindow;
use crate::Focus;
use crate::ui_screen::{project, read_state, update_state};
use crate::ui_test_support::fresh_window_at;
use crate::ui_screen_view::Route;
use crate::ui_settings::open_settings;

/// Height of the window in every test, tall enough for the whole Settings list.
const WINDOW_HEIGHT: f32 = 900.0;

/// Height of the section heading above the first row: 16 above the label, the label, and 8 below.
const HEADING_HEIGHT: f32 = 44.0;

thread_local! {
    /// The texts each save call received on this test thread, so a test sees what would have reached the session file.
    static SAVED: std::cell::RefCell<Vec<(Option<String>, Option<String>)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// What: `record_save` stands in for the session file: it keeps the texts it is given and writes nothing.
fn record_save(track_rows: Option<&str>, playing: Option<&str>) -> std::io::Result<()> {
    SAVED.with(|saved| saved.borrow_mut().push((track_rows.map(str::to_string), playing.map(str::to_string))));
    return Ok(());
}

/// What: `saved_texts` lists the saves of this test so far.
fn saved_texts() -> Vec<(Option<String>, Option<String>)> {
    return SAVED.with(|saved| return saved.borrow().clone());
}

/// What: `settle` lets Slint run its pending `changed` handlers and timers, as the event loop does before a frame.
fn settle() {
    slint::platform::update_timers_and_animations();
}

/// What: `window_at` builds the window at `width`, light scheme, with the Settings callbacks and the key route wired.
/// Why: the app registers the same callbacks in `ui_settings::wire` and `ui_wire::wire_keys`; the engine is left out
/// because no Settings key needs it, and the session file is never read or written.
fn window_at(width: f32) -> AppWindow {
    let (app, _host) = fresh_window_at(width, WINDOW_HEIGHT);
    crate::ui_settings::replace_saver(record_save);
    return app;
}

/// What: `open_settings_at` builds the window at `width` and opens Settings on it.
fn open_settings_at(width: f32) -> AppWindow {
    let app = window_at(width);
    open_settings(&app);
    settle();
    return app;
}

/// What: `elements` lists the elements with a qualified id, in tree order.
fn elements(app: &AppWindow, id: &str) -> Vec<ElementHandle> {
    return ElementHandle::find_by_element_id(app, id).collect();
}

/// What: `count_of` counts the elements of one component type under the window.
fn count_of(app: &AppWindow, type_name: &str) -> usize {
    return ElementHandle::find_by_element_type_name(app, type_name).count();
}

/// What: `box_of` gives an element's window position and size as `(x, y, width, height)`.
fn box_of(element: &ElementHandle) -> (f32, f32, f32, f32) {
    let position = element.absolute_position();
    let size = element.size();
    return (position.x, position.y, size.width, size.height);
}

/// What: `row_layout` lists the geometry of every row element, so two moments can be compared for any shift.
/// Why: a row is its touch area, its icon circle and its title; the editor pane's own box is part of the layout too.
fn row_layout(app: &AppWindow) -> Vec<(String, (f32, f32, f32, f32))> {
    let mut layout: Vec<(String, (f32, f32, f32, f32))> = Vec::new();
    for id in ["SettingsPage::row-touch", "SettingsPage::row-icon", "SettingsPage::row-title", "SettingsPage::row-pill"] {
        for (index, element) in elements(app, id).iter().enumerate() {
            layout.push((format!("{id}#{index}"), box_of(element)));
        }
    }
    for element in ElementHandle::find_by_element_type_name(app, "TemplateEditor") {
        layout.push(("TemplateEditor".to_string(), box_of(&element)));
    }
    for element in ElementHandle::find_by_element_type_name(app, "SettingsPage") {
        layout.push(("SettingsPage".to_string(), box_of(&element)));
    }
    return layout;
}

/// What: `advance` moves the mock clock forward in 16 ms frames, running the timers after each, as a window does.
/// Why: a timer fires once per update however far the clock jumped, so one big jump would count as one frame.
fn advance(milliseconds: u64) {
    for _ in 0..milliseconds / 16 {
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(16));
        settle();
    }
}

/// What: `click_center` presses and releases the left pointer button at the center of an element.
fn click_center(app: &AppWindow, element: &ElementHandle) {
    let (x, y, width, height) = box_of(element);
    let position = LogicalPosition::new(x + width / 2.0, y + height / 2.0);
    app.window().dispatch_event(WindowEvent::PointerMoved { position });
    app.window().dispatch_event(WindowEvent::PointerPressed { position, button: PointerEventButton::Left });
    app.window().dispatch_event(WindowEvent::PointerReleased { position, button: PointerEventButton::Left });
    settle();
}

/// What: `press` sends one key press and release to the focused element.
fn press(app: &AppWindow, key: Key) {
    let text: SharedString = key.into();
    app.window().dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window().dispatch_event(WindowEvent::KeyReleased { text });
    settle();
}

/// What: `selected_flags` reads which rows draw as selected, as the rows report them to assistive technology.
fn selected_flags(app: &AppWindow) -> Vec<bool> {
    return elements(app, "SettingsPage::row-touch")
        .iter()
        .map(|row| return row.accessible_checked().unwrap_or(false))
        .collect();
}

/// What: `open_kind` gives the template the editor value is open on, or None when Settings holds no editor.
fn open_kind() -> Option<TemplateKind> {
    return read_state(|state| return state.editor.as_ref().map(|editor| return editor.kind()));
}

/// What: `assert_rows_follow_geometry` checks every row of a pane that starts at `pane_x`, `pane_width` wide.
/// Why: one set of assertions covers the single form and the wide form, because the rows are the same list.
fn assert_rows_follow_geometry(app: &AppWindow, pane_x: f32, pane_width: f32) {
    let touches = elements(app, "SettingsPage::row-touch");
    let icons = elements(app, "SettingsPage::row-icon");
    let titles = elements(app, "SettingsPage::row-title");
    assert_eq!(touches.len(), TemplateKind::all().len(), "one row per template");
    assert_eq!(icons.len(), touches.len());
    assert_eq!(titles.len(), touches.len());
    let first_row_y = HEADER_HEIGHT + HEADING_HEIGHT;
    for (index, touch) in touches.iter().enumerate() {
        let row_y = first_row_y + row_top(index);
        assert_eq!(box_of(touch), (pane_x + SIDE_MARGIN, row_y, row_width(pane_width), ROW_HEIGHT), "row {index}");
        let (icon_x, icon_y, icon_width, icon_height) = box_of(&icons[index]);
        assert_eq!((icon_x, icon_y), (pane_x + SIDE_MARGIN + icon_x_in_row(), row_y + icon_y_in_row()), "icon {index}");
        assert_eq!((icon_width, icon_height), (ICON_DIAMETER, ICON_DIAMETER), "icon size {index}");
        let (title_x, _, _, _) = box_of(&titles[index]);
        assert_eq!(title_x, pane_x + SIDE_MARGIN + title_x_in_row(), "title {index}");
    }
}

#[test]
fn the_single_form_rows_run_margin_to_margin_with_the_measured_icon_and_title_positions() {
    // Below 600 there is one pane of the whole width: no maximum column width and no centering.
    for width in [360.0, 443.0, 599.0] {
        let app = open_settings_at(width);
        assert_eq!(app.get_route(), Route::Settings.to_int(), "width {width}");
        assert_eq!(count_of(&app, "SettingsPage"), 1, "width {width}");
        assert_eq!(count_of(&app, "TemplateEditor"), 0, "no editor pane at width {width}");
        assert_rows_follow_geometry(&app, 0.0, list_pane_width(width));
        // The icon is 32 from the window edge and the title 84, as the stock rows measure.
        let icon = &elements(&app, "SettingsPage::row-icon")[0];
        assert_eq!(box_of(icon).0, 32.0, "width {width}");
        let title = &elements(&app, "SettingsPage::row-title")[0];
        assert_eq!(box_of(title).0, 84.0, "width {width}");
    }
}

#[test]
fn the_single_form_selects_no_row_and_a_tap_opens_that_editor_page() {
    let app = open_settings_at(443.0);
    assert_eq!(selected_flags(&app), vec![false, false], "the single form draws no selection");
    let second = elements(&app, "SettingsPage::row-touch")[1].clone();
    second.invoke_accessible_default_action();
    settle();
    assert_eq!(app.get_route(), Route::TemplateEditor.to_int());
    assert_eq!(open_kind(), Some(TemplateKind::PlayingTrack));
    assert_eq!(count_of(&app, "TemplateEditor"), 1);
    assert_eq!(count_of(&app, "SettingsPage"), 0);
    // Back from the editor page returns to the list.
    app.invoke_editor_back();
    settle();
    assert_eq!(app.get_route(), Route::Settings.to_int());
    assert_eq!(count_of(&app, "SettingsPage"), 1);
}

#[test]
fn the_wide_form_has_a_list_pane_at_the_default_ratio_and_an_editor_pane_for_the_rest() {
    for width in [600.0, 850.0, 1200.0] {
        let app = open_settings_at(width);
        let list = list_pane_width(width);
        assert_eq!(count_of(&app, "SettingsPage"), 1, "width {width}");
        let page = ElementHandle::find_by_element_type_name(&app, "SettingsPage").next().expect("the page exists");
        assert_eq!(box_of(&page), (0.0, 0.0, list, WINDOW_HEIGHT), "list pane at width {width}");
        let editor = ElementHandle::find_by_element_type_name(&app, "TemplateEditor").next().expect("the editor pane exists");
        assert_eq!(
            box_of(&editor),
            (list, 0.0, editor_pane_width(width), WINDOW_HEIGHT),
            "editor pane at width {width}"
        );
        assert_eq!(list + editor_pane_width(width), width, "no band between the panes at width {width}");
        assert_rows_follow_geometry(&app, 0.0, list);
    }
}

#[test]
fn the_wide_form_opens_with_the_first_row_selected_and_its_editor_in_the_right_pane() {
    let app = open_settings_at(850.0);
    assert_eq!(selected_flags(&app), vec![true, false]);
    assert_eq!(open_kind(), Some(TemplateKind::TrackRows));
    assert_eq!(app.get_settings_selected(), 0);
    assert_eq!(app.get_editor_title().as_str(), TemplateKind::TrackRows.label());
    assert_eq!(app.get_settings_pane_opacity(), 1.0, "opening Settings does not fade");
}

#[test]
fn selecting_by_click_changes_only_the_selection_and_shifts_nothing() {
    let app = open_settings_at(850.0);
    let before = row_layout(&app);
    let second = elements(&app, "SettingsPage::row-touch")[1].clone();
    click_center(&app, &second);
    assert_eq!(selected_flags(&app), vec![false, true]);
    assert_eq!(open_kind(), Some(TemplateKind::PlayingTrack));
    assert_eq!(app.get_route(), Route::Settings.to_int(), "selecting never navigates");
    assert_eq!(app.get_editor_title().as_str(), TemplateKind::PlayingTrack.label());
    assert_eq!(row_layout(&app), before, "no row, icon, title or pane moved");
    // And back to the first row by a second click.
    let first = elements(&app, "SettingsPage::row-touch")[0].clone();
    click_center(&app, &first);
    assert_eq!(selected_flags(&app), vec![true, false]);
    assert_eq!(row_layout(&app), before);
}

#[test]
fn selecting_by_keyboard_changes_only_the_selection_and_shifts_nothing() {
    let app = open_settings_at(850.0);
    let before = row_layout(&app);
    press(&app, Key::DownArrow);
    assert_eq!(selected_flags(&app), vec![false, true]);
    assert_eq!(open_kind(), Some(TemplateKind::PlayingTrack));
    assert_eq!(app.get_route(), Route::Settings.to_int());
    assert_eq!(row_layout(&app), before);
    // Down from the last template row reaches the Auto-play on select switch row, which selects nothing (D129).
    press(&app, Key::DownArrow);
    assert_eq!(selected_flags(&app), vec![false, true]);
    press(&app, Key::UpArrow);
    assert_eq!(selected_flags(&app), vec![false, true], "Up from the switch row lands on the already selected row");
    press(&app, Key::UpArrow);
    assert_eq!(selected_flags(&app), vec![true, false]);
    // End jumps to the Licenses row, the last stop, and leaves the selection where it was; six Ups pass the Reset text
    // sizes row and the four Text size steppers and reach the switch row, which select nothing either, and one more Up
    // returns to a template row.
    press(&app, Key::End);
    assert_eq!(selected_flags(&app), vec![true, false]);
    for _ in 0..6 {
        press(&app, Key::UpArrow);
        assert_eq!(selected_flags(&app), vec![true, false]);
    }
    press(&app, Key::UpArrow);
    assert_eq!(selected_flags(&app), vec![false, true]);
    press(&app, Key::Home);
    assert_eq!(selected_flags(&app), vec![true, false]);
    assert_eq!(row_layout(&app), before);
}

#[test]
fn the_selected_title_box_is_reserved_at_the_bold_width() {
    // The title's width comes from the bold copy, so it is the same for a selected row and for an unselected one.
    let app = open_settings_at(850.0);
    let widths_before: Vec<f32> = elements(&app, "SettingsPage::row-title").iter().map(|title| return box_of(title).2).collect();
    press(&app, Key::DownArrow);
    let widths_after: Vec<f32> = elements(&app, "SettingsPage::row-title").iter().map(|title| return box_of(title).2).collect();
    assert_eq!(widths_before, widths_after);
    assert!(widths_before.iter().all(|width| return *width > 0.0));
}

#[test]
fn a_new_selection_fades_the_right_pane_in_over_about_150_ms() {
    let app = open_settings_at(850.0);
    assert_eq!(app.get_settings_pane_opacity(), 1.0);
    press(&app, Key::DownArrow);
    assert_eq!(app.get_settings_pane_opacity(), 0.0, "the new content starts transparent");
    advance(80);
    let halfway = app.get_settings_pane_opacity();
    assert!(halfway > 0.2 && halfway < 0.9, "halfway through the fade the opacity is {halfway}");
    advance(96);
    assert_eq!(app.get_settings_pane_opacity(), 1.0, "the fade ends fully opaque after about 160 ms");
    // Choosing the already selected row restarts nothing.
    let second = elements(&app, "SettingsPage::row-touch")[1].clone();
    click_center(&app, &second);
    assert_eq!(app.get_settings_pane_opacity(), 1.0);
}

#[test]
fn enter_acts_on_the_focused_row_in_each_form() {
    let wide = open_settings_at(850.0);
    press(&wide, Key::Return);
    assert_eq!(wide.get_route(), Route::Settings.to_int(), "Enter selects in the wide form and never navigates");
    let narrow = open_settings_at(443.0);
    press(&narrow, Key::DownArrow);
    assert_eq!(narrow.get_route(), Route::Settings.to_int(), "Down only moves the focus in the single form");
    assert_eq!(open_kind(), Some(TemplateKind::TrackRows));
    press(&narrow, Key::Return);
    assert_eq!(narrow.get_route(), Route::TemplateEditor.to_int(), "Enter opens the focused row's editor page");
    assert_eq!(open_kind(), Some(TemplateKind::PlayingTrack));
}

/// What: `escape` classifies and runs an Escape key press the way the window's key scope does.
fn escape(app: &AppWindow) -> bool {
    let key_scope_used = app.invoke_key_pressed("\u{1b}".into(), false, false, false, false, false, false);
    settle();
    return key_scope_used;
}

#[test]
fn escape_closes_settings_in_both_forms() {
    for width in [443.0, 850.0] {
        let app = open_settings_at(width);
        assert_eq!(app.get_route(), Route::Settings.to_int(), "width {width}");
        assert!(escape(&app), "the key scope uses Escape at width {width}");
        assert_eq!(app.get_route(), Route::Player.to_int(), "Escape closed Settings at width {width}");
        assert_eq!(open_kind(), None, "the editor value is dropped at width {width}");
    }
}

/// What: `focus_template_field` clicks the template line of the right pane, so it holds keyboard focus like a person's click.
fn focus_template_field(app: &AppWindow) {
    let field = elements(app, "TemplateEditor::field-input").into_iter().next().expect("the template line exists");
    click_center(app, &field);
    settle();
}

#[test]
fn escape_in_the_template_line_returns_to_the_list_and_a_second_escape_closes_settings() {
    let app = open_settings_at(850.0);
    focus_template_field(&app);
    assert_eq!(app.global::<Focus>().get_zone(), 7, "the click put focus in the text field");
    assert!(escape(&app), "the first Escape is used by the page");
    assert_eq!(app.get_route(), Route::Settings.to_int(), "the first Escape does not close Settings");
    assert_eq!(open_kind(), Some(TemplateKind::TrackRows));
    assert_eq!(app.global::<Focus>().get_zone(), 8, "focus returned to the Settings list row");
    assert!(escape(&app), "the second Escape is used by the page");
    assert_eq!(app.get_route(), Route::Player.to_int(), "the second Escape closed Settings");
}

#[test]
fn escape_does_not_close_the_player_or_the_search_page_as_settings() {
    let app = window_at(850.0);
    assert!(!escape(&app), "Escape on the player is not a Settings key");
    assert_eq!(app.get_route(), Route::Player.to_int());
    update_state(|state| {
        state.route = Route::Search;
    });
    project(&app);
    assert!(escape(&app), "Escape leaves Search as before");
    assert_eq!(app.get_route(), Route::Player.to_int());
}

#[test]
fn the_header_back_arrow_closes_settings_in_both_forms() {
    for width in [443.0, 850.0] {
        let app = open_settings_at(width);
        let back = elements(&app, "SettingsPage::back-touch").into_iter().next().expect("the Back target exists");
        back.invoke_accessible_default_action();
        settle();
        assert_eq!(app.get_route(), Route::Player.to_int(), "Back closed Settings at width {width}");
        assert_eq!(open_kind(), None, "width {width}");
    }
}

#[test]
fn widening_the_window_on_an_editor_page_continues_it_in_the_right_pane() {
    let app = open_settings_at(443.0);
    elements(&app, "SettingsPage::row-touch")[1].invoke_accessible_default_action();
    settle();
    assert_eq!(app.get_route(), Route::TemplateEditor.to_int());
    app.window().set_size(slint::LogicalSize::new(850.0, WINDOW_HEIGHT));
    settle();
    assert_eq!(app.get_route(), Route::Settings.to_int(), "the editor page became the two-pane Settings");
    assert_eq!(open_kind(), Some(TemplateKind::PlayingTrack), "the same template stays selected");
    assert_eq!(selected_flags(&app), vec![false, true]);
    assert_eq!(count_of(&app, "TemplateEditor"), 1);
    // Escape then closes it, because Settings is the visible page.
    assert!(escape(&app));
    assert_eq!(app.get_route(), Route::Player.to_int());
}

#[test]
fn narrowing_the_window_on_the_two_pane_settings_shows_the_list_page() {
    let app = open_settings_at(850.0);
    press(&app, Key::DownArrow);
    app.window().set_size(slint::LogicalSize::new(443.0, WINDOW_HEIGHT));
    settle();
    assert_eq!(app.get_route(), Route::Settings.to_int());
    assert_eq!(count_of(&app, "TemplateEditor"), 0);
    assert_eq!(selected_flags(&app), vec![false, false], "the single form draws no selection");
    assert!(escape(&app));
    assert_eq!(app.get_route(), Route::Player.to_int());
}

#[test]
fn the_icons_follow_the_templates() {
    let app = open_settings_at(850.0);
    let rows = app.get_settings_rows();
    let icons: Vec<String> = (0..slint::Model::row_count(&rows))
        .map(|index| return slint::Model::row_data(&rows, index).expect("row exists").icon.to_string())
        .collect();
    assert_eq!(icons, vec!["queue_music".to_string(), "play_circle".to_string()]);
}

#[test]
fn each_template_keeps_its_own_text_when_the_one_editor_instance_is_reused_across_selections() {
    let app = open_settings_at(850.0);
    let default_playing = TemplateKind::PlayingTrack.default_template().to_string();
    // Type in the Track rows field the way the pane's text field reports an edit: whole text, then the selection.
    app.invoke_editor_text_edited("Plain track line".into(), 16, 16);
    settle();
    assert_eq!(read_state(|state| return state.editor.as_ref().map(|editor| return editor.text().to_string())), Some("Plain track line".to_string()));
    // Choosing the other template applies and saves the first, and shows the other one's own text.
    press(&app, Key::DownArrow);
    assert_eq!(open_kind(), Some(TemplateKind::PlayingTrack));
    assert_eq!(app.get_editor_text().as_str(), default_playing);
    assert_eq!(read_state(|state| return state.track_rows_template.clone()), Some("Plain track line".to_string()));
    assert_eq!(read_state(|state| return state.playing_template.clone()), None, "the other template was not written");
    assert_eq!(saved_texts(), vec![(Some("Plain track line".to_string()), None)]);
    // Coming back shows the applied edit, and the field kept in step with it.
    press(&app, Key::UpArrow);
    assert_eq!(open_kind(), Some(TemplateKind::TrackRows));
    assert_eq!(app.get_editor_text().as_str(), "Plain track line");
    // The rows of the list show the applied lines.
    assert_eq!(slint::Model::row_data(&app.get_settings_rows(), 0).expect("row").line.as_str(), "Plain track line");
}

#[test]
fn leaving_settings_without_an_edit_saves_nothing() {
    let app = open_settings_at(850.0);
    press(&app, Key::DownArrow);
    press(&app, Key::UpArrow);
    assert!(escape(&app));
    assert_eq!(saved_texts(), Vec::<(Option<String>, Option<String>)>::new(), "unchanged templates are not written");
}

#[test]
fn closing_settings_saves_an_edit_made_in_the_right_pane() {
    let app = open_settings_at(850.0);
    app.invoke_editor_text_edited("Plain track line".into(), 16, 16);
    settle();
    app.invoke_settings_back();
    settle();
    assert_eq!(app.get_route(), Route::Player.to_int());
    assert_eq!(saved_texts(), vec![(Some("Plain track line".to_string()), None)]);
}

#[test]
fn narrowing_the_window_clears_a_focus_left_in_the_destroyed_text_field() {
    let app = open_settings_at(850.0);
    focus_template_field(&app);
    assert_eq!(app.global::<Focus>().get_zone(), 7);
    app.window().set_size(slint::LogicalSize::new(443.0, WINDOW_HEIGHT));
    settle();
    assert_ne!(app.global::<Focus>().get_zone(), 7, "the destroyed field no longer holds the zone");
    assert!(escape(&app), "Escape on the list page closes Settings once the field is gone");
    assert_eq!(app.get_route(), Route::Player.to_int());
}

#[test]
fn the_icon_paths_are_explicit_absolute_commands_that_slint_draws_whole() {
    // The embedded data holds only absolute M, L, Q and Z with every number separated by a space, on the 24 grid, and no
    // closing line back to a contour's start before its Z (femtovg draws a rectangle that has one as a wrong shape).
    let source = include_str!("../ui/settings.slint");
    for name in ["queue-music-path", "play-circle-path"] {
        let prefix = format!("property <string> {name}: \"");
        let start = source.find(&prefix).unwrap_or_else(|| panic!("{name} is missing")) + prefix.len();
        let data = &source[start..start + source[start..].find('"').expect("the string ends")];
        assert!(data.starts_with("M "), "{name} starts with an absolute move");
        for token in data.split(' ') {
            let is_command = matches!(token, "M" | "L" | "Q" | "Z");
            assert!(is_command || token.parse::<f64>().is_ok(), "{name} holds the unexpected token {token}");
        }
        assert!(data.contains(" Q "), "{name} keeps its curves");
        let tokens: Vec<&str> = data.split(' ').collect();
        let mut contour_start: Option<(&str, &str)> = None;
        for (index, token) in tokens.iter().enumerate() {
            match *token {
                "M" => contour_start = Some((tokens[index + 1], tokens[index + 2])),
                "Z" => {
                    // The last line of a contour must not return to the contour's start, because Z already does.
                    let ends_with_line = index >= 3 && tokens[index - 3] == "L";
                    let returns = ends_with_line && contour_start == Some((tokens[index - 2], tokens[index - 1]));
                    assert!(!returns, "{name} has a redundant closing line before a Z");
                }
                _ => {}
            }
        }
    }
}

#[test]
fn the_selection_pill_radius_is_the_layout_rule() {
    let source = include_str!("../ui/settings.slint");
    let line = source.lines().find(|line| return line.contains("property <length> pill-radius:")).expect("the pill radius is declared");
    let value: f32 = line.split(':').nth(1).and_then(|rest| return rest.trim().trim_end_matches(';').trim_end_matches("px").parse().ok()).expect("a pixel value");
    assert_eq!(value, PILL_RADIUS, "settings.slint and settings_layout agree");
    assert_eq!(PILL_RADIUS, 21.0, "21 px, inside the stock 21 to 22 dp");
}

#[test]
fn the_icon_circle_uses_the_secondary_pair_and_the_editor_header_is_the_canvas_color() {
    let settings = include_str!("../ui/settings.slint");
    let from_circle = &settings[settings.find("row-icon := Rectangle").expect("the icon circle exists")..];
    let circle = &from_circle[..from_circle.find("title-measure").expect("the title follows the circle")];
    assert!(circle.contains("background: Theme.secondary;"), "the circle fill is the secondary role");
    assert!(circle.contains("fill: Theme.on-secondary;"), "the glyph is the on-secondary role");
    let editor = include_str!("../ui/template_editor.slint");
    // D109: the page header has no band of its own, so the page's canvas color shows through, and no divider under it.
    let from_band = &editor[editor.find("Page header with the Back target").expect("the page header exists")..];
    let band = &from_band[..from_band.find("Reset to default, at the right end of the header").expect("the Reset button follows the header")];
    assert!(!band.contains("background:"), "the header paints no band, so the canvas shows");
    assert!(!band.contains("surface-high"), "no surface-high band");
    assert!(editor.contains("background: Theme.canvas;"), "the page itself is the canvas color");
    assert!(!editor.contains("Divider under the header"), "the editor page draws no divider under its header");
    assert!(editor.contains("background: Theme.outline-variant;"), "the preview hairlines stay");
    assert!(editor.contains("border-color: root.mistake-lines.length > 0"), "the field outline stays");
}

#[test]
fn the_secondary_pair_follows_the_primary_pair_in_both_schemes() {
    let app = window_at(443.0);
    for scheme in [1, 2] {
        app.global::<crate::Theme>().set_scheme_override(scheme);
        let theme = app.global::<crate::Theme>();
        assert_eq!(theme.get_secondary(), theme.get_primary(), "scheme {scheme}");
        assert_eq!(theme.get_on_secondary(), theme.get_on_primary(), "scheme {scheme}");
    }
}
