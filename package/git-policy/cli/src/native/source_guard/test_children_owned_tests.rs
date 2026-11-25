//! What:
//!  The control that refuses a test file's unowned child start and its signal to its own process,
//!  with the ledger of the sites the tree still holds.
//! Why:
//!  A crate's tests run as threads of one process.
//!  A child no test thread owns outlives the run that started it:
//!  `lfs::lfs_upload`'s stand-in ran 45 minutes after its run ended.
//!  A signal a test aims at its own process reaches every thread of the run,
//!  and where no handler is installed at that instant its default action ends the whole test binary
//!  (issue 6463, `service::service_client_interrupted`).
//!  So a test starts a child through `test_support::spawn_owned`,
//!  or starts it and hands its number to `test_support::own_group` at once,
//!  and signals only the process or group it started.
//!  The sites the tree still holds are one line each in `test_children_not_yet_owned.txt`,
//!  so this control fails on a new site and on a ledger line whose site is gone,
//!  and the list can only shrink.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(bareStarts(sources)).toEqual(ledgerSites());   // 176 sites today, and fewer every round
//! expect(selfSignals(sources)).toEqual([]);             // no site may stay
//! ```

use std::collections::BTreeMap;
use std::path::Path;

use super::test_children_owned_ledger_tests::{Entry, entries, ledger_findings};
use super::test_children_owned_scan_tests::{bare_child_starts, self_signals};
use crate::source_scan_tests::{CrateRoot, SourceFile, classified_sources, real_roots, source_root, write_tree};

/// The ledger of the child starts that still name no owner,
/// one line per site with its path and why it is not moved yet.
const LEDGER: &str = include_str!("test_children_not_yet_owned.txt");

/// The repair a finding names,
/// said once so every message sends the reader the same way.
const REPAIR: &str = "start the child with `test_support::spawn_owned`, or hand its number to \
                      `test_support::own_group` at once, and signal only a process or group the test started";

/// What:
///  `fn test_sources_in(root, roots) -> Result<Vec<SourceFile>, String>` is every file of the crate the build compiles
///  for tests only.
///  `Result<Vec<SourceFile>, String>` is "the files or one text saying what is wrong" (sibling: `Option<Vec<SourceFile>>`,
///  which cannot say why).
/// Why:
///  The rule is about tests,
///  and a file the release build also compiles starts children for the program rather than for a test.
///  The classification comes from the module tree,
///  so a file that moves between the two sides of `cfg(test)` moves here too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function testSourcesIn(root: string, roots: CrateRoot[]): SourceFile[]  // throws Error
/// ```
fn test_sources_in(root: &Path, roots: &[CrateRoot]) -> Result<Vec<SourceFile>, String> {
    let mut found: Vec<SourceFile> = Vec::new();
    for file in classified_sources(root, roots)? {
        if file.test_only {
            found.push(file);
        }
    }
    return Ok(found);
}

/// What:
///  `fn test_source_paths(root, roots) -> Result<Vec<String>, String>` is the identity of every test-only file of the crate.
/// Why:
///  The ledger names files by that identity,
///  so a line that names anything else has outlived the file it was written for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function testSourcePaths(root: string, roots: CrateRoot[]): string[]  // throws Error
/// ```
fn test_source_paths(root: &Path, roots: &[CrateRoot]) -> Result<Vec<String>, String> {
    return Ok(test_sources_in(root, roots)?.into_iter().map(|file| return file.path).collect());
}

/// What:
///  `fn bare_starts_in(root, roots) -> Result<BTreeMap<String, Vec<usize>>, String>` maps each test file's identity to
///  the lines of the child starts in it that name no owner.
///  `BTreeMap<String, Vec<usize>>` is a map from text to a list, kept in key order (TS: `Map`, iterated sorted).
/// Why:
///  The ledger is counted per file,
///  and a file that holds no such start is absent rather than present with an empty list,
///  so a ledger line for it is reported as one whose site is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bareStartsIn(root: string, roots: CrateRoot[]): Map<string, number[]>  // throws Error
/// ```
fn bare_starts_in(root: &Path, roots: &[CrateRoot]) -> Result<BTreeMap<String, Vec<usize>>, String> {
    let mut found: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for file in test_sources_in(root, roots)? {
        let path: std::path::PathBuf = root.join(file.path.as_str());
        let text: String = std::fs::read_to_string(path.as_path()).map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        let lines: Vec<usize> = bare_child_starts(text.as_str());
        if !lines.is_empty() {
            found.insert(file.path, lines);
        }
    }
    return Ok(found);
}

/// What:
///  `fn self_signals_in(root, roots) -> Result<Vec<String>, String>` is every signal of a test file whose target names the
///  process the test binary is,
///  as `identity:line`.
/// Why:
///  A finding names a place a person can open,
///  and the line is the only handle such a call has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function selfSignalsIn(root: string, roots: CrateRoot[]): string[]  // throws Error
/// ```
fn self_signals_in(root: &Path, roots: &[CrateRoot]) -> Result<Vec<String>, String> {
    let mut found: Vec<String> = Vec::new();
    for file in test_sources_in(root, roots)? {
        let path: std::path::PathBuf = root.join(file.path.as_str());
        let text: String = std::fs::read_to_string(path.as_path()).map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        for line in self_signals(text.as_str()) {
            found.push(format!("{}:{line}", file.path));
        }
    }
    return Ok(found);
}

