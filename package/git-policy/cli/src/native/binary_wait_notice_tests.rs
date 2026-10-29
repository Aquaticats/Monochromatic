//! Binary-level controls for the notices of a commit that waits (issue 1811):
//!  every wait before the push stage names what it waits for and who holds it,
//!  repeats at a steady interval, and ends at its bound with a diagnostic that names the holder.
//! The holder is this test process where a lock record or an open descriptor is enough, and a second real wrapper where
//!  the control is about two wrappers.
//! A control of a repeated notice reads the wrapper's standard error line by line and ends the wait on the notice that
//!  counts, so the count holds however slowly a loaded host runs the wrapper; a control of a bound only asks for a
//!  notice before the refusal, which the wait guarantees.
//! Every control runs the built executable on a disposable repository with the host's Git.

use super::commit::{COMMIT_BOUND_SECONDS, await_marker, remove_settled, wrapped_commit};
use super::support::{
    Fixture, Observed, bounded_for, executable, fixture, git, observe_raw, repository, stderr_of,
};
use git_policy_cli::owner_lock::{OwnerLock, try_acquire_owner_lock};
use git_policy_cli::transaction_registry::{LANDING_LOCK_NAME, RESERVATION_LOCK_NAME, ensure_transaction_root};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// The repeat interval the controls ask for, in seconds: the smallest the variable accepts.
const NOTICE_SECONDS: &str = "1";

/// The lock wait limit of a control that ends its wait on a notice: far beyond what the control needs.
const GENEROUS_LIMIT_SECONDS: &str = "120";

/// The lock wait limit of a control of a bound, in seconds: long enough for the first notice.
const BOUND_SECONDS: &str = "2";

/// What a watched commit showed its caller.
struct Watched {
    /// Exit code, absent when a signal ended it.
    code: Option<i32>,
    /// Every line of standard error.
    stderr: String,
}

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

/// A waiting commit's command: the notice interval of the controls and the given lock wait limit.
fn waiting_commit(fixture: &Fixture, repo: &Path, limit_seconds: &str) -> Command {
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "--message=waiter", "a.txt"]);
    command
        .env("CLI_GIT_WAIT_NOTICE_SECONDS", NOTICE_SECONDS)
        .env("CLI_GIT_LOCK_WAIT_SECONDS", limit_seconds);
    return command;
}

/// The lines of standard error that contain `needle`.
fn lines_with<'text>(stderr: &'text str, needle: &str) -> Vec<&'text str> {
    return stderr.lines().filter(|line| { return line.contains(needle); }).collect();
}

/// What:
///  Run `command` with standard error read line by line, and run `on_count` once when `count` lines containing `needle`
///  have been read.
/// Why:
///   The action ends the wait the notices describe (it releases the lock the commit waits for),
///  so the wait stops on the counted notice and not on a timer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const line of child.stderr.lines()) { if (line.includes(needle) && ++seen === count) onCount(); }
/// ```
fn watch_notices(command: &mut Command, needle: &str, count: usize, on_count: impl FnOnce()) -> Watched {
    let mut child: Child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the commit starts");
    let reader: BufReader<std::process::ChildStderr> = BufReader::new(child.stderr.take().expect("piped"));
    let mut stderr: String = String::new();
    let mut seen: usize = 0;
    let mut action: Option<_> = Some(on_count);
    for read in reader.lines() {
        let Ok(line) = read else {
            break;
        };
        stderr.push_str(line.as_str());
        stderr.push('\n');
        if line.contains(needle) {
            seen += 1;
            if seen >= count && let Some(act) = action.take() {
                act();
            }
        }
    }
    let status = child.wait().expect("the commit ends");
    return Watched { code: status.code(), stderr };
}

/// Take a lock directory of the repository's registry in this test process, so this process is the holder.
fn hold_registry_lock(repo: &Path, name: &str) -> OwnerLock {
    let registry: PathBuf = repo.join(".git/cli-git-transactions");
    ensure_transaction_root(registry.as_path()).expect("registry");
    return try_acquire_owner_lock(registry.join(name).as_path(), None)
        .expect("lock attempt")
        .expect("the lock is free");
}

