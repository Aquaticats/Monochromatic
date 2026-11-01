// What:     Unit tests for `keymap.rs`, pulled in by
//           `#[cfg(test)] #[path = "keymap_tests.rs"] mod tests;` at the bottom of `keymap.rs`.
//           The first groups mirror the rows of the Kotlin `KeyboardMapTest` (D102), the later groups mirror the rows of
//           D106, one test per group of the decision's table.
// Why:      Keep the tests beside the code. Each assertion message names the row it mirrors.

// What:     `use super::*;` imports everything from the parent module into the test scope.
// Why:      Tests need the enums, `ShortcutRequest`, `shortcut_action`, and `KeymapError`.
use super::*;

// What:     `fn base_request() -> ShortcutRequest` builds the Windows and Linux Space key with no other state, pressed
//           with nothing focused on the player page.
// Why:      Each row changes only the fields it names, so one shared starting request keeps rows honest.
fn base_request() -> ShortcutRequest {
    return ShortcutRequest {
        platform: KeyboardPlatform::WindowsLinux,
        key: " ".to_owned(),
        ctrl: false,
        meta: false,
        alt: false,
        shift: false,
        repeat: false,
        zone: FocusZone::Surface,
        route: KeyRoute::Player,
        in_popup: false,
        picker_overlay: false,
        composing: false,
    };
}

// What:     `fn pressed(key)` builds a bare press of `key` on the player page with nothing focused.
// Why:      Most D106 rows start from a named key and add modifiers or a zone.
fn pressed(key: &str) -> ShortcutRequest {
    return ShortcutRequest { key: key.to_owned(), ..base_request() };
}

// What:     `fn ctrl(key)` builds a Control press of `key`.
// Why:      Control is the primary modifier on Windows and Linux.
fn ctrl(key: &str) -> ShortcutRequest {
    return ShortcutRequest { ctrl: true, ..pressed(key) };
}

// What:     `fn alt(key)` builds an Alt press of `key`.
// Why:      Alt carries the seek, folder and focus rows.
fn alt(key: &str) -> ShortcutRequest {
    return ShortcutRequest { alt: true, ..pressed(key) };
}

// What:     `fn ctrl_shift(key)` builds a Control plus Shift press of `key`.
// Why:      Control plus Shift carries the mode, dialog and row rows.
fn ctrl_shift(key: &str) -> ShortcutRequest {
    return ShortcutRequest { ctrl: true, shift: true, ..pressed(key) };
}

// What:     `fn in_zone(zone, request)` moves a request to a focus zone.
// Why:      The list, text and control rows differ from the surface only by the zone.
fn in_zone(zone: FocusZone, request: ShortcutRequest) -> ShortcutRequest {
    return ShortcutRequest { zone, ..request };
}

// What:     `fn on_route(route, zone, request)` moves a request to another page with a focus zone.
// Why:      Settings, its editor page and Search each answer their own keys.
fn on_route(route: KeyRoute, zone: FocusZone, request: ShortcutRequest) -> ShortcutRequest {
    return ShortcutRequest { route, zone, ..request };
}

// What:     `fn in_menu(request)` marks a request as pressed while the track menu is open.
// Why:      The open menu answers a few keys and keeps the rest.
fn in_menu(request: ShortcutRequest) -> ShortcutRequest {
    return ShortcutRequest { in_popup: true, ..request };
}

// What:     `fn on_mac(request)` moves a request to the macOS map.
// Why:      The macOS rows read Command and Option.
fn on_mac(request: ShortcutRequest) -> ShortcutRequest {
    return ShortcutRequest { platform: KeyboardPlatform::Mac, ..request };
}

// What:     `fn assert_action(row, expected, request)` classifies one request and compares the result.
// Why:      One helper keeps each row to a single line, and the row text appears in any failure message.
fn assert_action(row: &str, expected: ShortcutAction, request: &ShortcutRequest) {
    assert_eq!(shortcut_action(request), expected, "{row}");
}

// What:     `fn assert_rejects_platform(name)` checks that the parser rejects one name with the prototype's message.
// Why:      The prototype throws for unknown platforms, and the enum parser must reject them with the same text.
fn assert_rejects_platform(name: &str) {
    let error = KeyboardPlatform::from_name(name).expect_err("unknown platform names must be rejected");
    assert_eq!(
        error.to_string(),
        "Unknown keyboard-map platform.",
        "message for platform name '{name}'"
    );
}

// What:     `#[test] fn guard_keeps_other_pages_keys_unbound()` checks that the player's keys do not fire on Settings.
// Why:      Settings is a modal page; Space and the transport keys are the player page's.
#[test]
fn guard_keeps_other_pages_keys_unbound() {
    assert_action(
        "guard 1: Space on the Settings page with nothing focused stays unbound",
        ShortcutAction::Unbound,
        &on_route(KeyRoute::Settings, FocusZone::Surface, base_request()),
    );
}

