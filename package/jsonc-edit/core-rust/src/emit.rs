//! What:
//!  Deterministic JSONC emission from comment-bearing values.
//! Why:
//!  The editor writes a canonical tree,
//!  not byte-identical source splices.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function emitJsoncValue(node: JsoncValue): string;
//! ```

/// What:
///  Import safe comment placement decisions.
/// Why:
///  Multi-line comments cannot appear after a value's comma.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { leading, singleLine, trailing } from './comment';
/// ```
use crate::comment_merge::{leading, single_line, trailing};
/// What:
///  Borrow parsed node and tagged value types.
/// Why:
///  Emission reads source data without mutating an edit state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncValue, JsoncKind } from './value';
/// ```
use crate::value::{JsoncKind, JsoncValue};

/// What:
///  Serialize a whole JSONC document with its root-attached comment.
/// Why:
///  Callers get a deterministic,
///  parseable document in one operation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function emitJsoncValue(root: JsoncValue): string;
/// ```
pub fn emit_jsonc_value(root: &JsoncValue) -> String {
    // What: `String::new` owns the output buffer; `&JsoncValue` borrows input read-only.
    // Why: String construction must not modify or invalidate the old parsed state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let out = '';
    // ```
    let mut out = String::new();
    if let Some(comment) = &root.comment {
        out.push_str(&leading(comment, 0));
    }
    emit_bare(root, 0, &mut out);
    return out;
}

/// What:
///  Write only a value's syntax;
///  its owner positions the attached comment.
/// Why:
///  A member key and its value have different legal comment placements.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function emitBare(value: JsoncValue, depth: number, out: string[]): void;
/// ```
fn emit_bare(value: &JsoncValue, depth: usize, out: &mut String) {
    if let JsoncKind::Text { raw, .. } = &value.kind {
        out.push_str(raw);
        return;
    }
    if let JsoncKind::Number { raw, .. } = &value.kind {
        out.push_str(raw);
        return;
    }
    if let JsoncKind::Boolean { value: boolean } = &value.kind {
        out.push_str(if *boolean { "true" } else { "false" });
        return;
    }
    if let JsoncKind::Null = &value.kind {
        out.push_str("null");
        return;
    }
    if let JsoncKind::Array { elements } = &value.kind {
        if elements.is_empty() {
            out.push_str("[]");
            return;
        }
        out.push_str("[\n");
        let pad = "  ".repeat(depth + 1);
        for element in elements {
            // What: A let-chain checks both comment presence and placement in one branch.
            // Why: Only multi-line comments can safely lead an array element.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (comment !== undefined && !singleLine(comment)) output += leading(comment);
            // ```
            if let Some(comment) = &element.comment && !single_line(comment) {
                out.push_str(&leading(comment, depth + 1));
            }
            out.push_str(&pad);
            emit_bare(element, depth + 1, out);
            out.push(',');
            // A single-line comment may follow the element's comma.
            if let Some(comment) = &element.comment && single_line(comment) {
                out.push(' ');
                out.push_str(&trailing(comment));
            }
            out.push('\n');
        }
        out.push_str(&"  ".repeat(depth));
        out.push(']');
        return;
    }
    if let JsoncKind::Record { entries } = &value.kind {
        if entries.is_empty() {
            out.push_str("{}");
            return;
        }
        out.push_str("{\n");
        let pad = "  ".repeat(depth + 1);
        for entry in entries {
            if let Some(comment) = &entry.key.comment {
                out.push_str(&leading(comment, depth + 1));
            }
            out.push_str(&pad);
            out.push_str(&entry.key.raw);
            out.push(':');
            // A multi-line value comment must follow the colon so it cannot attach to the key.
            if let Some(comment) = &entry.value.comment && !single_line(comment) {
                out.push('\n');
                out.push_str(&leading(comment, depth + 1));
                out.push_str(&pad);
            } else {
                out.push(' ');
            }
            emit_bare(&entry.value, depth + 1, out);
            out.push(',');
            // Single-line value comments sit after the member's comma.
            if let Some(comment) = &entry.value.comment && single_line(comment) {
                out.push(' ');
                out.push_str(&trailing(comment));
            }
            out.push('\n');
        }
        out.push_str(&"  ".repeat(depth));
        out.push('}');
        return;
    }
    // Every declared JSONC value kind has an emission branch.
    unreachable!("unhandled JSONC value kind");
}
