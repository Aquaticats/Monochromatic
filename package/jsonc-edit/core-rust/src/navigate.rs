//! What:
//!      Address resolution inside a parsed JSONC document.
//!           A path is a slice of segments,
//!  and resolution walks one segment at a time through record
//!           members and array elements,
//!  reporting a missing address and a wrong-shaped target
//!           separately.
//! Why:
//!       Reads,
//!  edits and comment queries all need the same walk,
//!  and duplicate object keys must
//!           resolve the same way the maintained TypeScript package resolves them:
//!  the last member with
//!           that key wins.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module navigate: lookup(state, path), keys(state, path), has(state, path).
//! ```

/// What:
///      Import the edit failure enum and its two underlying causes.
/// Why:
///       Resolution distinguishes "nothing at this address" from "this target has the wrong shape".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncEditError, JsoncPathNotFoundError, JsoncTypeError } from './error';
/// ```
use crate::error::{JsoncEditError, JsoncPathNotFoundError, JsoncTypeError};
/// What:
///      Import the address segment type.
/// Why:
///       A segment states whether the caller means an object key or an array index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncPathSegment } from './path';
/// ```
use crate::path::JsoncPathSegment;
/// What:
///      Import the document model types resolution walks.
/// Why:
///       Members keep key and value comments apart,
///  so resolution must not flatten them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncEntry, JsoncKey, JsoncKind, JsoncValue } from './value';
/// ```
use crate::value::{JsoncEntry, JsoncKey, JsoncKind, JsoncValue};

/// What:
///      Encode ordinary Rust text as UTF-16 code units.
/// Why:
///       Stored keys are code units,
///  so a requested key must be compared in the same form to match
///           a key written with escapes or holding a lone surrogate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keyUnits(key: string): number[] { return [...key].map((ch) => ch.charCodeAt(0)); }
/// ```
pub(crate) fn key_units(key: &str) -> Vec<u16> {
    // What:     `collect` gathers the encoder's iterator into an owned vector.
    // Why:      The comparison needs a sequence, not a streaming iterator.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [...key].map((ch) => ch.charCodeAt(0));
    // ```
    return key.encode_utf16().collect();
}

/// What:
///      Find the last member whose decoded key equals the requested text.
/// Why:
///       Duplicate keys are unsupported input,
///  and the maintained TypeScript package reads the last
///           one,
///  so this crate resolves identically instead of picking an arbitrary match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findEntryIndex(entries: JsoncEntry[], key: string): number | undefined;
/// ```
pub(crate) fn find_entry_index(entries: &[JsoncEntry], key: &str) -> Option<usize> {
    let wanted = key_units(key);
    // What:     `enumerate` pairs each position with its member while iterating in source order.
    // Why:      The last match wins, so the walk keeps the highest matching position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let found; entries.forEach((entry, index) => { if (entry.key.value === key) found = index; });
    // ```
    let mut found: Option<usize> = None;
    for (index, entry) in entries.iter().enumerate() {
        if entry.key.units == wanted {
            found = Some(index);
        }
    }
    return found;
}

