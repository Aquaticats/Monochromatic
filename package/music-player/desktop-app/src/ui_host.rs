//! The host of keyboard actions: what a shortcut can ask of the application beyond the window.
//!
//! What: `Host` names the four things a key can ask for that leave the window: an engine command, the system folder
//! dialog, quitting, and a file action on a track row. `Engine` is the production host.
//! Why: the window tests press every shortcut through the real key handler and a recording host, so a shortcut that
//! would play audio, open a dialog, trash a file or quit the process can still be checked for the exact request it
//! made, and the end-to-end run checks the real effect.

use music_player::command::Command;
use music_player::engine::Engine;
use music_player::track_menu::TrackMenuAction;

use crate::AppWindow;

/// What: `Host` is what a keyboard action may ask of the application.
/// Why: see the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Host { send(command: Command): void; pickFolder(): void; quit(): void; runRowAction(window: AppWindow, action: TrackMenuAction): void }
/// ```
pub(crate) trait Host {
    /// What: `send` hands a command to the playback engine.
    /// Why: transport, mode, scope, seek and rescan keys all end as engine commands.
    fn send(&self, command: Command);

    /// What: `pick_folder` opens the system folder dialog.
    /// Why: Ctrl+Shift+O opens the same dialog as the Open pill.
    fn pick_folder(&self);

    /// What: `quit` ends the application.
    /// Why: Ctrl+Q quits like closing the window.
    fn quit(&self);

    /// What: `run_row_action` performs one track menu action on the row the menu target names.
    /// Why: the row chords (Delete, copy, show) and the open menu's Enter share the menu's actions.
    fn run_row_action(&self, window: &AppWindow, action: TrackMenuAction);
}

/// The production host.
impl Host for Engine {
    /// What: `send` forwards to the engine's own `send`.
    /// Why: the trait method and the inherent method have the same name; the inherent one is meant.
    fn send(&self, command: Command) {
        Engine::send(self, command);
    }

    /// What: `pick_folder` runs the dialog on a worker thread and opens the chosen folder.
    /// Why: the native dialog blocks, and the Open pill already does exactly this.
    fn pick_folder(&self) {
        tracing::info!("the system folder dialog was requested from the keyboard");
        crate::ui_wire::open_folder_dialog(self.sender());
    }

    /// What: `quit` stops the Slint event loop, which ends `main`'s run and shuts the engine down.
    /// Why: closing the window ends the program the same way.
    fn quit(&self) {
        tracing::info!("quit requested from the keyboard");
        if let Err(error) = slint::quit_event_loop() {
            tracing::warn!(?error, "the event loop could not be stopped");
        }
    }

    /// What: `run_row_action` runs the menu action for the row the screen state names.
    /// Why: it is the same function the menu's mouse path uses.
    fn run_row_action(&self, window: &AppWindow, action: TrackMenuAction) {
        crate::ui_menu::run_action(window, self, action);
    }
}
