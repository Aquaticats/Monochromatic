//! Differential controls for the hook entries of a wrapped commit:
//!  a wrapped `git commit` and a direct one show the same output, run the same hooks and make the same commit, in
//!  every shape of configured hook the user can have, whether or not the wrapper wrote an entry for the event.
//! Why:
//!  A configuration that defines no hook by name has no dispatcher entry and no `post-commit` run, which saves
//!  process starts and synced lock candidates; each shape below is one way the omission could change what a user
//!  sees.
//!  Real Git is the oracle, so the controls run the same commit directly and compare.
//!  Traditional hooks (executable scripts in a hooks directory) are unsupported, so they appear only in the
//!  controls that pin that the wrapper does not run them.

use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, remove_settled, run_commit};
use super::support::{Fixture, Observed, executable, fixture, git, observe, repository, run_direct, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The four commit events whose hooks the wrapper dispatches or runs.
const EVENTS: [&str; 4] = ["pre-commit", "prepare-commit-msg", "commit-msg", "post-commit"];

/// One way a user can have hooks.
#[derive(Clone, Copy, Debug)]
enum Shape {
    /// The default hooks directory holds only Git's `.sample` files.
    NoHooks,
    /// All four events have a hook defined by name in the configuration.
    AllConfigHooks,
    /// Only `commit-msg` has a hook defined by name.
    OneConfigHook,
    /// Hooks defined by name in the configuration, for two events, with command lines instead of scripts.
    ConfigCommands,
    /// `hook.<event>.enabled=false` for two events, with no hooks.
    DisabledWithoutHooks,
    /// Configured hooks in the common configuration, committed from a linked worktree.
    LinkedWorktree,
    /// A configured `pre-commit` hook that fails.
    FailingPreCommit,
    /// Executable traditional hooks for all four events and no configured hook; the wrapper must not run them.
    AllTraditional,
}

/// The script of one hook: it logs its name and argument count, and the message hooks edit the message.
fn script(event: &str, log: &Path, exit_status: u8) -> String {
    let edit: &str = match event {
        "prepare-commit-msg" => "printf 'prepared\\n' >> \"$1\"\n",
        "commit-msg" => "printf '\\nTrailer: commit-msg\\n' >> \"$1\"\n",
        _ => "",
    };
    return format!(
        "#!/bin/sh\nprintf '%s %s\\n' '{event}' \"$#\" >> '{}'\n{edit}exit {exit_status}\n",
        log.display()
    );
}

/// Write an executable hook script for `event` into `directory`.
fn install(directory: &Path, event: &str, log: &Path, exit_status: u8) {
    std::fs::create_dir_all(directory).expect("hooks directory");
    executable(directory.join(event).as_path(), script(event, log, exit_status).as_bytes());
}

/// Define the hook `name` in the configuration so it runs `command` for `event`.
fn configure_hook(fixture: &Fixture, repo: &Path, name: &str, event: &str, command: &str) {
    git(fixture, repo, &["config", format!("hook.{name}.event").as_str(), event]);
    git(fixture, repo, &["config", format!("hook.{name}.command").as_str(), command]);
}

/// Write the scripts of `events` into the repository's `scripts` directory and define a configured hook for each.
fn configure_script_hooks(fixture: &Fixture, repo: &Path, log: &Path, events: &[&str], exit_status: u8) {
    let scripts: PathBuf = repo.join("scripts");
    for event in events {
        install(scripts.as_path(), event, log, exit_status);
        configure_hook(
            fixture,
            repo,
            format!("{event}-script").as_str(),
            event,
            scripts.join(event).to_str().expect("UTF-8"),
        );
    }
}

/// A repository with one tracked file whose worktree copy holds an uncommitted edit.
fn edited_repository(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a");
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    return repo;
}

/// What one side of a comparison needs: the directory to commit in, the hook log, and the global options.
struct Side {
    /// Worktree root the commit runs in.
    commit_in: PathBuf,
    /// File every hook appends to.
    log: PathBuf,
    /// Arguments in front of `commit`.
    global: Vec<String>,
}

/// Build one side of a shape: a repository, its hooks and configuration.
fn build(fixture: &Fixture, side: &str, shape: Shape) -> Side {
    let repo: PathBuf = edited_repository(fixture, side);
    let log: PathBuf = fixture.root.join(format!("{side}.log"));
    let hooks: PathBuf = repo.join(".git").join("hooks");
    let mut commit_in: PathBuf = repo.clone();
    match shape {
        Shape::NoHooks => {}
        Shape::AllConfigHooks => configure_script_hooks(fixture, repo.as_path(), log.as_path(), &EVENTS, 0),
        Shape::OneConfigHook => configure_script_hooks(fixture, repo.as_path(), log.as_path(), &["commit-msg"], 0),
        Shape::ConfigCommands => {
            for (name, event) in [("early", "pre-commit"), ("late", "post-commit")] {
                configure_hook(
                    fixture,
                    repo.as_path(),
                    name,
                    event,
                    format!("echo {event} >> '{}'", log.display()).as_str(),
                );
            }
        }
        Shape::DisabledWithoutHooks => {
            git(fixture, repo.as_path(), &["config", "hook.pre-commit.enabled", "false"]);
            git(fixture, repo.as_path(), &["config", "hook.post-commit.enabled", "false"]);
        }
        Shape::LinkedWorktree => {
            // The base commit the linked worktree starts from carries the edit's baseline; the edit moves with a copy.
            git(fixture, repo.as_path(), &["stash", "--quiet"]);
            let linked: PathBuf = fixture.root.join(format!("{side}-linked"));
            git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("UTF-8")]);
            std::fs::write(linked.join("a.txt"), b"one\ntwo\n").expect("edit in the linked worktree");
            configure_script_hooks(fixture, repo.as_path(), log.as_path(), &EVENTS, 0);
            commit_in = linked;
        }
        Shape::FailingPreCommit => configure_script_hooks(fixture, repo.as_path(), log.as_path(), &["pre-commit"], 1),
        Shape::AllTraditional => {
            for event in EVENTS {
                install(hooks.as_path(), event, log.as_path(), 0);
            }
        }
    }
    return Side { commit_in, log, global: Vec::new() };
}

