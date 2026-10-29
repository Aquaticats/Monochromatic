//! Controls for the post-commit gate against real repositories.
//! The TypeScript corpus is `src/policy-engine/post-commit-lifecycle.unit.test.ts` and `post-commit-facts.unit.test.ts`.
//! The incumbent's test plugins have no native equivalent, so the same behaviors are driven with the built-in
//! final-newline and forbidden-strings policies at fixed severities.

use super::*;
use crate::auto_push_single_flight::PushGit;
use crate::policy_registry::{PolicyId, Severity};
use crate::repository_facts::RepositoryFacts;
use crate::config_schema::ForbiddenStringsOptions;
use crate::scanner_test_support::{needle, run_isolated};
use crate::test_support::{executable, fixture, git, remove};
use crate::wrapper_controls::no_controls;
use std::path::PathBuf;

/// Final-newline content of the landed file: a complete line.
const LANDED_CONTENT: &str = "landed content\n";

/// Environment every query of these fixtures runs with.
fn fixture_environment() -> Vec<(OsString, OsString)> {
    let mut environment: Vec<(OsString, OsString)> = [
        ("PATH", "/usr/bin:/bin"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ]
    .iter()
    .map(|(name, value)| {
        return (OsString::from(name), OsString::from(value));
    })
    .collect();
    // The per-user scratch directory of the child runners is made under a home beside the fixtures.
    environment.extend(crate::test_support::scratch_environment());
    return environment;
}

/// A repository with a baseline commit (`stable.txt`) and one landed commit that adds `landed.txt`.
fn landed_repository(name: &str, landed_bytes: &str) -> (PathBuf, String) {
    return landed_file(name, "landed.txt", landed_bytes);
}

/// A repository with a baseline commit (`stable.txt`) and one landed commit that adds `file`.
fn landed_file(name: &str, file: &str, landed_bytes: &str) -> (PathBuf, String) {
    let root = fixture(name);
    git(&root, &["init", "--quiet", "--initial-branch=main"]);
    git(&root, &["config", "user.email", "cli-git@example.invalid"]);
    git(&root, &["config", "user.name", "cli-git fixture"]);
    std::fs::write(root.join("stable.txt"), "stable content\n").expect("stable file");
    git(&root, &["add", "stable.txt"]);
    git(&root, &["commit", "--quiet", "-m", "baseline"]);
    std::fs::write(root.join(file), landed_bytes).expect("landed file");
    git(&root, &["add", file]);
    git(&root, &["commit", "--quiet", "-m", "landed"]);
    let landed = String::from_utf8_lossy(&git(&root, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_owned();
    return (root, landed);
}

/// The default configuration with one policy's severity replaced.
fn with_severity(policy: PolicyId, severity: Severity) -> PolicyConfig {
    let mut config: PolicyConfig = PolicyConfig::defaults();
    for setting in &mut config.settings {
        if setting.id == policy {
            setting.severity = severity;
        }
    }
    return config;
}

/// Build a gate request over a fixture repository.
fn request<'request>(
    landed: &'request str,
    global_prefix: &'request [OsString],
    overlay: &'request [(OsString, OsString)],
    config: PolicyConfig,
    controls: Controls,
) -> PostCommitRequest<'request> {
    return PostCommitRequest {
        real_git: Path::new("/usr/bin/git"),
        landed_oid: landed,
        arguments: &[],
        global_prefix,
        overlay,
        environment: overlay,
        config,
        controls,
    };
}

/// The `-C` prefix that makes every query read the fixture repository.
fn prefix_for(root: &Path) -> Vec<OsString> {
    return vec![OsString::from("-C"), root.as_os_str().to_os_string()];
}

/// A landed commit whose committed bytes are complete passes the gate and reaches the backup.
#[test]
fn a_clean_landed_commit_passes_the_gate() {
    let (root, landed) = landed_repository("pc-clean", LANDED_CONTENT);
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let outcome = run_post_commit_gate(&request(
        &landed,
        &prefix,
        &overlay,
        with_severity(PolicyId::FinalNewline, Severity::Error),
        no_controls(),
    ));
    assert!(!outcome.blocked, "{outcome:?}");
    assert!(outcome.events.is_empty(), "{outcome:?}");
    assert_eq!(outcome.oid, landed);
    remove(&root);
}

/// A landed file without a final newline blocks the backup and names the commit in a `commit-landed` event.
#[test]
fn a_blocked_landed_commit_ends_with_an_explicit_commit_landed_event() {
    let (root, landed) = landed_repository("pc-block", "no final newline");
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let outcome = run_post_commit_gate(&request(
        &landed,
        &prefix,
        &overlay,
        with_severity(PolicyId::FinalNewline, Severity::Error),
        no_controls(),
    ));
    assert!(outcome.blocked);
    assert!(
        matches!(outcome.events.first(), Some(PolicyEvent::Finding(_))),
        "{outcome:?}"
    );
    assert_eq!(
        outcome.events.last(),
        Some(&PolicyEvent::CommitLanded {
            oid: landed.clone()
        })
    );
    let rendered = render_policy_events(0, outcome.events.as_slice());
    assert!(
        rendered.contains(&format!(
            "\"message\":\"Commit {landed} remains local; post-commit gate blocked automatic backup.\""
        )),
        "{rendered}"
    );
    assert!(
        rendered.contains("\"outcome\":\"post-commit-blocked\""),
        "{rendered}"
    );
    remove(&root);
}

/// A warning-only result allows the backup, and its finding is still printed.
#[test]
fn a_warning_only_result_allows_the_backup() {
    let (root, landed) = landed_repository("pc-warning", "no final newline");
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let outcome = run_post_commit_gate(&request(
        &landed,
        &prefix,
        &overlay,
        with_severity(PolicyId::FinalNewline, Severity::Warn),
        no_controls(),
    ));
    assert!(!outcome.blocked, "{outcome:?}");
    assert!(
        matches!(outcome.events.as_slice(), [PolicyEvent::Finding(_)]),
        "{outcome:?}"
    );
    remove(&root);
}

/// A full-lifecycle escape of the blocking policy lets the backup run with no events.
#[test]
fn the_full_lifecycle_escape_is_honored() {
    let (root, landed) = landed_repository("pc-escape", "no final newline");
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let mut controls = no_controls();
    controls.escaped = vec![PolicyId::FinalNewline];
    let outcome = run_post_commit_gate(&request(
        &landed,
        &prefix,
        &overlay,
        with_severity(PolicyId::FinalNewline, Severity::Error),
        controls,
    ));
    assert!(!outcome.blocked, "{outcome:?}");
    assert!(outcome.events.is_empty(), "{outcome:?}");
    remove(&root);
}

/// A landed commit Git cannot read keeps the commit and reports the engine failure before the commit-landed event.
#[test]
fn an_engine_failure_retains_the_commit_and_reports_the_failure() {
    let (root, _landed) = landed_repository("pc-failure", LANDED_CONTENT);
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let missing = "f".repeat(40);
    let outcome = run_post_commit_gate(&request(
        &missing,
        &prefix,
        &overlay,
        with_severity(PolicyId::FinalNewline, Severity::Error),
        no_controls(),
    ));
    assert!(outcome.blocked);
    let kinds: Vec<&str> = outcome
        .events
        .iter()
        .map(|event| {
            return match event {
                PolicyEvent::EngineFailure { .. } => "engine-failure",
                PolicyEvent::CommitLanded { .. } => "commit-landed",
                _ => "other",
            };
        })
        .collect();
    assert_eq!(kinds.last(), Some(&"commit-landed"), "{kinds:?}");
    assert!(kinds.contains(&"engine-failure"), "{kinds:?}");
    remove(&root);
}

/// A landed name that is not an object name is an engine failure, never a clean gate.
#[test]
fn a_landed_name_that_is_not_an_object_name_is_an_engine_failure() {
    let (root, _landed) = landed_repository("pc-bad-name", LANDED_CONTENT);
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let outcome = run_post_commit_gate(&request(
        "not-an-oid",
        &prefix,
        &overlay,
        with_severity(PolicyId::FinalNewline, Severity::Error),
        no_controls(),
    ));
    assert!(outcome.blocked);
    assert!(matches!(
        outcome.events.first(),
        Some(PolicyEvent::EngineFailure {
            code: EngineFailureCode::ContentUnavailable,
            ..
        })
    ));
    remove(&root);
}

/// A blocked gate prints its events, exits `2` and never runs the backup.
#[test]
fn a_blocked_gate_exits_two_and_does_not_back_up() {
    let (root, landed) = landed_repository("pc-run-blocked", "no final newline");
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let git_context = PushGit {
        real_git: PathBuf::from("/usr/bin/git"),
        cwd: root.clone(),
        environment: overlay.clone(),
    };
    let mut err: Vec<u8> = Vec::new();
    let outcome = run_post_commit(
        &request(
            &landed,
            &prefix,
            &overlay,
            with_severity(PolicyId::FinalNewline, Severity::Error),
            no_controls(),
        ),
        &git_context,
        &mut err,
    )
    .expect("runs");
    assert_eq!(outcome.exit_code, POST_COMMIT_BLOCKED_EXIT_CODE);
    assert!(outcome.auto_push.is_none());
    let terminal = String::from_utf8_lossy(&err);
    assert!(
        terminal.contains("\"type\":\"commit-landed\""),
        "{terminal}"
    );
    remove(&root);
}

/// A clean gate hands the landed commit to auto-push; a repository without remotes is skipped with exit `0`.
#[test]
fn a_clean_gate_hands_the_commit_to_auto_push() {
    let (root, landed) = landed_repository("pc-run-clean", LANDED_CONTENT);
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let git_context = PushGit {
        real_git: PathBuf::from("/usr/bin/git"),
        cwd: root.clone(),
        environment: overlay.clone(),
    };
    let mut err: Vec<u8> = Vec::new();
    let outcome = run_post_commit(
        &request(
            &landed,
            &prefix,
            &overlay,
            with_severity(PolicyId::FinalNewline, Severity::Error),
            no_controls(),
        ),
        &git_context,
        &mut err,
    )
    .expect("runs");
    assert_eq!(outcome.exit_code, 0);
    let pushed = outcome.auto_push.expect("auto-push ran");
    assert_eq!(pushed.outcome, crate::auto_push::AutoPushOutcome::Skipped);
    assert!(err.is_empty());
    remove(&root);
}

/// The landed-delta candidates list only the paths the landed commit changed, classified as added.
#[test]
fn the_landed_delta_lists_only_the_changed_paths_as_added() {
    let (root, landed) = landed_repository("pc-delta", LANDED_CONTENT);
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let mut facts = git_facts(Path::new("/usr/bin/git"), &prefix, &overlay);
    let commit = parse_object_id(landed.as_bytes()).expect("object name");
    let prepared = facts
        .candidates(&CandidateRequest::Landed(commit))
        .expect("landed candidates");
    let paths: Vec<String> = prepared
        .version
        .candidates()
        .iter()
        .map(|candidate| {
            return String::from_utf8_lossy(&candidate.path).into_owned();
        })
        .collect();
    assert_eq!(paths, vec!["landed.txt".to_owned()], "{paths:?}");
    remove(&root);
}

/// The default configuration with the forbidden-strings policy at error and its options chosen.
fn with_scanner(options: ForbiddenStringsOptions) -> PolicyConfig {
    let mut config: PolicyConfig = with_severity(PolicyId::ForbiddenStrings, Severity::Error);
    config.forbidden_strings = options;
    return config;
}

/// Whether the gate reported a forbidden-strings finding.
fn reports_the_needle(outcome: &PostCommitGate) -> bool {
    return outcome.events.iter().any(|event| {
        return matches!(event, PolicyEvent::Finding(finding) if finding.policy == PolicyId::ForbiddenStrings);
    });
}

/// A repository whose landed commit holds the planted token and whose worktree holds `rules.txt` naming it.
fn landed_needle() -> (PathBuf, String) {
    let (root, landed) = landed_repository("pc-scanner", format!("first\n{}\n", needle()).as_str());
    std::fs::write(root.join("rules.txt"), format!("{}\n", needle())).expect("rules file");
    return (root, landed);
}

/// The configuration's rules file reaches the post-commit scanner: with the built-in rules off, the file is the
/// only rule, and the landed token is found in it.
#[test]
fn the_configured_rules_file_reaches_the_post_commit_scanner() {
    run_isolated(
        "post_commit::tests::the_configured_rules_file_reaches_the_post_commit_scanner",
        "pc-config-rules",
        configured_rules,
    );
}

/// The body of `the_configured_rules_file_reaches_the_post_commit_scanner`.
fn configured_rules(_work: &Path) {
    let (root, landed) = landed_needle();
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let outcome = run_post_commit_gate(&request(
        &landed,
        &prefix,
        &overlay,
        with_scanner(ForbiddenStringsOptions {
            builtin_rules: false,
            rules_file: Some(String::from("rules.txt")),
        }),
        no_controls(),
    ));
    assert!(outcome.blocked, "{outcome:?}");
    assert!(reports_the_needle(&outcome), "{outcome:?}");
    remove(&root);
}

/// The invocation's `FORBIDDEN_STRINGS_RULES` reaches the post-commit scanner when the configuration names no file.
#[test]
fn the_rules_variable_reaches_the_post_commit_scanner() {
    run_isolated(
        "post_commit::tests::the_rules_variable_reaches_the_post_commit_scanner",
        "pc-variable-rules",
        variable_rules,
    );
}

/// The body of `the_rules_variable_reaches_the_post_commit_scanner`.
fn variable_rules(_work: &Path) {
    let (root, landed) = landed_needle();
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let mut environment = fixture_environment();
    environment.push((OsString::from("FORBIDDEN_STRINGS_RULES"), OsString::from("rules.txt")));
    let mut asked = request(
        &landed,
        &prefix,
        &overlay,
        with_scanner(ForbiddenStringsOptions {
            builtin_rules: false,
            rules_file: None,
        }),
        no_controls(),
    );
    asked.environment = environment.as_slice();
    let outcome = run_post_commit_gate(&asked);
    assert!(outcome.blocked, "{outcome:?}");
    assert!(reports_the_needle(&outcome), "{outcome:?}");
    remove(&root);
}

/// The default configuration with `markdown/autofix` at warn, since the policy is off by default.
fn with_markdown() -> PolicyConfig {
    return with_severity(PolicyId::MarkdownAutofix, Severity::Warn);
}

/// A PATH holding `bin` and the system programs, and a home beside `bin` for the linter's scratch directory.
fn path_with(bin: &Path) -> Vec<(OsString, OsString)> {
    let mut path: OsString = bin.as_os_str().to_os_string();
    path.push(":/usr/bin:/bin");
    let home: OsString = bin.parent().unwrap_or(bin).join("home").into_os_string();
    return vec![(OsString::from("PATH"), path), (OsString::from("HOME"), home)];
}

/// A landed Markdown file is checked by the linter found on the invocation's PATH, not on an empty one.
#[test]
fn the_linter_is_found_on_the_invocations_path() {
    let (root, landed) = landed_file("pc-linter", "landed.md", "# T\n\nplain\n");
    let bin: PathBuf = root.join("linter-bin");
    std::fs::create_dir(&bin).expect("linter directory");
    executable(bin.join("monochromatic-lint").as_path(), b"#!/bin/sh\ncat\n");
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let environment = path_with(bin.as_path());
    let mut asked = request(&landed, &prefix, &overlay, with_markdown(), no_controls());
    asked.environment = environment.as_slice();
    let outcome = run_post_commit_gate(&asked);
    assert!(!outcome.blocked, "{outcome:?}");
    assert!(outcome.events.is_empty(), "{outcome:?}");
    remove(&root);
}

/// Without a linter on the invocation's PATH a landed Markdown file is unchecked and blocks the backup, naming the
/// post-commit trigger; the same file, once the configuration excludes it, needs no linter.
#[test]
fn the_markdown_options_reach_the_post_commit_policy() {
    let (root, landed) = landed_file("pc-markdown-options", "landed.md", "# T\n\nplain\n");
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let unchecked = run_post_commit_gate(&request(&landed, &prefix, &overlay, with_markdown(), no_controls()));
    assert!(unchecked.blocked, "{unchecked:?}");
    assert!(
        unchecked.events.iter().any(|event| {
            return matches!(
                event,
                PolicyEvent::EngineFailure {
                    code: EngineFailureCode::PolicyIncomplete,
                    trigger: Some(Trigger::PostCommit),
                    policy: Some(PolicyId::MarkdownAutofix),
                    ..
                }
            );
        }),
        "{unchecked:?}"
    );
    let mut config: PolicyConfig = with_markdown();
    config.markdown_autofix.exclude = vec![String::from("landed.md")];
    let excluded = run_post_commit_gate(&request(&landed, &prefix, &overlay, config, no_controls()));
    assert!(!excluded.blocked, "{excluded:?}");
    assert!(excluded.events.is_empty(), "{excluded:?}");
    remove(&root);
}
