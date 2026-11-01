//! Window tests of the interaction defects of the second polish review: click-through on Search and on a toast, the
//! track menu's position and width, the Play focus ring, the seek handle at the ends, the accessible names of track and
//! menu rows, the header geometry, the Reset button, the editor's layout while typing, and the fade on appearing.
//!
//! What: each test builds the real window on the testing backend, sends real pointer and key events at coordinates, and
//! reads what reached the recording host or where an element sits.
//! Why: these defects live in the composed window (a touch area that is missing, a coordinate frame that is wrong), so a
//! test of pure logic cannot see them. The end-to-end run repeats the main ones through the nested compositor.
//! Note: element lookup by id needs `SLINT_EMIT_DEBUG_INFO=1` at build time, which the repo test task sets.

use i_slint_backend_testing::{AccessibleRole, ElementHandle, ElementQuery};
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, Model};

use crate::AppWindow;
use crate::Focus;
use crate::ui_screen::{read_state, update_state};
use crate::ui_screen_view::Route;
use crate::ui_test_support::{
    RecordingHost, advance, chord, focus_label, fresh_window_at, load_library, settle, tab_labels, tap,
};

/// What: `library_window` builds a window of the given size with the library loaded.
fn library_window(width: f32, height: f32) -> (AppWindow, std::rc::Rc<RecordingHost>) {
    let (app, host) = fresh_window_at(width, height);
    load_library(&app);
    return (app, host);
}

/// What: `box_of` gives an element's window position and size as `(x, y, width, height)`.
fn box_of(element: &ElementHandle) -> (f32, f32, f32, f32) {
    let position = element.absolute_position();
    let size = element.size();
    return (position.x, position.y, size.width, size.height);
}

/// What: `elements` lists the elements with a qualified id, in tree order.
fn elements(app: &AppWindow, id: &str) -> Vec<ElementHandle> {
    return ElementHandle::find_by_element_id(app, id).collect();
}

/// What: `click` presses and releases a pointer button at a window position.
fn click(app: &AppWindow, x: f32, y: f32, button: PointerEventButton) {
    let position = LogicalPosition::new(x, y);
    app.window().dispatch_event(WindowEvent::PointerMoved { position });
    app.window().dispatch_event(WindowEvent::PointerPressed { position, button });
    app.window().dispatch_event(WindowEvent::PointerReleased { position, button });
    settle();
}

/// What: `left_click` clicks the left button at a window position.
fn left_click(app: &AppWindow, x: f32, y: f32) {
    click(app, x, y, PointerEventButton::Left);
}

/// What: `open_search` opens the Search page with Ctrl+F, as a user does.
fn open_search(app: &AppWindow) {
    chord(app, &["Ctrl"], "f");
    advance(200);
    assert_eq!(app.get_route(), Route::Search.to_int(), "Ctrl+F opened Search");
}

// ---- CT1: Search owns every pixel it covers ----

#[test]
fn search_swallows_clicks_on_the_hidden_track_list_in_the_two_pane_layout() {
    let (app, host) = library_window(1200.0, 800.0);
    open_search(&app);
    host.clear();
    assert_eq!(focus_label(&app), "search-field", "the query field has the typing focus");
    // The first track rows of the right pane sit at y 56 and below; the first review's clicks were here.
    for y in [90.0, 150.0, 224.0, 296.0] {
        left_click(&app, 900.0, y);
    }
    assert_eq!(host.sent_texts(), Vec::<String>::new(), "no click reached the hidden player");
    assert_eq!(app.get_route(), Route::Search.to_int(), "Search is still open");
    assert_eq!(focus_label(&app), "search-field", "the query field keeps the focus");
}

#[test]
fn search_swallows_clicks_on_the_hidden_deck_in_the_folded_layout() {
    let (app, host) = library_window(443.0, 994.0);
    open_search(&app);
    host.clear();
    // The first review's points: the empty result area, the mode group and the pause button.
    for (x, y) in [(200.0, 296.0), (66.0, 953.0), (221.0, 898.0), (40.0, 700.0), (400.0, 850.0)] {
        left_click(&app, x, y);
    }
    assert_eq!(host.sent_texts(), Vec::<String>::new(), "no click reached the hidden player");
    assert_eq!(app.get_route(), Route::Search.to_int());
    assert_eq!(focus_label(&app), "search-field");
}

