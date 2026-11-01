//! What:
//!  Controls for reading one linter run.
//! Why:
//!  Only exit status 0 with text output and well-formed warnings of a selected rule is
//!      usable;
//!  every other status,
//!  record or stream shape must be refused with its reason.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(() => interpretLinterRun({ input, rules, finished: { exit: 2 } })).toThrow('could not finish');
//! ```

/// The reader under test.
use super::{EXPLANATION_LIMIT, LintRun, RemainingFinding, explanation, interpret_linter_run};
use crate::bounded_child::{ChildExit, FinishedChild};
use crate::config_schema::MarkdownRule;

/// The selected rules of the shipped configuration.
const RULES: &[MarkdownRule] = &[MarkdownRule::LfsImageUrl];

/// A record exactly as the linter prints a warning of the LFS rule.
fn record(message: &str, line: u64, column: u64) -> String {
    return format!(
        "{{\"message\":{message:?},\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\",\"causes\":[],\
         \"filename\":\"pkg/README.md\",\"labels\":[{{\"span\":{{\"offset\":0,\"length\":3,\"line\":{line},\"column\":{column}}}}}],\
         \"related\":[]}}\n"
    );
}

/// A finished run with exit code `code`.
fn finished(code: i32, stdout: &[u8], stderr: &[u8]) -> FinishedChild {
    return FinishedChild {
        exit: ChildExit::Code(code),
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    };
}

/// A clean run returns the fixed source and no findings;
///  an unchanged one returns the input.
#[test]
fn a_clean_run_returns_the_fixed_source() {
    assert_eq!(
        interpret_linter_run(b"a", RULES, &finished(0, b"b", b"")),
        Ok(LintRun {
            fixed: b"b".to_vec(),
            remaining: Vec::new(),
        })
    );
    assert_eq!(
        interpret_linter_run(b"", RULES, &finished(0, b"", b"")),
        Ok(LintRun {
            fixed: Vec::new(),
            remaining: Vec::new(),
        })
    );
}

/// Warnings of the selected rule come back in order with their first label's position.
#[test]
fn warnings_of_a_selected_rule_are_remaining_findings() {
    let stderr: String = format!(
        "{}{}",
        record("Object URL names a.png, which no longer exists.", 3, 7),
        record("second \"quoted\"", 10, 1)
    );
    assert_eq!(
        interpret_linter_run(b"x", RULES, &finished(0, b"x", stderr.as_bytes())),
        Ok(LintRun {
            fixed: b"x".to_vec(),
            remaining: vec![
                RemainingFinding {
                    rule: MarkdownRule::LfsImageUrl,
                    line: 3,
                    column: 7,
                    message: String::from("Object URL names a.png, which no longer exists."),
                },
                RemainingFinding {
                    rule: MarkdownRule::LfsImageUrl,
                    line: 10,
                    column: 1,
                    message: String::from("second \"quoted\""),
                },
            ],
        })
    );
}

/// Every other exit ends the run with its own reason,
///  and quotes the linter's explanation.
#[test]
fn other_exits_are_refused_by_cause() {
    let setup: &[u8] = b"monochromatic-lint: Standard input is not valid UTF-8 (first invalid byte at offset 1).\n";
    assert_eq!(
        interpret_linter_run(b"x", RULES, &finished(2, b"", setup)),
        Err(String::from(
            "monochromatic-lint could not finish (exit status 2): Standard input is not valid UTF-8 (first invalid byte at offset 1)."
        ))
    );
    let processing: String = "{\"message\":\"Cannot inspect /r/.lfsconfig.\",\"code\":\"core/processing-failure\",\"severity\":\"error\",\"causes\":[],\"filename\":\"pkg/README.md\",\"labels\":[],\"related\":[]}\n".to_owned();
    assert_eq!(
        interpret_linter_run(b"x", RULES, &finished(2, b"x", processing.as_bytes())),
        Err(String::from(
            "monochromatic-lint could not finish (exit status 2): Cannot inspect /r/.lfsconfig."
        ))
    );
    for code in [1, 3, 101, -1] {
        assert_eq!(
            interpret_linter_run(b"x", RULES, &finished(code, b"x", b"")),
            Err(format!(
                "monochromatic-lint exited with status {code}, which its one-rule configuration never produces: it printed no explanation"
            ))
        );
    }
    let signaled: FinishedChild = FinishedChild {
        exit: ChildExit::Signal(9),
        stdout: b"x".to_vec(),
        stderr: Vec::new(),
    };
    assert_eq!(
        interpret_linter_run(b"x", RULES, &signaled),
        Err(String::from("monochromatic-lint was ended by signal 9"))
    );
    let unknown: FinishedChild = FinishedChild {
        exit: ChildExit::Unknown,
        ..signaled
    };
    assert_eq!(
        interpret_linter_run(b"x", RULES, &unknown),
        Err(String::from(
            "monochromatic-lint ended without an exit status"
        ))
    );
}

/// A successful run whose fixed source is not text,
///  or is empty for a non-empty input,
///  is refused.
#[test]
fn a_fixed_source_that_cannot_be_right_is_refused() {
    assert_eq!(
        interpret_linter_run(b"x", RULES, &finished(0, b"\xff", b"")),
        Err(String::from(
            "monochromatic-lint printed a fixed source that is not UTF-8"
        ))
    );
    assert_eq!(
        interpret_linter_run(b"x", RULES, &finished(0, b"", b"")),
        Err(String::from(
            "monochromatic-lint printed an empty fixed source for a file that is not empty"
        ))
    );
}

