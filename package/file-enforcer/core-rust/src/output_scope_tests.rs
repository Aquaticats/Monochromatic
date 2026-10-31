//! What:
//!  Tests for the committed and local output split of `check` and `write`.
//! Why:
//!  CI relies on `--outputs committed` ignoring every gitignored output and on `--outputs local` ignoring
//! every other one,
//! so each direction needs a drifting and a missing fixture that must change the result,
//! and the matcher needs the
//!  cases where the ignore rules decide through a parent directory,
//! a nested file or a negation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('output scope', () => { it('reports only committed drift', () => { ... }); });
//! ```

use crate::event::{CollectingSink, Event};
use crate::output_scope::OutputScope;
use crate::registry::RuleRegistry;
use crate::run::{RunMode, RunRequest, run};
use crate::test_support::{count_code, read_text, scratch_dir, write_files};
use std::fs;
use std::path::Path;

/// One committed output (`committed.txt`),
/// one output a file pattern ignores (`out.local.txt`) and one a directory pattern ignores (`gen/a.txt`).
const CONFIG: &str = r#"{"rules": [
    {"kind": "concat", "id": "committed-rule", "output": "committed.txt", "parts": ["new\n"]},
    {"kind": "concat", "id": "file-pattern-rule", "output": "out.local.txt", "parts": ["new\n"]},
    {"kind": "concat", "id": "directory-pattern-rule", "output": "gen/a.txt", "parts": ["new\n"]}]}"#;

/// The ignore rules of the fixture.
const IGNORE: &str = "*.local.*\n/gen/\n";

/// Run a config in a mode and scope against a root;
/// return the exit code and events.
fn run_scoped(root: &Path, config: &str, mode: RunMode, outputs: OutputScope) -> (i32, Vec<Event>) {
    let config_path = root.join("file-enforcer.config.jsonc");
    fs::write(&config_path, config).expect("the config is writable");
    let request = RunRequest { root: root.to_path_buf(), config_path, scope: crate::config_decode::ConfigScope::Repository, mode, outputs };
    let mut sink = CollectingSink::default();
    let code = run(&request, &RuleRegistry::with_defaults(), &mut sink);
    return (code, sink.events);
}

/// Check mode with no expected directory.
fn check() -> RunMode {
    return RunMode::Check { expected_dir: None };
}

/// The `path` members of the drift events,
/// sorted.
fn drifted_paths(events: &[Event]) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    for event in events {
        if event.event_type == "drift" {
            paths.push(event.fields["path"].as_str().expect("a drift names its path").to_string());
        }
    }
    paths.sort();
    return paths;
}

/// The final `result` event.
fn result_of(events: &[Event]) -> &Event {
    return events.iter().rev().find(|event| return event.event_type == "result").expect("every run ends with a result");
}

#[test]
fn missing_outputs_are_split_between_the_scopes() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".gitignore", IGNORE)]);
    let (all_code, all) = run_scoped(dir.path(), CONFIG, check(), OutputScope::All);
    assert_eq!(all_code, 1, "{all:?}");
    assert_eq!(drifted_paths(&all), ["committed.txt", "gen/a.txt", "out.local.txt"]);
    let (committed_code, committed) = run_scoped(dir.path(), CONFIG, check(), OutputScope::Committed);
    assert_eq!(committed_code, 1, "{committed:?}");
    assert_eq!(drifted_paths(&committed), ["committed.txt"]);
    assert_eq!(committed.iter().find(|event| return event.event_type == "drift").expect("drift").code, "drift-missing");
    let (local_code, local) = run_scoped(dir.path(), CONFIG, check(), OutputScope::Local);
    assert_eq!(local_code, 1, "{local:?}");
    assert_eq!(drifted_paths(&local), ["gen/a.txt", "out.local.txt"]);
    assert!(local.iter().filter(|event| return event.event_type == "drift").all(|event| return event.code == "drift-missing"));
}

#[test]
fn stale_outputs_are_split_between_the_scopes() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".gitignore", IGNORE), ("committed.txt", "old\n"), ("out.local.txt", "old\n"), ("gen/a.txt", "old\n")]);
    let (committed_code, committed) = run_scoped(dir.path(), CONFIG, check(), OutputScope::Committed);
    assert_eq!(committed_code, 1, "{committed:?}");
    assert_eq!(drifted_paths(&committed), ["committed.txt"]);
    assert_eq!(committed.iter().find(|event| return event.event_type == "drift").expect("drift").code, "drift-content");
    let (local_code, local) = run_scoped(dir.path(), CONFIG, check(), OutputScope::Local);
    assert_eq!(local_code, 1, "{local:?}");
    assert_eq!(drifted_paths(&local), ["gen/a.txt", "out.local.txt"]);
}