#[test]
fn a_click_on_the_empty_search_page_leaves_typing_working() {
    let (app, host) = library_window(443.0, 994.0);
    open_search(&app);
    host.clear();
    left_click(&app, 200.0, 400.0);
    tap(&app, "t");
    tap(&app, "o");
    assert_eq!(app.global::<Focus>().get_label().as_str(), "search-field");
    let field = elements(&app, "SearchPage::field").into_iter().next().expect("the query field");
    assert_eq!(field.accessible_value().unwrap_or_default().as_str(), "to", "the field took the keys that were typed after the click");
    assert_eq!(host.sent_texts(), Vec::<String>::new());
}

#[test]
fn the_back_button_of_the_search_page_still_takes_clicks() {
    let (app, host) = library_window(443.0, 994.0);
    open_search(&app);
    let back = ElementHandle::find_by_accessible_label(&app, "Back").next().expect("the Search page has a Back button");
    let (x, y, width, height) = box_of(&back);
    left_click(&app, x + width / 2.0, y + height / 2.0);
    assert_eq!(app.get_route(), Route::Player.to_int(), "Back leaves Search, so the shield is under the controls");
    assert_eq!(host.sent_texts(), Vec::<String>::new());
}

// ---- CT2 and MN3: a toast takes its own clicks, stays in its pane and stays long enough to read ----

/// What: `toast_card` finds the first toast card.
fn toast_card(app: &AppWindow) -> ElementHandle {
    return elements(app, "FeedbackOverlay::card").into_iter().next().expect("a toast is showing");
}

#[test]
fn a_toast_body_swallows_the_click_that_would_reach_the_transport_under_it() {
    let (app, host) = library_window(443.0, 900.0);
    crate::ui_overlays::show_long_notice(&app, "04 Quo Vadis.wav \u{b7} 1.3 MB \u{b7} \u{2026}/lib-main/Camellia");
    advance(300);
    let (card_x, card_y, card_width, card_height) = box_of(&toast_card(&app));
    // The point must sit on the message text, left of the dismiss button, over one of the deck's buttons.
    let covered = ["Previous track", "Next track"]
        .into_iter()
        .filter_map(|label| return ElementHandle::find_by_accessible_label(&app, label).next())
        .map(|button| return box_of(&button))
        .find(|(bx, by, bw, bh)| {
            let x = bx + bw / 2.0;
            let y = card_y + card_height / 2.0;
            return x > card_x && x < card_x + card_width - 60.0 && y > *by && y < by + bh;
        });
    let (bx, _by, bw, _bh) = covered.expect("the toast covers a transport button in this window");
    host.clear();
    left_click(&app, bx + bw / 2.0, card_y + card_height / 2.0);
    assert_eq!(host.sent_texts(), Vec::<String>::new(), "the click on the toast did not press the button under it");
}

#[test]
fn a_toast_still_dismisses_from_its_own_button() {
    let (app, _host) = library_window(443.0, 900.0);
    let dismissed = std::rc::Rc::new(std::cell::Cell::new(0_u32));
    let counter = std::rc::Rc::clone(&dismissed);
    app.on_feedback_dismiss(move |_id| counter.set(counter.get() + 1));
    crate::ui_overlays::show_notice(&app, "Copied 04 Quo Vadis.wav to the clipboard");
    advance(300);
    let dismiss = ElementHandle::find_by_accessible_label(&app, "Dismiss").next().expect("the toast has a dismiss button");
    let (x, y, width, height) = box_of(&dismiss);
    left_click(&app, x + width / 2.0, y + height / 2.0);
    assert_eq!(dismissed.get(), 1, "the shield is under the dismiss button, which still reports its press");
}

#[test]
fn on_the_two_pane_layout_a_toast_stays_over_the_track_pane() {
    let (app, _host) = library_window(1200.0, 800.0);
    crate::ui_overlays::show_long_notice(
        &app,
        "04 Quo Vadis with a long title that would run across the whole window.wav \u{b7} 1.3 MB \u{b7} \u{2026}/lib-main/Camellia",
    );
    advance(300);
    let (x, _y, width, _height) = box_of(&toast_card(&app));
    assert!(x >= 600.0, "the toast starts in the right pane, at {x}");
    assert!(x + width <= 1200.0 - 12.0 + 0.5, "and ends inside the window, at {}", x + width);
}

// ---- MN1 and MN2: the track menu opens at the pointer and shows its whole shortcut ----

