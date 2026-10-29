//! Parses the tail-format sectioned rule file into named engine-ready rules.
//!
//! A tail-format file is what `tail --verbose -n +1` over per-rule files would
//! produce:
//!  `==> name <==` header lines open sections whose bodies are rules.
//! Every section is exactly one rule,
//!  so every rule carries its own name.
//!  A body
//! with one significant line classifies by the incumbent two-form rule (bare
//! literal or single-line `/PATTERN/FLAGS`);
//!  a body with more than one significant
//! line is one verbatim pattern handed to the always-verbose engine,
//!  which ignores
//! blank lines and first-column `#` comments itself.
//!  Arrow-leading near-headers,
//! duplicate names,
//!  empty sections,
//!  and pre-header content each fail the load
//! closed with a redacted,
//!  line-numbered error.
//!  Autodetection and the byte-for-byte
//! legacy path live in the sibling `format` module.

/// Imports the redacted load-error type reported on every fail-closed condition.
use super::error::LoadError;

/// Imports the shared significance test and the two-form single-line compiler.
use super::format::{is_significant, significant_line_rule};

/// Imports the name-to-line map backing duplicate-name enforcement.
use std::collections::HashMap;

/// The exact prefix a section header line opens with,
///  before the name.
const HEADER_PREFIX: &str = "==> ";

/// The exact suffix a section header line closes with,
///  after the name.
const HEADER_SUFFIX: &str = " <==";

/// The bare arrow whose trimmed-leading presence makes a line header-adjacent:
/// such a line must be exactly a strict header or the load fails closed.
const NEAR_ARROW: &str = "==>";

/// One parsed rule:
///  its engine pattern plus its retained section name,
///  if any.
///
/// Tail-format rules carry `Some(name)` (the section identity that will drive the
/// separate rule-identity UX decision);
///  legacy line-based rules carry `None`.
///  This
/// issue keeps finding output unchanged,
///  so the name is retained but not rendered;
/// its first production consumer is duplicate-name enforcement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedRule {
    /// Retained section name for a tail-format rule,
    ///  or `None` for a legacy rule.
    pub(super) name: Option<String>,
    /// Engine-ready pattern the compiler validates and assembles into the set.
    pub(super) pattern: String,
    /// Original exact literal bytes,
    ///  absent for explicit or multiline regex rule.
    pub(super) literal: Option<Vec<u8>>,
}

/// The classification of one line against the section-header grammar.
enum HeaderKind {
    /// A well-formed strict header carrying its validated section name.
    Strict(
        /// Validated lowercase-kebab-with-dots section name.
        String,
    ),
    /// A line whose trimmed form leads with `==>` yet is not exactly a strict header.
    Near,
    /// A line without a trimmed leading arrow that stays section-body content.
    NotHeader,
}

/// A section header opened but whose body has not yet been closed and classified.
struct OpenSection<'a> {
    /// Validated section name,
    ///  moved into the finished rule.
    name: String,
    /// 1-based source line of the header,
    ///  used for empty-section reporting.
    name_line: usize,
    /// Raw body lines collected verbatim until the next header or end of file.
    body: Vec<&'a str>,
}

/// Reports whether a character may open a strict section name (`[a-z0-9]`).
fn is_name_start(ch: char) -> bool {
    return ch.is_ascii_lowercase() || ch.is_ascii_digit();
}

/// Reports whether a character may continue a strict section name (`[a-z0-9.-]`).
fn is_name_char(ch: char) -> bool {
    return is_name_start(ch) || ch == '.' || ch == '-';
}

/// Extracts the validated section name from a line that is exactly a strict header.
///
/// A strict header is an exact full untrimmed line `==> ` + name + ` <==`,
///  the
/// name drawn from `[a-z0-9.-]` and opening with `[a-z0-9]`;
///  every other shape
/// returns `None`.
fn strict_name(line: &str) -> Option<String> {
    let middle = line.strip_prefix(HEADER_PREFIX)?.strip_suffix(HEADER_SUFFIX)?;
    let first = middle.chars().next()?;
    if !is_name_start(first) || !middle.chars().all(|ch| return is_name_char(ch)) {
        return None;
    }
    return Some(middle.to_string());
}

/// Classifies one line as a strict header,
///  a near-header,
///  or ordinary content.
///
/// Every line whose trimmed form starts with `==>` must be exactly a strict header
/// or it is a near-header failing the load closed (maintainer ruling 2026-07-20,
/// replacing a draft where out-of-alphabet middles such as `X9` or `/etc/passwd`
/// stayed content):
///  case typos,
///  indented would-be headers,
///  missing-space arrows,
/// and genuine arrow-leading content alike.
///  Genuine content uses the `[=]=> `
/// reshape,
///  whose trimmed form starts with `[`,
///  so nothing arrow-leading is ever
/// silently absorbed into a section body.
fn classify_header(line: &str) -> HeaderKind {
    if !line.trim_start().starts_with(NEAR_ARROW) {
        return HeaderKind::NotHeader;
    }
    // Strictness is judged on the untrimmed full line, so indentation disqualifies.
    return match strict_name(line) {
        Some(name) => HeaderKind::Strict(name),
        None => HeaderKind::Near,
    };
}

