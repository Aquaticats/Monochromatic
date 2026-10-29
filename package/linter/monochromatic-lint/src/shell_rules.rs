//! What:
//!  The `shell/` rules over one extracted run body.
//! Why:
//!  The line budget and the eight construct rules share one scan and one source,
//!  and report in the body's own coordinates;
//!  the processor maps them to the workflow file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function checkShell(filename: string, source: string, dialect: ShellDialect, settings: ShellRuleSettings): Diagnostic[];
//! ```

/// Import the finding model.
use crate::diagnostic::{Diagnostic, Span};
/// Import the dialect and construct names.
use crate::shell_dialect::{Construct, ShellDialect};
/// Import the typed settings the checks read.
use crate::shell_rule_settings::{ShellBudget, ShellRuleSettings};
/// Import the scan.
use crate::shell_scan::{Hit, scan};

/// A zero-based line start and end of one code line,
///  excluding its newline.
struct CodeLine {
    /// First byte of the line.
    start: usize,
    /// One past the last content byte.
    end: usize,
}

/// What:
///  Whether a trimmed line is only a comment in the dialect.
/// Why:
///  A comment line is not a line of logic,
///  so it does not count toward the budget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isCommentLine(trimmed: string, dialect: ShellDialect): boolean;
/// ```
fn is_comment_line(trimmed: &str, dialect: ShellDialect) -> bool {
    match dialect {
        ShellDialect::Cmd => {
            let lowered: String = trimmed.to_ascii_lowercase();
            return trimmed.starts_with("::")
                || lowered == "rem"
                || lowered.starts_with("rem ")
                || lowered.starts_with("rem\t");
        }
        _ => return trimmed.starts_with('#'),
    }
}

/// Collect the non-blank,
///  non-comment physical lines of a run body.
fn code_lines(source: &str, dialect: ShellDialect) -> Vec<CodeLine> {
    let mut lines: Vec<CodeLine> = Vec::new();
    let mut start: usize = 0;
    // `split_inclusive` keeps each newline so byte offsets stay exact.
    for raw in source.split_inclusive('\n') {
        let content: &str = raw.trim_end_matches(['\n', '\r']);
        let trimmed: &str = content.trim();
        if !trimmed.is_empty() && !is_comment_line(trimmed, dialect) {
            lines.push(CodeLine {
                start,
                end: start + content.len(),
            });
        }
        start += raw.len();
    }
    return lines;
}

/// Build a finding in the run body's coordinates;
///  the processor recomputes the line and column in the workflow file.
fn finding(
    code: &str,
    budget: &ShellBudget,
    message: String,
    filename: &str,
    range: (usize, usize),
) -> Diagnostic {
    return Diagnostic::new(
        code,
        budget.severity,
        message,
        String::from(filename),
        Span {
            offset: range.0,
            length: range.1 - range.0,
            line: 1,
            column: 1,
        },
    );
}

/// Report a body whose code lines exceed the budget,
///  once,
///  at the first line over the limit.
fn check_lines(
    filename: &str,
    source: &str,
    dialect: ShellDialect,
    budget: &ShellBudget,
) -> Option<Diagnostic> {
    let lines: Vec<CodeLine> = code_lines(source, dialect);
    if lines.len() <= budget.max {
        return None;
    }
    let first_over: &CodeLine = &lines[budget.max];
    let message: String = format!(
        "This shell step has {} code lines, and the limit is {}. Keep a step to one command; move longer logic into a TypeScript program run by a `shell: node {{0}}` step or a repository task.",
        lines.len(),
        budget.max
    );
    return Some(finding(
        "shell/max-lines",
        budget,
        message,
        filename,
        (first_over.start, first_over.end),
    ));
}

/// Report the occurrences of one construct beyond its budget.
fn check_construct(
    filename: &str,
    construct: Construct,
    budget: &ShellBudget,
    hits: &[Hit],
) -> Vec<Diagnostic> {
    let mut found: Vec<&Hit> = Vec::new();
    for hit in hits {
        if hit.construct == construct {
            found.push(hit);
        }
    }
    let mut reports: Vec<Diagnostic> = Vec::new();
    if found.len() <= budget.max {
        return reports;
    }
    for hit in &found[budget.max..] {
        let message: String = format!(
            "`{}` composes commands in a shell step ({} in this step, limit {}). Run one command per step and move the logic into a TypeScript program run by a `shell: node {{0}}` step or a repository task.",
            construct.token(),
            found.len(),
            budget.max
        );
        reports.push(finding(
            construct.rule_id(),
            budget,
            message,
            filename,
            (hit.start, hit.end),
        ));
    }
    return reports;
}

/// What:
///  Run every selected `shell/` rule over one run body.
/// Why:
///  The scan runs once and only when a construct rule is selected,
///  because the line budget needs no scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkShell(filename: string, source: string, dialect: ShellDialect, settings: ShellRuleSettings): Diagnostic[];
/// ```
pub fn check_shell(
    filename: &str,
    source: &str,
    dialect: ShellDialect,
    settings: &ShellRuleSettings,
) -> Vec<Diagnostic> {
    let mut findings: Vec<Diagnostic> = Vec::new();
    if let Some(budget) = &settings.max_lines
        && let Some(report) = check_lines(filename, source, dialect, budget)
    {
        findings.push(report);
    }
    if settings.constructs.is_empty() {
        return findings;
    }
    let hits: Vec<Hit> = scan(source, dialect);
    for (construct, budget) in &settings.constructs {
        findings.extend(check_construct(filename, *construct, budget, &hits));
    }
    return findings;
}

/// Shell rule controls stay outside release artifacts.
#[cfg(test)]
#[path = "shell_rules_tests.rs"]
mod tests;
