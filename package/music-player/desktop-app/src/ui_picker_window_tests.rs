//! Window tests of the folded folder picker: with a saved selection, at the cover width or any other size, the picker
//! never opens with an empty names pane.
//!
//! What: each test builds the real window on the testing backend, loads a queue the way a restored session does (the
//! library, the queue, and the saved track as the playing one), opens the picker, and reads where the name rows sit
//! relative to the names pane.
//! Why: the Folders pane was seen empty once at a window about 443 px wide with a saved session. More than a hundred
//! restart and resize cycles in the nested compositor did not repeat a lasting empty pane, so these tests pin the
//! invariants that no timing or size may break: a picker that has names shows some of them, its content is never pushed
//! below the pane's top, and the picker keys never open a picker before the library has arrived (the one empty-pane state
//! the window could be put in, by Alt+1 or Ctrl+O at launch). The picker writes its scroll offset from timers, using the
//! layout of that moment, which is why the tests also resize the window with the picker open. Slint clamps the
//! offset to the content, so a wrong offset alone cannot blank the pane.
//! Note: element lookup by id needs `SLINT_EMIT_DEBUG_INFO=1` at build time, which the repo test task sets.

use i_slint_backend_testing::ElementHandle;
use music_player::command::Update;
use slint::{ComponentHandle, LogicalSize};

use crate::AppWindow;
use crate::FolderLayout;
use crate::ui_test_support::{advance, chord, fresh_window_at, settle};

/// What: `Where` is how the test opens the picker.
/// Why: the header trigger and Alt+1 reach the picker through different code (the trigger checks for an empty library, the
/// key does not), and a restored session can use either.
#[derive(Clone, Copy, Debug)]
enum Where {
    /// The header folder trigger, which invokes `toggle-picker`.
    Trigger,
    /// Alt+1, which focuses the folder list and opens the overlay first on the folded layout.
    AltOne,
}

/// What: `library` names `folders` folders of two tracks each, spread over the alphabet and over a few writing systems,
/// plus a few loose files that sit in the virtual letter folders.
/// Why: a real library has many folders per letter and several rail sections, so the picker's heading rows, hairlines and
/// wrapped names all occur.
fn library(folders: usize) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for number in 0..folders {
        let letter = char::from(b'A' + u8::try_from(number % 26).unwrap_or(0));
        for track in 1..=2 {
            names.push(format!("{letter}olk {number:04} {}/{track:02} Song.mp3", "x".repeat(number % 23)));
        }
    }
    for folder in ["かめりあ(Camellia)", "Мир", "音楽", "1999 Mix"] {
        names.push(format!("{folder}/01 Song.mp3"));
    }
    for loose in ["apple.mp3", "banana.mp3", "zed.mp3"] {
        names.push(loose.to_owned());
    }
    return names;
}

/// What: `restored_window` builds a window of the given size and loads the library with the saved track playing.
/// Why: a restored session reports the library, the queue and the selected track, in that order, before the picker is used.
fn restored_window(width: f32, height: f32, names: &[String], saved: usize) -> AppWindow {
    let (app, _host) = fresh_window_at(width, height);
    let paths: Vec<std::path::PathBuf> = names.iter().map(|name| return std::path::PathBuf::from("/nonexistent-library").join(name)).collect();
    crate::ui_screen::apply_update(&app, &Update::Library(paths));
    crate::ui_screen::apply_update(&app, &Update::Queue(names.to_vec()));
    crate::ui_screen::apply_update(&app, &Update::NowPlaying { index: Some(saved), name: names[saved].clone(), duration: 200.0 });
    settle();
    advance(64);
    return app;
}

/// What: `open_picker` opens the folded picker the given way and lets its timers run.
fn open_picker(app: &AppWindow, how: Where) {
    match how {
        Where::Trigger => app.invoke_toggle_picker(),
        Where::AltOne => chord(app, &["Alt"], "1"),
    }
    advance(300);
}

/// What: `pane_box` gives the names pane of the picker on screen as `(x, y, width, height)`.
/// Why: the names are visible when they overlap this box.
fn pane_box(app: &AppWindow) -> (f32, f32, f32, f32) {
    let panes: Vec<ElementHandle> = ElementHandle::find_by_element_id(app, "FolderPickerPane::names-flick").collect();
    assert_eq!(panes.len(), 1, "exactly one names pane is built while the picker is open on the folded layout");
    let position = panes[0].absolute_position();
    let size = panes[0].size();
    return (position.x, position.y, size.width, size.height);
}