/// Reports whether a line is a strict section header,
///  for format autodetection.
///
/// The sibling `format` module's `parse_rules` calls this on the first
/// significant line:
///  a strict header there selects the tail-format path,
///  any other
/// shape selects the byte-for-byte legacy path.
pub(super) fn detect_tail_format(text: &str) -> bool {
    return text
        .lines()
        .find(|&line| return is_significant(line))
        .is_some_and(|line| return strict_name(line).is_some());
}

/// Classifies one closed section's body into a single named engine rule.
///
/// Trailing blank lines are the section-separator tail plus any the author left,
///  so
/// they are trimmed before significance is counted and before the verbatim body is
/// formed.
///  A body with no significant line fails closed as an empty section.
///  One
/// significant line classifies by the incumbent two-form rule at `index` (a
/// single-line regex there can still fail the flag policy).
///  More than one keeps the
/// trimmed body verbatim as one pattern,
///  comments and interior blanks included,
///  for
/// the always-verbose engine to consume.
fn finish_section(section: OpenSection<'_>, index: usize) -> Result<ParsedRule, LoadError> {
    let end = section
        .body
        .iter()
        .rposition(|line| return !line.trim().is_empty())
        .map_or(0, |last| return last + 1);
    let body = &section.body[..end];
    let significant: Vec<&str> =
        body.iter().copied().filter(|&line| return is_significant(line)).collect();
    if significant.is_empty() {
        return Err(LoadError::EmptySection { line: section.name_line });
    }
    if significant.len() == 1 {
        let classified = significant_line_rule(significant[0], index)?;
        return Ok(ParsedRule {
            name: Some(section.name),
            pattern: classified.pattern,
            literal: classified.literal,
        });
    }
    return Ok(ParsedRule {
        name: Some(section.name),
        pattern: body.join("\n"),
        literal: None,
    })
}

/// Closes one open section into the parallel rule and header-line lists.
///
/// The rule's index in the compiled set is its push position,
///  which the two-form
/// classifier reports on a bad flag;
///  the retained header line feeds duplicate-name
/// reporting after the walk.
fn close_section(
    section: OpenSection<'_>,
    rules: &mut Vec<ParsedRule>,
    name_lines: &mut Vec<usize>,
) -> Result<(), LoadError> {
    let name_line = section.name_line;
    let rule = finish_section(section, rules.len())?;
    rules.push(rule);
    name_lines.push(name_line);
    return Ok(());
}

/// Parses BOM-stripped tail-format text into named,
///  engine-ready rules in file order.
///
/// Walks lines once:
///  a strict header closes any open section and opens the next;
///  a
/// near-header,
///  or significant content before the first header,
///  fails closed with
/// its source line;
///  anything else accumulates into the current body.
///  After the
/// walk,
///  name uniqueness is enforced across the whole loaded input by reading each
/// rule's retained name.
///  Rule order is file order,
///  giving deterministic indices.
pub(super) fn parse_sections(text: &str) -> Result<Vec<ParsedRule>, LoadError> {
    let mut rules: Vec<ParsedRule> = Vec::new();
    let mut name_lines: Vec<usize> = Vec::new();
    let mut current: Option<OpenSection<'_>> = None;

    for (offset, line) in text.lines().enumerate() {
        let line_number = offset + 1;
        match classify_header(line) {
            HeaderKind::Strict(name) => {
                if let Some(section) = current.take() {
                    close_section(section, &mut rules, &mut name_lines)?;
                }
                current = Some(OpenSection { name, name_line: line_number, body: Vec::new() });
            }
            HeaderKind::Near => {
                return Err(LoadError::NearHeader { line: line_number });
            }
            HeaderKind::NotHeader => match current.as_mut() {
                Some(section) => section.body.push(line),
                None => {
                    if is_significant(line) {
                        return Err(LoadError::PreHeaderContent { line: line_number });
                    }
                }
            },
        }
    }
    if let Some(section) = current.take() {
        close_section(section, &mut rules, &mut name_lines)?;
    }

    // Autodetection guarantees at least one header reached this path, but fail
    // closed rather than return an empty set if that invariant is ever broken.
    if rules.is_empty() {
        return Err(LoadError::NoRules);
    }

    // Enforce name uniqueness over the whole loaded input; reading each rule's
    // retained name here is the parsed representation's first production consumer.
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for (rule, &name_line) in rules.iter().zip(&name_lines) {
        let Some(name) = rule.name.as_deref() else {
            continue;
        };
        if let Some(&first_line) = seen.get(name) {
            return Err(LoadError::DuplicateName { first_line, line: name_line });
        }
        seen.insert(name, name_line);
    }

    return Ok(rules);
}

/// Registers the header-grammar,
///  section-body,
///  and fail-closed tests (sidecar,
///  lint-exempt).
#[cfg(test)]
#[path = "sections_tests.rs"]
mod sections_tests;