// What:     `#[test] fn guard_keeps_composition_keys_with_their_owner()` checks the composition guard.
// Why:      A composing key stays with its owner.
#[test]
fn guard_keeps_composition_keys_with_their_owner() {
    assert_action(
        "guard 2: composing Ctrl+F stays with its owner",
        ShortcutAction::Owner,
        &ShortcutRequest { composing: true, ..ctrl("f") },
    );
}

// What:     `#[test] fn guard_keeps_popup_keys_with_their_owner()` checks the popup guard.
// Why:      A key pressed while a popup is open stays with the popup.
#[test]
fn guard_keeps_popup_keys_with_their_owner() {
    assert_action("guard 3: popup key stays with its owner", ShortcutAction::Owner, &in_menu(base_request()));
}

// What:     `#[test] fn common_player_rows_classify_as_expected()` checks toggle, owner, repeat, and unbound arrows.
// Why:      These are the common player and control keys.
#[test]
fn common_player_rows_classify_as_expected() {
    assert_action("row 1: Space toggles", ShortcutAction::Toggle, &base_request());
    assert_action(
        "row 2: editor keeps its key",
        ShortcutAction::Owner,
        &in_zone(FocusZone::TextField, base_request()),
    );
    assert_action(
        "row 3: control keeps its key",
        ShortcutAction::Owner,
        &in_zone(FocusZone::Button, base_request()),
    );
    assert_action(
        "row 4: repeated Space is ignored",
        ShortcutAction::IgnoreRepeat,
        &ShortcutRequest { repeat: true, ..base_request() },
    );
    assert_action("row 11: bare ArrowDown is unbound", ShortcutAction::Unbound, &pressed("ArrowDown"));
}

// What:     `#[test] fn windows_linux_rows_classify_as_expected()` checks the Windows and Linux shortcuts.
// Why:      Search, picker, Settings, previous and next, reveal, and mode each have one chord on this map.
#[test]
fn windows_linux_rows_classify_as_expected() {
    assert_action("row 5: Ctrl+F opens Search", ShortcutAction::Search, &ctrl("f"));
    assert_action("row 6: Ctrl+O opens picker", ShortcutAction::Picker, &ctrl("o"));
    assert_action(
        "row 7: Ctrl+Alt+S opens Settings",
        ShortcutAction::Settings,
        &ShortcutRequest { alt: true, ..ctrl("s") },
    );
    assert_action("row 8: Ctrl+ArrowLeft goes to previous", ShortcutAction::Previous, &ctrl("ArrowLeft"));
    assert_action("row 9: Ctrl+ArrowRight goes to next", ShortcutAction::Next, &ctrl("ArrowRight"));
    assert_action(
        "row 10: editor keeps Ctrl+ArrowLeft",
        ShortcutAction::Owner,
        &in_zone(FocusZone::TextField, ctrl("ArrowLeft")),
    );
    assert_action("row 22: Ctrl+G reveals current track", ShortcutAction::Reveal, &ctrl("g"));
    assert_action("row 23: Ctrl+M cycles mode", ShortcutAction::Mode, &ctrl("m"));
}

// What:     `#[test] fn mac_rows_classify_as_expected()` checks the macOS shortcuts.
// Why:      Command replaces Control on this map, and Control plus arrows is unbound.
#[test]
fn mac_rows_classify_as_expected() {
    let command = |key: &str| return ShortcutRequest { meta: true, ..on_mac(pressed(key)) };
    assert_action(
        "row 12: mac Control+ArrowRight is unbound",
        ShortcutAction::Unbound,
        &ShortcutRequest { ctrl: true, ..on_mac(pressed("ArrowRight")) },
    );
    assert_action("row 13: mac Command+F opens Search", ShortcutAction::Search, &command("f"));
    assert_action("row 14: mac Command+O opens picker", ShortcutAction::Picker, &command("o"));
    assert_action("row 15: mac Command+Comma opens Settings", ShortcutAction::Settings, &command(","));
    assert_action("row 16: mac Command+L reveals current track", ShortcutAction::Reveal, &command("l"));
    assert_action(
        "row 17: mac Command+Shift+[ goes to previous",
        ShortcutAction::Previous,
        &ShortcutRequest { shift: true, ..command("[") },
    );
    assert_action(
        "row 18: mac Command+Shift+] goes to next",
        ShortcutAction::Next,
        &ShortcutRequest { shift: true, ..command("]") },
    );
    assert_action("row 24: mac Command+M is unbound", ShortcutAction::Unbound, &command("m"));
}

