//! The ban on changing the directory of the process:
//! no source of this crate moves it,
//! so a control that needs the code under test to start somewhere passes that
//! directory as an input instead.
//! The current directory belongs to the whole process,
//! and `cargo test` runs a crate's tests as threads of one process,
//! so a control that moved it made every other control read its fixture,
//! or a fixture it had already deleted
//! (issue 6454).

use std::path::Path;

use crate::source_scan_tests::{CrateRoot, SourceFile, classified_sources, real_roots, source_root, write_tree};

/// The call whose presence is a finding.
/// A control that wants the code under test to start in a fixture passes the fixture's path
/// (`crate::start_directory::StartDirectory::at`),
/// which changes nobody's directory.
const CHANGING_THE_PROCESS_DIRECTORY: &str = "set_current_dir(";

/// The files that name the call without making it,
/// with why.
/// This check's own text holds the call twice: the text it searches for and the one its control plants in a fixture.
/// An entry here that matches nothing is reported,
/// so a file that moves or stops naming the call shrinks this list.
const NAMES_THE_CALL_WITHOUT_MAKING_IT: &[(&str, &str)] = &[(
    "source_guard/no_test_changes_the_process_directory_tests.rs",
    "this check names the call and plants it in a fixture, and never makes it",
)];

/// The lines of one file that make the call,
/// as line numbers.
/// A call split over two lines is found where it starts,
/// and a comment that names it is not a call.
fn lines_making_the_call(text: &str) -> Vec<usize> {
    let mut found: Vec<usize> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let mut searched: usize = 0;
        while let Some(place) = line[searched..].find(CHANGING_THE_PROCESS_DIRECTORY) {
            let column: usize = searched + place;
            let before: &str = &line[..column];
            let commented: bool = before.trim_start().starts_with("//");
            if !commented {
                found.push(index + 1);
            }
            searched = column + CHANGING_THE_PROCESS_DIRECTORY.len();
        }
    }
    return found;
}

/// The sources the scan reads:
/// every file the module tree reaches,
/// release files and test sidecars alike.
fn sources_in(root: &Path, roots: &[CrateRoot]) -> Result<Vec<SourceFile>, String> {
    return classified_sources(root, roots);
}

/// Every `file:line` of the crate's sources that changes the directory of the process.
fn findings_in(root: &Path, roots: &[CrateRoot]) -> Result<Vec<String>, String> {
    let mut found: Vec<String> = Vec::new();
    for file in sources_in(root, roots)? {
        if NAMES_THE_CALL_WITHOUT_MAKING_IT.iter().any(|(named, _)| return *named == file.path) {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        for number in lines_making_the_call(text.as_str()) {
            found.push(format!("{}:{number}", file.path));
        }
    }
    return Ok(found);
}

/// The findings in this crate's own sources.
fn findings() -> Vec<String> {
    return findings_in(&source_root(), &real_roots()).expect("the crate's sources");
}

/// No source of this crate changes the directory of the process.
/// A control that wants the code under test to start in a fixture passes the fixture's path as an input.
#[test]
fn no_source_changes_the_process_directory() {
    let found: Vec<String> = findings();
    assert!(
        found.is_empty(),
        "the directory belongs to the whole process, and a crate's tests run as threads of it, so a control passes its \
         fixture as an input (`crate::start_directory::StartDirectory::at`) and never moves the process:\n{}",
        found.join("\n")
    );
}

/// Every file the scan exempts is still a source of this crate,
/// so the list shrinks when a file moves rather than excusing nothing.
#[test]
fn every_exempt_file_is_a_source_of_this_crate() {
    let sources: Vec<SourceFile> = sources_in(&source_root(), &real_roots()).expect("the crate's sources");
    let stale: Vec<String> = NAMES_THE_CALL_WITHOUT_MAKING_IT
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

/// The scan fires on the call it exists for,
/// so a check that stopped reading would fail here rather than passing every tree.
#[test]
fn the_scan_finds_a_planted_call() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-process-directory");
    write_tree(
        root.as_path(),
        &[
            ("lib.rs", "mod helper_tests;\n"),
            (
                "helper_tests.rs",
                "fn a_control() {\n    std::env::set_current_dir(root.as_path()).expect(\"the fixture\");\n}\n\
                 fn a_split_call() {\n    std::env\n        .set_current_dir(root.as_path())\n        .expect(\"the fixture\");\n}\n\
                 // A comment that names std::env::set_current_dir is not a call.\n",
            ),
        ],
    );
    let found: Vec<String> = findings_in(root.as_path(), &[crate::source_scan_tests::release_root("lib.rs")])
        .expect("the fixture sources");
    assert_eq!(found, vec![String::from("helper_tests.rs:2"), String::from("helper_tests.rs:6")], "the planted calls were not found");
    crate::test_support::remove(root.as_path());
}
