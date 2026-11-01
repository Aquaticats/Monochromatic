//! External hooks through the built executable: none runs, the configuration that defines them is reported loudly, and
//!  the declared post-commit action does what a `post-commit` hook did.
//! Why:
//!  The wrapper supports no external hook program at all.
//!  Git never starts the wrapper for a hook, the wrapper starts no hook program, and the one behavior this repository
//!  used a hook for is a native action declared in `cli-git.config.jsonc`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git config hook.x.event pre-commit; git commit: the hook's marker file never appears and a warning explains why
//! ```

use super::commit::{remove_settled, run_commit};
use super::support::{Fixture, executable, fixture, git, observe, repository, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A repository with `a.txt` committed and edited in the worktree.
fn edited_repository(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit");
    return repo;
}

/// Define the hook `name` for `event` so it appends the event to `marker`.
fn configure_marker_hook(fixture: &Fixture, repo: &Path, name: &str, event: &str, marker: &Path) {
    let script: PathBuf = fixture.root.join(format!("hook-{name}"));
    executable(script.as_path(), format!("#!/bin/sh\necho {event} >> '{}'\nexit 9\n", marker.display()).as_bytes());
    git(fixture, repo, &["config", format!("hook.{name}.event").as_str(), event]);
    git(fixture, repo, &["config", format!("hook.{name}.command").as_str(), script.to_str().expect("UTF-8")]);
}

/// A hook for every commit event never runs, the commit lands with the message it was given, and standard error
/// carries one `configuration-warning` naming the hooks and the ways forward.
#[test]
fn configured_hooks_for_the_commit_events_never_run_and_are_reported() {
    let fixture: Fixture = fixture("external-commit-events");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let marker: PathBuf = fixture.root.join("marker");
    for event in ["pre-commit", "prepare-commit-msg", "commit-msg", "post-commit"] {
        configure_marker_hook(&fixture, repo.as_path(), format!("on-{event}").as_str(), event, marker.as_path());
    }
    let outcome = run_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=second", "--", "a.txt"]);
    assert_eq!(outcome.code, Some(0), "{}", stderr_of(&outcome));
    assert!(!marker.exists(), "a hook ran: {:?}", std::fs::read_to_string(&marker));
    let log: String = String::from_utf8_lossy(
        &git(&fixture, repo.as_path(), &["log", "-1", "--format=%B"]).stdout,
    )
    .trim()
    .to_string();
    assert_eq!(log, "second");
    let stderr: String = stderr_of(&outcome);
    assert!(stderr.contains("external-hooks-unsupported"), "{stderr}");
    for expected in ["hook.on-pre-commit", "hook.on-commit-msg", "hook.on-post-commit", "postCommit.actions", "git cli-git doctor"] {
        assert!(stderr.contains(expected), "{expected} in {stderr}");
    }
    remove_settled(&fixture);
}

/// A hook configured in the global file is switched off as well, and a repository with no hook gets no warning.
#[test]
fn a_global_hook_never_runs_and_a_clean_repository_is_not_warned() {
    let fixture: Fixture = fixture("external-global");
    let clean: PathBuf = edited_repository(&fixture, "clean");
    let quiet = run_commit(&fixture, clean.as_path(), &["commit", "--quiet", "--message=second", "--", "a.txt"]);
    assert_eq!(quiet.code, Some(0), "{}", stderr_of(&quiet));
    assert!(!stderr_of(&quiet).contains("external-hooks-unsupported"), "{}", stderr_of(&quiet));
    let repo: PathBuf = edited_repository(&fixture, "other");
    let marker: PathBuf = fixture.root.join("global-marker");
    let script: PathBuf = fixture.root.join("global-hook");
    executable(script.as_path(), format!("#!/bin/sh\necho global >> '{}'\n", marker.display()).as_bytes());
    let global: PathBuf = fixture.root.join("global-config");
    std::fs::write(
        &global,
        format!("[hook \"everywhere\"]\n\tevent = pre-commit\n\tcommand = {}\n", script.display()),
    )
    .expect("global config");
    let mut command: Command = super::commit::wrapped_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=second", "--", "a.txt"],
    );
    command.env("GIT_CONFIG_GLOBAL", &global);
    let outcome = observe(&mut command, b"");
    assert_eq!(outcome.code, Some(0), "{}", stderr_of(&outcome));
    assert!(!marker.exists(), "a global hook ran");
    assert!(stderr_of(&outcome).contains("hook.everywhere"), "{}", stderr_of(&outcome));
    remove_settled(&fixture);
}

