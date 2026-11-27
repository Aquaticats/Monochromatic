//! Finding the binding a key press or an action uses.
//!
//! What:
//! `binding_matching`,
//! `popup_binding_matching` and `binding_for` walk the table in
//! `src/keymap_bindings_table.rs`,
//! and `row_hits` is the three-part test they share.
//! Why:
//! split out of `src/keymap_bindings.rs` so each file stays within the line budget,
//! while the row types and the table itself stay where they were.

/// What:
///      `use super::{...}` brings the table,
///  the row type,
///  the held-modifier set and the scope enum from the parent module `keymap_bindings`.
/// Why:
///      These lookups are the only readers of the table,
///  so they take it from the parent instead of each naming a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BINDINGS, Binding, HeldModifiers, Scope } from "./keymap_bindings";
/// ```
use super::{
    BINDINGS,
    Binding,
    HeldModifiers,
    Scope,
};

/// What:
///      `use crate::keymap::{...}` brings the key press,
///  the action,
///  the map and the scope test the lookups call.
/// Why:
///      A press,
///  its action and its map are the parent module's sibling types,
///  so a lookup and the classifier agree on what a press means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { KeyboardPlatform, ShortcutAction, ShortcutRequest, scopeAllows } from "./keymap";
/// ```
use crate::keymap::{
    KeyboardPlatform,
    ShortcutAction,
    ShortcutRequest,
    scope_allows,
};

/// What:
///      `request_held` reads the four modifier flags of a key press.
/// Why:
///      A row matches only when the held flags equal the flags it requires,
///  so Control+Shift+F is not Control+F.
fn request_held(request: &ShortcutRequest) -> HeldModifiers {
    return HeldModifiers {
        ctrl: request.ctrl,
        meta: request.meta,
        alt: request.alt,
        shift: request.shift,
    };
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
    for binding in BINDINGS.iter() {
        if binding.scope != Scope::Popup
            && row_hits(binding, request)
            && scope_allows(binding.scope, request)
        {
            return Some(binding);
        }
    }
    return None;
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
    for binding in BINDINGS.iter() {
        if binding.menu
            && row_hits(binding, request)
            && request.route == crate::keymap::KeyRoute::Player
        {
            return Some(binding);
        }
    }
    return None;
}

/// What:
///      `row_hits` tells whether a row's platform,
///  modifiers and key match the key press.
/// Why:
///      Both lookups share the same three tests and differ only in the scope test.
fn row_hits(
    binding: &Binding,
    request: &ShortcutRequest,
) -> bool {
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
pub fn binding_for(
    action: ShortcutAction,
    platform: KeyboardPlatform,
) -> Option<&'static Binding> {
    for binding in BINDINGS.iter() {
        if binding.action == action && binding.applies_on(platform) {
            return Some(binding);
        }
    }
    return None;
}
