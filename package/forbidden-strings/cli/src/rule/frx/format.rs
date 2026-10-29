//! Autodetects the rule-file format and parses it into engine-ready patterns.
//!
//! A leading UTF-8 BOM is stripped once.
//!  The first significant line (after blank
//! and `#`-comment lines) then selects the format:
//!  a strict `==> name <==` header
//! routes the whole file through the tail-format parser in the sibling `sections`
//! module,
//!  and any other shape routes the whole file through the legacy line-based
//! path here,
//!  which parses byte-for-byte as before.
//!  One file never mixes formats.
//!
//! The legacy path treats a blank or `#`-comment line as a skip,
//!  a `/PATTERN/FLAGS`
//! line whose trailing run is all ASCII-lowercase as a regex rule (with `m`/`x`
//! accepted as no-ops and any other flag a hard fail-closed error),
//!  and every other
//! non-blank line as a bare literal escaped into the verbose dialect,
//!  short ones
//! gated behind word boundaries.
//!  That single-line classification is shared with a
//! one-significant-line tail-format section,
//!  so both keep the incumbent two forms.

/// Imports the redacted load-error type reported on a bad flag or empty source.
use super::error::LoadError;

/// Imports the literal-to-dialect compiler (escaping plus short-literal boundary
/// gating) for bare literal lines.
use super::escape::literal_pattern;

/// Imports the tail-format detector,
///  parser,
///  and the shared parsed-rule type.
use super::sections::{detect_tail_format, parse_sections, ParsedRule};

/// The UTF-8 byte-order-mark code point,
///  stripped from the start of a source.
const BOM: char = '\u{FEFF}';

/// Strips a leading UTF-8 BOM,
///  returning the remaining source unchanged.
fn strip_bom(text: &str) -> &str {
    return text.strip_prefix(BOM).unwrap_or(text);
}

/// Reports whether a line contributes a rule:
///  non-blank and not a `#`-comment.
///
/// Shared by the legacy line scan,
///  tail-format autodetection,
///  and tail-format body
/// significance counting,
///  so every layer agrees on which lines are meaningful.
///  The
/// test trims first,
///  so an indented `#` line counts as a comment too,
///  matching the
/// engine's always-verbose comment handling.
pub(super) fn is_significant(line: &str) -> bool {
    let trimmed = line.trim();
    return !trimmed.is_empty() && !trimmed.starts_with('#');
}

/// Reports the first flag letter outside the `{m, x}` no-op set,
///  if any.
///
/// `m` (multiline) and `x` (verbose) are always on in the engine,
///  so they are
/// dropped as no-ops.
///  Any other letter would change match semantics if silently
/// dropped (an `i` or `s`),
///  so the caller fails the load closed on it.
fn first_bad_flag(flags: &str) -> Option<char> {
    return flags.chars().find(|&c| return c != 'm' && c != 'x');
}

/// One classified significant rule preserving exact-literal identity before escaping.
pub(super) struct SignificantRule {
    /// Engine-ready pattern used by restricted-regex compiler.
    pub(super) pattern: String,
    /// Original exact literal bytes,
    ///  absent for explicit regex syntax.
    pub(super) literal: Option<Vec<u8>>,
}

/// Compiles one significant line into its engine pattern by the incumbent two forms.
///
/// A regex line needs at least two delimiting slashes,
///  a closing slash past the
/// opening one,
///  and a trailing run that is entirely ASCII-lowercase (an empty run
/// counts);
///  its `m`/`x` flags drop as no-ops and any other flag fails closed with a
/// redacted error at `index`.
///  Any other shape,
///  including a slashed line whose
/// trailing run is not all-lowercase,
///  is a bare literal escaped into the verbose
/// dialect and short-literal boundary-gated.
///  The caller guarantees `line` is
/// significant,
///  so there is no skip case here.
pub(super) fn significant_line_rule(
    line: &str,
    index: usize,
) -> Result<SignificantRule, LoadError> {
    let trimmed = line.trim();
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 2
        && bytes[0] == b'/'
        && let Some(last) = trimmed.rfind('/')
        && last > 0
    {
        let flags = &trimmed[last + 1..];
        // Only an all-lowercase trailing run is a flags slot; otherwise the line is
        // a literal that merely happens to contain slashes.
        if flags.chars().all(|c| return c.is_ascii_lowercase()) {
            if let Some(flag) = first_bad_flag(flags) {
                return Err(LoadError::UnsupportedFlag { index, flag });
            }
            return Ok(SignificantRule {
                pattern: trimmed[1..last].to_string(),
                literal: None,
            });
        }
    }
    return Ok(SignificantRule {
        pattern: literal_pattern(trimmed),
        literal: Some(trimmed.as_bytes().to_vec()),
    })
}

/// Projects classified significant rule to engine-ready pattern for format tests.
#[cfg(test)]
pub(super) fn significant_line_pattern(
    line: &str,
    index: usize,
) -> Result<String, LoadError> {
    return significant_line_rule(line, index).map(|rule| return rule.pattern)
}

/// Parses a legacy line-based source into unnamed,
///  engine-ready rules.
///
/// Each significant line is classified by the two-form rule,
///  its 0-based position
/// among the kept rules becoming the opaque index a bad flag reports.
///  Returns
/// `NoRules` when no rule line survives,
///  and carries `None` as every rule's name
/// because the legacy format has no section identity.
fn parse_legacy(text: &str) -> Result<Vec<ParsedRule>, LoadError> {
    let mut rules: Vec<ParsedRule> = Vec::new();
    for line in text.lines() {
        if !is_significant(line) {
            continue;
        }
        let classified = significant_line_rule(line, rules.len())?;
        rules.push(ParsedRule {
            name: None,
            pattern: classified.pattern,
            literal: classified.literal,
        });
    }
    if rules.is_empty() {
        return Err(LoadError::NoRules);
    }
    return Ok(rules);
}

/// Parses a rule source into named,
///  engine-ready rules,
///  autodetecting the format.
///
/// Strips a leading BOM,
///  then routes on the first significant line:
///  a strict header
/// parses the whole file as tail-format sections (every rule named),
///  otherwise the
/// whole file parses as the legacy line-based format unchanged (every rule unnamed).
/// The names drive finding identity (`rule=<name>`);
///  the patterns feed the engine.
/// Every error is redacted,
///  carrying only an opaque index or a source line number,
/// never rule text.
pub(super) fn parse_rules(text: &str) -> Result<Vec<ParsedRule>, LoadError> {
    let stripped = strip_bom(text);
    if detect_tail_format(stripped) {
        return parse_sections(stripped);
    }
    return parse_legacy(stripped);
}

/// Parses a rule source into the engine-ready pattern list,
///  dropping rule names.
///
/// A projection over [`parse_rules`] for the format and compile tests,
///  which
/// assert against the bare `Vec<String>` pattern shape;
///  production callers all
/// need the names,
///  so this exists only in test builds.
#[cfg(test)]
pub(super) fn parse_patterns(text: &str) -> Result<Vec<String>, LoadError> {
    let rules = parse_rules(text)?;
    return Ok(rules.into_iter().map(|rule| return rule.pattern).collect());
}

/// Registers the format,
///  flags-policy,
///  and BOM-strip tests (sidecar,
///  lint-exempt).
#[cfg(test)]
#[path = "format_tests.rs"]
mod format_tests;
