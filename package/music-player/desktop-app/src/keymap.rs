//! Keyboard shortcut classifier,
//!  ported from `KeyboardMap.kt`,
//!  which follows the
//! accepted keyboard-map prototype (D102),
//!  and extended with the D106 table.
//!
//! The classifier reads one key press together with the page the window shows,
//!  the focus zone that
//! received the key,
//!  and the popup state.
//!  Every chord lives in the table of `keymap_bindings`,
//!  so the
//! classifier,
//!  the key hint badges,
//!  the tooltips and the menu rows all read the same rows.
//!  Unknown platform names are rejected by
//! `KeyboardPlatform::from_name` before a request can be built,
//!  so `shortcut_action` never sees one.

/// What:
/// `mod shortcut_action;` the action vocabulary the classifier answers with
/// (`src/keymap/shortcut_action.rs`).
/// Why:
/// the enum and its wire names are a long list that every caller switches on,
/// so it stands
/// beside the classifier rather than inside it.
mod shortcut_action;

/// Re-exports the action enum,
/// which callers still reach through `keymap::ShortcutAction`.
pub use self::shortcut_action::ShortcutAction;

/// Imports the binding scopes and lookups that map a request to a row of the table.
use crate::keymap_bindings::{
    Scope,
    binding_matching,
    popup_binding_matching,
};

/// Re-exports the keyboard maps and their error,
/// which callers still reach through `keymap`.
pub use crate::keymap_platform::{
    KeyboardPlatform,
    KeymapError,
};

/// What:
///      `pub enum ShortcutOwner` names who owns the keys of a focus zone.
/// Why:
///      Owners decide whether a key belongs to the player or to a widget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShortcutOwner = "player" | "editor" | "control" | "surface";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutOwner {
    /// The playback shortcut area itself,
    ///  the only owner that gets transport keys.
    Player,
    /// A text field or editable region that keeps its own keys.
    Editor,
    /// A focused button,
    ///  input,
    ///  select,
    ///  or summary that keeps its own keys.
    Control,
    /// Any other surface,
    ///  whose keys are never bound to playback.
    Surface,
}

/// What:
///      `pub enum KeyRoute` names the page the window shows over the player.
/// Why:
///      Most chords belong to the player page only,
///  and Settings,
///  its editor page and Search each answer a few keys of their own (D106).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeyRoute = "player" | "search" | "settings" | "editor-page" | "licenses";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyRoute {
    /// The player:
    /// folder picker,
    ///  track list and transport deck.
    Player,
    /// The Search page.
    Search,
    /// The Settings page,
    ///  including the template editor in its right pane on a wide window.
    Settings,
    /// The full-width template editor page of a narrow window.
    EditorPage,
    /// The Licenses page that the Licenses row of Settings opens.
    Licenses,
}

/// What:
///      `pub enum FocusZone` names the kind of element that holds keyboard focus.
/// Why:
///      D102 gives some keys to the playback area and others to the focused control or text field,
///  and the list keys of D106 act only inside a list,
///  so the classifier needs the zone and not just the page.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FocusZone = "surface" | "folder-list" | "track-list" | "seek" | "mode-group" | "deck-button"
///   | "button" | "text-field" | "settings-list";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusZone {
    /// No element has focus,
    ///  or the window's key scope itself does.
    Surface,
    /// The folder picker list.
    FolderList,
    /// The track list.
    TrackList,
    /// The seek bar.
    Seek,
    /// The playback mode group.
    ModeGroup,
    /// Previous,
    ///  play and pause,
    ///  or next.
    DeckButton,
    /// Any other button,
    ///  such as Search,
    ///  Settings,
    ///  Open,
    ///  Back,
    ///  Clear or Reset.
    Button,
    /// A text field:
    /// the Search query or the template line.
    TextField,
    /// The Settings list.
    SettingsList,
}

