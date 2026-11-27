//! Writing a binding as the text a person reads.
//!
//! What:
//! `key_label` names one key,
//! `modifier_label` names one modifier on one map,
//! `chord_text` joins a row's modifiers and first key into one chord,
//! and `shortcut_text` does the same for an action.
//! Why:
//! tooltips,
//! menu rows and badges must spell a chord exactly as the table spells it,
//! and this file is split out of `src/keymap_bindings.rs` so each stays within
//! the line budget.

/// What:
///      `use super::{...}` brings the lookup that finds an action's row,
///  and the row and modifier types its text is written from.
/// Why:
///      The text of a chord is decided by the row,
///  so this module reads the parent module's types instead of restating them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { binding_for, Binding, Modifier } from "./keymap_bindings";
/// ```
use super::{
    Binding,
    Modifier,
    binding_for,
};

/// What:
///      `use crate::keymap::{...}` brings the action and the map a chord is written for.
/// Why:
///      The same row is spelled with Ctrl on Windows and Linux,
///  and with the Command glyph on macOS,
///  and that choice is the map's.
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
fn modifier_label(
    modifier: Modifier,
    platform: KeyboardPlatform,
) -> &'static str {
    return match (modifier, platform) {
        (Modifier::Primary, KeyboardPlatform::WindowsLinux)
        | (Modifier::Ctrl, KeyboardPlatform::WindowsLinux) => "Ctrl",
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
pub fn chord_text(
    binding: &Binding,
    platform: KeyboardPlatform,
) -> String {
    let key = match binding.keys.first() {
        Some(name) => key_label(name),
        None => String::new(),
    };
    let mut parts: Vec<String> = Vec::new();
    for modifier in binding.modifiers.iter() {
        parts.push(modifier_label(*modifier, platform).to_owned());
    }
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
pub fn shortcut_text(
    action: ShortcutAction,
    platform: KeyboardPlatform,
) -> String {
    return match binding_for(action, platform) {
        Some(binding) => chord_text(binding, platform),
        None => String::new(),
    };
}
