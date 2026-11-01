//! What:
//!  Controls for the per-file pipeline:
//!  read,
//!  check or fix,
//!  refuse,
//!  write.
//! Why:
//!  What lands on disk and what is reported must agree in every outcome,
//!  including refusals.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe(processFile.name, () => { /* lint, fix and write, unchanged, refused, unreadable */ });
//! ```

/// Import the pipeline and fixture helpers.
use super::{FileOutcome, SourceOutcome, process_file, process_file_with, process_source};
use crate::run_lfs::LfsRepos;
use crate::run_plan::FilePlan;
use crate::run_test_support::{ALL_RULES, CONFIG, plan, read, write};
use crate::run_write::WriteError;
use crate::test_fs::Fixture;
use std::path::Path;

/// The codes of an outcome's findings,
///  in report order.
fn codes(findings: &[crate::diagnostic::Diagnostic]) -> Vec<&str> {
    let mut found: Vec<&str> = Vec::<&str>::new();
    for finding in findings {
        found.push(finding.code.as_str());
    }
    return found;
}

/// Process one file below a root that already holds a configuration.
fn process(root: &Path, file: &str, fix: bool) -> FileOutcome {
    let planned: FilePlan = plan(root, file);
    let lfs: LfsRepos = LfsRepos::new();
    return process_file(&planned, fix, &lfs, None);
}

/// Lint mode reports findings in position order and never touches the file.
#[test]
fn lint_mode_reports_in_position_order_and_writes_nothing() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    let source: &str = "# Title.\n\n```\ncode\n```\n\n# Second\n";
    write(&fixture.path, "a.md", source);
    let outcome: FileOutcome = process(&fixture.path, "a.md", false);
    assert!(!outcome.written);
    assert_eq!(read(&fixture.path, "a.md"), source);
    // Rules ran in registry order (single-h1 before fenced-code-language); the report is by position.
    assert_eq!(
        codes(outcome.findings.as_slice()),
        [
            "markdown/no-trailing-punctuation",
            "markdown/fenced-code-language",
            "markdown/single-h1"
        ]
    );
    let mut previous: usize = 0;
    for finding in &outcome.findings {
        assert!(finding.labels[0].span.offset >= previous);
        previous = finding.labels[0].span.offset;
        assert_eq!(finding.filename, "a.md");
    }
}

/// Fix mode writes the fixed source once and reports only what remains.
#[test]
fn fix_mode_writes_fixed_source_and_reports_the_remainder() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    write(
        &fixture.path,
        "a.md",
        "# Title.\n\n```\ncode\n```\n\n# Second\n",
    );
    let outcome: FileOutcome = process(&fixture.path, "a.md", true);
    assert!(outcome.written);
    assert_eq!(
        read(&fixture.path, "a.md"),
        "# Title\n\n```text\ncode\n```\n\n# Second\n"
    );
    assert_eq!(codes(outcome.findings.as_slice()), ["markdown/single-h1"]);
    assert_eq!(outcome.findings[0].labels[0].span.line, 7);
    assert_eq!(outcome.notes.len(), 1);
    assert!(
        outcome.notes[0].contains("fix loop"),
        "{}",
        outcome.notes[0]
    );
    // A second run finds nothing to fix and leaves the file alone.
    let again: FileOutcome = process(&fixture.path, "a.md", true);
    assert!(!again.written);
    assert_eq!(codes(again.findings.as_slice()), ["markdown/single-h1"]);
}

/// A clean file is clean in both modes and is never rewritten.
#[test]
fn clean_files_stay_untouched() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    write(&fixture.path, "a.md", "# Title\n\nOne sentence.\n");
    for fix in [false, true] {
        let outcome: FileOutcome = process(&fixture.path, "a.md", fix);
        assert!(outcome.findings.is_empty(), "{:?}", outcome.findings);
        assert!(!outcome.written);
    }
    assert_eq!(read(&fixture.path, "a.md"), "# Title\n\nOne sentence.\n");
}

/// A fix that would empty a non-empty file is refused:
///  the file is unchanged and the refusal is reported.
#[test]
fn a_fix_that_would_empty_the_file_is_refused() {
    let fixture: Fixture = Fixture::new();
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "files": ["**/*.md"], "rules": { "markdown/link-image-reference-definitions": { "severity": "warn" } } }]"#,
    );
    let source: &str = "[unused]: https://example.com\n";
    write(&fixture.path, "a.md", source);
    let outcome: FileOutcome = process(&fixture.path, "a.md", true);
    assert!(!outcome.written);
    assert_eq!(read(&fixture.path, "a.md"), source);
    assert_eq!(
        codes(outcome.findings.as_slice()),
        [
            "core/fix-refused",
            "markdown/link-image-reference-definitions"
        ]
    );
    let refusal = &outcome.findings[0];
    assert!(refusal.processing_failure);
    assert!(
        refusal.message.contains("empty output"),
        "{}",
        refusal.message
    );
    // Lint mode on the same file reports only the rule finding.
    let lint: FileOutcome = process(&fixture.path, "a.md", false);
    assert_eq!(
        codes(lint.findings.as_slice()),
        ["markdown/link-image-reference-definitions"]
    );
}

