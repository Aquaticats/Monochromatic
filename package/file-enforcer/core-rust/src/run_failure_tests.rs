//! What:
//!  Tests for the failure and edge branches of a run:
//!  a snapshot that keeps changing,
//! an unbound variable,
//! `explain` on a directory,
//!  and `--expected-dir` on a directory that is not empty.
//! Why:
//!  Each branch has its own diagnostic code and exit code,
//! and the happy-path suites never reach them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('run failures', () => { it('gives up after three replans', () => { ... }); });
//! ```

use crate::config_decode::ConfigScope;
use crate::config_types::RuleConfig;
use crate::diagnostic::Diagnostic;
use crate::event::CollectingSink;
use crate::registry::RuleRegistry;
use crate::rule::{Change, Rule, RuleBuilder, RulePlan};
use crate::run::{RunMode, RunRequest, run};
use crate::test_support::{count_code, run_config, scratch_dir, write_files};
use crate::view::View;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// What:
///  A rule that reads `input.txt` and then rewrites it,
/// differently on every plan.
///  `Arc<AtomicUsize>` is a counter shared between threads and owners (TS:
/// a `{ n: number }` object both sides hold).
/// Why:
///  A plan whose input changes under it is thrown away and redone;
///  a writer that never stops makes every plan stale,
/// which must end in a named failure and not a loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ChurningRule implements Rule { plans: { n: number }; plan(view) { /* read input, then rewrite it */ } }
/// ```
struct ChurningRule {
    /// How many times the engine has planned this rule.
    plans: Arc<AtomicUsize>,
}

/// What:
///  The rule's behavior.
/// Why:
///  The engine needs an id,
/// a kind and a plan;
/// the plan is the part under test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ChurningRule { id(): string; kind(): string; plan(view: View): RulePlan }
/// ```
impl Rule for ChurningRule {
    /// The rule's config id.
    fn id(&self) -> &str {
        return "churning";
    }

    /// The kind it stands in for.
    fn kind(&self) -> &str {
        return "concat";
    }

    /// Read the input,
    /// then change it so the engine's revalidation sees a different file.
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic> {
        let text = view.read_text("input.txt")?.unwrap_or_default();
        let number = self.plans.fetch_add(1, Ordering::SeqCst);
        fs::write(view.root().join("input.txt"), format!("version {number}")).expect("the churn write succeeds");
        let change = Change::Write { path: "copy.txt".to_string(), content: text.into_bytes(), mode: None };
        return Ok(RulePlan { changes: vec![change], warnings: Vec::new() });
    }
}

/// What:
///  The builder that hands the engine a [`ChurningRule`].
/// Why:
///  The registry builds rules from config entries through builders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ChurningBuilder implements RuleBuilder { build(config): Rule }
/// ```
struct ChurningBuilder {
    /// The counter every built rule shares.
    plans: Arc<AtomicUsize>,
}

/// What:
///  Builder behavior.
/// Why:
///  The registry calls it once per config entry of the registered kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ChurningBuilder { build(config: RuleConfig): Rule }
/// ```
impl RuleBuilder for ChurningBuilder {
    /// Build the rule around the shared counter.
    fn build(&self, _config: &RuleConfig) -> Result<Box<dyn Rule>, Diagnostic> {
        return Ok(Box::new(ChurningRule { plans: Arc::clone(&self.plans) }));
    }
}

#[test]
fn a_file_that_keeps_changing_ends_in_snapshot_changed_after_three_plans() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("input.txt", "start")]);
    let config_path = dir.path().join("file-enforcer.config.jsonc");
    let config = r#"{"rules": [{"kind": "concat", "id": "churning", "output": "copy.txt", "parts": []}]}"#;
    fs::write(&config_path, config).expect("config");
    let plans = Arc::new(AtomicUsize::new(0));
    let mut registry = RuleRegistry::with_defaults();
    registry.register("concat", Box::new(ChurningBuilder { plans: Arc::clone(&plans) }));
    let request = RunRequest { root: dir.path().to_path_buf(), config_path, scope: ConfigScope::Repository, mode: RunMode::Write };
    let mut sink = CollectingSink::default();
    let code = run(&request, &registry, &mut sink);
    assert_eq!(code, 1, "{:?}", sink.events);
    assert_eq!(count_code(&sink.events, "snapshot-changed"), 1, "{:?}", sink.events);
    assert_eq!(plans.load(Ordering::SeqCst), 3, "the engine gives up after exactly three plans");
    assert!(!dir.path().join("copy.txt").exists(), "a run that never settled publishes nothing");
    let failure = sink.events.iter().find(|event| return event.code == "snapshot-changed").expect("the failure event");
    assert!(failure.message.contains("input.txt"), "the message names the file that kept changing: {}", failure.message);
    assert!(failure.hint.as_deref().is_some_and(|hint| return hint.contains("stop the other writer")));
}