/// Methods of `FocusZone`.
impl FocusZone {
    /// What:
    ///      `from_int` reads the zone number the window reports.
    /// Why:
    ///      Slint passes the zone as an integer,
    ///  and an unknown number must fall back to the surface rather than invent a zone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fromInt = (n: number): FocusZone => ZONES[n] ?? "surface";
    /// ```
    pub fn from_int(value: i32) -> Self {
        return match value {
            1 => Self::FolderList,
            2 => Self::TrackList,
            3 => Self::Seek,
            4 => Self::ModeGroup,
            5 => Self::DeckButton,
            6 => Self::Button,
            7 => Self::TextField,
            8 => Self::SettingsList,
            _ => Self::Surface,
        };
    }

    /// What:
    ///      `owner` gives who owns the keys of this zone.
    /// Why:
    ///      A text field keeps its own keys,
    ///  a button or a group of options keeps Enter,
    ///  Space and the arrows,
    ///  and the rest of the playback area gets transport keys.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const owner = (zone: FocusZone): ShortcutOwner => ...;
    /// ```
    pub fn owner(self) -> ShortcutOwner {
        return match self {
            Self::TextField => ShortcutOwner::Editor,
            Self::ModeGroup | Self::DeckButton | Self::Button | Self::SettingsList => {
                ShortcutOwner::Control
            }
            Self::Surface | Self::FolderList | Self::TrackList | Self::Seek => {
                ShortcutOwner::Player
            }
        };
    }

    /// What:
    ///      `in_playback_area` is true for the zones where transport chords such as Control+M work.
    /// Why:
    ///      D102 limits transport keys to the playback area so editing fields and the other pages' controls keep their keys;
    ///  the deck and the lists belong to the playback area.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const inPlaybackArea = (zone: FocusZone) => !["button", "text-field", "settings-list"].includes(zone);
    /// ```
    pub fn in_playback_area(self) -> bool {
        return !matches!(self, Self::Button | Self::TextField | Self::SettingsList);
    }
}

/// What:
///      `pub struct ShortcutRequest` is one key press with the focus and popup state the classifier reads.
/// Why:
///      The classifier needs all of these inputs together,
///  and named fields keep call sites readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShortcutRequest = {
///   platform: KeyboardPlatform; key: string; ctrl: boolean; meta: boolean; alt: boolean;
///   shift: boolean; repeat: boolean; zone: FocusZone; route: KeyRoute; inPopup: boolean;
///   pickerOverlay: boolean; composing: boolean;
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutRequest {
    /// Keyboard map that interprets the key.
    pub platform: KeyboardPlatform,
    /// Key text as the platform reports it,
    ///  such as `f`,
    ///  ` `,
    ///  or `ArrowLeft`.
    pub key: String,
    /// True while Control is held.
    pub ctrl: bool,
    /// True while Command (macOS) or Meta (Windows and Linux) is held.
    pub meta: bool,
    /// True while Alt or Option is held.
    pub alt: bool,
    /// True while Shift is held.
    pub shift: bool,
    /// True when the event is an auto-repeat of a held key.
    pub repeat: bool,
    /// Kind of element that received the key.
    pub zone: FocusZone,
    /// Page the window shows.
    pub route: KeyRoute,
    /// True while a popup (the track menu) is open.
    pub in_popup: bool,
    /// True while the folder picker covers the track list on the folded layout.
    pub picker_overlay: bool,
    /// True while an input method is composing text.
    pub composing: bool,
}

/// What:
///      `pub fn shortcut_action` classifies one key press into the action the prototype would take.
/// Why:
///      It is the port of the prototype's classifier extended with D106,
///  and the first group that matches decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shortcutAction(request: ShortcutRequest): ShortcutAction {
///   return composingOrPopupAction(request) ?? tableAction(request) ?? ownerAction(request);
/// }
/// ```
pub fn shortcut_action(request: &ShortcutRequest) -> ShortcutAction {
    if let Some(action) = composing_or_popup_action(request) {
        return action;
    }
    if let Some(action) = table_action(request) {
        return action;
    }
    if let Some(action) = list_letter_action(request) {
        return action;
    }
    return owner_action(request);
}

