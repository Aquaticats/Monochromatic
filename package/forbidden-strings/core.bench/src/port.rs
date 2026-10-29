//! Mechanically ports a resharp/PCRE rule into the forbidden-regex dialect.
//!
//! What:
//!  strips the boundary/context around a secret,
//!  wraps `|` branches and `&`
//! operands as single atoms,
//!  and hands `regex` the complement-stripped positive;
//! normalization (quantifier bounding,
//!  case flags,
//!  POSIX classes) lives in the
//! sibling module.
//!  Why:
//!  the real ruleset is written in a richer dialect than this
//! engine accepts,
//!  so each rule is adapted;
//!  whatever fails to compile is dropped by
//! the caller's compile-filter,
//!  so soundness is preserved.

/// Imports the dialect normalizer.
use crate::normalize::normalize;

/// Ports one rule body into `(ours, bare)`.
///
/// What:
///  extracts just the secret from the positive operand (the capturing group's
/// content,
///  or the boundary-stripped whole),
///  keeps any `&`/`~` operands,
///  and yields
/// `ours` (with operands wrapped to single atoms) and `bare` (the positive alone,
/// for `regex`).
///  Why:
///  a secret is just as leaky surrounded,
///  so the ruleset carries
/// no boundary or context wrappers,
///  only the secret shape and its set algebra.
pub fn port(inner: &str) -> (String, String) {
    let operands = split_top_level(inner, b'&');
    let secret = strip_context(&normalize(&operands[0]));
    let bare = secret.clone();
    if operands.len() == 1 {
        return (wrap_branches(&secret), bare);
    }
    let mut ours_ops = vec![format!("(?:{})", wrap_branches(&secret))];
    for operand in &operands[1..] {
        let normalized = normalize(operand);
        ours_ops.push(wrap_operand(&normalized));
    }
    return (ours_ops.join("&"), bare)
}

/// Strips the leading and trailing context that surrounds a secret.
///
/// What:
///  repeatedly removes a leading boundary (`\b`/`^`) or context class-repeat
/// (`[\w.-]{0,50}`),
///  and a trailing boundary (`\b`/`$`) or context group
/// (`(?:…|$)`).
///  Why:
///  a secret is just as leaky surrounded,
///  so betterleaks' boundary
/// and delimiter context is dropped while the keyword,
///  gap,
///  and value are kept.
fn strip_context(s: &str) -> String {
    let mut cur = s.to_string();
    loop {
        let before = cur.len();
        let trimmed = strip_trailing(&strip_leading(&cur));
        if trimmed.len() == before {
            return trimmed;
        }
        cur = trimmed;
    }
}

/// Strips one leading context element.
///
/// What:
///  a leading `\b`,
///  a nullable class-repeat `[...]{0,N}`,
///  or such a class-repeat
/// nested right after a leading `(?:` group open;
///  a leading `^` is kept.
///  Why:
///  `\b` and
/// the class-repeats are pure context (the engine flattens the kept `(?:` groups,
///  so
/// the keyword inside becomes the leading literal),
///  but `^` is a real line-start
/// anchor that lets the engine check the rule only at line starts.
fn strip_leading(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("\\b") {
        return rest.to_string();
    }
    if let Some(rest) = strip_leading_class_repeat(s) {
        return rest.to_string();
    }
    if let Some(rest) = s.strip_prefix("(?:")
        && let Some(after) = strip_leading_class_repeat(rest)
    {
        return format!("(?:{after}");
    }
    return s.to_string()
}

/// Strips a leading nullable class-repeat such as `[\w.-]{0,50}`.
///
/// What:
///  a class at the start immediately followed by `{0,N}`.
///  Why:
///  it is pure
/// preceding context under unanchored search,
///  never part of the secret.
fn strip_leading_class_repeat(s: &str) -> Option<&str> {
    if !s.starts_with('[') {
        return None;
    }
    let after_class = class_span_end(s.as_bytes())?;
    let rest = &s[after_class..];
    if rest.starts_with("{0,") {
        let brace = rest.find('}')?;
        return Some(&rest[brace + 1..]);
    }
    return None
}

/// Returns the index just past a class starting at index 0.
///
/// What:
///  scans to the closing `]`,
///  honoring escapes and a leading `]`.
///  Why:
///  a
/// leading class-repeat's bound starts right after its class.
fn class_span_end(b: &[u8]) -> Option<usize> {
    let mut i = 1;
    if b.get(i) == Some(&b'^') {
        i += 1;
    }
    if b.get(i) == Some(&b']') {
        i += 1;
    }
    while i < b.len() && b[i] != b']' {
        i += if b[i] == b'\\' { 2 } else { 1 };
    }
    if i < b.len() { return Some(i + 1) } else { return None }
}