/// What: `row_point` gives a point inside the first track row of the visible track list.
fn row_point(app: &AppWindow, unfolded: bool) -> (f32, f32) {
    let rows: Vec<ElementHandle> = elements(app, "TrackListPane::row-touch");
    let wanted_x = if unfolded { 600.0 } else { 0.0 };
    let row = rows.iter().find(|row| return box_of(row).0 >= wanted_x && box_of(row).2 > 0.0).expect("a visible track row");
    let (x, y, width, height) = box_of(row);
    return (x + width / 2.0, y + height / 2.0);
}

#[test]
fn the_menu_opens_at_the_pointer_in_the_two_pane_layout() {
    // The card is about 320 px wide, so a pointer up to 880 px keeps the card inside the 1200 px window unclamped.
    for pointer_x in [650.0_f32, 700.0, 850.0] {
        let (app, _host) = library_window(1200.0, 800.0);
        let (_x, y) = row_point(&app, true);
        click(&app, pointer_x, y, PointerEventButton::Right);
        advance(32);
        assert!(app.get_menu_open(), "a right click opened the menu");
        assert!((app.get_menu_card_x() - pointer_x).abs() < 1.0, "x {} should be the pointer {}", app.get_menu_card_x(), pointer_x);
        assert!((app.get_menu_card_y() - y).abs() < 1.0, "y {} should be the pointer {}", app.get_menu_card_y(), y);
    }
}

#[test]
fn the_menu_opens_at_the_pointer_in_the_folded_layout() {
    let (app, _host) = library_window(443.0, 900.0);
    let (_x, y) = row_point(&app, false);
    click(&app, 100.0, y, PointerEventButton::Right);
    advance(32);
    assert!(app.get_menu_open());
    assert!((app.get_menu_card_x() - 100.0).abs() < 1.0, "x {}", app.get_menu_card_x());
    assert!((app.get_menu_card_y() - y).abs() < 1.0, "y {} should be the pointer {}", app.get_menu_card_y(), y);
}

#[test]
fn the_menu_is_pulled_back_inside_the_window_near_the_right_edge() {
    let (app, _host) = library_window(1200.0, 800.0);
    let (_x, y) = row_point(&app, true);
    click(&app, 1190.0, y, PointerEventButton::Right);
    advance(32);
    assert!(app.get_menu_open());
    let right_edge = app.get_menu_card_x() + app.get_menu_card_width();
    assert!((right_edge - 1200.0).abs() < 1.0, "the card ends at the window edge, not at {right_edge}");
}

#[test]
fn the_menu_key_opens_the_menu_inside_the_track_pane_of_the_two_pane_layout() {
    let (app, _host) = library_window(1200.0, 800.0);
    chord(&app, &["Alt"], "2");
    tap(&app, "Menu");
    advance(32);
    assert!(app.get_menu_open());
    assert!((app.get_menu_card_x() - 632.0).abs() < 1.0, "32 px inside the track pane, not {}", app.get_menu_card_x());
}

#[test]
fn the_menu_is_wide_enough_for_its_longest_shortcut() {
    let (app, _host) = library_window(1200.0, 800.0);
    chord(&app, &["Alt"], "2");
    tap(&app, "Menu");
    advance(32);
    assert!(app.get_menu_open());
    let card_width = app.get_menu_card_width();
    assert!((280.0..=360.0).contains(&card_width), "the card is {card_width} px wide");
    // The widest shortcut is the Ctrl+Shift+E of Show in file manager; its text must end inside the card.
    let shortcuts: Vec<ElementHandle> = ElementQuery::from_root(&app)
        .match_descendants()
        .match_predicate(|element| return element.accessible_description().is_some_and(|text| return text.as_str() == "Ctrl+Shift+E"))
        .find_all();
    assert!(!shortcuts.is_empty(), "the Show in file manager row is described by its shortcut");
    for row in shortcuts {
        let (x, _y, width, _height) = box_of(&row);
        assert!(x + width <= app.get_menu_card_x() + card_width + 0.5, "the row {x}+{width} fits the card ({card_width})");
    }
}

// ---- KF2: track rows and menu rows are named buttons ----