/// What:
///      `composing_or_popup_action` handles an input method still composing,
///  and an open popup.
/// Why:
///      These rules come first,
///  so a composing key or a popup key never reaches a shortcut except the few the open track menu answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function composingOrPopupAction(request: ShortcutRequest): ShortcutAction | null {
///   if (request.composing) return "owner";
///   if (request.inPopup) return popupAction(request);
///   return null;
/// }
/// ```
fn composing_or_popup_action(request: &ShortcutRequest) -> Option<ShortcutAction> {
    if request.composing {
        return Some(ShortcutAction::Owner);
    }
    if request.in_popup {
        return Some(popup_action(request));
    }
    return None;
}

/// What:
///      `popup_action` answers a key while the track menu is open:
///  Escape closes it,
///  Up,
///  Down and Enter choose a row,
///  an access letter runs its row,
///  and the track row chords act on the row the menu is open on.
/// Why:
///      D106 gives the open menu those keys and nothing else,
///  so every other key stays with the popup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function popupAction(request: ShortcutRequest): ShortcutAction {
///   if (request.key === "Escape") return "close-popup";
///   return popupBinding(request)?.action ?? "owner";
/// }
/// ```
fn popup_action(request: &ShortcutRequest) -> ShortcutAction {
    if request.key == "Escape" {
        return ShortcutAction::ClosePopup;
    }
    if let Some(binding) = popup_binding_matching(request) {
        return if request.repeat && !binding.repeats {
            ShortcutAction::IgnoreRepeat
        } else {
            binding.action
        };
    }
    return menu_access_action(request).unwrap_or(ShortcutAction::Owner);
}

/// What:
///      `menu_access_action` maps a plain letter pressed while the track menu is open to the row it runs.
/// Why:
///      D114 gives every menu row an access letter;
/// the letters come from the one table in `track_menu`,
/// so the underline
///  the window draws and the key this accepts cannot disagree.
/// A letter held with Control,
/// Alt or Meta is not an access
///  key,
/// and a held-down letter does not run the row twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function menuAccessAction(r: ShortcutRequest): ShortcutAction | null {
///   if (r.ctrl || r.meta || r.alt || r.key.length !== 1) return null;
///   return accessAction(r.key) === null ? null : r.repeat ? "ignore-repeat" : { kind: "menu-access", letter: r.key.toLowerCase() };
/// }
/// ```
fn menu_access_action(request: &ShortcutRequest) -> Option<ShortcutAction> {
    if request.ctrl || request.meta || request.alt {
        return None;
    }
    let mut characters = request.key.chars();
    let first = characters.next()?;
    if characters.next().is_some() {
        return None;
    }
    crate::track_menu::access_action(first)?;
    if request.repeat {
        return Some(ShortcutAction::IgnoreRepeat);
    }
    return Some(ShortcutAction::MenuAccess(first.to_lowercase().next()?));
}

/// What:
///      `table_action` looks the key up in the D106 table.
/// Why:
///      One table holds every chord,
///  and a repeated press of a chord that must not repeat is reported as `IgnoreRepeat` so the window still uses the key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tableAction(request: ShortcutRequest): ShortcutAction | null {
///   const binding = bindingMatching(request);
///   if (!binding) return null;
///   return request.repeat && !binding.repeats ? "ignore-repeat" : binding.action;
/// }
/// ```
fn table_action(request: &ShortcutRequest) -> Option<ShortcutAction> {
    let binding = binding_matching(request)?;
    if request.repeat && !binding.repeats {
        return Some(ShortcutAction::IgnoreRepeat);
    }
    return Some(binding.action);
}

