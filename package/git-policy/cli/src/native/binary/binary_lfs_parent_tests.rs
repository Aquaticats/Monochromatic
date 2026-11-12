//! Binary-level controls for who starts and who explains the Git LFS upload.
//! The wrapper runs `git-lfs push` itself,
//! as a child,
//! before the push;
//! the wrapper is never started for it,
//! the push
//!  carries no hook,
//! and the parent that ran the child prints the explanation,
//! so no output text or trace is read.

use super::commit::{remove_settled, wrapped_commit};
use super::lfs_pre_push::{World, commit_directly, ref_of, runs, wrapped, world};
use super::support::{Fixture, Observed, executable, fixture, observe, stderr_of};
use std::path::Path;

/// The sentence of the failure explanation that no other text of the wrapper contains.
const EXPLANATION: &str = "so this push was not started and nothing was published";

/// Make the origin refuse every push after the upload,
/// printing `message` as its own report.
fn refusing_origin(world: &World, message: &str) {
    let hook: std::path::PathBuf = world.origin.join("hooks/pre-receive");
    std::fs::create_dir_all(hook.parent().expect("hooks directory")).expect("hooks directory");
    executable(hook.as_path(), format!("#!/bin/sh\ncat > /dev/null\nprintf '%s\\n' '{message}' >&2\nexit 1\n").as_bytes());
}

/// A stand-in `git-lfs` that records the executable names of its ancestors,
/// nearest first,
/// then succeeds.
fn ancestry_stand_in(fixture: &Fixture, record: &Path) {
    let source: String = format!(
        "#!/bin/sh\npid=$$\nwhile [ \"$pid\" -gt 1 ]; do\n  pid=$(ps -o ppid= -p \"$pid\" | tr -d ' ')\n  \
         [ -n \"$pid\" ] && basename \"$(readlink /proc/$pid/exe)\" >> '{}'\ndone\nexit 0\n",
        record.display()
    );
    executable(fixture.root.join("bin/git-lfs").as_path(), source.as_bytes());
}

/// The upload is a child of the one wrapper the caller started,
/// and the push that follows carries no hook.
#[test]
fn the_wrapper_runs_the_upload_itself_and_the_push_carries_no_hook() {
    let fixture: Fixture = fixture("lfs-parent-chain");
    let world: World = world(&fixture, true, None);
    let ancestors: std::path::PathBuf = fixture.root.join("ancestors.txt");
    ancestry_stand_in(&fixture, ancestors.as_path());
    let trace: std::path::PathBuf = fixture.root.join("trace.json");
    let mut command = wrapped_commit(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    command.env("GIT_TRACE2_EVENT", &trace);
    let pushed: Observed = observe(&mut command, b"");
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    let names: Vec<String> =
        std::fs::read_to_string(&ancestors).expect("the stand-in ran").lines().map(str::to_owned).collect();
    assert!(
        names.first().is_some_and(|name| {
            return name.starts_with("monochromatic-git-policy-cli");
        }),
        "the wrapper is the parent: {names:?}"
    );
    let wrappers: usize = names
        .iter()
        .filter(|name| {
            return name.starts_with("monochromatic-git-policy-cli");
        })
        .count();
    assert_eq!(wrappers, 1, "no second wrapper stands anywhere above the upload: {names:?}");
    let events: String = std::fs::read_to_string(&trace).expect("Git wrote its events to the caller's target");
    assert!(events.contains("\"event\":\"start\""), "the trace saw the push: {events}");
    let hooks: Vec<&str> = events
        .lines()
        .filter(|line| {
            return line.contains("\"child_class\":\"hook\"");
        })
        .collect();
    assert!(hooks.is_empty(), "the push ran no hook of any kind: {hooks:?}");
    remove_settled(&fixture);
}

/// A failed upload is explained by the parent once,
/// with the child's own status,
/// and the push is never started.
#[test]
fn a_failed_upload_is_explained_once_and_the_push_is_not_started() {
    let fixture: Fixture = fixture("lfs-parent-explains");
    let failing_status: i32 = 7;
    let world: World = world(&fixture, true, Some(failing_status));
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    let stderr: String = stderr_of(&pushed);
    assert_ne!(pushed.code, Some(0), "{stderr}");
    assert!(stderr.contains(&format!("`git-lfs push` exited with status {failing_status}")), "{stderr}");
    assert_eq!(stderr.matches(EXPLANATION).count(), 1, "{stderr}");
    assert!(!stderr.contains("To ") && !stderr.contains("failed to push some refs"), "Git never started: {stderr}");
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent");
    remove_settled(&fixture);
}

/// A remote refusal after a good upload is not blamed on the upload,
/// whatever text the remote prints.
#[test]
fn a_remote_refusal_is_not_blamed_on_the_upload_even_when_it_imitates_the_explanation() {
    let fixture: Fixture = fixture("lfs-parent-forged");
    let world: World = world(&fixture, true, Some(0));
    refusing_origin(&world, "monochromatic-git-policy-cli: git-lfs push exited with status 1, so this push was not started and nothing was published");
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    let stderr: String = stderr_of(&pushed);
    assert_ne!(pushed.code, Some(0), "{stderr}");
    assert_eq!(stderr.matches(EXPLANATION).count(), 1, "only the remote's own line carries the text, the wrapper adds none: {stderr}");
    assert_eq!(runs(&world).len(), 1, "the upload ran once");
    remove_settled(&fixture);
}

/// Another configured `pre-push` hook of the caller does not run (the wrapper runs no configured hook),
/// so its failure
///  can neither stop the push nor be blamed on the upload.
#[test]
fn another_configured_pre_push_hook_does_not_run_and_is_not_blamed_on_the_upload() {
    let fixture: Fixture = fixture("lfs-parent-other-hook");
    let world: World = world(&fixture, true, Some(0));
    let pushed: Observed = wrapped(
        &fixture,
        world.repo.as_path(),
        &["-c", "hook.theirs.event=pre-push", "-c", "hook.theirs.command=false", "push", "origin", "main"],
    );
    let stderr: String = stderr_of(&pushed);
    assert_eq!(pushed.code, Some(0), "{stderr}");
    assert!(!stderr.contains(EXPLANATION), "{stderr}");
    assert_ne!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent", "{stderr}");
    remove_settled(&fixture);
}

/// With `git-lfs` missing from `PATH` the push is refused before anything starts,
/// unless it bypasses the upload.
#[test]
fn a_missing_program_is_refused_before_anything_starts_unless_the_upload_is_bypassed() {
    let fixture: Fixture = fixture("lfs-parent-missing");
    let world: World = world(&fixture, true, None);
    let refused: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    let stderr: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "{stderr}");
    assert!(stderr.contains("was not found in the folders this push searches"), "{stderr}");
    assert!(!stderr.contains("Writing objects") && !stderr.contains("To "), "nothing started: {stderr}");
    let bypassed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "--no-verify", "origin", "main"]);
    assert_eq!(bypassed.code, Some(0), "{}", stderr_of(&bypassed));
    assert_ne!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent");
    remove_settled(&fixture);
}

