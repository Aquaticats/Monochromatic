//! The D106 binding table: every chord of the desktop player as plain data.
//!
//! What: `BINDINGS` lists one row per chord with its modifiers, key names, platform, scope and repeat rule;
//! `binding_matching` finds the row a key press hits; `shortcut_text` writes a row the way a tooltip or a menu row
//! shows it.
//! Why: the classifier, the key hint badges, the tooltips and the menu rows must agree on every chord, so they all
//! read this one table and nothing else spells a chord out.

use crate::keymap::{KeyboardPlatform, ShortcutAction, ShortcutRequest, scope_allows};

/// What:
///      `pub enum Modifier` names a modifier a chord requires.
/// Why:
///      D106 writes chords with the primary modifier (Control on Windows and Linux,
///  Command on macOS) and Alt (Option on macOS),
///  and D102 keeps Control+M literal on both maps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Modifier = "primary" | "ctrl" | "alt" | "shift";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    /// Control on Windows and Linux,
    ///  Command on macOS.
    Primary,
    /// The Control key on every map.
    Ctrl,
    /// Alt,
    ///  or Option on macOS.
    Alt,
    /// Shift.
    Shift,
}

/// What:
///      `pub enum Scope` says on which page and in which focus zone a row applies.
/// Why:
///      The same key means different things per page and per zone,
///  and the scope keeps each row inert everywhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scope = "anywhere" | "player-page" | "playback-area" | "space-zones" | /* ... */;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Every page and zone.
    Anywhere,
    /// The player page,
    ///  whatever holds focus.
    PlayerPage,
    /// The player page with focus on the lists,
    ///  the deck or the surface (not a text field or another page's control).
    PlaybackArea,
    /// The player page with focus where Space plays and pauses instead of activating a control.
    SpaceZones,
    /// The player page with focus in the folder or the track list.
    ListZone,
    /// The player page with focus in the folder list.
    FolderList,
    /// The player page with focus in the track list.
    TrackList,
    /// The player page while the track menu is open.
    Popup,
    /// The player page while the folder picker covers the track list.
    PickerOverlay,
    /// The Search page,
    ///  whatever holds focus.
    SearchPage,
    /// The Search page with focus in its query field.
    SearchField,
    /// The Settings page with focus outside the template field.
    SettingsIdle,
    /// The Settings page with focus in the template field.
    SettingsField,
    /// The full-width template editor page.
    EditorPage,
    /// The Licenses page,
    ///  whatever holds focus.
    LicensesPage,
    /// The Settings page and the template editor page.
    EditorOrSettings,
}

/// What:
///      `pub struct Binding` is one row of the table.
/// Why:
///      Plain data keeps the table auditable: each row can be read against D106 line by line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Binding = {
///   action: ShortcutAction; platform: KeyboardPlatform | null; modifiers: Modifier[]; keys: string[];
///   scope: Scope; repeats: boolean; menu: boolean;
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Binding {
    /// What the chord does.
    pub action: ShortcutAction,
    /// The only map the row applies to,
    ///  or `None` for both.
    pub platform: Option<KeyboardPlatform>,
    /// Modifiers the chord requires and no others.
    pub modifiers: &'static [Modifier],
    /// Key names that trigger it: DOM-style names for named keys,
    ///  single characters for letters and digits (matched ignoring case),
    ///  and the shifted symbol of a digit key as a second name.
    pub keys: &'static [&'static str],
    /// Where the row applies.
    pub scope: Scope,
    /// True when holding the key may repeat the action.
    pub repeats: bool,
    /// True when the row also applies while the track menu is open.
    pub menu: bool,
}

/// The modifier lists the table uses.
const NONE: &[Modifier] = &[];
/// Primary modifier alone.
const P: &[Modifier] = &[Modifier::Primary];
/// Primary modifier with Shift.
const PS: &[Modifier] = &[Modifier::Primary, Modifier::Shift];
/// Primary modifier with Alt.
const PA: &[Modifier] = &[Modifier::Primary, Modifier::Alt];
/// Control alone,
///  literally,
///  on every map.
const C: &[Modifier] = &[Modifier::Ctrl];
/// Alt alone.
const A: &[Modifier] = &[Modifier::Alt];
/// Alt with Shift.
const AS: &[Modifier] = &[Modifier::Alt, Modifier::Shift];

