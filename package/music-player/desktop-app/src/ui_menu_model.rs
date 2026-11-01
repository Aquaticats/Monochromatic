//! Pure track menu model: the rows, icons and dividers for one track.
//!
//! What: `menu_items` builds the menu rows from the shared action list in `track_menu`.
//! Why: the action order, the grouping and the destructive row come from the Rust model; the window
//! only draws them.

use music_player::keymap::{KeyboardPlatform, ShortcutAction};
use music_player::keymap_bindings::shortcut_text;
use music_player::track_menu::{TrackMenuAction, access_label, label, track_menu_actions_for, track_menu_divider_flags};

/// What: `MenuItemView` is one menu row as plain values.
/// Why: the window's `TrackMenuItem` struct holds Slint strings, which the pure model does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MenuItemView = { label: string; icon: number; destructive: boolean; dividerBefore: boolean; value: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItemView {
    /// Visible label of the action.
    pub label: String,
    /// The part of the label before its access letter.
    pub label_before: String,
    /// The access letter as written in the label; the window draws it underlined.
    pub access: String,
    /// The part of the label after its access letter.
    pub label_after: String,
    /// Glyph index in the menu's icon order.
    pub icon: i32,
    /// True for the destructive action.
    pub destructive: bool,
    /// True when a divider sits above this row.
    pub divider_before: bool,
    /// Inline value after the label; only the details row carries one.
    pub value: String,
    /// Shortcut of the action on the running platform, drawn right-aligned; empty for an action with none.
    pub shortcut: String,
}

/// What: `icon_index` gives the glyph index the menu component draws for an action.
/// Why: the menu's icon order is fixed: play, shuffle, details, show, copy, trash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const iconIndex = (action: TrackMenuAction): number => ICON_ORDER.indexOf(action);
/// ```
pub fn icon_index(action: TrackMenuAction) -> i32 {
    return match action {
        TrackMenuAction::Play => 0,
        TrackMenuAction::ShuffleFromHere => 1,
        TrackMenuAction::Details => 2,
        TrackMenuAction::ShowInFileManager => 3,
        TrackMenuAction::CopyFilename => 4,
        TrackMenuAction::MoveToTrash => 5,
    };
}

/// What: `shortcut_action_of` gives the shortcut action whose chord a menu row shows.
/// Why: D106 shows each action's shortcut in its menu row; Play is Enter on the focused row, and Start shuffle from here and
/// File details have no shortcut (D114).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shortcutActionOf = (action: TrackMenuAction): ShortcutAction | null => ...;
/// ```
pub fn shortcut_action_of(action: TrackMenuAction) -> Option<ShortcutAction> {
    return match action {
        TrackMenuAction::Play => Some(ShortcutAction::ListActivate),
        TrackMenuAction::ShuffleFromHere | TrackMenuAction::Details => None,
        TrackMenuAction::ShowInFileManager => Some(ShortcutAction::RowShow),
        TrackMenuAction::CopyFilename => Some(ShortcutAction::RowCopy),
        TrackMenuAction::MoveToTrash => Some(ShortcutAction::RowTrash),
    };
}

/// What: `menu_items` builds every menu row, with the file size as the details value and each action's shortcut.
/// Why: the details row shows its value inline (D7), the value is read when the menu opens, and every row shows its
/// shortcut (D106).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const menuItems = (size: string | null, platform: KeyboardPlatform): MenuItemView[] => { ... };
/// ```
pub fn menu_items(size_text: Option<&str>, platform: KeyboardPlatform) -> Vec<MenuItemView> {
    let actions = track_menu_actions_for();
    let dividers = track_menu_divider_flags(actions);
    return actions
        .iter()
        .zip(dividers)
        .map(|(action, divider)| return MenuItemView {
            label: label(*action).to_string(),
            label_before: access_label(*action).0,
            access: access_label(*action).1,
            label_after: access_label(*action).2,
            icon: icon_index(*action),
            destructive: action.destructive(),
            divider_before: divider,
            value: match action {
                TrackMenuAction::Details => size_text.unwrap_or("").to_string(),
                _ => String::new(),
            },
            shortcut: shortcut_action_of(*action).map_or_else(String::new, |chord| return shortcut_text(chord, platform)),
        })
        .collect();
}

/// What: `short_folder` shortens a folder path to its last two folders, with a leading ellipsis when folders were dropped.
/// Why: a message about a file names where it sits, but the whole path of a deep library does not fit a toast; the full path
/// stays in the log and one click away in the file manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shortFolder = (folder: string) => { const names = splitPath(folder); return names.length > 2 ? "\u2026/" + names.slice(-2).join("/") : names.join("/"); };
/// ```
pub fn short_folder(folder: &str) -> String {
    let mut names: Vec<String> = Vec::new();
    for part in std::path::Path::new(folder).components() {
        if let std::path::Component::Normal(name) = part {
            names.push(name.to_string_lossy().into_owned());
        }
    }
    let separator = std::path::MAIN_SEPARATOR.to_string();
    if names.is_empty() {
        return folder.to_string();
    }
    if names.len() <= 2 {
        return names.join(&separator);
    }
    return format!("\u{2026}{separator}{}", names[names.len() - 2..].join(&separator));
}

/// What: `details_text` writes the File details message: the file name, its size and the folder it sits in.
/// Why: the message has to fit a toast of two lines (D83), so the folder is shortened to its last two folders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const detailsText = (file: string, size: string | null, folder: string) => `${file} \u00b7 ${size ?? "size unknown"} \u00b7 ${shortFolder(folder)}`;
/// ```
pub fn details_text(file_name: &str, size: Option<&str>, folder: &str) -> String {
    return format!("{file_name} \u{b7} {} \u{b7} {}", size.unwrap_or("size unknown"), short_folder(folder));
}

/// What: `copied_text` writes the message shown after Copy filename worked.
/// Why: the clipboard changes where the user cannot see it, so the action says what it put there.
pub fn copied_text(file_name: &str) -> String {
    return format!("Copied {file_name} to the clipboard");
}

/// What: `shown_in_file_manager_text` writes the message shown after Show in file manager started the file manager.
/// Why: the file manager opens as its own window that may land behind this one, so the action confirms that it ran.
pub fn shown_in_file_manager_text(file_name: &str) -> String {
    return format!("Opened the folder of {file_name} in your file manager");
}

/// What: `human_size` formats a byte count with one decimal in KB, MB or GB.
/// Why: the details row shows a size a person can read at a glance.
pub fn human_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    let value = bytes as f64;
    if value >= KB * KB * KB {
        return format!("{:.1} GB", value / (KB * KB * KB));
    }
    if value >= KB * KB {
        return format!("{:.1} MB", value / (KB * KB));
    }
    return format!("{:.1} KB", value / KB);
}

#[cfg(test)]
#[path = "ui_menu_model_tests.rs"]
mod tests;
