//! Window tests of the Text size setting: the Settings steppers, the Reset row, the live window, the saves, and the layout
//! of the five texts at both ends of the range in the two-pane form and in the single form.
//!
//! What: each test builds the real window on the testing backend, drives the stepper buttons through their accessible
//! default action (what a pointer press or a screen reader does) or through real key events, and reads the sizes in the
//! screen state, the `TextSize` global, the recorded saves, and the boxes of the elements that draw the texts.
//! Why: the rules are pure (`text_size.rs`), but the wiring from a button to the saved file and to the five texts, and the
//! room each text needs at 32 px, only exist in the composed window.
//! Note: element lookup by id needs `SLINT_EMIT_DEBUG_INFO=1` at build time, which the repo test task sets.

use std::cell::RefCell;

use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use music_player::text_size::{MAX_TEXT_SIZE_PX, MIN_TEXT_SIZE_PX, TextElement, TextSizes};
use music_player::track_measure::TrackMeasure;
use slint::platform::{Key, WindowEvent};
use slint::{ComponentHandle, SharedString};

use crate::ui_screen::{project, read_state, update_state};
use crate::ui_settings::open_settings;
use crate::ui_test_support::{advance, fresh_window_at, load_library, settle};
use crate::{AppWindow, TextSize};

/// Height of the window in every test, tall enough for the whole Settings list and the track list.
const WINDOW_HEIGHT: f32 = 1400.0;

thread_local! {
    /// The sizes each save call received on this test thread, so a test sees what would have reached the session file.
    static SAVED: RefCell<Vec<TextSizes>> = const { RefCell::new(Vec::new()) };
}

/// What: `record_save` stands in for the session file: it keeps the sizes it is given and writes nothing.
fn record_save(sizes: TextSizes) -> std::io::Result<()> {
    SAVED.with(|saved| saved.borrow_mut().push(sizes));
    return Ok(());
}

/// What: `saves` lists the saves of this test so far.
fn saves() -> Vec<TextSizes> {
    return SAVED.with(|saved| return saved.borrow().clone());
}

/// What: `sizes` reads the sizes in the screen state.
fn sizes() -> TextSizes {
    return read_state(|state| return state.text_sizes);
}

/// What: `window_at` builds the window at `width` with the recording saver, the library loaded and every size at its default.
/// Why: a press must never write the user's session file, and the track list needs rows to measure.
fn window_at(width: f32) -> AppWindow {
    let (app, _host) = fresh_window_at(width, WINDOW_HEIGHT);
    SAVED.with(|saved| saved.borrow_mut().clear());
    crate::ui_text_size::replace_saver(record_save);
    load_library(&app);
    return app;
}

/// What: `settings_at` builds the window at `width` and opens Settings on it.
fn settings_at(width: f32) -> AppWindow {
    let app = window_at(width);
    open_settings(&app);
    settle();
    return app;
}

/// What: `show_sizes` puts `next` into the state and the window, as loading a saved session does.
fn show_sizes(app: &AppWindow, next: TextSizes) {
    update_state(|state| {
        state.text_sizes = next;
    });
    project(app);
    advance(64);
}

/// What: `all_at` gives sizes with every element at `px`.
fn all_at(px: u16) -> TextSizes {
    return TextElement::ALL.iter().fold(TextSizes::default(), |next, element| return next.with(*element, px));
}

/// What: `elements` lists the elements with a qualified id, in tree order.
fn elements(app: &AppWindow, id: &str) -> Vec<ElementHandle> {
    return ElementHandle::find_by_element_id(app, id).collect();
}

/// What: `box_of` gives an element's window position and size as `(x, y, width, height)`.
fn box_of(element: &ElementHandle) -> (f32, f32, f32, f32) {
    let position = element.absolute_position();
    let size = element.size();
    return (position.x, position.y, size.width, size.height);
}

/// What: `labelled` finds the one button, or other element that is not plain text, with an accessible label.
/// Why: a row's title Text carries the same words as its button, and only the button is pressed.
fn labelled(app: &AppWindow, label: &str) -> ElementHandle {
    let mut found: Vec<ElementHandle> = ElementHandle::find_by_accessible_label(app, label)
        .filter(|element| return element.accessible_role() != Some(AccessibleRole::Text))
        .collect();
    assert_eq!(found.len(), 1, "exactly one element is labelled {label:?}, found {}", found.len());
    return found.remove(0);
}