/// A commit that waits for the landing lock prints a notice naming the lock and this holder and repeats it every
///  interval; the third notice makes the holder let go, and the commit then lands.
#[test]
fn a_commit_waiting_for_the_landing_lock_repeats_its_notice() {
    let fixture: Fixture = fixture("wait-notice-landing");
    let repo: PathBuf = edited_repository(&fixture);
    let held: OwnerLock = hold_registry_lock(repo.as_path(), LANDING_LOCK_NAME);
    let needle: String = format!("waiting for process {} to release the landing lock", std::process::id());
    let watched: Watched = watch_notices(
        &mut waiting_commit(&fixture, repo.as_path(), GENEROUS_LIMIT_SECONDS),
        needle.as_str(),
        3,
        move || { held.release().expect("release"); },
    );
    let notices: Vec<&str> = lines_with(watched.stderr.as_str(), needle.as_str());
    assert!(notices.len() >= 3, "a first notice and two repeats: {}", watched.stderr);
    assert!(notices.iter().all(|line| { return line.contains("giving up after 120 seconds"); }), "{}", watched.stderr);
    assert_eq!(watched.code, Some(0), "{}", watched.stderr);
    remove_settled(&fixture);
}

/// A commit that waits for the landing lock past its limit was announced first, and stops with a diagnostic that names
///  the holder and every way forward.
#[test]
fn a_commit_that_gives_up_on_the_landing_lock_names_the_holder_and_every_way_forward() {
    let fixture: Fixture = fixture("wait-notice-landing-bound");
    let repo: PathBuf = edited_repository(&fixture);
    let held: OwnerLock = hold_registry_lock(repo.as_path(), LANDING_LOCK_NAME);
    let observed: Observed = observe_raw(&mut waiting_commit(&fixture, repo.as_path(), BOUND_SECONDS), b"");
    let stderr: String = stderr_of(&observed);
    let needle: String = format!("waiting for process {} to release the landing lock", std::process::id());
    assert!(!lines_with(stderr.as_str(), needle.as_str()).is_empty(), "{stderr}");
    assert_ne!(observed.code, Some(0), "{stderr}");
    assert!(stderr.contains("gave up after waiting"), "{stderr}");
    assert!(stderr.contains(format!("process {} still holds", std::process::id()).as_str()), "{stderr}");
    assert!(stderr.contains("let that process finish and run the command again"), "{stderr}");
    assert!(stderr.contains("check with `ps`"), "{stderr}");
    assert!(stderr.contains("CLI_GIT_LOCK_WAIT_SECONDS"), "{stderr}");
    held.release().expect("release");
    remove_settled(&fixture);
}

/// A commit that waits for another transaction's reservation says so, naming the holder, and repeats it;
///  the third notice makes the holder let go, and the commit then lands.
#[test]
fn a_commit_waiting_for_a_reservation_names_the_holder_and_repeats() {
    let fixture: Fixture = fixture("wait-notice-reservation");
    let repo: PathBuf = edited_repository(&fixture);
    let held: OwnerLock = hold_registry_lock(repo.as_path(), RESERVATION_LOCK_NAME);
    let needle: String = format!("waiting for process {} to release the landing reservation", std::process::id());
    let watched: Watched = watch_notices(
        &mut waiting_commit(&fixture, repo.as_path(), GENEROUS_LIMIT_SECONDS),
        needle.as_str(),
        3,
        move || { held.release().expect("release"); },
    );
    assert!(lines_with(watched.stderr.as_str(), needle.as_str()).len() >= 3, "{}", watched.stderr);
    assert_eq!(watched.code, Some(0), "{}", watched.stderr);
    remove_settled(&fixture);
}

