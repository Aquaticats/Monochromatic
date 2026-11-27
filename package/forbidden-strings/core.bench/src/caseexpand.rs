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
//!
//! The three-casing of one literal run lives in the sibling
//! `caseexpand/three_casing_of_literal_runs.rs`,
//!  and the class widening in
//! `caseexpand/letter_case_widening_of_classes.rs`;
//!  both are split out to honor the max-lines budget.

/// Registers the three-casing of one flag-on literal run.
///  The `#[path]` is spelled out because this module is itself registered
/// through a `#[path]`,
///  which puts a plain `mod` child in `src/` instead of the module folder.
#[path = "caseexpand/three_casing_of_literal_runs.rs"]
mod three_casing_of_literal_runs;

/// Registers the class and quantified-letter widening applied under the flag.
///  The `#[path]` is spelled out for the same reason as the sibling above.
#[path = "caseexpand/letter_case_widening_of_classes.rs"]
mod letter_case_widening_of_classes;

/// Imports the class-span and group-span helpers shared with the porter.
use crate::port::{
    class_end,
    matching_close,
};

/// Imports the three-casing alternation a flag-on run flushes through.
use crate::caseexpand::three_casing_of_literal_runs::three_case;

/// Imports the class widening a `[...]` class body gets under the flag.
use crate::caseexpand::letter_case_widening_of_classes::widen_class;

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
/// //! A case-insensitive literal becomes the alternation of its three spellings.
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
fn expand_scope(
    s: &str,
    initial_ci: bool,
) -> String {
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
fn flush_run(
    run: &mut String,
    ci: bool,
    out: &mut String,
) {
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
fn emit_quantified_unit(
    unit: &str,
    quant: &str,
    ci: bool,
    out: &mut String,
) {
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
fn quant_len(
    b: &[u8],
    pos: usize,
) -> usize {
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
fn brace_end(
    b: &[u8],
    pos: usize,
) -> usize {
    for (offset, byte) in b[pos + 1..].iter().enumerate() {
        if *byte == b'}' {
            return pos + offset + 2;
        }
    }
    return pos + 1;
}