// What:     `#[test] fn popup_search_and_unbound_rows_classify_as_expected()` checks popup, Search, and unbound rows.
// Why:      Mixed-modifier letter combinations must stay unbound, and Escape has popup and Search meanings.
#[test]
fn popup_search_and_unbound_rows_classify_as_expected() {
    assert_action("row 19: Escape closes popup", ShortcutAction::ClosePopup, &in_menu(pressed("Escape")));
    assert_action("row 20: Ctrl+F in popup stays with the popup", ShortcutAction::Owner, &in_menu(ctrl("f")));
    assert_action(
        "row 21: Escape in Search from the query field returns back",
        ShortcutAction::Back,
        &on_route(KeyRoute::Search, FocusZone::TextField, pressed("Escape")),
    );
    assert_action(
        "row 25: Ctrl+Command+F is unbound",
        ShortcutAction::Unbound,
        &ShortcutRequest { meta: true, ..ctrl("f") },
    );
    assert_action(
        "row 26: Ctrl+Shift+F is unbound",
        ShortcutAction::Unbound,
        &ShortcutRequest { shift: true, ..ctrl("f") },
    );
}

// What:     `#[test] fn parses_stored_platform_names_to_their_maps()` checks the two stored names.
// Why:      The prototype's platform names must parse to the matching enum entries.
#[test]
fn parses_stored_platform_names_to_their_maps() {
    assert_eq!(
        KeyboardPlatform::from_name("windows-linux"),
        Ok(KeyboardPlatform::WindowsLinux),
        "platform name windows-linux"
    );
    assert_eq!(KeyboardPlatform::from_name("mac"), Ok(KeyboardPlatform::Mac), "platform name mac");
}

// What:     `#[test] fn rejects_unknown_platform_names_with_the_prototype_message()` checks rejection.
// Why:      Unknown, differently cased, and empty platform names must all fail with the prototype's message.
#[test]
fn rejects_unknown_platform_names_with_the_prototype_message() {
    assert_rejects_platform("unknown");
    assert_rejects_platform("Mac");
    assert_rejects_platform("");
}

// What:     `#[test] fn wire_names_match_the_prototype_spelling()` checks the action text.
// Why:      Extra coverage for `wire_name`, which no Kotlin test reaches directly.
#[test]
fn wire_names_match_the_prototype_spelling() {
    assert_eq!(ShortcutAction::IgnoreRepeat.wire_name(), "ignore-repeat");
    assert_eq!(ShortcutAction::ClosePopup.wire_name(), "close-popup");
    assert_eq!(ShortcutAction::Toggle.wire_name(), "toggle");
    assert_eq!(ShortcutAction::ListLetter('a').wire_name(), "list-letter");
    assert_eq!(ShortcutAction::SeekBackLong.wire_name(), "seek-back-long");
    assert_eq!(KeyboardPlatform::Mac.stored_name(), "mac");
}

// What:     `#[test] fn zones_map_to_the_owner_that_keeps_their_keys()` checks the zone to owner table and the playback area.
// Why:      D102 gives text fields their own keys and limits transport chords to the playback area.
#[test]
fn zones_map_to_the_owner_that_keeps_their_keys() {
    assert_eq!(FocusZone::TextField.owner(), ShortcutOwner::Editor);
    assert_eq!(FocusZone::DeckButton.owner(), ShortcutOwner::Control);
    assert_eq!(FocusZone::TrackList.owner(), ShortcutOwner::Player);
    assert!(FocusZone::DeckButton.in_playback_area());
    assert!(FocusZone::Seek.in_playback_area());
    assert!(!FocusZone::Button.in_playback_area());
    assert!(!FocusZone::TextField.in_playback_area());
    assert_eq!(FocusZone::from_int(2), FocusZone::TrackList);
    assert_eq!(FocusZone::from_int(7), FocusZone::TextField);
    assert_eq!(FocusZone::from_int(99), FocusZone::Surface);
}

// ---- D106 rows ----