/// What:
///      `row` builds a table row that applies to both maps,
///  does not repeat and does not apply to the open menu.
/// Why:
///      Most rows are that plain,
///  so the table lists only the exceptions through `on`,
///  `repeating` and `in_menu`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const row = (action, modifiers, keys, scope): Binding => ({ action, platform: null, modifiers, keys, scope, repeats: false, menu: false });
/// ```
const fn row(action: ShortcutAction, modifiers: &'static [Modifier], keys: &'static [&'static str], scope: Scope) -> Binding {
    return Binding { action, platform: None, modifiers, keys, scope, repeats: false, menu: false };
}

/// Builder methods of `Binding` for the table's exceptions.
impl Binding {
    /// What:
    ///      `on` restricts a row to one map.
    /// Why:
    ///      D102 gives different chords for Settings,
    ///  reveal and track change on macOS.
    const fn on(self, platform: KeyboardPlatform) -> Self {
        return Binding { platform: Some(platform), ..self };
    }

    /// What:
    ///      `repeating` lets holding the key repeat the action.
    /// Why:
    ///      Seeking and moving a list ring should keep going while the key is held.
    const fn repeating(self) -> Self {
        return Binding { repeats: true, ..self };
    }

    /// What:
    ///      `in_menu` makes the row apply while the track menu is open.
    /// Why:
    ///      D106 lets the row chords act on the row the menu is open on.
    const fn in_menu(self) -> Self {
        return Binding { menu: true, ..self };
    }

    /// What:
    ///      `applies_on` tells whether the row exists on this map.
    /// Why:
    ///      A row restricted to one map is invisible on the other.
    pub fn applies_on(&self, platform: KeyboardPlatform) -> bool {
        return self.platform.is_none_or(|only| return only == platform);
    }

    /// What:
    ///      `required` gives the four modifier flags (control,
    ///  meta,
    ///  alt,
    ///  shift) this row needs on a map.
    /// Why:
    ///      The primary modifier is Control on Windows and Linux and Command on macOS,
    ///  and a key press must match all four flags exactly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function required(b: Binding, platform: KeyboardPlatform): [boolean, boolean, boolean, boolean];
    /// ```
    pub fn required(&self, platform: KeyboardPlatform) -> HeldModifiers {
        let mut held = HeldModifiers { ctrl: false, meta: false, alt: false, shift: false };
        for modifier in self.modifiers {
            match (modifier, platform) {
                (Modifier::Primary, KeyboardPlatform::WindowsLinux) | (Modifier::Ctrl, _) => held.ctrl = true,
                (Modifier::Primary, KeyboardPlatform::Mac) => held.meta = true,
                (Modifier::Alt, _) => held.alt = true,
                (Modifier::Shift, _) => held.shift = true,
            }
        }
        return held;
    }

    /// What:
    ///      `matches_key` tells whether a key name triggers this row.
    /// Why:
    ///      Letters and digits arrive in either case (Shift changes them),
    ///  while named keys match exactly.
    pub fn matches_key(&self, key: &str) -> bool {
        return self.keys.iter().any(|name| {
            return if name.chars().count() == 1 { name.eq_ignore_ascii_case(key) } else { *name == key };
        });
    }
}

/// What:
///      `pub struct HeldModifiers` is the set of modifier keys down at one moment.
/// Why:
///      Both the classifier and the key hint badges compare a held set with the set a row needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldModifiers = { ctrl: boolean; meta: boolean; alt: boolean; shift: boolean };
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HeldModifiers {
    /// Control is down.
    pub ctrl: bool,
    /// Command (macOS) or Meta is down.
    pub meta: bool,
    /// Alt or Option is down.
    pub alt: bool,
    /// Shift is down.
    pub shift: bool,
}

/// Methods of `HeldModifiers`.
impl HeldModifiers {
    /// What:
    ///      `is_empty` is true when no modifier is down.
    /// Why:
    ///      Badges show only while at least one modifier is held.
    pub fn is_empty(self) -> bool {
        return !(self.ctrl || self.meta || self.alt || self.shift);
    }
}

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
    row(ShortcutAction::MenuUp, NONE, &["ArrowUp"], Scope::Popup).repeating().in_menu(),
    row(ShortcutAction::MenuDown, NONE, &["ArrowDown"], Scope::Popup).repeating().in_menu(),
    row(ShortcutAction::MenuChoose, NONE, &["Enter"], Scope::Popup).in_menu(),
    // Escape, from the innermost thing outward: the picker overlay, the editor line, the Settings list, Search, Licenses.
    row(ShortcutAction::ClosePicker, NONE, &["Escape"], Scope::PickerOverlay),
    row(ShortcutAction::SettingsFocusList, NONE, &["Escape"], Scope::SettingsField),
    row(ShortcutAction::Back, NONE, &["Escape"], Scope::SettingsIdle),
    row(ShortcutAction::EditorLeave, NONE, &["Escape"], Scope::EditorPage),
    row(ShortcutAction::Back, NONE, &["Escape"], Scope::SearchPage),
    row(ShortcutAction::Back, NONE, &["Escape"], Scope::LicensesPage),
    // Editor pages: Control+Enter applies and leaves, Control+Shift+Backspace resets the template.
    row(ShortcutAction::EditorApply, P, &["Enter"], Scope::EditorOrSettings),
    row(ShortcutAction::EditorReset, PS, &["Backspace"], Scope::EditorOrSettings),
    // Search: Up and Down move through the results even while the query field has focus; Enter opens the result.
    row(ShortcutAction::SearchUp, NONE, &["ArrowUp"], Scope::SearchPage).repeating(),
    row(ShortcutAction::SearchDown, NONE, &["ArrowDown"], Scope::SearchPage).repeating(),
    row(ShortcutAction::SearchOpen, NONE, &["Enter"], Scope::SearchField),
    // Kept from D102.
    row(ShortcutAction::Search, P, &["f"], Scope::PlayerPage),
    row(ShortcutAction::Picker, P, &["o"], Scope::PlayerPage),
    row(ShortcutAction::Settings, PA, &["s"], Scope::PlayerPage).on(KeyboardPlatform::WindowsLinux),
    row(ShortcutAction::Settings, P, &[","], Scope::PlayerPage).on(KeyboardPlatform::Mac),
    row(ShortcutAction::Reveal, P, &["g"], Scope::PlayerPage).on(KeyboardPlatform::WindowsLinux),
    row(ShortcutAction::Reveal, P, &["l"], Scope::PlayerPage).on(KeyboardPlatform::Mac),
    row(ShortcutAction::Toggle, NONE, &[" "], Scope::SpaceZones),
    row(ShortcutAction::Mode, C, &["m"], Scope::PlaybackArea),
    row(ShortcutAction::Previous, C, &["ArrowLeft"], Scope::PlaybackArea).on(KeyboardPlatform::WindowsLinux),
    row(ShortcutAction::Next, C, &["ArrowRight"], Scope::PlaybackArea).on(KeyboardPlatform::WindowsLinux),
    row(ShortcutAction::Previous, PS, &["[", "{"], Scope::PlaybackArea).on(KeyboardPlatform::Mac),
    row(ShortcutAction::Next, PS, &["]", "}"], Scope::PlaybackArea).on(KeyboardPlatform::Mac),
    // Playback (D106): seek 10 s and 60 s, pick a mode, restart the track.
    row(ShortcutAction::SeekBack, A, &["ArrowLeft"], Scope::PlayerPage).repeating(),
    row(ShortcutAction::SeekForward, A, &["ArrowRight"], Scope::PlayerPage).repeating(),
    row(ShortcutAction::SeekBackLong, AS, &["ArrowLeft"], Scope::PlayerPage).repeating(),
    row(ShortcutAction::SeekForwardLong, AS, &["ArrowRight"], Scope::PlayerPage).repeating(),
    row(ShortcutAction::ModeRepeat, PS, &["1", "!"], Scope::PlayerPage),
    row(ShortcutAction::ModeInOrder, PS, &["2", "@"], Scope::PlayerPage),
    row(ShortcutAction::ModeShuffleFolder, PS, &["3", "#"], Scope::PlayerPage),
    row(ShortcutAction::ModeShuffleAll, PS, &["4", "$"], Scope::PlayerPage),
    row(ShortcutAction::Restart, P, &["Home"], Scope::PlaybackArea),
    // Browsing (D106).
    row(ShortcutAction::PreviousFolder, A, &["ArrowUp"], Scope::PlayerPage),
    row(ShortcutAction::NextFolder, A, &["ArrowDown"], Scope::PlayerPage),
    row(ShortcutAction::RevealFolder, PS, &["g"], Scope::PlayerPage),
    row(ShortcutAction::FocusFolders, A, &["1"], Scope::PlayerPage),
    row(ShortcutAction::FocusTracks, A, &["2"], Scope::PlayerPage),
    row(ShortcutAction::OpenFolderDialog, PS, &["o"], Scope::PlayerPage),
    row(ShortcutAction::Quit, P, &["q"], Scope::Anywhere),
    // Inside a focused list: the arrows, Home, End, the page keys, Enter. A letter has its own rule in the classifier.
    row(ShortcutAction::ListUp, NONE, &["ArrowUp"], Scope::ListZone).repeating(),
    row(ShortcutAction::ListDown, NONE, &["ArrowDown"], Scope::ListZone).repeating(),
    row(ShortcutAction::ListUp, NONE, &["ArrowLeft"], Scope::FolderList).repeating(),
    row(ShortcutAction::ListDown, NONE, &["ArrowRight"], Scope::FolderList).repeating(),
    row(ShortcutAction::ListHome, NONE, &["Home"], Scope::ListZone),
    row(ShortcutAction::ListEnd, NONE, &["End"], Scope::ListZone),
    row(ShortcutAction::ListPageUp, NONE, &["PageUp"], Scope::ListZone).repeating(),
    row(ShortcutAction::ListPageDown, NONE, &["PageDown"], Scope::ListZone).repeating(),
    row(ShortcutAction::ListActivate, NONE, &["Enter"], Scope::ListZone),
    // The focused track row: its menu (Menu key or Alt+Enter), trash, copy and show.
    row(ShortcutAction::RowMenu, NONE, &["ContextMenu"], Scope::TrackList),
    row(ShortcutAction::RowMenu, A, &["Enter"], Scope::TrackList),
    row(ShortcutAction::RowTrash, NONE, &["Delete"], Scope::TrackList).in_menu(),
    row(ShortcutAction::RowCopy, PS, &["c"], Scope::TrackList).in_menu(),
    row(ShortcutAction::RowShow, PS, &["e"], Scope::TrackList).in_menu(),
];