/// What: `press_button` presses the stepper button with the label `label` the way a pointer press or a screen reader does.
fn press_button(app: &AppWindow, label: &str) {
    labelled(app, label).invoke_accessible_default_action();
    settle();
}

/// What: `press` sends one key press and release to the focused element.
fn press(app: &AppWindow, key: Key) {
    let text: SharedString = key.into();
    app.window().dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window().dispatch_event(WindowEvent::KeyReleased { text });
    settle();
}

/// What: `press_text` sends one key press and release that carries `text`, as a character key does.
fn press_text(app: &AppWindow, typed: &str) {
    let text: SharedString = typed.into();
    app.window().dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window().dispatch_event(WindowEvent::KeyReleased { text });
    settle();
}

/// What: `global_sizes` reads the four sizes the `TextSize` global holds, in pixels.
fn global_sizes(app: &AppWindow) -> [f32; 4] {
    let global = app.global::<TextSize>();
    return [global.get_pane_title(), global.get_group_heading(), global.get_folder_name(), global.get_track_title()];
}

/// The Slint defaults are the sizes the player drew before the setting existed, and they equal the Rust defaults.
#[test]
fn the_markup_defaults_equal_the_element_defaults_and_the_limits() {
    let (_first, _host) = fresh_window_at(800.0, 600.0);
    // A second window, never projected: it shows what the markup holds when no Rust has written the global.
    let bare = AppWindow::new().expect("a second window builds");
    let expected = TextElement::ALL.map(|element| return f32::from(element.default_px()));
    assert_eq!(global_sizes(&bare), expected);
    assert_eq!(global_sizes(&bare), [20.0, 20.0, 14.0, 14.0]);
    assert_eq!(bare.global::<TextSize>().get_min_size(), f32::from(MIN_TEXT_SIZE_PX));
    assert_eq!(bare.global::<TextSize>().get_max_size(), f32::from(MAX_TEXT_SIZE_PX));
}

/// The Settings rows are named like the elements, in the order of the steppers.
#[test]
fn the_rows_carry_the_names_of_the_elements() {
    let app = settings_at(900.0);
    for element in TextElement::ALL {
        labelled(&app, &format!("Smaller {}", element.label()));
        labelled(&app, &format!("Larger {}", element.label()));
    }
}

/// Nothing changes until a stepper is pressed: the window draws the old sizes and a press saves nothing by itself.
#[test]
fn the_window_starts_at_the_default_sizes_and_saves_nothing() {
    let app = settings_at(900.0);
    assert_eq!(global_sizes(&app), [20.0, 20.0, 14.0, 14.0]);
    assert!(app.get_text_sizes_default());
    assert!(saves().is_empty());
}

/// A plus press moves one element by one pixel, shows it in the live window at once, and saves it.
#[test]
fn a_plus_press_steps_one_pixel_updates_the_window_and_saves() {
    let app = settings_at(900.0);
    press_button(&app, "Larger Track titles");
    assert_eq!(sizes().get(TextElement::TrackTitle), 15);
    assert_eq!(global_sizes(&app), [20.0, 20.0, 14.0, 15.0], "only the track titles moved");
    assert!(!app.get_text_sizes_default());
    assert_eq!(saves(), vec![TextSizes::default().with(TextElement::TrackTitle, 15)]);
    press_button(&app, "Smaller Folder names");
    assert_eq!(global_sizes(&app), [20.0, 20.0, 13.0, 15.0]);
    assert_eq!(saves().len(), 2);
}

/// Each of the four steppers moves its own element and no other.
#[test]
fn each_stepper_moves_only_its_own_element() {
    for (index, element) in TextElement::ALL.iter().enumerate() {
        let app = settings_at(900.0);
        press_button(&app, &format!("Larger {}", element.label()));
        let mut expected = [20.0, 20.0, 14.0, 14.0];
        expected[index] += 1.0;
        assert_eq!(global_sizes(&app), expected, "{element:?}");
    }
}

/// The size shows with its unit between the buttons, and follows each press.
#[test]
fn the_value_shows_in_pixels_between_the_buttons() {
    let app = settings_at(900.0);
    let count_of_label = |label: &str| -> usize { return ElementHandle::find_by_accessible_label(&app, label).count() };
    assert_eq!(count_of_label("Track titles 14 pixels"), 1);
    press_button(&app, "Larger Track titles");
    assert_eq!(count_of_label("Track titles 14 pixels"), 0);
    assert_eq!(count_of_label("Track titles 15 pixels"), 1);
    let shown: Vec<ElementHandle> = ElementHandle::find_by_element_type_name(&app, "StepperControls").collect();
    assert_eq!(shown.len(), 4, "one stepper for each text");
}

