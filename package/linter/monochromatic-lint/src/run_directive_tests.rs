//! What:
//!  End-to-end controls for inline directives through the production checker and command.
//! Why:
//!  The filter is unit-tested on synthetic findings;
//!  these controls run real rules,
//!  rustdoc and fence processors,
//!  the semantic engine,
//!  the fix loop,
//!  standard input and JSON output to show that one filter governs all of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('directives end to end', () => { /* real rules, processors, semantic, --fix, JSON, flags */ });
//! ```

/// Import the production checker,
///  planner,
///  command runner and fixtures.
use crate::diagnostic::{Diagnostic, Severity};
use crate::directive_options::DirectiveOptions;
use crate::run_check::HostChecker;
use crate::run_lfs::LfsRepos;
use crate::run_output::RunOutput;
use crate::run_plan::FilePlan;
use crate::run_test_support::{CONFIG, codes, plan, read, records, run, write};
use crate::rust_file_engine::RustFileEngine;
use crate::rust_semantic_test_support::{prepare_lockfile, progress};
use crate::rust_workspace::WorkspacePreparation;
use crate::test_fs::Fixture;
use std::path::Path;

/// Rules whose findings land on predictable lines.
const RULES: &str = r#"[
  { "files": ["**/*.rs"], "rules": { "rust/tests-in-sidecars": { "severity": "error" } } },
  { "files": ["**/*.md"], "rules": {
    "markdown/no-bare-urls": { "severity": "error" },
    "markdown/single-h1": { "severity": "error" },
    "markdown/fenced-code-language": { "severity": "error" }
  } },
  { "files": ["**/*.md/*.rs"], "rules": { "rust/require-rustdoc": { "severity": "error" } } }
]"#;

/// Check a source under a configuration with chosen unused-directive reporting.
fn check(
    root: &Path,
    config: &str,
    file: &str,
    source: &str,
    unused: Option<Severity>,
) -> Vec<Diagnostic> {
    write(root, CONFIG, config);
    let mut planned: FilePlan = plan(root, file);
    planned.directives = DirectiveOptions { unused };
    let lfs: LfsRepos = LfsRepos::new();
    let mut checker: HostChecker<'_> = HostChecker::new(&planned, &lfs, None);
    return checker.check_snapshot(source);
}

/// `code@line` for every finding.
fn located(found: &[Diagnostic]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::<String>::new();
    for item in found {
        lines.push(format!("{}@{}", item.code, item.labels[0].span.line));
    }
    return lines;
}

/// A real Rust rule is suppressed at the line it reports,
///  which for an inline test module is the line of the module name,
///  and a directive on any other line leaves it.
#[test]
fn real_rust_rule_findings_follow_the_reported_line() {
    let fixture: Fixture = Fixture::new();
    let bare: &str = "#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n";
    let control: Vec<Diagnostic> = check(&fixture.path, RULES, "a.rs", bare, None);
    assert_eq!(located(&control), ["rust/tests-in-sidecars@2"]);
    let covered: &str = "#[cfg(test)]\n// monochromatic-lint-disable-next-line rust/tests-in-sidecars -- generated\nmod tests {\n    #[test]\n    fn t() {}\n}\n";
    assert!(check(&fixture.path, RULES, "a.rs", covered, Some(Severity::Error)).is_empty());
    let too_early: &str = "// monochromatic-lint-disable-next-line rust/tests-in-sidecars -- generated\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n";
    let missed: Vec<Diagnostic> = check(&fixture.path, RULES, "a.rs", too_early, None);
    assert_eq!(located(&missed), ["rust/tests-in-sidecars@3"]);
}

/// Markdown findings inside rustdoc are positioned in the Rust host,
///  so a Rust comment directive covers them.
#[test]
fn rustdoc_markdown_findings_are_covered_by_the_rust_hosts_directives() {
    let fixture: Fixture = Fixture::new();
    let control: &str = "//! File.\n\n/// See https://example.com/a for details.\nfn item() {}\n";
    let uncovered: Vec<Diagnostic> = check(&fixture.path, RULES, "lib.rs", control, None);
    assert_eq!(located(&uncovered), ["markdown/no-bare-urls@3"]);
    let covered: &str = "//! File.\n\n// monochromatic-lint-disable-next-line markdown/no-bare-urls -- r\n/// See https://example.com/a for details.\nfn item() {}\n";
    assert!(
        check(
            &fixture.path,
            RULES,
            "lib.rs",
            covered,
            Some(Severity::Error)
        )
        .is_empty()
    );
    // A Markdown comment inside the rustdoc is doc text,
    //  and the Rust host reads only its own comments.
    let inside: &str = "//! File.\n\n/// <!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- r -->\n/// See https://example.com/a for details.\nfn item() {}\n";
    let still: Vec<Diagnostic> = check(
        &fixture.path,
        RULES,
        "lib.rs",
        inside,
        Some(Severity::Error),
    );
    assert_eq!(located(&still), ["markdown/no-bare-urls@4"]);
}