// What:     `#[test] fn d106_playback_rows_classify_as_expected()` checks seek, mode picks, restart and the kept keys.
// Why:      D106 Playback: Alt+Left and Alt+Right seek 10 s, Shift makes it 60 s, Ctrl+Shift+1 to 4 pick a mode, Ctrl+Home restarts.
#[test]
fn d106_playback_rows_classify_as_expected() {
    let alt_shift = |key: &str| return ShortcutRequest { shift: true, ..alt(key) };
    assert_action("playback: Alt+Left seeks back 10 s", ShortcutAction::SeekBack, &alt("ArrowLeft"));
    assert_action("playback: Alt+Right seeks forward 10 s", ShortcutAction::SeekForward, &alt("ArrowRight"));
    assert_action("playback: Alt+Shift+Left seeks back 60 s", ShortcutAction::SeekBackLong, &alt_shift("ArrowLeft"));
    assert_action("playback: Alt+Shift+Right seeks forward 60 s", ShortcutAction::SeekForwardLong, &alt_shift("ArrowRight"));
    assert_action("playback: Ctrl+Shift+1 picks Repeat", ShortcutAction::ModeRepeat, &ctrl_shift("!"));
    assert_action("playback: Ctrl+Shift+2 picks In order", ShortcutAction::ModeInOrder, &ctrl_shift("@"));
    assert_action("playback: Ctrl+Shift+3 picks Shuffle the folder", ShortcutAction::ModeShuffleFolder, &ctrl_shift("#"));
    assert_action("playback: Ctrl+Shift+4 picks Shuffle all", ShortcutAction::ModeShuffleAll, &ctrl_shift("$"));
    assert_action("playback: the digit itself also picks a mode", ShortcutAction::ModeRepeat, &ctrl_shift("1"));
    assert_action("playback: Ctrl+Home restarts the track", ShortcutAction::Restart, &ctrl("Home"));
}

// What:     `#[test] fn d106_seek_repeats_but_toggles_do_not()` checks the repeat rule of the table.
// Why:      Holding Alt+Right keeps scrubbing, while a held Ctrl+M must cycle once.
#[test]
fn d106_seek_repeats_but_toggles_do_not() {
    assert_action(
        "repeat: a held Alt+Right keeps seeking",
        ShortcutAction::SeekForward,
        &ShortcutRequest { repeat: true, ..alt("ArrowRight") },
    );
    assert_action(
        "repeat: a held Ctrl+M is ignored",
        ShortcutAction::IgnoreRepeat,
        &ShortcutRequest { repeat: true, ..ctrl("m") },
    );
    assert_action(
        "repeat: a held Ctrl+Right is ignored",
        ShortcutAction::IgnoreRepeat,
        &ShortcutRequest { repeat: true, ..ctrl("ArrowRight") },
    );
}

// What:     `#[test] fn d106_browsing_rows_classify_as_expected()` checks folders, focus, dialog and quit.
// Why:      D106 Browsing: Alt+Up and Alt+Down change folder, Ctrl+Shift+G reveals the folder, Alt+1 and Alt+2 focus lists,
//           Ctrl+Shift+O opens the dialog, Ctrl+Q quits. F5 is unbound (D113: the library refreshes by itself).
#[test]
fn d106_browsing_rows_classify_as_expected() {
    assert_action("browsing: Alt+Up shows the previous folder", ShortcutAction::PreviousFolder, &alt("ArrowUp"));
    assert_action("browsing: Alt+Down shows the next folder", ShortcutAction::NextFolder, &alt("ArrowDown"));
    assert_action("browsing: Ctrl+Shift+G reveals the playing folder", ShortcutAction::RevealFolder, &ctrl_shift("G"));
    assert_action("browsing: Alt+1 focuses the folder picker", ShortcutAction::FocusFolders, &alt("1"));
    assert_action("browsing: Alt+2 focuses the track list", ShortcutAction::FocusTracks, &alt("2"));
    assert_action("browsing: Ctrl+Shift+O opens the system dialog", ShortcutAction::OpenFolderDialog, &ctrl_shift("O"));
    assert_action("browsing: F5 is unbound, the library refreshes by itself (D113)", ShortcutAction::Unbound, &pressed("F5"));
    assert_action("browsing: Ctrl+Q quits", ShortcutAction::Quit, &ctrl("q"));
    assert_action(
        "browsing: Ctrl+Q quits from Settings too",
        ShortcutAction::Quit,
        &on_route(KeyRoute::Settings, FocusZone::SettingsList, ctrl("q")),
    );
}

// What:     `#[test] fn d106_mac_rows_use_command_and_option()` checks the macOS map of the new rows.
// Why:      D106 says Command replaces Ctrl and Option replaces Alt where D102 gives no macOS chord.
#[test]
fn d106_mac_rows_use_command_and_option() {
    let command = |key: &str| return ShortcutRequest { meta: true, ..on_mac(pressed(key)) };
    assert_action("mac: Command+Q quits", ShortcutAction::Quit, &command("q"));
    assert_action("mac: Option+Right seeks forward", ShortcutAction::SeekForward, &on_mac(alt("ArrowRight")));
    assert_action(
        "mac: Command+Shift+1 picks Repeat",
        ShortcutAction::ModeRepeat,
        &ShortcutRequest { shift: true, ..command("!") },
    );
    assert_action(
        "mac: Command+Shift+O opens the dialog",
        ShortcutAction::OpenFolderDialog,
        &ShortcutRequest { shift: true, ..command("o") },
    );
    assert_action("mac: Control+Shift+O is unbound", ShortcutAction::Unbound, &on_mac(ctrl_shift("o")));
    assert_action("mac: Command+Home restarts", ShortcutAction::Restart, &command("Home"));
    assert_action("mac: Ctrl+M still cycles the mode", ShortcutAction::Mode, &on_mac(ctrl("m")));
}

