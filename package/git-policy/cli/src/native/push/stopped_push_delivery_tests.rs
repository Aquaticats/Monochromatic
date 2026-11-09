//! Controls for the report of a push that was stopped while a local remote ran a hook after the update:
//! the references are on the remote,
//! and the report must say so instead of telling the person to push again.

use crate::auto_push_single_flight::PushAttempt;
use crate::bounded_push::{DELIVERED_BEFORE_THE_STOP, Patience, run_bounded_push};
use crate::test_support::{REAL_GIT, fixture, git, git_output, isolated_overlay, remove, repository};
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// The window the stop is given in these controls,
/// so they are quick and nothing depends on timing luck.
const STALL: Duration = Duration::from_millis(2000);

/// The grace the stop is given,
/// as a short one: the hook is a `sleep` that ends on `SIGTERM` at once.
const GRACE: Duration = Duration::from_secs(2);

/// The environment the fixture's push and its queries run with:
/// nothing of the host's Git configuration reaches them.
fn environment(root: &Path) -> Vec<(OsString, OsString)> {
    let mut pairs: Vec<(OsString, OsString)> = vec![
        (OsString::from("PATH"), OsString::from("/usr/bin:/bin")),
        (OsString::from("HOME"), root.join("home").into_os_string()),
    ];
    pairs.extend(isolated_overlay());
    return pairs;
}

/// The push of the fixture,
/// with the environment the wrapper gives its pushes.
fn push_command(repo: &Path, pairs: &[(OsString, OsString)]) -> Command {
    let mut command: Command = Command::new(REAL_GIT);
    command.args(["push", "origin", "refs/heads/main:refs/heads/main"]).current_dir(repo).env_clear();
    for pair in pairs {
        command.env(&pair.0, &pair.1);
    }
    return command;
}

/// A bare remote whose `post-receive` hook records itself,
/// writes its own process number and then never works again.
///
/// `core.fsync=none` keeps the fixture's own writes off the slow path this host measures for a first push
/// (measured 2026-11-09:
/// 24 ms to 21 s for the same small push,
/// with the whole wait in the remote's reference write),
/// so the window the control uses bounds the hook and not the disk.
fn remote_with_a_stalled_post_receive(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let remote: PathBuf = root.join("remote.git");
    git(root, &["init", "--quiet", "--bare", remote.to_str().expect("UTF-8")]);
    git(
        remote.as_path(),
        &["config", "core.fsync", "none"],
    );
    let markers: PathBuf = root.join("markers.txt");
    let pid_file: PathBuf = root.join("hook.pid");
    let hook: PathBuf = remote.join("hooks").join("post-receive");
    std::fs::write(
        hook.as_path(),
        format!(
            "#!/bin/sh\necho ran >> '{}'\necho $$ > '{}'\nexec sleep 600\n",
            markers.display(),
            pid_file.display()
        ),
    )
    .expect("hook");
    std::fs::set_permissions(hook.as_path(), std::fs::Permissions::from_mode(0o755)).expect("hook mode");
    return (remote, markers, pid_file);
}

/// The hook that runs after the references are written is ended with the push,
/// the references stay on the remote,
/// and the report says the push delivered them and that the hook's work is what was cut short.
#[test]
fn a_stopped_push_whose_refs_landed_reports_the_delivery_and_the_hook_it_ended() {
    let root: PathBuf = fixture("stopped-push-delivery");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let (remote, markers, pid_file): (PathBuf, PathBuf, PathBuf) = remote_with_a_stalled_post_receive(root.as_path());
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    let pairs: Vec<(OsString, OsString)> = environment(root.as_path());
    let attempt: PushAttempt = run_bounded_push(push_command(repo.as_path(), pairs.as_slice()), Patience::Progress { stall: STALL }, GRACE, pairs.as_slice())
        .expect("the push runs");
    let ran: String = std::fs::read_to_string(markers.as_path()).unwrap_or_default();
    assert_eq!(ran.trim(), "ran", "the remote's post-receive hook ran: {}", attempt.output);
    let local: Vec<u8> = git_output(repo.as_path(), &["rev-parse", "refs/heads/main"]).stdout;
    let published: Vec<u8> = git_output(remote.as_path(), &["rev-parse", "refs/heads/main"]).stdout;
    assert_eq!(local, published, "the reference landed on the remote before the stop: {}", attempt.output);
    assert!(attempt.output.contains(DELIVERED_BEFORE_THE_STOP), "the report says what was delivered: {}", attempt.output);
    assert!(
        attempt.output.contains("refs/heads/main"),
        "the report names the reference the remote holds: {}",
        attempt.output
    );
    assert!(
        !attempt.output.contains("push again when the connection works"),
        "the report does not send the person to push again, which cannot run that hook again: {}",
        attempt.output
    );
    assert!(
        attempt.output.contains("post-receive"),
        "the report names the remote's own work as what was ended: {}",
        attempt.output
    );
    let hook_pid: String = std::fs::read_to_string(pid_file.as_path()).unwrap_or_default().trim().to_owned();
    assert!(!hook_pid.is_empty(), "the hook wrote its process number: {}", attempt.output);
    assert!(
        !Path::new(format!("/proc/{hook_pid}").as_str()).exists(),
        "the person's post-receive hook was ended: process {hook_pid}"
    );
    remove(root.as_path());
}
