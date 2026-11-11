//! Controls for the options every real-Git start carries:
//! every hook switched off,
//! and Git's file-watching monitor switched off
//! for the Git the wrapper starts for its own work.
//! Where the options go,
//! which commands go without them,
//! and what real Git does with them.

use super::*;
use crate::test_support::{REAL_GIT, executable, fixture, git, isolated_overlay, remove, repository};
use std::path::{Path, PathBuf};

/// The tokens a built command carries.
fn tokens(command: &Command) -> Vec<String> {
    return command.get_args().map(|argument| { return argument.to_string_lossy().into_owned(); }).collect();
}

/// The options as they are spelled on the command line:
/// the hooks path,
/// then one switch per event,
/// and the switch that keeps the file-watching monitor from starting,
/// which only the Git the wrapper starts for its own work carries.
fn prefix() -> Vec<String> {
    let mut tokens: Vec<String> = vec![
        String::from("-c"),
        format!("core.hooksPath={NO_HOOKS_DIRECTORY}"),
        String::from("-c"),
        String::from(MONITOR_OFF_SETTING),
    ];
    for event in switched_off_events() {
        tokens.push(String::from("-c"));
        tokens.push(format!("hook.{event}.enabled=false"));
    }
    return tokens;
}

/// The same options for a command the person typed:
/// every hook is off,
/// and the monitor stays the person's setting.
fn person_prefix() -> Vec<String> {
    let mut tokens: Vec<String> = vec![String::from("-c"), format!("core.hooksPath={NO_HOOKS_DIRECTORY}")];
    for event in switched_off_events() {
        tokens.push(String::from("-c"));
        tokens.push(format!("hook.{event}.enabled=false"));
    }
    return tokens;
}

/// The expected tokens:
/// the caller's global options,
/// then the prefix of `owner`,
/// then the subcommand and its arguments.
fn with_prefix_of(globals: &[&str], rest: &[&str], owner: StartOwner) -> Vec<String> {
    let mut tokens: Vec<String> = globals.iter().map(|token| { return String::from(*token); }).collect();
    tokens.extend(if owner == StartOwner::Wrapper { prefix() } else { person_prefix() });
    tokens.extend(rest.iter().map(|token| { return String::from(*token); }));
    return tokens;
}

/// The expected tokens for the wrapper's own Git.
fn with_prefix(globals: &[&str], rest: &[&str]) -> Vec<String> {
    return with_prefix_of(globals, rest, StartOwner::Wrapper);
}

/// Build the command the wrapper builds for `arguments` once the options are switched on,
/// for `owner`.
fn built_as(owner: StartOwner, arguments: &[&str]) -> Command {
    let owned: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    let mut command: Command = Command::new(REAL_GIT);
    command.args(hooks_off_arguments(owned.as_slice(), owner));
    return command;
}

/// Build the command for a start of the wrapper's own Git.
fn built(arguments: &[&str]) -> Command {
    return built_as(StartOwner::Wrapper, arguments);
}

/// Build the command for a command the person typed.
fn built_person(arguments: &[&str]) -> Command {
    return built_as(StartOwner::Person, arguments);
}

/// Without the switch the command carries nothing extra,
/// which keeps the other unit controls exact.
#[test]
fn the_option_is_added_only_after_the_process_switched_it_on() {
    let owned: Vec<OsString> = vec![OsString::from("status")];
    let mut command: Command = Command::new(REAL_GIT);
    add_arguments(&mut command, owned.as_slice(), StartOwner::Wrapper);
    assert_eq!(tokens(&command), vec!["status"]);
}

/// The options follow every global option of the caller and precede the subcommand,
/// so nothing the caller writes among
///  its global options (`-c`,
/// `--config-env`) can switch a hook or the hooks path back on.
#[test]
fn the_options_follow_the_global_options_of_the_caller() {
    assert_eq!(tokens(&built(&["status"])), with_prefix(&[], &["status"]));
    assert_eq!(
        tokens(&built(&["-C", "dir", "-c", "core.hooksPath=mine", "--config-env=hook.post-commit.enabled=V", "push"])),
        with_prefix(&["-C", "dir", "-c", "core.hooksPath=mine", "--config-env=hook.post-commit.enabled=V"], &["push"])
    );
    // A start that names no subcommand has no global options to follow: the options come first.
    assert_eq!(tokens(&built(&["--version"])), with_prefix(&[], &["--version"]));
}

/// A configuration query reports what the repository configures,
/// so it goes without the option.
#[test]
fn a_configuration_query_goes_without_the_option() {
    assert_eq!(tokens(&built(&["config", "--get", "core.hooksPath"])), vec!["config", "--get", "core.hooksPath"]);
    assert_eq!(
        tokens(&built(&["-C", "dir", "-c", "a=b", "config", "--list"])),
        vec!["-C", "dir", "-c", "a=b", "config", "--list"]
    );
    // A word that only looks like the command is not one.
    assert_eq!(tokens(&built(&["log", "config"])), with_prefix(&[], &["log", "config"]));
    // A configuration command that opens an editor starts programs, so it gets the options; `--` ends the options.
    assert_eq!(tokens(&built(&["config", "--edit"])), with_prefix(&[], &["config", "--edit"]));
    assert_eq!(tokens(&built(&["config", "--global", "-e"])), with_prefix(&[], &["config", "--global", "-e"]));
    assert_eq!(tokens(&built(&["config", "--", "-e"])), vec!["config", "--", "-e"]);
}