/// The commit arguments of one side, with the global options in front.
fn commit_arguments(side: &Side) -> Vec<String> {
    let mut arguments: Vec<String> = side.global.clone();
    arguments.extend(["commit", "--quiet", "--message=second", "--", "a.txt"].map(String::from));
    return arguments;
}

/// Standard error with the repository's directory removed, so the two sides' paths compare equal.
fn normalized(observed: &Observed, side: &Side, fixture: &Fixture) -> String {
    let text: String = stderr_of(observed);
    let root: String = format!("{}/", fixture.root.display());
    let repo: String = format!("{}/", side.commit_in.display());
    let without_repo: String = text.replace(repo.as_str(), "");
    // Absolute hooks paths name the side's own directory, which differs between the two sides by its name.
    return without_repo.replace(root.as_str(), "").replace("direct-", "").replace("wrapped-", "");
}

/// What a finished commit left that a user can see: the message, the tree and the files in the worktree.
fn outcome(fixture: &Fixture, side: &Side) -> (String, String, String) {
    return (
        git_text(fixture, side.commit_in.as_path(), &["log", "-1", "--format=%B"]),
        git_text(fixture, side.commit_in.as_path(), &["rev-parse", "HEAD^{tree}"]),
        git_text(fixture, side.commit_in.as_path(), &["status", "--porcelain"]),
    );
}

/// The lines a hook log holds, in order.
fn log_lines(log: &Path) -> Vec<String> {
    return std::fs::read_to_string(log).unwrap_or_default().lines().map(String::from).collect();
}

/// Run the shape directly and wrapped, and return both sides with what each showed.
fn compare(name: &str, shape: Shape) -> (Fixture, Side, Observed, Side, Observed) {
    let fixture: Fixture = fixture(format!("hook-shim-{name}").as_str());
    let direct_side: Side = build(&fixture, "direct", shape);
    let wrapped_side: Side = build(&fixture, "wrapped", shape);
    let direct_arguments: Vec<String> = commit_arguments(&direct_side);
    let wrapped_arguments: Vec<String> = commit_arguments(&wrapped_side);
    let direct: Observed = run_direct(
        &fixture,
        direct_side.commit_in.as_path(),
        direct_arguments.iter().map(String::as_str).collect::<Vec<&str>>().as_slice(),
    );
    let wrapped: Observed = run_commit(
        &fixture,
        wrapped_side.commit_in.as_path(),
        wrapped_arguments.iter().map(String::as_str).collect::<Vec<&str>>().as_slice(),
    );
    return (fixture, direct_side, direct, wrapped_side, wrapped);
}

