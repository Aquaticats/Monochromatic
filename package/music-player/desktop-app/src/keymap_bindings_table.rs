//! The D106 binding table itself:
//! one row per chord,
//! in matching order.
//!
//! What:
//! `BINDINGS` is the whole table as plain data,
//! kept apart from the row types,
//! the builders and the lookups so each file stays within the line budget.
//! Why:
//! the table is audited line by line against D106,
//! so a reader opening this file finds the chords and nothing else.

/// What:
///      `use super::{...}` brings the row builder,
///  the row type,
///  the scope enum and the modifier lists from the parent module `keymap_bindings`.
/// Why:
///      The table is built from those helpers,
///  and a second copy here could drift from the one the lookups read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { row, Scope, NONE, P, PS, PA, C, A, AS, Binding } from "./keymap_bindings";
/// ```
use super::{
    A,
    AS,
    Binding,
    C,
    NONE,
    P,
    PA,
    PS,
    Scope,
    row,
};

/// What:
///      `use crate::keymap::{...}` brings the action each row names,
///  and the map a row may be restricted to.
/// Why:
///      A row's action and platform are types of the parent module's sibling `keymap`,
///  so the table and the key classifier share one vocabulary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { KeyboardPlatform, ShortcutAction } from "./keymap";
/// ```
use crate::keymap::{
    KeyboardPlatform,
    ShortcutAction,
};