// What:     `#[test] fn d106_list_rows_classify_as_expected()` checks the keys inside a focused list.
// Why:      D106: Up, Down, Home, End, PageUp and PageDown move the ring, Enter acts, Space keeps play and pause, a letter jumps.
#[test]
fn d106_list_rows_classify_as_expected() {
    for zone in [FocusZone::FolderList, FocusZone::TrackList] {
        assert_action("list: Up", ShortcutAction::ListUp, &in_zone(zone, pressed("ArrowUp")));
        assert_action("list: Down", ShortcutAction::ListDown, &in_zone(zone, pressed("ArrowDown")));
        assert_action("list: Home", ShortcutAction::ListHome, &in_zone(zone, pressed("Home")));
        assert_action("list: End", ShortcutAction::ListEnd, &in_zone(zone, pressed("End")));
        assert_action("list: PageUp", ShortcutAction::ListPageUp, &in_zone(zone, pressed("PageUp")));
        assert_action("list: PageDown", ShortcutAction::ListPageDown, &in_zone(zone, pressed("PageDown")));
        assert_action("list: Enter", ShortcutAction::ListActivate, &in_zone(zone, pressed("Enter")));
        assert_action("list: Space keeps play and pause", ShortcutAction::Toggle, &in_zone(zone, pressed(" ")));
    }
    let folders = |request: ShortcutRequest| return in_zone(FocusZone::FolderList, request);
    assert_action("folder list: Right moves like Down", ShortcutAction::ListDown, &folders(pressed("ArrowRight")));
    assert_action("folder list: Left moves like Up", ShortcutAction::ListUp, &folders(pressed("ArrowLeft")));
    assert_action("folder list: a letter jumps", ShortcutAction::ListLetter('b'), &folders(pressed("b")));
    assert_action(
        "folder list: a capital letter jumps too",
        ShortcutAction::ListLetter('b'),
        &folders(ShortcutRequest { shift: true, ..pressed("B") }),
    );
    assert_action("folder list: a digit jumps", ShortcutAction::ListLetter('7'), &folders(pressed("7")));
    assert_action("track list: a letter does nothing", ShortcutAction::Unbound, &in_zone(FocusZone::TrackList, pressed("b")));
    assert_action("folder list: Ctrl+B is not a jump", ShortcutAction::Unbound, &folders(ctrl("b")));
    assert_action("folder list: a named key is not a jump", ShortcutAction::Unbound, &folders(pressed("Tab")));
    assert_action("surface: Up keeps its native meaning", ShortcutAction::Unbound, &pressed("ArrowUp"));
}

// What:     `#[test] fn d106_track_row_rows_classify_as_expected()` checks the keys on the focused track row.
// Why:      D106 and D114: the Menu key or Alt+Enter opens the menu, Delete trashes, Ctrl+Shift+C copies, Ctrl+Shift+E shows.
//           Shift+F10 (Run in JetBrains), F5 and Ctrl+Shift+F5 are unbound (D113, D114).
#[test]
fn d106_track_row_rows_classify_as_expected() {
    let row = |request: ShortcutRequest| return in_zone(FocusZone::TrackList, request);
    assert_action("row: Menu key opens the menu", ShortcutAction::RowMenu, &row(pressed("ContextMenu")));
    assert_action("row: Alt+Enter opens the menu", ShortcutAction::RowMenu, &row(alt("Enter")));
    assert_action(
        "row: Shift+F10 is unbound",
        ShortcutAction::Unbound,
        &row(ShortcutRequest { shift: true, ..pressed("F10") }),
    );
    assert_action("row: Delete moves to the trash", ShortcutAction::RowTrash, &row(pressed("Delete")));
    assert_action("row: Ctrl+Shift+C copies", ShortcutAction::RowCopy, &row(ctrl_shift("C")));
    assert_action("row: Ctrl+Shift+E shows in the file manager", ShortcutAction::RowShow, &row(ctrl_shift("E")));
    assert_action("row: Ctrl+Shift+F5 is unbound (D113)", ShortcutAction::Unbound, &row(ctrl_shift("F5")));
    assert_action("row: F5 is unbound (D113)", ShortcutAction::Unbound, &row(pressed("F5")));
    assert_action("not on a row: Delete does nothing", ShortcutAction::Unbound, &pressed("Delete"));
    assert_action("not on a row: Ctrl+Shift+C does nothing", ShortcutAction::Unbound, &ctrl_shift("C"));
    assert_action(
        "folder list: Delete does nothing",
        ShortcutAction::Unbound,
        &in_zone(FocusZone::FolderList, pressed("Delete")),
    );
}

