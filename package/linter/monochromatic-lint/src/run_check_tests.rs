//! What:
//!  Controls for host rules plus always-on processors over one snapshot.
//! Why:
//!  Virtual files must be matched by their own logical paths,
//!  reported at host positions,
//! and fixed through the host,
//!  and every inability to check must surface as a processing finding.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('HostChecker', () => { /* host rules, virtual paths, host positions, failures */ });
//! ```

/// Import the checker and the production fix applier.
use super::HostChecker;
use crate::diagnostic::{Diagnostic, Severity};
use crate::edits::{Fix, apply_fixes};
use crate::fix_loop::SourceChecker;
use crate::run_lfs::LfsRepos;
use crate::run_plan::FilePlan;
use crate::run_test_support::{CONFIG, plan, write};
use crate::test_fs::Fixture;
use std::path::Path;

/// Check one source under the configuration written at the fixture root.
fn check(root: &Path, config: &str, file: &str, source: &str) -> Vec<Diagnostic> {
    write(root, CONFIG, config);
    let planned: FilePlan = plan(root, file);
    let lfs: LfsRepos = LfsRepos::new();
    let mut checker: HostChecker<'_> = HostChecker::new(&planned, &lfs, None);
    return checker.check_snapshot(source);
}

/// The codes of findings with their host line numbers,
///  in report order.
fn located(findings: &[Diagnostic]) -> Vec<(String, usize)> {
    let mut pairs: Vec<(String, usize)> = Vec::<(String, usize)>::new();
    for finding in findings {
        pairs.push((finding.code.clone(), finding.labels[0].span.line));
    }
    return pairs;
}

/// The first finding with a code,
///  failing the test when none has it.
fn with_code<'found>(findings: &'found [Diagnostic], code: &str) -> &'found Diagnostic {
    for finding in findings {
        if finding.code == code {
            return finding;
        }
    }
    panic!("no finding has code {code}");
}

/// Apply every advertised fix once.
fn fixed(source: &str, findings: &[Diagnostic]) -> String {
    let mut fixes: Vec<Fix> = Vec::<Fix>::new();
    for finding in findings {
        if let Some(group) = &finding.fix {
            fixes.push(group.clone());
        }
    }
    return apply_fixes(source, fixes.as_slice())
        .expect("fixes apply")
        .source;
}

/// A Rust fence in Markdown is checked under the rules its virtual path resolves to,
///  at host positions.
#[test]
fn markdown_fences_are_checked_as_virtual_rust_at_host_positions() {
    let fixture: Fixture = Fixture::new();
    let config: &str = r#"[
      { "files": ["**/*.md"], "rules": { "markdown/single-h1": { "severity": "error" } } },
      { "files": ["**/*.md/*.rs"], "rules": { "rust/require-rustdoc": { "severity": "warn" }, "rust/max-lines": { "severity": "error", "max": 0 } } }
    ]"#;
    let source: &str =
        "# Title\n\n```rust\n//! Docs.\nlet value = 1; fn helper() {}\n```\n\n# Second\n";
    let findings: Vec<Diagnostic> = check(&fixture.path, config, "doc/a.md", source);
    for finding in &findings {
        assert_eq!(finding.filename, "doc/a.md");
    }
    let mut pairs: Vec<(String, usize)> = located(findings.as_slice());
    pairs.sort();
    assert_eq!(
        pairs,
        [
            (String::from("markdown/single-h1"), 8),
            (String::from("rust/max-lines"), 5),
            (String::from("rust/require-rustdoc"), 5),
        ]
    );
    let item: &Diagnostic = with_code(findings.as_slice(), "rust/require-rustdoc");
    // The undocumented function starts after `let value = 1; ` on host line 5.
    assert_eq!(item.labels[0].span.column, 16);
    assert_eq!(
        &source[item.labels[0].span.offset..item.labels[0].span.offset + 14],
        "fn helper() {}"
    );
    assert_eq!(item.severity, Severity::Warn);
    // The budget anchors on the first authored line beyond it, at the start of that line.
    let budget: &Diagnostic = with_code(findings.as_slice(), "rust/max-lines");
    assert_eq!(budget.labels[0].span.column, 1);
    assert_eq!(budget.severity, Severity::Error);
}

