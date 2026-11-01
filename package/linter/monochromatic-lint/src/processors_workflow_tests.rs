//! What:
//!  Controls for the virtual shell files extracted from workflows.
//! Why:
//!  The name selects rules by glob,
//!  the dialect picks the scanner,
//!  and the mapping decides where a finding lands in the workflow.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('workflow extraction', () => { /* names, dialects, mapping, refusals */ });
//! ```

/// Import the real extraction seam and the finding model.
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::processors::{ProcessorError, ProcessorLanguage, VirtualSource, extract};

/// Extract the virtual files of one workflow.
fn steps(text: &str) -> Result<Vec<VirtualSource>, ProcessorError> {
    return extract(
        String::from(".github/workflows/ci.yml"),
        String::from(text),
        ProcessorLanguage::Yaml,
    );
}

/// The names of the extracted files.
fn names(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for input in steps(text).expect("extraction") {
        found.push(String::from(input.filename()));
    }
    return found;
}

/// A finding at a byte range of the virtual file.
fn finding(offset: usize, length: usize) -> Diagnostic {
    return Diagnostic::new(
        "shell/no-pipe",
        Severity::Error,
        String::from("x"),
        String::from("virtual"),
        Span {
            offset,
            length,
            line: 1,
            column: 1,
        },
    );
}

/// Steps are numbered in order and named by dialect suffix.
#[test]
fn names_and_dialects() {
    let text: &str = "jobs:\n  a:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: x\n      - run: one\n      - run: two\n        shell: pwsh\n      - run: three\n        shell: cmd\n      - run: four\n        shell: fish\n      - run: five\n        shell: node {0}\n      - run: six\n        shell: env zsh {0}\n";
    assert_eq!(
        names(text),
        [
            ".github/workflows/ci.yml/0.sh",
            ".github/workflows/ci.yml/1.ps1",
            ".github/workflows/ci.yml/2.cmd",
            ".github/workflows/ci.yml/3.fish",
            ".github/workflows/ci.yml/4.sh",
        ]
    );
}

/// With no `shell:`,
///  the job's default,
///  the workflow's default and a Windows runner decide.
#[test]
fn default_shells() {
    let windows: &str = "jobs:\n  a:\n    runs-on: windows-latest\n    steps:\n      - run: x\n  b:\n    runs-on: ubuntu-latest\n    steps:\n      - run: y\n";
    assert_eq!(
        names(windows),
        [
            ".github/workflows/ci.yml/0.ps1",
            ".github/workflows/ci.yml/1.sh"
        ]
    );
    let workflow_default: &str = "defaults:\n  run:\n    shell: pwsh\njobs:\n  a:\n    runs-on: ubuntu-latest\n    steps:\n      - run: x\n";
    assert_eq!(names(workflow_default), [".github/workflows/ci.yml/0.ps1"]);
    let job_default: &str = "defaults:\n  run:\n    shell: pwsh\njobs:\n  a:\n    defaults:\n      run:\n        shell: bash\n    steps:\n      - run: x\n        shell: python\n      - run: y\n";
    assert_eq!(names(job_default), [".github/workflows/ci.yml/0.sh"]);
}

/// A block literal loses its indentation,
///  and a finding maps to the workflow's own bytes.
#[test]
fn block_text_and_mapping() {
    let text: &str =
        "jobs:\n  a:\n    steps:\n      - run: |\n          echo a\n          echo b | c\n";
    let found: Vec<VirtualSource> = steps(text).expect("extraction");
    assert_eq!(found[0].source(), "echo a\necho b | c\n");
    assert_eq!(found[0].language(), ProcessorLanguage::Shell);
    let pipe: usize = found[0].source().find('|').expect("pipe");
    let mapped: Diagnostic = found[0]
        .project_diagnostic(finding(pipe, 1))
        .expect("projection")
        .expect("authored position");
    // The first `|` in the workflow is the block header, so the finding's pipe is the last one.
    let host: usize = text.rfind('|').expect("host pipe");
    assert_eq!(mapped.labels[0].span.offset, host);
    assert_eq!(mapped.labels[0].span.line, 6);
    assert_eq!(mapped.labels[0].span.column, 18);
    assert_eq!(mapped.filename, ".github/workflows/ci.yml");
}

/// A plain command maps at its own column.
#[test]
fn plain_text_and_mapping() {
    let text: &str = "jobs:\n  a:\n    steps:\n      - run: echo a && b # note\n";
    let found: Vec<VirtualSource> = steps(text).expect("extraction");
    assert_eq!(found[0].source(), "echo a && b");
    let mapped: Diagnostic = found[0]
        .project_diagnostic(finding(7, 2))
        .expect("projection")
        .expect("authored position");
    assert_eq!(mapped.labels[0].span.offset, text.find("&&").expect("and"));
    assert_eq!(mapped.labels[0].span.length, 2);
}

/// Windows line endings stay in the host and out of the finding's range.
#[test]
fn crlf_hosts() {
    let text: &str =
        "jobs:\r\n  a:\r\n    steps:\r\n      - run: |\r\n          one\r\n          two | x\r\n";
    let found: Vec<VirtualSource> = steps(text).expect("extraction");
    assert_eq!(found[0].source(), "one\r\ntwo | x\r\n");
    let pipe: usize = found[0].source().find('|').expect("pipe");
    let mapped: Diagnostic = found[0]
        .project_diagnostic(finding(pipe, 1))
        .expect("projection")
        .expect("authored position");
    assert_eq!(
        mapped.labels[0].span.offset,
        text.rfind('|').expect("host pipe")
    );
}

/// A shell the rules cannot classify refuses the file at the shell value.
#[test]
fn unknown_shells_are_refused() {
    let text: &str = "jobs:\n  a:\n    steps:\n      - run: x\n        shell: elvish {0}\n";
    let error: ProcessorError = steps(text).expect_err("unknown shell");
    assert_eq!(error.offset, text.find("elvish").expect("shell"));
    assert!(error.message.contains("elvish"), "{}", error.message);
    let expression: &str =
        "jobs:\n  a:\n    steps:\n      - run: x\n        shell: ${{ matrix.shell }}\n";
    assert!(
        steps(expression)
            .expect_err("expression")
            .message
            .contains("expression")
    );
}

/// An unreadable form refuses the whole file at its byte.
#[test]
fn unreadable_forms_are_refused() {
    let text: &str = "jobs:\n  a:\n    steps:\n      - run: \"quoted\"\n";
    let error: ProcessorError = steps(text).expect_err("quoted");
    assert_eq!(error.filename, ".github/workflows/ci.yml");
    assert_eq!(error.offset, text.find("\"quoted\"").expect("quote"));
}

/// A step with no script and a workflow with no steps extract nothing.
#[test]
fn nothing_to_extract() {
    assert!(names("jobs:\n  a:\n    steps:\n      - uses: x\n").is_empty());
    assert!(names("name: empty\n").is_empty());
    assert!(names("").is_empty());
}
