//! Binary-level controls for the notices of a commit that waits (issue 1811):
//!  every wait before the push stage names what it waits for and who holds it,
//!  repeats at a steady interval, and ends at its bound with a diagnostic that names the holder.
//! The holder is this test process where a lock record or an open descriptor is enough, and a second real wrapper where
//!  the control is about two wrappers.
//! Every control runs the built executable on a disposable repository with the host's Git.

use super::commit::{COMMIT_BOUND_SECONDS, await_marker, remove_settled, wrapped_commit};
use super::support::{
    Fixture, Observed, executable, fixture, git, observe_raw, repository, stderr_of,
};
use git_policy_cli::owner_lock::{OwnerLock, try_acquire_owner_lock};
use git_policy_cli::transaction_registry::{LANDING_LOCK_NAME, RESERVATION_LOCK_NAME, ensure_transaction_root};
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// The repeat interval the controls ask for, in seconds: the smallest the variable accepts.
const NOTICE_SECONDS: &str = "1";

/// The lock wait limit the controls ask for, in seconds: long enough for a first notice and two repeats.
const LIMIT_SECONDS: &str = "4";

/// A repository with one tracked file whose worktree copy holds an uncommitted edit.
fn edited_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a");
    git(fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    return repo;
}

/// A waiting commit's command: the notice interval and the lock wait limit of the controls.
fn waiting_commit(fixture: &Fixture, repo: &Path) -> Command {
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "--message=waiter", "a.txt"]);
    command
        .env("CLI_GIT_WAIT_NOTICE_SECONDS", NOTICE_SECONDS)
        .env("CLI_GIT_LOCK_WAIT_SECONDS", LIMIT_SECONDS);
    return command;
}

/// The lines of standard error that contain `needle`.
fn lines_with<'text>(stderr: &'text str, needle: &str) -> Vec<&'text str> {
    return stderr.lines().filter(|line| { return line.contains(needle); }).collect();
}

/// Take a lock directory of the repository's registry in this test process, so this process is the holder.
fn hold_registry_lock(repo: &Path, name: &str) -> OwnerLock {
    let registry: PathBuf = repo.join(".git/cli-git-transactions");
    ensure_transaction_root(registry.as_path()).expect("registry");
    return try_acquire_owner_lock(registry.join(name).as_path(), None)
        .expect("lock attempt")
        .expect("the lock is free");
}

/// A commit that waits for the landing lock prints a notice naming the lock and this holder, repeats it every interval,
///  and at the limit stops with a diagnostic that names the holder and every way forward.
#[test]
fn a_commit_waiting_for_the_landing_lock_repeats_its_notice_and_ends_at_the_limit() {
    let fixture: Fixture = fixture("wait-notice-landing");
    let repo: PathBuf = edited_repository(&fixture);
    let held: OwnerLock = hold_registry_lock(repo.as_path(), LANDING_LOCK_NAME);
    let observed: Observed = observe_raw(&mut waiting_commit(&fixture, repo.as_path()), b"");
    let stderr: String = stderr_of(&observed);
    let holder: String = format!("process {}", std::process::id());
    let notices: Vec<&str> = lines_with(stderr.as_str(), format!("waiting for {holder} to release the landing lock").as_str());
    assert!(notices.len() >= 3, "a first notice and two repeats: {stderr}");
    assert!(notices.iter().all(|line| { return line.contains("giving up after 4 seconds"); }), "{stderr}");
    assert_ne!(observed.code, Some(0), "{stderr}");
    assert!(stderr.contains("gave up after waiting"), "{stderr}");
    assert!(stderr.contains("CLI_GIT_LOCK_WAIT_SECONDS"), "{stderr}");
    held.release().expect("release");
    remove_settled(&fixture);
}

/// A commit that waits for another transaction's reservation says so, naming the holder, and ends at the limit.
#[test]
fn a_commit_waiting_for_a_reservation_names_the_holder_and_ends_at_the_limit() {
    let fixture: Fixture = fixture("wait-notice-reservation");
    let repo: PathBuf = edited_repository(&fixture);
    let held: OwnerLock = hold_registry_lock(repo.as_path(), RESERVATION_LOCK_NAME);
    let observed: Observed = observe_raw(&mut waiting_commit(&fixture, repo.as_path()), b"");
    let stderr: String = stderr_of(&observed);
    let holder: String = format!("process {}", std::process::id());
    let notices: Vec<&str> = lines_with(stderr.as_str(), format!("waiting for {holder} to release the landing reservation").as_str());
    assert!(notices.len() >= 3, "a first notice and two repeats: {stderr}");
    assert_ne!(observed.code, Some(0), "{stderr}");
    assert!(stderr.contains(format!("{holder} holds the reservation").as_str()), "{stderr}");
    assert!(stderr.contains("CLI_GIT_LOCK_WAIT_SECONDS"), "{stderr}");
    held.release().expect("release");
    remove_settled(&fixture);
}

/// A commit that waits for a live `index.lock` holder repeats the line that names it, with the elapsed time and the limit.
#[test]
fn a_commit_waiting_for_a_live_index_lock_holder_repeats_its_notice() {
    let fixture: Fixture = fixture("wait-notice-index-live");
    let repo: PathBuf = edited_repository(&fixture);
    // This process holds the lock file open, which is what proves it alive to the wrapper.
    let lock: File = File::create(repo.join(".git/index.lock")).expect("index lock");
    let observed: Observed = observe_raw(&mut waiting_commit(&fixture, repo.as_path()), b"");
    let stderr: String = stderr_of(&observed);
    let notices: Vec<&str> = lines_with(stderr.as_str(), format!("PID {}", std::process::id()).as_str());
    assert!(notices.len() >= 3, "a first notice and two repeats: {stderr}");
    assert!(notices.iter().all(|line| { return line.contains("giving up after 4 seconds"); }), "{stderr}");
    assert_ne!(observed.code, Some(0), "{stderr}");
    assert!(stderr.contains("Gave up after waiting"), "{stderr}");
    drop(lock);
    remove_settled(&fixture);
}

