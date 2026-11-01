//! Controls for the hooks-off option: where it goes, which commands go without it, and what real Git does with it.

use super::*;
use crate::test_support::{REAL_GIT, executable, fixture, git, isolated_overlay, remove, repository};
use std::path::{Path, PathBuf};

/// The tokens a built command carries.
fn tokens(command: &Command) -> Vec<String> {
    return command.get_args().map(|argument| { return argument.to_string_lossy().into_owned(); }).collect();
}

/// The options as they are spelled on the command line: the hooks path, then one switch per event.
fn prefix() -> Vec<String> {
    let mut tokens: Vec<String> = vec![String::from("-c"), format!("core.hooksPath={NO_HOOKS_DIRECTORY}")];
    for event in switched_off_events() {
        tokens.push(String::from("-c"));
        tokens.push(format!("hook.{event}.enabled=false"));
    }
    return tokens;
}

/// The expected tokens: the prefix, then the caller's.
fn with_prefix(rest: &[&str]) -> Vec<String> {
    let mut tokens: Vec<String> = prefix();
    tokens.extend(rest.iter().map(|token| { return String::from(*token); }));
    return tokens;
}

/// Build the command the wrapper builds for `arguments` once the option is switched on.
fn built(arguments: &[&str]) -> Command {
    let owned: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    let mut command: Command = Command::new(REAL_GIT);
    add_hooks_off_option(&mut command, owned.as_slice());
    command.args(owned.as_slice());
    return command;
}

/// Without the switch the command carries nothing extra, which keeps the other unit controls exact.
#[test]
fn the_option_is_added_only_after_the_process_switched_it_on() {
    let owned: Vec<OsString> = vec![OsString::from("status")];
    let mut command: Command = Command::new(REAL_GIT);
    disable_traditional_hooks(&mut command, owned.as_slice());
    assert!(tokens(&command).is_empty());
}

/// The option goes first, so any `-c core.hooksPath` of the caller comes later and wins.
#[test]
fn the_option_precedes_every_argument_of_the_caller() {
    assert_eq!(tokens(&built(&["status"])), with_prefix(&["status"]));
    assert_eq!(
        tokens(&built(&["-C", "dir", "-c", "core.hooksPath=mine", "push"])),
        with_prefix(&["-C", "dir", "-c", "core.hooksPath=mine", "push"])
    );
    assert_eq!(tokens(&built(&["--version"])), with_prefix(&["--version"]));
}

/// A configuration query reports what the repository configures, so it goes without the option.
#[test]
fn a_configuration_query_goes_without_the_option() {
    assert_eq!(tokens(&built(&["config", "--get", "core.hooksPath"])), vec!["config", "--get", "core.hooksPath"]);
    assert_eq!(
        tokens(&built(&["-C", "dir", "-c", "a=b", "config", "--list"])),
        vec!["-C", "dir", "-c", "a=b", "config", "--list"]
    );
    // A word that only looks like the command is not one.
    assert_eq!(tokens(&built(&["log", "config"])), with_prefix(&["log", "config"]));
}

/// The directory is below a device, so it can never exist.
#[test]
fn the_directory_can_never_exist() {
    assert!(!Path::new(NO_HOOKS_DIRECTORY).exists());
    assert!(std::fs::create_dir(NO_HOOKS_DIRECTORY).is_err());
}

/// Run the built command in `directory` with the isolated overlay.
fn run(directory: &Path, arguments: &[&str]) -> std::process::Output {
    let mut command: Command = built(arguments);
    for (name, value) in isolated_overlay() {
        command.env(name, value);
    }
    return command.current_dir(directory).output().expect("git starts");
}

