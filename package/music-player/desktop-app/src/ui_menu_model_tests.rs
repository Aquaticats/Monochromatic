//! Tests for the pure track menu model in `ui_menu_model`.

use super::*;

#[test]
fn menu_has_the_six_actions_in_order_with_their_icons() {
    let items = menu_items(None, KeyboardPlatform::WindowsLinux);
    let icons: Vec<i32> = items.iter().map(|item| return item.icon).collect();
    assert_eq!(icons, vec![0, 1, 2, 3, 4, 5]);
    assert_eq!(items[0].label, "Play");
    assert_eq!(items[5].label, "Move to trash");
}

#[test]
fn only_move_to_trash_is_destructive() {
    let items = menu_items(None, KeyboardPlatform::WindowsLinux);
    let destructive: Vec<&str> = items
        .iter()
        .filter(|item| return item.destructive)
        .map(|item| return item.label.as_str())
        .collect();
    assert_eq!(destructive, vec!["Move to trash"]);
}

#[test]
fn details_row_carries_the_size_value_and_no_other_row_does() {
    let items = menu_items(Some("4.2 MB"), KeyboardPlatform::WindowsLinux);
    assert_eq!(items[2].value, "4.2 MB");
    let others: Vec<&str> = items
        .iter()
        .enumerate()
        .filter(|(index, _)| return *index != 2)
        .map(|(_, item)| return item.value.as_str())
        .collect();
    assert!(others.iter().all(|value| return value.is_empty()));
}

#[test]
fn human_size_uses_kb_mb_and_gb_steps() {
    assert_eq!(human_size(512), "0.5 KB");
    assert_eq!(human_size(3 * 1024 * 1024 + 512 * 1024), "3.5 MB");
    assert_eq!(human_size(2 * 1024 * 1024 * 1024), "2.0 GB");
}

#[test]
fn every_row_but_the_shuffle_row_shows_its_shortcut_on_windows_and_linux() {
    let items = menu_items(None, KeyboardPlatform::WindowsLinux);
    let shortcuts: Vec<&str> = items.iter().map(|item| return item.shortcut.as_str()).collect();
    assert_eq!(shortcuts, vec!["Enter", "", "", "Ctrl+Shift+E", "Ctrl+Shift+C", "Delete"]);
}

#[test]
fn the_shortcuts_use_the_mac_glyphs_on_the_mac_map() {
    let items = menu_items(None, KeyboardPlatform::Mac);
    assert_eq!(items[4].shortcut, "\u{2318}\u{21E7}C");
    assert_eq!(items[5].shortcut, "Delete");
}

#[test]
fn a_long_folder_is_shortened_to_its_last_two_folders() {
    let sep = std::path::MAIN_SEPARATOR;
    let deep = ["", "var", "mnt", "encrypted", "temp", "agent", "agent-polish2", "work", "lib-main", "Camellia"].join(&sep.to_string());
    assert_eq!(short_folder(&deep), format!("\u{2026}{sep}lib-main{sep}Camellia"));
    assert_eq!(short_folder(&["", "music", "Camellia"].join(&sep.to_string())), format!("music{sep}Camellia"), "two folders need no ellipsis");
    assert_eq!(short_folder("Camellia"), "Camellia");
    assert_eq!(short_folder(""), "", "an empty folder stays empty");
}

#[test]
fn the_details_message_is_short_enough_for_a_toast_whatever_the_folder_depth() {
    let sep = std::path::MAIN_SEPARATOR.to_string();
    let folder = ["", "var", "mnt", "encrypted", "temp", "agent", "agent-polish2", "work", "lib-main", "Camellia"].join(&sep);
    let message = details_text("04 Quo Vadis.wav", Some("1.3 MB"), &folder);
    assert_eq!(message, format!("04 Quo Vadis.wav \u{b7} 1.3 MB \u{b7} \u{2026}{sep}lib-main{sep}Camellia"));
    assert!(message.chars().count() < 60, "{message}");
    assert_eq!(details_text("a.wav", None, "Camellia"), "a.wav \u{b7} size unknown \u{b7} Camellia");
}

#[test]
fn the_menu_actions_that_change_nothing_visible_confirm_in_plain_words() {
    assert_eq!(copied_text("04 Quo Vadis.wav"), "Copied 04 Quo Vadis.wav to the clipboard");
    assert_eq!(shown_in_file_manager_text("04 Quo Vadis.wav"), "Opened the folder of 04 Quo Vadis.wav in your file manager");
}