/// A forwarded command runs no configured hook either: a `post-checkout` hook stays silent on `checkout`.
#[test]
fn a_forwarded_command_runs_no_configured_hook() {
    let fixture: Fixture = fixture("external-forwarded");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    git(&fixture, repo.as_path(), &["stash", "--quiet"]);
    git(&fixture, repo.as_path(), &["branch", "feature"]);
    let marker: PathBuf = fixture.root.join("marker");
    configure_marker_hook(&fixture, repo.as_path(), "after-checkout", "post-checkout", marker.as_path());
    let outcome = run_commit(&fixture, repo.as_path(), &["checkout", "--quiet", "feature"]);
    assert_eq!(outcome.code, Some(0), "{}", stderr_of(&outcome));
    assert!(!marker.exists(), "a hook ran on a forwarded command");
    remove_settled(&fixture);
}

/// The declared action runs once after the commit lands, in the top level, with its fields filled in, and sees the
/// landed commit as `HEAD`.
#[test]
fn a_declared_action_runs_after_the_commit_lands() {
    let fixture: Fixture = fixture("external-action");
    let repo: PathBuf = std::fs::canonicalize(edited_repository(&fixture, "repo")).expect("canonical");
    let record: PathBuf = fixture.root.join("action.log");
    let program: PathBuf = fixture.root.join("action-program");
    executable(
        program.as_path(),
        format!("#!/bin/sh\n{{ echo \"args=$*\"; echo \"cwd=$(pwd -P)\"; echo \"head=$(/usr/bin/git rev-parse HEAD)\"; }} >> '{}'\n", record.display()).as_bytes(),
    );
    std::fs::write(
        repo.join("cli-git.config.jsonc"),
        format!(
            "{{ \"postCommit\": {{ \"actions\": [{{ \"name\": \"record\", \"argv\": [\"{}\", \"--unit=u-{{shortHead}}\", \"{{head}}\", \"{{topLevel}}\", \"{{env:HOME}}\"] }}] }} }}\n",
            program.display()
        ),
    )
    .expect("config");
    let outcome = run_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=second", "--", "a.txt"]);
    assert_eq!(outcome.code, Some(0), "{}", stderr_of(&outcome));
    let head: String = String::from_utf8_lossy(&git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout).trim().to_string();
    let seen: String = std::fs::read_to_string(&record).expect("the action ran");
    let short: String = head.chars().take(git_policy_cli::post_commit_template::SHORT_HEAD_LENGTH).collect();
    let home: PathBuf = fixture.root.join("home");
    assert!(seen.contains(&format!("args=--unit=u-{short} {head} {} {}", repo.display(), home.display())), "{seen}");
    assert!(seen.contains(&format!("cwd={}", repo.display())), "{seen}");
    assert!(seen.contains(&format!("head={head}")), "{seen}");
    assert_eq!(seen.matches("cwd=").count(), 1, "the action runs once");
    remove_settled(&fixture);
}

/// An action that fails never fails the commit: the commit exits 0 and a notice names the action.
#[test]
fn a_failing_action_never_fails_the_commit() {
    let fixture: Fixture = fixture("external-action-fails");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let failing: PathBuf = fixture.root.join("fails");
    executable(failing.as_path(), b"#!/bin/sh\nexit 5\n");
    std::fs::write(
        repo.join("cli-git.config.jsonc"),
        format!("{{ \"postCommit\": {{ \"actions\": [{{ \"name\": \"broken\", \"argv\": [\"{}\"] }}] }} }}\n", failing.display()),
    )
    .expect("config");
    let outcome = run_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=second", "--", "a.txt"]);
    assert_eq!(outcome.code, Some(0), "{}", stderr_of(&outcome));
    let stderr: String = stderr_of(&outcome);
    assert!(stderr.contains("post-commit action broken exited with status 5") && stderr.contains("stays landed"), "{stderr}");
    remove_settled(&fixture);
}