/// A virtual path no block selects,
///  or one a block ignores,
///  is not checked;
///  the host's own rules still run.
#[test]
fn unconfigured_and_ignored_virtual_paths_are_not_checked() {
    let fixture: Fixture = Fixture::new();
    let source: &str = "# Title\n\n```rust\nlet value = 1;\n```\n\n# Second\n";
    let host_only: &str =
        r#"[{ "files": ["**/*.md"], "rules": { "markdown/single-h1": { "severity": "error" } } }]"#;
    assert_eq!(
        located(check(&fixture.path, host_only, "doc/a.md", source).as_slice()),
        [(String::from("markdown/single-h1"), 7)]
    );
    let ignoring: &str = r#"[
      { "files": ["**/*.rs"], "ignores": ["**/*.md/**"], "rules": { "rust/require-rustdoc": { "severity": "error" } } },
      { "files": ["**/*.md"], "rules": { "markdown/single-h1": { "severity": "error" } } }
    ]"#;
    assert_eq!(
        located(check(&fixture.path, ignoring, "doc/a.md", source).as_slice()),
        [(String::from("markdown/single-h1"), 7)]
    );
    // `**/*.rs` alone reaches the fence, and the unconfigured host contributes nothing of its own.
    let virtual_only: &str = r#"[{ "files": ["**/*.rs"], "rules": { "rust/require-rustdoc": { "severity": "error" } } }]"#;
    assert_eq!(
        located(check(&fixture.path, virtual_only, "doc/a.md", source).as_slice()),
        [(String::from("rust/require-rustdoc"), 4)]
    );
}

/// Rustdoc is checked as virtual Markdown,
///  doc tests two levels deep,
///  and fixes land in the host comment.
#[test]
fn rustdoc_and_doc_tests_are_checked_and_fixed_through_the_host() {
    let fixture: Fixture = Fixture::new();
    let config: &str = r#"[
      { "files": ["**/*.rs"], "ignores": ["**/*.md/**"], "rules": { "rust/require-rustdoc": { "severity": "error" } } },
      { "files": ["**/*.rs/*.md"], "rules": { "markdown/fenced-code-language": { "severity": "error" }, "markdown/no-trailing-punctuation": { "severity": "error" } } },
      { "files": ["**/*.rs/*.md/*.rs"], "rules": { "rust/require-rustdoc": { "severity": "warn" } } }
    ]"#;
    let source: &str = "//! File docs.\n\n/// # Heading.\n///\n/// ```\n/// //! Example.\n/// let value = 1; fn helper() {}\n/// ```\nfn item() {}\n";
    let findings: Vec<Diagnostic> = check(&fixture.path, config, "src/a.rs", source);
    let mut pairs: Vec<(String, usize)> = located(findings.as_slice());
    pairs.sort();
    assert_eq!(
        pairs,
        [
            (String::from("markdown/fenced-code-language"), 5),
            (String::from("markdown/no-trailing-punctuation"), 3),
            (String::from("rust/require-rustdoc"), 7),
        ]
    );
    for finding in &findings {
        assert_eq!(finding.filename, "src/a.rs");
    }
    // Inside rustdoc the unlabeled fence becomes `rust`, and both fixes keep the comment prefix.
    assert_eq!(
        fixed(source, findings.as_slice()),
        "//! File docs.\n\n/// # Heading\n///\n/// ```rust\n/// //! Example.\n/// let value = 1; fn helper() {}\n/// ```\nfn item() {}\n"
    );
}

