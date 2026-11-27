//! What:
//! Reading the positions the JSONC parser reported,
//! in the order it reported them:
//! one value span per value and one key span per object member,
//! so the tree and its positions are walked together,
//! and a list that does not line up with the tree is refused rather than misread.
//! Why:
//! The plan's text is spliced at the byte offsets the parser recorded,
//! so pairing a value with another value's position would write the new entry into the middle of an old one.
//! Only `plan_insert` and the place reader beside it read this module,
//! which is why every name here is `pub(super)`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // plan-insert-spans.ts
//! class SpanWalk { value(): Span | undefined; key(): Span | undefined }
//! function skipChildren(walk: SpanWalk, value: JsoncValue, label: string): void  // throws
//! ```

/// Imports the parsed tree the spans belong to.
use monochromatic_jsonc_edit::{
    JsoncKind,
    JsoncValue,
};

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;

/// A reader of the parser's two span lists,
/// in the order the parser recorded them.
pub(super) struct SpanWalk<'a> {
    /// One span per value,
    /// the root first.
    pub(super) values: &'a [monochromatic_jsonc_edit::JsoncSpan],
    /// One span per object member key,
    /// in source order.
    pub(super) keys: &'a [monochromatic_jsonc_edit::JsoncSpan],
    /// The next value span to read.
    pub(super) value_index: usize,
    /// The next key span to read.
    pub(super) key_index: usize,
}

/// The methods of `SpanWalk`.
impl<'a> SpanWalk<'a> {
    /// Reads the next value span.
    pub(super) fn value(&mut self) -> Option<monochromatic_jsonc_edit::JsoncSpan> {
        let span: monochromatic_jsonc_edit::JsoncSpan = *self.values.get(self.value_index)?;
        self.value_index += 1;
        return Some(span);
    }

    /// Reads the next key span.
    pub(super) fn key(&mut self) -> Option<monochromatic_jsonc_edit::JsoncSpan> {
        let span: monochromatic_jsonc_edit::JsoncSpan = *self.keys.get(self.key_index)?;
        self.key_index += 1;
        return Some(span);
    }
}

/// The internal error for a span list that does not line up with the tree it came with:
/// the crate's own invariant,
/// so a person reading it knows the file was not written.
pub(super) fn misaligned(label: &str) -> ToolchainError {
    return ToolchainError::config(format!(
        "{label}: the parser's positions do not line up with the values it returned, so no entry was added and the file is as it was"
    ));
}

/// Consumes the spans of the children of `value`,
/// whose own span the caller already read.
pub(super) fn skip_children(
    walk: &mut SpanWalk,
    value: &JsoncValue,
    label: &str,
) -> Result<(), ToolchainError> {
    match &value.kind {
        JsoncKind::Record {
            entries,
        } => {
            for entry in entries {
                walk.key().ok_or_else(|| return misaligned(label))?;
                walk.value().ok_or_else(|| return misaligned(label))?;
                skip_children(walk, &entry.value, label)?;
            }
        }
        JsoncKind::Array {
            elements,
        } => {
            for element in elements {
                walk.value().ok_or_else(|| return misaligned(label))?;
                skip_children(walk, element, label)?;
            }
        }
        JsoncKind::Null
        | JsoncKind::Boolean {
            ..
        }
        | JsoncKind::Number {
            ..
        }
        | JsoncKind::Text {
            ..
        } => {}
    }
    return Ok(());
}

/// Consumes the spans of `value` and of everything below it.
pub(super) fn skip_value(
    walk: &mut SpanWalk,
    value: &JsoncValue,
    label: &str,
) -> Result<(), ToolchainError> {
    walk.value().ok_or_else(|| return misaligned(label))?;
    return skip_children(walk, value, label);
}
