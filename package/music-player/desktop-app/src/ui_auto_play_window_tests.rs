//! Window tests of Auto-play on select (D129): what a click, Enter and the menu Play do with the setting on and off, and
//! the Settings switch row (default, click, Space, Enter, the state it shows, the two-pane form, the ring).
//!
//! What: each test builds the real window on the testing backend or the software renderer, sends real pointer and key
//! events, and reads what reached the recording host, what the switch row says to assistive technology, where its parts
//! sit, and what it draws. The Search result cases are in `ui_search_activation_tests.rs`.
//! Why: the rule is one pure function, but the wiring from the window to it (the click, Enter, the switch, the saved
//! value) only exists in the composed window.
//! Note: element lookup by id needs `SLINT_EMIT_DEBUG_INFO=1` at build time, which the repo test task sets.

use std::cell::RefCell;
use std::rc::Rc;

use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use music_player::row_activation::AUTO_PLAY_ON_SELECT_TITLE;
use music_player::settings_layout::{
    HEADER_HEIGHT, ICON_DIAMETER, ICON_X_FROM_EDGE, ROW_GAP, ROW_HEIGHT, SIDE_MARGIN, TITLE_X_FROM_EDGE,
};
use music_player::template_model::TemplateKind;
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition};

use crate::AppWindow;
use crate::ui_focus_ring_tests::{Case, case, check_focused, roles_of};
use crate::ui_render_support::{Frame, Rendered};
use crate::ui_screen::{project, read_state, update_state};
use crate::ui_screen_view::Route;
use crate::ui_settings::{close_settings, open_settings, replace_auto_play_saver};
use crate::ui_test_support::{RecordingHost, chord, fresh_window_at, load_library, settle, tap};

/// Height of the section heading above a group: 16 above the label, the label, and 8 below.
const HEADING_HEIGHT: f32 = 44.0;

thread_local! {
    /// The values each save of the setting received on this test thread, so a test sees what would reach the session file.
    static SAVED: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
}

/// What: `record_save` stands in for the session file: it keeps the value it is given and writes nothing.
fn record_save(enabled: bool) -> std::io::Result<()> {
    SAVED.with(|saved| saved.borrow_mut().push(enabled));
    return Ok(());
}

/// What: `saved_values` lists the saves of this test so far.
fn saved_values() -> Vec<bool> {
    return SAVED.with(|saved| return saved.borrow().clone());
}

/// What: `window_at` builds a window with the library loaded and the setting saver replaced by the recording one.
/// Why: a click on the switch must never write the user's session file, and the test reads what would have been written.
fn window_at(width: f32, height: f32) -> (AppWindow, Rc<RecordingHost>) {
    let (app, host) = fresh_window_at(width, height);
    SAVED.with(|saved| saved.borrow_mut().clear());
    replace_auto_play_saver(record_save);
    load_library(&app);
    return (app, host);
}

