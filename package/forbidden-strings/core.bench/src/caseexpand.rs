//! Three-casing expansion for inline `(?i)` scopes in the faithful port.
//!
//! What:
//!  rewrites the case-insensitive spans of a rule body into case-sensitive dialect that
//! covers the three shapes people actually write.
//!  A keyword literal run under `(?i)` becomes a
//! non-capturing alternation of its lowercase,
//!  per-run-Capitalized,
//!  and UPPERCASE forms
//! (`adobe` -> `(?:adobe|Adobe|ADOBE)`,
//!  `api_key` -> `(?:api_key|Api_Key|API_KEY)`);
//!  a character
//! class under `(?i)` widens each letter range or single to both cases (`[a-z]` -> `[a-zA-Z]`);
//!  a
//! single quantified letter under `(?i)` widens to a two-case class (`A{22}` -> `[aA]{22}`).
//!  Why:
//! the engine has no case-insensitivity flag,
//!  and the decided policy prefers matching the three
//! consistent shapes over a per-character both-case expansion that would also match mixed-case
//! noise like `AdOBe_`.
//!
//! Scope tracking follows PCRE:
//!  `(?i)` turns case-insensitivity on for the remainder of its
//! enclosing group,
//!  `(?i:...)` turns it on inside the group,
//!  and `(?-i:...)` turns it back off;
//! child groups inherit the flag state at their point.
//!  A pattern with no inline case flag is
//! returned byte-identical,
//!  so only the affected rules are transformed.

/// Imports the class-span and group-span helpers shared with the porter.
use crate::port::{class_end, matching_close};

/// Selects which casing `recase` applies to a literal run.
enum CaseMode {
    /// Every letter lowercase.
    Lower,
    /// First letter of each alphabetic run uppercase,
    ///  the rest lowercase.
    Cap,
    /// Every letter uppercase.
    Upper,
}

/// Expands every inline case-insensitive span of `pattern` into case-sensitive dialect.
///
/// What:
///  returns `pattern` unchanged when it carries no inline case flag,
///  else walks it under a
/// case-insensitivity flag that starts off.
///  Why:
///  guaranteeing byte-identity for the flagless
/// rules keeps the port's non-case output exactly as before.
///
/// @example
/// ```rust
/// assert_eq!(expand_case("(?i)adobe"), "(?:adobe|Adobe|ADOBE)");
/// ```
pub(crate) fn expand_case(pattern: &str) -> String {
    if !has_case_flag(pattern) {
        return pattern.to_string();
    }
    return expand_scope(pattern, false);
}

/// Reports whether `s` carries any inline case flag the walker must interpret.
///
/// What:
///  true for `(?i)`,
///  `(?i:`,
///  or `(?-i:`.
///  Why:
///  gates the byte-identity fast path.
fn has_case_flag(s: &str) -> bool {
    return s.contains("(?i)") || s.contains("(?i:") || s.contains("(?-i:");
}

