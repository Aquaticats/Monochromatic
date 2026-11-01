//! Window callbacks for the library screen: each user action becomes engine commands.
//!
//! What: `wire` registers the transport, row, mode, folder picker and open-folder callbacks on `AppWindow`.
//! Why: `main` stays a startup sequence, and the mapping from user action to `Command` lives in one place.

use std::rc::Rc;

use music_player::command::Command;
use music_player::engine::Engine;
use music_player::row_activation::{RowActivation, row_activation_for};
use slint::ComponentHandle;

use crate::ui_host::Host;

use crate::AppWindow;
use crate::ui_playback::int_to_playback_mode;
use crate::ui_screen::{project, read_state, update_state};
use crate::ui_screen_view::{select_folder_label, selected_page_scope};

thread_local! {
    /// What: the main window, for messages raised from a worker thread through the event loop.
    /// Why: the folder chooser runs off the UI thread and, when none exists, must show a message in the window.
    static MAIN_WINDOW: std::cell::RefCell<Option<slint::Weak<AppWindow>>> = const { std::cell::RefCell::new(None) };
}

/// What: `wire` registers every library-screen callback on the window.
/// Why: the callbacks share the engine handle, so they are registered together after the engine starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wire(app: AppWindow, engine: Engine): void { ... }
/// ```
pub(crate) fn wire(app: &AppWindow, engine: &Rc<Engine>) {
    MAIN_WINDOW.with(|cell| {
        *cell.borrow_mut() = Some(app.as_weak());
    });
    let host: Rc<dyn Host> = Rc::clone(engine) as Rc<dyn Host>;
    wire_host_callbacks(app, &host);
    let open_sender = engine.sender();
    app.on_open_files(move || {
        open_folder_dialog(open_sender.clone());
    });
    crate::ui_overlays::wire(app, engine);
    crate::ui_menu::wire(app, engine);
    crate::ui_settings::wire(app);
    crate::ui_search::wire(app, engine);
}

/// What: `mark_user_acted` records that the user has started to drive the player.
/// Why: the one-time reveal of the restored playing track must not fire for a track the user starts themselves, which
/// would scroll the list under them.
pub(crate) fn mark_user_acted() {
    update_state(|state| {
        state.startup_reveal_done = true;
    });
}

/// What: `wire_host_callbacks` registers the callbacks that only send commands to a host: the transport deck, row play,
/// folder pick, the picker toggle and the keyboard.
/// Why: the window tests register the same callbacks with a recording host, so a click or a key reaches the same code as
/// in the application.
pub(crate) fn wire_host_callbacks(app: &AppWindow, host: &Rc<dyn Host>) {
    wire_transport(app, host);
    wire_library(app, host);
    wire_keys(app, host);
}

/// What: `wire_transport` registers play, previous, next, seek and mode callbacks.
/// Why: the transport deck and the mode group send these without knowing the engine.
fn wire_transport(app: &AppWindow, host: &Rc<dyn Host>) {
    let toggle_host = Rc::clone(host);
    app.on_toggle_play(move || {
        mark_user_acted();
        toggle_play(toggle_host.as_ref());
    });
    let prev_host = Rc::clone(host);
    app.on_prev(move || {
        mark_user_acted();
        prev_host.send(Command::Prev);
    });
    let next_host = Rc::clone(host);
    app.on_next(move || {
        mark_user_acted();
        next_host.send(Command::Next);
    });
    let seek_host = Rc::clone(host);
    app.on_seek(move |secs| seek_host.send(Command::Seek(f64::from(secs))));
    let reveal_weak = app.as_weak();
    app.on_reveal_playing(move || {
        if let Some(window) = reveal_weak.upgrade() {
            crate::ui_keys_actions::reveal_playing(&window);
        }
    });
    let folder_reveal_weak = app.as_weak();
    app.on_reveal_folder(move || {
        if let Some(window) = folder_reveal_weak.upgrade() {
            crate::ui_keys_actions::reveal_playing_folder(&window);
        }
    });
    let mode_host = Rc::clone(host);
    let mode_weak = app.as_weak();
    app.on_select_mode(move |mode| {
        if mode_weak.upgrade().is_some() {
            let scope = read_state(selected_page_scope);
            mode_host.send(Command::SetPageScope(scope));
            mode_host.send(Command::SetPlaybackMode(int_to_playback_mode(mode)));
        }
    });
}