/// The reason a standard error of a successful run is refused,
///  or the findings.
fn records(stderr: &str) -> Result<Vec<RemainingFinding>, String> {
    return match interpret_linter_run(b"x", RULES, &finished(0, b"x", stderr.as_bytes())) {
        Ok(run) => Ok(run.remaining),
        Err(reason) => Err(reason),
    };
}

/// The refusal of a record on line 2 of standard error.
fn on_line_two(problem: &str) -> String {
    return format!(
        "monochromatic-lint wrote a record cli-git cannot use on line 2 of its standard error: {problem}"
    );
}

/// Each malformed record is refused,
///  naming its line and what is wrong with it.
#[test]
fn malformed_records_are_refused_with_their_line() {
    let good: String = record("m", 1, 1);
    for (bad, problem) in [
        ("not json\n", "it is not JSON"),
        ("[1]\n", "it is not a JSON object"),
        ("\n", "it is not JSON"),
        ("{\"message\":\"m\"}\n", "it has no \"code\""),
        ("{\"code\":1}\n", "its \"code\" is not a string"),
        (
            "{\"code\":\"markdown/semantic-line-breaks\"}\n",
            "its code \"markdown/semantic-line-breaks\" is not a selected rule",
        ),
        (
            "{\"code\":\"lfs-image-url\"}\n",
            "its code \"lfs-image-url\" is not a selected rule",
        ),
        (
            "{\"code\":\"core/processing-failure\"}\n",
            "its code \"core/processing-failure\" is not a selected rule",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"severity\":\"error\"}\n",
            "its severity is not the configured \"warn\"",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"code\":\"markdown/lfs-image-url\"}\n",
            "it names \"code\" twice",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\"}\n",
            "it has no \"message\"",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\",\"message\":\"m\"}\n",
            "it has no \"labels\"",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\",\"message\":\"m\",\"labels\":[]}\n",
            "its \"labels\" is not a list with a label",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\",\"message\":\"m\",\"labels\":{}}\n",
            "its \"labels\" is not a list with a label",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\",\"message\":\"m\",\"labels\":[1]}\n",
            "its first label is not an object",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\",\"message\":\"m\",\"labels\":[{}]}\n",
            "its first label has no \"span\" object",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\",\"message\":\"m\",\"labels\":[{\"span\":{\"column\":1}}]}\n",
            "its span has no \"line\"",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\",\"message\":\"m\",\"labels\":[{\"span\":{\"line\":\"1\",\"column\":1}}]}\n",
            "its span's \"line\" is not a number",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\",\"message\":\"m\",\"labels\":[{\"span\":{\"line\":-1,\"column\":1}}]}\n",
            "its span's \"line\" is not a whole number",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\",\"message\":\"m\",\"labels\":[{\"span\":{\"line\":1,\"column\":1.5}}]}\n",
            "its span's \"column\" is not a whole number",
        ),
        (
            "{\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\",\"message\":\"\\ud800\",\"labels\":[{\"span\":{\"line\":1,\"column\":1}}]}\n",
            "its \"message\" is not valid text",
        ),
    ] {
        assert_eq!(
            records(format!("{good}{bad}").as_str()),
            Err(on_line_two(problem)),
            "{bad:?}"
        );
    }
    assert_eq!(
        records("\n"),
        Err(String::from(
            "monochromatic-lint wrote a record cli-git cannot use on line 1 of its standard error: it is not JSON"
        ))
    );
    assert_eq!(
        records(good.trim_end()),
        Err(String::from(
            "monochromatic-lint ended its standard error without a line feed after the last record"
        ))
    );
    assert_eq!(
        interpret_linter_run(b"x", RULES, &finished(0, b"x", b"\xff\n")),
        Err(String::from(
            "monochromatic-lint wrote standard error that is not UTF-8 text"
        ))
    );
    assert_eq!(
        records(good.as_str()),
        Ok(vec![RemainingFinding {
            rule: MarkdownRule::LfsImageUrl,
            line: 1,
            column: 1,
            message: String::from("m"),
        }])
    );
}

/// Without any selected rule,
///  every record names an unselected rule.
#[test]
fn no_selected_rule_accepts_no_record() {
    assert_eq!(
        interpret_linter_run(b"x", &[], &finished(0, b"x", record("m", 1, 1).as_bytes())),
        Err(String::from(
            "monochromatic-lint wrote a record cli-git cannot use on line 1 of its standard error: its code \"markdown/lfs-image-url\" is not a selected rule"
        ))
    );
}

/// The explanation joins prefixed lines and record messages,
///  ignores the rest,
///  and stops
/// at its limit.
#[test]
fn the_explanation_quotes_the_linter_and_stays_short() {
    assert_eq!(
        explanation(b"monochromatic-lint: one\nnoise\n{\"message\":\"two\"}\n{\"code\":1}\nmonochromatic-lint: three"),
        "one; two; three"
    );
    assert_eq!(explanation(b""), "it printed no explanation");
    assert_eq!(explanation(b"noise only\n"), "it printed no explanation");
    let long: String = format!("monochromatic-lint: {}", "é".repeat(EXPLANATION_LIMIT + 5));
    let quoted: String = explanation(long.as_bytes());
    assert_eq!(quoted.chars().count(), EXPLANATION_LIMIT + 3);
    assert!(quoted.ends_with("é..."), "{quoted}");
    let exact: String = format!("monochromatic-lint: {}", "a".repeat(EXPLANATION_LIMIT));
    assert_eq!(explanation(exact.as_bytes()), "a".repeat(EXPLANATION_LIMIT));
}