/// Require the two sides to have shown and done the same thing, and return the hook log both made.
fn require_same(name: &str, shape: Shape) -> Vec<String> {
    let (fixture, direct_side, direct, wrapped_side, wrapped) = compare(name, shape);
    assert_eq!(wrapped.code, direct.code, "{name}: {}", stderr_of(&wrapped));
    assert_eq!(wrapped.stdout, direct.stdout, "{name}");
    assert_eq!(
        normalized(&wrapped, &wrapped_side, &fixture),
        normalized(&direct, &direct_side, &fixture),
        "{name}: stderr"
    );
    assert_eq!(log_lines(wrapped_side.log.as_path()), log_lines(direct_side.log.as_path()), "{name}: hook log");
    assert_eq!(outcome(&fixture, &wrapped_side), outcome(&fixture, &direct_side), "{name}: commit");
    if wrapped.code == Some(0) {
        assert_nothing_left(&fixture, wrapped_side.commit_in.as_path());
    }
    let log: Vec<String> = log_lines(wrapped_side.log.as_path());
    remove_settled(&fixture);
    return log;
}

/// A repository with no hook at all commits as direct Git does, and runs nothing.
#[test]
fn no_hooks_show_what_direct_git_shows() {
    assert_eq!(require_same("none", Shape::NoHooks), Vec::<String>::new());
}

/// Configured hooks for every event run in Git's order with Git's arguments and shape the message.
#[test]
fn configured_hooks_run_as_they_do_directly() {
    let log: Vec<String> = require_same("configured", Shape::AllConfigHooks);
    assert_eq!(
        log,
        vec![
            String::from("pre-commit 0"),
            String::from("prepare-commit-msg 2"),
            String::from("commit-msg 1"),
            String::from("post-commit 0"),
        ]
    );
}

/// One event with a configured hook runs that hook and only that one.
#[test]
fn a_single_event_hook_runs_alone() {
    assert_eq!(require_same("one-event", Shape::OneConfigHook), vec![String::from("commit-msg 1")]);
}

/// Hooks defined by name in the configuration run for their events.
#[test]
fn hooks_defined_in_the_configuration_run() {
    let log: Vec<String> = require_same("config-hooks", Shape::ConfigCommands);
    assert_eq!(log, vec![String::from("pre-commit"), String::from("post-commit")]);
}

/// Disabled events with no hook behave as they do directly.
#[test]
fn disabled_events_without_hooks_show_what_direct_git_shows() {
    assert_eq!(require_same("disabled", Shape::DisabledWithoutHooks), Vec::<String>::new());
}

/// Configured hooks in the common configuration run for a commit made in a linked worktree.
#[test]
fn hooks_run_for_a_commit_in_a_linked_worktree() {
    let log: Vec<String> = require_same("linked-worktree", Shape::LinkedWorktree);
    assert_eq!(log.len(), 4, "{log:?}");
}

/// A failing configured `pre-commit` hook stops the commit with its status, as it does directly.
#[test]
fn a_failing_pre_commit_hook_stops_the_commit_as_it_does_directly() {
    let log: Vec<String> = require_same("failing-pre-commit", Shape::FailingPreCommit);
    assert_eq!(log, vec![String::from("pre-commit 0")]);
}

/// Executable traditional hooks for every event are unsupported: a wrapped commit runs none of them, and still
///  succeeds with the message the commit was given.
#[test]
fn traditional_hooks_do_not_run_in_a_wrapped_commit() {
    let fixture: Fixture = fixture("hook-shim-traditional");
    let side: Side = build(&fixture, "wrapped", Shape::AllTraditional);
    let arguments: Vec<String> = commit_arguments(&side);
    let wrapped: Observed = run_commit(
        &fixture,
        side.commit_in.as_path(),
        arguments.iter().map(String::as_str).collect::<Vec<&str>>().as_slice(),
    );
    assert_eq!(wrapped.code, Some(0), "{}", stderr_of(&wrapped));
    assert_eq!(log_lines(side.log.as_path()), Vec::<String>::new());
    assert_eq!(git_text(&fixture, side.commit_in.as_path(), &["log", "-1", "--format=%B"]).trim(), "second");
    remove_settled(&fixture);
}