/// What:
///      `request_held` reads the four modifier flags of a key press.
/// Why:
///      A row matches only when the held flags equal the flags it requires,
///  so Control+Shift+F is not Control+F.
fn request_held(request: &ShortcutRequest) -> HeldModifiers {
    return HeldModifiers { ctrl: request.ctrl, meta: request.meta, alt: request.alt, shift: request.shift };
}

/// What:
///      `binding_matching` finds the first row that the key press hits,
///  skipping rows that exist only while the track menu is open.
/// Why:
///      The first matching row decides,
///  and a row applies only on its map,
///  with its exact modifiers,
///  in its scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bindingMatching = (r: ShortcutRequest) => BINDINGS.find(b => matches(b, r) && b.scope !== "popup");
/// ```
pub fn binding_matching(request: &ShortcutRequest) -> Option<&'static Binding> {
    return BINDINGS.iter().find(|binding| {
        return binding.scope != Scope::Popup && row_hits(binding, request) && scope_allows(binding.scope, request);
    });
}

/// What:
///      `popup_binding_matching` finds the first row that applies while the track menu is open.
/// Why:
///      With the menu open only the rows marked for it apply,
///  and their own scope (the focused list) is replaced by the open menu.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const popupBindingMatching = (r: ShortcutRequest) => BINDINGS.find(b => b.menu && matches(b, r));
/// ```
pub fn popup_binding_matching(request: &ShortcutRequest) -> Option<&'static Binding> {
    return BINDINGS
        .iter()
        .find(|binding| return binding.menu && row_hits(binding, request) && request.route == crate::keymap::KeyRoute::Player);
}