/// A commit that waits for a reservation past its limit was announced first, and stops with the holder named.
#[test]
fn a_commit_that_gives_up_on_a_reservation_names_the_holder() {
    let fixture: Fixture = fixture("wait-notice-reservation-bound");
    let repo: PathBuf = edited_repository(&fixture);
    let held: OwnerLock = hold_registry_lock(repo.as_path(), RESERVATION_LOCK_NAME);
    let observed: Observed = observe_raw(&mut waiting_commit(&fixture, repo.as_path(), BOUND_SECONDS), b"");
    let stderr: String = stderr_of(&observed);
    let holder: String = format!("process {}", std::process::id());
    assert!(
        !lines_with(stderr.as_str(), format!("waiting for {holder} to release the landing reservation").as_str()).is_empty(),
        "{stderr}"
    );
    assert_ne!(observed.code, Some(0), "{stderr}");
    assert!(stderr.contains(format!("{holder} holds the reservation").as_str()), "{stderr}");
    assert!(stderr.contains("CLI_GIT_LOCK_WAIT_SECONDS"), "{stderr}");
    held.release().expect("release");
    remove_settled(&fixture);
}

/// A commit that waits for a live `index.lock` holder repeats the line that names it, with the elapsed time and the
///  limit; the second notice makes the holder let go, and the commit then lands.
#[test]
fn a_commit_waiting_for_a_live_index_lock_holder_repeats_its_notice() {
    let fixture: Fixture = fixture("wait-notice-index-live");
    let repo: PathBuf = edited_repository(&fixture);
    let lock_path: PathBuf = repo.join(".git/index.lock");
    // This process holds the lock file open, which is what proves it alive to the wrapper.
    let lock: File = File::create(&lock_path).expect("index lock");
    let needle: String = format!("waiting for PID {}", std::process::id());
    let watched: Watched = watch_notices(
        &mut waiting_commit(&fixture, repo.as_path(), GENEROUS_LIMIT_SECONDS),
        needle.as_str(),
        2,
        move || {
            drop(lock);
            std::fs::remove_file(&lock_path).expect("the holder lets go");
        },
    );
    let notices: Vec<&str> = lines_with(watched.stderr.as_str(), needle.as_str());
    assert!(notices.len() >= 2, "a first notice and a repeat: {}", watched.stderr);
    assert!(notices.iter().all(|line| { return line.contains("giving up after 120 seconds"); }), "{}", watched.stderr);
    assert_eq!(watched.code, Some(0), "{}", watched.stderr);
    remove_settled(&fixture);
}

/// A commit that waits for a live `index.lock` holder past its limit was announced (the first line and a repeat)
///  and stops with the holder, the limit and the way forward named.
#[test]
fn a_commit_that_gives_up_on_a_live_index_lock_holder_names_it() {
    let fixture: Fixture = fixture("wait-notice-index-live-bound");
    let repo: PathBuf = edited_repository(&fixture);
    let lock: File = File::create(repo.join(".git/index.lock")).expect("index lock");
    let observed: Observed = observe_raw(&mut waiting_commit(&fixture, repo.as_path(), "3"), b"");
    let stderr: String = stderr_of(&observed);
    let notices: Vec<&str> = lines_with(stderr.as_str(), format!("waiting for PID {}", std::process::id()).as_str());
    assert!(notices.len() >= 2, "a first notice and a repeat: {stderr}");
    assert_ne!(observed.code, Some(0), "{stderr}");
    assert!(stderr.contains("Gave up after waiting"), "{stderr}");
    assert!(stderr.contains("Look at that process, or raise the limit."), "{stderr}");
    drop(lock);
    remove_settled(&fixture);
}

