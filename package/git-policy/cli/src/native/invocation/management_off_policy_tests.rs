//! What:
//!  Controls for `git cli-git check|fix --policy <id>` when the selected policy is off,
//! warns or blocks.
//! Why:
//!  A direct run limited to a policy that does not run used to exit 0 with no output,
//! which a CI step cannot tell
//!  from a clean check.
//!  It now stops with exit status 2 and names the key that turns the policy on,
//! while a policy that runs reports
//!  its findings at its own severity.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(['check', '--all', '--policy', 'mono/catalog-specifiers'])).toMatchObject({ code: 2 });
//! ```
#![cfg(unix)]

use super::{exit, plan, with_git};
use crate::action::Action;
use crate::config_file::CONFIG_FILE_NAME;
use crate::real_git::ResolutionInputs;
use crate::test_support::{fixture, remove, repository};
use std::path::{Path, PathBuf};

/// The `config-invalid` event a refusal prints,
/// for the sentence `message` (already JSON text).
fn refusal_event(message: &str) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"{message}\",\"hint\":\"Hint: if you just edited cli-git.config.jsonc, fix the key the message names or revert the edit; an unknown key, a wrong type or an unknown policy id fails before any policy runs.\"}}\n"
    );
}

/// The refusal sentence for an unlisted policy,
/// as JSON text;
/// `on` is the severity that turns it on.
fn unlisted_sentence(name: &str, on: &str) -> String {
    return format!(
        "Policy {name} is off, so --policy {name} would check nothing and exit clean: cli-git.config.jsonc does not list \\\"{name}\\\", and an unlisted policy of this kind is off. To turn it on, set \\\"policies\\\": {{ \\\"{name}\\\": \\\"{on}\\\" }} in cli-git.config.jsonc and run the command again."
    );
}

/// A repository holding one manifest with a literal version,
/// committed nowhere:
/// `check --all` selects the worktree.
fn violating_repository(root: &Path, configuration: Option<&str>) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("package.json"), "{\"dependencies\": {\"zod\": \"^4.6.7\"}}\n").expect("manifest");
    if let Some(text) = configuration {
        std::fs::write(repo.join(CONFIG_FILE_NAME), text).expect("configuration");
    }
    return repo;
}

/// The code,
/// standard output and standard error of a management action.
fn parts(action: Action) -> (i32, String, String) {
    let Action::Exit { code, stdout, stderr } = action else {
        panic!("a management command ends in an exit action");
    };
    return (code, stdout, stderr);
}

/// Every optional policy that the configuration does not list is refused,
/// with the severity that turns it on,
///  whether the policy is old or new,
/// for `check` and for `fix`,
/// and nothing else is printed.
#[test]
fn every_unlisted_optional_policy_is_refused_for_check_and_fix() {
    let root: PathBuf = fixture("off-policy-unlisted");
    let repo: PathBuf = violating_repository(root.as_path(), None);
    let inputs: ResolutionInputs = with_git(root.as_path());
    for (name, on) in [
        ("markdown/autofix", "warn"),
        ("mono/forbidden-root-context", "error"),
        ("mono/dependent-version-bump", "error"),
        ("mono/catalog-specifiers", "error"),
        ("security/forbidden-strings", "error"),
    ] {
        for command in ["check", "fix"] {
            assert_eq!(
                plan(&[command, "--all", "--policy", name], repo.as_path(), &inputs),
                exit(2, refusal_event(unlisted_sentence(name, on).as_str()).as_str(), ""),
                "{command} {name}"
            );
        }
    }
    remove(root.as_path());
}

