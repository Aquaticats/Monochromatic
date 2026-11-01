//! Binary-level controls for the ways a configuration could still make a Git the wrapper started run a program that a
//!  hook is: an alias that carries its own `-c` options, `core.fsmonitor` naming a program, and the editor of
//!  `git config --edit`.
//! Each control has a positive control that runs the same setup through real Git started directly and must write the
//!  marker, so an empty marker file proves the wrapper's options, not a setup that never fires.

use super::commit::{remove_settled, run_commit, wrapped_commit};
use super::support::{Fixture, Observed, direct, executable, fixture, git, observe, repository, stderr_of};
use std::ffi::OsStr;
use std::path::PathBuf;

/// A repository with one commit, a branch `other`, and the file the programs write their markers to.
struct Scene {
    /// The working repository, on `main`.
    repo: PathBuf,
    /// The markers the programs write.
    markers: PathBuf,
}

/// Create the scene.
fn scene(fixture: &Fixture) -> Scene {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    git(fixture, repo.as_path(), &["branch", "other"]);
    return Scene { repo, markers: fixture.root.join("markers.txt") };
}

/// The markers written so far.
fn markers(scene: &Scene) -> String {
    return std::fs::read_to_string(&scene.markers).unwrap_or_default();
}

/// Write an executable `name` under the fixture that appends `word` to the markers, then runs `rest`.
fn program(fixture: &Fixture, scene: &Scene, name: &str, word: &str, rest: &str) -> PathBuf {
    let path: PathBuf = fixture.root.join(name);
    executable(
        path.as_path(),
        format!("#!/bin/sh\necho {word} >> '{}'\n{rest}\n", scene.markers.display()).as_bytes(),
    );
    return path;
}

/// Run a wrapped command in the scene's repository.
fn wrapped(fixture: &Fixture, scene: &Scene, arguments: &[&str]) -> Observed {
    return observe(&mut wrapped_commit(fixture, scene.repo.as_path(), arguments), b"");
}

/// Define a hook file and a configured hook for `post-checkout`, each writing its own marker.
fn post_checkout_hooks(fixture: &Fixture, scene: &Scene) -> PathBuf {
    let hooks: PathBuf = fixture.root.join("alias-hooks");
    std::fs::create_dir(&hooks).expect("hooks directory");
    executable(
        hooks.join("post-checkout").as_path(),
        format!("#!/bin/sh\necho file >> '{}'\n", scene.markers.display()).as_bytes(),
    );
    let configured: PathBuf = program(fixture, scene, "configured-hook", "configured", "");
    git(fixture, scene.repo.as_path(), &["config", "hook.control.event", "post-checkout"]);
    git(fixture, scene.repo.as_path(), &["config", "hook.control.command", configured.to_str().expect("UTF-8")]);
    return hooks;
}

/// An alias whose value carries `-c core.hooksPath` and `-c hook.post-checkout.enabled=true` cannot bring a hook back:
///  the wrapper hands Git the alias already expanded, so its options come after the alias's own.
#[test]
fn an_alias_that_carries_hook_options_cannot_bring_a_hook_back() {
    let fixture: Fixture = fixture("hooks-off-alias-escape");
    let scene: Scene = scene(&fixture);
    let hooks: PathBuf = post_checkout_hooks(&fixture, &scene);
    let value: String = format!(
        "-c core.hooksPath={} -c hook.post-checkout.enabled=true checkout --quiet other",
        hooks.display()
    );
    git(&fixture, scene.repo.as_path(), &["config", "alias.escape", value.as_str()]);
    let control = direct(&fixture).current_dir(&scene.repo).args(["escape"]).output().expect("git starts");
    assert!(control.status.success(), "{}", String::from_utf8_lossy(&control.stderr));
    let ran: String = markers(&scene);
    assert!(ran.contains("file") && ran.contains("configured"), "real Git must run both hooks: {ran:?}");
    std::fs::remove_file(&scene.markers).expect("clear the markers");
    git(&fixture, scene.repo.as_path(), &["checkout", "--quiet", "--force", "main"]);
    std::fs::remove_file(&scene.markers).expect("clear the markers of the reset");
    let switched: Observed = wrapped(&fixture, &scene, &["escape"]);
    assert_eq!(switched.code, Some(0), "{}", stderr_of(&switched));
    assert_eq!(markers(&scene), "", "no hook runs through an alias");
    remove_settled(&fixture);
}