/// Rust inside a Markdown fence is virtual:
///  only the unsuppressible rules apply to it,
///  a directive written inside the fence is code,
///  and a Markdown directive aimed at those rules is refused.
#[test]
fn virtual_rust_in_markdown_fences_is_not_suppressible() {
    let fixture: Fixture = Fixture::new();
    let fence: &str = "# Title\n\n```rust\n//! Docs.\n// monochromatic-lint-disable-next-line rust/require-rustdoc -- r\nfn helper() {}\n```\n";
    let found: Vec<Diagnostic> = check(
        &fixture.path,
        RULES,
        "doc/a.md",
        fence,
        Some(Severity::Error),
    );
    assert_eq!(located(&found), ["rust/require-rustdoc@6"]);
    let outside: &str = "# Title\n\n<!-- monochromatic-lint-disable-next-line rust/require-rustdoc -- r -->\n```rust\nlet value = 1;\n```\n";
    let refused: Vec<Diagnostic> = check(&fixture.path, RULES, "doc/a.md", outside, None);
    assert_eq!(
        located(&refused),
        ["rust/require-rustdoc@5", "directive/non-suppressible@3"]
    );
    let bare: &str =
        "# Title\n\n<!-- monochromatic-lint-disable -- r -->\n```rust\nlet value = 1;\n```\n";
    let kept: Vec<Diagnostic> = check(&fixture.path, RULES, "doc/a.md", bare, None);
    assert_eq!(located(&kept), ["rust/require-rustdoc@5"]);
}

/// Markdown directives cover real Markdown rules,
///  and the same text in a code fence does not.
#[test]
fn real_markdown_rule_findings_are_covered() {
    let fixture: Fixture = Fixture::new();
    let source: &str = "# Title\n\n<!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- legacy -->\nSee http://a.example now.\nSee http://b.example now.\n";
    let found: Vec<Diagnostic> = check(&fixture.path, RULES, "a.md", source, None);
    assert_eq!(located(&found), ["markdown/no-bare-urls@5"]);
    let fenced: &str = "# Title\n\n```text\n<!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- r -->\n```\nSee http://a.example now.\n";
    let kept: Vec<Diagnostic> = check(&fixture.path, RULES, "a.md", fenced, None);
    assert_eq!(located(&kept), ["markdown/no-bare-urls@6"]);
}

/// A host whose rules never ran for lack of a semantic engine reports a processing failure,
///  which no directive suppresses and which stops unused-directive judgments.
#[test]
fn processing_failures_survive_directives() {
    let fixture: Fixture = Fixture::new();
    let config: &str = r#"[{ "files": ["**/*.rs"], "rules": { "rust/require-explicit-types": { "severity": "error" } } }]"#;
    let source: &str = "// monochromatic-lint-disable -- r\n// monochromatic-lint-disable-next-line rust/require-explicit-types -- r\nfn main() {}\n";
    let found: Vec<Diagnostic> = check(
        &fixture.path,
        config,
        "src/a.rs",
        source,
        Some(Severity::Error),
    );
    assert_eq!(located(&found), ["core/processing-failure@1"]);
}