/// What: `with_setting` turns Auto-play on select on or off in the screen state, as loading a saved session does.
fn with_setting(app: &AppWindow, enabled: bool) {
    update_state(|state| {
        state.auto_play_on_select = enabled;
    });
    project(app);
    settle();
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

/// What: `click_center` presses and releases the left pointer button at the center of an element.
fn click_center(app: &AppWindow, element: &ElementHandle) {
    let (x, y, width, height) = box_of(element);
    let position = LogicalPosition::new(x + width / 2.0, y + height / 2.0);
    app.window().dispatch_event(WindowEvent::PointerMoved { position });
    app.window().dispatch_event(WindowEvent::PointerPressed { position, button: PointerEventButton::Left });
    app.window().dispatch_event(WindowEvent::PointerReleased { position, button: PointerEventButton::Left });
    settle();
}

/// What: `track_row` finds the right pane's track row whose accessible checked state is `current`.
/// Why: the window builds both layouts, and the one on screen at 1200 wide is the right pane.
fn track_row(app: &AppWindow, current: bool) -> ElementHandle {
    return elements(app, "TrackListPane::row-touch")
        .into_iter()
        .filter(|row| return box_of(row).0 >= 600.0)
        .find(|row| return row.accessible_checked() == Some(current))
        .expect("a track row in the wanted state");
}

/// What: `switch_touch` is the touch area of the switch row, which carries its accessibility.
fn switch_touch(app: &AppWindow) -> ElementHandle {
    return only(app, "SettingsPage::switch-touch");
}

/// What: `open_settings_at` builds a window of the given width and opens Settings on it.
fn open_settings_at(width: f32) -> AppWindow {
    let (app, _host) = window_at(width, 900.0);
    open_settings(&app);
    settle();
    return app;
}

//region The rule through the row click, Enter and the menu

#[test]
fn with_the_setting_on_a_click_on_an_unselected_row_selects_it_and_starts_it() {
    let (app, host) = window_at(1200.0, 800.0);
    with_setting(&app, true);
    let target = track_row(&app, false);
    host.clear();
    click_center(&app, &target);
    let sent = host.sent_texts();
    assert_eq!(sent.len(), 2, "a select and a play: {sent:?}");
    assert!(sent[0].starts_with("select "), "the click selects the row: {sent:?}");
    assert_eq!(sent[1], "play", "and starts it at once");
}

#[test]
fn with_the_setting_off_the_same_click_only_selects() {
    let (app, host) = window_at(1200.0, 800.0);
    with_setting(&app, false);
    let target = track_row(&app, false);
    host.clear();
    click_center(&app, &target);
    let sent = host.sent_texts();
    assert_eq!(sent.len(), 1, "no play after the select: {sent:?}");
    assert!(sent[0].starts_with("select "), "{sent:?}");
}

#[test]
fn with_the_setting_on_a_click_on_the_selected_row_still_toggles() {
    let (app, host) = window_at(1200.0, 800.0);
    with_setting(&app, true);
    let current = track_row(&app, true);
    host.clear();
    click_center(&app, &current);
    assert_eq!(host.sent_texts(), ["toggle"]);
}

#[test]
fn with_the_setting_on_the_rows_default_action_selects_and_starts_it() {
    let (app, host) = window_at(1200.0, 800.0);
    with_setting(&app, true);
    let target = track_row(&app, false);
    host.clear();
    target.invoke_accessible_default_action();
    settle();
    let sent = host.sent_texts();
    assert_eq!(sent.len(), 2, "{sent:?}");
    assert!(sent[0].starts_with("select "), "{sent:?}");
    assert_eq!(sent[1], "play");
}

#[test]
fn with_the_setting_on_enter_on_another_row_selects_and_starts_it_and_on_the_selected_row_it_toggles() {
    let (app, host) = window_at(850.0, 900.0);
    with_setting(&app, true);
    chord(&app, &["Alt"], "2");
    tap(&app, "Down");
    tap(&app, "Enter");
    assert_eq!(host.sent_texts(), ["select 2", "play"], "Enter on another row selects it and starts it");
    host.clear();
    tap(&app, "Up");
    tap(&app, "Enter");
    assert_eq!(host.sent_texts(), ["toggle"], "row 1 is the selected row");
}

#[test]
fn the_menu_play_item_starts_the_row_with_the_setting_on_or_off_and_the_selected_row_toggles() {
    for enabled in [false, true] {
        let (app, host) = window_at(1200.0, 800.0);
        with_setting(&app, enabled);
        host.clear();
        crate::ui_wire::play_row_now(host.as_ref(), 0);
        assert_eq!(host.sent_texts(), ["select 0", "play"], "the explicit Play, setting {enabled}");
        host.clear();
        crate::ui_wire::play_row_now(host.as_ref(), 1);
        assert_eq!(host.sent_texts(), ["toggle"], "row 1 is the selected row, setting {enabled}");
    }
}

#[test]
fn a_folder_choice_sends_only_a_scope_with_the_setting_on_or_off() {
    for enabled in [false, true] {
        let (app, host) = window_at(1200.0, 800.0);
        with_setting(&app, enabled);
        host.clear();
        crate::ui_wire::select_folder(host.as_ref(), "Beck");
        assert_eq!(host.sent_texts(), ["scope 2"], "setting {enabled}");
    }
}

//endregion

//region The Settings switch row

#[test]
fn the_switch_row_is_a_named_off_switch_with_its_line_by_default() {
    let app = open_settings_at(443.0);
    let touch = switch_touch(&app);
    assert_eq!(touch.accessible_role(), Some(AccessibleRole::Switch));
    assert_eq!(touch.accessible_label().as_deref(), Some(AUTO_PLAY_ON_SELECT_TITLE));
    assert_eq!(touch.accessible_checkable(), Some(true));
    assert_eq!(touch.accessible_checked(), Some(false), "off when nothing was saved");
    assert_eq!(touch.accessible_description().as_deref(), Some("Select first, then Play"));
    assert!(!app.get_auto_play_on_select());
}

#[test]
fn the_switch_row_follows_the_stock_sizes_in_the_single_form() {
    let app = open_settings_at(443.0);
    let (x, y, width, height) = box_of(&only(&app, "SettingsPage::switch-row"));
    // The header, the Templates heading, two rows with the gap between them, then the Playback heading.
    let expected_top = HEADER_HEIGHT + HEADING_HEIGHT + 2.0 * ROW_HEIGHT + ROW_GAP + HEADING_HEIGHT;
    assert_eq!((x, width), (SIDE_MARGIN, 443.0 - 2.0 * SIDE_MARGIN), "margin to margin like the template rows");
    assert_eq!(y, expected_top, "under its own heading, 16 px below the last template row");
    assert_eq!(height, ROW_HEIGHT, "the 72 px row");
    let (icon_x, icon_y, icon_width, icon_height) = box_of(&only(&app, "SettingsPage::switch-icon"));
    assert_eq!((icon_width, icon_height), (ICON_DIAMETER, ICON_DIAMETER), "the 40 px circle");
    assert_eq!(icon_x, ICON_X_FROM_EDGE, "32 px from the window edge");
    assert_eq!(icon_y, y + (ROW_HEIGHT - ICON_DIAMETER) / 2.0, "centered in the row");
    let (title_x, _title_y, _title_width, _title_height) = box_of(&only(&app, "SettingsPage::switch-title"));
    assert_eq!(title_x, TITLE_X_FROM_EDGE, "the title starts at 84 px like the template rows");
    let (track_x, track_y, track_width, track_height) = box_of(&only(&app, "AutoPlaySwitch::track"));
    assert_eq!((track_width, track_height), (52.0, 32.0), "the Material 3 switch");
    assert_eq!(track_x + track_width, x + width - 16.0, "16 px from the row's end");
    assert_eq!(track_y, y + (ROW_HEIGHT - 32.0) / 2.0, "centered in the row");
}

#[test]
fn a_click_toggles_the_switch_in_place_and_saves_each_value() {
    let app = open_settings_at(443.0);
    click_center(&app, &switch_touch(&app));
    assert_eq!(switch_touch(&app).accessible_checked(), Some(true));
    assert!(read_state(|state| return state.auto_play_on_select));
    assert_eq!(read_state(|state| return state.route), Route::Settings, "the page stays Settings");
    assert_eq!(ElementHandle::find_by_element_type_name(&app, "TemplateEditor").count(), 0, "no editor page opened");
    assert_eq!(
        switch_touch(&app).accessible_description().as_deref(),
        Some("Select and play at once"),
        "the line follows the state"
    );
    click_center(&app, &switch_touch(&app));
    assert_eq!(switch_touch(&app).accessible_checked(), Some(false));
    assert!(!read_state(|state| return state.auto_play_on_select));
    assert_eq!(saved_values(), [true, false]);
}

#[test]
fn the_accessible_default_action_toggles_the_switch() {
    let app = open_settings_at(443.0);
    switch_touch(&app).invoke_accessible_default_action();
    settle();
    assert_eq!(switch_touch(&app).accessible_checked(), Some(true));
    assert_eq!(saved_values(), [true]);
}

#[test]
fn the_arrow_keys_reach_the_switch_row_and_space_and_enter_toggle_it() {
    let app = open_settings_at(443.0);
    tap(&app, "Down");
    tap(&app, "Down");
    assert_eq!(switch_touch(&app).accessible_checked(), Some(false), "moving the ring toggles nothing");
    tap(&app, " ");
    assert_eq!(switch_touch(&app).accessible_checked(), Some(true), "Space toggles");
    tap(&app, "Enter");
    assert_eq!(switch_touch(&app).accessible_checked(), Some(false), "Enter toggles");
    assert_eq!(saved_values(), [true, false]);
    assert_eq!(read_state(|state| return state.route), Route::Settings, "Enter on the switch opens no editor");
    // Down moves through the four Text size steppers and the Reset text sizes row to the Licenses row, the last stop:
    // Space there opens its page and toggles nothing.
    for _ in 0..6 {
        tap(&app, "Down");
    }
    tap(&app, " ");
    assert_eq!(saved_values(), [true, false], "Space on the Licenses row toggles nothing");
    assert_eq!(read_state(|state| return state.route), Route::Licenses, "Space on the Licenses row opens its page");
    assert!(app.invoke_key_pressed("\u{1b}".into(), false, false, false, false, false, false), "Escape is used on the page");
    settle();
    assert_eq!(read_state(|state| return state.route), Route::Settings, "Escape returns to Settings");
    tap(&app, "Home");
    tap(&app, "Enter");
    assert_eq!(read_state(|state| return state.route), Route::TemplateEditor, "Home then Enter opens the first template");
}

#[test]
fn space_on_the_switch_row_sends_no_engine_command() {
    let (app, host) = window_at(443.0, 900.0);
    open_settings(&app);
    settle();
    host.clear();
    // End reaches the Licenses row, the last stop; six Ups pass the Reset text sizes row and the four Text size steppers
    // and reach the switch row above them.
    tap(&app, "End");
    for _ in 0..6 {
        tap(&app, "Up");
    }
    tap(&app, " ");
    assert_eq!(switch_touch(&app).accessible_checked(), Some(true));
    assert!(host.sent_texts().is_empty(), "no engine command: {:?}", host.sent_texts());
}

#[test]
fn the_new_value_governs_the_next_row_click_after_leaving_settings() {
    let (app, host) = window_at(443.0, 900.0);
    open_settings(&app);
    settle();
    click_center(&app, &switch_touch(&app));
    close_settings(&app);
    settle();
    assert_eq!(read_state(|state| return state.route), Route::Player);
    let target = elements(&app, "TrackListPane::row-touch")
        .into_iter()
        .find(|row| return row.accessible_checked() == Some(false))
        .expect("an unselected track row");
    host.clear();
    click_center(&app, &target);
    let sent = host.sent_texts();
    assert_eq!(sent.len(), 2, "{sent:?}");
    assert_eq!(sent[1], "play");
}

#[test]
fn in_the_two_pane_form_the_switch_changes_no_selection_and_the_editor_stays() {
    let app = open_settings_at(1200.0);
    let open_kind = || return read_state(|state| return state.editor.as_ref().map(|editor| return editor.kind()));
    assert_eq!(open_kind(), Some(TemplateKind::TrackRows));
    assert_eq!(ElementHandle::find_by_element_type_name(&app, "TemplateEditor").count(), 1, "the detail pane exists");
    click_center(&app, &switch_touch(&app));
    assert_eq!(switch_touch(&app).accessible_checked(), Some(true));
    assert_eq!(app.get_settings_selected(), 0, "the first template stays selected");
    assert_eq!(open_kind(), Some(TemplateKind::TrackRows), "and its editor stays in the detail pane");
    assert_eq!(read_state(|state| return state.route), Route::Settings);
    // The click put the keyboard position on the switch row, so Up reaches the second template row.
    tap(&app, "Up");
    assert_eq!(app.get_settings_selected(), 1, "Up onto the second template row selects it");
    assert_eq!(open_kind(), Some(TemplateKind::PlayingTrack));
    tap(&app, "Down");
    assert_eq!(app.get_settings_selected(), 1, "Down onto the switch row selects nothing");
    assert_eq!(open_kind(), Some(TemplateKind::PlayingTrack), "the detail pane keeps the second template");
    tap(&app, " ");
    assert_eq!(switch_touch(&app).accessible_checked(), Some(false), "Space on the switch row toggled it back");
    assert_eq!(app.get_settings_selected(), 1, "and the second template is still the selected one");
    assert_eq!(saved_values(), [true, false]);
    let flags: Vec<Option<bool>> =
        elements(&app, "SettingsPage::row-touch").iter().map(|row| return row.accessible_checked()).collect();
    assert_eq!(flags, [Some(false), Some(true)], "only the template rows carry a selected state");
}

#[test]
fn the_switch_row_is_not_one_of_the_template_rows() {
    let app = open_settings_at(1200.0);
    assert_eq!(elements(&app, "SettingsPage::row-touch").len(), TemplateKind::all().len());
    assert_ne!(switch_touch(&app).accessible_role(), Some(AccessibleRole::Button));
}

#[test]
fn a_narrow_list_pane_puts_the_switch_under_the_text_and_a_wide_one_after_it() {
    // List panes: 218 and 309 wide are narrower than 300 px of row; the single form at 443 and the pane at 1200 are not.
    for (width, stacked) in [(600.0, true), (850.0, true), (443.0, false), (1200.0, false)] {
        let app = open_settings_at(width);
        let row = box_of(&only(&app, "SettingsPage::switch-row"));
        assert!(row.3 >= ROW_HEIGHT, "never shorter than 72 px at {width}, found {}", row.3);
        let track = box_of(&only(&app, "AutoPlaySwitch::track"));
        let title = box_of(&only(&app, "SettingsPage::switch-title"));
        let line = box_of(&only(&app, "SettingsPage::switch-line"));
        assert!(track.1 >= row.1 && track.1 + track.3 <= row.1 + row.3, "the switch is inside the row at {width}: {track:?}");
        for text in [title, line] {
            assert!(text.1 >= row.1 && text.1 + text.3 <= row.1 + row.3, "the text is inside the row at {width}: {text:?}");
            assert!(text.0 >= row.0 && text.0 + text.2 <= row.0 + row.2, "the text is inside the row at {width}: {text:?}");
        }
        if stacked {
            assert!(track.1 >= line.1 + line.3, "the switch is under the line at {width}: {track:?} and {line:?}");
            assert_eq!(track.0, title.0, "and starts where the text starts at {width}");
        } else {
            assert!(title.0 + title.2 <= track.0 && line.0 + line.2 <= track.0, "the text ends before the switch at {width}");
            assert_eq!(row.3, ROW_HEIGHT, "one line each keeps the 72 px row at {width}");
        }
    }
}

//endregion

//region What the switch draws

/// What: `settings_window` builds a software-rendered window of the given width with Settings open.
fn settings_window(width: usize) -> Rendered {
    let rendered = Rendered::new(width, 900);
    chord(&rendered.app, &["Ctrl", "Alt"], "s");
    rendered.wait(300);
    assert_eq!(rendered.app.get_route(), 2, "Ctrl+Alt+S opened Settings");
    return rendered;
}

/// What: `color_of` reads a theme color as `[red, green, blue]`.
fn color_of(color: slint::Color) -> [u8; 3] {
    return [color.red(), color.green(), color.blue()];
}

/// What: `count_pixels` counts the pixels of a window box that have exactly one color.
fn count_pixels(frame: &Frame, area: (f32, f32, f32, f32), color: [u8; 3]) -> usize {
    let (left, top) = (area.0.round() as usize, area.1.round() as usize);
    let (width, height) = (area.2.round() as usize, area.3.round() as usize);
    let mut count = 0;
    for y in top..top + height {
        for x in left..left + width {
            if frame.at(x, y) == color {
                count += 1;
            }
        }
    }
    return count;
}

#[test]
fn the_switch_shows_its_state_by_thumb_position_track_fill_and_mark() {
    let rendered = settings_window(443);
    let (primary, on_primary) = roles_of(&rendered);
    let theme = rendered.app.global::<crate::Theme>();
    let (canvas, outline) = (color_of(theme.get_canvas()), color_of(theme.get_outline()));
    let track = box_of(&only(&rendered.app, "AutoPlaySwitch::track"));
    let middle_y = (track.1 + track.3 / 2.0).round() as usize;
    let off_thumb = box_of(&only(&rendered.app, "AutoPlaySwitch::thumb"));
    let off_frame = rendered.frame();
    assert_eq!(off_thumb.0 - track.0, 4.0, "off: the thumb is at the left end");
    assert_eq!(off_frame.at((track.0 + track.2 - 8.0) as usize, middle_y), canvas, "off: the track is the canvas");
    assert_eq!(off_frame.at((off_thumb.0 + 2.0) as usize, middle_y), outline, "off: the thumb is the outline color");
    assert!(count_pixels(&off_frame, off_thumb, canvas) > 20, "off: the cross is drawn in the thumb");

    click_center(&rendered.app, &switch_touch(&rendered.app));
    rendered.wait(300);
    let on_thumb = box_of(&only(&rendered.app, "AutoPlaySwitch::thumb"));
    let on_frame = rendered.frame();
    assert_eq!(on_thumb.0 - track.0, 24.0, "on: the thumb is at the right end");
    assert_eq!(on_frame.at((track.0 + 8.0) as usize, middle_y), primary, "on: the track is the accent");
    assert_eq!(on_frame.at((on_thumb.0 + 2.0) as usize, middle_y), on_primary, "on: the thumb is the on-accent color");
    assert!(count_pixels(&on_frame, on_thumb, primary) > 20, "on: the check is drawn in the thumb");
}

#[test]
fn in_the_dark_scheme_the_off_track_is_pure_black_with_an_outline_border() {
    let rendered = settings_window(443);
    rendered.set_dark(true);
    rendered.wait(300);
    let theme = rendered.app.global::<crate::Theme>();
    let outline = color_of(theme.get_outline());
    let track = box_of(&only(&rendered.app, "AutoPlaySwitch::track"));
    let middle_y = (track.1 + track.3 / 2.0).round() as usize;
    let frame = rendered.frame();
    assert_eq!(frame.at((track.0 + track.2 - 8.0) as usize, middle_y), [0, 0, 0], "off: the track is pure black");
    assert_eq!(frame.at((track.0 + 1.0) as usize, middle_y), outline, "the border is the outline color");
    assert_eq!(frame.at(4, middle_y), [0, 0, 0], "the page is pure black");
}

#[test]
fn the_keyboard_ring_hugs_the_switch_row_with_the_group_corners() {
    let rendered = settings_window(443);
    tap(&rendered.app, "Down");
    tap(&rendered.app, "Down");
    let row = Case { radii: [14.0; 4], ..case("settings-list", "SettingsPage::switch-row", (0.0, 0.0), 0.0) };
    check_focused(&rendered, &row);
}

//endregion
