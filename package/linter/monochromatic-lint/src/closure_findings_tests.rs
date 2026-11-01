//! What:
//!  Controls of the mapping from compiler reports to findings.
//! Why:
//!  Severity,
//! exemptions,
//! positions in bytes and the drop of unselected files are the behavior the owner asked for,
//!  and each is checked with a case that must move when the code is wrong.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('maps severity per file', () => expect(findingsFrom(reports, dir, targets).map(f => f.severity)).toEqual([...]));
//! ```

use super::{RULE, findings_from, key_of, span_of, targets_of};
use crate::{
    closure_report::Report,
    diagnostic::{Diagnostic, Severity},
    run_plan::FilePlan,
    run_test_support::{CONFIG, plan, write},
    test_fs::Fixture,
};
use std::path::PathBuf;

/// A configuration with the rule at warning level,
/// error for `ide`,
/// and off for tests.
const CONFIGURATION: &str = r#"[
  { "files": ["**/*.rs"], "rules": { "rust/avoidable-closure": { "severity": "warn", "library": "lib" } } },
  { "files": ["ide/**/*.rs"], "rules": { "rust/avoidable-closure": { "severity": "error" } } },
  { "files": ["**/tests/**/*.rs", "**/*_tests.rs"], "rules": { "rust/avoidable-closure": { "severity": "off" } } }
]"#;

/// A report at a byte range of a file.
fn report(file: &str, start: usize, end: usize) -> Report {
    return Report {
        file: String::from(file),
        byte_start: start,
        byte_end: end,
        line: 1,
        column: 1,
        message: String::from("this closure is avoidable"),
        help: Some(String::from("use a function")),
    };
}

/// Plan the named files of a fixture.
fn plans(root: &std::path::Path, files: &[&str]) -> Vec<FilePlan> {
    let mut found: Vec<FilePlan> = Vec::<FilePlan>::new();
    for file in files {
        found.push(plan(root, file));
    }
    return found;
}

/// Only files whose own rules select the check become targets,
/// each with its severity and display name.
#[test]
fn targets_carry_severity_and_skip_files_with_the_rule_off() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    write(&fixture.path, "pkg/src/lib.rs", "fn a() {}\n");
    write(&fixture.path, "ide/src/lib.rs", "fn a() {}\n");
    write(&fixture.path, "pkg/tests/it.rs", "fn a() {}\n");
    write(&fixture.path, "pkg/src/lib_tests.rs", "fn a() {}\n");
    write(&fixture.path, "pkg/README.md", "# x\n");
    let all: Vec<FilePlan> = plans(
        &fixture.path,
        &[
            "pkg/src/lib.rs",
            "ide/src/lib.rs",
            "pkg/tests/it.rs",
            "pkg/src/lib_tests.rs",
        ],
    );
    let targets = targets_of(all.as_slice());
    assert_eq!(targets.len(), 2);
    let own = &targets[&key_of(&fixture.path.join("pkg/src/lib.rs"))];
    assert_eq!(
        (own.display.as_str(), own.severity),
        ("pkg/src/lib.rs", Severity::Warn)
    );
    let ide = &targets[&key_of(&fixture.path.join("ide/src/lib.rs"))];
    assert_eq!(ide.severity, Severity::Error);
}

/// A Markdown plan never becomes a target.
#[test]
fn a_markdown_file_is_not_a_target() {
    let fixture: Fixture = Fixture::new();
    write(
        &fixture.path,
        CONFIG,
        r#"[{ "files": ["**/*.md"], "rules": { "markdown/single-h1": { "severity": "error" } } }]"#,
    );
    write(&fixture.path, "README.md", "# x\n");
    let all: Vec<FilePlan> = plans(&fixture.path, &["README.md"]);
    assert!(targets_of(all.as_slice()).is_empty());
}

/// Reports in files with the rule off or outside the run are dropped;
/// the rest carry their file's severity and help.
#[test]
fn findings_use_the_severity_of_their_file_and_drop_the_rest() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    write(&fixture.path, "pkg/src/lib.rs", "fn a() { run(|x| x); }\n");
    write(&fixture.path, "pkg/src/other.rs", "fn a() {}\n");
    write(&fixture.path, "pkg/tests/it.rs", "fn a() { run(|x| x); }\n");
    let selected: Vec<FilePlan> = plans(&fixture.path, &["pkg/src/lib.rs", "pkg/tests/it.rs"]);
    let targets = targets_of(selected.as_slice());
    let reports: Vec<Report> = vec![
        report("src/lib.rs", 11, 18),
        report("tests/it.rs", 11, 18),
        report("src/other.rs", 0, 1),
        report("src/missing.rs", 0, 1),
    ];
    let found: Vec<Diagnostic> =
        findings_from(reports.as_slice(), &fixture.path.join("pkg"), &targets);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].code, RULE);
    assert_eq!(found[0].severity, Severity::Warn);
    assert_eq!(found[0].filename, "pkg/src/lib.rs");
    assert_eq!(found[0].help.as_deref(), Some("use a function"));
    assert!(!found[0].processing_failure);
}