/// The semantic rule's findings pass through the same filter as every other rule's,
///  and an unrelated line directive does not cover them.
/// This test loads a real Cargo workspace,
/// like the other semantic controls.
#[test]
fn semantic_findings_follow_the_same_directive_rules() {
    let fixture: Fixture = Fixture::new();
    let directory = fixture.path.join("src");
    std::fs::create_dir(&directory).expect("fixture source directory");
    std::fs::write(
        fixture.path.join("Cargo.toml"),
        "[package]\nname = \"directive-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\nmembers = [\".\"]\n",
    )
    .expect("fixture manifest");
    std::fs::write(directory.join("main.rs"), "fn main() {}\n").expect("physical source");
    prepare_lockfile(&fixture.path);
    let config: &str = r#"[{ "files": ["**/*.rs"], "rules": { "rust/require-explicit-types": { "severity": "error" } } }]"#;
    write(&fixture.path, CONFIG, config);
    let mut planned: FilePlan = plan(&fixture.path, "src/main.rs");
    planned.directives = DirectiveOptions {
        unused: Some(Severity::Warn),
    };
    let lfs: LfsRepos = LfsRepos::new();
    let mut engine: RustFileEngine =
        RustFileEngine::new(WorkspacePreparation::SourceOnly, progress);
    let mut checker: HostChecker<'_> = HostChecker::new(&planned, &lfs, Some(&mut engine));
    let control: Vec<Diagnostic> =
        checker.check_snapshot("fn main() {\n    let value = 1_u16;\n}\n");
    assert_eq!(located(&control), ["rust/require-explicit-types@2"]);
    let covered: &str = "fn main() {\n    // monochromatic-lint-disable-next-line rust/require-explicit-types -- r\n    let value = 1_u16;\n    // monochromatic-lint-disable-next-line rust/require-explicit-types -- r\n}\n";
    let found: Vec<Diagnostic> = checker.check_snapshot(covered);
    // The first directive covers the binding;
    //  the second covers a line with no finding and is reported as unused.
    assert_eq!(located(&found), ["directive/unused@4"]);
}

/// `--fix` never fixes a finding that a directive covers,
///  fixes the others,
///  and leaves the directive in place.
#[test]
fn fix_skips_covered_findings() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, RULES);
    let source: &str = "# Title\n\n<!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- legacy -->\nSee http://a.example now.\nSee http://b.example now.\n";
    write(&fixture.path, "a.md", source);
    let output: RunOutput = run(&fixture.path, &["--fix"], "");
    assert_eq!((output.exit_code, output.stdout.as_str()), (0, ""));
    assert_eq!(
        read(&fixture.path, "a.md"),
        "# Title\n\n<!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- legacy -->\nSee http://a.example now.\nSee <http://b.example> now.\n"
    );
    // The control: without the directive both are fixed.
    let plain: &str = "# Title\n\nSee http://a.example now.\nSee http://b.example now.\n";
    write(&fixture.path, "b.md", plain);
    run(&fixture.path, &["--fix", "b.md"], "");
    assert_eq!(
        read(&fixture.path, "b.md"),
        "# Title\n\nSee <http://a.example> now.\nSee <http://b.example> now.\n"
    );
}

/// The same holds for rustdoc fixes in a Rust host and for standard-input fixing.
#[test]
fn fix_skips_covered_findings_in_rustdoc_and_stdin() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, RULES);
    let source: &str = "//! File.\n\n// monochromatic-lint-disable-next-line markdown/no-bare-urls -- r\n/// See https://example.com/a for details.\n///\n/// See https://example.com/b for details.\nfn item() {}\n";
    write(&fixture.path, "lib.rs", source);
    run(&fixture.path, &["--fix", "lib.rs"], "");
    assert_eq!(
        read(&fixture.path, "lib.rs"),
        "//! File.\n\n// monochromatic-lint-disable-next-line markdown/no-bare-urls -- r\n/// See https://example.com/a for details.\n///\n/// See <https://example.com/b> for details.\nfn item() {}\n"
    );
    let stdin_source: &str = "# Title\n\n<!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- r -->\nSee http://a.example now.\nSee http://b.example now.\n";
    let piped: RunOutput = run(
        &fixture.path,
        &["--stdin", "--stdin-filename", "n.md", "--fix"],
        stdin_source,
    );
    assert_eq!(piped.exit_code, 0);
    assert_eq!(
        piped.stdout,
        "# Title\n\n<!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- r -->\nSee http://a.example now.\nSee <http://b.example> now.\n"
    );
}

