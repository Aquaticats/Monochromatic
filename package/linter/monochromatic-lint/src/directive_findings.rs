//! What:
//!  The findings that directives themselves produce.
//! Why:
//!  A directive is checked like code:
//!  a broken one is reported with a stable identifier and the host's own position units,
//!  in the same record shape as every other finding.
//!  None of these identifiers is a configurable rule or a suppressible one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class DirectiveReporter { missingJustification(site, kind): Diagnostic; unknownRule(token): Diagnostic; /* ... */ }
//! ```

/// Import the finding model and the scanned file that positions it.
use crate::{
    diagnostic::{Diagnostic, Severity},
    directive_parse::{DirectiveKind, RuleToken},
    directive_scan::{CommentSite, ScannedComments},
    run_modes::RULES,
};

/// What:
///  The code of a directive that has no justification.
/// Why:
///  The directive then suppresses nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MISSING_JUSTIFICATION = 'directive/missing-justification';
/// ```
pub const MISSING_JUSTIFICATION: &str = "directive/missing-justification";

/// What:
///  The code of a directive entry naming an identifier outside the rule registry.
/// Why:
///  A misspelled identifier would otherwise suppress nothing without anyone noticing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNKNOWN_RULE = 'directive/unknown-rule';
/// ```
pub const UNKNOWN_RULE: &str = "directive/unknown-rule";

/// What:
///  The code of a directive entry aimed at a rule that declares itself unsuppressible.
/// Why:
///  The entry is refused and the rule's findings stay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NON_SUPPRESSIBLE = 'directive/non-suppressible';
/// ```
pub const NON_SUPPRESSIBLE: &str = "directive/non-suppressible";

/// What:
///  The code of a comment that begins like a directive but cannot be read as one.
/// Why:
///  An unknown keyword,
/// an empty list entry and a line directive spread over several lines all land here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MALFORMED = 'directive/malformed';
/// ```
pub const MALFORMED: &str = "directive/malformed";

/// What:
///  The code of a directive that changed nothing.
/// Why:
///  Reported only on request,
///  so stale directives can be removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNUSED = 'directive/unused';
/// ```
pub const UNUSED: &str = "directive/unused";

/// What:
///  Builds directive findings for one host file.
/// Why:
///  Every finding needs the display name and the host's position conventions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new DirectiveReporter(filename, scanned)
/// ```
pub struct Reporter<'file> {
    /// Display name of the host file,
    ///  as in every other finding.
    pub filename: &'file str,
    /// The parse whose line and column units position the findings.
    pub scanned: &'file ScannedComments,
}

/// What:
///  Construct the findings.
/// Why:
///  Each message names the affected comment or identifier,
///  says what the tool did about it,
///  and lists the ways to resolve it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reporter.malformed(site, reason)
/// ```
impl Reporter<'_> {
    /// Build one error or warning over a host byte range.
    fn report(
        &self,
        code: &str,
        severity: Severity,
        range: (usize, usize),
        message: String,
        help: String,
    ) -> Diagnostic {
        let mut finding: Diagnostic = Diagnostic::new(
            code,
            severity,
            message,
            String::from(self.filename),
            self.scanned.span(range.0, range.1 - range.0),
        );
        finding.help = Some(help);
        return finding;
    }

    /// A directive without a reason is void.
    pub fn missing_justification(&self, site: CommentSite, kind: DirectiveKind) -> Diagnostic {
        let keyword: &str = kind.keyword();
        return self.report(
            MISSING_JUSTIFICATION,
            Severity::Error,
            (site.start, site.end),
            format!("{keyword} has no justification, so it suppresses nothing."),
            format!(
                "Write \" -- <reason>\" after the rule identifiers, for example \"{keyword} rust/tests-in-sidecars -- generated bindings\", or remove the directive."
            ),
        );
    }

    /// A comment that starts like a directive but cannot be read.
    pub fn malformed(&self, site: CommentSite, reason: &str) -> Diagnostic {
        return self.report(
            MALFORMED,
            Severity::Error,
            (site.start, site.end),
            format!("This comment cannot be read as a directive: {reason}. It suppresses nothing."),
            String::from(
                "Correct the comment, or, if it is ordinary text, do not begin its first word with \"monochromatic-lint-\".",
            ),
        );
    }

    /// An identifier outside the registry,
    /// with a hint when a full identifier ends with the written text.
    pub fn unknown_rule(&self, token: &RuleToken) -> Diagnostic {
        let mut help: String = String::from(
            "Use a full identifier as listed by \"monochromatic-lint --rules\", such as \"rust/tests-in-sidecars\", or remove the entry.",
        );
        let suffix: String = format!("/{}", token.text);
        // Several namespaces can end alike, as `rust/max-lines` and `shell/max-lines` do, so name every candidate.
        let mut candidates: Vec<String> = Vec::<String>::new();
        for rule in RULES {
            if rule.id.ends_with(suffix.as_str()) {
                candidates.push(format!("\"{}\"", rule.id));
            }
        }
        if let Some(last) = candidates.pop() {
            let named: String = if candidates.is_empty() {
                last
            } else {
                format!("{} or {last}", candidates.join(", "))
            };
            help = format!("Did you mean {named}? Directives need the full identifier.");
        }
        return self.report(
            UNKNOWN_RULE,
            Severity::Error,
            (token.start, token.end),
            format!(
                "\"{}\" is not a rule identifier this linter ships, so the directive entry for it is ignored.",
                token.text
            ),
            help,
        );
    }

    /// An entry aimed at a rule that refuses directives.
    pub fn non_suppressible(&self, token: &RuleToken) -> Diagnostic {
        return self.report(
            NON_SUPPRESSIBLE,
            Severity::Error,
            (token.start, token.end),
            format!(
                "\"{}\" cannot be suppressed by a directive, so the entry for it is ignored and the rule's findings stay.",
                token.text
            ),
            format!(
                "Fix what \"{}\" reports, or remove it from the directive. The rule declares itself unsuppressible, and no configuration option changes that.",
                token.text
            ),
        );
    }

    /// A directive or one of its entries that changed nothing.
    pub fn unused(
        &self,
        severity: Severity,
        kind: DirectiveKind,
        range: (usize, usize),
        rule: Option<&str>,
    ) -> Diagnostic {
        let keyword: &str = kind.keyword();
        let cause: &str = if kind == DirectiveKind::Enable {
            "it enables nothing that was disabled at that point"
        } else {
            "it suppressed no finding"
        };
        let message: String = match rule {
            Some(id) => format!("Unused {keyword} directive for {id}: {cause}."),
            None => format!("Unused {keyword} directive: {cause}."),
        };
        return self.report(
            UNUSED,
            severity,
            range,
            message,
            String::from(
                "Remove the directive or correct its rule list. Unused directives are reported because --report-unused-disable-directives or --report-unused-disable-directives-severity is set.",
            ),
        );
    }
}

/// Message and position controls stay outside release artifacts.
#[cfg(test)]
#[path = "directive_findings_tests.rs"]
mod tests;
