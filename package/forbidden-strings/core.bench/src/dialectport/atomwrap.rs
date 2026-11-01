//! Single-atom operand wrapping and byte-cursor helpers for the dialect porter.
//!
//! The dialect's `|` and `&` take single-atom operands,
//!  so multi-atom branches
//! are wrapped in `(?:...)` at every nesting level;
//!  the escape-,
//!  class-,
//!  and
//! depth-aware cursor helpers live beside the wrapping they serve.
//!  Split out of
//! the `dialectport` bin to satisfy the max-lines budget.

/// Imports the character-class span helper shared with the porter.
use crate::port::class_end;

/// Returns the index just past one atom and its optional single quantifier.
///
/// What:
///  consumes an escape,
///  class,
///  group,
///  complement,
///  or literal,
///  then an optional `?` or
/// `{...}`.
///  Why:
///  `is_single_atom` needs the extent of the first postfix unit to decide
/// whether a whole fragment is exactly one atom.
fn unit_end(b: &[u8], start: usize) -> usize {
    let mut i = start;
    match b.get(i) {
        Some(b'\\') => i += if i + 1 < b.len() { 2 } else { 1 },
        Some(b'[') => i = class_end(b, i),
        Some(b'(') => i = matching_close(b, i) + 1,
        Some(b'~') if b.get(i + 1) == Some(&b'(') => i = matching_close(b, i + 1) + 1,
        Some(_) => i += 1,
        None => return i,
    }
    match b.get(i) {
        Some(b'?') => i += 1,
        Some(b'{') => {
            if let Some(rel) = b[i..].iter().position(|&x| return x == b'}') {
                i += rel + 1;
            }
        }
        _ => {}
    }
    return i;
}

/// Reports whether `s` is exactly one atom (with optional quantifier).
///
/// What:
///  true when the first postfix unit spans the whole string.
///  Why:
///  an operand of `|` or
/// `&` must be a single atom;
///  a fragment that already is one needs no `(?:...)` wrapper,
/// keeping the ported rule readable.
fn is_single_atom(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let b = s.as_bytes();
    return unit_end(b, 0) == b.len();
}

/// Re-emits a concatenation,
///  wrapping the alternations inside its groups and complements.
///
/// What:
///  copies escapes and classes verbatim and recurses into each `(?:...)` group and
/// `~(...)` complement with `wrap_alternation`.
///  Why:
///  branch wrapping must reach every
/// nesting level,
///  not just the top.
fn process_seq(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'\\' {
            out.push('\\');
            if i + 1 < b.len() {
                out.push(b[i + 1] as char);
            }
            i += 2;
        } else if c == b'[' {
            let end = class_end(b, i);
            out.push_str(&s[i..end]);
            i = end;
        } else if c == b'~' && b.get(i + 1) == Some(&b'(') {
            let close = matching_close(b, i + 1);
            out.push_str("~(");
            out.push_str(&wrap_alternation(&s[i + 2..close]));
            out.push(')');
            i = close + 1;
        } else if c == b'(' {
            let close = matching_close(b, i);
            let inner = group_inner_start(b, i);
            out.push_str(&s[i..inner]);
            out.push_str(&wrap_alternation(&s[inner..close]));
            out.push(')');
            i = close + 1;
        } else {
            out.push(c as char);
            i += 1;
        }
    }
    return out;
}

/// Wraps each multi-atom `|` branch of a fragment in a non-capturing group.
///
/// What:
///  splits on top-level `|`;
///  a lone branch is re-emitted,
///  otherwise each branch that is
/// not already a single atom is wrapped in `(?:...)`.
///  Why:
///  the dialect's `|` takes
/// single-atom operands,
///  and wrapping only where required keeps the output minimal.
pub(crate) fn wrap_alternation(s: &str) -> String {
    let branches = split_top_level(s, b'|');
    if branches.len() == 1 {
        return process_seq(&branches[0]);
    }
    let wrapped: Vec<String> = branches
        .iter()
        .map(|branch| {
            let seq = process_seq(branch);
            if is_single_atom(&seq) {
                return seq;
            }
            return format!("(?:{seq})");
        })
        .collect();
    return wrapped.join("|");
}

/// Wraps one `&` operand so it is a single atom.
///
/// What:
///  a complement is already an atom;
///  anything else is wrapped in `(?:...)` unless it
/// already is a single atom.
///  Why:
///  the dialect's `&` requires single-atom operands.
pub(crate) fn wrap_operand(body: &str) -> String {
    let expr = wrap_alternation(body);
    if body.trim_start().starts_with('~') {
        return expr;
    }
    if is_single_atom(&expr) {
        return expr;
    }
    return format!("(?:{expr})");
}

/// Returns the index of the `)` matching the `(` at `open`.
///
/// What:
///  tracks paren depth while skipping escapes and classes.
///  Why:
///  recursion into a group
/// or complement needs its exact extent.
fn matching_close(b: &[u8], open: usize) -> usize {
    let mut depth = 0i32;
    let mut i = open;
    let mut in_class = false;
    while i < b.len() {
        let c = b[i];
        if c == b'\\' {
            i += 2;
            continue;
        }
        if in_class {
            if c == b']' {
                in_class = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'[' => in_class = true,
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    return b.len().saturating_sub(1);
}

/// Returns the index where a group's inner content starts after its prefix.
///
/// What:
///  skips `(?:` when present,
///  else just the `(`.
///  Why:
///  only the content is rewrapped;
/// the prefix is copied as-is.
fn group_inner_start(b: &[u8], open: usize) -> usize {
    if open + 2 < b.len() && b[open + 1] == b'?' && b[open + 2] == b':' {
        return open + 3;
    }
    return open + 1;
}

/// Splits `s` on a delimiter byte that sits at paren depth zero,
///  outside classes.
///
/// What:
///  an escape-,
///  class-,
///  and depth-aware split.
///  Why:
///  `&` operands and `|` branches must
/// be separated only at the current level.
pub(crate) fn split_top_level(s: &str, delim: u8) -> Vec<String> {
    let b = s.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
    let mut in_class = false;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'\\' {
            i += 2;
            continue;
        }
        if in_class {
            if c == b']' {
                in_class = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'[' => in_class = true,
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ if c == delim && depth == 0 => {
                parts.push(s[start..i].to_string());
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(s[start..].to_string());
    return parts;
}
