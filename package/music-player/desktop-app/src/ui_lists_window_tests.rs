//! Window tests of the list defects of the second polish review: long names, the folder picker revealing its selection,
//! the track list keeping its place across the 600 px breakpoint, the scrollbars, the playing-row marker, the deck that no
//! longer moves with the folder name, and a list of ten thousand rows that builds only what is on screen.
//!
//! What: each test builds the real window on the testing backend, loads a queue through the same engine updates the app
//! uses, sends real pointer and key events, and reads where elements sit and what the lists report.
//! Why: these defects live in the composed window (a pane that builds every row, a picker that never scrolls, a layout that
//! follows a label), so a test of pure logic cannot see them.
//! Note: element lookup by id needs `SLINT_EMIT_DEBUG_INFO=1` at build time, which the repo test task sets.

use i_slint_backend_testing::ElementHandle;
use music_player::command::Update;
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition};

use crate::AppWindow;
use crate::Focus;
use crate::FolderLayout;
use crate::LastInput;
use crate::TrackLayout;
use crate::ui_screen::{project, update_state};
use crate::ui_screen_scroll::ScrollRequest;
use crate::ui_test_support::{RecordingHost, advance, chord, focus_label, fresh_window_at, settle};

/// What: `load_names` gives the window a queue of the given display names, with the track at `playing` loaded.
/// Why: the lists show whatever the engine reports, so the tests report their own queues the way the app does.
fn load_names(app: &AppWindow, names: &[String], playing: Option<usize>) {
    let paths: Vec<std::path::PathBuf> = names.iter().map(|name| return std::path::PathBuf::from("/nonexistent-library").join(name)).collect();
    crate::ui_screen::apply_update(app, &Update::Library(paths));
    crate::ui_screen::apply_update(app, &Update::Queue(names.to_vec()));
    if let Some(index) = playing {
        crate::ui_screen::apply_update(app, &Update::NowPlaying { index: Some(index), name: names[index].clone(), duration: 200.0 });
    }
    settle();
}

/// What: `one_folder` names `count` tracks of one folder, numbered from zero.
fn one_folder(count: usize) -> Vec<String> {
    return (0..count).map(|number| return format!("Big/Track {number:05}.mp3")).collect();
}

/// What: `many_folders` names `count` folders of one track each, numbered from zero.
fn many_folders(count: usize) -> Vec<String> {
    return (0..count).map(|number| return format!("Folder {number:03}/01 Song.mp3")).collect();
}

