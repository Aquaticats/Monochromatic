//! What:
//!  JsoncComment attachment and safe canonical emission.
//! Why:
//!  Comments are data on keys and values rather than discarded parser trivia.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function attach(existing: JsoncComment | undefined, additions: JsoncComment[]): JsoncComment | undefined;
//! ```

/// What:
///  Import the comment value and its discriminated source kind.
/// Why:
///  This module combines and emits only those parser-owned values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncComment, JsoncCommentKind } from './value';
/// ```
use crate::value::{JsoncComment, JsoncCommentKind};

/// What:
///  Merge one existing comment with incoming comments in source order.
/// Why:
///  The model admits one queryable comment per key or value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attach(existing: JsoncComment | undefined, additions: JsoncComment[]): JsoncComment | undefined;
/// ```
pub fn attach(existing: Option<JsoncComment>, additions: Vec<JsoncComment>) -> Option<JsoncComment> {
    // What: `Option` is a present/absent wrapper; a mutable local receives each merge.
    // Why: We keep input order without requiring a recursive fold.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let current: JsoncComment | undefined = existing;
    // ```
    let mut current = existing;
    for next in additions {
        // What: `if let Some` extracts a present comment; `None` stays absent.
        // Why: The first attached comment retains its original source kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // current = current ? merge(current, next) : next;
        // ```
        current = if let Some(previous) = current {
            let kind = if previous.kind == next.kind { previous.kind } else { JsoncCommentKind::Mixed };
            // `Some` wraps a newly merged value so following iterations can attach again.
            Some(JsoncComment { kind, text: format!("{}\n{}", previous.text, next.text) })
        } else {
            Some(next)
        };
    }
    // What: An explicit return hands the caller the optional attachment.
    // Why: Keep ownership and control flow clear at the module seam.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return current;
    // ```
    return current;
}

/// What:
///  Check whether a comment body fits after a value on one line.
/// Why:
///  A multi-line comment must be moved before the value during emission.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function singleLine(comment: JsoncComment): boolean { return !comment.text.includes('\n'); }
/// ```
pub fn single_line(comment: &JsoncComment) -> bool {
    // What: Test for every line terminator, not just LF.
    // Why: A `//` comment ends at CR, LF or CRLF, so a body carrying a bare CR cannot be emitted
    //      in trailing form: the comment would end early and the rest of the body would be parsed
    //      as code. Fuzzing found exactly that as an emission the parser then rejected.
    return !comment.text.contains('\n') && !comment.text.contains('\r');
}

/// What:
///  Emit a comment before a node with two-space indentation.
/// Why:
///  A line comment is safe even when a block body contains `*/`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function leading(comment: JsoncComment, depth: number): string;
/// ```
pub fn leading(comment: &JsoncComment, depth: usize) -> String {
    // What: `"  ".repeat` allocates a prefix for the current nesting level.
    // Why: Canonical output follows the existing two-space house style.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pad = '  '.repeat(depth);
    // ```
    let pad = "  ".repeat(depth);
    // What: Prefer the block form for a block comment, or for any body carrying a bare CR.
    // Why: A block comment may contain CR and LF verbatim, while a `//` line ends at either. A
    //      body with only LF keeps the established one `//` line per body line layout.
    let block_safe = !comment.text.contains("*/");
    if block_safe && (comment.kind == JsoncCommentKind::Block || comment.text.contains('\r')) {
        return format!("{pad}/*{}*/\n", comment.text);
    }
    // What: An owned `String` accumulates lines, unlike borrowed `&str`.
    // Why: Split preserving a final empty body line and prefix every line with `//`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let output = '';
    // ```
    let mut output = String::new();
    // What: Split body lines on CRLF, LF or CR.
    // Why: This fallback is reached only when the body contains the block close delimiter, so it
    //      cannot be emitted as a block. Splitting on CR too is lossy for that character but never
    //      corrupts the document, which an emitted `//` line containing CR would.
    // What: Normalize CRLF to LF first, then split on either remaining terminator.
    // Why: `str::split` accepts an array of chars as a pattern but not an array of strings, and
    //      splitting a CRLF pair as two terminators would emit an empty comment line between them.
    let normalized = comment.text.replace("\r\n", "\n");
    for line in normalized.split(['\n', '\r']) {
        output.push_str(&pad);
        output.push_str("//");
        output.push_str(line);
        output.push('\n');
    }
    return output;
}

/// What:
///  Emit a single-line attached comment after a value.
/// Why:
///  Using `//` avoids the block-comment close delimiter in arbitrary edited text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trailing(comment: JsoncComment): string { return `//${comment.text}`; }
/// ```
pub fn trailing(comment: &JsoncComment) -> String {
    return format!("//{}", comment.text);
}