/// A commit that waits for an `index.lock` nobody can be proven to hold says so while it retries;
///  the second notice removes the lock, and the commit then lands.
#[test]
fn a_commit_waiting_for_an_unproven_index_lock_says_so_while_it_retries() {
    let fixture: Fixture = fixture("wait-notice-index-unproven");
    let repo: PathBuf = edited_repository(&fixture);
    std::fs::write(repo.join("cli-git.config.jsonc"), "{ \"indexLock\": { \"unprovenOwnerTimeoutMs\": 120000 } }\n")
        .expect("configuration");
    let lock_path: PathBuf = repo.join(".git/index.lock");
    std::fs::write(&lock_path, b"").expect("index lock nobody holds");
    let watched: Watched = watch_notices(
        &mut waiting_commit(&fixture, repo.as_path(), GENEROUS_LIMIT_SECONDS),
        "cannot prove alive",
        2,
        move || { std::fs::remove_file(&lock_path).expect("the stale lock is removed"); },
    );
    let notices: Vec<&str> = lines_with(watched.stderr.as_str(), "cannot prove alive");
    assert!(notices.len() >= 2, "a first notice and a repeat: {}", watched.stderr);
    assert!(notices.iter().all(|line| { return line.contains("indexLock.unprovenOwnerTimeoutMs"); }), "{}", watched.stderr);
    assert_eq!(watched.code, Some(0), "{}", watched.stderr);
    remove_settled(&fixture);
}

/// A commit whose Markdown linter runs long prints a notice naming the linter and the file while it waits;
///  the second notice releases the linter, and the commit then lands.
#[test]
fn a_commit_waiting_for_the_markdown_linter_names_it() {
    let fixture: Fixture = fixture("wait-notice-linter");
    let repo: PathBuf = edited_repository(&fixture);
    let release: PathBuf = fixture.root.join("linter-release");
    let program: PathBuf = fixture.root.join("slow-linter");
    executable(
        program.as_path(),
        format!("#!/bin/sh\nwhile [ ! -e '{}' ]; do sleep 0.05; done\ncat\n", release.display()).as_bytes(),
    );
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
    let mut command: Command = bounded_for(
        &fixture,
        fixture.root.join("bin/git").as_path(),
        path.as_os_str(),
        COMMIT_BOUND_SECONDS,
    );
    command
        .current_dir(&repo)
        .args(["commit", "--message=waiter", "doc.md"])
        .env("CLI_GIT_WAIT_NOTICE_SECONDS", NOTICE_SECONDS);
    let watched: Watched = watch_notices(&mut command, "waiting for monochromatic-lint", 2, move || {
        std::fs::write(&release, b"").expect("release the linter");
    });
    assert_eq!(watched.code, Some(0), "{}", watched.stderr);
    let notices: Vec<&str> = lines_with(watched.stderr.as_str(), "waiting for monochromatic-lint");
    assert!(notices.len() >= 2, "a first notice and a repeat: {}", watched.stderr);
    assert!(notices.iter().all(|line| { return line.contains("doc.md"); }), "{}", watched.stderr);
    remove_settled(&fixture);
}

/// Two real wrappers: the second commit queues behind the first, which holds the landing lock, and prints the notice
///  naming the lock until the first lets go; the second notice makes the first wrapper continue, and both commits land.
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
    let release: PathBuf = markers.join("landing-locked.release");
    let mut second: Command =
        wrapped_commit(&fixture, repo.as_path(), &["commit", "--message=second", "b.txt"]);
    second
        .env("CLI_GIT_WAIT_NOTICE_SECONDS", NOTICE_SECONDS)
        .env("CLI_GIT_LOCK_WAIT_SECONDS", GENEROUS_LIMIT_SECONDS);
    let watched: Watched = watch_notices(&mut second, "waiting for process", 2, move || {
        std::fs::write(&release, b"").expect("release the first commit");
    });
    let finished_first = first.wait_with_output().expect("the first commit ends");
    assert_eq!(finished_first.status.code(), Some(0), "{}", String::from_utf8_lossy(finished_first.stderr.as_slice()));
    assert_eq!(watched.code, Some(0), "{}", watched.stderr);
    let notices: Vec<&str> = lines_with(watched.stderr.as_str(), "waiting for process");
    assert!(notices.len() >= 2, "a first notice and a repeat: {}", watched.stderr);
    assert!(notices.iter().all(|line| { return line.contains("the landing lock"); }), "{}", watched.stderr);
    remove_settled(&fixture);
}