/// A minus or plus button is a target of at least 48 by 48 px, in the narrow list pane of the two-pane form (where the
/// stepper sits under the name) and in a single form wide enough to hold it beside the name.
#[test]
fn every_button_is_at_least_48_by_48_px() {
    for width in [360.0, 560.0, 900.0] {
        let app = settings_at(width);
        for element in TextElement::ALL {
            for prefix in ["Smaller", "Larger"] {
                let (_x, _y, button_width, button_height) = box_of(&labelled(&app, &format!("{prefix} {}", element.label())));
                assert!(button_width >= 48.0 && button_height >= 48.0, "{prefix} {element:?} at width {width} is {button_width} by {button_height}");
            }
        }
    }
}

/// At the smallest size the minus button is off, at the largest the plus button is off; the end shows by the glyph
/// weight (1 px against 2 px) as well as by the state, and a press there changes and saves nothing.
#[test]
fn a_button_at_the_end_of_the_range_is_disabled_and_draws_a_lighter_glyph() {
    let app = settings_at(900.0);
    for (px, off_prefix, on_prefix) in [(MIN_TEXT_SIZE_PX, "Smaller", "Larger"), (MAX_TEXT_SIZE_PX, "Larger", "Smaller")] {
        show_sizes(&app, all_at(px));
        for element in TextElement::ALL {
            let off = labelled(&app, &format!("{off_prefix} {}", element.label()));
            let on = labelled(&app, &format!("{on_prefix} {}", element.label()));
            assert_eq!(off.accessible_enabled(), Some(false), "{off_prefix} {element:?} at {px}");
            assert_eq!(on.accessible_enabled(), Some(true), "{on_prefix} {element:?} at {px}");
        }
        // The glyph bar of a button at its end is 1 px thick and of a working button 2 px; the bars are listed in row and
        // button order (minus then plus), so the end buttons are every second one.
        let bars: Vec<f32> = elements(&app, "StepButton::glyph-bar").iter().map(|bar| return box_of(bar).3).collect();
        assert_eq!(bars.len(), 8);
        for (index, thickness) in bars.iter().enumerate() {
            let is_minus = index % 2 == 0;
            let at_end = if px == MIN_TEXT_SIZE_PX { is_minus } else { !is_minus };
            assert_eq!(*thickness, if at_end { 1.0 } else { 2.0 }, "bar {index} at {px}");
        }
    }
    SAVED.with(|saved| saved.borrow_mut().clear());
    show_sizes(&app, all_at(MAX_TEXT_SIZE_PX));
    press_button(&app, "Larger Pane titles");
    assert_eq!(sizes(), all_at(MAX_TEXT_SIZE_PX), "a press past the end changes nothing");
    assert!(saves().is_empty(), "and saves nothing");
}

/// Reset puts all four sizes back with one save, and does nothing (and says so) at the defaults.
#[test]
fn reset_puts_all_four_back_and_saves_once() {
    let app = settings_at(900.0);
    let idle_reset = labelled(&app, "Reset text sizes");
    assert_eq!(idle_reset.accessible_enabled(), Some(false), "nothing to reset at the defaults");
    idle_reset.invoke_accessible_default_action();
    settle();
    assert!(saves().is_empty(), "a press at the defaults saves nothing");

    show_sizes(&app, all_at(27));
    let reset = labelled(&app, "Reset text sizes");
    assert_eq!(reset.accessible_enabled(), Some(true));
    reset.invoke_accessible_default_action();
    settle();
    assert_eq!(sizes(), TextSizes::default());
    assert_eq!(global_sizes(&app), [20.0, 20.0, 14.0, 14.0]);
    assert_eq!(saves(), vec![TextSizes::default()]);
    assert!(app.get_text_sizes_default());
}

/// A stepper index outside the four changes nothing.
#[test]
fn a_stepper_that_does_not_exist_changes_nothing() {
    let app = settings_at(900.0);
    app.invoke_settings_step_text_size(4, 1);
    app.invoke_settings_step_text_size(-1, 1);
    settle();
    assert_eq!(sizes(), TextSizes::default());
    assert!(saves().is_empty());
}

