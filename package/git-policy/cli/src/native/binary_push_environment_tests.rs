//! Binary-level controls for the environment every push-related Git child receives:
//!  the coordination queries, the auto-push after a landed commit, and the manual-push probe.
//! Every control runs the built executable on a disposable repository whose `origin` is an `ext::` transport
//!  that runs a recording shim in the Git child's environment, then forwards to a local bare repository.

use super::support::{
    Fixture, Observed, REAL_GIT, bounded_for, executable, fixture, git, observe, remove, repository, stderr_of,
    wrapper_first_path,
};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The bound of one wrapped command under host load, in seconds.
const PUSH_BOUND_SECONDS: &str = "120";

/// The variable only the control sets in the wrapper's environment, so only inheritance can deliver it.
const MARKER_NAME: &str = "CLI_GIT_PUSH_ENVIRONMENT_MARKER";

/// The marker's value.
const MARKER_VALUE: &str = "carried-by-the-invocation";

/// The per-user scratch directory relative to `HOME`, where the manual-push probe makes its private directory.
const SCRATCH_RELATIVE: &str = ".local/state/cli-git/scratch";

/// One repository, its bare origin, and the shim's record.
struct PushWorld {
    /// The working repository.
    repo: PathBuf,
    /// The bare repository `origin` reaches through the shim.
    bare: PathBuf,
    /// The file the shim appends one line to for every transport start.
    record: PathBuf,
}

/// One transport start the shim recorded.
#[derive(Debug)]
struct Start {
    /// `git-upload-pack` or `git-receive-pack`.
    service: String,
    /// The marker variable's value, or `unset`.
    marker: String,
    /// `HOME`, or `unset`.
    home: String,
    /// `PATH`, or `unset`.
    path: String,
    /// The forward-target variable the wrapper adds to every Git child, or `unset`.
    forward_target: String,
    /// The environment-supplied configuration count the lock PID injection adds, or `unset`.
    config_count: String,
    /// The entry names in the invocation's per-user scratch directory at the moment of the start, comma-terminated.
    scratch: String,
}

/// The shim: records the child's environment, refuses without the marker, then serves the bare repository.
///  Absolute paths only, because the defect under test leaves the child without `PATH`.
fn shim_source(record: &Path) -> String {
    return format!(
        "#!/bin/sh\n\
         printf 'service=%s|marker=%s|home=%s|path=%s|target=%s|count=%s|scratch=%s\\n' \
         \"$1\" \"${{{MARKER_NAME}-unset}}\" \"${{HOME-unset}}\" \"${{PATH-unset}}\" \
         \"${{CLI_GIT_NATIVE_FORWARD_TARGET-unset}}\" \"${{GIT_CONFIG_COUNT-unset}}\" \
         \"$(ls \"${{HOME}}/{SCRATCH_RELATIVE}\" 2>/dev/null | tr '\\n' ',')\" >> '{record}'\n\
         [ \"${{{MARKER_NAME}-}}\" = '{MARKER_VALUE}' ] || {{ echo 'the push environment lost the marker' >&2; exit 1; }}\n\
         exec {REAL_GIT} \"${{1#git-}}\" \"$2\"\n",
        record = record.display(),
    );
}

/// A repository with one committed file, an `origin` that goes through the shim, and an edit to that file.
fn push_world(fixture: &Fixture) -> PushWorld {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let bare: PathBuf = fixture.root.join("origin.git");
    git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "origin.git"]);
    let record: PathBuf = fixture.root.join("shim-record.txt");
    let shim: PathBuf = fixture.root.join("shim.sh");
    executable(shim.as_path(), shim_source(record.as_path()).as_bytes());
    let url: String = format!("ext::{} %S {}", shim.display(), bare.display());
    git(fixture, repo.as_path(), &["remote", "add", "origin", url.as_str()]);
    git(fixture, repo.as_path(), &["config", "protocol.ext.allow", "always"]);
    git(fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a.txt");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a.txt");
    return PushWorld { repo, bare, record };
}

