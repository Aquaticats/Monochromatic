//! The action vocabulary of the keyboard shortcut classifier:
//!  every result a key press can produce.
//!
//! What:
//! `ShortcutAction` names what a key press means,
//!  and `wire_name` spells each name the way the accepted prototype does.
//! Why:
//! the classifier,
//!  the key hint badges and the window's own dispatch all switch on this one enum,
//!  so it lives beside the classifier that returns it rather than inside it,
//!  and `keymap` re-exports it so every caller keeps the `keymap::ShortcutAction` path.
/// What:
///      `pub enum ShortcutAction` lists every result the classifier can return.
/// Why:
///      Each variant maps to one named result,
///  and `wire_name` keeps the prototype's spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShortcutAction = "toggle" | "owner" | "ignore-repeat" | "search" | "picker" | "settings"
///   | "previous" | "next" | "close-popup" | "back" | "reveal" | "mode" | "unbound" | "seek-back" | /* ... */;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    /// The focused text,
    ///  control,
    ///  or composition keeps this key.
    Owner,
    /// Escape closes the open popup.
    ClosePopup,
    /// Opens Search.
    Search,
    /// Opens or closes the folder picker.
    Picker,
    /// Opens in-app Settings.
    Settings,
    /// Focuses the current track row without a transport action.
    Reveal,
    /// Returns from Search or Settings to the player.
    Back,
    /// No shortcut applies.
    Unbound,
    /// A repeated key event that the player ignores.
    IgnoreRepeat,
    /// Plays or pauses the player.
    Toggle,
    /// Cycles the end-of-track mode.
    Mode,
    /// Moves to the previous track.
    Previous,
    /// Moves to the next track.
    Next,
    /// Seeks 10 seconds back.
    SeekBack,
    /// Seeks 10 seconds forward.
    SeekForward,
    /// Seeks 60 seconds back.
    SeekBackLong,
    /// Seeks 60 seconds forward.
    SeekForwardLong,
    /// Picks the Repeat mode.
    ModeRepeat,
    /// Picks the In order mode.
    ModeInOrder,
    /// Picks the Shuffle the folder mode.
    ModeShuffleFolder,
    /// Picks the Shuffle all mode.
    ModeShuffleAll,
    /// Restarts the track.
    Restart,
    /// Shows the previous folder.
    PreviousFolder,
    /// Shows the next folder.
    NextFolder,
    /// Shows the folder of the playing track.
    RevealFolder,
    /// Moves keyboard focus to the folder picker.
    FocusFolders,
    /// Moves keyboard focus to the track list.
    FocusTracks,
    /// Moves the list focus ring up one entry.
    ListUp,
    /// Moves the list focus ring down one entry.
    ListDown,
    /// Moves the list focus ring to the first entry.
    ListHome,
    /// Moves the list focus ring to the last entry.
    ListEnd,
    /// Moves the list focus ring up one page.
    ListPageUp,
    /// Moves the list focus ring down one page.
    ListPageDown,
    /// Activates the entry under the list focus ring.
    ListActivate,
    /// Moves the folder focus ring to the first folder that starts with this letter.
    ListLetter(
        /// The letter that was pressed.
        char,
    ),
    /// Opens the system folder dialog.
    OpenFolderDialog,
    /// Quits the application.
    Quit,
    /// Opens the menu of the focused track row.
    RowMenu,
    /// Moves the focused track row to the trash.
    RowTrash,
    /// Copies the focused track row's file name.
    RowCopy,
    /// Shows the focused track row in the file manager.
    RowShow,
    /// Runs the track menu row whose access letter was pressed while the menu is open.
    MenuAccess(
        /// The access letter that was pressed.
        char,
    ),
    /// Moves the track menu's highlighted row up.
    MenuUp,
    /// Moves the track menu's highlighted row down.
    MenuDown,
    /// Runs the track menu's highlighted row.
    MenuChoose,
    /// Closes the folder picker overlay of the folded layout.
    ClosePicker,
    /// Moves the Search page's result highlight up.
    SearchUp,
    /// Moves the Search page's result highlight down.
    SearchDown,
    /// Opens the highlighted Search result.
    SearchOpen,
    /// Moves focus from the template field to the Settings list.
    SettingsFocusList,
    /// Applies the template and leaves the editor.
    EditorApply,
    /// Resets the open template to its default.
    EditorReset,
    /// Leaves the template editor page for the Settings list.
    EditorLeave,
}

/// Methods of `ShortcutAction`.
impl ShortcutAction {
    /// What:
    ///      `wire_name` returns the action text exactly as the prototype returns it,
    ///  and a kebab-case name for the actions D106 adds.
    /// Why:
    ///      Tests and any future bridge compare the same spelling the prototype uses.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function wireName(action: ShortcutAction): string;
    /// ```
    pub fn wire_name(self) -> &'static str {
        return match self {
            Self::Owner => "owner",
            Self::ClosePopup => "close-popup",
            Self::Search => "search",
            Self::Picker => "picker",
            Self::Settings => "settings",
            Self::Reveal => "reveal",
            Self::Back => "back",
            Self::Unbound => "unbound",
            Self::IgnoreRepeat => "ignore-repeat",
            Self::Toggle => "toggle",
            Self::Mode => "mode",
            Self::Previous => "previous",
            Self::Next => "next",
            Self::SeekBack => "seek-back",
            Self::SeekForward => "seek-forward",
            Self::SeekBackLong => "seek-back-long",
            Self::SeekForwardLong => "seek-forward-long",
            Self::ModeRepeat => "mode-repeat",
            Self::ModeInOrder => "mode-in-order",
            Self::ModeShuffleFolder => "mode-shuffle-folder",
            Self::ModeShuffleAll => "mode-shuffle-all",
            Self::Restart => "restart",
            Self::PreviousFolder => "previous-folder",
            Self::NextFolder => "next-folder",
            Self::RevealFolder => "reveal-folder",
            Self::FocusFolders => "focus-folders",
            Self::FocusTracks => "focus-tracks",
            Self::ListUp => "list-up",
            Self::ListDown => "list-down",
            Self::ListHome => "list-home",
            Self::ListEnd => "list-end",
            Self::ListPageUp => "list-page-up",
            Self::ListPageDown => "list-page-down",
            Self::ListActivate => "list-activate",
            Self::ListLetter(_) => "list-letter",
            Self::OpenFolderDialog => "open-folder-dialog",
            Self::Quit => "quit",
            Self::RowMenu => "row-menu",
            Self::RowTrash => "row-trash",
            Self::RowCopy => "row-copy",
            Self::RowShow => "row-show",
            Self::MenuAccess(_) => "menu-access",
            Self::MenuUp => "menu-up",
            Self::MenuDown => "menu-down",
            Self::MenuChoose => "menu-choose",
            Self::ClosePicker => "close-picker",
            Self::SearchUp => "search-up",
            Self::SearchDown => "search-down",
            Self::SearchOpen => "search-open",
            Self::SettingsFocusList => "settings-focus-list",
            Self::EditorApply => "editor-apply",
            Self::EditorReset => "editor-reset",
            Self::EditorLeave => "editor-leave",
        };
    }
}