// What:     `#[test] fn d106_menu_rows_classify_as_expected()` checks the keys while the track menu is open.
// Why:      D106: Up, Down and Enter choose a row, Escape closes the menu, and the row chords act on the menu's row.
#[test]
fn d106_menu_rows_classify_as_expected() {
    assert_action("menu: Up", ShortcutAction::MenuUp, &in_menu(pressed("ArrowUp")));
    assert_action("menu: Down", ShortcutAction::MenuDown, &in_menu(pressed("ArrowDown")));
    assert_action("menu: Enter", ShortcutAction::MenuChoose, &in_menu(pressed("Enter")));
    assert_action("menu: Escape", ShortcutAction::ClosePopup, &in_menu(pressed("Escape")));
    assert_action("menu: Delete trashes the menu's row", ShortcutAction::RowTrash, &in_menu(pressed("Delete")));
    assert_action("menu: Alt+Enter has no meaning inside the open menu", ShortcutAction::Owner, &in_menu(alt("Enter")));
    assert_action("menu: Ctrl+Shift+E shows the file", ShortcutAction::RowShow, &in_menu(ctrl_shift("E")));
    assert_action("menu: Space stays with the menu", ShortcutAction::Owner, &in_menu(pressed(" ")));
    assert_action("menu: Ctrl+Q is not offered to the menu", ShortcutAction::Owner, &in_menu(ctrl("q")));
    assert_action("menu: a letter that is no access key stays with the menu", ShortcutAction::Owner, &in_menu(pressed("a")));
    assert_action("menu: p runs Play", ShortcutAction::MenuAccess('p'), &in_menu(pressed("p")));
    assert_action("menu: a capital letter counts too", ShortcutAction::MenuAccess('m'), &in_menu(pressed("M")));
    assert_action("menu: Ctrl+p is not an access key", ShortcutAction::Owner, &in_menu(ctrl("p")));
    assert_action(
        "menu: a held access key does not run twice",
        ShortcutAction::IgnoreRepeat,
        &in_menu(ShortcutRequest { repeat: true, ..pressed("p") }),
    );
    assert_action(
        "menu: a held Down keeps moving",
        ShortcutAction::MenuDown,
        &in_menu(ShortcutRequest { repeat: true, ..pressed("ArrowDown") }),
    );
}

// What:     `#[test] fn d106_search_page_rows_classify_as_expected()` checks the Search page keys.
// Why:      D106: Up and Down move through results and Enter opens the focused one, even while the query field has focus.
#[test]
fn d106_search_page_rows_classify_as_expected() {
    let search = |key: &str| return on_route(KeyRoute::Search, FocusZone::TextField, pressed(key));
    assert_action("search: Up", ShortcutAction::SearchUp, &search("ArrowUp"));
    assert_action("search: Down", ShortcutAction::SearchDown, &search("ArrowDown"));
    assert_action("search: Enter in the query field", ShortcutAction::SearchOpen, &search("Enter"));
    assert_action("search: Escape leaves", ShortcutAction::Back, &search("Escape"));
    assert_action(
        "search: Enter on the Back button stays with the button",
        ShortcutAction::Owner,
        &on_route(KeyRoute::Search, FocusZone::Button, pressed("Enter")),
    );
    assert_action(
        "search: Down on the Clear button still moves through results",
        ShortcutAction::SearchDown,
        &on_route(KeyRoute::Search, FocusZone::Button, pressed("ArrowDown")),
    );
    assert_action(
        "search: Ctrl+F is not offered again",
        ShortcutAction::Owner,
        &on_route(KeyRoute::Search, FocusZone::TextField, ctrl("f")),
    );
    assert_action("search: Home stays with the field", ShortcutAction::Owner, &search("Home"));
}

