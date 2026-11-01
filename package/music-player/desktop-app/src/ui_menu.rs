//! Window binding for the track menu: opens it for a row, and runs the chosen action.
//!
//! What: `wire` registers the menu request and selection callbacks; `run_action` performs one action.
//! Why: the menu's actions map onto engine commands, platform file actions, and feedback messages in one place.

use std::path::Path;
use std::rc::Rc;

use music_player::command::Command;
use music_player::engine::Engine;
use music_player::track_entries::track_title_for;
use music_player::track_menu::{TrackMenuAction, track_menu_actions_for};
use slint::ComponentHandle;

use crate::ui_files::{FileActionError, can_restore_from_trash, copy_text, move_to_trash, show_in_file_manager};
use crate::ui_host::Host;
use crate::ui_menu_model::{copied_text, details_text, human_size, menu_items, shown_in_file_manager_text};
use crate::ui_overlays::{show_error, show_file_error, show_long_notice, show_notice, show_trash_notice, show_undo};
use crate::ui_screen::{model_of, read_state, update_state};
use crate::ui_screen_view::selected_page_scope;
use crate::{AppWindow, TrackMenuItem};

/// What: `wire` registers the track menu callbacks on the window.
/// Why: the list reports a menu request for a row, and the menu reports the chosen row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wireMenu(app: AppWindow, engine: Engine): void { ... };
/// ```
pub(crate) fn wire(app: &AppWindow, engine: &Rc<Engine>) {
    wire_request(app);
    let select_weak = app.as_weak();
    let select_engine = engine.clone();
    app.on_menu_select(move |position| {
        if let Some(window) = select_weak.upgrade() {
            let chosen = usize::try_from(position).ok().and_then(|at| return track_menu_actions_for().get(at).copied());
            if let Some(action) = chosen {
                run_action(&window, select_engine.as_ref(), action);
            }
        }
    });
}

/// What: `wire_request` registers the menu request and dismiss callbacks, which need no engine.
/// Why: the window tests open the menu from the keyboard and need the same request path without an engine.
pub(crate) fn wire_request(app: &AppWindow) {
    let request_weak = app.as_weak();
    app.on_menu_request(move |index| {
        if let Some(window) = request_weak.upgrade() {
            fill_menu(&window, usize::try_from(index).ok());
        }
    });
    app.on_menu_dismiss(|| {
        tracing::debug!("track menu dismissed without an action");
        update_state(|state| {
            state.menu_ring = None;
        });
    });
}

/// What: `fill_menu` writes the title and rows of the menu for the library row `index`.
/// Why: the details row shows the file size, which is read from disk when the menu opens.
fn fill_menu(window: &AppWindow, index: Option<usize>) {
    update_state(|state| {
        state.menu_index = index;
        // A menu the keyboard asked for opens with its first row highlighted; a pointer menu has no highlight until a
        // key moves it.
        state.menu_ring = if state.menu_by_key { Some(0) } else { None };
        state.menu_by_key = false;
    });
    window.set_menu_ring(read_state(|state| return state.menu_ring).and_then(|row| return i32::try_from(row).ok()).unwrap_or(-1));
    let target = read_state(|state| return state.menu_target());
    let Some((_index, name, path)) = target else {
        return;
    };
    let size = std::fs::metadata(&path).ok().map(|meta| return human_size(meta.len()));
    let rows: Vec<TrackMenuItem> = menu_items(size.as_deref(), crate::ui_keys::platform())
        .into_iter()
        .map(|row| return TrackMenuItem {
            label: row.label.as_str().into(),
            label_before: row.label_before.as_str().into(),
            access: row.access.as_str().into(),
            label_after: row.label_after.as_str().into(),
            icon: row.icon,
            destructive: row.destructive,
            divider_before: row.divider_before,
            value: row.value.as_str().into(),
            shortcut: row.shortcut.as_str().into(),
        })
        .collect();
    window.set_menu_title(track_title_for(&name).as_str().into());
    window.set_menu_items(model_of(rows));
    tracing::debug!("track menu filled");
}

/// What: `FileEffects` is the two actions of the menu that reach outside the window and that the user cannot see happen: the
/// clipboard write and the file manager.
/// Why: both report success only through feedback, so a window test swaps in effects that answer at once and reads the
/// feedback, and no test writes the real clipboard or starts a file manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface FileEffects { copyText(text: string): void; showInFileManager(path: string): void }
/// ```
pub(crate) trait FileEffects {
    /// What: `copy_text` writes text to the clipboard.
    fn copy_text(&self, text: &str) -> Result<(), FileActionError>;