#[test]
fn check_does_not_replan_a_changing_file() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("input.txt", "start")]);
    let config_path = dir.path().join("file-enforcer.config.jsonc");
    let config = r#"{"rules": [{"kind": "concat", "id": "churning", "output": "copy.txt", "parts": []}]}"#;
    fs::write(&config_path, config).expect("config");
    let plans = Arc::new(AtomicUsize::new(0));
    let mut registry = RuleRegistry::with_defaults();
    registry.register("concat", Box::new(ChurningBuilder { plans: Arc::clone(&plans) }));
    let mode = RunMode::Check { expected_dir: None };
    let request = RunRequest { root: dir.path().to_path_buf(), config_path, scope: ConfigScope::Repository, mode };
    let mut sink = CollectingSink::default();
    let code = run(&request, &registry, &mut sink);
    assert_eq!(code, 1, "{:?}", sink.events);
    assert_eq!(plans.load(Ordering::SeqCst), 1, "only write revalidates; check reports what it saw once");
    assert_eq!(count_code(&sink.events, "snapshot-changed"), 0);
}

#[test]
fn an_unbound_variable_fails_the_run_and_names_what_the_rule_does_bind() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("package/a/b/Cargo.toml", "[package]\nname = \"b\"\n")]);
    let toml = r#"{"rules": [{"kind": "toml-keys", "id": "t", "manifests": ["package/*/*/Cargo.toml"], "blocks": [],
        "edits": [{"guard": ["package"], "path": ["package", "readme"], "value": {"var": "nope"}}]}]}"#;
    let (code, events) = run_config(dir.path(), toml, RunMode::Write);
    assert_eq!(code, 1, "{events:?}");
    assert_eq!(count_code(&events, "variable-unbound"), 1, "{events:?}");
    let message = &events.iter().find(|event| return event.code == "variable-unbound").expect("the failure").message;
    assert!(message.contains("\"nope\"") && message.contains("this rule binds"), "{message}");
    let manifest = fs::read_to_string(dir.path().join("package/a/b/Cargo.toml")).expect("manifest");
    assert_eq!(manifest, "[package]\nname = \"b\"\n", "a failed run writes nothing");
}

#[test]
fn a_concat_rule_binds_no_variables_so_any_var_is_unbound() {
    let dir = scratch_dir();
    let config = r#"{"rules": [{"kind": "concat", "id": "c", "output": "out.txt", "parts": [{"var": "dir"}]}]}"#;
    let (code, events) = run_config(dir.path(), config, RunMode::Write);
    assert_eq!(code, 1, "{events:?}");
    assert_eq!(count_code(&events, "variable-unbound"), 1, "{events:?}");
    assert!(!dir.path().join("out.txt").exists());
}

#[test]
fn explain_on_a_directory_reports_the_rule_that_lists_it_and_ignores_slash_and_dot_spellings() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("src/a.txt", "a"), ("dst/old.txt", "old")]);
    let config = r#"{"rules": [{"kind": "mirror", "id": "m", "from": "src", "to": "dst"}]}"#;
    for spelling in ["src", "src/", "./src/", "./src"] {
        let (code, events) = run_config(dir.path(), config, RunMode::Explain { path: spelling.to_string() });
        assert_eq!(code, 0, "{spelling}: {events:?}");
        assert_eq!(count_code(&events, "explain-input"), 1, "{spelling}: the mirror lists the source directory: {events:?}");
        assert_eq!(events[0].fields["path"], "src", "{spelling}: the path is normalized");
        assert_eq!(events[0].fields["rule"], "m");
    }
    let (_, target) = run_config(dir.path(), config, RunMode::Explain { path: "dst/".to_string() });
    assert_eq!(count_code(&target, "explain-input"), 1, "the mirror lists its target directory too: {target:?}");
    let (_, file) = run_config(dir.path(), config, RunMode::Explain { path: "dst/a.txt".to_string() });
    assert_eq!(count_code(&file, "explain-output"), 1, "files under the target are owned one by one: {file:?}");
    let (_, unrelated) = run_config(dir.path(), config, RunMode::Explain { path: "elsewhere/".to_string() });
    assert_eq!(count_code(&unrelated, "explain-none"), 1, "{unrelated:?}");
}

#[test]
fn an_expected_dir_that_already_holds_files_is_refused_and_an_empty_one_is_accepted() {
    let dir = scratch_dir();
    let elsewhere = scratch_dir();
    write_files(dir.path(), &[("out.txt", "old\n")]);
    let config = r#"{"rules": [{"kind": "concat", "id": "c", "output": "out.txt", "parts": ["new\n"]}]}"#;
    write_files(elsewhere.path(), &[("occupied/leftover.txt", "x")]);
    let occupied = elsewhere.path().join("occupied");
    let (refused, events) = run_config(dir.path(), config, RunMode::Check { expected_dir: Some(occupied.clone()) });
    assert_eq!(refused, 2, "{events:?}");
    let message = &events.iter().find(|event| return event.code == "expected-dir-invalid").expect("the refusal").message;
    assert!(message.contains("is not empty"), "{message}");
    assert!(!occupied.join("actual").exists(), "nothing is written into a directory that was refused");
    let empty = elsewhere.path().join("empty");
    fs::create_dir_all(&empty).expect("an existing empty directory");
    let (accepted, accepted_events) = run_config(dir.path(), config, RunMode::Check { expected_dir: Some(empty.clone()) });
    assert_eq!(accepted, 1, "drift is still exit 1: {accepted_events:?}");
    assert_eq!(count_code(&accepted_events, "expected-dir-invalid"), 0);
    assert_eq!(fs::read_to_string(empty.join("expected/out.txt")).expect("expected side"), "new\n");
}
