//! What:
//!  Controls for reading an in-process linter run:
//!  the usable run,
//! every refusal by cause,
//! and the explanation's bound.
//! Why:
//!  The policy trusts only a status 0 run whose findings are warnings of a selected rule with a position;
//!  each other shape must end the policy with its own reason instead of passing an unchecked file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(() => interpretEmbeddedRun({ input, rules, outcome: failed(1) })).toThrow('never produces');
//! ```

use super::{
    EXPLANATION_LIMIT, INCOMPLETE_STATUS, LintRun, MAX_FINDINGS_BYTES, MAX_FIXED_BYTES, RemainingFinding, explanation,
    interpret_embedded_run,
};
use crate::config_schema::{MarkdownRule, markdown_rule_name};
use monochromatic_lint::diagnostic::{Diagnostic, Severity, Span};
use monochromatic_lint::run_embedded::{EmbeddedFailure, EmbeddedRun};

/// The rules the policy's configuration selects.
const RULES: &[MarkdownRule] = &[MarkdownRule::LfsImageUrl];

/// The finding code the linter gives the selected rule.
fn selected_code() -> String {
    return format!("markdown/{}", markdown_rule_name(MarkdownRule::LfsImageUrl));
}

/// A finding of `code` at `severity` with its first label at `line` and `column`.
fn finding(code: &str, severity: Severity, message: &str, line: usize, column: usize) -> Diagnostic {
    return Diagnostic::new(
        code,
        severity,
        String::from(message),
        String::from("a.md"),
        Span {
            offset: 0,
            length: 0,
            line,
            column,
        },
    );
}

/// A usable run with this fixed text and these findings.
fn run(fixed: &str, findings: Vec<Diagnostic>) -> Result<EmbeddedRun, EmbeddedFailure> {
    return Ok(EmbeddedRun {
        fixed: String::from(fixed),
        findings,
    });
}

/// A run that ended with `status` and this explanation.
fn failed(status: u8, explanation_text: &str) -> Result<EmbeddedRun, EmbeddedFailure> {
    return Err(EmbeddedFailure {
        exit_status: status,
        explanation: String::from(explanation_text),
    });
}

/// A clean run returns the fixed source and no findings;
///  an empty input may stay empty.
#[test]
fn a_clean_run_returns_the_fixed_source() {
    assert_eq!(
        interpret_embedded_run(b"a", RULES, run("b", Vec::new())),
        Ok(LintRun {
            fixed: b"b".to_vec(),
            remaining: Vec::new(),
        })
    );
    assert_eq!(
        interpret_embedded_run(b"", RULES, run("", Vec::new())),
        Ok(LintRun {
            fixed: Vec::new(),
            remaining: Vec::new(),
        })
    );
}

/// Warnings of the selected rule come back in order with their first label's position.
#[test]
fn warnings_of_a_selected_rule_are_remaining_findings() {
    let code: String = selected_code();
    let findings: Vec<Diagnostic> = vec![
        finding(code.as_str(), Severity::Warn, "first", 3, 7),
        finding(code.as_str(), Severity::Warn, "second \"quoted\"", 10, 1),
    ];
    let expected: Vec<RemainingFinding> = findings
        .iter()
        .map(|found| {
            return RemainingFinding {
                rule: MarkdownRule::LfsImageUrl,
                line: u64::try_from(found.labels[0].span.line).expect("line"),
                column: u64::try_from(found.labels[0].span.column).expect("column"),
                message: found.message.clone(),
            };
        })
        .collect();
    assert_eq!(
        interpret_embedded_run(b"x", RULES, run("x", findings)),
        Ok(LintRun {
            fixed: b"x".to_vec(),
            remaining: expected,
        })
    );
}

/// A run that could not finish ends with its own reason and the linter's explanation;
///  every other status says the configuration never produces it.
#[test]
fn other_statuses_are_refused_by_cause() {
    let reason: &str = "Cannot inspect /r/.lfsconfig.";
    assert_eq!(
        interpret_embedded_run(b"x", RULES, failed(INCOMPLETE_STATUS, reason)),
        Err(format!("monochromatic-lint could not finish (exit status {INCOMPLETE_STATUS}): {reason}"))
    );
    let other: u8 = INCOMPLETE_STATUS + 1;
    assert_eq!(
        interpret_embedded_run(b"x", RULES, failed(other, reason)),
        Err(format!(
            "monochromatic-lint exited with status {other}, which its one-rule configuration never produces: {reason}"
        ))
    );
}