/// A commit that waits for an `index.lock` nobody can be proven to hold says so while it retries.
#[test]
fn a_commit_waiting_for_an_unproven_index_lock_says_so_while_it_retries() {
    let fixture: Fixture = fixture("wait-notice-index-unproven");
    let repo: PathBuf = edited_repository(&fixture);
    std::fs::write(repo.join("cli-git.config.jsonc"), "{ \"indexLock\": { \"unprovenOwnerTimeoutMs\": 3500 } }\n")
        .expect("configuration");
    std::fs::write(repo.join(".git/index.lock"), b"").expect("index lock nobody holds");
    let observed: Observed = observe_raw(&mut waiting_commit(&fixture, repo.as_path()), b"");
    let stderr: String = stderr_of(&observed);
    let notices: Vec<&str> = lines_with(stderr.as_str(), "cannot prove alive");
    assert!(notices.len() >= 2, "a first notice and a repeat: {stderr}");
    assert!(notices.iter().all(|line| { return line.contains("indexLock.unprovenOwnerTimeoutMs"); }), "{stderr}");
    assert!(stderr.contains("index-lock-unproven-owner"), "{stderr}");
    remove_settled(&fixture);
}

/// A commit whose Markdown linter runs long prints a notice naming the linter and the file while it waits.
#[test]
fn a_commit_waiting_for_the_markdown_linter_names_it() {
    let fixture: Fixture = fixture("wait-notice-linter");
    let repo: PathBuf = edited_repository(&fixture);
    let program: PathBuf = fixture.root.join("slow-linter");
    executable(program.as_path(), b"#!/bin/sh\nsleep 3\ncat\n");
    let directory: PathBuf = fixture.root.join("linter");
    std::fs::create_dir_all(&directory).expect("linter directory");
    std::os::unix::fs::symlink(&program, directory.join("monochromatic-lint")).expect("link");
    std::fs::write(repo.join(".lfsconfig"), b"[lfs]\n\turl = https://lfs.example.com/org/repo\n").expect("lfsconfig");
    std::fs::write(
        repo.join("cli-git.config.jsonc"),
        "{ \"policies\": { \"final-newline\": \"off\", \"markdown/autofix\": \"warn\" } }\n",
    )
    .expect("configuration");
    std::fs::write(repo.join("doc.md"), b"# Title\n").expect("markdown");
    let mut path: std::ffi::OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(directory);
    path.push(":/usr/bin:/bin");
    let mut command: Command = super::support::bounded_for(
        &fixture,
        fixture.root.join("bin/git").as_path(),
        path.as_os_str(),
        COMMIT_BOUND_SECONDS,
    );
    command
        .current_dir(&repo)
        .args(["commit", "--message=waiter", "doc.md"])
        .env("CLI_GIT_WAIT_NOTICE_SECONDS", NOTICE_SECONDS);
    let observed: Observed = observe_raw(&mut command, b"");
    let stderr: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{stderr}");
    let notices: Vec<&str> = lines_with(stderr.as_str(), "waiting for monochromatic-lint");
    assert!(notices.len() >= 2, "a first notice and a repeat: {stderr}");
    assert!(notices.iter().all(|line| { return line.contains("doc.md"); }), "{stderr}");
    remove_settled(&fixture);
}

/// Two real wrappers: the second commit queues behind the first, which holds the landing lock, and prints the notice
///  naming the lock and the first wrapper until the first lets go; then both commits land.
#[test]
fn the_second_of_two_real_wrappers_prints_the_notice_until_the_first_lets_go() {
    let fixture: Fixture = fixture("wait-notice-two-wrappers");
    let repo: PathBuf = edited_repository(&fixture);
    std::fs::write(repo.join("b.txt"), b"b\n").expect("b");
    git(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("marker directory");
    let first: Child = wrapped_commit(&fixture, repo.as_path(), &["commit", "--message=first", "a.txt"])
        .env("CLI_GIT_TEST_ONLY_PHASE_SIGNAL", format!("landing-locked:pause:{}", markers.display()))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the first commit starts");
    await_marker(markers.join("landing-locked.reached").as_path());
    let second: Child = wrapped_commit(&fixture, repo.as_path(), &["commit", "--message=second", "b.txt"])
        .env("CLI_GIT_WAIT_NOTICE_SECONDS", NOTICE_SECONDS)
        .env("CLI_GIT_LOCK_WAIT_SECONDS", "60")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the second commit starts");
    // The second commit has waited long enough for a first notice and a repeat before the first one lets go.
    std::thread::sleep(std::time::Duration::from_millis(3500));
    std::fs::write(markers.join("landing-locked.release"), b"").expect("release the first commit");
    let finished_first = first.wait_with_output().expect("the first commit ends");
    let finished_second = second.wait_with_output().expect("the second commit ends");
    let stderr: String = String::from_utf8_lossy(finished_second.stderr.as_slice()).into_owned();
    assert_eq!(finished_first.status.code(), Some(0), "{}", String::from_utf8_lossy(finished_first.stderr.as_slice()));
    assert_eq!(finished_second.status.code(), Some(0), "{stderr}");
    let notices: Vec<&str> = lines_with(stderr.as_str(), "waiting for process");
    assert!(notices.len() >= 2, "a first notice and a repeat: {stderr}");
    assert!(notices.iter().all(|line| { return line.contains("the landing lock"); }), "{stderr}");
    remove_settled(&fixture);
}
