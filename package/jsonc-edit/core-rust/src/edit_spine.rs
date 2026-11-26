//! What:
//!      The immutable edits that act on one address:
//!  replacing the addressed value,
//!  setting that value's
//!           comment,
//!  deleting the addressed member or element,
//!  and setting the comment of the key that names
//!           it.
//! Why:
//!       Each of them changes one node and returns a new document,
//!  leaving the input state usable,
//!  and
//!           each reports a missing address or a wrong-shaped target instead of panicking.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module editSpine: setValue, replaceComment, setKeyComment, deleteValue(root, path).
//! ```

/// What:
///      Import the shared walk the edits in this file are built on.
/// Why:
///       Descent and rebuild are the same for every edit,
///  so this file holds only what each edit does
///           to the node it reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { descend, rebuild, notFound, typeError } from './editSpineWalk';
/// ```
use crate::edit_spine_walk::{
    descend,
    not_found,
    rebuild,
    type_error,
};
/// What:
///      Import the edit failure enum.
/// Why:
///       Every edit here returns either a new document or one of its failures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncEditError } from './error';
/// ```
use crate::error::JsoncEditError;
/// What:
///      Import the member lookup and key-comparison helpers.
/// Why:
///       A key edit finds its member by the last-duplicate rule,
///  and a key delete compares the key's
///           UTF-16 units.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { findEntryIndex, keyUnits } from './navigate';
/// ```
use crate::navigate::{
    find_entry_index,
    key_units,
};
/// What:
///      Import the address segment type.
/// Why:
///       Each edit is given the address to act on and must reject a step of the wrong kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncPathSegment } from './path';
/// ```
use crate::path::JsoncPathSegment;
/// What:
///      Import the document model types the edits copy and construct.
/// Why:
///       A new document keeps the untouched siblings and the parent's own comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncComment, JsoncEntry, JsoncKey, JsoncKind, JsoncValue } from './value';
/// ```
use crate::value::{
    JsoncComment,
    JsoncEntry,
    JsoncKey,
    JsoncKind,
    JsoncValue,
};

/// What:
///      Return a new document with one address replaced,
///  creating only a missing final segment.
/// Why:
///       Replacement keeps the addressed node's own comment,
///  because that comment describes the
///           address rather than the incoming value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setValue(root: JsoncValue, path: JsoncPathSegment[], value: JsoncValue): JsoncValue;
/// ```
///
/// # Errors
/// Returns the missing address when a step names nothing,
///  or a shape failure when a step does not fit.
pub(crate) fn set_value(
    root: &JsoncValue,
    path: &[JsoncPathSegment],
    value: JsoncValue,
) -> Result<JsoncValue, JsoncEditError> {
    if path.is_empty() {
        return Ok(JsoncValue {
            kind: value.kind,
            comment: root.comment.clone(),
        });
    }
    let (levels, target) = descend(root, path, path, true)?;
    let replacement = match target {
        Some(addressed) => JsoncValue {
            kind: value.kind,
            comment: addressed.comment.clone(),
        },
        None => value,
    };
    return Ok(rebuild(levels, replacement));
}

/// What:
///      The node with its kind kept and its comment replaced.
/// Why:
///       A comment edit changes one field of one node,
///  and a named function carries the new comment as a parameter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withComment(node: JsoncValue, comment: JsoncComment | undefined): JsoncValue;
/// ```
fn with_comment(
    node: &JsoncValue,
    comment: Option<JsoncComment>,
) -> JsoncValue {
    return JsoncValue {
        kind: node.kind.clone(),
        comment,
    };
}

/// What:
///      Return a new document whose addressed value carries the given comment.
/// Why:
///       Comment edits change one field of one node and must not insert missing structure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setCommentAt(root, path, comment): JsoncValue;
/// ```
///
/// # Errors
/// Returns the missing address when a step names nothing,
///  or a shape failure when a step does not fit.
pub(crate) fn set_comment_at(
    root: &JsoncValue,
    path: &[JsoncPathSegment],
    comment: Option<JsoncComment>,
) -> Result<JsoncValue, JsoncEditError> {
    if path.is_empty() {
        return Ok(with_comment(root, comment));
    }
    let (levels, target) = descend(root, path, path, false)?;
    let Some(addressed) = target else {
        return Err(not_found(path));
    };
    return Ok(rebuild(levels, with_comment(addressed, comment)));
}

