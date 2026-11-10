//! Controls for the post-commit gate against real repositories.
//! The corpus is `package/git-policy/cli/src/policy-engine/post-commit-lifecycle.unit.test.ts` and
//! `package/git-policy/cli/src/policy-engine/post-commit-facts.unit.test.ts`,
//! both deleted on 2026-10-13 in `642b6d72cb` with the TypeScript wrapper,
//! so `git show 642b6d72cb^` holds each.
//! The TypeScript tests' plugins have no native equivalent,
//! so the same behaviors are driven with the built-in
//! final-newline and forbidden-strings policies at fixed severities.

use super::*;
use crate::policy_events::render_policy_events;
use crate::policy_registry::{PolicyId, Severity};
use crate::repository_facts::RepositoryFacts;
use crate::config_schema::{ForbiddenStringsOptions, RulesFileEntry};
use crate::scanner_test_support::{needle, run_isolated};
use crate::test_support::{fixture, git, remove};
use crate::wrapper_controls::no_controls;
use std::path::PathBuf;

/// Final-newline content of the landed file:
/// a complete line.
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
    git(&root, &["config", "user.name", "monochromatic-git-policy-cli fixture"]);
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

/// What:
///  A gate that blocks says which check refused the commit and what it found,
/// in one line.
/// Why:
///   The next push of the branch stops before that commit and every command says why,
/// so the words of the verdict are what
///  the person reads instead of a backup that did not happen;
/// a policy id alone,
/// or a message with a line break in it,
/// would
///  leave them guessing or split one notice into two lines.
#[test]
fn a_blocked_gate_words_its_reason_as_one_line() {
    let (root, landed) = landed_repository("pc-reason", "no final newline");
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let outcome = run_post_commit_gate(&request(
        &landed,
        &prefix,
        &overlay,
        with_severity(PolicyId::FinalNewline, Severity::Error),
        no_controls(),
    ));
    assert!(outcome.blocked, "{outcome:?}");
    let reason: String = blocked_reason(outcome.events.as_slice());
    assert!(reason.starts_with("final-newline: "), "{reason}");
    assert!(!reason.contains('\n'), "the verdict is one line: {reason}");
    // The control: a gate that passed says nothing to hold the branch back with.
    let clean = run_post_commit_gate(&request(
        &landed,
        &prefix,
        &overlay,
        PolicyConfig::defaults(),
        no_controls(),
    ));
    assert!(!clean.blocked, "{clean:?}");
    assert_eq!(blocked_reason(clean.events.as_slice()), String::new());
    remove(&root);
}

/// A warning-only result allows the backup,
/// and its finding is still printed.
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
    // The words the verdict keeps for the push and the notice name the check that refused the commit,
    // in one line.
    let reason: String = blocked_reason(outcome.events.as_slice());
    assert!(reason.starts_with("final-newline: "), "{reason}");
    assert!(!reason.contains('\n'), "one line: {reason}");
    remove(&root);
}

/// What:
///  A failure no policy is responsible for is named with its own message,
/// and one that belongs to a policy names the policy first.
/// Why:
///   The verdict is what the person reads for a push that did not happen,
/// so an empty policy name would read as a
///  refusal by nothing.
#[test]
fn a_reason_names_the_policy_that_failed_and_nothing_when_none_did() {
    let without_policy = [PolicyEvent::EngineFailure {
        code: EngineFailureCode::ContentUnavailable,
        hint: crate::policy_events::EngineFailureHint::OfCode,
        message: String::from("the repository's top level cannot be read"),
        trigger: Some(Trigger::PostCommit),
        policy: None,
        path: None,
    }];
    assert_eq!(blocked_reason(without_policy.as_slice()), "the repository's top level cannot be read");
    let with_policy = [PolicyEvent::EngineFailure {
        code: EngineFailureCode::ContentUnavailable,
        hint: crate::policy_events::EngineFailureHint::OfCode,
        message: String::from("the rules file cannot be read"),
        trigger: Some(Trigger::PostCommit),
        policy: Some(PolicyId::ForbiddenStrings),
        path: None,
    }];
    assert_eq!(blocked_reason(with_policy.as_slice()), "security/forbidden-strings: the rules file cannot be read");
}