#[test]
fn every_track_row_is_a_named_button_with_its_state_and_action() {
    let (app, host) = library_window(1200.0, 800.0);
    // The window builds both layouts; the track list on screen is the one in the right pane.
    let rows: Vec<ElementHandle> = elements(&app, "TrackListPane::row-touch")
        .into_iter()
        .filter(|row| return box_of(row).0 >= 600.0)
        .collect();
    assert_eq!(rows.len(), 3, "the three tracks of the first folder, found {}", rows.len());
    assert!(rows.iter().all(|row| return row.accessible_role() == Some(AccessibleRole::Button)), "each row is a button");
    let current: Vec<&ElementHandle> = rows.iter().filter(|row| return row.accessible_checked() == Some(true)).collect();
    assert_eq!(current.len(), 1, "only the playing track reads as checked");
    assert!(current[0].accessible_label().is_some_and(|label| return label.starts_with("02 Fernando")), "it is the second track");
    assert!(rows.iter().all(|row| return row.accessible_checkable() == Some(true)));
    host.clear();
    let waterloo = rows
        .iter()
        .find(|row| return row.accessible_label().is_some_and(|label| return label.starts_with("01 Waterloo")))
        .expect("the first track's row");
    waterloo.invoke_accessible_default_action();
    settle();
    assert_eq!(host.sent_texts(), ["select 0"], "the row's default action only selects it; it stays paused (D124)");
}

#[test]
fn a_left_click_on_the_selected_row_toggles_play_and_pause_and_never_reloads_it() {
    let (app, host) = library_window(1200.0, 800.0);
    let current: ElementHandle = elements(&app, "TrackListPane::row-touch")
        .into_iter()
        .filter(|row| return box_of(row).0 >= 600.0)
        .find(|row| return row.accessible_checked() == Some(true))
        .expect("the selected track's row");
    let (x, y, width, height) = box_of(&current);
    host.clear();
    left_click(&app, x + width / 2.0, y + height / 2.0);
    assert_eq!(host.sent_texts(), ["toggle"], "a click on the selected row only toggles play and pause");
}

#[test]
fn a_left_click_on_an_unselected_row_only_selects_it() {
    let (app, host) = library_window(1200.0, 800.0);
    let unselected: ElementHandle = elements(&app, "TrackListPane::row-touch")
        .into_iter()
        .filter(|row| return box_of(row).0 >= 600.0)
        .find(|row| return row.accessible_checked() == Some(false))
        .expect("a row that is not the selected track");
    let (x, y, width, height) = box_of(&unselected);
    host.clear();
    left_click(&app, x + width / 2.0, y + height / 2.0);
    let sent = host.sent_texts();
    assert_eq!(sent.len(), 1, "one command, and no play after it: {sent:?}");
    assert!(sent[0].starts_with("select "), "the click selects the row: {sent:?}");
}

#[test]
fn every_menu_row_is_a_named_button_with_its_shortcut_as_description() {
    let (app, _host) = library_window(1200.0, 800.0);
    chord(&app, &["Alt"], "2");
    tap(&app, "Menu");
    advance(32);
    let rows: Vec<ElementHandle> = elements(&app, "TrackMenu::row-touch");
    assert_eq!(rows.len(), 6, "the menu has six action rows");
    assert!(rows.iter().all(|row| return row.accessible_role() == Some(AccessibleRole::Button)), "each row is a button");
    let labels: Vec<String> = rows.iter().map(|row| return row.accessible_label().unwrap_or_default().to_string()).collect();
    assert_eq!(labels[0], "Play");
    assert!(labels[2].starts_with("File details"), "the size follows the name: {}", labels[2]);
    let last = rows.last().expect("the last row");
    assert_eq!(last.accessible_label().unwrap_or_default().as_str(), "Move to trash");
    assert_eq!(last.accessible_description().unwrap_or_default().as_str(), "Delete");
}

// ---- KF1 and VD3: the Play ring and the seek handle ----