#[test]
fn drift_outside_the_scope_neither_fails_the_run_nor_appears_in_it() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".gitignore", IGNORE), ("committed.txt", "new\n")]);
    let (committed_code, committed) = run_scoped(dir.path(), CONFIG, check(), OutputScope::Committed);
    assert_eq!(committed_code, 0, "the two local outputs are missing but out of scope: {committed:?}");
    assert_eq!(drifted_paths(&committed), Vec::<String>::new());
    assert_eq!(result_of(&committed).code, "check-clean");
    assert_eq!(result_of(&committed).fields["drifted"], 0);
    assert_eq!(result_of(&committed).fields["outputs"], "committed");
    assert_eq!(result_of(&committed).message, "check: every committed generated file is up to date");
    let (local_code, local) = run_scoped(dir.path(), CONFIG, check(), OutputScope::Local);
    assert_eq!(local_code, 1, "{local:?}");
    assert_eq!(result_of(&local).code, "check-drift");
    assert_eq!(result_of(&local).fields["drifted"], 2);
    assert_eq!(result_of(&local).message, "check: local generated files differ from their rules");
    write_files(dir.path(), &[("committed.txt", "old\n"), ("out.local.txt", "new\n"), ("gen/a.txt", "new\n")]);
    let (local_clean_code, local_clean) = run_scoped(dir.path(), CONFIG, check(), OutputScope::Local);
    assert_eq!(local_clean_code, 0, "the stale committed output is out of scope: {local_clean:?}");
    assert_eq!(result_of(&local_clean).message, "check: every local generated file is up to date");
}

#[test]
fn the_default_scope_reports_everything_and_adds_no_outputs_member() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".gitignore", IGNORE)]);
    let (code, events) = run_scoped(dir.path(), CONFIG, check(), OutputScope::All);
    assert_eq!(code, 1);
    assert_eq!(result_of(&events).fields["drifted"], 3);
    assert!(!result_of(&events).fields.contains_key("outputs"), "the unscoped line keeps its documented shape");
    assert_eq!(result_of(&events).message, "check: generated files differ from their rules");
}

#[test]
fn write_publishes_only_the_selected_scope() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".gitignore", IGNORE)]);
    let (committed_code, committed) = run_scoped(dir.path(), CONFIG, RunMode::Write, OutputScope::Committed);
    assert_eq!(committed_code, 0, "{committed:?}");
    assert_eq!(read_text(dir.path(), "committed.txt"), "new\n");
    assert!(!dir.path().join("out.local.txt").exists() && !dir.path().join("gen").exists(), "local outputs stay absent");
    assert_eq!(count_code(&committed, "file-written"), 1);
    assert_eq!(result_of(&committed).message, "write: every committed generated file is now up to date");
    assert_eq!(result_of(&committed).fields["outputs"], "committed");
    let (local_code, local) = run_scoped(dir.path(), CONFIG, RunMode::Write, OutputScope::Local);
    assert_eq!(local_code, 0, "{local:?}");
    assert_eq!(count_code(&local, "file-written"), 2);
    assert_eq!(read_text(dir.path(), "gen/a.txt"), "new\n");
    let (all_code, all) = run_scoped(dir.path(), CONFIG, check(), OutputScope::All);
    assert_eq!(all_code, 0, "two scoped writes converge to a clean full check: {all:?}");
}

#[test]
fn a_scoped_write_leaves_stale_files_of_the_other_scope_untouched() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".gitignore", IGNORE), ("committed.txt", "old\n"), ("out.local.txt", "old\n")]);
    let (code, events) = run_scoped(dir.path(), CONFIG, RunMode::Write, OutputScope::Local);
    assert_eq!(code, 0, "{events:?}");
    assert_eq!(read_text(dir.path(), "committed.txt"), "old\n");
    assert_eq!(read_text(dir.path(), "out.local.txt"), "new\n");
}

#[test]
fn the_expected_trees_hold_only_the_selected_scope() {
    let dir = scratch_dir();
    let elsewhere = scratch_dir();
    write_files(dir.path(), &[(".gitignore", IGNORE)]);
    let target = elsewhere.path().join("diff");
    let (code, events) = run_scoped(dir.path(), CONFIG, RunMode::Check { expected_dir: Some(target.clone()) }, OutputScope::Local);
    assert_eq!(code, 1, "{events:?}");
    assert_eq!(read_text(&target, "expected/out.local.txt"), "new\n");
    assert!(!target.join("expected/committed.txt").exists(), "committed drift is outside the scope");
}

#[test]
fn a_tree_without_any_ignore_file_has_only_committed_outputs() {
    let dir = scratch_dir();
    let (committed_code, committed) = run_scoped(dir.path(), CONFIG, check(), OutputScope::Committed);
    assert_eq!(committed_code, 1);
    assert_eq!(drifted_paths(&committed).len(), 3, "{committed:?}");
    let (local_code, local) = run_scoped(dir.path(), CONFIG, check(), OutputScope::Local);
    assert_eq!(local_code, 0, "{local:?}");
}

