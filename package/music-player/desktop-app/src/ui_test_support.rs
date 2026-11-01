//! Shared helpers of the window tests that press real key events.
//!
//! What: a recording `Host`, the testing-backend setup, and helpers that dispatch key presses, releases and chords to a
//! window the way the platform backend does.
//! Why: every shortcut is tested through the window's own key scope and callbacks, so the tests share one setup and one
//! recording host instead of each file inventing its own.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use music_player::command::Command;
use music_player::track_menu::TrackMenuAction;
use slint::platform::{Key, WindowEvent};
use slint::{ComponentHandle, SharedString};

use crate::AppWindow;
use crate::ui_host::Host;
use crate::ui_screen::{project, update_state};
use crate::ui_screen_view::ScreenState;

/// What: `RecordingHost` is a host that records what the shortcuts ask for and does nothing else.
/// Why: a test must see the request without playing audio, opening a dialog, trashing a file or ending the process.
#[derive(Default)]
pub(crate) struct RecordingHost {
    /// The engine commands sent, in order, described as short texts.
    pub sent: RefCell<Vec<String>>,
    /// How many times the system folder dialog was requested.
    pub dialogs: Cell<u32>,
    /// How many times quitting was requested.
    pub quits: Cell<u32>,
    /// The track menu actions run on a row, in order.
    pub row_actions: RefCell<Vec<TrackMenuAction>>,
}

/// Methods of `RecordingHost`.
impl RecordingHost {
    /// What: `describe` writes a command as a short text a test can compare.
    /// Why: `Command` is not `Debug`, and the tests need only its kind and a number or two.
    fn describe(command: &Command) -> String {
        return match command {
            Command::TogglePlay => "toggle".to_owned(),
            Command::Prev => "prev".to_owned(),
            Command::Next => "next".to_owned(),
            Command::Play => "play".to_owned(),
            Command::Pause => "pause".to_owned(),
            Command::Seek(seconds) => format!("seek {seconds}"),
            Command::SetPlaybackMode(mode) => format!("mode {mode:?}"),
            Command::SetPageScope(indices) => format!("scope {}", indices.len()),
            Command::SelectIndex(index) => format!("select {index}"),
            _ => "other".to_owned(),
        };
    }

    /// What: `sent_texts` lists the commands sent so far.
    /// Why: tests compare the whole list.
    pub(crate) fn sent_texts(&self) -> Vec<String> {
        return self.sent.borrow().clone();
    }

    /// What: `clear` forgets everything recorded so far.
    /// Why: a test checks one key at a time.
    pub(crate) fn clear(&self) {
        self.sent.borrow_mut().clear();
        self.row_actions.borrow_mut().clear();
        self.dialogs.set(0);
        self.quits.set(0);
    }
}

/// The recording host.
impl Host for RecordingHost {
    /// What: `send` records the command.
    fn send(&self, command: Command) {
        self.sent.borrow_mut().push(Self::describe(&command));
    }

    /// What: `pick_folder` counts the request instead of opening a dialog.
    fn pick_folder(&self) {
        self.dialogs.set(self.dialogs.get() + 1);
    }

    /// What: `quit` counts the request instead of ending the process.
    fn quit(&self) {
        self.quits.set(self.quits.get() + 1);
    }

    /// What: `run_row_action` records the action instead of touching a file.
    fn run_row_action(&self, _window: &AppWindow, action: TrackMenuAction) {
        self.row_actions.borrow_mut().push(action);
    }
}

thread_local! {
    /// What: the test clock in milliseconds, which `advance` moves together with Slint's mock clock.
    static CLOCK_MS: Cell<u64> = const { Cell::new(10_000) };
}

/// What: `init_platform` installs the testing backend once per test thread.
/// Why: `init_no_event_loop` installs a thread-local platform, and each test runs on its own thread.
pub(crate) fn init_platform() {
    thread_local! {
        static INSTALLED: Cell<bool> = const { Cell::new(false) };
    }
    if !INSTALLED.get() {
        i_slint_backend_testing::init_no_event_loop();
        INSTALLED.set(true);
    }
}

/// What: `settle` lets Slint run its pending `changed` handlers and timers, as the event loop does before a frame.
pub(crate) fn settle() {
    slint::platform::update_timers_and_animations();
}

/// What: `advance` moves the mock clock forward in 16 ms frames, running the timers after each, as a window does.
/// Why: a timer fires once per update however far the clock jumped, so one big jump would count as one frame.
pub(crate) fn advance(milliseconds: u64) {
    let mut remaining = milliseconds;
    while remaining > 0 {
        let step = remaining.min(16);
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(step));
        CLOCK_MS.set(CLOCK_MS.get() + step);
        crate::ui_hints::set_test_clock_ms(CLOCK_MS.get());
        settle();
        remaining -= step;
    }
}