#[test]
fn the_play_button_ring_is_drawn_flush_outside_its_filled_circle() {
    let (app, _host) = library_window(850.0, 900.0);
    tab_labels(&app, 6);
    assert_eq!(focus_label(&app), "play", "Tab reached Play");
    assert!(app.global::<Focus>().get_ring_visible(), "the keyboard ring is on");
    let touches = ElementHandle::find_by_accessible_label(&app, "Play").collect::<Vec<_>>();
    let play = touches.iter().find(|touch| return touch.size().width > 0.0).expect("the Play button");
    let (px, py, pw, ph) = box_of(play);
    // The ring is a FocusRing the size of the 40 px filled circle (the target less 4 px on each side). Its near ring is 2 px
    // wide and flush outside the circle, and its outer ring is 1 px wide around that, so a ring of the same primary role as the
    // circle's fill is still a separate band: the outer ring in the on-primary role sets it apart.
    let boxes_of = |id: &str| return ElementHandle::find_by_element_id(&app, id).map(|ring| return box_of(&ring)).collect::<Vec<_>>();
    let has = |boxes: &[(f32, f32, f32, f32)], wanted: (f32, f32, f32, f32)| {
        return boxes.iter().any(|(x, y, width, height)| return (*x - wanted.0).abs() < 0.5 && (*y - wanted.1).abs() < 0.5 && (*width - wanted.2).abs() < 0.5 && (*height - wanted.3).abs() < 0.5);
    };
    let rings: Vec<(f32, f32, f32, f32)> = ElementHandle::find_by_element_type_name(&app, "FocusRing").map(|ring| return box_of(&ring)).collect();
    assert!(has(&rings, (px + 4.0, py + 4.0, pw - 8.0, ph - 8.0)), "a ring is the filled circle's size");
    assert!(has(&boxes_of("FocusRing::ring-box"), (px + 2.0, py + 2.0, pw - 4.0, ph - 4.0)), "its near ring is the circle grown by 2 px");
    assert!(has(&boxes_of("FocusRing::outer-ring"), (px + 1.0, py + 1.0, pw - 2.0, ph - 2.0)), "its outer ring is the circle grown by 3 px");
}

#[test]
fn the_seek_handle_sits_on_the_track_ends_and_keeps_its_gap_to_the_bars() {
    use music_player::command::Update;
    let (app, _host) = library_window(443.0, 900.0);
    for (position, label) in [(0.0, "start"), (100.0, "middle"), (200.0, "end")] {
        crate::ui_screen::apply_update(&app, &Update::Position(position));
        settle();
        let thumbs = elements(&app, "TransportDeck::seek-thumb");
        let touches = elements(&app, "TransportDeck::seek-touch");
        let filled = elements(&app, "TransportDeck::seek-filled");
        let unfilled = elements(&app, "TransportDeck::seek-unfilled");
        assert_eq!(thumbs.len(), touches.len());
        for index in 0..thumbs.len() {
            let (thumb_x, _ty, thumb_width, _th) = box_of(&thumbs[index]);
            let (touch_x, _y, touch_width, _h) = box_of(&touches[index]);
            let (filled_x, _fy, filled_width, _fh) = box_of(&filled[index]);
            let (unfilled_x, _uy, unfilled_width, _uh) = box_of(&unfilled[index]);
            match label {
                "start" => {
                    assert!((thumb_x - touch_x).abs() < 0.5, "at 0 the handle starts on the track start: {thumb_x} against {touch_x}");
                    assert!(filled_width < 0.5, "nothing is filled at 0, not a stub: {filled_width}");
                }
                "end" => {
                    assert!(((thumb_x + thumb_width) - (touch_x + touch_width)).abs() < 0.5, "at the end the handle ends on the track end");
                    assert!(unfilled_width < 0.5, "nothing is left unfilled at the end");
                }
                _ => {
                    assert!((thumb_x - (filled_x + filled_width) - 6.0).abs() < 0.5, "the gap before the handle is 6 px in the middle");
                }
            }
            if unfilled_width > 0.5 {
                assert!((unfilled_x - (thumb_x + thumb_width) - 6.0).abs() < 0.5, "the gap after the handle is 6 px at the {label}: {unfilled_x} against {thumb_x}+{thumb_width}");
            }
        }
    }
}

// ---- VD2: the template editor page header matches the Settings header ----

#[test]
fn the_editor_page_header_has_the_geometry_of_the_settings_header() {
    let (app, _host) = fresh_window_at(443.0, 900.0);
    crate::ui_settings::open_settings(&app);
    advance(200);
    let settings_back = box_of(&elements(&app, "SettingsPage::back-touch")[0]);
    app.invoke_settings_open_template(0);
    advance(200);
    assert_eq!(app.get_route(), Route::TemplateEditor.to_int());
    let editor_back = box_of(&elements(&app, "TemplateEditor::back-touch")[0]);
    assert_eq!(editor_back, settings_back, "the Back target is in the same place on both pages");
}

// ---- KF4: Reset to default is in the header ----

