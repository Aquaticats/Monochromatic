//! What:
//!      The comment-as-data surface:
//!  reading and replacing the comment attached to a value or to an
//!           object key.
//! Why:
//!       Comments are queryable and editable in this crate rather than merely preserved,
//!  and a key's
//!           comment is separate from the comment on the value it introduces.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module editComment: jsoncComment(root, path), jsoncSetComment(root, path, comment), and the key pair.
//! ```

/// What:
///      Import the iterative spine operations used by comment edits.
/// Why:
///       A comment edit rebuilds the same spine a value edit does,
///  so both share one walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { replaceAt, setKeyComment } from './editSpine';
/// ```
use crate::edit_spine::{replace_at, set_key_comment};
/// What:
///      Import the edit failure enum and its underlying causes.
/// Why:
///       A key query can fail because the address is missing or because its shape cannot name a key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncEditError, JsoncPathNotFoundError, JsoncTypeError } from './error';
/// ```
use crate::error::{JsoncEditError, JsoncPathNotFoundError, JsoncTypeError};
/// What:
///      Import the member lookup helper and the address walk.
/// Why:
///       A key comment lives on the parent record's member,
///  not on the addressed value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { findEntryIndex, resolve } from './navigate';
/// ```
use crate::navigate::{find_entry_index, resolve};
/// What:
///      Import the address segment type.
/// Why:
///       Only a key segment can name a member whose key comment is queried.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncPathSegment } from './path';
/// ```
use crate::path::JsoncPathSegment;
/// What:
///      Import the comment and document types this surface reads and rebuilds.
/// Why:
///       A replaced comment keeps the node's payload and its other comment slot untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncComment, JsoncKind, JsoncValue } from './value';
/// ```
use crate::value::{JsoncComment, JsoncKind, JsoncValue};

/// What:
///      Split one address into its parent address and its final key name.
/// Why:
///       Key-comment operations address a member of the parent record,
///  so both parts are needed and an
///           empty or index-final address must be rejected before any walk happens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitKeyPath(path: JsoncPathSegment[]): [JsoncPathSegment[], string];
/// ```
///
/// # Errors
/// Returns a shape failure when the address is empty or ends in an array index.
fn split_key_path(path: &[JsoncPathSegment]) -> Result<(&[JsoncPathSegment], &str), JsoncEditError> {
    let (last, parent) = path.split_last().ok_or_else(|| {
        return JsoncEditError::Type {
            error: JsoncTypeError { message: String::from("jsonc key comment: empty path has no key") },
        };
    })?;
    let JsoncPathSegment::Key { key } = last else {
        return Err(JsoncEditError::Type {
            error: JsoncTypeError {
                message: String::from("jsonc key comment: final segment is not a key"),
            },
        });
    };
    return Ok((parent, key.as_str()));
}

/// What:
///      Read the comment attached to one addressed value.
/// Why:
///       Querying is half of comment-as-data,
///  and absence is a distinct answer from an empty comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncComment(root: JsoncValue, path: JsoncPathSegment[]): JsoncComment | undefined;
/// ```
///
/// # Errors
/// Returns the missing address,
///  or a shape failure when a step does not fit its target.
pub fn jsonc_comment<'a>(
    root: &'a JsoncValue,
    path: &[JsoncPathSegment],
) -> Result<Option<&'a JsoncComment>, JsoncEditError> {
    let target = resolve(root, path)?;
    return Ok(target.comment.as_ref());
}

/// What:
///      Return a new document whose addressed value carries the given comment.
/// Why:
///       Comment edits are immutable like every other edit,
///  and passing absence clears the comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncSetComment(root, path, comment): JsoncValue;
/// ```
///
/// # Errors
/// Returns the missing address,
///  or a shape failure when a step does not fit its target.
pub fn jsonc_set_comment(
    root: &JsoncValue,
    path: &[JsoncPathSegment],
    comment: Option<JsoncComment>,
) -> Result<JsoncValue, JsoncEditError> {
    return replace_at(root, path, path, |target| {
        return JsoncValue { kind: target.kind.clone(), comment: comment.clone() };
    });
}

/// What:
///      Read the comment attached to one addressed object key.
/// Why:
///       A key's comment documents the member name,
///  and the maintained package keeps it separate from
///           the value's comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncKeyComment(root: JsoncValue, path: JsoncPathSegment[]): JsoncComment | undefined;
/// ```
///
/// # Errors
/// Returns a shape failure for an empty or index-final address,
///  and the missing address when the parent is
/// not a record or the member does not exist.
pub fn jsonc_key_comment<'a>(
    root: &'a JsoncValue,
    path: &[JsoncPathSegment],
) -> Result<Option<&'a JsoncComment>, JsoncEditError> {
    let (parent_path, key) = split_key_path(path)?;
    let parent = resolve(root, parent_path)?;
    let JsoncKind::Record { entries } = &parent.kind else {
        return Err(JsoncEditError::PathNotFound {
            error: JsoncPathNotFoundError { path: Vec::from(path) },
        });
    };
    let index = find_entry_index(entries, key).ok_or_else(|| {
        return JsoncEditError::PathNotFound {
            error: JsoncPathNotFoundError { path: Vec::from(path) },
        };
    })?;
    let entry = entries.get(index).ok_or_else(|| {
        return JsoncEditError::PathNotFound {
            error: JsoncPathNotFoundError { path: Vec::from(path) },
        };
    })?;
    return Ok(entry.key.comment.as_ref());
}

/// What:
///      Return a new document whose addressed key carries the given comment.
/// Why:
///       Editing a key comment must not disturb the value it introduces or the value's own comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncSetKeyComment(root, path, comment): JsoncValue;
/// ```
///
/// # Errors
/// Returns a shape failure for an empty or index-final address,
///  and the missing address when the parent is
/// not a record or the member does not exist.
pub fn jsonc_set_key_comment(
    root: &JsoncValue,
    path: &[JsoncPathSegment],
    comment: Option<JsoncComment>,
) -> Result<JsoncValue, JsoncEditError> {
    return set_key_comment(root, path, comment);
}