/// A landed name that is not an object name is an engine failure,
/// never a clean gate.
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

/// The landed-delta candidates list only the paths the landed commit changed,
/// classified as added.
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

/// The configuration's rules file reaches the post-commit scanner:
/// with the built-in rules off,
/// the file is the
/// only rule,
/// and the landed token is found in it.
#[test]
fn the_configured_rules_file_reaches_the_post_commit_scanner() {
    run_isolated(
        "commit::post_commit::tests::the_configured_rules_file_reaches_the_post_commit_scanner",
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
            rules_files: Some(vec![RulesFileEntry { path: String::from("rules.txt"), optional: false }]),
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
        "commit::post_commit::tests::the_rules_variable_reaches_the_post_commit_scanner",
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
            rules_files: None,
        }),
        no_controls(),
    );
    asked.environment = environment.as_slice();
    let outcome = run_post_commit_gate(&asked);
    assert!(outcome.blocked, "{outcome:?}");
    assert!(reports_the_needle(&outcome), "{outcome:?}");
    remove(&root);
}

/// The default configuration with `markdown/autofix` at warn,
/// since the policy is off by default.
fn with_markdown() -> PolicyConfig {
    return with_severity(PolicyId::MarkdownAutofix, Severity::Warn);
}

/// A landed Markdown file is checked in-process after landing:
/// nothing is looked up on the invocation's PATH,
/// and the landed bytes' stale object URL comes back as the policy's finding,
/// which only a run of the linter
/// can produce.
#[test]
fn a_landed_markdown_file_is_checked_in_process() {
    let stale: String = format!("![shot](https://lfs.example.com/org/repo/{}/pkg/asset/gone.png)\n", "b".repeat(64));
    let (root, landed) = landed_file("pc-linter", "landed.md", stale.as_str());
    // The linter reads the endpoint from the worktree it runs in.
    std::fs::write(root.join(".lfsconfig"), b"[lfs]\n\turl = https://lfs.example.com/org/repo\n").expect("lfsconfig");
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let mut asked = request(&landed, &prefix, &overlay, with_markdown(), no_controls());
    asked.environment = &[];
    let outcome = run_post_commit_gate(&asked);
    assert!(!outcome.blocked, "a warning does not block: {outcome:?}");
    assert!(
        outcome.events.iter().any(|event| {
            return matches!(
                event,
                PolicyEvent::Finding(finding)
                    if finding.policy == PolicyId::MarkdownAutofix
                        && finding.trigger == Trigger::PostCommit
                        && finding.code == crate::policy_markdown::MARKDOWN_VIOLATION_CODE
            );
        }),
        "{outcome:?}"
    );
    remove(&root);
}

/// The policy's options reach the post-commit policy:
/// an exclude pattern that does not compile leaves the landed
/// file unchecked and blocks the backup,
/// naming the post-commit trigger;
/// the same file,
/// once the configuration
/// excludes it with a valid pattern,
/// is not checked and passes.
#[test]
fn the_markdown_options_reach_the_post_commit_policy() {
    let (root, landed) = landed_file("pc-markdown-options", "landed.md", "# T\n\nplain\n");
    let prefix = prefix_for(&root);
    let overlay = fixture_environment();
    let mut broken: PolicyConfig = with_markdown();
    broken.markdown_autofix.exclude = vec![String::from(UNCOMPILABLE_PATTERN)];
    let unchecked = run_post_commit_gate(&request(&landed, &prefix, &overlay, broken, no_controls()));
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

/// A trailing escape,
/// which the glob compiler refuses and Git matches nothing with.
/// A literal brace and a character range that runs backwards are no longer such patterns:
/// each is read the way Git reads it (`markdown_exclude`,
/// `monochromatic_lint::gitignore_pattern`).
const UNCOMPILABLE_PATTERN: &str = "a\\";
