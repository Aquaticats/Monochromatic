//! Differential controls for the hook entries of a wrapped commit:
//!  a wrapped `git commit` and a direct one show the same output, run the same hooks and make the same commit, in
//!  every shape of hook the user can have, whether or not the wrapper wrote an entry for the event.
//! Why:
//!  An event with no user hook has no dispatcher entry and no `post-commit` run, which saves process starts and
//!  synced lock candidates; each shape below is one way the omission could change what a user sees.
//!  Real Git is the oracle, so the controls run the same commit directly and compare.

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
    /// All four events have an executable hook.
    AllExecutable,
    /// Only `commit-msg` has a hook.
    OneEvent,
    /// All four events have a file that is not executable, so Git prints a hint for each.
    NotExecutable,
    /// A dangling link for `post-commit` and a working link for `pre-commit`.
    Symlinks,
    /// Hooks defined by name in the configuration, for two events.
    ConfigHooks,
    /// A relative `core.hooksPath` holding hooks, while the default directory also holds hooks that must not run.
    RelativeHooksPath,
    /// An absolute `core.hooksPath` holding one hook.
    AbsoluteHooksPath,
    /// A `core.hooksPath` naming an empty directory, while the default directory holds hooks that must not run.
    EmptyHooksPath,
    /// A `core.hooksPath` naming a directory that does not exist.
    MissingHooksPath,
    /// The hooks path comes from a global `-c` option.
    HooksPathFromGlobalOption,
    /// `hook.<event>.enabled=false` for two events, with no hooks.
    DisabledWithoutHooks,
    /// Hooks in the common directory, committed from a linked worktree.
    LinkedWorktree,
    /// A `pre-commit` hook that installs the `post-commit` hook while it runs.
    HookInstalledDuringTheCommit,
    /// A `pre-commit` hook that fails.
    FailingPreCommit,
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

