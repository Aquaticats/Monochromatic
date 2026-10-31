//! What:
//!  Tests for whole-run behavior: validation, ordering, snapshot revalidation, containment, expected trees, explain.
//! Why:
//!  These are the engine's guarantees that no single rule kind can show.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('run', () => { it('writes nothing when any rule fails', () => { ... }); });
//! ```

use crate::config_decode::ConfigScope;
use crate::config_types::RuleConfig;
use crate::diagnostic::Diagnostic;
use crate::event::CollectingSink;
use crate::publish::STAGING_MARKER;
use crate::registry::RuleRegistry;
use crate::rule::{Change, Rule, RuleBuilder, RulePlan};
use crate::run::{RunMode, RunRequest, run};
use crate::test_support::{count_code, read_text, run_config, scratch_dir, write_files};
use crate::view::View;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Check mode with no expected directory.
fn check() -> RunMode {
    return RunMode::Check { expected_dir: None };
}

#[test]
fn a_failing_rule_stops_the_whole_run_before_anything_is_written() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("a.txt", "a")]);
    let config = r#"{"rules": [
        {"kind": "concat", "id": "good", "output": "good.txt", "parts": [{"file": "a.txt"}]},
        {"kind": "concat", "id": "bad", "output": "bad.txt", "parts": [{"file": "missing.txt"}]}]}"#;
    let (code1, events1) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code1, 1);
    assert_eq!(count_code(&events1, "input-missing"), 1, "{events1:?}");
    assert!(!dir.path().join("good.txt").exists(), "validation failures write nothing");
}

#[test]
fn two_writers_of_one_path_fail_unless_both_compose() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("a.txt", "a")]);
    let config = r#"{"rules": [
        {"kind": "concat", "id": "one", "output": "out.txt", "parts": ["1"]},
        {"kind": "concat", "id": "two", "output": "out.txt", "parts": ["2"]}]}"#;
    let (code1, events1) = run_config(dir.path(), config, check());
    assert_eq!(code1, 1);
    assert_eq!(count_code(&events1, "output-conflict"), 1, "{events1:?}");
}

#[test]
fn two_toml_rules_compose_on_one_manifest() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("package/a/b/Cargo.toml", "[package]\nname = \"b\"\n")]);
    let config = r#"{"rules": [
        {"kind": "toml-keys", "id": "one", "manifests": ["package/*/*/Cargo.toml"], "blocks": [],
         "edits": [{"guard": ["package"], "path": ["package", "edition"], "value": "2024"}]},
        {"kind": "toml-keys", "id": "two", "manifests": ["package/*/*/Cargo.toml"], "blocks": [],
         "edits": [{"guard": ["package"], "path": ["package", "readme"], "value": "README.md"}]}]}"#;
    let (code1, events1) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code1, 0, "{events1:?}");
    let text = read_text(dir.path(), "package/a/b/Cargo.toml");
    assert!(text.contains("edition = \"2024\"") && text.contains("readme = \"README.md\""), "{text}");
}

#[test]
fn a_rule_that_reads_a_later_rules_output_is_an_ordering_error() {
    let dir = scratch_dir();
    let config = r#"{"rules": [
        {"kind": "concat", "id": "reader", "output": "copy.txt", "parts": [{"file": "generated.txt"}]},
        {"kind": "concat", "id": "writer", "output": "generated.txt", "parts": ["x"]}]}"#;
    let (code1, events1) = run_config(dir.path(), config, check());
    assert_eq!(code1, 1);
    assert_eq!(count_code(&events1, "rule-order"), 1, "{events1:?}");
    let swapped = r#"{"rules": [
        {"kind": "concat", "id": "writer", "output": "generated.txt", "parts": ["x"]},
        {"kind": "concat", "id": "reader", "output": "copy.txt", "parts": [{"file": "generated.txt"}]}]}"#;
    let (code2, events2) = run_config(dir.path(), swapped, RunMode::Write);
    assert_eq!(code2, 0, "downstream rules read upstream planned output: {events2:?}");
    assert_eq!(read_text(dir.path(), "copy.txt"), "x");
}

#[test]
fn a_config_problem_is_exit_code_two() {
    let dir = scratch_dir();
    let (code1, events1) = run_config(dir.path(), r#"{"rules": [{"kind": "nope", "id": "x"}]}"#, check());
    assert_eq!(code1, 2);
    assert_eq!(events1[0].code, "config-invalid");
}

#[test]
fn a_symlinked_ancestor_of_an_output_is_refused_and_nothing_follows_it() {
    let dir = scratch_dir();
    let outside = scratch_dir();
    std::os::unix::fs::symlink(outside.path(), dir.path().join(".claude")).expect("symlink");
    let config = r#"{"rules": [{"kind": "concat", "id": "c", "output": ".claude/out.txt", "parts": ["x"]}]}"#;
    let (code1, events1) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code1, 1);
    assert_eq!(count_code(&events1, "symlink-rejected"), 1, "{events1:?}");
    assert!(!outside.path().join("out.txt").exists(), "the link was never followed");
}