#[test]
fn reset_to_default_sits_in_the_header_and_only_while_the_template_differs() {
    let (app, _host) = fresh_window_at(443.0, 900.0);
    crate::ui_settings::open_settings(&app);
    app.invoke_settings_open_template(0);
    advance(200);
    assert!(elements(&app, "TemplateEditor::reset-touch").into_iter().next().is_none(), "nothing to reset at the default");
    app.invoke_editor_text_edited("Edited".into(), 6, 6);
    advance(32);
    let reset = elements(&app, "TemplateEditor::reset-touch").into_iter().next().expect("Reset appears once the text differs");
    let (_x, y, _width, height) = box_of(&reset);
    assert!(y + height <= 64.0 + 0.5, "Reset is in the 64 px header, at y {y}, not below the fields");
    reset.invoke_accessible_default_action();
    advance(32);
    assert_eq!(app.get_editor_text().as_str(), "$tf(mi(len), m:ss)$ $mi(peak)$", "the text is the default again");
    assert!(elements(&app, "TemplateEditor::reset-touch").into_iter().next().is_none(), "Reset restored the default");
}

// ---- TE2: typing moves nothing ----

/// What: `editor_field_top` gives the top of the template field.
fn editor_field_top(app: &AppWindow) -> f32 {
    return box_of(&elements(app, "TemplateEditor::field-input")[0]).1;
}

/// What: `first_field_row_top` gives the top of the first row of the Fields list.
fn first_field_row_top(app: &AppWindow) -> f32 {
    return box_of(&elements(app, "TemplateEditor::field-touch")[0]).1;
}

#[test]
fn typing_an_invalid_template_moves_neither_the_field_nor_the_field_list() {
    let (app, _host) = fresh_window_at(443.0, 900.0);
    crate::ui_settings::open_settings(&app);
    app.invoke_settings_open_template(0);
    advance(200);
    let field_top = editor_field_top(&app);
    let list_top = first_field_row_top(&app);
    app.invoke_editor_focus_changed(true);
    // Type three templates one character at a time, as a person does: every prefix is checked, so the moments with a mistake
    // line, with the help, with both and with neither are all covered.
    for template in ["$tf(mi(len), q:q)$ ", "$mi(nope)$ $tc(up, mi(title))$", "$mi(title)$ $zzz(1)$", "$$"] {
        let mut typed = String::new();
        for character in template.chars() {
            typed.push(character);
            let end = i32::try_from(typed.len()).unwrap_or(0);
            app.invoke_editor_text_edited(typed.as_str().into(), end, end);
            advance(16);
            assert!((editor_field_top(&app) - field_top).abs() < 0.5, "the field moved at {typed:?}");
            assert!((first_field_row_top(&app) - list_top).abs() < 0.5, "the Fields list moved at {typed:?}");
        }
    }
}

#[test]
fn the_help_under_the_field_moves_nothing_either() {
    let (app, _host) = fresh_window_at(443.0, 900.0);
    crate::ui_settings::open_settings(&app);
    app.invoke_settings_open_template(0);
    advance(200);
    let list_top = first_field_row_top(&app);
    app.invoke_editor_focus_changed(true);
    // The caret inside tf( shows the three-line help.
    app.invoke_editor_text_edited("$tf(".into(), 4, 4);
    advance(32);
    assert!(app.get_editor_has_help());
    assert!((first_field_row_top(&app) - list_top).abs() < 0.5, "the Fields list stayed put when the help opened");
}

// ---- TE3: a draft that does not apply is kept ----

#[test]
fn a_draft_that_does_not_apply_survives_leaving_and_reopening_the_editor() {
    let (app, _host) = fresh_window_at(443.0, 900.0);
    crate::ui_settings::open_settings(&app);
    app.invoke_settings_open_template(0);
    advance(200);
    app.invoke_editor_text_edited("$mi(nope".into(), 8, 8);
    advance(32);
    assert!(app.get_editor_mistakes().row_count() > 0);
    app.invoke_editor_back();
    advance(32);
    assert_eq!(app.get_route(), Route::Settings.to_int());
    app.invoke_settings_open_template(0);
    advance(200);
    assert_eq!(app.get_editor_text().as_str(), "$mi(nope", "the draft is still there");
    assert!(app.get_editor_mistakes().row_count() > 0, "with its mistake");
    assert!(!app.get_editor_kept_note().is_empty(), "and the note that the rows keep the last valid template");
    // Leaving Settings altogether keeps it too.
    app.invoke_settings_back();
    advance(32);
    crate::ui_settings::open_settings(&app);
    app.invoke_settings_open_template(0);
    advance(200);
    assert_eq!(app.get_editor_text().as_str(), "$mi(nope", "the draft survives closing Settings");
    update_state(|state| {
        state.editor = None;
    });
    assert!(read_state(|state| return state.editor.is_none()));
}