/// A processing failure anywhere in the file stops fixing:
///  no edit is published,
///  and both the failure and the refusal are reported.
#[test]
fn incomplete_processing_blocks_every_fix_in_the_file() {
    let fixture: Fixture = Fixture::new();
    write(
        &fixture.path,
        CONFIG,
        r#"[
          { "files": ["**/*.md"], "rules": { "markdown/no-trailing-punctuation": { "severity": "error" } } },
          { "files": ["**/*.md/*.rs"], "rules": { "rust/require-explicit-types": { "severity": "error" } } }
        ]"#,
    );
    let source: &str = "# Title.\n\n```rust\n//! Docs.\nlet value: u32 = 1;\n```\n";
    write(&fixture.path, "a.md", source);
    let outcome: FileOutcome = process(&fixture.path, "a.md", true);
    assert!(!outcome.written);
    assert_eq!(read(&fixture.path, "a.md"), source);
    let mut sorted: Vec<&str> = codes(outcome.findings.as_slice());
    sorted.sort();
    assert_eq!(
        sorted,
        [
            "core/fix-refused",
            "core/processing-failure",
            "markdown/no-trailing-punctuation"
        ]
    );
}

/// Unreadable and non-UTF-8 files are processing findings at the start of the file,
///  in both modes.
#[test]
fn unreadable_and_non_utf8_files_are_processing_findings() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    let missing: FileOutcome = process(&fixture.path, "missing.md", true);
    assert_eq!(
        codes(missing.findings.as_slice()),
        ["core/processing-failure"]
    );
    assert!(
        missing.findings[0].message.starts_with("Cannot read "),
        "{}",
        missing.findings[0].message
    );
    assert!(missing.findings[0].processing_failure);
    assert_eq!(missing.findings[0].labels[0].span.line, 1);
    assert!(!missing.written);
    std::fs::write(fixture.path.join("binary.md"), [b'#', b' ', 0xff, b'\n']).expect("bytes");
    let binary: FileOutcome = process(&fixture.path, "binary.md", true);
    assert_eq!(
        codes(binary.findings.as_slice()),
        ["core/processing-failure"]
    );
    assert!(
        binary.findings[0]
            .message
            .contains("not valid UTF-8 (first invalid byte at offset 2)"),
        "{}",
        binary.findings[0].message
    );
    assert_eq!(
        std::fs::read(fixture.path.join("binary.md")).expect("bytes"),
        [b'#', b' ', 0xff, b'\n']
    );
}

/// The in-memory step returns fixed source only in fix mode,
///  and unchanged source when nothing applies.
#[test]
fn in_memory_processing_returns_fixed_source_only_when_fixing() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    let planned: FilePlan = plan(&fixture.path, "virtual/never-written.md");
    let lfs: LfsRepos = LfsRepos::new();
    let lint: SourceOutcome = process_source(&planned, "# Title.\n", false, &lfs, None);
    assert_eq!(lint.fixed, None);
    assert_eq!(
        codes(lint.findings.as_slice()),
        ["markdown/no-trailing-punctuation"]
    );
    let fixed: SourceOutcome = process_source(&planned, "# Title.\n", true, &lfs, None);
    assert_eq!(fixed.fixed.as_deref(), Some("# Title\n"));
    assert!(fixed.findings.is_empty());
    let clean: SourceOutcome = process_source(&planned, "# Title\n", true, &lfs, None);
    assert_eq!(clean.fixed.as_deref(), Some("# Title\n"));
    assert!(!fixture.path.join("virtual").exists());
}

/// A directory named like a source file cannot be read and is a processing finding,
///  not a crash.
#[test]
fn a_directory_named_like_a_source_file_is_a_processing_finding() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    std::fs::create_dir(fixture.path.join("dir.md")).expect("directory");
    let outcome: FileOutcome = process(&fixture.path, "dir.md", true);
    assert_eq!(
        codes(outcome.findings.as_slice()),
        ["core/processing-failure"]
    );
    assert!(!outcome.written);
    assert!(fixture.path.join("dir.md").is_dir());
}

/// A writer that always refuses,
///  standing in for a full disk or a read-only directory.
fn refusing_writer(path: &Path, _contents: &[u8]) -> Result<(), WriteError> {
    return Err(WriteError {
        message: format!(
            "Cannot replace {}: refused by the test writer.",
            path.display()
        ),
    });
}

/// A failed write reports the original source's findings plus the write failure,
///  and changes nothing.
#[test]
fn a_failed_write_reports_the_original_findings() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    let source: &str = "# Title.\n\n# Second\n";
    write(&fixture.path, "a.md", source);
    let planned: FilePlan = plan(&fixture.path, "a.md");
    let lfs: LfsRepos = LfsRepos::new();
    let outcome: FileOutcome = process_file_with(&planned, true, &lfs, None, refusing_writer);
    assert!(!outcome.written);
    assert_eq!(read(&fixture.path, "a.md"), source);
    // The punctuation finding is back, at its original position, because the fix never reached disk.
    assert_eq!(
        codes(outcome.findings.as_slice()),
        [
            "markdown/no-trailing-punctuation",
            "markdown/single-h1",
            "core/processing-failure"
        ]
    );
    let failure = &outcome.findings[2];
    assert!(failure.processing_failure);
    assert!(
        failure.message.contains("refused by the test writer"),
        "{}",
        failure.message
    );
    assert!(
        failure
            .message
            .ends_with("The original file bytes have not been changed."),
        "{}",
        failure.message
    );
    // A writer that is never needed is never called: lint mode and already-fixed sources skip it.
    let lint: FileOutcome = process_file_with(&planned, false, &lfs, None, refusing_writer);
    assert_eq!(
        codes(lint.findings.as_slice()),
        ["markdown/no-trailing-punctuation", "markdown/single-h1"]
    );
    write(&fixture.path, "a.md", "# Title\n");
    let stable: FileOutcome = process_file_with(&planned, true, &lfs, None, refusing_writer);
    assert!(stable.findings.is_empty());
    assert!(!stable.written);
}