/// Write an executable hook for `event` into `directory`.
fn install(directory: &Path, event: &str, log: &Path, exit_status: u8) {
    std::fs::create_dir_all(directory).expect("hooks directory");
    executable(directory.join(event).as_path(), script(event, log, exit_status).as_bytes());
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
    let mut global: Vec<String> = Vec::new();
    let mut commit_in: PathBuf = repo.clone();
    match shape {
        Shape::NoHooks => {}
        Shape::AllExecutable => {
            for event in EVENTS {
                install(hooks.as_path(), event, log.as_path(), 0);
            }
        }
        Shape::OneEvent => install(hooks.as_path(), "commit-msg", log.as_path(), 0),
        Shape::NotExecutable => {
            for event in EVENTS {
                std::fs::write(hooks.join(event), script(event, log.as_path(), 0)).expect("plain hook file");
            }
        }
        Shape::Symlinks => {
            install(repo.join("elsewhere").as_path(), "pre-commit", log.as_path(), 0);
            std::os::unix::fs::symlink(repo.join("elsewhere").join("pre-commit"), hooks.join("pre-commit"))
                .expect("working link");
            std::os::unix::fs::symlink(repo.join("nowhere"), hooks.join("post-commit")).expect("dangling link");
        }
        Shape::ConfigHooks => {
            for (name, event) in [("early", "pre-commit"), ("late", "post-commit")] {
                git(fixture, repo.as_path(), &["config", format!("hook.{name}.event").as_str(), event]);
                git(
                    fixture,
                    repo.as_path(),
                    &["config", format!("hook.{name}.command").as_str(), format!("echo {event} >> '{}'", log.display()).as_str()],
                );
            }
        }
        Shape::RelativeHooksPath => {
            git(fixture, repo.as_path(), &["config", "core.hooksPath", "custom-hooks"]);
            for event in EVENTS {
                install(repo.join("custom-hooks").as_path(), event, log.as_path(), 0);
                install(hooks.as_path(), event, fixture.root.join("must-not-run.log").as_path(), 0);
            }
        }
        Shape::AbsoluteHooksPath => {
            let custom: PathBuf = fixture.root.join(format!("{side}-absolute-hooks"));
            install(custom.as_path(), "commit-msg", log.as_path(), 0);
            git(fixture, repo.as_path(), &["config", "core.hooksPath", custom.to_str().expect("UTF-8")]);
        }
        Shape::EmptyHooksPath => {
            std::fs::create_dir(repo.join("empty-hooks")).expect("empty hooks directory");
            git(fixture, repo.as_path(), &["config", "core.hooksPath", "empty-hooks"]);
            for event in EVENTS {
                install(hooks.as_path(), event, fixture.root.join("must-not-run.log").as_path(), 0);
            }
        }
        Shape::MissingHooksPath => {
            git(fixture, repo.as_path(), &["config", "core.hooksPath", "no-such-directory"]);
        }
        Shape::HooksPathFromGlobalOption => {
            let custom: PathBuf = fixture.root.join(format!("{side}-option-hooks"));
            install(custom.as_path(), "pre-commit", log.as_path(), 0);
            install(custom.as_path(), "post-commit", log.as_path(), 0);
            global = vec![String::from("-c"), format!("core.hooksPath={}", custom.display())];
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
            for event in EVENTS {
                install(hooks.as_path(), event, log.as_path(), 0);
            }
            commit_in = linked;
        }
        Shape::HookInstalledDuringTheCommit => {
            let installer: String = format!(
                "#!/bin/sh\nprintf 'pre-commit %s\\n' \"$#\" >> '{log}'\ncat > '{target}' <<'EOF'\n{post}EOF\nchmod +x '{target}'\n",
                log = log.display(),
                target = hooks.join("post-commit").display(),
                post = script("post-commit", log.as_path(), 0),
            );
            std::fs::create_dir_all(hooks.as_path()).expect("hooks directory");
            executable(hooks.join("pre-commit").as_path(), installer.as_bytes());
        }
        Shape::FailingPreCommit => install(hooks.as_path(), "pre-commit", log.as_path(), 1),
    }
    return Side { commit_in, log, global };
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
    assert!(
        !fixture.root.join("must-not-run.log").exists(),
        "{name}: a hook outside the configured hooks path ran"
    );
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

/// Executable hooks for every event run in Git's order with Git's arguments and shape the message.
#[test]
fn executable_hooks_run_as_they_do_directly() {
    let log: Vec<String> = require_same("executable", Shape::AllExecutable);
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

/// One event with a hook runs that hook and only that one.
#[test]
fn a_single_event_hook_runs_alone() {
    assert_eq!(require_same("one-event", Shape::OneEvent), vec![String::from("commit-msg 1")]);
}

/// A hook file that is not executable is skipped with Git's hint, event by event, as it is directly.
#[test]
fn non_executable_hook_files_print_gits_hints() {
    assert_eq!(require_same("not-executable", Shape::NotExecutable), Vec::<String>::new());
    let (fixture, _direct_side, direct, wrapped_side, wrapped) = compare("not-executable-hints", Shape::NotExecutable);
    assert!(stderr_of(&direct).contains("was ignored because it's not set as executable"), "{}", stderr_of(&direct));
    assert_eq!(
        stderr_of(&wrapped).matches("was ignored because it's not set as executable").count(),
        stderr_of(&direct).matches("was ignored because it's not set as executable").count(),
        "{}",
        normalized(&wrapped, &wrapped_side, &fixture)
    );
    remove_settled(&fixture);
}

/// A link that works runs its target, and a dangling link behaves as Git treats it.
#[test]
fn working_and_dangling_links_behave_as_they_do_directly() {
    let log: Vec<String> = require_same("symlinks", Shape::Symlinks);
    assert_eq!(log, vec![String::from("pre-commit 0")]);
}

/// Hooks defined by name in the configuration run for their events.
#[test]
fn hooks_defined_in_the_configuration_run() {
    let log: Vec<String> = require_same("config-hooks", Shape::ConfigHooks);
    assert_eq!(log, vec![String::from("pre-commit"), String::from("post-commit")]);
}

/// A relative hooks path decides which hooks run, and the default directory's hooks stay silent.
#[test]
fn a_relative_hooks_path_replaces_the_default_directory() {
    let log: Vec<String> = require_same("relative-hooks-path", Shape::RelativeHooksPath);
    assert_eq!(log.len(), 4, "{log:?}");
}

/// An absolute hooks path with one hook runs that hook.
#[test]
fn an_absolute_hooks_path_with_one_hook_runs_it() {
    assert_eq!(require_same("absolute-hooks-path", Shape::AbsoluteHooksPath), vec![String::from("commit-msg 1")]);
}

/// A hooks path to an empty directory silences the default directory's hooks, and a missing directory runs nothing.
#[test]
fn an_empty_or_missing_hooks_path_runs_nothing() {
    assert_eq!(require_same("empty-hooks-path", Shape::EmptyHooksPath), Vec::<String>::new());
    assert_eq!(require_same("missing-hooks-path", Shape::MissingHooksPath), Vec::<String>::new());
}

/// A hooks path given with a global option decides which hooks run.
#[test]
fn a_hooks_path_from_a_global_option_decides_the_hooks() {
    let log: Vec<String> = require_same("global-option", Shape::HooksPathFromGlobalOption);
    assert_eq!(log, vec![String::from("pre-commit 0"), String::from("post-commit 0")]);
}

/// Disabled events with no hook behave as they do directly.
#[test]
fn disabled_events_without_hooks_show_what_direct_git_shows() {
    assert_eq!(require_same("disabled", Shape::DisabledWithoutHooks), Vec::<String>::new());
}

/// Hooks in the common directory run for a commit made in a linked worktree.
#[test]
fn hooks_run_for_a_commit_in_a_linked_worktree() {
    let log: Vec<String> = require_same("linked-worktree", Shape::LinkedWorktree);
    assert_eq!(log.len(), 4, "{log:?}");
}

/// A `post-commit` hook that a `pre-commit` hook installs during the commit runs, because the wrapper looks for
///  the hook when `post-commit` comes due.
#[test]
fn a_post_commit_hook_installed_during_the_commit_still_runs() {
    let log: Vec<String> = require_same("installed-during", Shape::HookInstalledDuringTheCommit);
    assert_eq!(log, vec![String::from("pre-commit 0"), String::from("post-commit 0")]);
}

/// A failing `pre-commit` hook stops the commit with its status, as it does directly.
#[test]
fn a_failing_pre_commit_hook_stops_the_commit_as_it_does_directly() {
    let log: Vec<String> = require_same("failing-pre-commit", Shape::FailingPreCommit);
    assert_eq!(log, vec![String::from("pre-commit 0")]);
}

/// A repository with hooks disabled by the switch and hooks present keeps skipping them, as the wrapper always has.
///  Direct Git runs a traditional hook whatever `hook.<event>.enabled` says, so this is the one shape in which the
///  wrapper and Git differ; the control pins that the omission of entries did not change it.
#[test]
fn a_disabled_event_with_a_hook_is_still_skipped_by_the_wrapper() {
    let fixture: Fixture = fixture("hook-shim-disabled-with-hook");
    let wrapped_side: Side = build(&fixture, "wrapped", Shape::NoHooks);
    install(wrapped_side.commit_in.join(".git").join("hooks").as_path(), "pre-commit", wrapped_side.log.as_path(), 0);
    install(wrapped_side.commit_in.join(".git").join("hooks").as_path(), "post-commit", wrapped_side.log.as_path(), 0);
    install(wrapped_side.commit_in.join(".git").join("hooks").as_path(), "commit-msg", wrapped_side.log.as_path(), 0);
    git(&fixture, wrapped_side.commit_in.as_path(), &["config", "hook.pre-commit.enabled", "false"]);
    git(&fixture, wrapped_side.commit_in.as_path(), &["config", "hook.post-commit.enabled", "false"]);
    let wrapped: Observed = run_commit(
        &fixture,
        wrapped_side.commit_in.as_path(),
        &["commit", "--quiet", "--message=second", "--", "a.txt"],
    );
    assert_eq!(wrapped.code, Some(0), "{}", stderr_of(&wrapped));
    assert_eq!(log_lines(wrapped_side.log.as_path()), vec![String::from("commit-msg 1")]);
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

/// The budget of a commit with all four hooks: one runner for each event, as before, and one configuration read.
#[test]
fn a_commit_with_four_hooks_starts_four_hook_runners() {
    let commands: Vec<String> = traced_commit("four", Shape::AllExecutable);
    assert_eq!(hook_runner_starts(commands.as_slice()), 4, "{commands:#?}");
    assert_eq!(starts_of(commands.as_slice(), "config"), 1, "{commands:#?}");
}

/// The budget of a commit with one hook: one runner, for the event that has the hook.
#[test]
fn a_commit_with_one_hook_starts_one_hook_runner() {
    let commands: Vec<String> = traced_commit("one", Shape::OneEvent);
    assert_eq!(hook_runner_starts(commands.as_slice()), 1, "{commands:#?}");
}