/// The IDE's error severity reaches its findings.
#[test]
fn an_error_severity_file_produces_error_findings() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    write(&fixture.path, "ide/src/lib.rs", "fn a() { run(|x| x); }\n");
    let selected: Vec<FilePlan> = plans(&fixture.path, &["ide/src/lib.rs"]);
    let found = findings_from(
        &[report("src/lib.rs", 11, 18)],
        &fixture.path.join("ide"),
        &targets_of(selected.as_slice()),
    );
    assert_eq!(found[0].severity, Severity::Error);
}

/// A path the compiler prints with a `..` or through the package directory still matches the file.
#[test]
fn a_path_with_parent_components_matches() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    write(&fixture.path, "pkg/src/lib.rs", "fn a() { run(|x| x); }\n");
    let selected: Vec<FilePlan> = plans(&fixture.path, &["pkg/src/lib.rs"]);
    let targets = targets_of(selected.as_slice());
    let found = findings_from(
        &[report("../pkg/src/../src/lib.rs", 11, 18)],
        &fixture.path.join("pkg"),
        &targets,
    );
    assert_eq!(found.len(), 1);
    let absolute: String = fixture
        .path
        .join("pkg/src/lib.rs")
        .to_string_lossy()
        .into_owned();
    assert_eq!(
        findings_from(
            &[report(absolute.as_str(), 11, 18)],
            &fixture.path.join("pkg"),
            &targets
        )
        .len(),
        1
    );
}

/// Findings come out ordered by file and offset whatever order cargo produced them in.
#[test]
fn findings_are_sorted_by_file_and_offset() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, CONFIGURATION);
    write(&fixture.path, "pkg/src/b.rs", "0123456789\n0123456789\n");
    write(&fixture.path, "pkg/src/a.rs", "0123456789\n0123456789\n");
    let selected: Vec<FilePlan> = plans(&fixture.path, &["pkg/src/b.rs", "pkg/src/a.rs"]);
    let reports: Vec<Report> = vec![
        report("src/b.rs", 15, 16),
        report("src/a.rs", 14, 15),
        report("src/b.rs", 2, 3),
        report("src/a.rs", 1, 2),
    ];
    let found = findings_from(
        reports.as_slice(),
        &fixture.path.join("pkg"),
        &targets_of(selected.as_slice()),
    );
    let order: Vec<(String, usize)> = found
        .iter()
        .map(|finding| return (finding.filename.clone(), finding.labels[0].span.offset))
        .collect::<Vec<(String, usize)>>();
    assert_eq!(
        order,
        vec![
            (String::from("pkg/src/a.rs"), 1),
            (String::from("pkg/src/a.rs"), 14),
            (String::from("pkg/src/b.rs"), 2),
            (String::from("pkg/src/b.rs"), 15),
        ]
    );
}

/// Columns count bytes,
/// not characters:
/// the compiler's character column is not used when the text is available.
#[test]
fn columns_count_bytes_after_multibyte_text() {
    let source: &str = "// \u{e9}\u{e9}\u{e9}\nfn a() { run(\u{e9}|x| x); }\n";
    let offset: usize = source.find('|').expect("the closure starts here");
    let mut at = report("src/lib.rs", offset, offset + 7);
    at.line = 99;
    at.column = 99;
    let span = span_of(&at, Some(source));
    assert_eq!(span.line, 2);
    assert_eq!(
        span.column,
        source
            .lines()
            .nth(1)
            .expect("line two")
            .find('|')
            .expect("bar")
            + 1
    );
    assert_eq!((span.offset, span.length), (offset, 7));
}

/// Without the file text the compiler's own position stands in,
/// and a start past the end falls back too.
#[test]
fn without_the_text_the_compiler_position_is_used() {
    let mut at = report("src/lib.rs", 40, 45);
    at.line = 7;
    at.column = 3;
    let span = span_of(&at, None);
    assert_eq!(
        (span.offset, span.length, span.line, span.column),
        (40, 5, 7, 3)
    );
    let beyond = span_of(&at, Some("short\n"));
    assert_eq!((beyond.line, beyond.column), (7, 3));
}

/// A reversed range has length zero instead of wrapping.
#[test]
fn a_reversed_range_has_length_zero() {
    assert_eq!(span_of(&report("a.rs", 9, 3), None).length, 0);
}

/// A directory name with a space and the same file reached twice produce one key.
#[test]
fn keys_resolve_spaces_and_parent_components() {
    let fixture: Fixture = Fixture::new();
    let file: PathBuf = write(&fixture.path, "my pkg/src/lib.rs", "");
    assert_eq!(
        key_of(&fixture.path.join("my pkg/src/../src/lib.rs")),
        key_of(&file)
    );
    assert_eq!(
        key_of(&fixture.path.join("absent/../absent.rs")),
        fixture.path.join("absent.rs")
    );
}
