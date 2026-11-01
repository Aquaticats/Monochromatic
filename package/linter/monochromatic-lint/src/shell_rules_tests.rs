//! What:
//!  Controls for the `shell/` rules over one run body.
//! Why:
//!  The budgets decide when a body is reported and where,
//!  and the processor maps the reported range to the workflow file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('shell rules', () => { /* line budget, occurrence budget, severity */ });
//! ```

/// Import the checker and the settings it reads.
use super::check_shell;
use crate::diagnostic::{Diagnostic, Severity};
use crate::shell_dialect::{Construct, ShellDialect};
use crate::shell_rule_settings::{ShellBudget, ShellRuleSettings};

/// A line budget only.
fn lines(max: usize) -> ShellRuleSettings {
    return ShellRuleSettings {
        max_lines: Some(ShellBudget {
            severity: Severity::Error,
            max,
        }),
        constructs: Vec::new(),
    };
}

/// One construct rule only.
fn construct(construct: Construct, max: usize, severity: Severity) -> ShellRuleSettings {
    return ShellRuleSettings {
        max_lines: None,
        constructs: vec![(construct, ShellBudget { severity, max })],
    };
}

/// The first label's byte range.
fn range(finding: &Diagnostic) -> (usize, usize) {
    let span = &finding.labels[0].span;
    return (span.offset, span.offset + span.length);
}

/// A body over the budget is reported once,
///  at the first line past the limit.
#[test]
fn lines_over_the_budget_are_reported_once() {
    let source: &str = "a\nb\nlong third\nfourth\n";
    let found: Vec<Diagnostic> = check_shell("ci.yml/0.sh", source, ShellDialect::Posix, &lines(2));
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].code, "shell/max-lines");
    assert_eq!(range(&found[0]), (4, 14));
    assert_eq!(found[0].filename, "ci.yml/0.sh");
    assert!(found[0].message.contains("4 code lines"));
}

/// Blank lines and comment lines are free,
///  and a body at the limit is clean.
#[test]
fn blank_and_comment_lines_do_not_count() {
    let source: &str = "# note\n\na\n  # indented note\nb\n\n";
    assert!(check_shell("x.sh", source, ShellDialect::Posix, &lines(2)).is_empty());
    assert_eq!(
        check_shell("x.sh", source, ShellDialect::Posix, &lines(1)).len(),
        1
    );
}

/// Continuations and heredoc content count as physical lines.
#[test]
fn continuations_and_heredocs_count() {
    let source: &str = "cmd \\\n  --flag \\\n  --other\n";
    assert_eq!(
        check_shell("x.sh", source, ShellDialect::Posix, &lines(2)).len(),
        1
    );
    let heredoc: &str = "cat <<EOF\nline\nEOF\n";
    assert_eq!(
        check_shell("x.sh", heredoc, ShellDialect::Posix, &lines(2)).len(),
        1
    );
}

/// Windows line endings do not shift the reported range.
#[test]
fn carriage_returns_stay_outside_the_range() {
    let found: Vec<Diagnostic> =
        check_shell("x.sh", "a\r\nb\r\nc\r\n", ShellDialect::Posix, &lines(2));
    assert_eq!(range(&found[0]), (6, 7));
}

/// `cmd` comments are `rem` and `::` lines.
#[test]
fn cmd_comment_lines_are_free() {
    let source: &str = "rem a\n:: b\nREM c\necho x\n";
    assert!(check_shell("x.cmd", source, ShellDialect::Cmd, &lines(1)).is_empty());
    assert_eq!(
        check_shell("x.cmd", source, ShellDialect::Posix, &lines(1)).len(),
        1
    );
}

/// With a budget of none,
///  every occurrence is reported at its own range.
#[test]
fn every_occurrence_over_the_budget_is_reported() {
    let settings: ShellRuleSettings = construct(Construct::Pipe, 0, Severity::Error);
    let found: Vec<Diagnostic> = check_shell("x.sh", "a | b | c", ShellDialect::Posix, &settings);
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].code, "shell/no-pipe");
    assert_eq!(range(&found[0]), (2, 3));
    assert_eq!(range(&found[1]), (6, 7));
    assert!(found[0].message.contains("2 in this step, limit 0"));
}

/// A positive budget excuses that many occurrences and reports the rest.
#[test]
fn a_positive_budget_reports_only_the_excess() {
    let settings: ShellRuleSettings = construct(Construct::Pipe, 1, Severity::Warn);
    let found: Vec<Diagnostic> = check_shell("x.sh", "a | b | c", ShellDialect::Posix, &settings);
    assert_eq!(found.len(), 1);
    assert_eq!(range(&found[0]), (6, 7));
    assert_eq!(found[0].severity, Severity::Warn);
    let enough: ShellRuleSettings = construct(Construct::Pipe, 2, Severity::Error);
    assert!(check_shell("x.sh", "a | b | c", ShellDialect::Posix, &enough).is_empty());
}

/// Only the selected construct is reported,
///  and the dialect table still applies.
#[test]
fn other_constructs_and_inapplicable_dialects_are_quiet() {
    let settings: ShellRuleSettings = construct(Construct::Backtick, 0, Severity::Error);
    assert_eq!(
        check_shell("x.sh", "echo `x` | y", ShellDialect::Posix, &settings).len(),
        1
    );
    assert!(check_shell("x.ps1", "echo `x`", ShellDialect::PowerShell, &settings).is_empty());
    assert!(
        check_shell(
            "x.sh",
            "echo ok",
            ShellDialect::Posix,
            &ShellRuleSettings::default()
        )
        .is_empty()
    );
}

/// Both budgets report from one call.
#[test]
fn both_budgets_report_together() {
    let settings: ShellRuleSettings = ShellRuleSettings {
        max_lines: lines(1).max_lines,
        constructs: construct(Construct::Semicolon, 0, Severity::Error).constructs,
    };
    let found: Vec<Diagnostic> = check_shell("x.sh", "a; b\nc\n", ShellDialect::Posix, &settings);
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].code, "shell/max-lines");
    assert_eq!(found[1].code, "shell/no-semicolon");
}
