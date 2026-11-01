//! Per-pattern porting pass and line-level classification for the dialect porter.
//!
//! Normalizes one PCRE body into the restricted dialect (case expansion,
//! normalization,
//!  escape fixing,
//!  leading-context stripping,
//!  operand wrapping,
//! and the two settled reshapes),
//!  classifies each `/PATTERN/FLAGS` line,
//!  and
//! ports a whole source keeping every line on its original number.
//!  Split out of
//! the `dialectport` bin to satisfy the max-lines budget.

/// Imports the inline `(?i)` three-casing expander applied before normalization.
use crate::caseexpand::expand_case;

/// Imports the dialect normalizer applied before wrapping.
use crate::normalize::normalize;

/// Imports the character-class span helper shared with the porter.
use crate::port::class_end;

/// Imports the operand-wrapping and top-level-split helpers.
use crate::atomwrap::{split_top_level, wrap_alternation, wrap_operand};

/// Imports the escape and verbose-whitespace rewrite pass.
use crate::escapefix::fix_escapes;

/// Sentinel char that stands in for an escaped backslash (`\\`) while `normalize` runs.
///
/// What:
///  U+0001,
///  absent from the printable-ASCII rule corpus.
///  Why:
///  `normalize`'s naive
/// `\_`/`\x60` string replacements would corrupt a `\\_` (escaped backslash then literal
/// underscore) into `\_`;
///  protecting every `\\` as this sentinel across the `normalize`
/// call,
///  then restoring it,
///  keeps escaped backslashes intact.
const SENTINEL: &str = "\u{1}";

/// Normalizes and escape-fixes one operand into a dialect body,
///  before branch wrapping.
///
/// What:
///  three-casing-expands inline `(?i)` scopes,
///  protects escaped backslashes,
///  runs the
/// shared normalizer (POSIX classes,
///  capturing groups,
///  quantifier bounding;
///  its case-flag
/// strip is now a no-op because `expand_case` already consumed every `(?i)`),
///  restores
/// backslashes,
///  then fixes the remaining escapes and verbose whitespace.
///  Why:
///  `expand_case`
/// must see the raw case flags before normalization,
///  and `normalize`'s naive `\_` replacement
/// would otherwise corrupt an escaped backslash that precedes an underscore.
fn dialect_body(operand: &str) -> String {
    let expanded = expand_case(operand);
    let protected = expanded.replace("\\\\", SENTINEL);
    let normalized = normalize(&protected);
    let restored = normalized.replace(SENTINEL, "\\\\");
    return fix_escapes(&restored);
}

/// Returns the byte length of a leading nullable class-repeat `[...]{0,N}`,
///  or `None`.
///
/// What:
///  matches a class at index zero immediately followed by `{0,N}`.
///  Why:
///  such a repeat
/// is nullable,
///  so under unanchored line search it is pure redundant preceding context.
fn leading_class_repeat_len(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    if b.first() != Some(&b'[') {
        return None;
    }
    let after = class_end(b, 0);
    let rest = s.get(after..)?;
    let bounded = rest.strip_prefix("{0,")?;
    let brace = bounded.find('}')?;
    return Some(after + 3 + brace + 1);
}

/// Strips one leading nullable class-repeat,
///  at the pattern head or just inside a head group.
///
/// What:
///  removes a leading `[...]{0,N}`,
///  or one nested right after a leading `(?:`.
///  Why:
///  the
/// betterleaks generic detectors prepend `[\w.-]{0,50}` context that is redundant under
/// unanchored search but makes the engine's whole-line matcher determinize catastrophically
/// slowly (measured minutes per rule);
///  a leading `\b`/`^` anchor is real and left intact.
fn strip_one_leading(s: &str) -> Option<String> {
    if let Some(end) = leading_class_repeat_len(s) {
        return Some(s[end..].to_string());
    }
    if let Some(rest) = s.strip_prefix("(?:")
        && let Some(end) = leading_class_repeat_len(rest)
    {
        return Some(format!("(?:{}", &rest[end..]));
    }
    return None;
}

/// Repeatedly strips leading nullable class-repeats until none remains.
///
/// What:
///  applies `strip_one_leading` to a fixpoint.
///  Why:
///  a rule may nest a second redundant
/// `[\w.-]{0,50}` just inside its leading group.
fn strip_leading_redundant(s: &str) -> String {
    let mut cur = s.to_string();
    while let Some(next) = strip_one_leading(&cur) {
        cur = next;
    }
    return cur;
}