/// Left and Right change the stepper row under the keyboard ring by one pixel; Enter and Space on it do nothing.
#[test]
fn left_and_right_step_the_focused_row_and_enter_does_not() {
    let app = settings_at(900.0);
    // Two template rows, the switch row, then the first stepper (Pane titles).
    for _ in 0..3 {
        press(&app, Key::DownArrow);
    }
    press(&app, Key::RightArrow);
    assert_eq!(global_sizes(&app), [21.0, 20.0, 14.0, 14.0]);
    press(&app, Key::LeftArrow);
    press(&app, Key::LeftArrow);
    assert_eq!(global_sizes(&app), [19.0, 20.0, 14.0, 14.0]);
    press(&app, Key::Return);
    press_text(&app, " ");
    assert_eq!(global_sizes(&app), [19.0, 20.0, 14.0, 14.0], "Enter and Space step nothing");
    // The next row is Group heading letter.
    press(&app, Key::DownArrow);
    press(&app, Key::RightArrow);
    assert_eq!(global_sizes(&app), [19.0, 21.0, 14.0, 14.0]);
    assert_eq!(saves().len(), 4);
}

/// Left and Right do nothing on a row that is not a stepper.
#[test]
fn left_and_right_do_nothing_on_the_switch_row() {
    let app = settings_at(900.0);
    press(&app, Key::DownArrow);
    press(&app, Key::DownArrow);
    press(&app, Key::RightArrow);
    press(&app, Key::LeftArrow);
    assert_eq!(sizes(), TextSizes::default());
    assert!(saves().is_empty());
}

/// Enter on the Reset row puts the sizes back, and the Licenses row is still the last stop after the new rows.
#[test]
fn the_reset_row_and_the_licenses_row_keep_their_keyboard_places() {
    let app = settings_at(900.0);
    show_sizes(&app, all_at(12));
    press(&app, Key::End);
    press(&app, Key::UpArrow);
    press(&app, Key::Return);
    assert_eq!(sizes(), TextSizes::default(), "Enter on the Reset row resets");
    press(&app, Key::End);
    press(&app, Key::Return);
    assert_eq!(app.get_route(), crate::ui_screen_view::Route::Licenses.to_int(), "End then Enter opens Licenses");
}

/// The rows of the group keep clear of each other: in the stacked form the stepper sits under the name, and in the wide
/// form beside it, at both ends of the range.
#[test]
fn a_stepper_row_never_overlaps_its_name_at_either_end_of_the_range() {
    for width in [360.0, 560.0, 900.0] {
        for px in [MIN_TEXT_SIZE_PX, MAX_TEXT_SIZE_PX] {
            let app = settings_at(width);
            show_sizes(&app, all_at(px));
            let labels = elements(&app, "TextSizeRow::label-line");
            let controls = elements(&app, "TextSizeRow::controls");
            assert_eq!((labels.len(), controls.len()), (4, 4), "width {width}");
            for (label, control) in labels.iter().zip(&controls) {
                let (label_x, label_y, label_width, label_height) = box_of(label);
                let (control_x, control_y, control_width, control_height) = box_of(control);
                let apart_horizontally = label_x + label_width <= control_x + 0.01 || control_x + control_width <= label_x + 0.01;
                let apart_vertically = label_y + label_height <= control_y + 0.01 || control_y + control_height <= label_y + 0.01;
                assert!(apart_horizontally || apart_vertically, "name {label_x},{label_y} {label_width}x{label_height} meets controls {control_x},{control_y} {control_width}x{control_height} at width {width}");
            }
        }
    }
}

/// What: `two_line_window_at` builds the window at `width` with measured tracks, so every track row has its second line.
fn two_line_window_at(width: f32) -> AppWindow {
    let app = window_at(width);
    update_state(|state| {
        for path in state.paths.clone() {
            state.measures.insert(path, TrackMeasure { duration_secs: Some(200.0), peak_dbtp: Some(-1.0), peak_exact: true });
        }
    });
    project(&app);
    advance(64);
    return app;
}

/// What: `visible` keeps the elements that are inside the window, so the copy of a list the other layout built is left out.
fn visible(app: &AppWindow, id: &str) -> Vec<ElementHandle> {
    let window_width = app.window().size().width as f32 / app.window().scale_factor();
    return elements(app, id).into_iter().filter(|element| {
        let (x, _y, width, height) = box_of(element);
        return width > 0.0 && height > 0.0 && x < window_width;
    }).collect();
}