/// The directory is below a device,
/// so it can never exist.
#[test]
fn the_directory_can_never_exist() {
    assert!(!Path::new(NO_HOOKS_DIRECTORY).exists());
    assert!(std::fs::create_dir(NO_HOOKS_DIRECTORY).is_err());
}

/// Run the built command in `directory` with the isolated overlay.
fn run(directory: &Path, arguments: &[&str]) -> std::process::Output {
    return run_as(StartOwner::Wrapper, directory, arguments);
}

/// Run the command built for `owner` in `directory` with the isolated overlay.
fn run_as(owner: StartOwner, directory: &Path, arguments: &[&str]) -> std::process::Output {
    let mut command: Command = built_as(owner, arguments);
    for (name, value) in isolated_overlay() {
        command.env(name, value);
    }
    return command.current_dir(directory).output().expect("git starts");
}

/// A repository with a failing `pre-commit` hook file,
/// a marker-writing configured hook,
/// and a staged change.
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

/// Neither a hook file nor a configured hook runs with the options;
/// the same repository runs both without them.
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

/// A repository whose `core.fsmonitor` names a program that records each call in `root/monitor-ran`,
/// with one file staged.
fn monitored_repository(root: &Path) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(root, "monitor-repo");
    let marker: PathBuf = root.join("monitor-ran");
    let program: PathBuf = root.join("fsmonitor");
    executable(program.as_path(), format!("#!/bin/sh\necho call >> '{}'\nexit 1\n", marker.display()).as_bytes());
    git(repo.as_path(), &["config", "core.fsmonitor", program.to_str().expect("UTF-8")]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a.txt");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    return (repo, marker);
}

/// The wrapper's own Git never asks the person's file-watching monitor,
/// because the prefix switches it off;
/// the same command without the options runs the program (the positive control),
/// so an empty record proves the option and not a setting that never fires.
#[test]
fn the_wrappers_own_start_never_runs_the_core_fsmonitor_program() {
    let root: PathBuf = fixture("hooks-off-monitor-own");
    let (repo, marker) = monitored_repository(root.as_path());
    std::fs::remove_file(marker.as_path()).expect("clear the record the setup wrote");
    let switched = run(repo.as_path(), &["status", "--short"]);
    assert!(switched.status.success(), "{}", String::from_utf8_lossy(&switched.stderr));
    assert!(!marker.exists(), "the wrapper's own Git must not run the configured monitor program");
    let direct = crate::test_support::git_output(repo.as_path(), &["status", "--short"]);
    assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
    assert!(marker.exists(), "real Git without the options must run the configured monitor program");
    remove(root.as_path());
}

/// `core.fsmonitor` is the person's setting for the command the person typed:
/// a forwarded start carries the hook switches and no monitor switch,
/// so Git runs the configured program.
#[test]
fn a_forwarded_start_runs_the_core_fsmonitor_program() {
    let root: PathBuf = fixture("hooks-off-monitor-forward");
    let (repo, marker) = monitored_repository(root.as_path());
    std::fs::remove_file(marker.as_path()).expect("clear the record the setup wrote");
    // The tokens say it without Git: the person's start has no monitor switch, and the wrapper's has one.
    assert!(!tokens(&built_person(&["status"])).contains(&String::from(MONITOR_OFF_SETTING)));
    assert!(tokens(&built(&["status"])).contains(&String::from(MONITOR_OFF_SETTING)));
    let forwarded = run_as(StartOwner::Person, repo.as_path(), &["status", "--short"]);
    assert!(forwarded.status.success(), "{}", String::from_utf8_lossy(&forwarded.stderr));
    assert!(marker.exists(), "a forwarded start must run the configured monitor program");
    remove(root.as_path());
}

/// A `-c core.hooksPath` or `-c hook.<event>.enabled=true` of the caller cannot bring a hook back:
/// the options follow it.
#[test]
fn the_callers_own_options_cannot_switch_a_hook_back_on() {
    let root: PathBuf = fixture("hooks-off-caller");
    let (repo, file_marker, config_marker) = hooked_repository(root.as_path());
    let hooks: String = repo.join(".git/hooks").display().to_string();
    let path_of_caller: String = format!("core.hooksPath={hooks}");
    let committed = run(
        repo.as_path(),
        &["-c", path_of_caller.as_str(), "-c", "hook.pre-commit.enabled=true", "commit", "--quiet", "--message=one"],
    );
    assert!(committed.status.success(), "{}", String::from_utf8_lossy(&committed.stderr));
    assert!(!file_marker.exists(), "the hook file ran");
    assert!(!config_marker.exists(), "the configured hook ran");
    remove(root.as_path());
}

/// The repository's own `core.hooksPath` is shadowed for a run,
/// and a configuration query still reports it.
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

