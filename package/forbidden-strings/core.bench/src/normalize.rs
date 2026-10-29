//! Normalizes a rule body into the forbidden-regex dialect.
//!
//! What:
//!  string-level substitutions (POSIX classes,
//!  case flags,
//!  `\_`/`\x60`) plus an
//! escape-aware rewrite of capturing groups and unbounded quantifiers.
//!  Why:
//!  the real
//! ruleset is written in a richer dialect than this engine accepts,
//!  so each rule is
//! adapted before it is parsed.

/// Imports the class-span helper shared with the porter.
use crate::port::class_end;

/// Upper bound substituted for an unbounded `*`,
///  `+`,
///  or `{n,}` quantifier.
const QUANT_CAP: usize = 512;

/// POSIX class spellings and their byte-class equivalents.
const POSIX_CLASSES: [(&str, &str); 5] = [
    ("[[:alnum:]]", "[A-Za-z0-9]"),
    ("[[:digit:]]", "[0-9]"),
    ("[[:xdigit:]]", "[0-9a-fA-F]"),
    ("[[:alpha:]]", "[A-Za-z]"),
    ("[[:space:]]", "\\s"),
];

/// Applies the string-level substitutions,
///  then the scan-based rewrite.
///
/// What:
///  POSIX classes,
///  case-flag stripping,
///  `\_`/`\x60` literals,
///  then capturing
/// and quantifier rewriting.
///  Why:
///  the cheap global replacements first keep the scan
/// simpler.
pub(crate) fn normalize(inner: &str) -> String {
    let mut s = inner.to_string();
    for (from, to) in POSIX_CLASSES {
        s = s.replace(from, to);
    }
    s = s.replace("(?i:", "(?:");
    s = s.replace("(?-i:", "(?:");
    s = s.replace("(?i)", "");
    s = s.replace("\\_", "_");
    s = s.replace("\\x60", "`");
    return scan_rewrite(&s)
}

/// Rewrites capturing groups and unbounded quantifiers in one escape-aware pass.
///
/// What:
///  a plain `(` becomes `(?:`,
///  `*`/`+` become bounded repetitions,
///  and `{n,}`
/// gains an upper bound;
///  complement parens and class contents are left untouched.
/// Why:
///  the engine has no capturing groups and rejects unbounded repetition.
fn scan_rewrite(s: &str) -> String {
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
            continue;
        }
        if c == b'[' {
            let end = class_end(b, i);
            out.push_str(&s[i..end]);
            i = end;
            continue;
        }
        i = rewrite_token(s, b, i, &mut out);
    }
    return out
}

/// Rewrites one non-class,
///  non-escape token,
///  returning the next index.
///
/// What:
///  handles a group open,
///  the `*`/`+` quantifiers,
///  and a `{n,}` bound;
///  copies
/// anything else verbatim.
///  Why:
///  keeps `scan_rewrite` within its line budget.
fn rewrite_token(s: &str, b: &[u8], i: usize, out: &mut String) -> usize {
    match b[i] {
        b'(' if open_is_capturing(b, i) => {
            out.push_str("(?:");
            return i + 1
        }
        b'*' => {
            out.push_str(&format!("{{0,{QUANT_CAP}}}"));
            return i + 1
        }
        b'+' => {
            out.push_str(&format!("{{1,{QUANT_CAP}}}"));
            return i + 1
        }
        b'{' => return rewrite_brace(s, b, i, out),
        other => {
            out.push(other as char);
            return i + 1
        }
    }
}

/// Reports whether a `(` at `i` opens a plain capturing group.
///
/// What:
///  false when it is `(?...` or the complement `~(`.
///  Why:
///  only capturing
/// groups are rewritten to non-capturing.
fn open_is_capturing(b: &[u8], i: usize) -> bool {
    let is_flagged = i + 1 < b.len() && b[i + 1] == b'?';
    let is_complement = i > 0 && b[i - 1] == b'~' && (i < 2 || b[i - 2] != b'\\');
    return !is_flagged && !is_complement
}

/// Rewrites a `{...}` token,
///  bounding `{n,}`,
///  returning the next index.
///
/// What:
///  copies `{n}`/`{n,m}` as-is and turns `{n,}` into `{n,CAP}`.
///  Why:
///  the engine
/// rejects an open-ended upper bound.
fn rewrite_brace(s: &str, b: &[u8], i: usize, out: &mut String) -> usize {
    let Some(close) = b[i..].iter().position(|&x| return x == b'}').map(|p| return i + p) else {
        out.push('{');
        return i + 1;
    };
    let body = &s[i + 1..close];
    if body.len() >= 2 && body.ends_with(',') && body[..body.len() - 1].bytes().all(|x| return x.is_ascii_digit()) {
        out.push_str(&format!("{{{},{QUANT_CAP}}}", &body[..body.len() - 1]));
    } else {
        out.push_str(&s[i..=close]);
    }
    return close + 1
}