/// A configuration with an unknown field in an action stops the commit with the field list, before anything is done.
#[test]
fn an_unknown_field_in_an_action_is_a_configuration_failure() {
    let fixture: Fixture = fixture("external-action-bad-field");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(
        repo.join("cli-git.config.jsonc"),
        "{ \"postCommit\": { \"actions\": [{ \"name\": \"x\", \"argv\": [\"true\", \"{commit}\"] }] } }\n",
    )
    .expect("config");
    let outcome = run_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=second", "--", "a.txt"]);
    assert_ne!(outcome.code, Some(0));
    let stderr: String = stderr_of(&outcome);
    assert!(stderr.contains("{commit}") && stderr.contains("{shortHead}"), "{stderr}");
    assert_eq!(
        String::from_utf8_lossy(&git(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).stdout).trim(),
        "base",
        "nothing landed"
    );
    remove_settled(&fixture);
}

/// The retired `hooks.concurrentCommits` key still loads: the commit lands and nothing complains.
#[test]
fn the_retired_hooks_key_still_loads_in_a_commit() {
    let fixture: Fixture = fixture("external-retired-key");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("cli-git.config.jsonc"), "{ \"hooks\": { \"concurrentCommits\": true } }\n").expect("config");
    let outcome = run_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=second", "--", "a.txt"]);
    assert_eq!(outcome.code, Some(0), "{}", stderr_of(&outcome));
    remove_settled(&fixture);
}

/// `git cli-git doctor` lists the configured hooks, exits 1, and names the ways forward; the retired key is noted.
#[test]
fn doctor_lists_configured_hooks_and_notes_the_retired_key() {
    let fixture: Fixture = fixture("external-doctor");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let marker: PathBuf = fixture.root.join("marker");
    configure_marker_hook(&fixture, repo.as_path(), "review", "post-commit", marker.as_path());
    std::fs::write(repo.join("cli-git.config.jsonc"), "{ \"hooks\": { \"concurrentCommits\": false } }\n").expect("config");
    let outcome = run_commit(&fixture, repo.as_path(), &["cli-git", "doctor"]);
    assert_eq!(outcome.code, Some(1), "{}", stderr_of(&outcome));
    assert!(String::from_utf8_lossy(&outcome.stdout).contains("hook.review (events: post-commit)"), "{:?}", outcome.stdout);
    let stderr: String = stderr_of(&outcome);
    assert!(stderr.contains("postCommit.actions") && stderr.contains("accepted so older files still load, and ignored"), "{stderr}");
    assert!(!marker.exists());
    remove_settled(&fixture);
}

/// A start name that used to be a hook entry's name behaves as `git`: nothing dispatches on the name the wrapper was
/// started under.
#[test]
fn a_start_name_that_used_to_be_an_entry_behaves_as_git() {
    let fixture: Fixture = fixture("external-other-names");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let hooks: PathBuf = repo.join(".git/cli-git-transactions/0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10/hooks");
    std::fs::create_dir_all(&hooks).expect("hooks directory");
    let elsewhere: PathBuf = fixture.root.join("elsewhere");
    std::fs::create_dir(&elsewhere).expect("elsewhere");
    let mut starts: Vec<PathBuf> = Vec::new();
    for event in git_policy_cli::hooks_off::HOOK_EVENTS {
        for directory in [hooks.as_path(), elsewhere.as_path()] {
            let start: PathBuf = directory.join(event);
            std::os::unix::fs::symlink(&fixture.wrapper, &start).expect("link");
            starts.push(start);
        }
    }
    for start in starts {
        let mut command: Command = super::support::bounded(
            &fixture,
            start.as_path(),
            super::support::wrapper_first_path(&fixture).as_os_str(),
        );
        command.arg("--version").current_dir(&repo);
        let outcome = observe(&mut command, b"");
        assert_eq!(outcome.code, Some(0), "{}", String::from_utf8_lossy(&outcome.stderr));
        assert!(String::from_utf8_lossy(&outcome.stdout).starts_with("git version "), "{start:?}");
    }
    remove_settled(&fixture);
}