/// What:
///      `row_hits` tells whether a row's platform,
///  modifiers and key match the key press.
/// Why:
///      Both lookups share the same three tests and differ only in the scope test.
fn row_hits(binding: &Binding, request: &ShortcutRequest) -> bool {
    return binding.applies_on(request.platform)
        && binding.required(request.platform) == request_held(request)
        && binding.matches_key(&request.key);
}

/// What:
///      `binding_for` finds the first row of an action on a map.
/// Why:
///      Tooltips,
///  menu rows and badges show the chord of an action,
///  and an action with two rows shows its first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bindingFor = (a: ShortcutAction, p: KeyboardPlatform) => BINDINGS.find(b => b.action === a && appliesOn(b, p));
/// ```
pub fn binding_for(action: ShortcutAction, platform: KeyboardPlatform) -> Option<&'static Binding> {
    return BINDINGS.iter().find(|binding| return binding.action == action && binding.applies_on(platform));
}

/// What:
///      `key_label` writes a key name the way a person reads it.
/// Why:
///      Badges and tooltips show `Left`,
///  `Space` and `F`,
///  not the DOM names `ArrowLeft`,
///  ` ` and `f`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keyLabel = (key: string) => ({ ArrowLeft: "Left", " ": "Space", ContextMenu: "Menu" })[key] ?? key.toUpperCase();
/// ```
pub fn key_label(key: &str) -> String {
    return match key {
        "ArrowLeft" => "Left".to_owned(),
        "ArrowRight" => "Right".to_owned(),
        "ArrowUp" => "Up".to_owned(),
        "ArrowDown" => "Down".to_owned(),
        " " => "Space".to_owned(),
        "ContextMenu" => "Menu".to_owned(),
        named if named.chars().count() > 1 => named.to_owned(),
        single => single.to_uppercase(),
    };
}