/// Processes one (sub)pattern under case-insensitivity flag `initial_ci`.
///
/// What:
///  accumulates consecutive literal bytes into a run,
///  flushing it (three-cased when the
/// flag is on) at every class,
///  group,
///  metacharacter,
///  or flag change;
///  recurses into each group
/// with the child's flag;
///  an inline `(?i)` turns the flag on for the rest of this scope.
///  Why:
/// PCRE scopes a flag to its enclosing group,
///  so a single left-to-right walk with per-group
/// recursion models the exact case-insensitive spans.
fn expand_scope(s: &str, initial_ci: bool) -> String {
    let b = s.as_bytes();
    let mut out = String::new();
    let mut run = String::new();
    let mut ci = initial_ci;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'(' && s[i..].starts_with("(?i)") {
            flush_run(&mut run, ci, &mut out);
            ci = true;
            i += 4;
            continue;
        }
        if c == b'(' {
            flush_run(&mut run, ci, &mut out);
            let (prefix, child_ci, open_len): (&str, bool, usize) = if s[i..].starts_with("(?i:") {
                ("(?:", true, 4)
            } else if s[i..].starts_with("(?-i:") {
                ("(?:", false, 5)
            } else if s[i..].starts_with("(?:") {
                ("(?:", ci, 3)
            } else {
                ("(", ci, 1)
            };
            let close = matching_close(b, i);
            out.push_str(prefix);
            out.push_str(&expand_scope(&s[i + open_len..close], child_ci));
            out.push(')');
            i = close + 1;
            continue;
        }
        if c == b'[' {
            flush_run(&mut run, ci, &mut out);
            let end = class_end(b, i);
            if ci {
                out.push_str(&widen_class(&s[i..end]));
            } else {
                out.push_str(&s[i..end]);
            }
            i = end;
            continue;
        }
        if c == b'\\' {
            let n = b.get(i + 1).copied().unwrap_or(b'\\');
            if i + 1 < b.len() && is_run_literal_escape(n) {
                let q = quant_len(b, i + 2);
                if q > 0 {
                    flush_run(&mut run, ci, &mut out);
                    emit_quantified_unit(&s[i..i + 2], &s[i + 2..i + 2 + q], ci, &mut out);
                    i += 2 + q;
                } else {
                    run.push('\\');
                    run.push(n as char);
                    i += 2;
                }
            } else {
                flush_run(&mut run, ci, &mut out);
                out.push('\\');
                if i + 1 < b.len() {
                    out.push(n as char);
                }
                i += 2;
            }
            continue;
        }
        if matches!(c, b'?' | b'*' | b'+' | b')' | b']' | b'}') {
            flush_run(&mut run, ci, &mut out);
            out.push(c as char);
            i += 1;
            continue;
        }
        if c == b'{' {
            flush_run(&mut run, ci, &mut out);
            let end = brace_end(b, i);
            out.push_str(&s[i..end]);
            i = end;
            continue;
        }
        if matches!(c, b'.' | b'^' | b'$' | b'|' | b'~' | b'&') {
            flush_run(&mut run, ci, &mut out);
            out.push(c as char);
            i += 1;
            continue;
        }
        let q = quant_len(b, i + 1);
        if q > 0 {
            flush_run(&mut run, ci, &mut out);
            emit_quantified_unit(&s[i..i + 1], &s[i + 1..i + 1 + q], ci, &mut out);
            i += 1 + q;
        } else {
            run.push(c as char);
            i += 1;
        }
    }
    flush_run(&mut run, ci, &mut out);
    return out;
}

/// Emits the pending literal run,
///  three-cased when the case-insensitivity flag is on.
///
/// What:
///  no-op on an empty run;
///  otherwise appends the three-casing alternation (flag on) or the
/// run verbatim (flag off),
///  then clears it.
///  Why:
///  a run is the unit the three-casing applies to.
fn flush_run(run: &mut String, ci: bool, out: &mut String) {
    if run.is_empty() {
        return;
    }
    if ci {
        out.push_str(&three_case(run));
    } else {
        out.push_str(run);
    }
    run.clear();
}

/// Builds the non-capturing three-casing alternation of a literal run.
///
/// What:
///  the distinct lowercase,
///  per-run-Capitalized,
///  and UPPERCASE forms joined with `|` inside
/// `(?:...)`;
///  a run with no letter (all three forms equal) is returned bare.
///  Why:
///  matching only
/// the three consistent shapes is the decided approximation of case-insensitivity.
fn three_case(run: &str) -> String {
    let mut forms: Vec<String> = Vec::new();
    for form in [
        recase(run, CaseMode::Lower),
        recase(run, CaseMode::Cap),
        recase(run, CaseMode::Upper),
    ] {
        if !forms.contains(&form) {
            forms.push(form);
        }
    }
    if forms.len() == 1 {
        return forms.into_iter().next().unwrap_or_default();
    }
    return format!("(?:{})", forms.join("|"));
}