/// A host that does not parse is one processing finding;
///  extraction is not attempted on it.
#[test]
fn an_mdx_error_is_one_processing_finding() {
    let fixture: Fixture = Fixture::new();
    let config: &str = r#"[
      { "files": ["**/*.mdx"], "rules": { "markdown/single-h1": { "severity": "error" } } },
      { "files": ["**/*.rs"], "rules": { "rust/require-rustdoc": { "severity": "error" } } }
    ]"#;
    let source: &str = "# Title\n\n<A>\ntext\n</B>\n";
    let findings: Vec<Diagnostic> = check(&fixture.path, config, "doc/a.mdx", source);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "core/processing-failure");
    assert!(findings[0].processing_failure);
    assert_eq!(findings[0].filename, "doc/a.mdx");
    assert!(
        findings[0].message.starts_with("MDX parsing failed:"),
        "{}",
        findings[0].message
    );
    // The same text is ordinary Markdown in a `.md` file.
    let markdown: &str =
        r#"[{ "files": ["**/*.md"], "rules": { "markdown/single-h1": { "severity": "error" } } }]"#;
    assert!(check(&fixture.path, markdown, "doc/a.md", source).is_empty());
    // With no host rules, the same MDX failure still surfaces once, through extraction.
    let virtual_only: &str = r#"[{ "files": ["**/*.rs"], "rules": { "rust/require-rustdoc": { "severity": "error" } } }]"#;
    let extracted: Vec<Diagnostic> = check(&fixture.path, virtual_only, "doc/a.mdx", source);
    assert_eq!(extracted.len(), 1);
    assert!(extracted[0].processing_failure);
    assert_eq!(extracted[0].filename, "doc/a.mdx");
}

/// Unavailable semantic coverage is a processing finding on host and virtual Rust;
///  syntax rules still report.
#[test]
fn unavailable_semantic_coverage_is_reported_and_syntax_rules_still_run() {
    let fixture: Fixture = Fixture::new();
    let config: &str = r#"[{ "files": ["**/*.rs"], "rules": { "rust/require-explicit-types": { "severity": "error" }, "rust/max-lines": { "severity": "error", "max": 0 } } }]"#;
    // No semantic engine is supplied to this checker, as on a worker thread.
    let host: Vec<Diagnostic> = check(&fixture.path, config, "src/a.rs", "fn main() {}\n");
    let mut codes: Vec<&str> = Vec::<&str>::new();
    for finding in &host {
        codes.push(finding.code.as_str());
    }
    codes.sort();
    assert_eq!(codes, ["core/processing-failure", "rust/max-lines"]);
    let failure: &Diagnostic = with_code(host.as_slice(), "core/processing-failure");
    assert!(failure.processing_failure);
    assert!(
        failure.message.contains("rust/require-explicit-types"),
        "{}",
        failure.message
    );
    // A fence has no Cargo context at all: the processor reports the coverage boundary itself.
    let fence: Vec<Diagnostic> = check(
        &fixture.path,
        config,
        "doc/a.md",
        "```rust\n//! Docs.\nlet value: u32 = 1;\n```\n",
    );
    assert_eq!(fence.len(), 1);
    assert!(fence[0].processing_failure);
    assert_eq!(fence[0].filename, "doc/a.md");
}

/// The fix loop's interface returns the same findings as the snapshot check.
#[test]
fn the_fix_loop_interface_returns_snapshot_findings() {
    let fixture: Fixture = Fixture::new();
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "files": ["**/*.md"], "rules": { "markdown/no-trailing-punctuation": { "severity": "error" } } }]"#,
    );
    let planned: FilePlan = plan(&fixture.path, "a.md");
    let lfs: LfsRepos = LfsRepos::new();
    let mut checker: HostChecker<'_> = HostChecker::new(&planned, &lfs, None);
    let direct: Vec<Diagnostic> = checker.check_snapshot("# Title.\n");
    let through_trait: Vec<Diagnostic> = checker.check("# Title.\n").expect("infallible");
    assert_eq!(direct.len(), 1);
    assert_eq!(direct, through_trait);
    assert!(checker.notes.is_empty());
}