/// What:
///      Return a new document without the addressed member or element.
/// Why:
///       Deleting a key removes every member with that name,
///  matching the maintained TypeScript
///           behavior for unsupported duplicate keys.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deleteValue(root: JsoncValue, path: JsoncPathSegment[]): JsoncValue;
/// ```
///
/// # Errors
/// Returns a shape failure for an empty address or a mismatched step,
///  and the missing address when a step
/// names nothing.
pub(crate) fn delete_value(
    root: &JsoncValue,
    path: &[JsoncPathSegment],
) -> Result<JsoncValue, JsoncEditError> {
    let Some((last, parent_path)) = path.split_last() else {
        return Err(type_error(String::from(
            "jsonc delete: cannot delete the document root",
        )));
    };
    let (levels, target) = descend(root, parent_path, path, false)?;
    let Some(parent) = target else {
        return Err(not_found(path));
    };
    let rebuilt = match (&parent.kind, last) {
        (
            JsoncKind::Record {
                entries,
            },
            JsoncPathSegment::Key {
                key,
            },
        ) => {
            let wanted = key_units(key);
            let mut kept: Vec<JsoncEntry> = Vec::new();
            for entry in entries {
                if entry.key.units != wanted {
                    kept.push(entry.clone());
                }
            }
            JsoncValue {
                kind: JsoncKind::Record {
                    entries: kept,
                },
                comment: parent.comment.clone(),
            }
        }
        (
            JsoncKind::Array {
                elements,
            },
            JsoncPathSegment::Index {
                index,
            },
        ) => {
            let position = *index;
            if position >= elements.len() {
                return Err(not_found(path));
            }
            let mut kept: Vec<JsoncValue> = Vec::new();
            for (offset, element) in elements.iter().enumerate() {
                if offset != position {
                    kept.push(element.clone());
                }
            }
            JsoncValue {
                kind: JsoncKind::Array {
                    elements: kept,
                },
                comment: parent.comment.clone(),
            }
        }
        (
            JsoncKind::Record {
                ..
            },
            JsoncPathSegment::Index {
                index,
            },
        ) => {
            return Err(type_error(format!(
                "jsonc delete: cannot index an object with position {index}"
            )));
        }
        (
            JsoncKind::Array {
                ..
            },
            JsoncPathSegment::Key {
                key,
            },
        ) => {
            return Err(type_error(format!(
                "jsonc delete: cannot index an array with key {key:?}"
            )));
        }
        _ => {
            return Err(type_error(String::from(
                "jsonc delete: cannot index a scalar value",
            )));
        }
    };
    return Ok(rebuild(levels, rebuilt));
}

/// What:
///      Return a new document whose addressed member key carries the given comment.
/// Why:
///       A key comment lives on the parent record's member,
///  so the walk stops one segment early and
///           rewrites that member's key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setKeyComment(root, path, comment): JsoncValue;
/// ```
///
/// # Errors
/// Returns a shape failure for an empty or index-final address,
///  and the missing address when the parent is
/// not a record or the member does not exist.
pub(crate) fn set_key_comment(
    root: &JsoncValue,
    path: &[JsoncPathSegment],
    comment: Option<JsoncComment>,
) -> Result<JsoncValue, JsoncEditError> {
    let Some((last, parent_path)) = path.split_last() else {
        return Err(type_error(String::from(
            "jsonc key comment: empty path has no key",
        )));
    };
    let JsoncPathSegment::Key {
        key,
    } = last
    else {
        return Err(type_error(String::from(
            "jsonc key comment: final segment is not a key",
        )));
    };
    let (levels, target) = descend(root, parent_path, path, false)?;
    let Some(parent) = target else {
        return Err(not_found(path));
    };
    let JsoncKind::Record {
        entries,
    } = &parent.kind
    else {
        return Err(not_found(path));
    };
    let Some(index) = find_entry_index(entries, key) else {
        return Err(not_found(path));
    };
    let mut rebuilt: Vec<JsoncEntry> = Vec::with_capacity(entries.len());
    for (position, entry) in entries.iter().enumerate() {
        if position == index {
            rebuilt.push(JsoncEntry {
                key: JsoncKey {
                    comment: comment.clone(),
                    ..entry.key.clone()
                },
                value: entry.value.clone(),
            });
        } else {
            rebuilt.push(entry.clone());
        }
    }
    let updated = JsoncValue {
        kind: JsoncKind::Record {
            entries: rebuilt,
        },
        comment: parent.comment.clone(),
    };
    return Ok(rebuild(levels, updated));
}