/// A wrapped command for the fixture's `bin/git` that also carries the marker in its environment.
fn wrapped_with_marker(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Command {
    let mut command: Command = bounded_for(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        PUSH_BOUND_SECONDS,
    );
    command.current_dir(directory).args(arguments).env(MARKER_NAME, MARKER_VALUE);
    return command;
}

/// Every transport start the shim recorded, in order.
fn starts(world: &PushWorld) -> Vec<Start> {
    let text: String = std::fs::read_to_string(world.record.as_path()).unwrap_or_default();
    return text
        .lines()
        .map(|line| {
            let field = |name: &str| -> String {
                return line
                    .split('|')
                    .find_map(|part| { return part.strip_prefix(format!("{name}=").as_str()); })
                    .unwrap_or("missing")
                    .to_owned();
            };
            return Start {
                service: field("service"),
                marker: field("marker"),
                home: field("home"),
                path: field("path"),
                forward_target: field("target"),
                config_count: field("count"),
                scratch: field("scratch"),
            };
        })
        .collect();
}

/// What the child of a push-related Git must have: the invocation's own variables and the wrapper's additions.
fn assert_inherited(fixture: &Fixture, started: &[Start]) {
    let expected_path: String = wrapper_first_path(fixture).to_string_lossy().into_owned();
    let expected_home: String = fixture.root.join("home").to_string_lossy().into_owned();
    for start in started {
        assert_eq!(start.marker, MARKER_VALUE, "the marker reaches {start:?}");
        assert_eq!(start.home, expected_home, "HOME reaches {start:?}");
        assert!(start.path.contains(expected_path.as_str()), "PATH reaches {start:?}");
        assert_ne!(start.forward_target, "unset", "the forward-target marker is added to {start:?}");
        assert_ne!(start.config_count, "unset", "the lock PID injection is added to {start:?}");
    }
}

/// The auto-push after a landed commit runs its Git with the invocation's environment and publishes the commit.
#[test]
fn an_auto_push_after_a_landed_commit_runs_git_with_the_invocation_environment() {
    let fixture: Fixture = fixture("push-env-auto");
    let world: PushWorld = push_world(&fixture);
    let landed: Observed = observe(
        &mut wrapped_with_marker(&fixture, world.repo.as_path(), &["commit", "--message=second", "a.txt"]),
        b"",
    );
    let stderr: String = stderr_of(&landed);
    assert_eq!(landed.code, Some(0), "{stderr}");
    assert!(!stderr.contains("auto-push to origin failed"), "{stderr}");
    let head: String = git_head(&fixture, world.repo.as_path());
    let published: String = git_head_of(&fixture, world.bare.as_path(), "refs/heads/main");
    assert_eq!(published, head, "the landed commit is on origin: {stderr}");
    let started: Vec<Start> = starts(&world);
    assert!(
        started.iter().any(|start| { return start.service == "git-receive-pack"; }),
        "the push went through the shim: {started:?}"
    );
    assert_inherited(&fixture, started.as_slice());
    remove(&fixture);
}

/// The auto-push after a landed commit needs no system temporary directory: with `TMPDIR` naming a path that does not
/// exist the commit lands, the push and its queries still run, and the per-user scratch directory is left empty.
#[test]
fn an_auto_push_needs_no_system_temporary_directory() {
    let fixture: Fixture = fixture("push-env-auto-tmpdir");
    let world: PushWorld = push_world(&fixture);
    let unusable: PathBuf = fixture.root.join("no-such-tmpdir");
    let landed: Observed = observe(
        wrapped_with_marker(&fixture, world.repo.as_path(), &["commit", "--message=second", "a.txt"])
            .env("TMPDIR", &unusable),
        b"",
    );
    let stderr: String = stderr_of(&landed);
    assert_eq!(landed.code, Some(0), "{stderr}");
    assert!(!stderr.contains("auto-push to origin failed"), "{stderr}");
    let head: String = git_head(&fixture, world.repo.as_path());
    assert_eq!(git_head_of(&fixture, world.bare.as_path(), "refs/heads/main"), head, "the landed commit is on origin: {stderr}");
    let directory: PathBuf = fixture.root.join("home").join(SCRATCH_RELATIVE);
    assert_eq!(std::fs::read_dir(&directory).expect("the per-user scratch directory exists").count(), 0);
    assert!(!unusable.exists(), "nothing created the unusable temporary directory");
    remove(&fixture);
}

/// A manual push probes its updates, confirms them against the remote, and pushes, all with the invocation's environment.
#[test]
fn a_manual_push_probes_and_pushes_with_the_invocation_environment() {
    let fixture: Fixture = fixture("push-env-manual");
    let world: PushWorld = push_world(&fixture);
    git(&fixture, world.repo.as_path(), &["commit", "--quiet", "--message=second", "a.txt"]);
    let pushed: Observed = observe(
        &mut wrapped_with_marker(&fixture, world.repo.as_path(), &["push", "origin", "main"]),
        b"",
    );
    let stderr: String = stderr_of(&pushed);
    assert_eq!(pushed.code, Some(0), "{stderr}");
    assert!(!stderr.contains("content-unavailable"), "{stderr}");
    let head: String = git_head(&fixture, world.repo.as_path());
    assert_eq!(git_head_of(&fixture, world.bare.as_path(), "refs/heads/main"), head, "{stderr}");
    let started: Vec<Start> = starts(&world);
    let services: Vec<&str> = started.iter().map(|start| { return start.service.as_str(); }).collect();
    assert_eq!(
        services,
        ["git-receive-pack", "git-upload-pack", "git-receive-pack"],
        "probe dry run, confirmation query, then the push itself: {started:?}"
    );
    assert_inherited(&fixture, started.as_slice());
    remove(&fixture);
}

/// A manual push needs no system temporary directory: with `TMPDIR` naming a path that does not exist it still
/// pushes, the probe's private directory exists in the per-user scratch directory while the dry run is in flight,
/// and it is gone before the push itself starts and after the command ends.
#[test]
fn a_manual_push_keeps_its_probe_scratch_in_the_per_user_state_directory() {
    let fixture: Fixture = fixture("push-scratch-location");
    let world: PushWorld = push_world(&fixture);
    git(&fixture, world.repo.as_path(), &["commit", "--quiet", "--message=second", "a.txt"]);
    let unusable: PathBuf = fixture.root.join("no-such-tmpdir");
    let pushed: Observed = observe(
        wrapped_with_marker(&fixture, world.repo.as_path(), &["push", "origin", "main"]).env("TMPDIR", &unusable),
        b"",
    );
    let stderr: String = stderr_of(&pushed);
    assert_eq!(pushed.code, Some(0), "{stderr}");
    let head: String = git_head(&fixture, world.repo.as_path());
    assert_eq!(git_head_of(&fixture, world.bare.as_path(), "refs/heads/main"), head, "{stderr}");
    let started: Vec<Start> = starts(&world);
    let scratch: Vec<&str> = started.iter().map(|start| { return start.scratch.as_str(); }).collect();
    assert_eq!(scratch.len(), 3, "probe dry run, confirmation query, push: {started:?}");
    assert!(
        scratch.first().is_some_and(|names| { return names.starts_with("cli-git-push-facts-"); }),
        "the dry run sees its probe directory: {started:?}"
    );
    assert_eq!(scratch.get(2).copied(), Some(""), "the push starts after the probe directory is gone: {started:?}");
    let directory: PathBuf = fixture.root.join("home").join(SCRATCH_RELATIVE);
    let left: Vec<std::ffi::OsString> = std::fs::read_dir(&directory)
        .expect("the per-user scratch directory exists")
        .map(|entry| { return entry.expect("entry").file_name(); })
        .collect();
    assert!(left.is_empty(), "nothing is left in {}: {left:?}", directory.display());
    assert!(!unusable.exists(), "nothing created the unusable temporary directory");
    remove(&fixture);
}

/// The commit `HEAD` names in the working repository.
fn git_head(fixture: &Fixture, repo: &Path) -> String {
    return git_head_of(fixture, repo, "HEAD");
}

/// The object a reference names in a repository, or an empty text when it names none.
fn git_head_of(fixture: &Fixture, repo: &Path, reference: &str) -> String {
    let output = super::support::direct(fixture)
        .current_dir(repo)
        .args(["rev-parse", "--verify", "--quiet", reference])
        .output()
        .expect("rev-parse starts");
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
}

/// A transport that never answers must not hold the commit: the auto-push is stopped at its deadline,
///  the commit stays saved with the wrapper's own exit status, and the stop is named on the terminal.
#[test]
fn a_stalled_auto_push_is_stopped_and_the_commit_stays_saved() {
    let fixture: Fixture = fixture("push-stalled");
    let world: PushWorld = push_world(&fixture);
    let shim: PathBuf = fixture.root.join("shim.sh");
    executable(shim.as_path(), b"#!/bin/sh\nexec sleep 8\n");
    let started: std::time::Instant = std::time::Instant::now();
    let landed: Observed = observe(
        wrapped_with_marker(&fixture, world.repo.as_path(), &["commit", "--message=second", "a.txt"])
            .env("CLI_GIT_PUSH_DEADLINE_SECONDS", "1"),
        b"",
    );
    let stderr: String = stderr_of(&landed);
    assert!(started.elapsed() < std::time::Duration::from_secs(60), "{:?}", started.elapsed());
    assert_eq!(landed.code, Some(0), "{stderr}");
    assert!(stderr.contains("git push did not finish within 1 seconds and was stopped"), "{stderr}");
    assert!(stderr.contains("auto-push to origin failed"), "{stderr}");
    let head: String = git_head(&fixture, world.repo.as_path());
    assert_ne!(head, "", "the commit is saved locally: {stderr}");
    assert_eq!(git_head_of(&fixture, world.bare.as_path(), "refs/heads/main"), "", "nothing reached origin");
    remove(&fixture);
}
