//! What:
//!      The iterative spine walk that every immutable edit shares:
//!  descend an address while cloning
//!           each level's children,
//!  then rebuild the document bottom-up.
//! Why:
//!       A recursive rebuild overflows a debug test thread's stack at the 512-container depth this
//!           crate accepts,
//!  so the walk keeps its own explicit stack of levels instead of call frames.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module editSpine: descend(root, path), rebuild(levels, value), set/delete/replace entry points.
//! ```

/// What:
///      Import the edit failure constructors' underlying types.
/// Why:
///       The walk reports a missing address and a wrong-shaped target as different failures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncEditError, JsoncPathNotFoundError, JsoncTypeError } from './error';
/// ```
use crate::error::{JsoncEditError, JsoncPathNotFoundError, JsoncTypeError};
/// What:
///      Import the member lookup helper.
/// Why:
///       Duplicate keys resolve to the last member,
///  the same rule reads use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { findEntryIndex, keyUnits } from './navigate';
/// ```
use crate::navigate::{find_entry_index, key_units};
/// What:
///      Import the address segment type.
/// Why:
///       Each step is either a key or an index,
///  and the walk must reject a mismatch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncPathSegment } from './path';
/// ```
use crate::path::JsoncPathSegment;
/// What:
///      Import the document model types the walk clones and rebuilds.
/// Why:
///       A level holds its children so one slot can be replaced without touching the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncComment, JsoncEntry, JsoncKey, JsoncKind, JsoncValue } from './value';
/// ```
use crate::value::{JsoncComment, JsoncEntry, JsoncKey, JsoncKind, JsoncValue};

/// What:
///      One descended level:
///  a container's cloned children,
///  the slot addressed there,
///  and the
///           container's own comment.
/// Why:
///       Rebuilding needs the untouched siblings and the parent comment,
///  and an absent slot index
///           means "insert here" rather than "replace here".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SpineLevel = { kind: 'record'; entries: JsoncEntry[]; index?: number; insertKey?: string; comment?: JsoncComment }
///                 | { kind: 'array'; elements: JsoncValue[]; index?: number; comment?: JsoncComment };
/// ```
pub(crate) enum SpineLevel {
    /// A record level with its members cloned in source order.
    Record {
        /// What:
        ///     Members cloned from the source document.
        /// Why:
        ///      `entries` stores the siblings so one slot can change without re-reading the original.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// entries: JsoncEntry[];
        /// ```
        entries: Vec<JsoncEntry>,
        /// What:
        ///     The member position addressed at this level,
        ///  or absence for an insertion.
        /// Why:
        ///      `index` stores the resolved position,
        ///  so rebuilding never re-searches the members.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// index?: number;
        /// ```
        index: Option<usize>,
        /// What:
        ///     The key text to append when this level inserts a new member.
        /// Why:
        ///      `insert_key` stores the caller's requested name,
        ///  which the document does not hold yet.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// insertKey?: string;
        /// ```
        insert_key: Option<String>,
        /// What:
        ///     The record's own comment.
        /// Why:
        ///      `comment` stores it so rebuilding preserves the container's attached comment.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// comment?: JsoncComment;
        /// ```
        comment: Option<JsoncComment>,
    },
    /// An array level with its elements cloned in source order.
    Array {
        /// What:
        ///     Elements cloned from the source document.
        /// Why:
        ///      `elements` stores the siblings so one slot can change without re-reading the original.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// elements: JsoncValue[];
        /// ```
        elements: Vec<JsoncValue>,
        /// What:
        ///     The element position addressed at this level,
        ///  or absence for an append.
        /// Why:
        ///      `index` stores the resolved position,
        ///  so rebuilding never re-checks the bounds.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// index?: number;
        /// ```
        index: Option<usize>,
        /// What:
        ///     The array's own comment.
        /// Why:
        ///      `comment` stores it so rebuilding preserves the container's attached comment.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// comment?: JsoncComment;
        /// ```
        comment: Option<JsoncComment>,
    },
}

/// What:
///      Build the missing-address failure for one requested path.
/// Why:
///       Every absent-address branch reports the address the caller supplied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function notFound(path: JsoncPathSegment[]): JsoncEditError;
/// ```
pub(crate) fn not_found(path: &[JsoncPathSegment]) -> JsoncEditError {
    return JsoncEditError::PathNotFound {
        error: JsoncPathNotFoundError { path: Vec::from(path) },
    };
}