/// Every listed name is a hook event real Git knows:
/// `git hook list` refuses an unknown event name.
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

/// A configured hook for each switched-off event is listed by Git as disabled when the options are given,
/// and as enabled
///  without them (the positive control),
/// event by event.
#[test]
fn a_configured_hook_for_each_switched_off_event_does_not_run() {
    let root: PathBuf = fixture("hooks-off-each-event");
    let repo: PathBuf = repository(root.as_path(), "repo");
    for (index, event) in switched_off_events().enumerate() {
        git(repo.as_path(), &["config", format!("hook.named-{index}.event").as_str(), event]);
        git(repo.as_path(), &["config", format!("hook.named-{index}.command").as_str(), "true"]);
    }
    for (index, event) in switched_off_events().enumerate() {
        let name: String = format!("named-{index}");
        let switched = run(repo.as_path(), &["hook", "list", event]);
        let shown: String = String::from_utf8_lossy(&switched.stdout).into_owned();
        assert!(shown.contains("event-disabled") && shown.contains(name.as_str()), "{event}: {shown}");
        let plain = crate::test_support::git_output(repo.as_path(), &["hook", "list", event]);
        let plain_shown: String = String::from_utf8_lossy(&plain.stdout).into_owned();
        assert!(plain_shown.contains(name.as_str()) && !plain_shown.contains("event-disabled"), "{event}: {plain_shown}");
    }
    remove(root.as_path());
}

/// Whether `name` is spelled like a hook event:
/// lowercase letters,
/// digits and hyphens.
fn is_event_spelling(name: &str) -> bool {
    return !name.is_empty()
        && name.chars().all(|character| {
            return character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-';
        });
}

/// The events the rendered manual page lists:
/// each is a heading indented by three spaces.
/// `None` when `man` cannot
///  render the page.
fn events_from_manual() -> Option<Vec<String>> {
    let output = Command::new("man")
        .args(["--pager=cat", "githooks"])
        .env("MANWIDTH", "200")
        .env("MANPAGER", "cat")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text: String = String::from_utf8_lossy(&output.stdout).into_owned();
    return Some(
        text.lines()
            .filter_map(|line| { return line.strip_prefix("   "); })
            .filter(|name| { return is_event_spelling(name); })
            .map(String::from)
            .collect(),
    );
}

/// The events the AsciiDoc source of the page lists:
/// each is a line of event spelling followed by a line of tildes.
///  `None` when no installed copy of the source exists.
fn events_from_source() -> Option<Vec<String>> {
    // A host's own documentation source. The gate image installs the distribution's Git, which ships the rendered page
    //  and not this source, so there the rendered page (through `man-db`) answers instead.
    let directories: [&str; 2] = ["/usr/share/doc/git", "/usr/share/doc/git-doc"];
    let text: String = directories
        .iter()
        .find_map(|directory| { return std::fs::read_to_string(Path::new(directory).join("githooks.adoc")).ok(); })?;
    let lines: Vec<&str> = text.lines().collect();
    return Some(
        lines
            .windows(2)
            .filter(|pair| {
                return is_event_spelling(pair[0]) && !pair[1].is_empty() && pair[1].chars().all(|c| { return c == '~'; });
            })
            .map(|pair| { return String::from(pair[0]); })
            .collect(),
    );
}

/// The hook events the host Git documents in `githooks(5)`,
/// from the installed AsciiDoc source or the rendered page.
///  The binary lists its hook names in generated tables whose layout is not a contract,
/// and `git hook list` can only
///  confirm a name it is given,
/// never find a missing one,
/// so the documentation is the independent source.
///  The test fails when neither copy is installed,
/// because a guard that skips on a host that lacks its source checks
///  nothing there;
/// install Git's documentation (the `git-doc` or `git-man` package) on such a host.
fn documented_events() -> Vec<String> {
    let found: Option<Vec<String>> = events_from_source().or_else(events_from_manual);
    return found.expect(
        "no copy of the githooks documentation is installed (looked for githooks.adoc under /usr/share/doc/git and \
         a rendered page through `man githooks`), so the list of hook events cannot be checked for completeness",
    );
}

/// Every hook event the host Git documents is in `HOOK_EVENTS`,
/// so a configured hook for it is switched off.
///  The positive control is a name every manual lists.
#[test]
fn every_documented_hook_event_is_listed() {
    let documented: Vec<String> = documented_events();
    assert!(documented.iter().any(|name| { return name == "pre-commit"; }), "the manual was not read: {documented:?}");
    let missing: Vec<&String> = documented.iter().filter(|name| { return !HOOK_EVENTS.contains(&name.as_str()); }).collect();
    assert!(missing.is_empty(), "documented by this Git but not switched off: {missing:?}");
}

/// Both sources agree on the events,
/// so a layout change in one is caught by the other.
#[test]
fn the_source_and_the_rendered_page_list_the_same_events() {
    let (Some(source), Some(manual)) = (events_from_source(), events_from_manual()) else {
        return;
    };
    let mut from_source: Vec<String> = source;
    let mut from_manual: Vec<String> = manual;
    from_source.sort();
    from_manual.sort();
    assert_eq!(from_source, from_manual);
}
