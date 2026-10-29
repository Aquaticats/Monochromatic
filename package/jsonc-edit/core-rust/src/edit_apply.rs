//! What:
//!      The public structural edit surface:
//!  replacing or inserting a value at an address,
//!  and
//!           deleting a member or element.
//! Why:
//!       Each operation returns a new document and leaves the input untouched,
//!  which is the immutable
//!           edit contract the maintained TypeScript package documents.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module editApply: jsoncSet(root, path, value), jsoncDelete(root, path).
//! ```

/// What:
///      Import the iterative spine operations that perform the rebuild.
/// Why:
///       Descent and rebuild live in one place so reads,
///  edits and comment edits cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { deleteValue, setValue } from './editSpine';
/// ```
use crate::edit_spine::{delete_value, set_value};
/// What:
///      Import the edit failure enum.
/// Why:
///       An edit can hit a missing address or a wrong-shaped target,
///  and both are ordinary results.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncEditError } from './error';
/// ```
use crate::error::JsoncEditError;
/// What:
///      Import the address segment type.
/// Why:
///       A caller states each step as a key or an index rather than as a string path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncPathSegment } from './path';
/// ```
use crate::path::JsoncPathSegment;
/// What:
///      Import the document value type.
/// Why:
///       Edits take and return documents,
///  never borrowed slices of one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncValue } from './value';
/// ```
use crate::value::{JsoncKind, JsoncValue};

/// What:
///      Return a new document with one address set to a replacement value.
/// Why:
///       Setting is the common edit;
///  it replaces an existing value,
///  keeps that address's comment,
///  and
///           appends only when the missing segment is the final one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncSet(state: JsoncEditState, path: JsoncPathSegment[], value: JsoncValue): JsoncEditState;
/// ```
///
/// # Errors
/// Returns the missing address when an intermediate step does not exist,
///  or a shape failure when a step
/// does not fit its target.
pub fn jsonc_set(
    root: &JsoncValue,
    path: &[JsoncPathSegment],
    value: JsoncValue,
) -> Result<JsoncValue, JsoncEditError> {
    // What: Refuse a root replacement that is not a container.
    // Why: The document contract is a record or array root. Accepting a scalar here would return a
    //      state whose canonical emission the parser rejects, so the invariant would break inside
    //      the crate's own round trip. Fuzzing found it as an emission of `null`.
    if path.is_empty() && !matches!(value.kind, JsoncKind::Record { .. } | JsoncKind::Array { .. }) {
        return Err(JsoncEditError::Type {
            error: crate::error::JsoncTypeError {
                message: "jsonc set: the document root must stay an object or array".to_string(),
            },
        });
    }
    return set_value(root, path, value);
}

/// What:
///      Return a new document with one addressed member or element removed.
/// Why:
///       Deletion is the other structural edit,
///  and removing a duplicate key removes every member with
///           that name,
///  matching the maintained TypeScript behavior.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncDelete(state: JsoncEditState, path: JsoncPathSegment[]): JsoncEditState;
/// ```
///
/// # Errors
/// Returns a shape failure for an empty address or a mismatched step,
///  and the missing address when a step
/// names nothing.
pub fn jsonc_delete(root: &JsoncValue, path: &[JsoncPathSegment]) -> Result<JsoncValue, JsoncEditError> {
    return delete_value(root, path);
}