/// What:
///      Build the wrong-shape failure for one message.
/// Why:
///       Indexing a scalar,
///  or an object with a position,
///  is a caller mistake worth naming.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function typeError(message: string): JsoncEditError;
/// ```
pub(crate) fn type_error(message: String) -> JsoncEditError {
    return JsoncEditError::Type { error: JsoncTypeError { message } };
}

/// What:
///      Walk an address,
///  cloning each container level,
///  and return the levels with the addressed
///           value when the whole path resolved.
/// Why:
///       Descent and rebuild are separate phases so neither needs a call frame per container,
///  which
///           is what keeps the accepted 512-depth document editable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function descend(root: JsoncValue, path: JsoncPathSegment[]): [SpineLevel[], JsoncValue | undefined];
/// ```
///
/// # Errors
/// Returns the missing address when a step names nothing,
///  or a shape failure when a step does not fit its
/// target.
///  With `allow_final_insert`,
///  a missing final segment records an insertion instead of failing.
pub(crate) fn descend<'a>(
    root: &'a JsoncValue,
    path: &[JsoncPathSegment],
    full_path: &[JsoncPathSegment],
    allow_final_insert: bool,
) -> Result<(Vec<SpineLevel>, Option<&'a JsoncValue>), JsoncEditError> {
    let mut levels: Vec<SpineLevel> = Vec::with_capacity(path.len());
    let mut current = root;
    for (depth, segment) in path.iter().enumerate() {
        let is_last = depth + 1 == path.len();
        match (&current.kind, segment) {
            (JsoncKind::Record { entries }, JsoncPathSegment::Key { key }) => {
                match find_entry_index(entries, key) {
                    Some(index) => {
                        let child = entries.get(index).ok_or_else(|| return not_found(full_path))?;
                        levels.push(SpineLevel::Record {
                            entries: entries.clone(),
                            index: Some(index),
                            insert_key: None,
                            comment: current.comment.clone(),
                        });
                        current = &child.value;
                    }
                    None => {
                        if !is_last || !allow_final_insert {
                            return Err(not_found(full_path));
                        }
                        levels.push(SpineLevel::Record {
                            entries: entries.clone(),
                            index: None,
                            insert_key: Some(String::from(key.as_str())),
                            comment: current.comment.clone(),
                        });
                        return Ok((levels, None));
                    }
                }
            }
            (JsoncKind::Array { elements }, JsoncPathSegment::Index { index }) => {
                let position = *index;
                if position < elements.len() {
                    let child = elements.get(position).ok_or_else(|| return not_found(full_path))?;
                    levels.push(SpineLevel::Array {
                        elements: elements.clone(),
                        index: Some(position),
                        comment: current.comment.clone(),
                    });
                    current = child;
                } else if position == elements.len() && is_last && allow_final_insert {
                    levels.push(SpineLevel::Array {
                        elements: elements.clone(),
                        index: None,
                        comment: current.comment.clone(),
                    });
                    return Ok((levels, None));
                } else {
                    return Err(not_found(full_path));
                }
            }
            (JsoncKind::Record { .. }, JsoncPathSegment::Index { index }) => {
                return Err(type_error(format!("jsonc edit: cannot index an object with position {index}")));
            }
            (JsoncKind::Array { .. }, JsoncPathSegment::Key { key }) => {
                return Err(type_error(format!("jsonc edit: cannot index an array with key {key:?}")));
            }
            _ => return Err(type_error(String::from("jsonc edit: cannot index a scalar value"))),
        }
    }
    return Ok((levels, Some(current)));
}

/// What:
///      Rebuild a document by writing one value into the deepest recorded level and then folding the
///           levels back outward.
/// Why:
///       A bottom-up loop replaces recursion,
///  so rebuild cost grows with the address length rather
///           than with the call stack.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rebuild(levels: SpineLevel[], value: JsoncValue): JsoncValue;
/// ```
pub(crate) fn rebuild(levels: Vec<SpineLevel>, value: JsoncValue) -> JsoncValue {
    let mut current = value;
    // What:     `into_iter().rev()` consumes the levels from the deepest outward.
    // Why:      Each level wraps the value built from the level below it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const level of [...levels].reverse()) { value = wrap(level, value); }
    // ```
    for level in levels.into_iter().rev() {
        current = match level {
            SpineLevel::Record { mut entries, index, insert_key, comment } => match index {
                Some(position) => {
                    if let Some(entry) = entries.get_mut(position) {
                        entry.value = current;
                    }
                    JsoncValue { kind: JsoncKind::Record { entries }, comment }
                }
                None => {
                    let key_text = insert_key.expect("an insert level records its key");
                    entries.push(JsoncEntry { key: JsoncKey::from_text(&key_text), value: current });
                    JsoncValue { kind: JsoncKind::Record { entries }, comment }
                }
            },
            SpineLevel::Array { mut elements, index, comment } => {
                match index {
                    Some(position) => {
                        if let Some(slot) = elements.get_mut(position) {
                            *slot = current;
                        }
                    }
                    None => elements.push(current),
                }
                JsoncValue { kind: JsoncKind::Array { elements }, comment }
            }
        };
    }
    return current;
}

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
        return Ok(JsoncValue { kind: value.kind, comment: root.comment.clone() });
    }
    let (levels, target) = descend(root, path, path, true)?;
    let replacement = match target {
        Some(addressed) => JsoncValue { kind: value.kind, comment: addressed.comment.clone() },
        None => value,
    };
    return Ok(rebuild(levels, replacement));
}