/// What: `fresh_window_at` builds a window with the host-only callbacks wired to a recording host.
/// Why: the app wires the same callbacks to the engine; the session file and the engine are never touched.
pub(crate) fn fresh_window_at(width: f32, height: f32) -> (AppWindow, Rc<RecordingHost>) {
    init_platform();
    crate::ui_hints::set_test_clock_ms(CLOCK_MS.get());
    update_state(|state| {
        *state = ScreenState::default();
    });
    crate::ui_settings::replace_saver(|_track_rows, _playing| return Ok(()));
    crate::ui_settings::replace_auto_play_saver(|_enabled| return Ok(()));
    crate::ui_text_size::replace_saver(|_sizes| return Ok(()));
    let app = AppWindow::new().expect("the window should build on the testing backend");
    app.window().set_size(slint::LogicalSize::new(width, height));
    app.global::<crate::Theme>().set_scheme_override(1);
    let host = Rc::new(RecordingHost::default());
    let as_host: Rc<dyn Host> = Rc::clone(&host) as Rc<dyn Host>;
    crate::ui_wire::wire_host_callbacks(&app, &as_host);
    crate::ui_settings::wire_callbacks(&app);
    crate::ui_search::wire_route(&app);
    crate::ui_search::wire_open(&app, Rc::clone(&as_host));
    crate::ui_menu::wire_request(&app);
    app.show().expect("the window should show on the testing backend");
    app.window().dispatch_event(WindowEvent::WindowActiveChanged(true));
    project(&app);
    settle();
    return (app, host);
}

/// What: `key_text` gives the text a platform backend would report for a named key or a character.
/// Why: Slint reports special keys as private characters.
fn key_text(key: &str) -> SharedString {
    return match key {
        "Up" => Key::UpArrow.into(),
        "Down" => Key::DownArrow.into(),
        "Left" => Key::LeftArrow.into(),
        "Right" => Key::RightArrow.into(),
        "Home" => Key::Home.into(),
        "End" => Key::End.into(),
        "PageUp" => Key::PageUp.into(),
        "PageDown" => Key::PageDown.into(),
        "Enter" => Key::Return.into(),
        "Escape" => Key::Escape.into(),
        "Delete" => Key::Delete.into(),
        "Backspace" => Key::Backspace.into(),
        "Tab" => Key::Tab.into(),
        "Backtab" => Key::Backtab.into(),
        "F5" => Key::F5.into(),
        "F10" => Key::F10.into(),
        "Menu" => Key::Menu.into(),
        "Ctrl" => Key::Control.into(),
        "Shift" => Key::Shift.into(),
        "Alt" => Key::Alt.into(),
        "Meta" => Key::Meta.into(),
        other => other.into(),
    };
}

/// What: `key_down` sends one key press.
pub(crate) fn key_down(app: &AppWindow, key: &str) {
    app.window().dispatch_event(WindowEvent::KeyPressed { text: key_text(key) });
    settle();
}

/// What: `key_up` sends one key release.
pub(crate) fn key_up(app: &AppWindow, key: &str) {
    app.window().dispatch_event(WindowEvent::KeyReleased { text: key_text(key) });
    settle();
}

/// What: `tap` presses and releases one key.
pub(crate) fn tap(app: &AppWindow, key: &str) {
    key_down(app, key);
    key_up(app, key);
}

/// What: `chord` holds the modifiers, taps the key, and releases the modifiers, as a person does.
/// Why: Slint tracks the modifiers from the modifier key events, so a chord must press them first.
pub(crate) fn chord(app: &AppWindow, modifiers: &[&str], key: &str) {
    for modifier in modifiers {
        key_down(app, modifier);
    }
    tap(app, key);
    for modifier in modifiers.iter().rev() {
        key_up(app, modifier);
    }
}

/// What: `LIBRARY` is the queue the window tests load: four folders, the first with three tracks.
/// Why: lists, folder steps and the row keys need more than one folder and more than one track.
pub(crate) const LIBRARY: [&str; 8] = [
    "Abba/01 Waterloo.mp3",
    "Abba/02 Fernando.mp3",
    "Abba/03 Chiquitita.mp3",
    "Beck/01 Loser.mp3",
    "Beck/02 Devil's Haircut.mp3",
    "Cher/01 Believe.mp3",
    "Cher/02 Strong Enough.mp3",
    "Zed/01 Last.mp3",
];

/// What: `load_library` gives the window the queue, a playing track (the second of the first folder), a duration and a
/// position, as the engine's updates would.
/// Why: the keys act on the library and on the playing track, and the updates are the production way to set them.
pub(crate) fn load_library(app: &AppWindow) {
    use music_player::command::Update;
    let names: Vec<String> = LIBRARY.iter().map(|name| return (*name).to_owned()).collect();
    let paths: Vec<std::path::PathBuf> = LIBRARY.iter().map(|name| return std::path::PathBuf::from("/nonexistent-library").join(name)).collect();
    crate::ui_screen::apply_update(app, &Update::Library(paths));
    crate::ui_screen::apply_update(app, &Update::Queue(names));
    crate::ui_screen::apply_update(
        app,
        &Update::NowPlaying { index: Some(1), name: LIBRARY[1].to_owned(), duration: 200.0 },
    );
    crate::ui_screen::apply_update(app, &Update::Position(100.0));
    settle();
}

/// What: `focus_label` reads the name of the element that holds keyboard focus.
/// Why: the tests assert where the focus is by name.
pub(crate) fn focus_label(app: &AppWindow) -> String {
    return app.global::<crate::Focus>().get_label().to_string();
}

/// What: `tab_labels` presses Tab `count` times and lists the focus label after each press.
/// Why: the focus order is the sequence of labels.
pub(crate) fn tab_labels(app: &AppWindow, count: usize) -> Vec<String> {
    return (0..count)
        .map(|_| {
            tap(app, "Tab");
            return focus_label(app);
        })
        .collect();
}