/// `GIT_LFS_SKIP_PUSH` skips the upload,
/// as the explanation promises.
#[test]
fn the_skip_variable_skips_the_upload() {
    let fixture: Fixture = fixture("lfs-parent-skip");
    let world: World = world(&fixture, true, Some(9));
    let mut command = wrapped_commit(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    command.env("GIT_LFS_SKIP_PUSH", "1");
    let pushed: Observed = observe(&mut command, b"");
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    assert!(runs(&world).is_empty());
    remove_settled(&fixture);
}

/// A forced push,
/// which is never integrated,
/// still has a parent that explains a failed upload once.
#[test]
fn a_forced_push_that_is_never_integrated_still_has_a_parent_that_explains() {
    let fixture: Fixture = fixture("lfs-parent-forced");
    let world: World = world(&fixture, true, Some(9));
    commit_directly(&fixture, world.repo.as_path(), "more.txt", "more\n");
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "--force", "origin", "main"]);
    let stderr: String = stderr_of(&pushed);
    assert_ne!(pushed.code, Some(0), "{stderr}");
    assert_eq!(stderr.matches(EXPLANATION).count(), 1, "{stderr}");
    remove_settled(&fixture);
}

/// A bare push resolves its remote and branch from the repository's configuration,
/// and `-C` selects the directory the
///  upload runs in.
#[test]
fn a_bare_push_and_a_directory_option_select_the_remote_branch_and_directory() {
    let fixture: Fixture = fixture("lfs-parent-bare");
    let world: World = world(&fixture, true, None);
    let record: std::path::PathBuf = fixture.root.join("seen.txt");
    let source: String = format!("#!/bin/sh\npwd > '{record}'\nprintf '%s\\n' \"$*\" >> '{record}'\nexit 0\n", record = record.display());
    executable(fixture.root.join("bin/git-lfs").as_path(), source.as_bytes());
    super::support::git(&fixture, world.repo.as_path(), &["config", "branch.main.remote", "origin"]);
    super::support::git(&fixture, world.repo.as_path(), &["config", "push.default", "current"]);
    let mut command = wrapped_commit(&fixture, fixture.root.as_path(), &["-C", world.repo.to_str().expect("UTF-8"), "push"]);
    command.current_dir(&fixture.root);
    let pushed: Observed = observe(&mut command, b"");
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    let seen: Vec<String> = std::fs::read_to_string(&record).expect("the stand-in ran").lines().map(str::to_owned).collect();
    assert_eq!(seen, vec![world.repo.to_string_lossy().into_owned(), String::from("push origin refs/heads/main")]);
    remove_settled(&fixture);
}