// What:     `#[test] fn d106_settings_and_editor_rows_classify_as_expected()` checks the Escape ladder and the editor chords.
// Why:      The field's Escape returns to the list, the list's Escape closes Settings, the editor page's Escape leaves it,
//           Ctrl+Enter applies and Ctrl+Shift+Backspace resets.
#[test]
fn d106_settings_and_editor_rows_classify_as_expected() {
    let settings = |zone: FocusZone, request: ShortcutRequest| return on_route(KeyRoute::Settings, zone, request);
    let editor = |request: ShortcutRequest| return on_route(KeyRoute::EditorPage, FocusZone::TextField, request);
    assert_action(
        "settings: Escape in the template field returns to the list",
        ShortcutAction::SettingsFocusList,
        &settings(FocusZone::TextField, pressed("Escape")),
    );
    assert_action(
        "settings: Escape on the list closes Settings",
        ShortcutAction::Back,
        &settings(FocusZone::SettingsList, pressed("Escape")),
    );
    assert_action(
        "settings: Escape on the Back button closes Settings",
        ShortcutAction::Back,
        &settings(FocusZone::Button, pressed("Escape")),
    );
    assert_action("editor page: Escape leaves for the Settings list", ShortcutAction::EditorLeave, &editor(pressed("Escape")));
    assert_action(
        "editor page: Escape on the Back button leaves too",
        ShortcutAction::EditorLeave,
        &on_route(KeyRoute::EditorPage, FocusZone::Button, pressed("Escape")),
    );
    assert_action("editor page: Ctrl+Enter applies and leaves", ShortcutAction::EditorApply, &editor(ctrl("Enter")));
    assert_action(
        "editor page: Ctrl+Shift+Backspace resets",
        ShortcutAction::EditorReset,
        &editor(ctrl_shift("Backspace")),
    );
    assert_action(
        "settings: Ctrl+Enter applies in the right pane",
        ShortcutAction::EditorApply,
        &settings(FocusZone::TextField, ctrl("Enter")),
    );
    assert_action(
        "settings: Ctrl+Enter applies from the list",
        ShortcutAction::EditorApply,
        &settings(FocusZone::SettingsList, ctrl("Enter")),
    );
    assert_action(
        "settings: Enter on the list stays with the list",
        ShortcutAction::Owner,
        &settings(FocusZone::SettingsList, pressed("Enter")),
    );
    assert_action(
        "settings: Down on the list stays with the list",
        ShortcutAction::Owner,
        &settings(FocusZone::SettingsList, pressed("ArrowDown")),
    );
}

// What:     `#[test] fn d106_escape_closes_the_innermost_thing_first()` checks the Escape ladder on the player page.
// Why:      D106: Escape closes the innermost popup, otherwise leaves the current page, including the folder picker overlay.
#[test]
fn d106_escape_closes_the_innermost_thing_first() {
    assert_action(
        "escape: the open menu closes before the picker overlay",
        ShortcutAction::ClosePopup,
        &ShortcutRequest { in_popup: true, picker_overlay: true, ..pressed("Escape") },
    );
    assert_action(
        "escape: the picker overlay closes",
        ShortcutAction::ClosePicker,
        &ShortcutRequest { picker_overlay: true, ..pressed("Escape") },
    );
    assert_action(
        "escape: the picker overlay closes from its list",
        ShortcutAction::ClosePicker,
        &ShortcutRequest { picker_overlay: true, zone: FocusZone::FolderList, ..pressed("Escape") },
    );
    assert_action("escape: the player with nothing open ignores it", ShortcutAction::Unbound, &pressed("Escape"));
    assert_action(
        "escape: Settings closes even while the picker flag is set underneath",
        ShortcutAction::Back,
        &ShortcutRequest {
            picker_overlay: true,
            ..on_route(KeyRoute::Settings, FocusZone::SettingsList, pressed("Escape"))
        },
    );
}

// What:     `#[test] fn d106_player_page_chords_do_nothing_on_other_pages()` checks that Settings and Search are modal.
// Why:      D106: Settings and the template editor are modal pages where Search is not offered, so Ctrl+F does nothing there;
//           the same holds for the other player page chords.
#[test]
fn d106_player_page_chords_do_nothing_on_other_pages() {
    let chords = [
        ("Ctrl+F", ctrl("f")),
        ("Ctrl+O", ctrl("o")),
        ("Ctrl+G", ctrl("g")),
        ("Ctrl+Alt+S", ShortcutRequest { alt: true, ..ctrl("s") }),
        ("Alt+Right", alt("ArrowRight")),
        ("Alt+1", alt("1")),
        ("Ctrl+Shift+1", ctrl_shift("!")),
        ("Ctrl+Shift+O", ctrl_shift("O")),
        ("F5", pressed("F5")),
        ("Ctrl+M", ctrl("m")),
        ("Ctrl+Home", ctrl("Home")),
    ];
    let pages = [
        (KeyRoute::Settings, FocusZone::SettingsList),
        (KeyRoute::Settings, FocusZone::TextField),
        (KeyRoute::EditorPage, FocusZone::TextField),
        (KeyRoute::Search, FocusZone::TextField),
        (KeyRoute::Search, FocusZone::Button),
    ];
    for (name, chord) in chords {
        for (route, zone) in pages {
            let action = shortcut_action(&on_route(route, zone, chord.clone()));
            assert!(
                matches!(action, ShortcutAction::Owner | ShortcutAction::Unbound),
                "{name} on {route:?} with focus in {zone:?} must do nothing, got {action:?}"
            );
        }
    }
}