#[test]
fn a_directory_pattern_matches_a_removed_directory_and_the_files_in_it() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".gitignore", "old/\n"), ("src/keep.txt", "k\n"), ("dst/keep.txt", "k\n"), ("dst/old/f.txt", "f\n"), ("dst/stale.txt", "s\n")]);
    let config = r#"{"rules": [{"kind": "mirror", "id": "m", "from": "src", "to": "dst"}]}"#;
    let (local_code, local) = run_scoped(dir.path(), config, check(), OutputScope::Local);
    assert_eq!(local_code, 1, "{local:?}");
    assert_eq!(drifted_paths(&local), ["dst/old", "dst/old/f.txt"], "the directory pattern matches the directory and, through it, its file");
    let (committed_code, committed) = run_scoped(dir.path(), config, check(), OutputScope::Committed);
    assert_eq!(committed_code, 1, "{committed:?}");
    assert_eq!(drifted_paths(&committed), ["dst/stale.txt"]);
    let (write_code, written) = run_scoped(dir.path(), config, RunMode::Write, OutputScope::Committed);
    assert_eq!(write_code, 0, "{written:?}");
    assert!(dir.path().join("dst/old/f.txt").exists(), "a committed-only write removes nothing ignored");
    assert!(!dir.path().join("dst/stale.txt").exists());
}

#[test]
fn a_nested_ignore_file_overrides_its_parent_and_a_parent_directory_cannot_be_reopened() {
    let dir = scratch_dir();
    write_files(
        dir.path(),
        &[(".gitignore", "*.local.*\n/gen/\n"), ("sub/.gitignore", "!keep.local.txt\n/own.txt\n"), ("gen/.gitignore", "!a.txt\n")],
    );
    let config = r#"{"rules": [
        {"kind": "concat", "id": "kept", "output": "sub/keep.local.txt", "parts": ["x"]},
        {"kind": "concat", "id": "other", "output": "sub/other.local.txt", "parts": ["x"]},
        {"kind": "concat", "id": "own", "output": "sub/own.txt", "parts": ["x"]},
        {"kind": "concat", "id": "reopened", "output": "gen/a.txt", "parts": ["x"]},
        {"kind": "concat", "id": "anchored", "output": "elsewhere/own.txt", "parts": ["x"]}]}"#;
    let (local_code, local) = run_scoped(dir.path(), config, check(), OutputScope::Local);
    assert_eq!(local_code, 1, "{local:?}");
    assert_eq!(drifted_paths(&local), ["gen/a.txt", "sub/other.local.txt", "sub/own.txt"]);
    let (committed_code, committed) = run_scoped(dir.path(), config, check(), OutputScope::Committed);
    assert_eq!(committed_code, 1, "{committed:?}");
    assert_eq!(drifted_paths(&committed), ["elsewhere/own.txt", "sub/keep.local.txt"]);
}

#[test]
fn a_planned_ignore_file_classifies_by_its_planned_text() {
    let dir = scratch_dir();
    let config = r#"{"rules": [
        {"kind": "concat", "id": "ignore-file", "output": ".gitignore", "parts": ["generated/\n"]},
        {"kind": "concat", "id": "output", "output": "generated/x.txt", "parts": ["x"]}]}"#;
    let (local_code, local) = run_scoped(dir.path(), config, check(), OutputScope::Local);
    assert_eq!(local_code, 1, "{local:?}");
    assert_eq!(drifted_paths(&local), ["generated/x.txt"]);
    let (committed_code, committed) = run_scoped(dir.path(), config, check(), OutputScope::Committed);
    assert_eq!(drifted_paths(&committed), [".gitignore"], "{committed:?}");
    assert_eq!(committed_code, 1);
}

#[test]
fn an_uncompilable_pattern_is_a_named_failure_only_for_a_scoped_run() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".gitignore", "[z-a]\n")]);
    let (scoped_code, scoped) = run_scoped(dir.path(), CONFIG, check(), OutputScope::Committed);
    assert_eq!(scoped_code, 1, "{scoped:?}");
    assert_eq!(count_code(&scoped, "gitignore-invalid"), 1, "{scoped:?}");
    let failure = scoped.iter().find(|event| return event.code == "gitignore-invalid").expect("the failure");
    assert_eq!(failure.fields["path"], ".gitignore");
    assert!(failure.hint.as_deref().is_some_and(|hint| return hint.contains("--outputs")), "{failure:?}");
    let (all_code, all) = run_scoped(dir.path(), CONFIG, check(), OutputScope::All);
    assert_eq!(all_code, 1, "{all:?}");
    assert_eq!(count_code(&all, "gitignore-invalid"), 0, "the unscoped run never reads ignore files");
    assert_eq!(drifted_paths(&all).len(), 3);
}

#[test]
fn explain_ignores_the_scope() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".gitignore", IGNORE)]);
    let (code, events) = run_scoped(dir.path(), CONFIG, RunMode::Explain { path: "committed.txt".to_string() }, OutputScope::Local);
    assert_eq!(code, 0, "{events:?}");
    assert_eq!(count_code(&events, "explain-output"), 1, "{events:?}");
}

#[test]
fn scope_words_round_trip_and_unknown_words_are_rejected() {
    for scope in [OutputScope::All, OutputScope::Committed, OutputScope::Local] {
        assert_eq!(OutputScope::parse(scope.name()), Some(scope));
    }
    assert_eq!(OutputScope::parse("Committed"), None);
    assert_eq!(OutputScope::parse(""), None);
    assert_eq!(OutputScope::default(), OutputScope::All);
}