/// What:
///      `BINDINGS` is the whole D106 table in matching order.
/// Why:
///      The first row that matches a key press and whose scope applies decides,
///  so a more specific row sits above a general one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BINDINGS: readonly Binding[] = [ /* one row per chord */ ];
/// ```
pub static BINDINGS: &[Binding] = &[
    // Popup rows come first: with the track menu open these decide before any other row.
    row(ShortcutAction::MenuUp, NONE, &["ArrowUp"], Scope::Popup)
        .repeating()
        .in_menu(),
    row(ShortcutAction::MenuDown, NONE, &["ArrowDown"], Scope::Popup)
        .repeating()
        .in_menu(),
    row(ShortcutAction::MenuChoose, NONE, &["Enter"], Scope::Popup).in_menu(),
    // Escape, from the innermost thing outward: the picker overlay, the editor line, the Settings list, Search, Licenses.
    row(
        ShortcutAction::ClosePicker,
        NONE,
        &["Escape"],
        Scope::PickerOverlay,
    ),
    row(
        ShortcutAction::SettingsFocusList,
        NONE,
        &["Escape"],
        Scope::SettingsField,
    ),
    row(ShortcutAction::Back, NONE, &["Escape"], Scope::SettingsIdle),
    row(
        ShortcutAction::EditorLeave,
        NONE,
        &["Escape"],
        Scope::EditorPage,
    ),
    row(ShortcutAction::Back, NONE, &["Escape"], Scope::SearchPage),
    row(ShortcutAction::Back, NONE, &["Escape"], Scope::LicensesPage),
    // Editor pages: Control+Enter applies and leaves, Control+Shift+Backspace resets the template.
    row(
        ShortcutAction::EditorApply,
        P,
        &["Enter"],
        Scope::EditorOrSettings,
    ),
    row(
        ShortcutAction::EditorReset,
        PS,
        &["Backspace"],
        Scope::EditorOrSettings,
    ),
    // Search: Up and Down move through the results even while the query field has focus; Enter opens the result.
    row(
        ShortcutAction::SearchUp,
        NONE,
        &["ArrowUp"],
        Scope::SearchPage,
    )
    .repeating(),
    row(
        ShortcutAction::SearchDown,
        NONE,
        &["ArrowDown"],
        Scope::SearchPage,
    )
    .repeating(),
    row(
        ShortcutAction::SearchOpen,
        NONE,
        &["Enter"],
        Scope::SearchField,
    ),
    // Kept from D102.
    row(ShortcutAction::Search, P, &["f"], Scope::PlayerPage),
    row(ShortcutAction::Picker, P, &["o"], Scope::PlayerPage),
    row(ShortcutAction::Settings, PA, &["s"], Scope::PlayerPage).on(KeyboardPlatform::WindowsLinux),
    row(ShortcutAction::Settings, P, &[","], Scope::PlayerPage).on(KeyboardPlatform::Mac),
    row(ShortcutAction::Reveal, P, &["g"], Scope::PlayerPage).on(KeyboardPlatform::WindowsLinux),
    row(ShortcutAction::Reveal, P, &["l"], Scope::PlayerPage).on(KeyboardPlatform::Mac),
    row(ShortcutAction::Toggle, NONE, &[" "], Scope::SpaceZones),
    row(ShortcutAction::Mode, C, &["m"], Scope::PlaybackArea),
    row(
        ShortcutAction::Previous,
        C,
        &["ArrowLeft"],
        Scope::PlaybackArea,
    )
    .on(KeyboardPlatform::WindowsLinux),
    row(
        ShortcutAction::Next,
        C,
        &["ArrowRight"],
        Scope::PlaybackArea,
    )
    .on(KeyboardPlatform::WindowsLinux),
    row(
        ShortcutAction::Previous,
        PS,
        &["[", "{"],
        Scope::PlaybackArea,
    )
    .on(KeyboardPlatform::Mac),
    row(ShortcutAction::Next, PS, &["]", "}"], Scope::PlaybackArea).on(KeyboardPlatform::Mac),
    // Playback (D106): seek 10 s and 60 s, pick a mode, restart the track.
    row(
        ShortcutAction::SeekBack,
        A,
        &["ArrowLeft"],
        Scope::PlayerPage,
    )
    .repeating(),
    row(
        ShortcutAction::SeekForward,
        A,
        &["ArrowRight"],
        Scope::PlayerPage,
    )
    .repeating(),
    row(
        ShortcutAction::SeekBackLong,
        AS,
        &["ArrowLeft"],
        Scope::PlayerPage,
    )
    .repeating(),
    row(
        ShortcutAction::SeekForwardLong,
        AS,
        &["ArrowRight"],
        Scope::PlayerPage,
    )
    .repeating(),
    row(
        ShortcutAction::ModeRepeat,
        PS,
        &["1", "!"],
        Scope::PlayerPage,
    ),
    row(
        ShortcutAction::ModeInOrder,
        PS,
        &["2", "@"],
        Scope::PlayerPage,
    ),
    row(
        ShortcutAction::ModeShuffleFolder,
        PS,
        &["3", "#"],
        Scope::PlayerPage,
    ),
    row(
        ShortcutAction::ModeShuffleAll,
        PS,
        &["4", "$"],
        Scope::PlayerPage,
    ),
    row(ShortcutAction::Restart, P, &["Home"], Scope::PlaybackArea),
    // Browsing (D106).
    row(
        ShortcutAction::PreviousFolder,
        A,
        &["ArrowUp"],
        Scope::PlayerPage,
    ),
    row(
        ShortcutAction::NextFolder,
        A,
        &["ArrowDown"],
        Scope::PlayerPage,
    ),
    row(ShortcutAction::RevealFolder, PS, &["g"], Scope::PlayerPage),
    row(ShortcutAction::FocusFolders, A, &["1"], Scope::PlayerPage),
    row(ShortcutAction::FocusTracks, A, &["2"], Scope::PlayerPage),
    row(
        ShortcutAction::OpenFolderDialog,
        PS,
        &["o"],
        Scope::PlayerPage,
    ),
    row(ShortcutAction::Quit, P, &["q"], Scope::Anywhere),
    // Inside a focused list: the arrows, Home, End, the page keys, Enter. A letter has its own rule in the classifier.
    row(ShortcutAction::ListUp, NONE, &["ArrowUp"], Scope::ListZone).repeating(),
    row(
        ShortcutAction::ListDown,
        NONE,
        &["ArrowDown"],
        Scope::ListZone,
    )
    .repeating(),
    row(
        ShortcutAction::ListUp,
        NONE,
        &["ArrowLeft"],
        Scope::FolderList,
    )
    .repeating(),
    row(
        ShortcutAction::ListDown,
        NONE,
        &["ArrowRight"],
        Scope::FolderList,
    )
    .repeating(),
    row(ShortcutAction::ListHome, NONE, &["Home"], Scope::ListZone),
    row(ShortcutAction::ListEnd, NONE, &["End"], Scope::ListZone),
    row(
        ShortcutAction::ListPageUp,
        NONE,
        &["PageUp"],
        Scope::ListZone,
    )
    .repeating(),
    row(
        ShortcutAction::ListPageDown,
        NONE,
        &["PageDown"],
        Scope::ListZone,
    )
    .repeating(),
    row(
        ShortcutAction::ListActivate,
        NONE,
        &["Enter"],
        Scope::ListZone,
    ),
    // The focused track row: its menu (Menu key or Alt+Enter), trash, copy and show.
    row(
        ShortcutAction::RowMenu,
        NONE,
        &["ContextMenu"],
        Scope::TrackList,
    ),
    row(ShortcutAction::RowMenu, A, &["Enter"], Scope::TrackList),
    row(
        ShortcutAction::RowTrash,
        NONE,
        &["Delete"],
        Scope::TrackList,
    )
    .in_menu(),
    row(ShortcutAction::RowCopy, PS, &["c"], Scope::TrackList).in_menu(),
    row(ShortcutAction::RowShow, PS, &["e"], Scope::TrackList).in_menu(),
];
