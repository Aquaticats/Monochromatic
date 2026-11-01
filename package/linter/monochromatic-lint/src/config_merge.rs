//! What:
//!  Ordered merging of the JSONC values used by rule settings.
//! Why:
//!  The linter needs record merging and array concatenation,
//!  not a general JavaScript merge library.
//! Inputs are borrowed and never changed;
//!  configuration validation rejects duplicate keys first.
//! Merged records retain each key's first spelling and key comment;
//!  the final value owns its value comment.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // merge all matching rule-settings objects together, not pairwise.
//! ```

/// What:
///  Import an ordered lookup map from Rust's standard library.
/// Why:
///  Grouping equal decoded keys avoids repeatedly searching all earlier object members.
/// A map supplies lookup;
///  a separate vector retains the author's first-seen key order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Map<string, JsoncValue[]> with a separately retained key order.
/// ```
use std::collections::BTreeMap;

/// What:
///  Import the parser's value,
///  key and object-member types.
/// Why:
///  Merging retains exact numbers and string code units without a second JSON representation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncValue, JsoncKey, JsoncEntry } from 'jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{JsoncEntry, JsoncKey, JsoncValue};

/// What:
///  Merge every input together in its original order.
/// Why:
///  A type mismatch anywhere in the input group makes the final value win outright.
/// Pairwise folding would incorrectly merge later matching values after an earlier mismatch.
/// An empty input group produces an empty settings object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeValues(values: readonly JsoncValue[]): JsoncValue;
/// ```
pub fn merge_values(values: &[&JsoncValue]) -> JsoncValue {
    // What: Inspect a borrowed slice without taking ownership of any parsed input.
    // Why: The same configuration blocks remain available for every matched file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (values.length === 0) return {};
    // ```
    if values.is_empty() {
        // What: Construct an empty owned vector rather than a borrowed slice or fixed-size array.
        // Why: The result owns its member storage independently of the input lifetime.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return jsoncRecord([]);
        // ```
        return JsoncValue::record(Vec::new());
    }
    // The non-empty check makes both indexes valid; these copy references, not document trees.
    let first = values[0];
    let last = values[values.len() - 1];
    // What: Borrow each payload and compare its enum variant, ignoring that variant's data.
    // Why: Strings and numbers are different kinds even when their displayed text happens to match.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const firstKind = first.kind;
    // ```
    let first_kind = std::mem::discriminant(&first.kind);
    for value in values {
        // Borrow the payload solely to read its kind.
        if std::mem::discriminant(&value.kind) != first_kind {
            // What: Clone the final value into independently owned storage.
            // Why: Returning a borrow would tie the merged result to a caller's input documents.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return structuredClone(last);
            // ```
            return last.clone();
        }
    }
    // What: Select the merging operation using the value model's optional container views.
    // Why: Scalar values need no recursive operation and simply retain the last input.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let result = isRecord(first) ? mergeRecords(values)
    //   : isArray(first) ? mergeArrays(values) : structuredClone(last);
    // ```
    let mut result = if first.entries().is_some() {
        merge_records(values)
    } else if first.elements().is_some() {
        merge_arrays(values)
    } else {
        last.clone()
    };
    // Clone the final container comment independently, just as scalar replacement does.
    result.comment = last.comment.clone();
    return result;
}

/// What:
///  Merge groups already proven to contain only records.
/// Why:
///  Each key must see all its values at once to preserve the all-input mismatch rule.
/// Parsed documents have bounded structural depth;
///  no recursion walks a flat member list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeRecords(values: readonly JsoncValue[]): JsoncValue;
/// ```
fn merge_records(values: &[&JsoncValue]) -> JsoncValue {
    // What: Own decoded-key vectors and borrow their matching values in a sorted lookup map.
    // Why: UTF-16 code units distinguish every JSON key, including escaped unpaired surrogates.
    // Vec<u16> owns variable-length units; a slice would borrow, and a fixed array cannot fit arbitrary keys.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const grouped = new Map<string, JsoncValue[]>();
    // ```
    let mut grouped: BTreeMap<Vec<u16>, Vec<&JsoncValue>> = BTreeMap::new();
    // What: Own the keys in first-seen order in a growable vector.
    // Why: Iterating the sorted lookup map alone would reorder authored rule settings.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const keys: JsoncKey[] = [];
    // ```
    let mut keys: Vec<JsoncKey> = Vec::new();
    for value in values {
        // What: Extract the optional record view after the caller classified every input.
        // Why: A missing view here is a violated internal invariant, not a user-input fallback.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const entries = assertRecord(value).entries;
        // ```
        let entries = value
            .entries()
            .expect("merge_records receives classified record values");
        for entry in entries {
            // What: Borrow the key's decoded units for lookup without copying them.
            // Why: Escape spelling must not split two equal logical keys into different groups.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (!grouped.has(entry.key.value)) keys.push(structuredClone(entry.key));
            // ```
            if !grouped.contains_key(&entry.key.units) {
                // Clone the first key's spelling and comment into result-owned storage.
                keys.push(entry.key.clone());
            }
            // What: Find or create the group, then lend it this input value.
            // Why: Only key storage is copied here; nested input trees are not cloned per ancestor.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const group = grouped.get(key) ?? [];
            // grouped.set(key, group);
            // group.push(entry.value);
            // ```
            grouped
                .entry(entry.key.units.clone())
                .or_default()
                .push(&entry.value);
        }
    }
    // Own the resulting members; capacity follows the measured number of distinct keys.
    let mut merged = Vec::with_capacity(keys.len());
    for key in keys {
        // Borrow the decoded key for lookup; the collecting loop created every ordered key's group.
        let members = grouped
            .get(&key.units)
            .expect("each ordered key has a collected group");
        // Recurse only into this key's structural child values, never the flat member sequence.
        let value = merge_values(members.as_slice());
        // Move the owned key and merged value into the result member.
        merged.push(JsoncEntry { key, value });
    }
    return JsoncValue::record(merged);
}

/// What:
///  Concatenate groups already proven to contain only arrays.
/// Why:
///  Array elements append in input order and are not merged with one another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeArrays(values: readonly JsoncValue[]): JsoncValue;
/// ```
fn merge_arrays(values: &[&JsoncValue]) -> JsoncValue {
    // What: Own a growable element vector rather than borrowing any input array.
    // Why: The merged configuration remains independent of every parsed block.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const merged: JsoncValue[] = [];
    // ```
    let mut merged = Vec::new();
    for value in values {
        // What: Extract the array view established by the caller's classification.
        // Why: A wrong kind here is an internal invariant failure rather than silent data loss.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const elements = assertArray(value).elements;
        // ```
        let elements = value
            .elements()
            .expect("merge_arrays receives classified array values");
        // Clone each appended element into result-owned storage without flattening nested arrays.
        merged.extend_from_slice(elements);
    }
    return JsoncValue::array(merged);
}

/// What:
///  Compile regression tests only for the test build.
/// Why:
///  Verification inputs and assertions do not belong in the installed executable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Separate test module loaded by the test runner.
/// ```
#[cfg(test)]
#[path = "config_merge_tests.rs"]
mod tests;