/// A successful run whose fixed source is empty for a non-empty input is refused.
#[test]
fn an_empty_fixed_source_for_a_file_that_is_not_empty_is_refused() {
    assert_eq!(
        interpret_embedded_run(b"x", RULES, run("", Vec::new())),
        Err(String::from(
            "monochromatic-lint produced an empty fixed source for a file that is not empty"
        ))
    );
}

/// A finding that is not a warning of a selected rule with a position cannot be used,
///  and the refusal names which finding.
#[test]
fn findings_the_policy_cannot_use_are_refused_with_their_number() {
    let code: String = selected_code();
    let usable: Diagnostic = finding(code.as_str(), Severity::Warn, "ok", 1, 1);
    let mut unlabeled: Diagnostic = finding(code.as_str(), Severity::Warn, "no label", 1, 1);
    unlabeled.labels.clear();
    let cases: Vec<(Diagnostic, &str)> = vec![
        (finding("markdown/other-rule", Severity::Warn, "m", 1, 1), "is not a selected rule"),
        (finding("rust/max-lines", Severity::Warn, "m", 1, 1), "is not a selected rule"),
        (finding(code.as_str(), Severity::Error, "m", 1, 1), "severity is not the configured"),
        (unlabeled, "no label with a position"),
    ];
    for (bad, needle) in cases {
        let refusal: String = interpret_embedded_run(b"x", RULES, run("x", vec![usable.clone(), bad]))
            .expect_err("refused");
        assert!(refusal.contains("finding 2 "), "{refusal}");
        assert!(refusal.contains(needle), "{refusal}");
    }
}

/// With no rule selected no finding is accepted.
#[test]
fn no_selected_rule_accepts_no_finding() {
    let code: String = selected_code();
    let refusal: String = interpret_embedded_run(
        b"x",
        &[],
        run("x", vec![finding(code.as_str(), Severity::Warn, "m", 1, 1)]),
    )
    .expect_err("refused");
    assert!(refusal.contains("is not a selected rule"), "{refusal}");
}

/// The explanation is quoted whole when short and cut with an ellipsis at the bound.
#[test]
fn the_explanation_stays_short() {
    assert_eq!(explanation("short"), "short");
    let exact: String = "a".repeat(EXPLANATION_LIMIT);
    assert_eq!(explanation(exact.as_str()), exact);
    let long: String = "b".repeat(EXPLANATION_LIMIT + 1);
    assert_eq!(explanation(long.as_str()), format!("{}...", "b".repeat(EXPLANATION_LIMIT)));
}

/// A fixed source at the bound is accepted and one byte over it is refused,
/// as the child's output cap was.
#[test]
fn the_fixed_source_is_bounded() {
    let at_bound: String = "a".repeat(MAX_FIXED_BYTES);
    assert!(interpret_embedded_run(b"a", RULES, run(at_bound.as_str(), Vec::new())).is_ok());
    let over: String = "a".repeat(MAX_FIXED_BYTES + 1);
    let refusal: String =
        interpret_embedded_run(b"a", RULES, run(over.as_str(), Vec::new())).expect_err("refused");
    assert!(refusal.contains(MAX_FIXED_BYTES.to_string().as_str()), "{refusal}");
}

/// Findings whose text passes the bound are refused,
/// as the child's standard error cap refused them.
#[test]
fn the_findings_are_bounded() {
    let code: String = selected_code();
    let huge: String = "m".repeat(MAX_FINDINGS_BYTES + 1);
    let refusal: String = interpret_embedded_run(
        b"x",
        RULES,
        run("x", vec![finding(code.as_str(), Severity::Warn, huge.as_str(), 1, 1)]),
    )
    .expect_err("refused");
    assert!(refusal.contains(MAX_FINDINGS_BYTES.to_string().as_str()), "{refusal}");
}
