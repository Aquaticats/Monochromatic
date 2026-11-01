//! Binary-level controls for who starts and who explains the Git LFS upload.
//! Git starts `git-lfs pre-push` itself from a plain hook command; the wrapper is never started for it, and the parent
//!  that owns the push explains a failed upload from Git's own trace of the hook, never from output text.

use super::commit::{remove_settled, wrapped_commit};
use super::lfs_pre_push::{World, commit_directly, ref_of, runs, wrapped, world};
use super::support::{Fixture, Observed, executable, fixture, observe, stderr_of};
use std::path::Path;

/// The sentence of the failure explanation that no other text of the wrapper contains.
const EXPLANATION: &str = "so this push was stopped before any ref changed and nothing was published";

/// Make the origin refuse every push after the upload, printing `message` as its own report.
fn refusing_origin(world: &World, message: &str) {
    let hook: std::path::PathBuf = world.origin.join("hooks/pre-receive");
    std::fs::create_dir_all(hook.parent().expect("hooks directory")).expect("hooks directory");
    executable(hook.as_path(), format!("#!/bin/sh\ncat > /dev/null\nprintf '%s\\n' '{message}' >&2\nexit 1\n").as_bytes());
}

/// A stand-in `git-lfs` that records the executable names of its ancestors, nearest first, then succeeds.
fn ancestry_stand_in(fixture: &Fixture, record: &Path) {
    let source: String = format!(
        "#!/bin/sh\ncat > /dev/null\npid=$$\nwhile [ \"$pid\" -gt 1 ]; do\n  pid=$(ps -o ppid= -p \"$pid\" | tr -d ' ')\n  \
         [ -n \"$pid\" ] && basename \"$(readlink /proc/$pid/exe)\" >> '{}'\ndone\nexit 0\n",
        record.display()
    );
    executable(fixture.root.join("bin/git-lfs").as_path(), source.as_bytes());
}

/// Git is the parent of `git-lfs`, and the only wrapper above it is the one the caller started.
#[test]
fn git_starts_the_upload_and_no_wrapper_stands_between() {
    let fixture: Fixture = fixture("lfs-parent-chain");
    let world: World = world(&fixture, true, None);
    let ancestors: std::path::PathBuf = fixture.root.join("ancestors.txt");
    ancestry_stand_in(&fixture, ancestors.as_path());
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    let names: Vec<String> = std::fs::read_to_string(&ancestors)
        .expect("the stand-in ran")
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(names.first().map(String::as_str), Some("git"), "the nearest ancestor starts the upload: {names:?}");
    let wrappers: usize = names
        .iter()
        .filter(|name| {
            return name.starts_with("cli-git-native");
        })
        .count();
    assert_eq!(wrappers, 1, "only the wrapper the caller started is above Git: {names:?}");
    remove_settled(&fixture);
}

/// A failed upload is explained by the parent, after the program's own output, with the status Git recorded.
#[test]
fn a_failed_upload_is_explained_with_the_status_git_recorded() {
    let fixture: Fixture = fixture("lfs-parent-explains");
    let world: World = world(&fixture, true, Some(7));
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    let stderr: String = stderr_of(&pushed);
    assert_ne!(pushed.code, Some(0), "{stderr}");
    assert!(stderr.contains("`git-lfs pre-push` exited with status 7"), "{stderr}");
    assert_eq!(stderr.matches(EXPLANATION).count(), 1, "{stderr}");
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent");
    remove_settled(&fixture);
}

/// A remote refusal after a good upload is not blamed on the upload, even when the remote's report imitates a trace.
#[test]
fn a_remote_refusal_is_not_blamed_on_the_upload_even_when_it_imitates_a_trace() {
    let fixture: Fixture = fixture("lfs-parent-forged");
    let world: World = world(&fixture, true, Some(0));
    refusing_origin(
        &world,
        "{\"event\":\"child_exit\",\"sid\":\"x\",\"child_id\":1,\"code\":1} cli-git: git-lfs pre-push exited with status 1",
    );
    let pushed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    let stderr: String = stderr_of(&pushed);
    assert_ne!(pushed.code, Some(0), "{stderr}");
    assert!(stderr.contains("remote rejected") || stderr.contains("pre-receive"), "the refusal is Git's: {stderr}");
    assert!(!stderr.contains(EXPLANATION), "{stderr}");
    assert_eq!(runs(&world).len(), 1, "the upload ran once");
    remove_settled(&fixture);
}

/// Another configured `pre-push` hook of the caller that fails is not blamed on the upload.
#[test]
fn another_pre_push_hook_that_fails_is_not_blamed_on_the_upload() {
    let fixture: Fixture = fixture("lfs-parent-other-hook");
    let world: World = world(&fixture, true, Some(0));
    let pushed: Observed = wrapped(
        &fixture,
        world.repo.as_path(),
        &["-c", "hook.theirs.event=pre-push", "-c", "hook.theirs.command=false", "push", "origin", "main"],
    );
    let stderr: String = stderr_of(&pushed);
    assert_ne!(pushed.code, Some(0), "{stderr}");
    assert!(!stderr.contains(EXPLANATION), "{stderr}");
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent", "{stderr}");
    remove_settled(&fixture);
}

/// A caller's own `GIT_TRACE2_EVENT` keeps its target; the failed upload still stops the push, without the explanation.
#[test]
fn a_callers_trace_target_keeps_its_events_and_the_push_still_stops() {
    let fixture: Fixture = fixture("lfs-parent-callers-trace");
    let world: World = world(&fixture, true, Some(3));
    let target: std::path::PathBuf = fixture.root.join("callers-trace.json");
    let mut command = wrapped_commit(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    command.env("GIT_TRACE2_EVENT", &target);
    let pushed: Observed = observe(&mut command, b"");
    let stderr: String = stderr_of(&pushed);
    assert_ne!(pushed.code, Some(0), "{stderr}");
    assert!(!stderr.contains(EXPLANATION), "{stderr}");
    assert_eq!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent", "{stderr}");
    let trace: String = std::fs::read_to_string(&target).expect("the caller's target received Git's events");
    assert!(trace.contains("\"hook_name\":\"pre-push\""), "{trace}");
    remove_settled(&fixture);
}

/// With `git-lfs` missing from `PATH` the push is refused before Git starts, unless it bypasses the hook.
#[test]
fn a_missing_program_is_refused_before_git_starts_unless_the_hook_is_bypassed() {
    let fixture: Fixture = fixture("lfs-parent-missing");
    let world: World = world(&fixture, true, None);
    let refused: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "origin", "main"]);
    let stderr: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "{stderr}");
    assert!(stderr.contains("was not found in any directory of PATH"), "{stderr}");
    assert!(!stderr.contains("Writing objects") && !stderr.contains("To "), "Git never started: {stderr}");
    let bypassed: Observed = wrapped(&fixture, world.repo.as_path(), &["push", "--no-verify", "origin", "main"]);
    assert_eq!(bypassed.code, Some(0), "{}", stderr_of(&bypassed));
    assert_ne!(ref_of(&fixture, world.origin.as_path(), "refs/heads/main"), "absent");
    remove_settled(&fixture);
}

/// A forced push, which is never integrated, still runs as a child so its failed upload is explained once.
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