/// A two-line track row is 72 px; at 32 px the title ends before its caption starts and the caption ends inside the row,
/// at both ends of the range, in the two-pane form and in the single form.
#[test]
fn a_track_title_never_meets_its_caption_in_a_72_px_row() {
    for width in [400.0, 1200.0] {
        for px in [MIN_TEXT_SIZE_PX, MAX_TEXT_SIZE_PX] {
            let app = two_line_window_at(width);
            show_sizes(&app, TextSizes::default().with(TextElement::TrackTitle, px));
            let titles = visible(&app, "TrackListPane::title-text");
            let captions = visible(&app, "TrackListPane::supporting-text");
            assert!(titles.len() >= 3, "the rows are built at width {width}: {}", titles.len());
            assert_eq!(titles.len(), captions.len(), "a caption under every title");
            for (title, caption) in titles.iter().zip(&captions) {
                let (_tx, title_y, _tw, title_height) = box_of(title);
                let (_cx, caption_y, _cw, caption_height) = box_of(caption);
                let row_top = title_y - 12.0;
                assert!(title_y + title_height <= caption_y + 0.01, "title {title_y}+{title_height} meets caption at {caption_y} ({px} px, width {width})");
                assert!(caption_y + caption_height <= row_top + 72.0 + 0.01, "caption ends at {} past the row end {} ({px} px, width {width})", caption_y + caption_height, row_top + 72.0);
            }
            let pitches: Vec<f32> = titles.windows(2).map(|pair| return box_of(&pair[1]).1 - box_of(&pair[0]).1).collect();
            assert!(pitches.iter().all(|pitch| return (*pitch - 72.0).abs() < 0.01), "the rows keep their 72 px pitch: {pitches:?}");
        }
    }
}

/// A row without a second line is the 48 px target; its title stays inside it and the rows keep their 48 px pitch.
#[test]
fn a_single_line_track_title_stays_inside_its_48_px_row() {
    for px in [MIN_TEXT_SIZE_PX, MAX_TEXT_SIZE_PX] {
        let app = window_at(1200.0);
        show_sizes(&app, TextSizes::default().with(TextElement::TrackTitle, px));
        let titles = visible(&app, "TrackListPane::title-text");
        assert!(titles.len() >= 3);
        for title in &titles {
            let (_x, _y, _w, height) = box_of(title);
            assert!(height <= 48.0, "a {px} px title box is {height} tall, past the 48 px row");
        }
        let pitches: Vec<f32> = titles.windows(2).map(|pair| return box_of(&pair[1]).1 - box_of(&pair[0]).1).collect();
        assert!(pitches.iter().all(|pitch| return (*pitch - 48.0).abs() < 0.01), "the rows keep their 48 px pitch: {pitches:?}");
    }
}

/// A very long title at 32 px stays inside the pane (it ends in an ellipsis) and does not wrap onto the caption.
#[test]
fn a_long_title_at_32_px_ends_in_an_ellipsis_inside_the_pane() {
    let long = "A very long title ".repeat(12);
    let (app, _host) = fresh_window_at(1200.0, WINDOW_HEIGHT);
    crate::ui_text_size::replace_saver(record_save);
    let names = vec![format!("Folder/{long}.mp3"), "Folder/Short.mp3".to_owned()];
    let paths: Vec<std::path::PathBuf> = names.iter().map(|name| return std::path::PathBuf::from("/nonexistent-library").join(name)).collect();
    crate::ui_screen::apply_update(&app, &music_player::command::Update::Library(paths.clone()));
    crate::ui_screen::apply_update(&app, &music_player::command::Update::Queue(names));
    update_state(|state| {
        for path in &paths {
            state.measures.insert(path.clone(), TrackMeasure { duration_secs: Some(200.0), peak_dbtp: Some(-1.0), peak_exact: true });
        }
    });
    show_sizes(&app, TextSizes::default().with(TextElement::TrackTitle, MAX_TEXT_SIZE_PX));
    let titles = visible(&app, "TrackListPane::title-text");
    assert_eq!(titles.len(), 2);
    for title in &titles {
        let (x, _y, width, height) = box_of(title);
        assert!(x + width <= 1200.0 - 16.0 + 0.5, "the title ends inside the pane at {}", x + width);
        assert!(height <= 42.0 + 0.01, "the title is one line of at most 42 px, not {height}");
    }
}

