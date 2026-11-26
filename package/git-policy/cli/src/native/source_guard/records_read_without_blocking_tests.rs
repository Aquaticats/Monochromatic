//! The ban on reading a record folder with a call that can wait:
//! every release file that names one of the folders this program fills reads it through
//! `plain_file` (`read_plain_text`, `read_plain_file`, `open_plain_file`),
//! which opens without blocking and refuses a handle that is not a plain file.
//! Why:
//!  A named pipe at a record's name holds a plain read for ever, because opening a pipe for reading waits for a writer that
//!  may never come: a person, a test or a hostile checkout that leaves one in `pending`,
//!  `failed`, `recovering`, `claimed`, `held`, a notice folder or the service's own notes finds every wrapped command
//!  stuck with nothing said (issue 6415; the review of `e347be3115` held `git status` and `git worktree remove` the same way).
//!  The folder names below are how a release file reaches those folders, so a module that reads one cannot avoid naming one,
//!  and a new module that reads a record folder with `std::fs::read_to_string` or `File::open` fails this check.

use std::path::Path;

use super::release_sources_in;
use crate::source_scan_tests::{
    CrateRoot,
    SourceFile,
    classified_sources,
    real_roots,
    source_root,
    write_tree,
};

/// The names that reach a folder of records this program creates and fills:
/// the work queue's states and helpers,
/// the notice folders of a removal and a failed push,
/// the service's notes and journal folder,
/// and the folder of owed index updates.
const RECORD_FOLDER_MARKERS: &[&str] = &[
    "state_directory(",
    "work_directory(",
    "WORK_DIRECTORY",
    "NOTICE_DIRECTORY",
    "ORPHAN_NOTICE_DIRECTORY",
    "KEPT_NOTICE_DIRECTORY",
    "LEFT_IN_PLACE_NOTICE_DIRECTORY",
    "BEGUN_FOLDER",
    "LEAVINGS_FOLDER",
    "RECORD_FOLDER",
    "NOTES_FOLDER",
    "OWED_FOLDER",
];

/// The calls that open a file to read it and can wait on a pipe:
/// the standard library's own readers,
/// and the open that every reader of a record folder must take instead through `plain_file`.
const BLOCKING_READS: &[&str] = &[
    "std::fs::read_to_string(",
    "std::fs::read(",
    "std::fs::File::open(",
    "OpenOptions::new()",
];

/// The files that name these calls without making them,
/// with why.
/// This check's own text holds every one of them, and so do its fixtures.
const NAMES_THE_CALLS_WITHOUT_MAKING_THEM: &[(&str, &str)] = &[(
    "source_guard/records_read_without_blocking_tests.rs",
    "this check names the reads and plants them in fixtures, and never makes one",
)];

/// The sources the scan reads:
/// every file the module tree reaches,
/// release files and test sidecars alike.
fn sources_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<SourceFile>, String> {
    return classified_sources(root, roots);
}

/// The release files that name a record folder and a call that can wait,
/// as `file: marker, call`,
/// and the line each call is made on.
fn findings_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<String>, String> {
    let mut found: Vec<String> = Vec::new();
    for path in release_sources_in(root, roots)? {
        if NAMES_THE_CALLS_WITHOUT_MAKING_THEM
            .iter()
            .any(|(named, _)| return *named == path)
        {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(path.as_str()))
            .map_err(|error| return format!("cannot read {path}: {error}"))?;
        let Some(marker) = RECORD_FOLDER_MARKERS
            .iter()
            .find(|marker| return text.contains(**marker))
        else {
            continue;
        };
        for (number, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            if let Some(call) = BLOCKING_READS
                .iter()
                .find(|call| return line.contains(**call))
            {
                found.push(format!(
                    "{path}:{}: names `{marker}` and reads with `{call}`",
                    number + 1
                ));
            }
        }
    }
    return Ok(found);
}

/// The findings in this crate's own sources.
fn findings() -> Vec<String> {
    return findings_in(&source_root(), &real_roots()).expect("the crate's sources");
}