/// What: `wire_library` registers row play and the folder picker callbacks.
/// Why: these change the selected folder or start a track, and each one ends with a window projection.
fn wire_library(app: &AppWindow, host: &Rc<dyn Host>) {
    let row_host = Rc::clone(host);
    let row_weak = app.as_weak();
    app.on_select_index(move |index| {
        if let Some(window) = row_weak.upgrade() {
            mark_user_acted();
            play_row(row_host.as_ref(), usize::try_from(index).unwrap_or(usize::MAX));
            project(&window);
        }
    });
    let picker_weak = app.as_weak();
    app.on_toggle_picker(move || {
        if let Some(window) = picker_weak.upgrade() {
            update_state(|state| {
                // An empty library has nothing to list, so the folder button does not open an empty picker (EM5).
                state.picker_open = !state.picker_open && !state.names.is_empty();
            });
            project(&window);
        }
    });
    let folder_host = Rc::clone(host);
    let folder_weak = app.as_weak();
    app.on_select_folder(move |label| {
        if let Some(window) = folder_weak.upgrade() {
            mark_user_acted();
            select_folder(folder_host.as_ref(), label.as_str());
            project(&window);
        }
    });
}

/// What: `wire_keys` routes each key press and release from the window's key scope to the shortcut handler, and the focus
/// zone and list focus reports to the list rings.
/// Why: the window answers true when a shortcut used the key, so the focused field keeps every other key.
fn wire_keys(app: &AppWindow, host: &Rc<dyn Host>) {
    crate::ui_hints::install_texts(app);
    let key_weak = app.as_weak();
    let press_host = Rc::clone(host);
    app.on_key_pressed(move |text, ctrl, meta, alt, shift, repeat, in_popup| {
        let Some(window) = key_weak.upgrade() else {
            return false;
        };
        let key = crate::ui_keys::RawKey {
            text: text.to_string(),
            ctrl,
            meta,
            alt,
            shift,
            repeat,
            in_popup,
        };
        return crate::ui_keys::on_key_pressed(&window, press_host.as_ref(), &key);
    });
    let release_weak = app.as_weak();
    app.on_key_released(move |text, ctrl, meta, alt, shift| {
        if let Some(window) = release_weak.upgrade() {
            let key = crate::ui_keys::RawKey {
                text: text.to_string(),
                ctrl,
                meta,
                alt,
                shift,
                repeat: false,
                in_popup: false,
            };
            crate::ui_keys::on_key_released(&window, &key);
        }
    });
    let list_weak = app.as_weak();
    app.on_list_focused(move |zone| {
        if let Some(window) = list_weak.upgrade() {
            crate::ui_lists::list_focused(&window, zone);
        }
    });
    let zone_weak = app.as_weak();
    app.global::<crate::Focus>().on_zone_changed(move |zone| {
        tracing::debug!(zone, "the focus zone changed");
        if let Some(window) = zone_weak.upgrade() {
            crate::ui_state_log::emit(&window, "focus");
        }
    });
}

/// What: `open_folder_dialog` asks for a folder on a worker thread and opens it without autoplay.
/// Why: the native dialog blocks, so it runs off the event-loop thread; the engine receives the chosen root.
pub(crate) fn open_folder_dialog(sender: music_player::engine::CommandSender) {
    std::thread::spawn(move || {
        // What:     the chooser is checked before it opens.
        // Why:      the dialog library answers "nothing" both for Cancel and for "no chooser installed" (PB5); only a
        //           missing chooser needs a message, and only the check can tell the two apart.
        if music_player::dialog_backend::dialog_backend_missing() {
            tracing::warn!("no folder chooser is available; the user is told how to proceed");
            let _ = slint::invoke_from_event_loop(|| {
                MAIN_WINDOW.with(|cell| {
                    if let Some(window) = cell.borrow().as_ref().and_then(slint::Weak::upgrade) {
                        crate::ui_overlays::show_error(&window, music_player::dialog_backend::NO_DIALOG_MESSAGE);
                    }
                });
            });
            return;
        }
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            sender.send(Command::OpenRoot {
                root: dir,
                select: None,
                play: false,
            });
        }
    });
}