/// A repository with a failing `pre-commit` hook file, a marker-writing configured hook, and a staged change.
fn hooked_repository(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let repo: PathBuf = repository(root, "repo");
    let file_marker: PathBuf = root.join("file-hook-ran");
    let config_marker: PathBuf = root.join("config-hook-ran");
    executable(
        repo.join(".git/hooks/pre-commit").as_path(),
        format!("#!/bin/sh\ntouch '{}'\nexit 1\n", file_marker.display()).as_bytes(),
    );
    executable(
        root.join("config-hook").as_path(),
        format!("#!/bin/sh\ntouch '{}'\n", config_marker.display()).as_bytes(),
    );
    git(repo.as_path(), &["config", "hook.marker.event", "pre-commit"]);
    git(repo.as_path(), &["config", "hook.marker.command", root.join("config-hook").to_str().expect("UTF-8")]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a.txt");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    return (repo, file_marker, config_marker);
}

/// Neither a hook file nor a configured hook runs with the options; the same repository runs both without them.
#[test]
fn real_git_runs_no_hook_file_and_no_configured_hook() {
    let root: PathBuf = fixture("hooks-off-real");
    let (repo, file_marker, config_marker) = hooked_repository(root.as_path());
    let committed = run(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    assert!(committed.status.success(), "{}", String::from_utf8_lossy(&committed.stderr));
    assert!(!file_marker.exists(), "the hook file must not run");
    assert!(!config_marker.exists(), "the configured hook must not run");
    // Positive control: without the options the same hook file runs and refuses the commit.
    git(repo.as_path(), &["reset", "--quiet", "--soft", "HEAD~1"]);
    let direct = crate::test_support::git_output(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    assert!(!direct.status.success());
    assert!(file_marker.exists(), "real Git without the options runs the hook file");
    assert!(config_marker.exists(), "real Git without the options runs the configured hook");
    remove(root.as_path());
}

/// A `-c core.hooksPath` of the caller comes later and brings its directory back.
#[test]
fn a_hooks_path_of_the_caller_wins() {
    let root: PathBuf = fixture("hooks-off-caller");
    let (repo, file_marker, _config_marker) = hooked_repository(root.as_path());
    let hooks: String = repo.join(".git/hooks").display().to_string();
    let option_of_caller: String = format!("core.hooksPath={hooks}");
    let refused = run(repo.as_path(), &["-c", option_of_caller.as_str(), "commit", "--quiet", "--message=one"]);
    assert!(!refused.status.success());
    assert!(file_marker.exists());
    remove(root.as_path());
}

/// The repository's own `core.hooksPath` is shadowed for a run, and a configuration query still reports it.
#[test]
fn a_configuration_query_still_reports_the_repositorys_hooks_path() {
    let root: PathBuf = fixture("hooks-off-query");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["config", "core.hooksPath", "/somewhere/hooks"]);
    let reported = run(repo.as_path(), &["config", "--get", "core.hooksPath"]);
    assert_eq!(String::from_utf8_lossy(&reported.stdout).trim(), "/somewhere/hooks");
    let listed = run(repo.as_path(), &["config", "--list"]);
    assert!(!String::from_utf8_lossy(&listed.stdout).contains("cli-git-no-hooks"));
    remove(root.as_path());
}

/// Every listed name is a hook event real Git knows: `git hook list` refuses an unknown event name.
#[test]
fn every_listed_event_is_a_hook_event_of_real_git() {
    let root: PathBuf = fixture("hooks-off-events");
    let repo: PathBuf = repository(root.as_path(), "repo");
    for event in HOOK_EVENTS {
        let listed = crate::test_support::git_output(repo.as_path(), &["hook", "list", event]);
        let text: String = String::from_utf8_lossy(&listed.stderr).into_owned();
        assert!(!text.contains("unknown hook event"), "{event}: {text}");
    }
    let unknown = crate::test_support::git_output(repo.as_path(), &["hook", "list", "not-a-hook-event"]);
    assert!(
        String::from_utf8_lossy(&unknown.stderr).contains("unknown hook event"),
        "positive control: Git must refuse a made-up name: {}",
        String::from_utf8_lossy(&unknown.stderr)
    );
    remove(root.as_path());
}

/// A configured hook of any name and any listed event stays off, the wrapper's push event aside.
#[test]
fn a_configured_hook_for_each_switched_off_event_does_not_run() {
    let root: PathBuf = fixture("hooks-off-each-event");
    let repo: PathBuf = repository(root.as_path(), "repo");
    for (index, event) in switched_off_events().enumerate() {
        git(repo.as_path(), &["config", format!("hook.named-{index}.event").as_str(), event]);
        git(repo.as_path(), &["config", format!("hook.named-{index}.command").as_str(), "true"]);
    }
    let listed = run(repo.as_path(), &["config", "--list"]);
    assert!(String::from_utf8_lossy(&listed.stdout).contains("hook.named-0.event"));
    let first = switched_off_events().next().expect("an event");
    let switched = run(repo.as_path(), &["hook", "list", first]);
    let shown: String = String::from_utf8_lossy(&switched.stdout).into_owned();
    assert!(shown.contains("event-disabled") && shown.contains("named-0"), "Git must list the hook as disabled: {shown}");
    // Positive control: without the switches Git lists the same hook as enabled.
    let plain = crate::test_support::git_output(repo.as_path(), &["hook", "list", first]);
    let plain_shown: String = String::from_utf8_lossy(&plain.stdout).into_owned();
    assert!(plain_shown.contains("named-0") && !plain_shown.contains("event-disabled"), "{plain_shown}");
    remove(root.as_path());
}