/// The trace-file lines that start a Git process, as their command lines.
fn started_commands(trace: &Path) -> Vec<String> {
    return std::fs::read_to_string(trace)
        .unwrap_or_default()
        .lines()
        .filter(|line| { return line.contains("| start "); })
        .filter_map(|line| { return line.rsplit_once("| ").map(|(_head, command)| { return command.to_string(); }); })
        .collect();
}

/// How many started processes ran the Git subcommand `name` as their first argument.
fn starts_of(commands: &[String], name: &str) -> usize {
    return commands
        .iter()
        .filter(|command| { return command.split(' ').nth(1) == Some(name); })
        .count();
}

/// How many started processes were Git's hook runner, whatever options come before `hook run`.
fn hook_runner_starts(commands: &[String]) -> usize {
    return commands.iter().filter(|command| { return command.contains(" hook run "); }).count();
}

/// Run one wrapped commit with Git's process trace on and return the commands Git was started with.
fn traced_commit(name: &str, shape: Shape) -> Vec<String> {
    let fixture: Fixture = fixture(format!("hook-shim-trace-{name}").as_str());
    let side: Side = build(&fixture, "wrapped", shape);
    let trace: PathBuf = fixture.root.join("trace2.txt");
    let arguments: Vec<String> = commit_arguments(&side);
    let mut command = super::commit::wrapped_commit(
        &fixture,
        side.commit_in.as_path(),
        arguments.iter().map(String::as_str).collect::<Vec<&str>>().as_slice(),
    );
    command.env("GIT_TRACE2_PERF", trace.as_path());
    let observed: Observed = observe(&mut command, b"");
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    let commands: Vec<String> = started_commands(trace.as_path());
    remove_settled(&fixture);
    return commands;
}

/// The budget of a commit with no hooks: no hook runner, and one configuration read.
#[test]
fn a_commit_with_no_hooks_starts_no_hook_runner_and_one_config_read() {
    let commands: Vec<String> = traced_commit("none", Shape::NoHooks);
    assert_eq!(hook_runner_starts(commands.as_slice()), 0, "{commands:#?}");
    assert_eq!(starts_of(commands.as_slice(), "config"), 1, "{commands:#?}");
}

/// The budget of a commit with all four configured hooks: one runner for each event, and one configuration read.
#[test]
fn a_commit_with_four_hooks_starts_four_hook_runners() {
    let commands: Vec<String> = traced_commit("four", Shape::AllConfigHooks);
    assert_eq!(hook_runner_starts(commands.as_slice()), 4, "{commands:#?}");
    assert_eq!(starts_of(commands.as_slice(), "config"), 1, "{commands:#?}");
}

/// The budget of a commit with one configured hook: the configuration defines a hook by name, so every commit event
///  keeps its entry (three preparation events, one runner each) and `post-commit` runs once, four runners in all.
#[test]
fn a_commit_with_one_configured_hook_starts_a_runner_for_each_event() {
    let commands: Vec<String> = traced_commit("one", Shape::OneConfigHook);
    assert_eq!(hook_runner_starts(commands.as_slice()), 4, "{commands:#?}");
}

/// The budget of a commit that has only traditional hooks: they are unsupported, so no runner starts for them.
#[test]
fn a_commit_with_only_traditional_hooks_starts_no_hook_runner() {
    let commands: Vec<String> = traced_commit("traditional", Shape::AllTraditional);
    assert_eq!(hook_runner_starts(commands.as_slice()), 0, "{commands:#?}");
    assert_eq!(starts_of(commands.as_slice(), "config"), 1, "{commands:#?}");
}

/// The warning lines on standard error: each is one `configuration-warning` event with the traditional-hooks code.
fn warning_lines(observed: &Observed) -> Vec<String> {
    return stderr_of(observed)
        .lines()
        .filter(|line| { return line.contains("\"code\":\"traditional-hooks-unsupported\""); })
        .map(String::from)
        .collect();
}

