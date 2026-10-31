//! What:
//!  Format-preserving edits to a TOML document: key lookup, guarded assignment, and block append.
//! Why:
//!  `Cargo.toml` files are hand-written: comments, key order and keys the tool does not own must survive, and a
//!  value that already equals the wanted one must not be rewritten (so an unchanged file stays byte-identical).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function setKey(doc: TomlDocument, path: string[], value: Evaluated): boolean
//! ```

use crate::value_eval::Evaluated;
use toml_edit::{Array, DocumentMut, Item, Value};

/// What:
///  Look up a key path as an item, if every step exists.
///  `Option<&Item>` is a borrowed item or nothing (TS: `Item | undefined`).
/// Why:
///  Guards and block checks only ask whether a path exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lookup(doc: TomlDocument, path: string[]): Item | undefined
/// ```
pub fn lookup<'a>(document: &'a DocumentMut, path: &[String]) -> Option<&'a Item> {
    let mut current: &Item = document.as_item();
    for key in path {
        current = current.get(key.as_str())?;
    }
    return Some(current);
}

/// What:
///  Whether an existing item already holds the wanted value.
/// Why:
///  Equal values are left untouched so formatting and comments around them survive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function itemMatches(item: Item, wanted: Evaluated): boolean
/// ```
fn item_matches(item: &Item, wanted: &Evaluated) -> bool {
    let Some(value) = item.as_value() else { return false };
    match wanted {
        Evaluated::Text(text) => return value.as_str() == Some(text.as_str()),
        Evaluated::Bool(flag) => return value.as_bool() == Some(*flag),
        Evaluated::Integer(number) => return value.as_integer() == Some(*number),
        Evaluated::List(items) => {
            let Some(array) = value.as_array() else { return false };
            if array.len() != items.len() {
                return false;
            }
            for (position, expected) in items.iter().enumerate() {
                let Some(element) = array.get(position) else { return false };
                if !item_matches(&Item::Value(element.clone()), expected) {
                    return false;
                }
            }
            return true;
        }
    }
}

/// What:
///  Convert a computed value into a TOML value.
/// Why:
///  Edits insert what the config computed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function toTomlValue(value: Evaluated): Value
/// ```
fn to_toml_value(wanted: &Evaluated) -> Value {
    match wanted {
        Evaluated::Text(text) => return Value::from(text.as_str()),
        Evaluated::Bool(flag) => return Value::from(*flag),
        Evaluated::Integer(number) => return Value::from(*number),
        Evaluated::List(items) => {
            let mut array = Array::new();
            for item in items {
                array.push(to_toml_value(item));
            }
            return Value::Array(array);
        }
    }
}

/// What:
///  Set `path` to `wanted` when the guard path exists and the value differs; return whether anything changed.
///  Intermediate tables must already exist; the last key is inserted or replaced in its parent table.
/// Why:
///  The tool owns a key only where the crate already opted in with the guard.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setKey(doc: TomlDocument, guard: string[], path: string[], wanted: Evaluated): boolean
/// ```
pub fn set_key(document: &mut DocumentMut, guard: &[String], path: &[String], wanted: &Evaluated) -> bool {
    if !guard.is_empty() && lookup(document, guard).is_none() {
        return false;
    }
    let Some((last, parents)) = path.split_last() else { return false };
    if let Some(existing) = lookup(document, path)
        && item_matches(existing, wanted)
    {
        return false;
    }
    let mut current: &mut Item = document.as_item_mut();
    for key in parents {
        let Some(next) = current.get_mut(key.as_str()) else { return false };
        current = next;
    }
    let new_value = to_toml_value(wanted);
    if let Some(table) = current.as_table_like_mut() {
        if let Some(old) = table.get_mut(last.as_str()) {
            let mut replacement = new_value;
            if let Some(old_value) = old.as_value() {
                *replacement.decor_mut() = old_value.decor().clone();
            }
            *old = Item::Value(replacement);
        } else {
            table.insert(last.as_str(), Item::Value(new_value));
        }
        return true;
    }
    return false;
}

/// What:
///  Append a block of TOML text after the document, separated by one blank line.
/// Why:
///  Whole tables such as `[lints.clippy]` are added once, as written in the content file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function appendBlock(text: string, block: string): string
/// ```
pub fn append_block(text: &str, block: &str) -> String {
    let mut result = text.to_string();
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    if !result.is_empty() {
        result.push('\n');
    }
    result.push_str(block);
    if !result.ends_with('\n') {
        result.push('\n');
    }
    return result;
}
