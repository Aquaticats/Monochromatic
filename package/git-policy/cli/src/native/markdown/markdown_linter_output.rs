//! What:
//!  Read what one in-process `monochromatic-lint` fix run produced:
//!  the fixed source and the findings it could not fix,
//!  or the reason the run cannot be trusted.
//! Why:
//!  The linter's own contract (`package/linter/monochromatic-lint/src/run_embedded.rs`) is the executable's
//!  `--stdin --fix` run with its one-rule configuration: status 0 with the fixed source and warning findings,
//!  because the policy's configuration gives the rule severity `warn`;
//!  1 only for error findings or a warning limit,
//!  neither of which that configuration can produce;
//!  and 2 for a run that could not finish.
//!  Anything else,
//!  and any finding that is not a warning of a selected rule with a position,
//!  means the run cannot be trusted,
//!  so the policy fails instead of passing an unchecked file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const run = interpretEmbeddedRun({ input, rules, outcome }); // throws string
//! ```

/// Import the selectable rules and their configuration spelling.
use crate::config_schema::{MarkdownRule, markdown_rule_name};
/// Import the linter's in-process result types and finding model.
use monochromatic_lint::{
    diagnostic::{Diagnostic, Severity},
    run_embedded::{EmbeddedFailure, EmbeddedRun},
};

/// The longest explanation quoted from the linter's own words,
///  in characters.
pub const EXPLANATION_LIMIT: usize = 400;

/// The most bytes of fixed source the policy accepts from one run:
///  the bound the former child process had on its standard output,
///  which allows a large file whose every link grows into an object URL.
pub const MAX_FIXED_BYTES: usize = 64 << 20;

/// The most bytes of findings the policy accepts from one run:
///  the bound the former child process had on its standard error.
///  A finding counts its message, code and file name.
pub const MAX_FINDINGS_BYTES: usize = 4 << 20;

/// The status the linter reports for a run that could not finish.
pub const INCOMPLETE_STATUS: u8 = 2;

/// What:
///  One finding the linter reported and could not fix.
///  `u64` holds the one-based
///       line and column the linter gives.
/// Why:
///   The policy reports it with the incumbent's words,
///  naming the rule as configured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemainingFinding = { rule: MarkdownRule; line: number; column: number; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemainingFinding {
    /// The rule that reported it.
    pub rule: MarkdownRule,
    /// One-based line of the finding's first label.
    pub line: u64,
    /// One-based column of that label,
    ///  in the linter's units.
    pub column: u64,
    /// The linter's explanation.
    pub message: String,
}

/// What:
///  A run the policy can use:
///  the source after every fix,
///  and what is left.
/// Why:
///   The fixed bytes equal the input exactly when nothing changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LintRun = { fixed: Uint8Array; remaining: RemainingFinding[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LintRun {
    /// The fixed source,
    ///  byte for byte as the linter produced it.
    pub fixed: Vec<u8>,
    /// Findings the fixes did not remove,
    ///  in the linter's order.
    pub remaining: Vec<RemainingFinding>,
}

/// What:
///  The selected rule a finding code names.
///  `&[MarkdownRule]` borrows the rules the
///       configuration selected.
/// Why:
///   The linter prefixes every Markdown rule with `markdown/`;
///  a code of a rule the
///       one-rule configuration did not select means the linter ran something else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const selectedRule = (code, rules) => rules.find(rule => code === `markdown/${ruleName(rule)}`);
/// ```
fn selected_rule(code: &str, rules: &[MarkdownRule]) -> Option<MarkdownRule> {
    for rule in rules {
        if code.strip_prefix("markdown/") == Some(markdown_rule_name(*rule)) {
            return Some(*rule);
        }
    }
    return None;
}

/// What:
///  A one-based line or column as the whole number the policy reports.
/// Why:
///  A count too large for the type is not a position, so the run is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function position(name: string, value: number): number; // throws string
/// ```
fn position(name: &str, value: usize) -> Result<u64, String> {
    match u64::try_from(value) {
        Ok(count) => return Ok(count),
        Err(_) => return Err(format!("its {name} is not a position")),
    }
}