/// What:
///      `list_letter_action` maps a plain letter or digit in the folder list to a jump to the first folder that starts with it.
/// Why:
///      The table lists named keys only,
///  and the letter shortcut accepts any character,
///  so it has its own rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listLetterAction(r: ShortcutRequest): ShortcutAction | null {
///   if (r.route !== "player" || r.zone !== "folder-list" || r.ctrl || r.meta || r.alt) return null;
///   return isSingleLetterOrDigit(r.key) ? { kind: "list-letter", letter: r.key.toLowerCase() } : null;
/// }
/// ```
fn list_letter_action(request: &ShortcutRequest) -> Option<ShortcutAction> {
    if request.route != KeyRoute::Player
        || request.zone != FocusZone::FolderList
        || request.ctrl
        || request.meta
        || request.alt
    {
        return None;
    }
    let mut characters = request.key.chars();
    let first = characters.next()?;
    if characters.next().is_some() || !first.is_alphanumeric() {
        return None;
    }
    let lowered = first.to_lowercase().next()?;
    return Some(ShortcutAction::ListLetter(lowered));
}

/// What:
///      `owner_action` decides what an unmatched key means by who owns focus:
///  editors and controls keep it,
///  and the rest of the window leaves it unbound.
/// Why:
///      Text and controls keep their own keys,
///  and a key nobody binds must reach them untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownerAction(request: ShortcutRequest): ShortcutAction {
///   return request.zone.owner === "player" ? "unbound" : "owner";
/// }
/// ```
fn owner_action(request: &ShortcutRequest) -> ShortcutAction {
    return match request.zone.owner() {
        ShortcutOwner::Editor | ShortcutOwner::Control => ShortcutAction::Owner,
        ShortcutOwner::Player | ShortcutOwner::Surface => ShortcutAction::Unbound,
    };
}

/// What:
///      `scope_allows` tells whether a table row applies to this request's page and focus zone.
/// Why:
///      The same key means different things per page and per zone,
///  and keeping the test here lets the table stay plain data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scopeAllows = (scope: Scope, r: ShortcutRequest): boolean => ...;
/// ```
pub(crate) fn scope_allows(
    scope: Scope,
    request: &ShortcutRequest,
) -> bool {
    let on_player = request.route == KeyRoute::Player;
    return match scope {
        Scope::Anywhere => true,
        Scope::PlayerPage => on_player,
        Scope::PlaybackArea => on_player && request.zone.in_playback_area(),
        Scope::SpaceZones => {
            on_player
                && matches!(
                    request.zone,
                    FocusZone::Surface
                        | FocusZone::FolderList
                        | FocusZone::TrackList
                        | FocusZone::Seek
                )
        }
        Scope::ListZone => {
            on_player && matches!(request.zone, FocusZone::FolderList | FocusZone::TrackList)
        }
        Scope::FolderList => on_player && request.zone == FocusZone::FolderList,
        Scope::TrackList => on_player && request.zone == FocusZone::TrackList,
        Scope::Popup => on_player && request.in_popup,
        Scope::PickerOverlay => on_player && request.picker_overlay,
        Scope::SearchPage => request.route == KeyRoute::Search,
        Scope::SearchField => {
            request.route == KeyRoute::Search && request.zone == FocusZone::TextField
        }
        Scope::SettingsIdle => {
            request.route == KeyRoute::Settings && request.zone != FocusZone::TextField
        }
        Scope::SettingsField => {
            request.route == KeyRoute::Settings && request.zone == FocusZone::TextField
        }
        Scope::EditorPage => request.route == KeyRoute::EditorPage,
        Scope::LicensesPage => request.route == KeyRoute::Licenses,
        Scope::EditorOrSettings => {
            matches!(request.route, KeyRoute::Settings | KeyRoute::EditorPage)
        }
    };
}

/// Unit tests in `keymap_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "keymap_tests.rs"]
mod tests;
