//! What:
//!  The checkout of a new worktree runs with Git's parallel checkout switched on only when the person's own configuration says
//!  nothing about it, through the built executable and a stand-in Git that records the command line it receives.
//! Why:
//!  Git's `checkout.workers` defaults to one worker, and a twenty-thousand-file checkout measured several times faster with
//!  parallel workers. The owner decided on 2026-11-03 (question tool, against the coordinator's recommendation) "Pass only if
//!  unset": a value the person set in any scope Git reads, in `GIT_CONFIG_*` or in a `-c` they typed always wins, so the
//!  wrapper adds `-c checkout.workers=0` only where nothing sets it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(recordedLine('worktree add')).toContain('-c checkout.workers=0 worktree add');
//! // with the key set anywhere: expect(recordedLine('worktree add')).not.toContain('checkout.workers=0');
//! ```

/// Import the shared fixtures and the bounded process helpers.
use super::support::{Fixture, Observed, bounded, executable, fixture, git, observe, remove, repository, stderr_of};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The option the wrapper adds, which opens the recorded command line: the wrapper puts its own hook-switching options after the
///  person's leading global options and before the subcommand.
const ADDED: &str = "-c checkout.workers=0 ";

/// A repository and a stand-in Git ahead of real Git that appends every command line it receives to a log, then runs real Git.
///  The repository holds an untracked file and is no subvolume, so neither the clone rung nor the snapshot rung serves an add
///  from it: Git's own checkout does, which is the run these controls watch.
fn recorded_repository(fixture: &Fixture) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("untracked.txt"), b"untracked\n").expect("untracked file");
    std::fs::create_dir(fixture.root.join("shim")).expect("shim directory");
    executable(fixture.root.join("shim/git").as_path(), b"#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$CLI_GIT_TEST_SPAWN_LOG\"\nexec /usr/bin/git \"$@\"\n");
    let log: PathBuf = fixture.root.join("spawn.log");
    std::fs::write(&log, b"").expect("empty spawn log");
    return (repo, log);
}

/// Run `git <arguments>` through the wrapper in `repo` with the stand-in behind it and `extra` variables added; return what the
///  caller saw and the command lines the stand-in recorded.
fn run_recorded(fixture: &Fixture, repo: &Path, log: &Path, arguments: &[&str], extra: &[(&str, &str)]) -> (Observed, Vec<String>) {
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(fixture.root.join("shim"));
    let mut command = bounded(fixture, fixture.root.join("bin/git").as_path(), path.as_os_str());
    command.env("CLI_GIT_TEST_SPAWN_LOG", log).current_dir(repo).args(arguments);
    for (name, value) in extra {
        command.env(name, value);
    }
    let observed: Observed = observe(&mut command, b"");
    let lines: Vec<String> = std::fs::read_to_string(log).expect("spawn log").lines().map(str::to_owned).collect();
    return (observed, lines);
}

/// The one recorded line that is the checkout of the new worktree, or the panic that says it was not run.
fn add_line(lines: &[String], observed: &Observed) -> String {
    let found: Vec<&String> = lines.iter().filter(|line: &&String| { return line.contains("worktree add"); }).collect();
    assert_eq!(found.len(), 1, "one worktree add reached Git: {lines:?} {}", stderr_of(observed));
    return found[0].clone();
}

/// With nothing set, the add that Git's checkout serves gets `-c checkout.workers=0` before the subcommand, and succeeds.
#[test]
fn an_unset_checkout_workers_gets_parallel_checkout_before_the_subcommand() {
    let fixture: Fixture = fixture("workers-unset");
    let (repo, log) = recorded_repository(&fixture);
    let (observed, lines) = run_recorded(&fixture, repo.as_path(), log.as_path(), &["worktree", "add", "-b", "topic", "../topic"], &[]);
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    let line: String = add_line(&lines, &observed);
    assert!(line.starts_with(ADDED) && line.contains(" worktree add "), "{line}\n{}", stderr_of(&observed));
    assert!(fixture.root.join("topic/.git").exists(), "the worktree was made");
    remove(&fixture);
}

/// A value in the repository's configuration, any value, stops the addition and leaves Git the person's value.
#[test]
fn a_checkout_workers_in_the_configuration_is_left_alone() {
    let fixture: Fixture = fixture("workers-config");
    let (repo, log) = recorded_repository(&fixture);
    for (index, value) in ["1", "4", "-1", "0"].iter().enumerate() {
        git(&fixture, repo.as_path(), &["config", "checkout.workers", value]);
        std::fs::write(&log, b"").expect("empty spawn log");
        let name: String = format!("topic{index}");
        let (observed, lines) = run_recorded(&fixture, repo.as_path(), log.as_path(), &["worktree", "add", "-b", name.as_str(), format!("../{name}").as_str()], &[]);
        assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
        let line: String = add_line(&lines, &observed);
        assert!(!line.contains("checkout.workers"), "the person's value {value} stands alone: {line}");
    }
    remove(&fixture);
}

/// A `-c checkout.workers=<n>` the person typed, and the `GIT_CONFIG_*` variables, are the person's value too.
#[test]
fn a_typed_option_or_an_environment_value_is_left_alone() {
    let fixture: Fixture = fixture("workers-typed");
    let (repo, log) = recorded_repository(&fixture);
    let (typed, typed_lines) = run_recorded(&fixture, repo.as_path(), log.as_path(), &["-c", "checkout.workers=3", "worktree", "add", "-b", "typed", "../typed"], &[]);
    assert_eq!(typed.code, Some(0), "{}", stderr_of(&typed));
    let typed_line: String = add_line(&typed_lines, &typed);
    assert!(typed_line.starts_with("-c checkout.workers=3 ") && !typed_line.contains("checkout.workers=0"), "{typed_line}");
    std::fs::write(&log, b"").expect("empty spawn log");
    let environment = [("GIT_CONFIG_COUNT", "1"), ("GIT_CONFIG_KEY_0", "checkout.workers"), ("GIT_CONFIG_VALUE_0", "2")];
    let (set, set_lines) = run_recorded(&fixture, repo.as_path(), log.as_path(), &["worktree", "add", "-b", "environment", "../environment"], &environment);
    assert_eq!(set.code, Some(0), "{}", stderr_of(&set));
    let set_line: String = add_line(&set_lines, &set);
    assert!(!set_line.contains("checkout.workers"), "{set_line}");
    remove(&fixture);
}
