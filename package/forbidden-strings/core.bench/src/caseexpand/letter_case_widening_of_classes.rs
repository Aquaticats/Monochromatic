//! Widening of a `[...]` class under an inline `(?i)` scope to cover both cases.
//!
//! What:
//!  preserves a leading `^` and every non-letter member of the class,
//!  and inserts the
//! opposite-case range or letter after each letter one the class does not already
//! contain.
//!  Why:
//!  a class under `(?i)` matched both cases,
//!  so widening is exact,
//!  while
//! re-adding a case the class already spans would only grow the automaton.
//!  The walker that decides when a class is under the flag lives in the parent
//! `caseexpand.rs` module,
//!  which also widens a single quantified letter to a two-case
//! class.

/// Widens each letter range or single of a `[...]` class to cover both cases.
///
/// What:
///  preserves a leading `^` and every non-letter member,
///  and inserts the opposite-case
/// range or letter after each letter one it does not already contain.
///  Why:
///  a class under `(?i)`
/// matched both cases,
///  so widening is exact.
pub(super) fn widen_class(class: &str) -> String {
    if class.len() < 2 || !class.ends_with(']') {
        return class.to_string();
    }
    let inner = &class[1..class.len() - 1];
    let (neg, body) = match inner.strip_prefix('^') {
        Some(rest) => ("^", rest),
        None => ("", inner),
    };
    let bb = body.as_bytes();
    let mut out = String::from("[");
    out.push_str(neg);
    let mut i = 0;
    while i < bb.len() {
        let (atom, next) = read_class_atom(body, i);
        if next < bb.len() && bb[next] == b'-' && next + 1 < bb.len() && bb[next + 1] != b']' {
            let (atom2, after) = read_class_atom(body, next + 1);
            out.push_str(&atom);
            out.push('-');
            out.push_str(&atom2);
            if let Some(widened) = widen_letter_range(&atom, &atom2, body) {
                out.push_str(&widened);
            }
            i = after;
        } else {
            out.push_str(&atom);
            if let Some(widened) = widen_letter_single(&atom, body) {
                out.push_str(&widened);
            }
            i = next;
        }
    }
    out.push(']');
    return out;
}

/// Reads one class member (an escape pair or a single byte),
///  returning it and the next index.
///
/// What:
///  two bytes for `\X`,
///  else one byte.
///  Why:
///  range detection must step over whole members.
fn read_class_atom(
    body: &str,
    i: usize,
) -> (String, usize) {
    let bb = body.as_bytes();
    if bb[i] == b'\\' && i + 1 < bb.len() {
        return (body[i..i + 2].to_string(), i + 2);
    }
    return (body[i..i + 1].to_string(), i + 1);
}

/// Returns the opposite-case range for a same-case letter range absent from `body`,
///  or `None`.
///
/// What:
///  for `a-z` yields `A-Z` unless `body` already contains it.
///  Why:
///  avoids re-adding a case
/// a class such as `[a-zA-Z0-9]` already spans.
fn widen_letter_range(
    lo: &str,
    hi: &str,
    body: &str,
) -> Option<String> {
    let (l, h) = (single_letter(lo)?, single_letter(hi)?);
    if l.is_ascii_lowercase() != h.is_ascii_lowercase() {
        return None;
    }
    let opp = format!("{}-{}", swap_case(l) as char, swap_case(h) as char);
    if body.contains(&opp) {
        return None;
    }
    return Some(opp);
}

/// Returns the opposite-case letter for a single-letter member absent from `body`,
///  or `None`.
///
/// What:
///  for `a` yields `A` unless `body` already contains it.
///  Why:
///  widens a lone letter such as
/// the `a` through `f` of a hex class.
fn widen_letter_single(
    atom: &str,
    body: &str,
) -> Option<String> {
    let a = single_letter(atom)?;
    let opp = swap_case(a) as char;
    if body.contains(opp) {
        return None;
    }
    return Some(opp.to_string());
}

/// Returns the byte of a one-character ASCII-letter atom,
///  or `None`.
///
/// What:
///  `Some(byte)` only for a bare single letter.
///  Why:
///  only letters have an opposite case.
fn single_letter(atom: &str) -> Option<u8> {
    let bytes = atom.as_bytes();
    if bytes.len() == 1 && bytes[0].is_ascii_alphabetic() {
        return Some(bytes[0]);
    }
    return None;
}

/// Returns the opposite ASCII case of a letter byte,
///  leaving non-letters unchanged.
///
/// What:
///  lowercase to uppercase and the reverse.
///  Why:
///  widening inserts the counterpart case.
fn swap_case(c: u8) -> u8 {
    if c.is_ascii_lowercase() {
        return c.to_ascii_uppercase();
    }
    return c.to_ascii_lowercase();
}
