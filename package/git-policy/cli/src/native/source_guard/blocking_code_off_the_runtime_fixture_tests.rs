//! Filesystem-backed controls of the scan in `blocking_code_off_the_runtime_tests.rs`:
//! each builds a small crate layout on disk and runs the scan over it,
//! so the discovery,
//! the release or test-only
//! classification and the place a rule reads are exercised on a layout rather than assumed from this crate's own.

use super::rules::Allowed;
use super::{Finding, findings_in, stale, unexplained};
use crate::source_scan_tests::{release_root, write_tree};
use crate::test_support::fixture;

/// A line the `thread-start` rule matches.
const THREAD_START: &str = "    let started = std::thread::Builder::new().spawn(move || work());\n";

/// A line the `storage-read-on-a-task` rule matches.
const STORAGE_READ: &str = "    let head = std::fs::read_to_string(git_dir.join(\"HEAD\"));\n";

/// What:
///  `fn scan(name: &str, files: &[(&str, &str)]) -> Result<Vec<Finding>, String>` writes `files` into a fresh fixture
///  directory called `name` and scans it as a crate whose only root is `lib.rs`.
///  `&[(&str, &str)]` is a borrowed list of (relative name,
/// text) pairs (TS:
/// `[string, string][]`).
/// Why:
///  Every control states its layout as data and reads the scan's answer,
/// with the error kept as a value so a control can assert that a layout is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(name: string, files: [string, string][]): Finding[]  // throws Error
/// ```
fn scan(name: &str, files: &[(&str, &str)]) -> Result<Vec<Finding>, String> {
    let root = fixture(name);
    write_tree(&root, files);
    return findings_in(&root, &[release_root("lib.rs")]);
}

/// A thread started in a nested release file is found,
/// and the same line in a test-only module is not.
#[test]
fn a_thread_start_in_a_release_file_is_found_and_one_in_test_support_is_not() {
    let found: Vec<Finding> = scan(
        "blocking-scan-nested-production",
        &[
            ("lib.rs", "mod engine;\n#[cfg(test)]\nmod support;\n"),
            ("engine/mod.rs", "mod launch;\n"),
            ("engine/launch.rs", &format!("fn launch() {{\n{THREAD_START}}}\n")),
            ("support/mod.rs", &format!("fn support() {{\n{THREAD_START}}}\n")),
        ],
    )
    .expect("a resolvable layout");
    assert_eq!(found, vec![(String::from("engine/launch.rs"), "thread-start", 2)]);
}

/// A storage read inside an `async fn` is found,
/// and the same read inside a closure handed to the blocking pool,
/// or inside a function that is
/// not `async`,
/// is not.
#[test]
fn only_the_work_of_a_task_is_a_finding_and_a_pool_job_is_not() {
    let found: Vec<Finding> = scan(
        "blocking-scan-a-task",
        &[
            ("lib.rs", "mod read_head;\n"),
            (
                "read_head.rs",
                &format!(
                    "async fn on_a_task(git_dir: std::path::PathBuf) {{\n{STORAGE_READ}}}\n\nasync fn on_the_pool(git_dir: \
                     std::path::PathBuf) {{\n    let head = run_blocking_job(move || {{\n        std::fs::read_to_string(git_dir)   \
                     \n    }});\n}}\n\nfn on_a_thread(git_dir: &std::path::Path) {{\n{STORAGE_READ}}}\n"
                ),
            ),
        ],
    )
    .expect("a resolvable layout");
    assert_eq!(found, vec![(String::from("read_head.rs"), "storage-read-on-a-task", 2)]);
}

/// A child started inside a function that builds a `Command` is found,
/// while a method named `status` on something else
/// in a function that holds no command is not.
#[test]
fn a_start_beside_a_command_is_a_finding_and_a_look_alike_is_not() {
    let found: Vec<Finding> = scan(
        "blocking-scan-a-command",
        &[
            ("lib.rs", "mod editors;\n"),
            (
                "editors.rs",
                "fn launch(program: &str) {\n    let mut command: std::process::Command = std::process::Command::new(program);\n\n    \
                 command.status().expect(\"the editor starts\");\n}\n\nfn answered(response: Reply) {\n    let status: u16 = \
                 response.status().as_u16();\n}\n",
            ),
        ],
    )
    .expect("a resolvable layout");
    // The line above the start is joined to it, so both lines answer for the one call.
    assert_eq!(
        found,
        vec![(String::from("editors.rs"), "unretried-start", 3), (String::from("editors.rs"), "unretried-start", 4)]
    );
}

/// The retried start is not a finding,
/// so the rule reads the call rather than the word.
#[test]
fn the_retried_start_is_not_a_finding() {
    let found: Vec<Finding> = scan(
        "blocking-scan-retried-start",
        &[
            ("lib.rs", "mod editors;\n"),
            (
                "editors.rs",
                "fn launch(program: &str) {\n    let mut command: std::process::Command = std::process::Command::new(program);\n    \
                 command.status_retrying().expect(\"the editor starts\");\n}\n",
            ),
        ],
    )
    .expect("a resolvable layout");
    assert_eq!(found, Vec::<Finding>::new());
}

