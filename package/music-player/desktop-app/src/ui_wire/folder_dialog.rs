//! The folder chooser the Open button starts,
//!  and the two messages its outcome may need.
//!
//! What:
//! `open_folder_dialog` asks the desktop for a folder on a worker thread,
//!  opens the chosen one without autoplay,
//!  and when there is no chooser or the chooser failed,
//!  tells the user in the main window.
//! Why:
//! the chooser blocks,
//!  so it cannot run on the event-loop thread,
//!  and only that thread may touch the window;
//! keeping the three functions together shows why the messages are posted back rather
//! than shown where they happen.

/// What:
///      `use super::MAIN_WINDOW;`.
///  The window the two failure messages go to,
///  recorded by `wire` when the callbacks are registered.
/// Why:
///       The chooser runs off the UI thread and has no window handle of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { mainWindow } from "./ui_wire";
/// ```
use super::MAIN_WINDOW;
/// What:
///      `use music_player::command::Command;`.
///  The UI-to-engine message enum,
///  whose `OpenRoot` variant opens the chosen folder.
/// Why:
///       The dialog's only success path is one command to the engine.
///       The enum lives in the library crate,
///       which is why the path names the crate instead of `crate::`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command } from "music-player/command";
/// ```
use music_player::command::Command;
/// What:
/// `show_missing_chooser_message` tells the user in the main window that no folder chooser is installed.
/// Why:
/// the chooser check runs on a worker thread,
/// and only the event-loop thread may touch the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function showMissingChooserMessage(): void { mainWindow?.showError(NO_DIALOG_MESSAGE); }
/// ```
fn show_missing_chooser_message() {
    let main_window = MAIN_WINDOW
        .with(std::cell::RefCell::clone)
        .into_inner()
        .as_ref()
        .and_then(slint::Weak::upgrade);
    if let Some(window) = main_window {
        crate::ui_overlays::show_error(&window, music_player::dialog_backend::NO_DIALOG_MESSAGE);
    }
}

/// What:
/// `show_chooser_failure_message` tells the user in the main window what the folder chooser said when it failed.
/// Why:
/// the chooser runs on a worker thread,
/// and only the event-loop thread may touch the window;
/// the message itself is built on that thread because it is plain text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function showChooserFailureMessage(message: string): void { mainWindow?.showError(message); }
/// ```
fn show_chooser_failure_message(message: String) {
    let main_window = MAIN_WINDOW
        .with(std::cell::RefCell::clone)
        .into_inner()
        .as_ref()
        .and_then(slint::Weak::upgrade);
    if let Some(window) = main_window {
        crate::ui_overlays::show_error(&window, &message);
    }
}

/// What:
/// `open_folder_dialog` asks for a folder on a worker thread and opens it without autoplay.
/// Why:
/// the chooser blocks,
/// so it runs off the event-loop thread;
/// the engine receives the chosen root.
/// What happened is decided in `src/folder_chooser.rs`,
/// so a cancel,
/// a missing chooser,
/// and a chooser that failed are no longer the same silence
/// (issue 1010).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openFolderDialog(sender): void { spawn(() => { switch (advice(chooseFolder())) { case open: sender.send({kind: 'openRoot', root, select: undefined, play: false}); break; case nothing: break; case tellNoChooser: showMissingChooserMessage(); break; case tellFailure: showChooserFailureMessage(message); } }); }
/// ```
pub(crate) fn open_folder_dialog(sender: music_player::engine::CommandSender) {
    std::thread::spawn(move || {
        match music_player::folder_chooser::advice(music_player::folder_chooser::choose_folder()) {
            music_player::folder_chooser::ChooserAdvice::Open {
                folder,
            } => sender.send(Command::OpenRoot {
                root: folder,
                select: None,
                play: false,
            }),
            // A cancel closes the window the user just used, so it says nothing.
            music_player::folder_chooser::ChooserAdvice::Nothing => {}
            music_player::folder_chooser::ChooserAdvice::TellNoChooser => {
                tracing::warn!("no folder chooser is available; the user is told how to proceed");
                let _ = slint::invoke_from_event_loop(show_missing_chooser_message);
            }
            music_player::folder_chooser::ChooserAdvice::TellFailure {
                message,
            } => {
                tracing::error!(%message, "the user is told why the folder chooser failed");
                let _ =
                    slint::invoke_from_event_loop(move || show_chooser_failure_message(message));
            }
        }
    });
}