#[test]
fn check_writes_paired_trees_and_rejects_a_directory_inside_the_worktree() {
    let dir = scratch_dir();
    let elsewhere = scratch_dir();
    write_files(dir.path(), &[("out.txt", "old\n")]);
    let config = r#"{"rules": [{"kind": "concat", "id": "c", "output": "out.txt", "parts": ["new\n"]}]}"#;
    let target = elsewhere.path().join("diff");
    let (code1, events1) = run_config(dir.path(), config, RunMode::Check { expected_dir: Some(target.clone()) });
    assert_eq!(code1, 1, "{events1:?}");
    assert_eq!(fs::read_to_string(target.join("actual/out.txt")).expect("actual side"), "old\n");
    assert_eq!(fs::read_to_string(target.join("expected/out.txt")).expect("expected side"), "new\n");
    let inside = dir.path().join("diff");
    let (code2, events2) = run_config(dir.path(), config, RunMode::Check { expected_dir: Some(inside) });
    assert_eq!(code2, 2);
    assert_eq!(count_code(&events2, "expected-dir-invalid"), 1, "{events2:?}");
}

#[test]
fn explain_names_the_output_rule_and_the_input_readers() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("AGENTS.md", "x")]);
    let config = r#"{"rules": [{"kind": "concat", "id": "claude", "output": "CLAUDE.md", "parts": [{"file": "AGENTS.md"}]}]}"#;
    let (_, output_events) = run_config(dir.path(), config, RunMode::Explain { path: "./CLAUDE.md".to_string() });
    assert_eq!(count_code(&output_events, "explain-output"), 1, "{output_events:?}");
    let (_, input_events) = run_config(dir.path(), config, RunMode::Explain { path: "AGENTS.md".to_string() });
    assert_eq!(count_code(&input_events, "explain-input"), 1, "{input_events:?}");
    let (code1, events1) = run_config(dir.path(), config, RunMode::Explain { path: "other".to_string() });
    assert_eq!((code1, count_code(&events1, "explain-none")), (0, 1));
}

#[test]
fn staging_files_left_by_a_killed_run_are_cleaned_and_modes_are_set_before_rename() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(&format!(".out.txt{STAGING_MARKER}999999"), "half")]);
    let config = r#"{"rules": [{"kind": "concat", "id": "c", "output": "out.txt", "mode": "0640", "parts": ["x"]}]}"#;
    let (code1, _) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code1, 0);
    let leftovers: Vec<String> = fs::read_dir(dir.path())
        .expect("list")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().to_string())
        .filter(|name| return name.contains(STAGING_MARKER))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

/// What:
///  A rule that reads a file once, then changes it, to prove the run replans.
///  `Arc<AtomicUsize>` is a shared counter several owners can bump (TS: a shared mutable object).
/// Why:
///  Snapshot revalidation must notice a file changing between the plan's read and its publish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RacingRule implements Rule { plans = 0; }
/// ```
struct RacingRule {
    plans: Arc<AtomicUsize>,
}

impl Rule for RacingRule {
    /// The rule id.
    fn id(&self) -> &str {
        return "racing";
    }

    /// The kind tag.
    fn kind(&self) -> &str {
        return "concat";
    }

    /// Reads `input.txt`, rewrites it on the first plan only, and copies what it read to `copy.txt`.
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic> {
        let text = view.read_text("input.txt")?.unwrap_or_default();
        if self.plans.fetch_add(1, Ordering::SeqCst) == 0 {
            fs::write(view.root().join("input.txt"), "second").expect("racing write");
        }
        let change = Change::Write { path: "copy.txt".to_string(), content: text.into_bytes(), mode: None };
        return Ok(RulePlan { changes: vec![change], warnings: Vec::new() });
    }
}

/// Builds `RacingRule`s that share one counter.
struct RacingBuilder {
    plans: Arc<AtomicUsize>,
}

impl RuleBuilder for RacingBuilder {
    /// Build the racing rule; the config entry is ignored.
    fn build(&self, _config: &RuleConfig) -> Result<Box<dyn Rule>, Diagnostic> {
        return Ok(Box::new(RacingRule { plans: Arc::clone(&self.plans) }));
    }
}

#[test]
fn a_file_that_changes_between_read_and_publish_makes_the_run_replan() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("input.txt", "first")]);
    let config_path = dir.path().join("file-enforcer.config.jsonc");
    fs::write(&config_path, r#"{"rules": [{"kind": "concat", "id": "racing", "output": "copy.txt", "parts": []}]}"#).expect("config");
    let plans = Arc::new(AtomicUsize::new(0));
    let mut registry = RuleRegistry::with_defaults();
    registry.register("concat", Box::new(RacingBuilder { plans: Arc::clone(&plans) }));
    let request = RunRequest { root: dir.path().to_path_buf(), config_path, scope: ConfigScope::Repository, mode: RunMode::Write };
    let mut sink = CollectingSink::default();
    let code = run(&request, &registry, &mut sink);
    assert_eq!(code, 0, "{:?}", sink.events);
    assert_eq!(plans.load(Ordering::SeqCst), 2, "the first plan was discarded");
    assert_eq!(read_text(dir.path(), "copy.txt"), "second", "the published copy reflects the changed input");
}