/// What:
///      `modifier_label` writes one modifier for a map.
/// Why:
///      Windows and Linux spell Ctrl,
///  Alt and Shift,
///  and macOS shows the Command,
///  Option,
///  Shift and Control glyphs.
fn modifier_label(modifier: Modifier, platform: KeyboardPlatform) -> &'static str {
    return match (modifier, platform) {
        (Modifier::Primary, KeyboardPlatform::WindowsLinux) | (Modifier::Ctrl, KeyboardPlatform::WindowsLinux) => "Ctrl",
        (Modifier::Primary, KeyboardPlatform::Mac) => "\u{2318}",
        (Modifier::Ctrl, KeyboardPlatform::Mac) => "\u{2303}",
        (Modifier::Alt, KeyboardPlatform::WindowsLinux) => "Alt",
        (Modifier::Alt, KeyboardPlatform::Mac) => "\u{2325}",
        (Modifier::Shift, KeyboardPlatform::WindowsLinux) => "Shift",
        (Modifier::Shift, KeyboardPlatform::Mac) => "\u{21E7}",
    };
}

/// What:
///      `chord_text` writes a row as the chord a person presses,
///  such as `Ctrl+Shift+O` or the macOS glyphs followed by the key.
/// Why:
///      Tooltips and menu rows show the same text for the same row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const chordText = (b: Binding, p: KeyboardPlatform) => [...b.modifiers.map(label), keyLabel(b.keys[0])].join(p === "mac" ? "" : "+");
/// ```
pub fn chord_text(binding: &Binding, platform: KeyboardPlatform) -> String {
    let key = binding.keys.first().map_or_else(String::new, |name| return key_label(name));
    let mut parts: Vec<String> = binding
        .modifiers
        .iter()
        .map(|modifier| return modifier_label(*modifier, platform).to_owned())
        .collect();
    parts.push(key);
    return match platform {
        KeyboardPlatform::WindowsLinux => parts.join("+"),
        KeyboardPlatform::Mac => parts.concat(),
    };
}

/// What:
///      `shortcut_text` writes the chord of an action on a map,
///  or an empty string when the action has none there.
/// Why:
///      Tooltips and menu rows ask by action,
///  and an empty text means nothing is drawn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shortcutText = (a: ShortcutAction, p: KeyboardPlatform) => { const b = bindingFor(a, p); return b ? chordText(b, p) : ""; };
/// ```
pub fn shortcut_text(action: ShortcutAction, platform: KeyboardPlatform) -> String {
    return binding_for(action, platform).map_or_else(String::new, |binding| return chord_text(binding, platform));
}

/// Unit tests in `keymap_bindings_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "keymap_bindings_tests.rs"]
mod tests;
