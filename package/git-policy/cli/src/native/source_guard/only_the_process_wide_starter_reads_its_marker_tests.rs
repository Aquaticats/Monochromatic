//! The ban on a control reading a starter's marker variable to decide whether its body runs here:
//! a test hands the starter its body,
//! and the starter runs it in a child of the test binary.
//! Why:
//!  The marker belongs to the starter (`testing/process_wide_state_child_test_support.rs`, issue 6503):
//!  a control that reads it holds an answer it can read inverted,
//!  and an inverted read runs the body in the process whose state it was written to stay out of.
//!  The starter's own answer is not `pub(crate)` any more,
//!  so no module outside its file can even ask it,
//!  and this check covers the one route left: reading the variable itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of sources) if (file !== starter && file.text.includes('CLI_GIT_PROCESS_WIDE_STATE_CHILD')) fail(file);
//! ```

use std::path::Path;

use crate::source_scan_tests::{
    CrateRoot,
    SourceFile,
    classified_sources,
    real_roots,
    source_root,
    write_tree,
};

/// The variable whose presence marks the child a starter of this crate began,
/// which only that starter's own file may name.
const THE_MARKER: &str = "CLI_GIT_PROCESS_WIDE_STATE_CHILD";

/// The files that name the marker without a control reading it,
/// with why.
/// The starter's file is the one place the marker exists;
/// this check names it in the text it searches for and in the fixture it plants.
/// An entry here that matches no source of this crate is reported,
/// so a file that moves or stops naming the marker shrinks this list.
const NAMES_THE_MARKER_WITHOUT_READING_IT: &[(&str, &str)] = &[
    (
        "testing/process_wide_state_child_test_support.rs",
        "defines the marker, sets it on the child, and reads it for the child's own answer and the setters' refusal",
    ),
    (
        "source_guard/only_the_process_wide_starter_reads_its_marker_tests.rs",
        "this check names the marker and plants it in a fixture, and never reads it",
    ),
];

/// What:
///  The lines of one file that name the marker,
///  as line numbers counted from one.
/// Why:
///  A finding is `path:line`,
///  so the reader opens the line that decided by the marker,
///  and a comment that explains the starter is not one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linesNamingTheMarker(text: string): number[] { return text.split('\n').flatMap((line, index) => line.includes(THE_MARKER) && !line.trimStart().startsWith('//') ? [index + 1] : []); }
/// ```
fn lines_naming_the_marker(text: &str) -> Vec<usize> {
    let mut found: Vec<usize> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.contains(THE_MARKER) && !line.trim_start().starts_with("//") {
            found.push(index + 1);
        }
    }
    return found;
}

/// What:
///  Every source of the module tree this scan reads,
///  release files and test sidecars alike.
/// Why:
///  A marker a release file names would be a test-only name in the shipped program,
///  so the scan reads both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sourcesIn(root: string, roots: CrateRoot[]): SourceFile[] { return classifiedSources(root, roots); }
/// ```
fn sources_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<SourceFile>, String> {
    return classified_sources(root, roots);
}

/// What:
///  Every `file:line` of the crate's sources that names the marker outside the file that owns it.
/// Why:
///  The list is the finding list the ban's control asserts empty,
///  and the planted fixture below shows it fills.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsIn(root: string, roots: CrateRoot[]): string[] { return sourcesIn(root, roots).filter(notExempt).flatMap((file) => linesNamingTheMarker(read(file)).map((line) => `${file.path}:${line}`)); }
/// ```
fn findings_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<String>, String> {
    let mut found: Vec<String> = Vec::new();
    for file in sources_in(root, roots)? {
        if NAMES_THE_MARKER_WITHOUT_READING_IT
            .iter()
            .any(|(named, _)| return *named == file.path)
        {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        for number in lines_naming_the_marker(text.as_str()) {
            found.push(format!("{}:{number}", file.path));
        }
    }
    return Ok(found);
}

/// The findings in this crate's own sources.
fn findings() -> Vec<String> {
    return findings_in(&source_root(), &real_roots()).expect("the crate's sources");
}

/// What:
///  No source of this crate names the marker outside the starter's file.
/// Why:
///  A control that read it would hold the answer the body-taking starter exists to keep out of a caller's hands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(findings()).toEqual([]);
/// ```
#[test]
fn only_the_process_wide_starter_reads_its_marker() {
    let found: Vec<String> = findings();
    assert!(
        found.is_empty(),
        "the marker of a child process belongs to the starter that sets it, and a control that reads it decides by an \
         answer it can read inverted, so a control hands `run_the_body_in_a_child_of_its_own` its body instead \
         (`testing/process_wide_state_child_test_support.rs`, issue 6503):\n{}",
        found.join("\n")
    );
}

/// What:
///  Every file the scan exempts is still a source of this crate.
/// Why:
///  A list that excused a path nothing holds would hide a real finding if that path came back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(exempt.filter((named) => !sources.some((file) => file.path === named))).toEqual([]);
/// ```
#[test]
fn every_exempt_file_is_a_source_of_this_crate() {
    let sources: Vec<SourceFile> =
        sources_in(&source_root(), &real_roots()).expect("the crate's sources");
    let stale: Vec<String> = NAMES_THE_MARKER_WITHOUT_READING_IT
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

/// What:
///  The scan finds a control that reads the marker and says nothing about a comment that names it.
/// Why:
///  A check that stopped seeing its own needle would pass every tree,
///  and the comment beside the planted read is the shape the starter's own files carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(findingsIn(fixtureRoot, [releaseRoot('lib.rs')])).toEqual(['helper_tests.rs:3']);
/// ```
#[test]
fn the_scan_finds_a_planted_read_of_the_marker() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-child-marker");
    write_tree(
        root.as_path(),
        &[
            ("lib.rs", "mod helper_tests;\n"),
            (
                "helper_tests.rs",
                "fn a_control() {\n    // The marker is the starter's: `CLI_GIT_PROCESS_WIDE_STATE_CHILD` is set only there.\n\
                 let here = std::env::var_os(\"CLI_GIT_PROCESS_WIDE_STATE_CHILD\").is_some();\n}\n",
            ),
        ],
    );
    let found: Vec<String> = findings_in(
        root.as_path(),
        &[crate::source_scan_tests::release_root("lib.rs")],
    )
    .expect("the fixture sources");
    assert_eq!(
        found,
        vec![String::from("helper_tests.rs:3")],
        "the planted read of the marker was not the one finding"
    );
    crate::test_support::remove(root.as_path());
}