/// No release file reads a record folder with a call that can wait on a named pipe.
#[test]
fn no_release_file_reads_a_record_folder_with_a_call_that_can_wait() {
    let found: Vec<String> = findings();
    assert!(
        found.is_empty(),
        "a named pipe at a record's name holds a plain read for ever, so every record folder is read through \
         `crate::plain_file` (`read_plain_text`, `read_plain_file`, `open_plain_file`), which opens without blocking and \
         refuses a handle that is not a plain file (issue 6415):\n{}",
        found.join("\n")
    );
}

/// Every file the scan exempts is still a source of this crate,
/// so the list shrinks when a file moves rather than excusing nothing.
#[test]
fn every_exempt_file_is_a_source_of_this_crate() {
    let sources: Vec<SourceFile> =
        sources_in(&source_root(), &real_roots()).expect("the crate's sources");
    let stale: Vec<String> = NAMES_THE_CALLS_WITHOUT_MAKING_THEM
        .iter()
        .filter(|(named, _)| {
            return !sources.iter().any(|file| return file.path == *named);
        })
        .map(|(named, why)| return format!("{named}: {why}"))
        .collect();
    assert!(
        stale.is_empty(),
        "these entries name no source of this crate, so remove them:\n{}",
        stale.join("\n")
    );
}

/// The scan fires on a module that reads a record folder with a call that can wait,
/// and stays quiet for the same module reading through `plain_file`,
/// so a check that stopped reading would fail here rather than passing every tree.
#[test]
fn the_scan_finds_a_planted_read_and_passes_a_plain_one() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-record-folder-reads");
    write_tree(
        root.as_path(),
        &[
            (
                "queue.rs",
                "pub fn read(work: &std::path::Path, id: &str) -> Option<String> {\n    let path = \
                 crate::work_item_state_directories_and_claim_names::state_directory(work, \"pending\").join(id);\n    return \
                 std::fs::read_to_string(path).ok();\n}\n",
            ),
            (
                "plain.rs",
                "pub fn read(work: &std::path::Path, id: &str) -> Option<String> {\n    let path = \
                 crate::work_item_state_directories_and_claim_names::state_directory(work, \"pending\").join(id);\n    return \
                 crate::plain_file::read_plain_text(path).ok();\n}\n",
            ),
            ("lib.rs", "pub mod plain;\npub mod queue;\n"),
        ],
    );
    let found: Vec<String> = findings_in(
        root.as_path(),
        &[crate::source_scan_tests::release_root("lib.rs")],
    )
    .expect("the fixture");
    assert_eq!(
        found.len(),
        1,
        "one read is planted and one reads a plain file: {found:?}"
    );
    assert!(
        found[0].starts_with("queue.rs:"),
        "the finding names the file that reads a record folder with a waiting call: {found:?}"
    );
    crate::test_support::remove(root.as_path());
}

/// A file that names a record folder and reads a path that is not one,
/// as this check does,
/// is reported rather than quietly allowed,
/// so the exemption above stays the only one.
#[test]
fn a_file_that_reads_beside_a_marker_is_a_finding() {
    let root: std::path::PathBuf =
        crate::test_support::fixture("source-guard-record-folder-beside");
    write_tree(
        root.as_path(),
        &[(
            "beside.rs",
            "pub fn read() -> Option<String> {\n    let path = \
             crate::work_item_state_directories_and_claim_names::state_directory(std::path::Path::new(\".\"), \"pending\");\n    \
             return std::fs::read_to_string(path.join(\"../config\")).ok();\n}\n",
        )],
    );
    let found: Vec<String> = findings_in(
        root.as_path(),
        &[crate::source_scan_tests::release_root("beside.rs")],
    )
    .expect("the fixture");
    assert_eq!(
        found.len(),
        1,
        "a waiting read in a file that names a record folder is reported: {found:?}"
    );
    crate::test_support::remove(root.as_path());
}