/// What:
///      Resolve one address step against one value.
/// Why:
///       Every read and edit walks the same steps,
///  and a wrong-shaped step must be reported as a
///           shape failure rather than silently treated as absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function childAt(node: JsoncValue, segment: JsoncPathSegment): JsoncValue | undefined;
/// ```
///
/// # Errors
/// Returns a shape failure when a key addresses an array or an index addresses a record.
pub(crate) fn child_at<'a>(
    value: &'a JsoncValue,
    segment: &JsoncPathSegment,
) -> Result<Option<&'a JsoncValue>, JsoncTypeError> {
    return match segment {
        JsoncPathSegment::Key { key } => match &value.kind {
            JsoncKind::Record { entries } => {
                let index = match find_entry_index(entries, key) {
                    Some(found) => found,
                    None => return Ok(None),
                };
                // What:     Indexing a slice yields the member; `expect` would panic, so the found
                //           position is read through `get`.
                // Why:      The position came from this same slice, but a bounds-checked read keeps the
                //           invariant local instead of assumed.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return entries[found]?.value;
                // ```
                let entry = entries.get(index).ok_or_else(|| {
                    return JsoncTypeError {
                        message: String::from("resolved JSONC member index is out of range"),
                    };
                })?;
                return Ok(Some(&entry.value));
            }
            JsoncKind::Array { .. } => Err(JsoncTypeError {
                message: format!("jsonc lookup: cannot index an array with key {key:?}"),
            }),
            _ => Err(JsoncTypeError {
                message: String::from("jsonc lookup: cannot index a scalar value"),
            }),
        },
        JsoncPathSegment::Index { index } => match &value.kind {
            JsoncKind::Array { elements } => Ok(elements.get(*index)),
            JsoncKind::Record { .. } => Err(JsoncTypeError {
                message: format!("jsonc lookup: cannot index an object with position {index}"),
            }),
            _ => Err(JsoncTypeError {
                message: String::from("jsonc lookup: cannot index a scalar value"),
            }),
        },
    };
}

/// What:
///      Walk a whole address and return the value it names.
/// Why:
///       An empty address names the document root,
///  and every other address names one descendant,
///  so
///           callers never handle those two cases separately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolve(root: JsoncValue, path: JsoncPathSegment[]): JsoncValue;
/// ```
///
/// # Errors
/// Returns the missing address,
///  or a shape failure when a step does not fit its target.
pub(crate) fn resolve<'a>(
    root: &'a JsoncValue,
    path: &[JsoncPathSegment],
) -> Result<&'a JsoncValue, JsoncEditError> {
    let mut current = root;
    for segment in path {
        let next = child_at(current, segment)
            .map_err(|error| return JsoncEditError::Type { error })?
            .ok_or_else(|| {
                return JsoncEditError::PathNotFound {
                    error: JsoncPathNotFoundError { path: Vec::from(path) },
                };
            })?;
        current = next;
    }
    return Ok(current);
}

/// What:
///      Read the value one address names.
/// Why:
///       This is the crate's read surface:
///  a caller asks for a member or element and receives the
///           stored value,
///  comments included.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncLookup(state: JsoncEditState, path: JsoncPathSegment[]): JsoncValue;
/// ```
///
/// # Errors
/// Returns the missing address,
///  or a shape failure when a step does not fit its target.
pub fn jsonc_lookup<'a>(
    root: &'a JsoncValue,
    path: &[JsoncPathSegment],
) -> Result<&'a JsoncValue, JsoncEditError> {
    return resolve(root, path);
}

/// What:
///      Report whether one address names a value.
/// Why:
///       Existence checks must not be expressed as a caught error,
///  which would hide shape failures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncHas(state: JsoncEditState, path: JsoncPathSegment[]): boolean;
/// ```
pub fn jsonc_has(root: &JsoncValue, path: &[JsoncPathSegment]) -> bool {
    return resolve(root, path).is_ok();
}

/// What:
///      List one record's member keys in source order.
/// Why:
///       Callers iterating a document need its authored order and its key spellings,
///  including keys
///           that hold escapes or lone surrogates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncKeys(state: JsoncEditState, path: JsoncPathSegment[]): JsoncKey[];
/// ```
///
/// # Errors
/// Returns the missing address,
///  or a shape failure when the target is not a record.
pub fn jsonc_keys<'a>(
    root: &'a JsoncValue,
    path: &[JsoncPathSegment],
) -> Result<Vec<&'a JsoncKey>, JsoncEditError> {
    let target = resolve(root, path)?;
    let JsoncKind::Record { entries } = &target.kind else {
        return Err(JsoncEditError::Type {
            error: JsoncTypeError { message: String::from("jsonc keys: target is not an object") },
        });
    };
    // What:     `map` and `collect` build an owned vector of borrowed key references.
    // Why:      The caller receives keys in document order without cloning their text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return entries.map((entry) => entry.key);
    // ```
    return Ok(entries.iter().map(|entry| return &entry.key).collect());
}