/// A repository whose `core.hooksPath` names a directory that holds a hook, ready for a wrapped commit.
fn repository_with_hooks_path(fixture: &Fixture, name: &str, log: &Path) -> PathBuf {
    let repo: PathBuf = edited_repository(fixture, name);
    install(repo.join("custom-hooks").as_path(), "pre-commit", log, 0);
    git(fixture, repo.as_path(), &["config", "core.hooksPath", "custom-hooks"]);
    return repo;
}

/// A configured `core.hooksPath` prints one non-blocking warning, the commit still lands, and the hook there is not run.
#[test]
fn a_configured_hooks_path_warns_once_and_does_not_block() {
    let fixture: Fixture = fixture("hook-warning-set");
    let log: PathBuf = fixture.root.join("traditional.log");
    let repo: PathBuf = repository_with_hooks_path(&fixture, "repo", log.as_path());
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=second", "--", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    let warnings: Vec<String> = warning_lines(&landed);
    assert_eq!(warnings.len(), 1, "{}", stderr_of(&landed));
    let line: &str = warnings[0].as_str();
    assert!(line.contains("\"type\":\"configuration-warning\""), "{line}");
    assert!(line.contains("\"path\":\"custom-hooks\""), "{line}");
    assert!(line.contains("git cli-git doctor"), "{line}");
    assert!(line.contains("unsupported") || line.contains("does not support"), "{line}");
    assert!(!line.contains("core-finding"), "{line}");
    assert_eq!(log_lines(log.as_path()), Vec::<String>::new(), "the traditional hook must not run");
    assert_eq!(git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(), "second");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A commit with the setting given as a global `-c` option warns too, because the snapshot covers the command scope.
#[test]
fn a_hooks_path_given_with_a_global_option_warns() {
    let fixture: Fixture = fixture("hook-warning-option");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let landed: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["-c", "core.hooksPath=option-hooks", "commit", "--quiet", "--message=second", "--", "a.txt"],
    );
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(warning_lines(&landed).len(), 1, "{}", stderr_of(&landed));
    remove_settled(&fixture);
}

/// Without `core.hooksPath` the commit prints no warning.
#[test]
fn an_unset_hooks_path_prints_no_warning() {
    let fixture: Fixture = fixture("hook-warning-unset");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=second", "--", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(warning_lines(&landed), Vec::<String>::new(), "{}", stderr_of(&landed));
    assert!(!stderr_of(&landed).contains("configuration-warning"), "{}", stderr_of(&landed));
    remove_settled(&fixture);
}

/// A commit that a policy blocks still carries the warning, and the warning does not change the exit status.
#[test]
fn a_blocked_commit_carries_the_warning_too() {
    let fixture: Fixture = fixture("hook-warning-blocked");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    git(&fixture, repo.as_path(), &["config", "core.hooksPath", "custom-hooks"]);
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    let blocked: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=x"]);
    assert_eq!(blocked.code, Some(1), "{}", stderr_of(&blocked));
    assert!(stderr_of(&blocked).contains("commit-only/pathspec-required"), "{}", stderr_of(&blocked));
    assert_eq!(warning_lines(&blocked).len(), 1, "{}", stderr_of(&blocked));
    remove_settled(&fixture);
}

/// The warning starts no process: a commit with `core.hooksPath` set starts exactly the Git processes one without it
///  starts, apart from nothing, and none of them is a hook runner or an extra configuration read.
#[test]
fn the_warning_starts_no_extra_process() {
    let plain: Vec<String> = traced_commit("warning-baseline", Shape::NoHooks);
    let fixture: Fixture = fixture("hook-warning-trace");
    let log: PathBuf = fixture.root.join("traditional.log");
    let repo: PathBuf = repository_with_hooks_path(&fixture, "wrapped", log.as_path());
    let trace: PathBuf = fixture.root.join("trace2.txt");
    let mut command = super::commit::wrapped_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=second", "--", "a.txt"],
    );
    command.env("GIT_TRACE2_PERF", trace.as_path());
    let observed: Observed = observe(&mut command, b"");
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    let warned: Vec<String> = started_commands(trace.as_path());
    remove_settled(&fixture);
    assert_eq!(warned.len(), plain.len(), "warned: {warned:#?}\nplain: {plain:#?}");
    assert_eq!(hook_runner_starts(warned.as_slice()), 0, "{warned:#?}");
    assert_eq!(starts_of(warned.as_slice(), "config"), 1, "{warned:#?}");
}