/// What: `window_with` builds a window of the given size with a queue loaded.
fn window_with(width: f32, height: f32, names: &[String], playing: Option<usize>) -> (AppWindow, std::rc::Rc<RecordingHost>) {
    let (app, host) = fresh_window_at(width, height);
    load_names(&app, names, playing);
    advance(64);
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

/// What: `track_scroll` reads where the track list on screen is scrolled to.
fn track_scroll(app: &AppWindow) -> f32 {
    return app.global::<TrackLayout>().get_scroll_y();
}

/// What: `folder_scroll` reads where the folder picker's names are scrolled to.
fn folder_scroll(app: &AppWindow) -> f32 {
    return app.global::<FolderLayout>().get_scroll_y();
}

/// What: `press_at` presses the left button at a window position and keeps it down.
fn press_at(app: &AppWindow, x: f32, y: f32) {
    let position = LogicalPosition::new(x, y);
    app.window().dispatch_event(WindowEvent::PointerMoved { position });
    app.window().dispatch_event(WindowEvent::PointerPressed { position, button: PointerEventButton::Left });
    settle();
}

/// What: `move_to` moves the pointer to a window position.
fn move_to(app: &AppWindow, x: f32, y: f32) {
    app.window().dispatch_event(WindowEvent::PointerMoved { position: LogicalPosition::new(x, y) });
    settle();
}

/// What: `release_at` releases the left button at a window position.
fn release_at(app: &AppWindow, x: f32, y: f32) {
    app.window().dispatch_event(WindowEvent::PointerReleased { position: LogicalPosition::new(x, y), button: PointerEventButton::Left });
    settle();
}

/// What: `wheel_at` turns the mouse wheel at a window position by `delta_y` logical pixels (negative scrolls down).
fn wheel_at(app: &AppWindow, x: f32, y: f32, delta_y: f32) {
    app.window().dispatch_event(WindowEvent::PointerMoved { position: LogicalPosition::new(x, y) });
    app.window().dispatch_event(WindowEvent::PointerScrolled { position: LogicalPosition::new(x, y), delta_x: 0.0, delta_y });
    advance(400);
}

/// What: `request_reveal` asks the list for the playing row at the top, as Ctrl+G does.
fn request_reveal(app: &AppWindow) {
    update_state(|state| {
        state.scroll_request = Some(ScrollRequest::PlayingRow);
    });
    project(app);
    advance(64);
}

/// What: `visible_rows` lists the track rows of the pane at the given left edge that are built, as `(label, y)`.
/// Why: the window builds both layouts, so the rows of the pane that is on screen are told apart by where they start.
fn built_rows(app: &AppWindow, pane_left: f32, pane_right: f32) -> Vec<(String, f32)> {
    return elements(app, "TrackListPane::row-touch")
        .iter()
        .filter(|row| {
            let (x, _y, width, _height) = box_of(row);
            return width > 0.0 && x >= pane_left - 0.5 && x < pane_right;
        })
        .map(|row| return (row.accessible_label().unwrap_or_default().to_string(), box_of(row).1))
        .collect();
}

/// What: `top_row` gives the label of the row that starts at or just below the top of the pane that is on screen.
fn top_row(app: &AppWindow, pane_left: f32, pane_right: f32, pane_top: f32) -> String {
    let mut rows: Vec<(String, f32)> = built_rows(app, pane_left, pane_right).into_iter().filter(|(_label, y)| return *y >= pane_top - 0.5).collect();
    rows.sort_by(|a, b| return a.1.total_cmp(&b.1));
    return rows.first().map(|(label, _y)| return label.clone()).unwrap_or_default();
}

// ---- PF1: a long list builds only the rows on screen ----

#[test]
fn a_list_of_three_thousand_rows_builds_only_the_rows_on_screen() {
    let names = one_folder(3000);
    let (app, _host) = window_with(1200.0, 800.0, &names, None);
    let built = app.global::<TrackLayout>().get_built_rows();
    assert!(built > 0, "the list shows rows");
    // About eighteen rows for the pane on screen; before the windowing there were three thousand.
    assert!((1..=20).contains(&built), "{built} rows are built for a list of 3000");
    assert_eq!(app.global::<TrackLayout>().get_total_height(), 3000.0 * 48.0, "the content is as tall as every row together");
}

#[test]
fn revealing_a_far_row_builds_that_row_at_the_top_of_the_pane() {
    let names = one_folder(3000);
    let (app, _host) = window_with(1200.0, 800.0, &names, Some(2000));
    request_reveal(&app);
    assert_eq!(track_scroll(&app), -(2000.0 * 48.0), "the playing row is at the top of the list");
    assert_eq!(top_row(&app, 600.0, 1200.0, 56.0), "02000", "and it is the row that is built at the top");
    let built = app.global::<TrackLayout>().get_built_rows();
    assert!((1..=20).contains(&built), "{built} rows are built after a jump");
}

#[test]
fn rows_far_down_a_list_of_ten_thousand_are_built_at_their_place() {
    let names = one_folder(10_000);
    let (app, _host) = window_with(1200.0, 800.0, &names, Some(9000));
    request_reveal(&app);
    assert_eq!(top_row(&app, 600.0, 1200.0, 56.0), "09000");
    let rows = built_rows(&app, 600.0, 1200.0);
    let labels: Vec<&str> = rows.iter().map(|(label, _y)| return label.as_str()).collect();
    assert!(labels.iter().any(|label| return label.starts_with("09001")), "the next row follows it: {labels:?}");
    assert!(!labels.iter().any(|label| return label.starts_with("08999")), "and the one before it is not built");
}

#[test]
fn a_hundred_thousand_rows_build_as_few_as_a_thousand() {
    let small = window_with(1200.0, 800.0, &one_folder(1000), None).0.global::<TrackLayout>().get_built_rows();
    let large = window_with(1200.0, 800.0, &one_folder(100_000), None).0.global::<TrackLayout>().get_built_rows();
    assert_eq!(small, large, "the number of built rows does not follow the length of the list");
}

#[test]
fn the_wheel_scrolls_the_list_through_the_scrollbar_column() {
    let names = one_folder(3000);
    let (app, _host) = window_with(1200.0, 800.0, &names, None);
    wheel_at(&app, 1194.0, 300.0, -240.0);
    assert!(track_scroll(&app) < -100.0, "the wheel over the bar's own column scrolled the list to {}", track_scroll(&app));
    let after_column = track_scroll(&app);
    wheel_at(&app, 900.0, 300.0, -240.0);
    assert!(track_scroll(&app) < after_column, "and the wheel over the rows still scrolls it");
}

// ---- VD1: the scrollbar ----

/// What: `bar_in` finds the scrollbar whose column starts at the given left edge.
fn bar_at(app: &AppWindow, left: f32) -> ElementHandle {
    return ElementHandle::find_by_element_type_name(app, "Scrollbar")
        .find(|bar| {
            let (x, _y, width, _height) = box_of(bar);
            return width > 0.0 && (x - left).abs() < 0.5;
        })
        .unwrap_or_else(|| panic!("a scrollbar at {left}"));
}

#[test]
fn the_track_list_has_an_always_visible_twelve_pixel_scrollbar_with_a_thumb_of_at_least_48_pixels() {
    let names = one_folder(3000);
    let (app, _host) = window_with(1200.0, 800.0, &names, None);
    let bar = bar_at(&app, 1188.0);
    let (x, y, width, height) = box_of(&bar);
    assert_eq!((x, y, width), (1188.0, 56.0, 12.0), "a 12 px column on the right edge of the track pane (D22)");
    assert_eq!(height, 744.0, "as tall as the pane");
    // The thumb is a child of the bar; a 3000 row list has a thumb far shorter than 48 px in proportion, so it sits at the minimum.
    let thumbs: Vec<ElementHandle> = elements(&app, "Scrollbar::thumb");
    let thumb = thumbs.iter().find(|thumb| return (box_of(thumb).0 - 1188.0).abs() < 12.0 && box_of(thumb).1 >= 56.0 && box_of(thumb).2 > 0.0).expect("the thumb");
    assert_eq!(box_of(thumb).3, 48.0, "the thumb keeps the 48 px target along the scroll axis (C1)");
}

#[test]
fn dragging_the_thumb_scrolls_the_list_in_proportion_and_reaches_the_end() {
    let names = one_folder(3000);
    let (app, _host) = window_with(1200.0, 800.0, &names, None);
    let range = 3000.0 * 48.0 - 744.0;
    // The thumb is 48 px tall at the top of a 744 px column, so it can travel 696 px.
    press_at(&app, 1194.0, 56.0 + 10.0);
    move_to(&app, 1194.0, 56.0 + 10.0 + 348.0);
    assert!((track_scroll(&app) + range / 2.0).abs() < 2.0, "half the travel scrolls half the range, got {}", track_scroll(&app));
    move_to(&app, 1194.0, 56.0 + 10.0 + 5000.0);
    assert!((track_scroll(&app) + range).abs() < 2.0, "dragging past the end stops at the end, got {}", track_scroll(&app));
    release_at(&app, 1194.0, 56.0 + 10.0 + 5000.0);
    move_to(&app, 1194.0, 100.0);
    assert!((track_scroll(&app) + range).abs() < 2.0, "letting go leaves the list where it is");
}

#[test]
fn a_click_in_the_scrollbar_track_pages_toward_the_click() {
    let names = one_folder(3000);
    let (app, _host) = window_with(1200.0, 800.0, &names, None);
    press_at(&app, 1194.0, 56.0 + 600.0);
    release_at(&app, 1194.0, 56.0 + 600.0);
    let first_page = track_scroll(&app);
    assert!((first_page + 744.0 * 0.9).abs() < 1.0, "one click below the thumb pages down by 90 percent of the pane, got {first_page}");
}

#[test]
fn the_thumb_follows_the_list() {
    let names = one_folder(3000);
    let (app, _host) = window_with(1200.0, 800.0, &names, Some(1500));
    request_reveal(&app);
    let thumbs: Vec<ElementHandle> = elements(&app, "Scrollbar::thumb");
    let thumb = thumbs.iter().find(|thumb| return box_of(thumb).0 >= 1180.0 && box_of(thumb).2 > 0.0).expect("the thumb");
    let expected_top = 56.0 + ((744.0_f32 - 48.0) * (1500.0 * 48.0) / (3000.0 * 48.0 - 744.0)).round();
    assert!((box_of(thumb).1 - expected_top).abs() <= 1.0, "the thumb is at {} and should be at {expected_top}", box_of(thumb).1);
}

#[test]
fn a_touch_scrollbar_is_four_pixels_wide_and_the_pointer_brings_the_wide_one_back() {
    let names = one_folder(3000);
    let (app, _host) = window_with(1200.0, 800.0, &names, None);
    app.global::<LastInput>().set_touch(true);
    settle();
    let bar = bar_at(&app, 1196.0);
    assert_eq!(box_of(&bar).2, 4.0, "the touch bar is 4 px wide and takes no grab zone");
    // A pointer event over the rows is input of the other kind, so the always visible 12 px bar returns (D22: chosen by last input).
    move_to(&app, 900.0, 300.0);
    assert!(!app.global::<LastInput>().get_touch(), "a mouse move over the rows is not touch");
    assert_eq!(box_of(&bar_at(&app, 1188.0)).2, 12.0);
}

#[test]
fn a_short_list_has_no_scrollbar() {
    let names = one_folder(3);
    let (app, _host) = window_with(1200.0, 800.0, &names, None);
    let visible = ElementHandle::find_by_element_type_name(&app, "Scrollbar").filter(|bar| return box_of(bar).2 > 0.0 && box_of(bar).0 >= 1180.0).count();
    assert!(visible <= 1, "only the track pane's bar can exist");
    let thumbs: Vec<ElementHandle> = elements(&app, "Scrollbar::thumb");
    assert!(thumbs.iter().all(|thumb| return box_of(thumb).3 == 0.0 || box_of(thumb).0 < 1180.0), "a list that fits has no thumb to drag");
}

// ---- D125: the folder picker has no scrollbar ----

/// What: `long_picker_and_long_folder` names 150 one-track folders and one folder of 100 tracks.
/// Why: with the long folder playing, the picker and the track list both overflow their panes, so a scrollbar on either
/// would be drawn, and the track list's bar is the positive control that the lookup can see a bar at all.
fn long_picker_and_long_folder() -> Vec<String> {
    let mut names = many_folders(150);
    names.extend(one_folder(100));
    return names;
}

#[test]
fn the_wide_folder_picker_has_no_scrollbar_on_either_input_while_the_track_list_keeps_its_own() {
    let names = long_picker_and_long_folder();
    let (app, _host) = window_with(1200.0, 800.0, &names, Some(150));
    // Pointer input: the only bar is the track list's 12 px column at the right edge (D22); the picker, 600 px wide, has none.
    let bars = bars_on_screen(&app);
    assert_eq!(bars.len(), 1, "one bar is drawn, the track list's: {bars:?}");
    assert_eq!((bars[0].0, bars[0].1), (1188.0, 12.0), "and it is the 12 px column on the right edge of the track pane");
    // The wheel still scrolls the picker, which overflows its pane, so there was something for a bar to show.
    wheel_at(&app, 300.0, 300.0, -240.0);
    assert!(folder_scroll(&app) < -100.0, "the wheel over the names scrolled the picker to {}", folder_scroll(&app));
    assert_eq!(bars_on_screen(&app).len(), 1, "scrolling the picker draws no bar in it");
    // Touch input: the track list gets its 4 px fading bar, and the picker still has none.
    app.global::<LastInput>().set_touch(true);
    settle();
    let touch_bars = bars_on_screen(&app);
    assert_eq!(touch_bars.len(), 1, "one touch bar is drawn, the track list's: {touch_bars:?}");
    assert_eq!((touch_bars[0].0, touch_bars[0].1), (1196.0, 4.0), "the 4 px bar at the right edge of the track pane");
}

#[test]
fn the_folded_folder_picker_has_no_scrollbar_on_either_input_and_its_names_still_scroll() {
    let names = long_picker_and_long_folder();
    let (app, _host) = window_with(443.0, 900.0, &names, Some(150));
    update_state(|state| {
        state.picker_open = true;
    });
    project(&app);
    advance(100);
    // The folded layout swaps the track list for the picker, so nothing else on the page can own a bar.
    assert_eq!(bars_on_screen(&app), Vec::<(f32, f32, f32)>::new(), "the picker draws no bar with the pointer");
    wheel_at(&app, 300.0, 300.0, -240.0);
    assert!(folder_scroll(&app) < -100.0, "the wheel over the names scrolled the picker to {}", folder_scroll(&app));
    assert_eq!(bars_on_screen(&app), Vec::<(f32, f32, f32)>::new(), "scrolling it draws no bar");
    app.global::<LastInput>().set_touch(true);
    settle();
    assert_eq!(bars_on_screen(&app), Vec::<(f32, f32, f32)>::new(), "the picker draws no fading bar for touch either");
}

// ---- LS7: the track list keeps its position across the 600 px breakpoint ----

#[test]
fn the_track_list_keeps_its_place_when_the_window_narrows_and_widens_again() {
    let names = one_folder(3000);
    let (app, _host) = window_with(1200.0, 800.0, &names, Some(500));
    request_reveal(&app);
    assert_eq!(top_row(&app, 600.0, 1200.0, 56.0), "00500");
    app.window().set_size(slint::LogicalSize::new(443.0, 900.0));
    advance(200);
    let narrow_top = elements(&app, "TrackListPane::row-touch").iter().map(|row| return box_of(row).1).fold(f32::MAX, f32::min);
    assert_eq!(track_scroll(&app), -(500.0 * 48.0), "the folded layout shows the same offset");
    let folded_top = top_row(&app, 0.0, 443.0, narrow_top.max(56.0));
    assert!(folded_top.starts_with("005"), "the folded layout's top row is near 500, not the start: {folded_top}");
    assert_ne!(folded_top, "00000");
    app.window().set_size(slint::LogicalSize::new(1200.0, 800.0));
    advance(200);
    assert_eq!(track_scroll(&app), -(500.0 * 48.0), "and widening again keeps it");
    assert_eq!(top_row(&app, 600.0, 1200.0, 56.0), "00500");
}

#[test]
fn a_ring_moved_with_the_keys_scrolls_only_the_pane_on_screen_so_the_position_still_carries_over() {
    // End puts the ring on the last row, and Page Up then scrolls the list so the ring sits at the top of the pane. The pane
    // that is not on screen has a different height, so if it scrolled itself for the ring, the position that the breakpoint
    // carries over would be the offset of its own height (found by the end-to-end run).
    let names = one_folder(3000);
    let (app, _host) = window_with(1200.0, 800.0, &names, None);
    chord(&app, &["Alt"], "2");
    advance(100);
    crate::ui_test_support::tap(&app, "End");
    advance(300);
    crate::ui_test_support::tap(&app, "PageUp");
    advance(300);
    let wide = track_scroll(&app);
    assert!(wide < -100_000.0 && wide > -143_250.0, "the list is scrolled near its end, at {wide}");
    app.window().set_size(slint::LogicalSize::new(443.0, 900.0));
    advance(300);
    assert_eq!(track_scroll(&app), wide, "the folded layout shows the same offset the wide one had");
}

#[test]
fn a_folder_chosen_while_the_list_is_gone_is_shown_from_its_top_when_the_list_returns() {
    // The folded layout destroys its track list while the folder picker is open; a folder chosen there must not leave the list
    // at the position of the folder before.
    let names = one_folder(3000);
    let (app, _host) = window_with(443.0, 900.0, &names, Some(1000));
    request_reveal(&app);
    assert!(track_scroll(&app) < -1000.0);
    update_state(|state| {
        state.picker_open = true;
        state.scroll_request = Some(ScrollRequest::Top);
    });
    project(&app);
    advance(100);
    update_state(|state| {
        state.picker_open = false;
    });
    project(&app);
    advance(100);
    assert_eq!(track_scroll(&app), 0.0, "the returning list applies the request it missed");
}

#[test]
fn a_list_that_returns_without_a_new_request_keeps_the_position_it_had() {
    let names = one_folder(3000);
    let (app, _host) = window_with(443.0, 900.0, &names, Some(1000));
    request_reveal(&app);
    let before = track_scroll(&app);
    update_state(|state| {
        state.picker_open = true;
    });
    project(&app);
    advance(100);
    update_state(|state| {
        state.picker_open = false;
    });
    project(&app);
    advance(100);
    assert_eq!(track_scroll(&app), before, "closing the picker without choosing leaves the list where it was");
}

// ---- VD6: the playing row has a marker that does not depend on the font ----

#[test]
fn the_playing_row_has_a_four_pixel_marker_on_its_left_edge() {
    let names = one_folder(30);
    let (app, _host) = window_with(1200.0, 800.0, &names, Some(2));
    let bars: Vec<ElementHandle> = elements(&app, "TrackListPane::current-bar").into_iter().filter(|bar| return box_of(bar).0 >= 599.0).collect();
    assert_eq!(bars.len(), 1, "exactly one marker, on the playing track");
    let (x, _y, width, height) = box_of(&bars[0]);
    assert_eq!((x, width, height), (600.0, 4.0, 48.0));
}

#[test]
fn a_title_in_kana_and_latin_gets_the_marker_too() {
    let names = vec!["かめりあ(Camellia)/かめりあ(Camellia) - Aegis.flac".to_owned(), "かめりあ(Camellia)/かめりあ(Camellia) - Ghost.flac".to_owned()];
    let (app, _host) = window_with(1200.0, 800.0, &names, Some(0));
    let bars: Vec<ElementHandle> = elements(&app, "TrackListPane::current-bar").into_iter().filter(|bar| return box_of(bar).0 >= 599.0).collect();
    assert_eq!(bars.len(), 1, "the marker does not depend on the script of the title");
}

// ---- OV1: long names end in an ellipsis and keep their full text ----

#[test]
fn a_very_long_track_title_stays_inside_the_pane_and_keeps_its_full_name_for_assistive_technology() {
    let long = "A".repeat(40) + &" very long title".repeat(14);
    let names = vec![format!("Folder/{long}.mp3"), "Folder/Short.mp3".to_owned()];
    let (app, _host) = window_with(1200.0, 800.0, &names, None);
    let titles: Vec<ElementHandle> = elements(&app, "TrackListPane::title-text").into_iter().filter(|title| return box_of(title).0 >= 600.0).collect();
    assert!(!titles.is_empty());
    for title in &titles {
        let (x, _y, width, _height) = box_of(title);
        assert!(x + width <= 1200.0 - 16.0 + 0.5, "the title ends inside the pane, at {}", x + width);
    }
    let rows = built_rows(&app, 600.0, 1200.0);
    assert!(rows.iter().any(|(label, _y)| return label.starts_with(&long)), "the row's accessible label is the whole title");
}

// ---- LS2 and OV1: the picker reveals the selected folder and ends a long name in an ellipsis ----

/// What: `selected_name` finds the folder name that reads as selected in the picker of the given left edge.
fn selected_name(app: &AppWindow, pane_left: f32) -> Option<ElementHandle> {
    return elements(app, "FolderPickerPane::name-touch")
        .into_iter()
        .find(|name| return name.accessible_checked() == Some(true) && box_of(name).0 >= pane_left - 0.5 && box_of(name).2 > 0.0);
}

#[test]
fn a_folder_chosen_by_playing_a_track_elsewhere_is_scrolled_into_view() {
    let names = many_folders(150);
    let (app, _host) = window_with(1200.0, 800.0, &names, Some(0));
    assert_eq!(folder_scroll(&app), 0.0, "the picker opens at the top");
    // A shuffle, a Search result or Ctrl+Shift+G makes a track in a far folder the playing one.
    crate::ui_screen::apply_update(&app, &Update::NowPlaying { index: Some(140), name: names[140].clone(), duration: 100.0 });
    advance(200);
    assert!(folder_scroll(&app) < -500.0, "the picker scrolled to its selection, to {}", folder_scroll(&app));
    let selected = selected_name(&app, 48.0).expect("the selected folder is built");
    let (_x, y, _width, height) = box_of(&selected);
    assert!(y >= 56.0 && y + height <= 56.0 + (800.0 - 56.0), "the selected folder is on screen, at y {y}");
}

#[test]
fn a_click_on_a_folder_that_is_already_visible_does_not_move_the_picker() {
    let names = many_folders(150);
    let (app, host) = window_with(1200.0, 800.0, &names, Some(0));
    let before = folder_scroll(&app);
    crate::ui_wire::select_folder(&*host, "Folder 002");
    project(&app);
    advance(200);
    assert_eq!(folder_scroll(&app), before, "a selection that is in view leaves the picker where it is");
}

#[test]
fn tab_back_onto_the_folder_list_scrolls_its_ring_into_view() {
    let names = many_folders(150);
    let (app, _host) = window_with(1200.0, 800.0, &names, Some(140));
    advance(100);
    chord(&app, &["Alt"], "1");
    advance(100);
    assert_eq!(focus_label(&app), "folder-list");
    assert!(app.global::<Focus>().get_ring_visible());
    // Scroll the picker away from the ring, move the keyboard to the track list and back.
    wheel_at(&app, 300.0, 300.0, 100_000.0);
    assert!(folder_scroll(&app) > -50.0, "the picker was scrolled to the top, at {}", folder_scroll(&app));
    chord(&app, &["Alt"], "2");
    advance(100);
    chord(&app, &["Alt"], "1");
    advance(100);
    assert!(folder_scroll(&app) < -500.0, "the ring scrolled the picker back to the focused folder, to {}", folder_scroll(&app));
}

#[test]
fn a_hundred_character_folder_name_ends_inside_the_picker_with_its_full_name_available() {
    let long = "The Extraordinarily Long Named Orchestra of the Northern Territories and Their Many Collaborators Volume 2";
    let names = vec![format!("{long}/01 Song.mp3"), "Abba/01 Song.mp3".to_owned()];
    let (app, _host) = window_with(1200.0, 800.0, &names, None);
    let label = elements(&app, "FolderPickerPane::name-touch")
        .into_iter()
        .find(|name| return name.accessible_label().is_some_and(|text| return text.as_str() == long) && box_of(name).2 > 0.0 && box_of(name).0 >= 48.0)
        .expect("the long name is built");
    let (x, _y, width, _height) = box_of(&label);
    assert!(x + width <= 600.0 - 16.0 + 0.5, "the name ends inside the picker, at {}", x + width);
}

// ---- LS6 and OV2: the deck keeps its controls where they are, and a long Shuffle name is cut in the middle ----

/// What: `deck_boxes` lists where the deck's transport row, seek bar and mode segments are, for comparing two windows.
fn deck_boxes(app: &AppWindow) -> Vec<(String, (f32, f32, f32, f32))> {
    let mut boxes = Vec::new();
    for id in ["TransportDeck::seek-touch", "TransportDeck::seg-0", "TransportDeck::seg-1", "TransportDeck::seg-2", "TransportDeck::seg-3"] {
        for element in elements(app, id).iter().filter(|element| return box_of(element).2 > 0.0 && box_of(element).1 > 300.0) {
            boxes.push((id.to_owned(), box_of(element)));
        }
    }
    return boxes;
}

#[test]
fn the_deck_controls_do_not_move_with_the_folder_name() {
    for (width, height) in [(1200.0_f32, 800.0_f32), (700.0, 900.0), (443.0, 900.0)] {
        let mut layouts = Vec::new();
        for folder in ["L", "Camellia", "The Extraordinarily Long Named Orchestra of the Northern Territories"] {
            let names = vec![format!("{folder}/01 Song.mp3"), "Zed/01 Song.mp3".to_owned()];
            let (app, _host) = window_with(width, height, &names, Some(0));
            let boxes = deck_boxes(&app);
            assert!(boxes.len() >= 5, "the deck's controls are built at {width} wide: {boxes:?}");
            layouts.push(boxes);
        }
        assert_eq!(layouts[0], layouts[1], "the controls of a short name and a medium one are in the same places at {width} px");
        assert_eq!(layouts[1], layouts[2], "and so are those of a long name at {width} px");
    }
}

/// What: `tail_widths` lists the widths of the Shuffle segment's tail texts that are drawn.
/// Why: a text that is empty is not visible, and the element lookup lists visible elements only, so a name that fits has none.
fn tail_widths(app: &AppWindow) -> Vec<f32> {
    return elements(app, "ModeSegment::tail-text").iter().map(|tail| return box_of(tail).2).collect();
}

/// What: `drawn_shuffle_width` gives the width the Shuffle segment's label and tail take together, left to right.
fn drawn_shuffle_width(app: &AppWindow) -> f32 {
    // The deck on screen is the only one built visible, so its four segments are the four labels; the folder segment is third.
    let label = elements(app, "ModeSegment::label-text");
    let tail: f32 = tail_widths(app).first().copied().unwrap_or(0.0);
    return box_of(&label[2]).2 + tail;
}

#[test]
fn a_long_shuffle_name_is_cut_in_the_middle_with_its_end_after_the_ellipsis() {
    let names = vec!["ExtraordinarilyLongDirectory/01 Song.mp3".to_owned(), "Zed/01 Song.mp3".to_owned()];
    let (app, _host) = window_with(1200.0, 800.0, &names, Some(0));
    let widths = tail_widths(&app);
    assert_eq!(widths.len(), 1, "one tail is drawn, after the ellipsis of the Shuffle segment: {widths:?}");
    assert!(widths[0] > 10.0, "the end of the name (\"ory\") is {} px wide", widths[0]);
}

#[test]
fn a_short_shuffle_name_is_shown_whole_with_no_tail() {
    let names = vec!["Camellia/01 Song.mp3".to_owned(), "Zed/01 Song.mp3".to_owned()];
    let (app, _host) = window_with(1200.0, 800.0, &names, Some(0));
    assert_eq!(tail_widths(&app), Vec::<f32>::new(), "no tail is drawn for a name that fits");
}

#[test]
fn every_cut_shuffle_name_takes_the_same_width_and_a_short_one_less() {
    let build = |folder: &str| {
        let names = vec![format!("{folder}/01 Song.mp3"), "Zed/01 Song.mp3".to_owned()];
        let (app, _host) = window_with(1200.0, 800.0, &names, Some(0));
        return drawn_shuffle_width(&app);
    };
    let first = build("ExtraordinarilyLongDirectory");
    let second = build("AnotherEquallyExtraordinaryDirectoryName");
    let short = build("Zed Zed");
    assert!((first - second).abs() <= 1.0, "two cut names fill the same cap: {first} and {second}");
    assert!(short < first, "a short name takes less: {short} against {first}");
}

// ---- VD1 and OV1 on the other lists: Settings, the editor page and Search ----

/// What: `bars_on_screen` lists the scrollbars that are drawn, as `(left edge, width, height)`.
/// Why: a scrollbar of a list that fits is not visible, and the element lookup lists visible elements only.
fn bars_on_screen(app: &AppWindow) -> Vec<(f32, f32, f32)> {
    return ElementHandle::find_by_element_type_name(app, "Scrollbar")
        .map(|bar| {
            let (x, _y, width, height) = box_of(&bar);
            return (x, width, height);
        })
        .filter(|(_x, width, _height)| return *width > 0.0)
        .collect();
}

#[test]
fn the_settings_list_has_a_scrollbar_when_its_rows_do_not_fit() {
    let (app, _host) = fresh_window_at(443.0, 150.0);
    crate::ui_settings::open_settings(&app);
    advance(200);
    let bars = bars_on_screen(&app);
    assert!(bars.iter().any(|(x, width, _height)| return (*x - 431.0).abs() < 0.5 && *width == 12.0), "a 12 px bar on the right edge: {bars:?}");
}

#[test]
fn the_settings_list_has_no_scrollbar_when_its_rows_fit() {
    // 1400 px holds the whole list, including the Text size group (four stepper rows and the Reset row).
    let (app, _host) = fresh_window_at(443.0, 1400.0);
    crate::ui_settings::open_settings(&app);
    advance(200);
    assert_eq!(bars_on_screen(&app), Vec::<(f32, f32, f32)>::new(), "nothing to scroll, nothing to draw");
}

#[test]
fn the_template_editor_page_has_a_scrollbar_when_its_body_does_not_fit() {
    let (app, _host) = fresh_window_at(443.0, 300.0);
    crate::ui_settings::open_settings(&app);
    advance(100);
    app.invoke_settings_open_template(0);
    advance(300);
    assert_eq!(app.get_route(), 3, "the editor page is open");
    let bars = bars_on_screen(&app);
    assert!(bars.iter().any(|(x, width, _height)| return (*x - 431.0).abs() < 0.5 && *width == 12.0), "a 12 px bar on the right edge: {bars:?}");
}

#[test]
fn the_search_results_have_a_scrollbar_and_a_long_title_ends_inside_the_page() {
    use slint::{Model, ModelRc, VecModel};
    let (app, _host) = fresh_window_at(1200.0, 800.0);
    let segment = |text: &str| return crate::SearchSegment { text: text.into(), emphasized: false };
    let long = "Very long result title ".repeat(20);
    let rows: Vec<crate::SearchRow> = (0..30)
        .map(|index| {
            return crate::SearchRow {
                kind: 1,
                title_segments: ModelRc::new(VecModel::from(vec![segment(if index == 0 { long.as_str() } else { "Short" })])),
                parent_segments: ModelRc::new(VecModel::from(vec![segment("Folder")])),
                has_parent: true,
                current: false,
                playing: false,
                spoken: "".into(),
                index,
            };
        })
        .collect();
    assert_eq!(rows.len(), 30);
    app.set_search_rows(ModelRc::new(VecModel::from(rows)));
    app.set_route(1);
    advance(200);
    let bars = bars_on_screen(&app);
    assert!(bars.iter().any(|(x, width, _height)| return (*x - 1188.0).abs() < 0.5 && *width == 12.0), "a 12 px bar on the right edge of the Search page: {bars:?}");
    let titles: Vec<ElementHandle> = elements(&app, "SearchPage::title-text").into_iter().filter(|title| return box_of(title).2 > 0.0).collect();
    assert!(!titles.is_empty());
    for title in &titles {
        let (x, _y, width, _height) = box_of(title);
        assert!(x + width <= 1200.0 + 0.5, "a result title ends inside the page, at {}", x + width);
    }
    // A result's parent line sits under its title line; the lines once collapsed onto each other.
    let parents: Vec<ElementHandle> = elements(&app, "SearchPage::parent-text").into_iter().filter(|parent| return box_of(parent).2 > 0.0).collect();
    assert!(!parents.is_empty() && parents.len() == titles.len(), "every result has both lines");
    for (title, parent) in titles.iter().zip(parents.iter()) {
        let (_x, title_y, _width, title_height) = box_of(title);
        let (_px, parent_y, _pw, _ph) = box_of(parent);
        assert!(parent_y >= title_y + title_height - 0.5, "the parent line (y {parent_y}) is under the title line (y {title_y}, {title_height} tall)");
    }
    assert_eq!(app.get_search_rows().row_count(), 30);
}