    /// What: `show_in_file_manager` opens the file manager on a track's folder.
    fn show_in_file_manager(&self, path: &Path) -> Result<(), FileActionError>;
}

/// What: `SystemFileEffects` is the production pair of file effects.
/// Why: it forwards to the platform functions of `ui_files`.
struct SystemFileEffects;

/// The production effects.
impl FileEffects for SystemFileEffects {
    /// What: `copy_text` writes to the system clipboard.
    fn copy_text(&self, text: &str) -> Result<(), FileActionError> {
        return copy_text(text);
    }

    /// What: `show_in_file_manager` starts the platform file manager.
    fn show_in_file_manager(&self, path: &Path) -> Result<(), FileActionError> {
        return show_in_file_manager(path);
    }
}

/// What: `run_action` performs one menu action on the track the menu was opened for, with the platform's file effects.
/// Why: the target is read from the state at selection time, so the action uses the row the user saw.
pub(crate) fn run_action(window: &AppWindow, host: &dyn Host, action: TrackMenuAction) {
    run_action_with(window, host, &SystemFileEffects, action);
}

/// What: `run_action_with` performs one menu action with the given file effects.
/// Why: every action answers on screen: a confirmation for the ones that change nothing visible (copy, show) and an
/// error for the ones that failed, and a test checks that with effects that answer at once.
pub(crate) fn run_action_with(window: &AppWindow, host: &dyn Host, effects: &dyn FileEffects, action: TrackMenuAction) {
    let target = read_state(|state| return state.menu_target());
    let Some((index, name, path)) = target else {
        show_error(window, "The track is no longer in the library.");
        return;
    };
    match action {
        TrackMenuAction::Play => crate::ui_wire::play_row_now(host, index),
        TrackMenuAction::ShuffleFromHere => shuffle_from_here(host, index),
        TrackMenuAction::Details => show_details(window, &name, &path),
        TrackMenuAction::ShowInFileManager => match effects.show_in_file_manager(&path) {
            Ok(()) => show_notice(window, &shown_in_file_manager_text(&file_name_of(&name))),
            Err(error) => show_file_error(window, &error),
        },
        TrackMenuAction::CopyFilename => {
            let file_name = file_name_of(&name);
            match effects.copy_text(&file_name) {
                Ok(()) => show_notice(window, &copied_text(&file_name)),
                Err(error) => show_error(window, &error.to_string()),
            }
        }
        TrackMenuAction::MoveToTrash => move_track_to_trash(window, &name, &path),
    }
}

/// What: `show_details` shows the file name, size and folder of a track as a notice.
/// Why: File details has no page of its own; the menu row shows the size inline, and the keyboard shortcut (Alt+Enter)
/// shows the same facts with the folder as a message, so the shortcut has a visible effect.
fn show_details(window: &AppWindow, name: &str, path: &Path) {
    let size = std::fs::metadata(path).ok().map(|meta| return human_size(meta.len()));
    let folder = path.parent().map_or_else(String::new, |parent| return parent.display().to_string());
    let message = details_text(&file_name_of(name), size.as_deref(), &folder);
    tracing::info!(full_folder = %folder, %message, "file details shown");
    show_long_notice(window, &message);
}

/// What: `move_track_to_trash` moves the file to the trash and offers Undo where the platform can restore it.
/// Why: D8 deletes immediately; Undo restores from the OS trash, and macOS offers no Undo because the trash crate cannot restore there.
fn move_track_to_trash(window: &AppWindow, name: &str, path: &Path) {
    match move_to_trash(path) {
        Ok(()) => {
            let message = format!("Moved {} to the trash", file_name_of(name));
            if can_restore_from_trash() {
                show_undo(window, &message, path.to_path_buf());
            } else {
                show_trash_notice(window, &message);
            }
        }
        Err(error) => show_file_error(window, &error),
    }
}

/// What: `shuffle_from_here` shuffles the selected page and starts the track at `index`.
/// Why: Android's shuffle-from-here sets the page scope, then the shuffle mode, then plays the index (D7).
fn shuffle_from_here(host: &dyn Host, index: usize) {
    let scope = read_state(selected_page_scope);
    host.send(Command::SetPageScope(scope));
    host.send(Command::SetPlaybackMode(music_player::command::PlaybackMode::ShufflePage));
    host.send(Command::SelectIndex(index));
    host.send(Command::Play);
}

/// What: `file_name_of` drops the folders from a display path, keeping the file name with its extension.
/// Why: Copy filename copies the file name, not the folder path.
fn file_name_of(display_path: &str) -> String {
    return display_path
        .rsplit_once('/')
        .map_or(display_path, |(_, file_name)| return file_name)
        .to_string();
}