/// Directive findings are ordinary JSONL records with the established shape,
///  and unused directives are reported only when asked.
#[test]
fn json_output_carries_directive_findings() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, RULES);
    write(
        &fixture.path,
        "a.md",
        "# Title\n\n<!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- r -->\nplain\n<!-- monochromatic-lint-disable -->\n",
    );
    let quiet: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(quiet.exit_code, 1);
    assert_eq!(
        codes(quiet.stdout.as_str()),
        ["directive/missing-justification"]
    );
    let loud: RunOutput = run(&fixture.path, &["--report-unused-disable-directives"], "");
    assert_eq!(loud.exit_code, 1);
    let found = records(loud.stdout.as_str());
    assert_eq!(found.len(), 2);
    assert_eq!(found[0]["code"], "directive/unused");
    assert_eq!(found[0]["severity"], "warn");
    assert_eq!(found[0]["filename"], "a.md");
    assert_eq!(found[0]["labels"][0]["span"]["line"], 3);
    assert_eq!(found[0]["labels"][0]["span"]["column"], 43);
    assert_eq!(found[0]["causes"], serde_json::json!([]));
    assert_eq!(found[0]["related"], serde_json::json!([]));
    assert!(
        found[0]["help"]
            .as_str()
            .expect("help")
            .contains("Remove the directive")
    );
    assert_eq!(found[1]["code"], "directive/missing-justification");
    assert_eq!(found[1]["severity"], "error");
}

/// The warning flag reports at warning severity and passes unless warnings are limited,
///  the severity option reports errors or nothing,
///  and the two never combine.
#[test]
fn unused_directive_flags_set_severity_and_exit_status() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, RULES);
    write(
        &fixture.path,
        "a.md",
        "# Title\n\n<!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- r -->\nplain\n",
    );
    let off: RunOutput = run(&fixture.path, &[], "");
    assert_eq!((off.exit_code, off.stdout.as_str()), (0, ""));
    let warn: RunOutput = run(&fixture.path, &["--report-unused-disable-directives"], "");
    assert_eq!(warn.exit_code, 0);
    assert_eq!(codes(warn.stdout.as_str()), ["directive/unused"]);
    let limited: RunOutput = run(
        &fixture.path,
        &["--report-unused-disable-directives", "--max-warnings", "0"],
        "",
    );
    assert_eq!(limited.exit_code, 1);
    let error: RunOutput = run(
        &fixture.path,
        &["--report-unused-disable-directives-severity=error"],
        "",
    );
    assert_eq!(error.exit_code, 1);
    assert_eq!(records(error.stdout.as_str())[0]["severity"], "error");
    let allowed: RunOutput = run(
        &fixture.path,
        &["--report-unused-disable-directives-severity", "allow"],
        "",
    );
    assert_eq!((allowed.exit_code, allowed.stdout.as_str()), (0, ""));
}

/// No configuration key,
///  override or later block can make a rule suppressible:
///  the configuration rejects such a key,
///  and a directive aimed at the rule is refused whatever the configuration says.
#[test]
fn configuration_cannot_make_an_unsuppressible_rule_suppressible() {
    let fixture: Fixture = Fixture::new();
    let attempts: [&str; 3] = [
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/max-lines": { "severity": "error", "max": 0, "suppressible": true } } }]"#,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/require-rustdoc": { "severity": "error", "allowDirectives": true } } }]"#,
        r#"[{ "files": ["**/*.rs"], "suppressible": ["rust/max-lines"], "rules": { "rust/max-lines": { "severity": "error", "max": 0 } } }]"#,
    ];
    for config in attempts {
        write(&fixture.path, CONFIG, config);
        write(&fixture.path, "a.rs", "//! Docs.\nfn a() {}\n");
        let output: RunOutput = run(&fixture.path, &[], "");
        assert_eq!(output.exit_code, 2, "{config}");
        assert_eq!(output.stdout, "", "{config}");
    }
    let later: &str = r#"[
      { "files": ["**/*.rs"], "rules": { "rust/max-lines": { "severity": "error", "max": 0 } } },
      { "files": ["**/a.rs"], "rules": { "rust/max-lines": { "severity": "error", "max": 0 } } }
    ]"#;
    write(&fixture.path, CONFIG, later);
    write(
        &fixture.path,
        "a.rs",
        "// monochromatic-lint-disable-next-line rust/max-lines -- r\nfn a() {}\n",
    );
    let refused: RunOutput = run(&fixture.path, &[], "");
    assert_eq!(
        codes(refused.stdout.as_str()),
        ["directive/non-suppressible", "rust/max-lines"]
    );
    assert_eq!(refused.exit_code, 1);
}