/// What: `play_row` handles a click, a tap or Enter on a track row and the activation of a Search result that names a
/// track: the selected row toggles play and pause, any other row becomes the selected row and stays paused, or also
/// starts at once when Auto-play on select is on.
/// Why: the user's rule (D124) is that selecting and playing are separate intents; `SelectIndex` loads paused and
/// pauses whatever was playing, and only a second click on the now-selected row starts it, so no double-click detection
/// is needed. D129 lets the user ask for the old behavior, where the selection also starts the track. A selected row
/// that was restored and never loaded, or whose load failed, starts or retries through the same toggle as the Play
/// button; a row whose file cannot be opened stays the selected row (D128) with the setting on or off.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const playRow = (index: number) => send(rowActivationFor(autoPlayOnSelect, current === index), index);
/// ```
pub(crate) fn play_row(engine: &dyn Host, index: usize) {
    let (current, auto_play_on_select) = read_state(|state| return (state.current_index, state.auto_play_on_select));
    send_row_activation(engine, index, row_activation_for(auto_play_on_select, current == Some(index)));
}

/// What: `play_row_now` is the explicit Play item of the track menu: the selected row toggles play and pause, any other
/// row is selected and starts at once, whatever Auto-play on select says.
/// Why: a menu item named Play asks to hear that track, unlike a click on a row or a Search result, which only select
/// it unless the user turned Auto-play on select on (D124, D127, D129).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const playRowNow = (index: number) => send(rowActivationFor(true, current === index), index);
/// ```
pub(crate) fn play_row_now(engine: &dyn Host, index: usize) {
    let current = read_state(|state| return state.current_index);
    send_row_activation(engine, index, row_activation_for(true, current == Some(index)));
}

/// What: `send_row_activation` maps what a row activation does to the engine commands that do it.
/// Why: the row click, Enter, the Search result and the menu Play share the mapping, so they cannot differ; `SelectIndex`
/// loads paused, so a start at once is `Play` after it.
fn send_row_activation(engine: &dyn Host, index: usize, activation: RowActivation) {
    match activation {
        RowActivation::TogglePlay => engine.send(Command::TogglePlay),
        RowActivation::SelectOnly => engine.send(Command::SelectIndex(index)),
        RowActivation::SelectAndPlay => {
            engine.send(Command::SelectIndex(index));
            engine.send(Command::Play);
        }
    }
}

/// What: `select_folder` selects the folder named `label`, closes the picker, and scopes the engine to it.
/// Why: the selection and the engine scope come from the same state write.
pub(crate) fn select_folder(engine: &dyn Host, label: &str) {
    let mut chosen_scope: Option<Vec<usize>> = None;
    update_state(|state| {
        chosen_scope = select_folder_label(state, label);
    });
    if let Some(scope) = chosen_scope {
        engine.send(Command::SetPageScope(scope));
    }
}

/// What: `toggle_play` is the play and pause button and key: with nothing selected it first sends the shown page's
/// scope, so the engine starts the first track of the folder on screen.
/// Why: the engine knows no shown page until it is told one, and Play with nothing selected must start the first row of
/// the page the user is looking at (EM2).
pub(crate) fn toggle_play(engine: &dyn Host) {
    let nothing_selected: bool = read_state(|state| return state.current_index.is_none() && !state.names.is_empty());
    if nothing_selected {
        let scope: Vec<usize> = read_state(selected_page_scope);
        engine.send(Command::SetPageScope(scope));
    }
    engine.send(Command::TogglePlay);
}
