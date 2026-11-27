//! The guard of the lazy trace:
//! a production build must never compute,
//! read a file or format text for a trace event.
//!
//! A trace event is a `format_args!` value,
//! which formats nothing until a test build turns it into text,
//! but its arguments are evaluated at the call.
//! So every event whose arguments compute something is written as a statement that exists in a test build only
//! (`#[cfg(test)]` and `trace_text`),
//! and every other event may name only plain values.
//! These tests read the production sources of the bounded run and fail when either rule is broken.
//! A source that writes an event and is not in `SOURCES` is outside the guard,
//! so one test compares the list with the tree itself:
//! every production source below `src` that writes an event must be one of `SOURCES`,
//! and none of them is reached through a link (`src/test_scratch/source_tree_files.rs`).

/// What:
/// `fs` is the file system's own functions.
/// Why:
/// The coverage test reads the sources the list does not name and asks whether they write an event.
use std::fs;
/// What:
/// `PathBuf` is an owned file-system path.
/// Why:
/// The coverage test keeps the sources its walk answered.
use std::path::PathBuf;

/// What:
/// `source_tree_files` is the crate's one walk of its own sources,
/// which visits each file once and never enters a link.
/// Why:
/// A split can move an event into a new file,
/// and a list kept by hand would stop covering it without anything failing.
use crate::test_scratch::source_tree_files::source_tree_files;

/// The production sources of the bounded run that write trace events.
const SOURCES: [(&str, &str); 13] = [
    ("bounded_run.rs", include_str!("bounded_run.rs")),
    (
        "bounded_run_containment.rs",
        include_str!("bounded_run_containment.rs"),
    ),
    ("bounded_run_start.rs", include_str!("bounded_run_start.rs")),
    (
        "bounded_run_trace_and_hooks.rs",
        include_str!("bounded_run_trace_and_hooks.rs"),
    ),
    (
        "bounded_run_record_writes.rs",
        include_str!("bounded_run_record_writes.rs"),
    ),
    (
        "bounded_run_route_cache.rs",
        include_str!("bounded_run_route_cache.rs"),
    ),
    ("control_group.rs", include_str!("control_group.rs")),
    (
        "control_group_stop.rs",
        include_str!("control_group_stop.rs"),
    ),
    ("journal.rs", include_str!("journal.rs")),
    (
        "journal_directory_readers.rs",
        include_str!("journal_directory_readers.rs"),
    ),
    (
        "journal_done_marker.rs",
        include_str!("journal_done_marker.rs"),
    ),
    (
        "journal_kept_notices.rs",
        include_str!("journal_kept_notices.rs"),
    ),
    (
        "journal_transaction.rs",
        include_str!("journal_transaction.rs"),
    ),
];

/// How many code lines before a computed trace text may carry the `#[cfg(test)]` that keeps it out of a production build:
/// the attribute sits on the statement or on the `if` that holds it.
const ATTRIBUTE_REACH: usize = 3;

/// The calls that read a file or compute a text,
/// which a plain trace event must not contain.
const COMPUTING_CALLS: [&str; 6] = [
    "read_to_string",
    "file_name(",
    "weaker_notice(",
    "metadata(",
    "format!(",
    ".collect",
];

/// The code lines of a source that are not comments.
fn code_lines(source: &str) -> Vec<(usize, &str)> {
    let mut lines: Vec<(usize, &str)> = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let trimmed: &str = line.trim_start();
        if !trimmed.starts_with("//") {
            lines.push((index + 1, line));
        }
    }
    return lines;
}

/// A plain trace event names only values that already exist.
#[test]
fn a_plain_trace_event_computes_nothing() {
    let mut checked: usize = 0;
    for (name, source) in SOURCES {
        for (number, line) in code_lines(source) {
            if !line.contains("trace(format_args!(") {
                continue;
            }
            checked += 1;
            for call in COMPUTING_CALLS {
                assert!(
                    !line.contains(call),
                    "{name}:{number} passes `{call}` to a trace event that a production build would evaluate: {line}"
                );
            }
        }
    }
    assert!(
        checked > 0,
        "no plain trace event was found, so this guard checked nothing"
    );
}

/// An event that computes its text exists in a test build only.
#[test]
fn a_computed_trace_text_exists_in_a_test_build_only() {
    let mut checked: usize = 0;
    for (name, source) in SOURCES {
        let lines: Vec<(usize, &str)> = code_lines(source);
        for (position, (number, line)) in lines.iter().enumerate() {
            if !line.contains("trace_text(format!(") {
                continue;
            }
            checked += 1;
            let mut guarded: bool = false;
            for back in 1..=ATTRIBUTE_REACH {
                if position >= back && lines[position - back].1.trim() == "#[cfg(test)]" {
                    guarded = true;
                }
            }
            assert!(
                guarded,
                "{name}:{number} writes a computed trace text without `#[cfg(test)]` on one of the code lines before it, so a production build would compute it: {line}"
            );
        }
    }
    assert!(
        checked > 0,
        "no computed trace text was found, so this guard checked nothing"
    );
}

/// Every production source that writes a trace event is one of `SOURCES`:
/// the list covers the tree and not only the files a person named when it was written.
#[test]
fn every_production_source_that_writes_a_trace_event_is_read() {
    // What: the `src` folder of the crate this test binary was built from.
    let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    // What: the sources below it that write an event, held so the failure below can name them all.
    let mut writers: Vec<PathBuf> = Vec::new();
    for relative in source_tree_files(&root).expect("read the source tree") {
        // What: `to_string_lossy` reads the source's own kind from its name, because a path has no extension method.
        let name: String = relative.to_string_lossy().into_owned();
        // A test file may write events: the rule below is about a production build.
        if name.ends_with("_tests.rs") || name.ends_with("_test_support.rs") {
            continue;
        }
        let text: String = fs::read_to_string(root.join(&relative)).expect("read a source");
        if text.contains("trace(format_args!(") || text.contains("trace_text(format!(") {
            writers.push(relative);
        }
    }
    assert!(
        !writers.is_empty(),
        "no source writes a trace event, so this guard checked nothing"
    );
    // What: the names `SOURCES` holds, which the guard's two rules read.
    let mut listed: Vec<&str> = Vec::new();
    for (name, _text) in SOURCES {
        listed.push(name);
    }
    // What: the writers the list does not name, which the guard would therefore not check.
    let mut unread: Vec<PathBuf> = Vec::new();
    for path in &writers {
        let name: String = path.to_string_lossy().into_owned();
        if !listed.contains(&name.as_str()) {
            unread.push(path.clone());
        }
    }
    assert!(
        unread.is_empty(),
        "these production sources write trace events and this guard does not read them: {unread:?}; it reads {listed:?}"
    );
}