/// What:
///      Return a new document with the addressed value transformed in place.
/// Why:
///       Comment edits change one field of one node and must not insert missing structure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceAt(root, path, transform: (node) => node): JsoncValue;
/// ```
///
/// # Errors
/// Returns the missing address when a step names nothing,
///  or a shape failure when a step does not fit.
pub(crate) fn replace_at(
    root: &JsoncValue,
    path: &[JsoncPathSegment],
    full_path: &[JsoncPathSegment],
    transform: impl FnOnce(&JsoncValue) -> JsoncValue,
) -> Result<JsoncValue, JsoncEditError> {
    if path.is_empty() {
        return Ok(transform(root));
    }
    let (levels, target) = descend(root, path, full_path, false)?;
    let addressed = target.ok_or_else(|| return not_found(full_path))?;
    return Ok(rebuild(levels, transform(addressed)));
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
    let (last, parent_path) = path.split_last().ok_or_else(|| {
        return type_error(String::from("jsonc delete: cannot delete the document root"));
    })?;
    let (levels, target) = descend(root, parent_path, path, false)?;
    let parent = target.ok_or_else(|| return not_found(path))?;
    let rebuilt = match (&parent.kind, last) {
        (JsoncKind::Record { entries }, JsoncPathSegment::Key { key }) => {
            let wanted = key_units(key);
            let kept: Vec<JsoncEntry> = entries
                .iter()
                .filter(|entry| return entry.key.units != wanted)
                .cloned()
                .collect();
            JsoncValue { kind: JsoncKind::Record { entries: kept }, comment: parent.comment.clone() }
        }
        (JsoncKind::Array { elements }, JsoncPathSegment::Index { index }) => {
            let position = *index;
            if position >= elements.len() {
                return Err(not_found(path));
            }
            let kept: Vec<JsoncValue> = elements
                .iter()
                .enumerate()
                .filter(|(offset, _)| return *offset != position)
                .map(|(_, element)| return element.clone())
                .collect();
            JsoncValue { kind: JsoncKind::Array { elements: kept }, comment: parent.comment.clone() }
        }
        (JsoncKind::Record { .. }, JsoncPathSegment::Index { index }) => {
            return Err(type_error(format!("jsonc delete: cannot index an object with position {index}")));
        }
        (JsoncKind::Array { .. }, JsoncPathSegment::Key { key }) => {
            return Err(type_error(format!("jsonc delete: cannot index an array with key {key:?}")));
        }
        _ => return Err(type_error(String::from("jsonc delete: cannot index a scalar value"))),
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
    let (last, parent_path) = path.split_last().ok_or_else(|| {
        return type_error(String::from("jsonc key comment: empty path has no key"));
    })?;
    let JsoncPathSegment::Key { key } = last else {
        return Err(type_error(String::from("jsonc key comment: final segment is not a key")));
    };
    let (levels, target) = descend(root, parent_path, path, false)?;
    let parent = target.ok_or_else(|| return not_found(path))?;
    let JsoncKind::Record { entries } = &parent.kind else {
        return Err(not_found(path));
    };
    let index = find_entry_index(entries, key).ok_or_else(|| return not_found(path))?;
    let mut rebuilt: Vec<JsoncEntry> = Vec::with_capacity(entries.len());
    for (position, entry) in entries.iter().enumerate() {
        if position == index {
            rebuilt.push(JsoncEntry {
                key: JsoncKey { comment: comment.clone(), ..entry.key.clone() },
                value: entry.value.clone(),
            });
        } else {
            rebuilt.push(entry.clone());
        }
    }
    let updated = JsoncValue {
        kind: JsoncKind::Record { entries: rebuilt },
        comment: parent.comment.clone(),
    };
    return Ok(rebuild(levels, updated));
}