/// Applies one casing to a literal run,
///  casing bare ASCII letters and copying escapes intact.
///
/// What:
///  lowercases,
///  uppercases,
///  or Capitalizes (first letter of each alphabetic run upper,
///  rest
/// lower);
///  a backslash escape and any non-letter break the alphabetic run and pass through.
///  Why:
/// an escape such as `\_` or `\x60` must never have its bytes recased.
fn recase(run: &str, mode: CaseMode) -> String {
    let b = run.as_bytes();
    let mut out = String::new();
    let mut fresh = true;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' {
            out.push('\\');
            if i + 1 < b.len() {
                out.push(b[i + 1] as char);
            }
            fresh = true;
            i += 2;
            continue;
        }
        let c = b[i];
        if c.is_ascii_alphabetic() {
            let cased = match mode {
                CaseMode::Lower => c.to_ascii_lowercase(),
                CaseMode::Upper => c.to_ascii_uppercase(),
                CaseMode::Cap if fresh => c.to_ascii_uppercase(),
                CaseMode::Cap => c.to_ascii_lowercase(),
            };
            out.push(cased as char);
            fresh = false;
        } else {
            out.push(c as char);
            fresh = true;
        }
        i += 1;
    }
    return out;
}

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
fn widen_class(class: &str) -> String {
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
fn read_class_atom(body: &str, i: usize) -> (String, usize) {
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
fn widen_letter_range(lo: &str, hi: &str, body: &str) -> Option<String> {
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
fn widen_letter_single(atom: &str, body: &str) -> Option<String> {
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

/// Emits one quantified literal unit,
///  widening a single letter to a two-case class under `(?i)`.
///
/// What:
///  `A{22}` under the flag becomes `[aA]{22}`;
///  anything non-letter or with the flag off is
/// emitted verbatim with its quantifier.
///  Why:
///  a quantifier binds one atom,
///  so the letter cannot
/// join a keyword run and instead widens like a class.
fn emit_quantified_unit(unit: &str, quant: &str, ci: bool, out: &mut String) {
    let bytes = unit.as_bytes();
    if ci && bytes.len() == 1 && bytes[0].is_ascii_alphabetic() {
        out.push('[');
        out.push(bytes[0].to_ascii_lowercase() as char);
        out.push(bytes[0].to_ascii_uppercase() as char);
        out.push(']');
        out.push_str(quant);
        return;
    }
    out.push_str(unit);
    out.push_str(quant);
}

/// Reports whether `\n` denotes a literal that joins a run rather than a shorthand or anchor.
///
/// What:
///  false for the class shorthands,
///  anchors,
///  control escapes,
///  and backreference digits;
/// true otherwise.
///  Why:
///  escaped punctuation such as `\_` is part of a keyword,
///  but `\w` or `\b`
/// ends the run.
fn is_run_literal_escape(n: u8) -> bool {
    return !n.is_ascii_digit() && !b"wWdDsSbBAzZGnrtfv".contains(&n);
}

/// Returns the byte length of a quantifier at `pos`,
///  or zero when none.
///
/// What:
///  one for `?`/`*`/`+`,
///  the full `{...}` span for a brace,
///  zero otherwise.
///  Why:
///  the walker
/// must consume a quantifier together with the unit it binds.
fn quant_len(b: &[u8], pos: usize) -> usize {
    match b.get(pos) {
        Some(b'?' | b'*' | b'+') => return 1,
        Some(b'{') => {
            let span = brace_end(b, pos) - pos;
            if span > 1 {
                return span;
            }
            return 0;
        }
        _ => return 0,
    }
}

/// Returns the index just past the `}` closing a brace at `pos`,
///  or `pos + 1` when unterminated.
///
/// What:
///  scans forward to the first `}`.
///  Why:
///  a `{n,m}` quantifier is copied or measured whole.
fn brace_end(b: &[u8], pos: usize) -> usize {
    for (offset, byte) in b[pos + 1..].iter().enumerate() {
        if *byte == b'}' {
            return pos + offset + 2;
        }
    }
    return pos + 1;
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