// What:     `#[test] fn d106_bare_keys_never_fire_while_a_text_field_has_focus()` checks the text field rule.
// Why:      D106: bare keys never fire while a text field has focus, on any page; Escape, Enter and the arrows of Search
//           are the page's explicit rows and are tested above.
#[test]
fn d106_bare_keys_never_fire_while_a_text_field_has_focus() {
    let keys = [
        " ",
        "a",
        "z",
        "1",
        "F5",
        "Delete",
        "Home",
        "End",
        "PageUp",
        "PageDown",
        "ContextMenu",
        "ArrowLeft",
        "ArrowRight",
        "Backspace",
    ];
    for route in [KeyRoute::Player, KeyRoute::Settings, KeyRoute::EditorPage, KeyRoute::Search] {
        for key in keys {
            let request = on_route(route, FocusZone::TextField, pressed(key));
            assert_eq!(shortcut_action(&request), ShortcutAction::Owner, "{key} in a text field on {route:?}");
            let shifted = ShortcutRequest { shift: true, ..request };
            assert_eq!(shortcut_action(&shifted), ShortcutAction::Owner, "Shift+{key} in a text field on {route:?}");
        }
    }
}

// What:     `#[test] fn d106_text_fields_keep_the_playback_chords_that_clash_with_editing()` checks Ctrl+Left, Ctrl+Home and Space.
// Why:      D102: transport keys work only while the playback area has focus, so editing fields keep their keys.
#[test]
fn d106_text_fields_keep_the_playback_chords_that_clash_with_editing() {
    for (name, request) in [
        ("Ctrl+Left", ctrl("ArrowLeft")),
        ("Ctrl+Right", ctrl("ArrowRight")),
        ("Ctrl+Home", ctrl("Home")),
        ("Ctrl+M", ctrl("m")),
        ("Space", pressed(" ")),
    ] {
        assert_eq!(shortcut_action(&in_zone(FocusZone::TextField, request)), ShortcutAction::Owner, "{name}");
    }
}

// What:     `#[test] fn d106_focused_controls_keep_enter_and_space_and_the_other_buttons_keep_the_transport_chords()`
//           checks the control owner rules.
// Why:      D102: a focused control keeps its own keys; the deck belongs to the playback area, other buttons do not.
#[test]
fn d106_focused_controls_keep_enter_and_space_and_the_other_buttons_keep_the_transport_chords() {
    for zone in [FocusZone::DeckButton, FocusZone::ModeGroup, FocusZone::Button] {
        assert_action("control: Space", ShortcutAction::Owner, &in_zone(zone, pressed(" ")));
        assert_action("control: Enter", ShortcutAction::Owner, &in_zone(zone, pressed("Enter")));
    }
    assert_action("seek bar: Space plays and pauses", ShortcutAction::Toggle, &in_zone(FocusZone::Seek, pressed(" ")));
    assert_action("seek bar: Left stays with the slider", ShortcutAction::Unbound, &in_zone(FocusZone::Seek, pressed("ArrowLeft")));
    assert_action("deck button: Ctrl+M still cycles", ShortcutAction::Mode, &in_zone(FocusZone::DeckButton, ctrl("m")));
    assert_action("mode group: Ctrl+Right is next", ShortcutAction::Next, &in_zone(FocusZone::ModeGroup, ctrl("ArrowRight")));
    assert_action("header button: Ctrl+M stays with the button", ShortcutAction::Owner, &in_zone(FocusZone::Button, ctrl("m")));
    assert_action("header button: Ctrl+F still opens Search", ShortcutAction::Search, &in_zone(FocusZone::Button, ctrl("f")));
    assert_action("header button: Alt+Right still seeks", ShortcutAction::SeekForward, &in_zone(FocusZone::Button, alt("ArrowRight")));
}

// What:     `#[test] fn the_licenses_page_answers_escape_with_back_and_keeps_the_players_chords_inert()` checks the Licenses page.
// Why:      Escape on the Licenses page returns to Settings whatever holds focus, and the chords of the player page (Search,
//           Settings, play and pause) must not act on a page drawn over it.
#[test]
fn the_licenses_page_answers_escape_with_back_and_keeps_the_players_chords_inert() {
    for zone in [FocusZone::Surface, FocusZone::Button, FocusZone::SettingsList] {
        assert_action("licenses: Escape", ShortcutAction::Back, &on_route(KeyRoute::Licenses, zone, pressed("Escape")));
    }
    let settings_chord = ShortcutRequest { alt: true, ..ctrl("s") };
    for (name, request) in [("Ctrl+F", ctrl("f")), ("Ctrl+Alt+S", settings_chord), ("Ctrl+M", ctrl("m"))] {
        let action = shortcut_action(&on_route(KeyRoute::Licenses, FocusZone::Button, request));
        assert!(
            !matches!(action, ShortcutAction::Search | ShortcutAction::Settings | ShortcutAction::Mode),
            "{name} acted on the Licenses page: {action:?}"
        );
    }
}
