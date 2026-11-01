//! What:
//!      Fuzzer-driven address selection over a parsed document.
//! Why:
//!       Edit targets must address members that actually exist;
//!           generating paths blind would spend the whole budget on not-found errors.

/// What:
///      Import the unstructured-input API.
/// Why:
///       Each descent decision and child index comes from the fuzzer's byte budget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Unstructured } from 'arbitrary';
/// ```
use arbitrary::{Result as ArbitraryResult, Unstructured};

/// What:
///      Import the address model and the text conversion used to name keys.
/// Why:
///       A path segment names a member either by key text or by element position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncPathSegment, JsoncValue, unitsToString } from 'monochromatic-jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{units_to_string, JsoncKind, JsoncPathSegment, JsoncValue};

/// What:
///      Cap on generated address length.
/// Why:
///       Generated documents nest at most a few containers,
///           so a larger cap would only produce repeated not-found paths.
const MAX_SEGMENTS: usize = 6;

/// What:
///      Choose one address that resolves inside the given document.
/// Why:
///       Edits are only meaningful where a member exists,
///           and the fuzzer should still decide how deep to go and which child to take.
///           The walk stops at a key whose text cannot be represented as Rust `String`,
///           which happens for an escaped unpaired surrogate:
///           such a member is unreachable through the string-keyed address API.
///
/// # Arguments
///
/// * `root` - Parsed document to address into.
/// * `u` - Fuzzer input supplying every choice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function randomPath(root: JsoncValue, u: Unstructured): JsoncPathSegment[];
/// ```
pub fn random_path(root: &JsoncValue, u: &mut Unstructured<'_>) -> ArbitraryResult<Vec<JsoncPathSegment>> {
    let mut path: Vec<JsoncPathSegment> = Vec::new();
    let mut current = root;
    while path.len() < MAX_SEGMENTS {
        if let JsoncKind::Record { entries } = &current.kind {
            if entries.is_empty() || !u.arbitrary::<bool>()? {
                break;
            }
            let index = u.int_in_range(0..=(entries.len() - 1))?;
            let Ok(key) = units_to_string(&entries[index].key.units) else {
                break;
            };
            current = &entries[index].value;
            path.push(JsoncPathSegment::Key { key });
            continue;
        }
        if let JsoncKind::Array { elements } = &current.kind {
            if elements.is_empty() || !u.arbitrary::<bool>()? {
                break;
            }
            let index = u.int_in_range(0..=(elements.len() - 1))?;
            current = &elements[index];
            path.push(JsoncPathSegment::Index { index });
            continue;
        }
        break;
    }
    return Ok(path);
}