/// Strips one trailing context element.
///
/// What:
///  a trailing `\b`,
///  `$`,
///  or context group `(?:…|$)`.
///  Why:
///  these are the
/// end-context idioms betterleaks appends to a secret.
fn strip_trailing(s: &str) -> String {
    if let Some(rest) = s.strip_suffix("\\b") {
        return rest.to_string();
    }
    if s.ends_with('$') && !s.ends_with("\\$") {
        return s[..s.len() - 1].to_string();
    }
    match trailing_context_group(s) {
        Some(open) => return s[..open].to_string(),
        None => return s.to_string(),
    }
}

/// Returns the start index of a trailing context group `(?:…|$)`.
///
/// What:
///  the top-level group that ends the string and contains an end-anchor
/// (`$` or `\z`).
///  Why:
///  betterleaks closes a rule with such a delimiter group,
///  which
/// is context,
///  not secret.
fn trailing_context_group(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    if b.last() != Some(&b')') {
        return None;
    }
    let mut i = 0;
    let mut in_class = false;
    let mut stack: Vec<usize> = Vec::new();
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
        } else if c == b'[' {
            in_class = true;
        } else if c == b'(' {
            stack.push(i);
        } else if c == b')' {
            let open = stack.pop()?;
            if stack.is_empty() && i == b.len() - 1 {
                let content = &s[open + 1..i];
                if content.contains('$') || content.contains("\\z") {
                    return Some(open);
                }
            }
        }
        i += 1;
    }
    return None
}

/// Wraps one `&` operand so it is a single atom.
///
/// What:
///  a complement is already an atom;
///  anything else is wrapped in `(?:...)`
/// after its branches are wrapped.
///  Why:
///  the grammar requires single-atom operands.
fn wrap_operand(operand: &str) -> String {
    if operand.trim_start().starts_with('~') {
        return wrap_branches(operand)
    } else {
        return format!("(?:{})", wrap_branches(operand))
    }
}

/// Wraps each top-level `|` branch of a fragment in a non-capturing group.
///
/// What:
///  a single branch is returned processed;
///  multiple branches are each wrapped
/// so the alternation's operands are single atoms.
///  Why:
///  the grammar requires it.
fn wrap_branches(s: &str) -> String {
    let branches = split_top_level(s, b'|');
    if branches.len() == 1 {
        return process_seq(&branches[0]);
    }
    return branches
        .iter()
        .map(|branch| format!("(?:{})", process_seq(branch)))
        .collect::<Vec<_>>()
        .join("|")
}

/// Re-emits a branch,
///  recursing into groups and complements.
///
/// What:
///  copies escapes and classes verbatim and rewraps the contents of each
/// `(?:...)` group and `~(...)` complement.
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
            out.push(c as char);
            if i + 1 < b.len() {
                out.push(b[i + 1] as char);
            }
            i += 2;
        } else if c == b'[' {
            let end = class_end(b, i);
            out.push_str(&s[i..end]);
            i = end;
        } else if c == b'~' && i + 1 < b.len() && b[i + 1] == b'(' {
            let close = matching_close(b, i + 1);
            out.push_str("~(");
            out.push_str(&wrap_branches(&s[i + 2..close]));
            out.push(')');
            i = close + 1;
        } else if c == b'(' {
            let close = matching_close(b, i);
            let inner_start = group_inner_start(b, i);
            out.push_str(&s[i..inner_start]);
            out.push_str(&wrap_branches(&s[inner_start..close]));
            out.push(')');
            i = close + 1;
        } else {
            out.push(c as char);
            i += 1;
        }
    }
    return out
}

/// Returns the index just past a class that starts at `open`.
///
/// What:
///  scans to the closing `]`,
///  honoring escapes.
///  Why:
///  class contents must be
/// copied verbatim so their `(`/`|`/`*` are not misread as operators;
///  shared with
/// the normalizer.
pub(crate) fn class_end(b: &[u8], open: usize) -> usize {
    let mut i = open + 1;
    while i < b.len() && b[i] != b']' {
        i += if b[i] == b'\\' { 2 } else { 1 };
    }
    return (i + 1).min(b.len())
}

/// Returns the index of the `)` matching the `(` at `open`.
///
/// What:
///  tracks paren depth while skipping escapes and classes.
///  Why:
///  recursion into
/// a group needs its exact extent;
///  shared with the case-expansion module.
pub(crate) fn matching_close(b: &[u8], open: usize) -> usize {
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
    return b.len().saturating_sub(1)
}

/// Returns the index where a group's inner content starts after its prefix.
///
/// What:
///  skips `(?:` when present,
///  else just the `(`.
///  Why:
///  only the content is
/// rewrapped;
///  the prefix is copied as-is.
fn group_inner_start(b: &[u8], open: usize) -> usize {
    if open + 2 < b.len() && b[open + 1] == b'?' && b[open + 2] == b':' {
        return open + 3
    } else {
        return open + 1
    }
}

/// Splits `s` on a delimiter byte that sits at paren depth zero,
///  outside classes.
///
/// What:
///  an escape-,
///  class-,
///  and depth-aware split.
///  Why:
///  `&` operands and `|`
/// branches must be separated only at the current level.
fn split_top_level(s: &str, delim: u8) -> Vec<String> {
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
    return parts
}