/// What:
///  `fn start_findings_in(root, roots, ledger) -> Result<Vec<String>, String>` is every way the tree's unowned starts and
///  `ledger` disagree,
///  read from the sources under `root`.
///  `ledger` is the ledger's own text, so a control can hand in a planted one.
/// Why:
///  The control over the real tree and the control over a planted tree ask the same question,
///  so only the tree and the ledger differ between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startFindingsIn(root: string, roots: CrateRoot[], ledger: string): string[]  // throws Error
/// ```
fn start_findings_in(root: &Path, roots: &[CrateRoot], ledger: &str) -> Result<Vec<String>, String> {
    let sites: BTreeMap<String, Vec<usize>> = bare_starts_in(root, roots)?;
    let paths: Vec<String> = test_source_paths(root, roots)?;
    let read: Vec<Entry> = entries(ledger)?;
    return Ok(ledger_findings(&sites, &paths, &read));
}

/// Every unowned start of this crate's test files that its ledger does not account for,
/// and every ledger line whose site is gone.
fn start_findings() -> Vec<String> {
    return start_findings_in(source_root().as_path(), real_roots().as_slice(), LEDGER).expect("the crate's own sources");
}

/// Every signal of this crate's test files aimed at the process the test binary is.
fn self_signal_findings() -> Vec<String> {
    return self_signals_in(source_root().as_path(), real_roots().as_slice()).expect("the crate's own sources");
}

/// No test file starts a child that the test thread's fixtures will not end,
/// except the sites the ledger names with their reason.
#[test]
fn no_test_file_starts_a_child_it_does_not_own() {
    let found: Vec<String> = start_findings();
    assert!(found.is_empty(), "{REPAIR}:\n{}", found.join("\n"));
}

/// No test file signals the process, the group or the session the whole run shares.
#[test]
fn no_test_file_signals_the_process_the_run_shares() {
    let found: Vec<String> = self_signal_findings();
    assert!(
        found.is_empty(),
        "a signal to this process reaches every test of the run and ends it wherever no handler is installed; signal only a process or group the test started:\n{}",
        found.join("\n")
    );
}

/// What:
///  `fn test_root(path: &str) -> CrateRoot` is a root the build compiles for tests only,
///  which is how a planted test file is reached.
/// Why:
///  A fixture's file must be classified test-only for the scans to read it,
///  and a root is what the module tree starts from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const testRoot = (path: string): CrateRoot => ({ path, testOnly: true })
/// ```
fn test_root(path: &str) -> CrateRoot {
    return CrateRoot { path: String::from(path), test_only: true };
}

/// The scan finds a planted bare start and a planted self-signal,
/// and passes over a start its group owns and one only a comment or a text names.
#[test]
fn the_scan_finds_a_planted_start_and_a_planted_signal() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-test-children");
    let planted: &str = "fn owned_by_its_group() {\n    let mut command: Command = Command::new(\"sleep\");\n    let child: Child = command.spawn().expect(\"the child\");\n    own_group(&child);\n}\n\nfn bare_start() {\n    let mut command: Command = Command::new(\"sleep\");\n    let child: Child = command.spawn().expect(\"the child starts\");\n    child.wait().expect(\"the child ends\");\n}\n\nfn signal_to_itself() {\n    let _sent: libc::c_int = unsafe { libc::kill(libc::getpid(), libc::SIGTERM) };\n}\n\nfn a_group_of_its_own() {\n    let _sent: libc::c_int = unsafe { libc::killpg(libc::getpgrp(), libc::SIGKILL) };\n}\n\n// A comment naming command.spawn() and libc::kill(libc::getpid(), libc::SIGTERM) says nothing.\nconst A_NEEDLE: &str = \".spawn() and libc::kill(libc::getpid())\";\n";
    write_tree(root.as_path(), &[("planted_tests.rs", planted)]);
    let roots: [CrateRoot; 1] = [test_root("planted_tests.rs")];
    let sites: BTreeMap<String, Vec<usize>> = bare_starts_in(root.as_path(), roots.as_slice()).expect("the fixture sources");
    let signals: Vec<String> = self_signals_in(root.as_path(), roots.as_slice()).expect("the fixture sources");
    crate::test_support::remove(root.as_path());
    assert_eq!(sites.get("planted_tests.rs"), Some(&vec![9]), "only the start with no owner beside it is a site");
    assert_eq!(signals, vec![String::from("planted_tests.rs:14"), String::from("planted_tests.rs:18")], "both signals name this process or its group");
}

/// A ledger line whose site is gone fails,
/// and one for a file that is no test source fails too,
/// so the list can only shrink.
#[test]
fn a_ledger_line_whose_site_is_gone_is_reported() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-test-children-ledger");
    let planted: &str = "fn bare_start() {\n    let child: Child = command.spawn().expect(\"the child starts\");\n}\n";
    write_tree(root.as_path(), &[("planted_tests.rs", planted)]);
    let roots: [CrateRoot; 1] = [test_root("planted_tests.rs")];
    let stale: &str = "planted_tests.rs | one site, not moved yet\nplanted_tests.rs | a second site the file no longer holds\nmoved_away_tests.rs | a site of a file that is gone\n";
    let found: Vec<String> = start_findings_in(root.as_path(), roots.as_slice(), stale).expect("the fixture sources");
    let unnamed: Vec<String> = start_findings_in(root.as_path(), roots.as_slice(), "").expect("the fixture sources");
    crate::test_support::remove(root.as_path());
    assert_eq!(found.len(), 2, "the second line of that file and the line of the absent file are both reported: {found:?}");
    assert!(found.iter().any(|each| return each.contains("still names 2 site(s)")), "{found:?}");
    assert!(found.iter().any(|each| return each.contains("moved_away_tests.rs")), "{found:?}");
    assert_eq!(unnamed.len(), 1, "a tree whose ledger is empty reports its one site: {unnamed:?}");
    assert!(unnamed[0].contains("planted_tests.rs:2"), "{unnamed:?}");
}