/// What:
///  Read one finding of a successful run.
/// Why:
///   Every finding must be a warning of a selected rule with a position;
///  anything else is a contract the policy cannot rely on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function remainingFinding(finding: Diagnostic, rules: MarkdownRule[]): RemainingFinding; // throws string
/// ```
fn remaining_finding(finding: &Diagnostic, rules: &[MarkdownRule]) -> Result<RemainingFinding, String> {
    let Some(rule) = selected_rule(finding.code.as_str(), rules) else {
        return Err(format!("its code {:?} is not a selected rule", finding.code));
    };
    if finding.severity != Severity::Warn {
        return Err(String::from("its severity is not the configured \"warn\""));
    }
    let Some(label) = finding.labels.first() else {
        return Err(String::from("it has no label with a position"));
    };
    return Ok(RemainingFinding {
        rule,
        line: position("line", label.span.line)?,
        column: position("column", label.span.column)?,
        message: finding.message.clone(),
    });
}

/// What:
///  The linter's own explanation of a failed run,
///  cut at `EXPLANATION_LIMIT` characters.
/// Why:
///  The person needs the linter's reason;
///  the file name is left out,
///  so the explanation does not repeat the candidate's pathname the event already names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function explanation(text: string): string;
/// ```
pub fn explanation(text: &str) -> String {
    if text.chars().count() <= EXPLANATION_LIMIT {
        return String::from(text);
    }
    let kept: String = text.chars().take(EXPLANATION_LIMIT).collect();
    return format!("{kept}...");
}

/// The running size of the findings after adding `finding`.
fn add_finding_bytes(total: usize, finding: &Diagnostic) -> usize {
    return total
        .saturating_add(finding.message.len())
        .saturating_add(finding.code.len())
        .saturating_add(finding.filename.len());
}

/// What:
///  Interpret one finished in-process run over `input`.
///  `Result<LintRun, String>` is the
///       usable run,
///  or the reason it cannot be used,
///  for the person who ran the command.
/// Why:
///   See the module comment.
///  A successful run must produce text,
///  and must produce some for a non-empty input:
///  the linter refuses to empty a file,
///  so an empty fixed source would mean it never read the input,
///  and using it would empty the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function interpretEmbeddedRun({ input, rules, outcome }): LintRun; // throws string
/// ```
pub fn interpret_embedded_run(
    input: &[u8],
    rules: &[MarkdownRule],
    outcome: Result<EmbeddedRun, EmbeddedFailure>,
) -> Result<LintRun, String> {
    let run: EmbeddedRun = match outcome {
        Ok(usable) => usable,
        Err(failure) if failure.exit_status == INCOMPLETE_STATUS => {
            return Err(format!(
                "monochromatic-lint could not finish (exit status {INCOMPLETE_STATUS}): {}",
                explanation(failure.explanation.as_str())
            ));
        }
        Err(failure) => {
            return Err(format!(
                "monochromatic-lint exited with status {}, which its one-rule configuration never produces: {}",
                failure.exit_status,
                explanation(failure.explanation.as_str())
            ));
        }
    };
    if run.fixed.len() > MAX_FIXED_BYTES {
        return Err(format!(
            "monochromatic-lint produced a fixed source of {} bytes, over the {MAX_FIXED_BYTES} bytes monochromatic-git-policy-cli accepts",
            run.fixed.len()
        ));
    }
    let findings_bytes: usize = run.findings.iter().fold(0, add_finding_bytes);
    if findings_bytes > MAX_FINDINGS_BYTES {
        return Err(format!(
            "monochromatic-lint produced findings of {findings_bytes} bytes, over the {MAX_FINDINGS_BYTES} bytes monochromatic-git-policy-cli accepts"
        ));
    }
    if run.fixed.is_empty() && !input.is_empty() {
        return Err(String::from(
            "monochromatic-lint produced an empty fixed source for a file that is not empty",
        ));
    }
    let mut remaining: Vec<RemainingFinding> = Vec::new();
    for (index, finding) in run.findings.iter().enumerate() {
        match remaining_finding(finding, rules) {
            Ok(found) => remaining.push(found),
            Err(reason) => {
                return Err(format!(
                    "monochromatic-lint returned finding {} that monochromatic-git-policy-cli cannot use: {reason}",
                    index + 1
                ));
            }
        }
    }
    return Ok(LintRun {
        fixed: run.fixed.into_bytes(),
        remaining,
    });
}

/// Contract and explanation controls stay out of the release executable.
#[cfg(test)]
#[path = "markdown_linter_output_tests.rs"]
mod tests;