/// What: `visible_names` lists the folder names whose rows overlap the names pane.
/// Why: the window builds only the rows near the pane, so a pane that was scrolled past its content lists none.
fn visible_names(app: &AppWindow) -> Vec<String> {
    let (pane_x, pane_y, pane_width, pane_height) = pane_box(app);
    return ElementHandle::find_by_element_id(app, "FolderPickerPane::name-touch")
        .filter(|name| {
            let position = name.absolute_position();
            let size = name.size();
            return size.width > 0.0
                && position.x >= pane_x - 0.5
                && position.x < pane_x + pane_width
                && position.y < pane_y + pane_height
                && position.y + size.height > pane_y;
        })
        .filter_map(|name| return name.accessible_label().map(|label| return label.to_string()))
        .collect();
}

/// What: `assert_not_blank` checks the invariant: some names are on screen, and the names are not pushed down.
/// Why: a names pane with names and no visible row is the empty Folders pane; a scroll offset above zero would push every
/// row down and leave a blank band at the top.
fn assert_not_blank(app: &AppWindow, context: &str) {
    assert!(app.get_picker_open(), "{context}: the picker is open");
    let scroll = app.global::<FolderLayout>().get_scroll_y();
    let shown = visible_names(app);
    assert!(!shown.is_empty(), "{context}: the picker shows at least one folder name, scroll {scroll}");
    assert!(scroll <= 0.0, "{context}: the names are never pushed below the top of their pane, scroll {scroll}");
}

#[test]
fn the_folded_picker_is_not_blank_for_any_saved_selection_and_either_way_of_opening_it() {
    for folders in [3, 40, 150] {
        let names = library(folders);
        let last = names.len() - 1;
        for saved in [0, names.len() / 2, last] {
            for how in [Where::Trigger, Where::AltOne] {
                let app = restored_window(443.0, 900.0, &names, saved);
                open_picker(&app, how);
                assert_not_blank(&app, &format!("{folders} folders, saved track {saved}, opened by {how:?}"));
            }
        }
    }
}

#[test]
fn the_folded_picker_is_not_blank_in_a_short_window_or_at_the_widest_folded_width() {
    let names = library(60);
    let saved = names.len() / 2;
    for (width, height) in [(443.0, 600.0), (443.0, 480.0), (599.0, 900.0), (360.0, 700.0), (443.0, 1700.0)] {
        let app = restored_window(width, height, &names, saved);
        open_picker(&app, Where::AltOne);
        assert_not_blank(&app, &format!("window {width} x {height}"));
    }
}

#[test]
fn the_folded_picker_stays_filled_when_the_window_is_resized_with_it_open() {
    let names = library(120);
    let saved = names.len() - 7;
    let app = restored_window(443.0, 900.0, &names, saved);
    open_picker(&app, Where::AltOne);
    assert_not_blank(&app, "before any resize");
    // Across the 600 px fold and back: the unfolded layout shows its own picker, and the folded one is built again.
    for (width, height) in [(850.0, 900.0), (443.0, 900.0), (599.0, 900.0), (443.0, 600.0), (443.0, 1700.0), (443.0, 900.0), (601.0, 480.0), (443.0, 900.0)] {
        app.window().set_size(LogicalSize::new(width, height));
        advance(300);
        if width < 600.0 {
            assert_not_blank(&app, &format!("after a resize to {width} x {height}"));
        }
    }
}

#[test]
fn closing_and_reopening_the_folded_picker_never_leaves_it_blank() {
    let names = library(120);
    let app = restored_window(443.0, 900.0, &names, names.len() / 2);
    for round in 0..6 {
        open_picker(&app, if round % 2 == 0 { Where::Trigger } else { Where::AltOne });
        assert_not_blank(&app, &format!("opening, round {round}"));
        chord(&app, &["Alt"], "2");
        advance(100);
        assert!(!app.get_picker_open(), "Alt+2 closes the picker, round {round}");
    }
}

#[test]
fn the_picker_keys_do_not_open_an_empty_picker_before_the_library_has_arrived() {
    // At launch with a saved session the window is shown before the engine reports the library. The first-run pane has
    // nothing to say yet, so a picker opened then would be a pane with no rail and no names, as the header button's own
    // empty check already prevents.
    for (modifier, key) in [("Alt", "1"), ("Ctrl", "o")] {
        let (app, _host) = fresh_window_at(443.0, 900.0);
        advance(64);
        chord(&app, &[modifier], key);
        advance(300);
        assert!(!app.get_picker_open(), "{modifier}+{key} does not open a picker with nothing to list");
        // When the library arrives the picker stays closed, and the same key then opens it with names in it.
        let names = library(40);
        let paths: Vec<std::path::PathBuf> = names.iter().map(|name| return std::path::PathBuf::from("/nonexistent-library").join(name)).collect();
        crate::ui_screen::apply_update(&app, &Update::Library(paths));
        crate::ui_screen::apply_update(&app, &Update::Queue(names));
        settle();
        advance(300);
        assert!(!app.get_picker_open(), "the picker that was refused does not open by itself when the library arrives");
        chord(&app, &[modifier], key);
        advance(300);
        assert_not_blank(&app, &format!("{modifier}+{key} after the library arrived"));
    }
}