/// A policy the file lists as `off` is refused with the listed value,
/// and a selection with one off policy among
///  running ones is refused for the off one only.
#[test]
fn a_listed_off_policy_is_refused_and_a_running_one_is_not_named() {
    let root: PathBuf = fixture("off-policy-listed");
    let repo: PathBuf = violating_repository(
        root.as_path(),
        Some(r#"{ "policies": { "mono/catalog-specifiers": "off", "mono/forbidden-root-context": "error" } }"#),
    );
    let inputs: ResolutionInputs = with_git(root.as_path());
    let (code, stdout, stderr) = parts(plan(
        &["check", "--all", "--policy", "mono/forbidden-root-context", "--policy", "mono/catalog-specifiers"],
        repo.as_path(),
        &inputs,
    ));
    assert_eq!((code, stderr.as_str()), (2, ""));
    assert!(stdout.contains("Policy mono/catalog-specifiers is off,"), "{stdout}");
    assert!(stdout.contains("sets policies.\\\"mono/catalog-specifiers\\\" to \\\"off\\\""), "{stdout}");
    assert!(!stdout.contains("forbidden-root-context is off"), "{stdout}");
    assert_eq!(stdout.matches('\n').count(), 1, "one event: {stdout}");
    remove(root.as_path());
}

/// At `warn` the policy runs:
/// its finding is reported at that severity,
/// with the unsafe-warn notice,
/// and the exit is 0.
///  At `error` the finding blocks with exit 1.
#[test]
fn a_selected_policy_that_runs_reports_at_its_own_severity() {
    let root: PathBuf = fixture("off-policy-runs");
    let repo: PathBuf = violating_repository(root.as_path(), Some(r#"{ "policies": { "mono/catalog-specifiers": "warn" } }"#));
    let inputs: ResolutionInputs = with_git(root.as_path());
    let arguments: [&str; 4] = ["check", "--all", "--policy", "mono/catalog-specifiers"];
    let (code, stdout, stderr) = parts(plan(&arguments, repo.as_path(), &inputs));
    assert_eq!((code, stderr.as_str()), (0, ""));
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "{stdout}");
    assert!(lines[0].contains("\"policyId\":\"mono/catalog-specifiers\",\"severity\":\"warn\",\"code\":\"mono/catalog-specifiers/disallowed-specifier\""), "{stdout}");
    assert!(lines[0].contains("\"path\":\"package.json\",\"fix\":\"none\""), "{stdout}");
    assert!(lines[1].contains("\"type\":\"configuration-warning\""), "{stdout}");
    assert!(lines[1].contains("\"code\":\"warn-unsafe\""), "{stdout}");
    std::fs::write(repo.join(CONFIG_FILE_NAME), r#"{ "policies": { "mono/catalog-specifiers": "error" } }"#).expect("error");
    let (blocked, error_stdout, error_stderr) = parts(plan(&arguments, repo.as_path(), &inputs));
    assert_eq!((blocked, error_stderr.as_str()), (1, ""));
    assert!(error_stdout.contains("\"severity\":\"error\",\"code\":\"mono/catalog-specifiers/disallowed-specifier\""), "{error_stdout}");
    assert_eq!(error_stdout.matches('\n').count(), 1, "{error_stdout}");
    remove(root.as_path());
}

/// The policy is on,
/// but a direct fix does not run it (it proposes no correction),
/// so selecting it for a fix would
///  check nothing and exit clean:
/// it is refused with the command that does run it,
/// at every severity that is on.
///  A selection mixing a running policy with a non-running one is refused for the non-running one only.
#[test]
fn a_selected_policy_that_does_not_run_for_the_command_is_refused() {
    let root: PathBuf = fixture("off-policy-not-run");
    let repo: PathBuf = violating_repository(root.as_path(), Some(r#"{ "policies": { "mono/catalog-specifiers": "error" } }"#));
    let inputs: ResolutionInputs = with_git(root.as_path());
    let sentence: &str = "Policy mono/catalog-specifiers does not run for cli-git fix, so --policy mono/catalog-specifiers would check nothing and exit clean. It runs for cli-git check.";
    assert_eq!(
        plan(&["fix", "--all", "--policy", "mono/catalog-specifiers"], repo.as_path(), &inputs),
        exit(2, refusal_event(sentence).as_str(), "")
    );
    let (code, stdout, stderr) = parts(plan(
        &["fix", "--all", "--policy", "final-newline", "--policy", "mono/catalog-specifiers"],
        repo.as_path(),
        &inputs,
    ));
    assert_eq!((code, stderr.as_str()), (2, ""));
    assert!(stdout.contains(sentence), "{stdout}");
    assert!(!stdout.contains("Policy final-newline"), "{stdout}");
    // A policy that runs for neither direct command is told to run the Git command it guards.
    let (guard_code, guard_stdout, _) = parts(plan(&["check", "--all", "--policy", "add-explicit"], repo.as_path(), &inputs));
    assert_eq!(guard_code, 2);
    assert!(guard_stdout.contains("only when the Git command it guards is forwarded"), "{guard_stdout}");
    remove(root.as_path());
}

/// Without `--policy` nothing is refused:
/// a configuration that does not name the policy behaves as it did before the
///  policy shipped,
/// even beside a manifest that would be a finding.
#[test]
fn an_unselected_off_policy_is_skipped_without_a_refusal() {
    let root: PathBuf = fixture("off-policy-skipped");
    let repo: PathBuf = violating_repository(root.as_path(), Some(r#"{ "policies": { "mono/forbidden-root-context": "error" } }"#));
    let inputs: ResolutionInputs = with_git(root.as_path());
    let (code, stdout, stderr) = parts(plan(&["check", "--all"], repo.as_path(), &inputs));
    assert_eq!((code, stderr.as_str()), (0, ""));
    assert!(!stdout.contains("catalog-specifiers"), "{stdout}");
    remove(root.as_path());
}
