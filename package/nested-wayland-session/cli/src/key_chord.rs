//! Chord specs for the `key` command:
//!  `ctrl+shift+f10` is the keys Ctrl and Shift held while F10 is tapped.

/// Sibling modules of this crate.
use crate::key_name::resolve;

/// A chord as evdev codes in press order:
///  the modifiers first,
///  the target key last.
///
/// What:
///      `pub struct Chord { keys: Vec<u32> }`.
///  Never empty.
/// Why:
///       Press goes forward through the keys and release goes back through them,
///  the order a person's fingers use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Chord = { keys: number[] };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    /// Evdev codes,
    ///  modifiers first,
    ///  target last.
    pub keys: Vec<u32>,
}

/// Split a spec into its modifier names and its target name.
///
/// What:
///      `fn split_spec(spec: &str) -> Result<(Vec<&str>, &str), String>`.
///  `+` separates keys.
///  A spec that is `+`,
///  or ends in `++`,
///  has the plus key itself as the target.
///  A spec ending in a single `+` is a mistake and is refused.
/// Why:
///       `ctrl++` must mean Ctrl and the plus key,
///  and `ctrl+` must not silently mean something else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitSpec(spec: string): [string[], string] { ... }
/// ```
fn split_spec(spec: &str) -> Result<(Vec<&str>, &str), String> {
    if spec.is_empty() {
        return Err("key requires a name".to_string())
    }
    let (head, target) = if spec == "+" {
        ("", "+")
    } else if let Some(prefix) = spec.strip_suffix("++") {
        (prefix, "+")
    } else if spec.ends_with('+') {
        return Err(format!("key chord {spec} ends with a separator; write the plus key as ++"))
    } else {
        match spec.rsplit_once('+') {
            Some((modifiers, last)) => (modifiers, last),
            None => ("", spec),
        }
    };
    let modifiers: Vec<&str> = head.split('+').filter(|name| return !name.is_empty()).collect();
    return Ok((modifiers, target))
}

/// Parse a chord spec.
///
/// What:
///      `pub fn parse_chord(spec: &str) -> Result<Chord, String>`.
///  Every part resolves through `key_name::resolve`;
///  the first unknown name fails the whole chord as `unknown key: NAME`.
/// Why:
///       A chord with one bad name must not press the keys before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseChord(spec: string): Chord { ... }
/// ```
///
/// @example
/// ```ts
/// parseChord("ctrl+shift+f10"); // => { keys: [29, 42, 68] }
/// parseChord("ctrl++");         // => { keys: [29, 13] }  (the plus key is the = key)
/// ```
pub fn parse_chord(spec: &str) -> Result<Chord, String> {
    let (modifiers, target) = split_spec(spec)?;
    let mut keys = Vec::with_capacity(modifiers.len() + 1);
    for name in modifiers.iter().copied().chain(std::iter::once(target)) {
        let name_to_resolve = if name == "+" { "plus" } else { name };
        match resolve(name_to_resolve) {
            Some(code) => keys.push(code),
            None => return Err(format!("unknown key: {name}")),
        }
    }
    return Ok(Chord { keys })
}

/// What:
///      `#[cfg(test)] #[path = "key_chord_tests.rs"] mod tests;`.
/// Why:
///       Keep the chord grammar tests beside the parser.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./key_chord_tests";
/// ```
#[cfg(test)]
#[path = "key_chord_tests.rs"]
mod tests;