// ---- VD4: pages, the picker overlay, the menu and a toast fade in, in 150 ms or less ----

/// What: `opacity_of` gives the opacity of the first element matching a type name or a qualified id.
fn opacity_of(app: &AppWindow, wanted: &str) -> f32 {
    let element = if wanted.contains("::") {
        elements(app, wanted).into_iter().next()
    } else {
        ElementHandle::find_by_element_type_name(app, wanted).next()
    };
    return element.unwrap_or_else(|| panic!("no {wanted} element")).computed_opacity();
}

/// What: `assert_fades_in` checks an element starts transparent, is part way after 60 ms and is fully opaque within 150 ms.
fn assert_fades_in(app: &AppWindow, wanted: &str) {
    // The change handler that restarts the fade runs on the event loop's next turn, as it does before a frame is drawn.
    settle();
    let start = opacity_of(app, wanted);
    assert!(start < 0.99, "{wanted} starts transparent, at {start}");
    advance(60);
    let middle = opacity_of(app, wanted);
    assert!(middle > start && middle < 1.0, "{wanted} is part way after 60 ms, at {middle}");
    advance(90);
    assert!((opacity_of(app, wanted) - 1.0).abs() < 0.001, "{wanted} is opaque within 150 ms");
}

#[test]
fn search_settings_and_the_editor_page_fade_in() {
    let (app, _host) = fresh_window_at(443.0, 900.0);
    load_library(&app);
    chord(&app, &["Ctrl"], "f");
    assert_fades_in(&app, "SearchPage");
    tap(&app, "Escape");
    advance(200);
    crate::ui_settings::open_settings(&app);
    assert_fades_in(&app, "SettingsPage");
    app.invoke_settings_open_template(0);
    assert_fades_in(&app, "TemplateEditor");
}

#[test]
fn the_folder_picker_overlay_fades_in_on_the_folded_layout() {
    let (app, _host) = library_window(443.0, 900.0);
    chord(&app, &["Ctrl"], "o");
    assert!(app.get_picker_open(), "Ctrl+O opened the picker");
    assert_fades_in(&app, "FolderPickerPane");
}

#[test]
fn a_toast_fades_in() {
    let (app, _host) = library_window(443.0, 900.0);
    crate::ui_overlays::show_notice(&app, "Copied 04 Quo Vadis.wav to the clipboard");
    assert_fades_in(&app, "FeedbackOverlay::card");
}

#[test]
fn the_menu_card_fades_in_when_the_menu_opens() {
    let (app, _host) = library_window(1200.0, 800.0);
    chord(&app, &["Alt"], "2");
    tap(&app, "Menu");
    advance(16);
    assert!(app.get_menu_open());
    assert_fades_in(&app, "TrackMenu");
}

// ---- MN3 and MN4: the menu actions that change nothing visible answer with a toast ----

/// What: `FakeEffects` answers the two file effects at once and records what it was asked.
/// Why: the real ones write the clipboard and start a file manager, which a test must not do.
struct FakeEffects {
    /// Whether both effects fail.
    fail: bool,
    /// The texts written to the clipboard.
    copied: std::cell::RefCell<Vec<String>>,
    /// How many times the file manager was asked for.
    shown: std::cell::Cell<u32>,
}

/// The fake file effects.
impl crate::ui_menu::FileEffects for FakeEffects {
    /// What: `copy_text` records the text, or fails.
    fn copy_text(&self, text: &str) -> Result<(), crate::ui_files::FileActionError> {
        if self.fail {
            return Err(crate::ui_files::FileActionError::Clipboard("no clipboard here".to_string()));
        }
        self.copied.borrow_mut().push(text.to_string());
        return Ok(());
    }

    /// What: `show_in_file_manager` counts the request, or fails.
    fn show_in_file_manager(&self, _path: &std::path::Path) -> Result<(), crate::ui_files::FileActionError> {
        if self.fail {
            return Err(crate::ui_files::FileActionError::Spawn("no file manager here".to_string()));
        }
        self.shown.set(self.shown.get() + 1);
        return Ok(());
    }
}