/// Ports one `/PATTERN/` body into the restricted dialect,
///  faithful except for two
/// semantics-preserving simplifications:
///  leading nullable class-repeats are stripped and the
/// operators' operands are wrapped to single atoms.
///
/// What:
///  splits top-level `&` operands,
///  normalizes and escape-fixes each,
///  strips the leading
/// redundant repeat off the positive operand,
///  then wraps `|` and `&` operands.
///  Why:
///  the port
/// keeps the rule's full match shape (delimiters,
///  interior repeats,
///  trailer) while adapting
/// it to the engine's grammar and clearing the compile-time blowup on leading context.
fn faithful_port(pattern: &str) -> String {
    let operands = split_top_level(pattern, b'&');
    if operands.len() == 1 {
        return wrap_alternation(&strip_leading_redundant(&dialect_body(pattern)));
    }
    let parts: Vec<String> = operands
        .iter()
        .enumerate()
        .map(|(index, operand)| {
            let raw_body = dialect_body(operand);
            let body = if index == 0 { strip_leading_redundant(&raw_body) } else { raw_body };
            return wrap_operand(&body);
        })
        .collect();
    return parts.join("&");
}

/// Reports whether porting `pattern` strips a leading redundant class-repeat.
///
/// What:
///  recomputes the positive operand's dialect body and checks whether the leading strip
/// shortens it.
///  Why:
///  the review report records this semantics-preserving simplification.
fn leading_stripped(pattern: &str) -> bool {
    let operands = split_top_level(pattern, b'&');
    let body = dialect_body(&operands[0]);
    return strip_leading_redundant(&body).len() != body.len();
}

/// Marker prefix identifying the single cross-line curl basic-auth builtin rule.
const CURL_PREFIX: &str = "\\bcurl\\b";

/// Substring at which the curl rule's kept credential shape begins.
const CURL_CREDENTIAL_MARKER: &str = "(?:-u|--user)";

/// Line-start or horizontal-whitespace boundary required before curl user options.
const CURL_OPTION_BOUNDARY: &str = "(?:^|[ \\t])";

/// Source-pattern prefix identifying the single mongodb connection-string builtin rule.
const MONGODB_PREFIX: &str = "\\b(mongodb";

/// Credential-bearing core the mongodb rule is reshaped to:
///  the scheme,
///  bounded user and
/// password phases,
///  and the `@` delimiter.
///  The delimiter-excluding user and password classes
/// keep the phases deterministic,
///  and dropping the non-secret host,
///  port,
///  replica-set,
///  and path
/// suffix removes the nested counters that determinized past the engine's DFA state cap.
///
/// This pattern text does not self-match the compiled rule (probed 2026-07-20):
///  the rule
/// requires `://` immediately after the scheme,
///  but this text interposes `(?:\+srv)?`.
const MONGODB_CORE: &str = "\\bmongodb(?:\\+srv)?://[!-9;-~]{3,50}:[!-?A-~]{3,88}@";

/// Ports one rule pattern,
///  reshaping the curl and mongodb rules per the settled decisions.
///
/// What:
///  for the curl rule,
///  drops the leading `\bcurl\b` context and the cross-line window,
/// retains a line-start or horizontal-whitespace option boundary,
///  and ports the
/// `(?:-u|--user)` credential shape onward;
///  for the mongodb rule,
///  emits the credential-bearing
/// core directly;
///  every other rule is ported whole.
///  Why:
///  rule 172 is the only cross-line rule
/// and its separately-tokenized credential pair is the payload,
///  and rule 518 otherwise
/// determinizes past the state cap while its non-secret URI suffix carries no leak.
fn port_pattern(pattern: &str) -> (String, bool) {
    if pattern.starts_with(CURL_PREFIX) {
        let idx = pattern
            .find(CURL_CREDENTIAL_MARKER)
            .expect("curl rule contains the -u/--user credential marker");
        let credential_pattern = faithful_port(&pattern[idx..]);
        return (format!("{CURL_OPTION_BOUNDARY}{credential_pattern}"), true);
    }
    if pattern.starts_with(MONGODB_PREFIX) {
        return (MONGODB_CORE.to_string(), true);
    }
    return (faithful_port(pattern), false);
}

/// Splits a `/PATTERN/FLAGS` line into its pattern body and flags slot.
///
/// What:
///  mirrors the scanner's `rule::parse` exactly:
///  a line of two or more bytes starting
/// with `/` whose LAST `/` leaves an all-lowercase flags tail is a regex rule;
///  the body is
/// between the first and last `/`.
///  Why:
///  a rule body carries unescaped `/` (URLs,
///  `[.../]`
/// classes),
///  so anchoring on the last `/` (not the first) matches the loader;
///  anything else
/// is a literal line and returns `None`.
fn extract(line: &str) -> Option<(&str, &str)> {
    if line.len() < 2 || !line.starts_with('/') {
        return None;
    }
    let last = line.rfind('/')?;
    if last == 0 {
        return None;
    }
    let flags = &line[last + 1..];
    if !flags.chars().all(|c| return c.is_ascii_lowercase()) {
        return None;
    }
    return Some((&line[1..last], flags));
}

