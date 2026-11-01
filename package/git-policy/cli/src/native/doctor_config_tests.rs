//! Controls for the doctor's configuration findings against real repositories.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};

/// Configured hooks are found, listed one per line, and explained with the ways forward.
#[test]
fn configured_hooks_are_listed_and_explained() {
    let root: PathBuf = fixture("doctor-config-hooks");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["config", "hook.review.event", "post-commit"]);
    git(repo.as_path(), &["config", "hook.review.command", "run it"]);
    let findings: ConfigFindings = config_findings(&git_context(), repo.as_path());
    assert_eq!(findings.hooks.len(), 1);
    assert_eq!(render_hook_lines(&findings), "hook.review (events: post-commit)\n");
    let notes: String = render_notes(&findings);
    assert!(notes.contains("runs no program named by any configuration") && notes.contains("--unset-all"), "{notes}");
    assert!(!findings.retired_hooks_key);
    remove(root.as_path());
}

/// A repository with no hook and no `hooks` key has nothing to report.
#[test]
fn a_clean_repository_reports_nothing() {
    let root: PathBuf = fixture("doctor-config-clean");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let findings: ConfigFindings = config_findings(&git_context(), repo.as_path());
    assert_eq!(render_hook_lines(&findings), "");
    assert_eq!(render_notes(&findings), "");
    remove(root.as_path());
}

/// The retired `hooks` key is accepted and reported as ignored, and an unreadable file is named.
#[test]
fn the_retired_key_and_a_broken_file_are_reported() {
    let root: PathBuf = fixture("doctor-config-retired");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("cli-git.config.jsonc"), "{ \"hooks\": { \"concurrentCommits\": true } }\n").expect("config");
    let findings: ConfigFindings = config_findings(&git_context(), repo.as_path());
    assert!(findings.retired_hooks_key);
    assert!(render_notes(&findings).contains("accepted so older files still load, and ignored"));
    std::fs::write(repo.join("cli-git.config.jsonc"), "{ not json").expect("config");
    let broken: ConfigFindings = config_findings(&git_context(), repo.as_path());
    assert!(!broken.retired_hooks_key);
    assert_eq!(render_notes(&broken), "");
    remove(root.as_path());
}