/// `core.fsmonitor` naming a program is the hook `fsmonitor-watchman`; the wrapper's Git does not run it.
#[test]
fn a_core_fsmonitor_program_does_not_run() {
    let fixture: Fixture = fixture("hooks-off-fsmonitor");
    let scene: Scene = scene(&fixture);
    let monitor: PathBuf = program(&fixture, &scene, "fsmonitor", "fsmonitor", "exit 1");
    git(&fixture, scene.repo.as_path(), &["config", "core.fsmonitor", monitor.to_str().expect("UTF-8")]);
    let control = direct(&fixture).current_dir(&scene.repo).args(["status", "--short"]).output().expect("git starts");
    assert!(control.status.success());
    assert!(markers(&scene).contains("fsmonitor"), "real Git must run the program");
    std::fs::remove_file(&scene.markers).expect("clear the markers");
    let switched: Observed = wrapped(&fixture, &scene, &["status", "--short"]);
    assert_eq!(switched.code, Some(0), "{}", stderr_of(&switched));
    assert_eq!(markers(&scene), "", "the monitor program must not run through the wrapper");
    remove_settled(&fixture);
}

/// The editor of `git config --edit` is a child of a wrapper-started Git, so a Git it starts directly inherits the
///  options and runs no hook; `git config` goes without the options only when it just reports configuration.
#[test]
fn a_git_started_by_the_config_editor_runs_no_hook() {
    let fixture: Fixture = fixture("hooks-off-config-edit");
    let scene: Scene = scene(&fixture);
    let configured: PathBuf = program(&fixture, &scene, "configured-hook", "configured", "");
    git(&fixture, scene.repo.as_path(), &["config", "hook.control.event", "post-checkout"]);
    git(&fixture, scene.repo.as_path(), &["config", "hook.control.command", configured.to_str().expect("UTF-8")]);
    let editor: PathBuf = program(&fixture, &scene, "editor", "editor", "/usr/bin/git checkout --quiet other");
    let mut control = direct(&fixture);
    control.current_dir(&scene.repo).args(["config", "--edit"]).env("GIT_EDITOR", &editor).env("TERM", "xterm");
    assert!(control.status().expect("git starts").success());
    assert!(markers(&scene).contains("configured"), "real Git must run the hook: {:?}", markers(&scene));
    std::fs::remove_file(&scene.markers).expect("clear the markers");
    git(&fixture, scene.repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::remove_file(&scene.markers).expect("clear the markers of the reset");
    let mut command = wrapped_commit(&fixture, scene.repo.as_path(), &["config", "--edit"]);
    command.env("GIT_EDITOR", &editor).env("TERM", "xterm");
    let switched: Observed = observe(&mut command, b"");
    assert_eq!(switched.code, Some(0), "{}", stderr_of(&switched));
    assert_eq!(markers(&scene), "editor\n", "the editor ran and its Git ran no hook");
    remove_settled(&fixture);
}

/// A commit in a repository whose `core.fsmonitor` names a program runs no monitor program in any Git it starts, the
///  native commit in the shadow included, and the commit lands.
#[test]
fn a_commit_runs_no_core_fsmonitor_program() {
    let fixture: Fixture = fixture("hooks-off-fsmonitor-commit");
    let scene: Scene = scene(&fixture);
    std::fs::write(scene.repo.join("a.txt"), b"one\n").expect("file");
    git(&fixture, scene.repo.as_path(), &["add", "--all"]);
    git(&fixture, scene.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(scene.repo.join("a.txt"), b"one\ntwo\n").expect("edit");
    let monitor: PathBuf = program(&fixture, &scene, "fsmonitor", "fsmonitor", "exit 1");
    git(&fixture, scene.repo.as_path(), &["config", "core.fsmonitor", monitor.to_str().expect("UTF-8")]);
    let committed = run_commit(&fixture, scene.repo.as_path(), &["commit", "--quiet", "--message=second", "--", "a.txt"]);
    assert_eq!(committed.code, Some(0), "{}", stderr_of(&committed));
    assert_eq!(markers(&scene), "", "the monitor program must not run during a commit");
    let subject = git(&fixture, scene.repo.as_path(), &["log", "-1", "--format=%s"]);
    assert_eq!(String::from_utf8_lossy(&subject.stdout).trim(), "second");
    remove_settled(&fixture);
}