/// Reports whether a source pattern has an unbounded quantifier outside a class.
///
/// What:
///  an unescaped `*`,
///  `+`,
///  or `{n,}` outside `[...]`.
///  Why:
///  each becomes a bounded form
/// (cap 512),
///  a semantic change worth recording.
fn has_unbounded_quant(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' => i += if i + 1 < b.len() { 2 } else { 1 },
            b'[' => i = class_end(b, i),
            b'*' | b'+' => return true,
            b'{' => {
                let Some(rel) = b[i..].iter().position(|&x| return x == b'}') else {
                    i += 1;
                    continue;
                };
                let body = &s[i + 1..i + rel];
                if body.len() >= 2
                    && body.ends_with(',')
                    && body[..body.len() - 1].bytes().all(|x| return x.is_ascii_digit())
                {
                    return true;
                }
                i += rel + 1;
            }
            _ => i += 1,
        }
    }
    return false;
}

/// Reports whether a source pattern carries a bare `\n` or `\r` escape.
///
/// What:
///  a backslash immediately followed by `n` or `r`,
///  skipping escaped backslashes.
///  Why:
/// these are dropped,
///  a (line-scan-inert) change worth recording.
fn has_crlf_escape(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b'\\' {
            if b[i + 1] == b'n' || b[i + 1] == b'r' {
                return true;
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    return false;
}

/// One ported regex rule with its source line,
///  bodies,
///  and change classification.
///
/// What:
///  the 1-based source line,
///  the source and ported pattern bodies,
///  the dropped flags,
/// and the boolean change categories.
///  Why:
///  the verifier and the review report read these.
pub(crate) struct Ported {
    /// One-based source line number,
    ///  preserved in the output.
    pub(crate) line: usize,
    /// Source pattern body between the delimiters.
    pub(crate) source: String,
    /// Ported pattern body in the restricted dialect.
    pub(crate) ported: String,
    /// Flags slot dropped from the source line (`m`/`x` no-ops).
    pub(crate) flags: String,
    /// Inline case flag (`(?i)`,
    ///  `(?i:`,
    ///  `(?-i:`) three-casing-expanded (approximately preserved).
    pub(crate) case: bool,
    /// Unbounded quantifier bounded to the cap.
    pub(crate) quant: bool,
    /// Curl rule reshaped (curl context and cross-line window dropped;
    ///  option boundary retained).
    pub(crate) reshape: bool,
    /// Bare `\n`/`\r` escape dropped.
    pub(crate) crlf: bool,
    /// `\z` rewritten to `$`.
    pub(crate) zanchor: bool,
    /// Leading nullable class-repeat stripped (semantics-preserving,
    ///  clears compile blowup).
    pub(crate) leadstrip: bool,
}

/// Classifies and ports one `/PATTERN/FLAGS` line.
///
/// What:
///  extracts the body and flags,
///  ports the body,
///  and records every change category.
/// Why:
///  one place produces both the output line and the review classification.
fn classify(line_no: usize, pattern: &str, flags: &str) -> Ported {
    let (ported, reshape) = port_pattern(pattern);
    return Ported {
        line: line_no,
        source: pattern.to_string(),
        ported,
        flags: flags.to_string(),
        case: pattern.contains("(?i)") || pattern.contains("(?i:") || pattern.contains("(?-i:"),
        // The mongodb reshape drops (does not bound) its unbounded repeats, so it leaves the
        // quantifier-bounding category even though its source carries `*`/`+`.
        quant: has_unbounded_quant(pattern) && !pattern.starts_with(MONGODB_PREFIX),
        reshape,
        crlf: has_crlf_escape(pattern),
        zanchor: pattern.contains("\\z"),
        leadstrip: leading_stripped(pattern),
    };
}

/// Ports one whole rule-file text,
///  keeping every line on its original number.
///
/// What:
///  passes non-`/` lines (section headers,
///  literals,
///  comments,
///  blanks) through
/// byte-identically and rewrites each `/PATTERN/FLAGS` line to `/PORTED/` with flags
/// dropped.
///  Why:
///  the 1-based line alignment is load-bearing for differential review,
/// and the tail-format section headers stage one emits must reach the output untouched.
pub(crate) fn port_source(text: &str) -> (String, Vec<Ported>) {
    let mut out: Vec<String> = Vec::new();
    let mut rules: Vec<Ported> = Vec::new();
    for (index, line) in text.split('\n').enumerate() {
        let Some((pattern, flags)) = extract(line) else {
            // Header, literal, comment, blank line, or a `/`-line that is not a valid
            // regex rule: pass through byte-identically, keeping the line alignment.
            out.push(line.to_string());
            continue;
        };
        let ported = classify(index + 1, pattern, flags);
        out.push(format!("/{}/", ported.ported));
        rules.push(ported);
    }
    return (out.join("\n"), rules);
}
