//! The D106 binding table:
//! every chord of the desktop player as plain data.
//!
//! What:
//! `BINDINGS` lists one row per chord with its modifiers,
//! key names,
//! platform,
//! scope and repeat rule;
//! `binding_matching` finds the row a key press hits;
//! `shortcut_text` writes a row the way a tooltip or a menu row
//! shows it.
//! The table itself lives in `src/keymap_bindings_table.rs`,
//! the lookups in `src/keymap_bindings_lookup.rs` and the text writers in
//! `src/keymap_bindings_chord_text.rs`;
//! this file holds the row types and their builders.
//! Why:
//! the classifier,
//! the key hint badges,
//! the tooltips and the menu rows must agree on every chord,
//! so they all
//! read this one table and nothing else spells a chord out.

/// Imports the platform and action types that the row type and its builders name.
use crate::keymap::{
    KeyboardPlatform,
    ShortcutAction,
};

/// What:
///      `use crate::keymap::ShortcutRequest;` brings the key press type the unit
///           tests build.
/// Why:
///       Only `keymap_bindings_tests.rs` names it,
///  through its `use super::*`,
///  so the import exists in the test build alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { ShortcutRequest } from "./keymap";
/// ```
#[cfg(test)]
use crate::keymap::ShortcutRequest;

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
///      Plain data keeps the table auditable:
/// each row can be read against D106 line by line.
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
    /// Key names that trigger it:
    /// DOM-style names for named keys,
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
const fn row(
    action: ShortcutAction,
    modifiers: &'static [Modifier],
    keys: &'static [&'static str],
    scope: Scope,
) -> Binding {
    return Binding {
        action,
        platform: None,
        modifiers,
        keys,
        scope,
        repeats: false,
        menu: false,
    };
}

/// Builder methods of `Binding` for the table's exceptions.
impl Binding {
    /// What:
    ///      `on` restricts a row to one map.
    /// Why:
    ///      D102 gives different chords for Settings,
    ///  reveal and track change on macOS.
    const fn on(
        self,
        platform: KeyboardPlatform,
    ) -> Self {
        return Binding {
            platform: Some(platform),
            ..self
        };
    }

    /// What:
    ///      `repeating` lets holding the key repeat the action.
    /// Why:
    ///      Seeking and moving a list ring should keep going while the key is held.
    const fn repeating(self) -> Self {
        return Binding {
            repeats: true,
            ..self
        };
    }

    /// What:
    ///      `in_menu` makes the row apply while the track menu is open.
    /// Why:
    ///      D106 lets the row chords act on the row the menu is open on.
    const fn in_menu(self) -> Self {
        return Binding {
            menu: true,
            ..self
        };
    }

    /// What:
    ///      `applies_on` tells whether the row exists on this map.
    /// Why:
    ///      A row restricted to one map is invisible on the other.
    pub fn applies_on(
        &self,
        platform: KeyboardPlatform,
    ) -> bool {
        return match self.platform {
            Some(only) => only == platform,
            None => true,
        };
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
    pub fn required(
        &self,
        platform: KeyboardPlatform,
    ) -> HeldModifiers {
        let mut held = HeldModifiers {
            ctrl: false,
            meta: false,
            alt: false,
            shift: false,
        };
        for modifier in self.modifiers {
            match (modifier, platform) {
                (Modifier::Primary, KeyboardPlatform::WindowsLinux) | (Modifier::Ctrl, _) => {
                    held.ctrl = true
                }
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
    pub fn matches_key(
        &self,
        key: &str,
    ) -> bool {
        for name in self.keys.iter() {
            let same = if name.chars().count() == 1 {
                name.eq_ignore_ascii_case(key)
            } else {
                *name == key
            };
            if same {
                return true;
            }
        }
        return false;
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
///      `mod keymap_bindings_table;` loads the sibling
///           `src/keymap_bindings_table.rs`,
///  the D106 table itself.
/// Why:
///       The table is one row per chord and is read against D106 line by line,
///           so it keeps a file of its own while this module holds the row types
///           and their builders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as keymapBindingsTable from "./keymap_bindings_table";
/// ```
#[path = "keymap_bindings_table.rs"]
mod keymap_bindings_table;

/// What:
///      `pub use keymap_bindings_table::BINDINGS;` re-exports the table at this
///           module's path.
/// Why:
///       Callers name `keymap_bindings::BINDINGS`,
///  and only where the static is declared moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { BINDINGS } from "./keymap_bindings_table";
/// ```
pub use keymap_bindings_table::BINDINGS;

/// What:
///      `mod keymap_bindings_lookup;` loads the sibling
///           `src/keymap_bindings_lookup.rs`,
///  which finds the row a key press or an action uses.
/// Why:
///       The lookups are the table's only readers,
///           so they sit in their own file instead of at the foot of this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as keymapBindingsLookup from "./keymap_bindings_lookup";
/// ```
#[path = "keymap_bindings_lookup.rs"]
mod keymap_bindings_lookup;

/// What:
///      `pub use keymap_bindings_lookup::{...}` re-exports the three lookups at
///           this module's path.
/// Why:
///       Callers name `keymap_bindings::binding_matching` and its siblings,
///  and only where they are declared moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { binding_matching, popup_binding_matching, binding_for } from "./keymap_bindings_lookup";
/// ```
pub use keymap_bindings_lookup::{
    binding_for,
    binding_matching,
    popup_binding_matching,
};

/// What:
///      `mod keymap_bindings_chord_text;` loads the sibling
///           `src/keymap_bindings_chord_text.rs`,
///  which writes a row or an action as the text a person reads.
/// Why:
///       Badges,
///  tooltips and menu rows all draw that text,
///  and it is a spelling table of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as keymapBindingsChordText from "./keymap_bindings_chord_text";
/// ```
#[path = "keymap_bindings_chord_text.rs"]
mod keymap_bindings_chord_text;

/// What:
///      `pub use keymap_bindings_chord_text::{...}` re-exports the chord text
///           writers at this module's path.
/// Why:
///       Callers name `keymap_bindings::shortcut_text` and its siblings,
///  and only where they are declared moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { key_label, chord_text, shortcut_text } from "./keymap_bindings_chord_text";
/// ```
pub use keymap_bindings_chord_text::{
    chord_text,
    key_label,
    shortcut_text,
};

/// Unit tests in `keymap_bindings_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "keymap_bindings_tests.rs"]
mod tests;