/// What: `toast_texts` lists the texts of the toasts on screen.
fn toast_texts(app: &AppWindow) -> Vec<String> {
    return app.get_feedback_items().iter().map(|item| return item.text.to_string()).collect();
}

/// What: `run_menu_action` opens the menu on the first track row and runs one action with the fake effects.
fn run_menu_action(fail: bool, action: music_player::track_menu::TrackMenuAction) -> (AppWindow, std::rc::Rc<RecordingHost>, FakeEffects) {
    let (app, host) = library_window(1200.0, 800.0);
    let (_x, y) = row_point(&app, true);
    click(&app, 700.0, y, PointerEventButton::Right);
    advance(32);
    let effects = FakeEffects { fail, copied: std::cell::RefCell::new(Vec::new()), shown: std::cell::Cell::new(0) };
    host.clear();
    crate::ui_menu::run_action_with(&app, host.as_ref(), &effects, action);
    settle();
    return (app, host, effects);
}

#[test]
fn the_menu_play_item_selects_and_starts_the_row_at_once() {
    let (_app, host, _effects) = run_menu_action(false, music_player::track_menu::TrackMenuAction::Play);
    assert_eq!(host.sent_texts(), ["select 0", "play"], "the explicit Play item keeps starting the row (D124)");
}

#[test]
fn copy_filename_says_what_it_copied() {
    let (app, _host, effects) = run_menu_action(false, music_player::track_menu::TrackMenuAction::CopyFilename);
    assert_eq!(*effects.copied.borrow(), ["01 Waterloo.mp3"]);
    assert_eq!(toast_texts(&app), ["Copied 01 Waterloo.mp3 to the clipboard"]);
}

#[test]
fn show_in_file_manager_says_it_opened_the_folder() {
    let (app, _host, effects) = run_menu_action(false, music_player::track_menu::TrackMenuAction::ShowInFileManager);
    assert_eq!(effects.shown.get(), 1);
    assert_eq!(toast_texts(&app), ["Opened the folder of 01 Waterloo.mp3 in your file manager"]);
}

#[test]
fn a_failed_copy_or_show_gives_the_error_and_no_confirmation() {
    let (app, _host, _effects) = run_menu_action(true, music_player::track_menu::TrackMenuAction::CopyFilename);
    assert_eq!(toast_texts(&app), ["The clipboard could not be written: no clipboard here"]);
    let (shown_app, _shown_host, _shown_effects) = run_menu_action(true, music_player::track_menu::TrackMenuAction::ShowInFileManager);
    assert_eq!(toast_texts(&shown_app), ["The file manager could not be started: no file manager here"]);
}

#[test]
fn file_details_names_the_file_its_size_and_a_short_folder() {
    let (app, _host, _effects) = run_menu_action(false, music_player::track_menu::TrackMenuAction::Details);
    let separator = std::path::MAIN_SEPARATOR;
    assert_eq!(toast_texts(&app), [format!("01 Waterloo.mp3 \u{b7} size unknown \u{b7} nonexistent-library{separator}Abba")]);
}

// ---- MN3: how long a toast stays ----

/// What: `stays_about` asserts the first toast has about `low` to `high` milliseconds left.
fn stays_about(low: i64, high: i64) {
    let left = crate::ui_overlays::first_message_expires_in_ms().expect("a toast is showing");
    assert!((low..=high).contains(&left), "the toast has {left} ms left, expected {low} to {high}");
}

#[test]
fn a_confirmation_stays_about_four_seconds_and_a_message_with_undo_stays_longer() {
    let (app, _host) = library_window(443.0, 900.0);
    crate::ui_overlays::show_notice(&app, "Copied 01 Waterloo.mp3 to the clipboard");
    stays_about(3_800, 5_200);
    let (undo_app, _undo_host) = library_window(443.0, 900.0);
    crate::ui_overlays::show_undo(&undo_app, "Moved 01 Waterloo.mp3 to the trash", std::path::PathBuf::from("/nonexistent-library/Abba/01 Waterloo.mp3"));
    stays_about(7_800, 15_000);
}

#[test]
fn file_details_stays_long_enough_to_read_a_path() {
    let (app, _host, _effects) = run_menu_action(false, music_player::track_menu::TrackMenuAction::Details);
    assert_eq!(toast_texts(&app).len(), 1);
    stays_about(7_800, 15_000);
}