/// The group heading letter has the room its size needs: the letter's box ends inside its row, so it never runs into
/// the first folder name below it, at both ends of the range, in the two-pane form and in the folded picker.
#[test]
fn a_group_heading_letter_ends_inside_its_row() {
    for width in [400.0, 1200.0] {
        for px in [MIN_TEXT_SIZE_PX, MAX_TEXT_SIZE_PX] {
            let app = window_at(width);
            if width < 600.0 {
                update_state(|state| {
                    state.picker_open = true;
                });
            }
            show_sizes(&app, TextSizes::default().with(TextElement::GroupHeading, px));
            let headings = visible(&app, "FolderPickerPane::heading-label");
            let rows = visible(&app, "FolderPickerPane::name-row");
            assert!(!headings.is_empty(), "headings show at width {width}");
            for heading in &headings {
                let (_hx, heading_y, _hw, heading_height) = box_of(heading);
                let row = rows
                    .iter()
                    .map(box_of)
                    .find(|(_x, y, _w, height)| return heading_y >= *y - 0.01 && heading_y < *y + *height)
                    .expect("a heading sits in a row");
                assert!(heading_y + heading_height <= row.1 + row.3 + 0.01, "a {px} px heading ends at {} past its row end {} (width {width})", heading_y + heading_height, row.1 + row.3);
            }
        }
    }
}

/// Folder name rows keep their 48 px pitch at both ends of the range.
#[test]
fn folder_name_rows_keep_their_48_px_height() {
    for px in [MIN_TEXT_SIZE_PX, MAX_TEXT_SIZE_PX] {
        let app = window_at(1200.0);
        show_sizes(&app, TextSizes::default().with(TextElement::FolderName, px));
        let labels = visible(&app, "FolderPickerPane::name-label");
        assert!(!labels.is_empty());
        for label in &labels {
            assert_eq!(box_of(label).3, 48.0, "name label height at {px} px");
        }
    }
}

/// Selecting another folder never changes the width or place of any name, because the width probe and the visible name
/// read the same size: nothing re-wraps, at the largest size.
#[test]
fn selecting_a_folder_never_rewraps_the_names_at_32_px() {
    let app = window_at(1200.0);
    show_sizes(&app, TextSizes::default().with(TextElement::FolderName, MAX_TEXT_SIZE_PX));
    let layout = |window: &AppWindow| -> Vec<(f32, f32, f32, f32)> {
        return visible(window, "FolderPickerPane::name-row").iter().map(box_of).collect();
    };
    app.set_current_folder("Abba".into());
    settle();
    let first = layout(&app);
    app.set_current_folder("Beck".into());
    settle();
    let second = layout(&app);
    app.set_current_folder("Zed".into());
    settle();
    let third = layout(&app);
    assert!(!first.is_empty());
    assert_eq!(first, second, "selecting Beck moved or resized a name");
    assert_eq!(first, third, "selecting Zed moved or resized a name");
}

/// The pane titles keep inside their 56 px header at both ends of the range.
#[test]
fn the_pane_titles_fit_their_header_at_both_ends_of_the_range() {
    for px in [MIN_TEXT_SIZE_PX, MAX_TEXT_SIZE_PX] {
        let app = window_at(1200.0);
        show_sizes(&app, TextSizes::default().with(TextElement::PaneTitle, px));
        for kind in ["UnfoldedFoldersHeader", "UnfoldedTrackHeader"] {
            let headers: Vec<ElementHandle> = ElementHandle::find_by_element_type_name(&app, kind).collect();
            assert_eq!(headers.len(), 1, "{kind}");
            assert_eq!(box_of(&headers[0]).3, 56.0, "{kind} keeps its height at {px} px");
        }
    }
}

/// The five sites read the setting and keep no pixel literal of their own.
#[test]
fn the_five_text_sites_read_the_global_and_hold_no_literal_size() {
    let read = |file: &str| -> String {
        return std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui").join(file)).expect("the markup file reads");
    };
    let count = |text: &str, needle: &str| -> usize { return text.matches(needle).count() };
    let bars = read("bars.slint");
    let folders = read("folders.slint");
    let tracks = read("tracks.slint");
    assert_eq!(count(&bars, "font-size: TextSize.pane-title;"), 2, "both pane titles");
    assert_eq!(count(&folders, "font-size: TextSize.group-heading;"), 1);
    assert_eq!(count(&folders, "font-size: TextSize.folder-name;"), 2, "the width probe and the visible name use the same size");
    assert_eq!(count(&tracks, "font-size: TextSize.track-title;"), 1);
    for (file, text) in [("bars.slint", &bars), ("folders.slint", &folders), ("tracks.slint", &tracks)] {
        for literal in ["font-size: 14px;", "font-size: 20px;"] {
            assert_eq!(count(text, literal), 0, "{file} still holds {literal}");
        }
    }
}