/// A string that continues over a line break still leaves the lines after it numbered and balanced,
/// so a file that
///  wraps a long message cannot hide the functions after it.
#[test]
fn a_string_that_continues_over_a_line_break_keeps_the_file_readable() {
    let text: String = format!(
        "fn message(detail: &str) -> String {{\n    return format!(\n        \"a message about {{detail}} that wraps. \\\n         \
         Run the command again to see its own words.\"\n    );\n}}\n\nfn later() {{\n{THREAD_START}}}\n"
    );
    // The planted line is the one the scan must name, wherever the wrapped message put it.
    let planted: usize = text
        .lines()
        .position(|line| {
            return line.contains("std::thread::Builder");
        })
        .expect("the planted line")
        + 1;
    let found: Vec<Finding> = scan("blocking-scan-continued-string", &[("lib.rs", "mod message;\n"), ("message.rs", text.as_str())])
        .expect("a resolvable layout");
    assert_eq!(found, vec![(String::from("message.rs"), "thread-start", planted)]);
}

/// A start chained onto the command builder is found even where the function never names `Command` itself,
/// and the retried
///  start on the same builder is not.
#[test]
fn a_start_chained_onto_the_command_builder_is_found() {
    let found: Vec<Finding> = scan(
        "blocking-scan-built-start",
        &[
            ("lib.rs", "mod query;\n"),
            (
                "query.rs",
                "struct Ask;\n\nimpl Ask {\n    fn ask(&mut self) -> std::io::Result<()> {\n        let _kept = \
                 git_command(self.git, arguments, overlay).output_retrying()?;\n        let _done = \
                 git_command(self.git, arguments, overlay).output()?;\n        return Ok(());\n    }\n}\n",
            ),
        ],
    )
    .expect("a resolvable layout");
    assert_eq!(found, vec![(String::from("query.rs"), "unretried-built-start", 6)]);
}

/// An `async fn` whose parameters run over several lines is still read as a task,
/// so a signature the splitter misreads cannot
/// hide blocking work.
#[test]
fn an_async_signature_over_several_lines_still_names_a_task() {
    let found: Vec<Finding> = scan(
        "blocking-scan-a-long-signature",
        &[
            ("lib.rs", "mod long;\n"),
            (
                "long.rs",
                &format!(
                    "pub async fn read_it(\n    git_dir: &std::path::Path,\n    other: usize,\n) -> Option<String> {{\n    \
                     let _ = other;\n{STORAGE_READ}    return None;\n}}\n"
                ),
            ),
        ],
    )
    .expect("a resolvable layout");
    assert_eq!(found, vec![(String::from("long.rs"), "storage-read-on-a-task", 6)]);
}

/// An exception that names one relative path excuses that file only,
/// even when another directory holds a file of the same
/// name with the same finding.
#[test]
fn an_exception_for_one_path_does_not_excuse_another_file_with_the_same_name() {
    const ONLY_A: &[Allowed] =
        &[Allowed { file: "a/launch.rs", rule: "thread-start", why: "the step that moves it has not landed" }];
    let found: Vec<Finding> = scan(
        "blocking-scan-exception-by-path",
        &[
            ("lib.rs", "mod a;\nmod b;\n"),
            ("a/mod.rs", "mod launch;\n"),
            ("b/mod.rs", "mod launch;\n"),
            ("a/launch.rs", &format!("fn launch() {{\n{THREAD_START}}}\n")),
            ("b/launch.rs", &format!("fn launch() {{\n{THREAD_START}}}\n")),
        ],
    )
    .expect("a resolvable layout");
    let rejected: Vec<String> = unexplained(found.as_slice(), ONLY_A);
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert!(rejected[0].starts_with("b/launch.rs:2"), "{rejected:?}");
    assert_eq!(stale(found.as_slice(), ONLY_A), Vec::<String>::new(), "the exception for a/launch.rs still matches");
}

/// An exception whose code is gone is reported,
/// so the list cannot outlive what it excuses.
#[test]
fn an_exception_that_matches_nothing_is_stale() {
    const GONE: &[Allowed] = &[Allowed { file: "gone.rs", rule: "thread-start", why: "its code was removed" }];
    let found: Vec<Finding> = scan("blocking-scan-stale", &[("lib.rs", "")]).expect("a resolvable layout");
    let stale_entries: Vec<String> = stale(found.as_slice(), GONE);
    assert_eq!(stale_entries.len(), 1, "{stale_entries:?}");
    assert!(stale_entries[0].starts_with("gone.rs [thread-start]"), "{stale_entries:?}");
}

/// A file whose braces do not balance is an error,
/// so a file the splitter cannot read is never silently skipped.
#[test]
fn a_file_with_unbalanced_braces_is_an_error() {
    let outcome = scan("blocking-scan-unbalanced", &[("lib.rs", "mod half;\n"), ("half.rs", "fn half() {\n")]);
    assert!(outcome.is_err(), "{outcome:?}");
}

/// A module declared in the tree whose file is not there cannot be scanned,
/// so the scan is an error and not a pass.
#[test]
fn a_declared_module_without_a_file_is_an_error() {
    let outcome = scan("blocking-scan-missing-module", &[("lib.rs", "mod engine;\n")]);
    assert!(outcome.is_err(), "{outcome:?}");
}
