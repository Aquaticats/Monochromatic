//! Window tests of the focus model on the testing backend (D106): the Tab order, the reverse order, no keyboard trap, the
//! lists reachable by Alt+1 and Alt+2, and where the focus ring turns on and off.
//!
//! What: the real window receives real key events, and each test reads the name of the element that holds focus.
//! Why: the order is declared in the markup, so only the composed window shows it.

use i_slint_backend_testing::ElementHandle;
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition};

use crate::Focus;
use crate::ui_screen::read_state;
use crate::ui_screen_view::Route;
use crate::ui_test_support::{advance, chord, fresh_window_at, focus_label, load_library, settle, tab_labels, tap};

/// The unfolded Tab order D106 asks for: the folder picker, the track list, the deck controls, then Search and Settings.
const UNFOLDED_ORDER: [&str; 10] = [
    "open-pill",
    "folder-list",
    "track-list",
    "seek",
    "previous",
    "play",
    "next",
    "mode-group",
    "search",
    "settings",
];

/// The folded Tab order with the track list showing: the folder trigger and the Open pill (the folder picker's controls),
/// the track list, the deck, then Search and Settings.
const FOLDED_ORDER: [&str; 10] = [
    "folder-trigger",
    "open-pill",
    "track-list",
    "seek",
    "previous",
    "play",
    "next",
    "mode-group",
    "search",
    "settings",
];

#[test]
fn tab_walks_the_unfolded_player_page_in_the_asked_order_and_wraps_without_a_trap() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    let labels = tab_labels(&app, UNFOLDED_ORDER.len() + 3);
    assert_eq!(&labels[..UNFOLDED_ORDER.len()], UNFOLDED_ORDER.as_slice(), "the first pass");
    assert_eq!(&labels[UNFOLDED_ORDER.len()..], ["open-pill", "folder-list", "track-list"], "the order repeats: no trap");
}

#[test]
fn tab_walks_the_folded_player_page_in_the_asked_order() {
    let (app, _host) = fresh_window_at(443.0, 900.0);
    load_library(&app);
    let labels = tab_labels(&app, FOLDED_ORDER.len());
    assert_eq!(labels, FOLDED_ORDER);
}

#[test]
fn tab_walks_the_folded_page_with_the_folder_picker_open_through_the_picker() {
    let (app, _host) = fresh_window_at(443.0, 900.0);
    load_library(&app);
    chord(&app, &["Ctrl"], "o");
    advance(32);
    assert_eq!(focus_label(&app), "folder-list", "opening the picker puts the keyboard in it");
    let labels = tab_labels(&app, 9);
    assert_eq!(
        labels,
        ["seek", "previous", "play", "next", "mode-group", "search", "settings", "folder-trigger", "open-pill"],
        "the picker is the list; the track list is covered"
    );
}

#[test]
fn shift_tab_walks_the_same_order_backwards() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    tab_labels(&app, 7);
    assert_eq!(focus_label(&app), "next");
    chord(&app, &["Shift"], "Tab");
    assert_eq!(focus_label(&app), "play");
    chord(&app, &["Shift"], "Tab");
    assert_eq!(focus_label(&app), "previous");
    chord(&app, &["Shift"], "Backtab");
    assert_eq!(focus_label(&app), "seek", "a platform that sends Backtab moves the same way");
}

#[test]
fn alt_1_and_alt_2_focus_the_lists_on_the_unfolded_layout() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    chord(&app, &["Alt"], "1");
    assert_eq!(focus_label(&app), "folder-list");
    assert_eq!(app.global::<Focus>().get_zone(), 1);
    chord(&app, &["Alt"], "2");
    assert_eq!(focus_label(&app), "track-list");
    assert_eq!(app.global::<Focus>().get_zone(), 2);
    chord(&app, &["Alt"], "1");
    assert_eq!(focus_label(&app), "folder-list", "and back again");
}

#[test]
fn alt_1_opens_the_folder_picker_on_the_folded_layout_and_alt_2_closes_it() {
    let (app, _host) = fresh_window_at(443.0, 900.0);
    load_library(&app);
    assert!(!read_state(|state| return state.picker_open));
    chord(&app, &["Alt"], "1");
    advance(32);
    assert!(read_state(|state| return state.picker_open), "Alt+1 opens the picker overlay");
    assert_eq!(focus_label(&app), "folder-list", "and puts the keyboard in it");
    chord(&app, &["Alt"], "2");
    advance(32);
    assert!(!read_state(|state| return state.picker_open), "Alt+2 closes it");
    assert_eq!(focus_label(&app), "track-list", "and puts the keyboard in the list it uncovers");
}

#[test]
fn a_key_press_turns_the_ring_on_and_a_pointer_press_turns_it_off() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    assert!(!app.global::<Focus>().get_ring_visible(), "no ring before any key");
    tap(&app, "Tab");
    assert!(app.global::<Focus>().get_ring_visible(), "a key press shows the ring");
    let row = ElementHandle::find_by_element_id(&app, "TrackListPane::row-touch").next().expect("a track row exists");
    let position = row.absolute_position();
    let size = row.size();
    let center = LogicalPosition::new(position.x + size.width / 2.0, position.y + size.height / 2.0);
    app.window().dispatch_event(WindowEvent::PointerMoved { position: center });
    app.window().dispatch_event(WindowEvent::PointerPressed { position: center, button: PointerEventButton::Left });
    app.window().dispatch_event(WindowEvent::PointerReleased { position: center, button: PointerEventButton::Left });
    settle();
    assert!(!app.global::<Focus>().get_ring_visible(), "a click hides the ring");
    assert_eq!(focus_label(&app), "track-list", "and leaves the keyboard on the list that was clicked");
}

#[test]
fn modifier_keys_alone_do_not_show_the_ring() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    tap(&app, "Shift");
    tap(&app, "Ctrl");
    assert!(!app.global::<Focus>().get_ring_visible());
}

#[test]
fn the_search_page_keeps_tab_inside_it_and_does_not_trap() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    chord(&app, &["Ctrl"], "f");
    assert_eq!(app.get_route(), Route::Search.to_int());
    assert_eq!(focus_label(&app), "search-field", "Search opens with the query field focused");
    let labels = tab_labels(&app, 4);
    for label in &labels {
        assert!(
            ["search-back", "search-field", "search-clear"].contains(&label.as_str()),
            "Tab stays on the Search page, got {label}"
        );
    }
    assert!(labels.contains(&"search-back".to_owned()), "the Back button is reachable: {labels:?}");
    assert!(labels.contains(&"search-field".to_owned()), "the field is reachable again: {labels:?}");
}

#[test]
fn the_settings_page_keeps_tab_inside_it_and_a_template_line_does_not_swallow_tab() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    chord(&app, &["Ctrl", "Alt"], "s");
    assert_eq!(app.get_route(), Route::Settings.to_int());
    assert_eq!(focus_label(&app), "settings-list", "Settings opens with its list focused");
    let labels = tab_labels(&app, 12);
    for label in &labels {
        assert!(
            label == "settings-back" || label == "settings-list" || label == "template-field" || label.starts_with("insert-field-") || label == "reset",
            "Tab stays on the Settings page, got {label}"
        );
    }
    assert!(labels.contains(&"template-field".to_owned()), "the template line is reachable: {labels:?}");
    let after_field = labels.iter().position(|label| return label == "template-field").expect("reached");
    assert_ne!(labels.get(after_field + 1).map(String::as_str), Some("template-field"), "Tab leaves the template line");
}
