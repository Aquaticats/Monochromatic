//! Binary-level controls for `git commit` through the native wrapper:
//!  the transaction lands a commit, applies the policies, reports what blocked it, leaves nothing behind,
//!  and replays or fails a commit that lost a landing race.
//! Every control runs the built executable on a disposable repository with the host's Git.

use super::support::{
    Fixture, Observed, bounded_for, fixture, git, observe, porcelain, remove, repository, run_direct,
    stderr_of, wrapper_first_path,
};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// The bound of one commit under host load, in seconds; the shared 5-second bound is for single Git commands.
pub(super) const COMMIT_BOUND_SECONDS: &str = "120";

/// Remove the fixture once nothing writes into it any more.
///  A landed commit may leave a maintenance child finishing for a moment under load, and removing a directory that
///  still gains entries fails with "Directory not empty"; the removal is tried again for up to five seconds.
pub(super) fn remove_settled(fixture: &Fixture) {
    for _ in 0..50 {
        if std::fs::remove_dir_all(&fixture.root).is_ok() || !fixture.root.exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    remove(fixture);
}

/// A command for the fixture's `bin/git` with the wrapper first on PATH and the commit bound.
pub(super) fn wrapped_commit(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Command {
    let mut command: Command = bounded_for(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        COMMIT_BOUND_SECONDS,
    );
    command.current_dir(directory).args(arguments);
    return command;
}

/// Run one wrapped commit command to its end.
pub(super) fn run_commit(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Observed {
    return observe(&mut wrapped_commit(fixture, directory, arguments), b"");
}

/// The text of a Git query against the repository.
pub(super) fn git_text(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> String {
    let output = git(fixture, repo, arguments);
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
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

/// Make the automatic maintenance a landing starts run to its end before the wrapper returns.
///  Detached, Git 2.56.0 repacks even a repository this small in the background, and its `tmp_pack_*` and `tmp_idx_*`
///  files are what `git count-objects -v` reports as garbage while the repack runs; a check made right after the
///  wrapper returns then races with it.
pub(super) fn keep_maintenance_in_the_foreground(fixture: &Fixture, repo: &Path) {
    git(fixture, repo, &["config", "maintenance.autoDetach", "false"]);
}

/// The sorted file names in a pack directory, for failure messages that name what a garbage count stands for.
fn pack_entries(pack_directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(pack_directory)
        .map(|entries| {
            return entries
                .flatten()
                .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
                .collect();
        })
        .unwrap_or_default();
    names.sort();
    return names;
}

/// What a finished transaction must leave: no transaction directory, no lock, no shadow repository, no `.keep`.
pub(super) fn assert_nothing_left(fixture: &Fixture, repo: &Path) {
    let git_dir: PathBuf = repo.join(".git");
    let registry: Vec<String> = std::fs::read_dir(git_dir.join("cli-git-transactions"))
        .map(|entries| {
            return entries
                .flatten()
                .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
                .collect();
        })
        .unwrap_or_default();
    assert_eq!(registry, Vec::<String>::new(), "no transaction or lock stays in the registry");
    assert!(!git_dir.join("index.lock").exists(), "the real index lock is released");
    let shadows: Vec<String> = std::fs::read_dir(git_dir.join("cli-git/shadow"))
        .map(|entries| {
            return entries
                .flatten()
                .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
                .collect();
        })
        .unwrap_or_default();
    assert_eq!(shadows, Vec::<String>::new(), "no shadow repository stays");
    let pack_directory: PathBuf = git_dir.join("objects/pack");
    let keeps: String = git_text(fixture, repo, &["count-objects", "-v"]);
    assert!(keeps.contains("garbage: 0"), "{keeps}pack directory: {:?}", pack_entries(pack_directory.as_path()));
    let kept: Vec<String> = std::fs::read_dir(pack_directory)
        .map(|entries| {
            return entries
                .flatten()
                .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
                .filter(|name| { return name.ends_with(".keep"); })
                .collect();
        })
        .unwrap_or_default();
    assert_eq!(kept, Vec::<String>::new(), "no pack keeps the commit's objects alive");
}

/// How many incompressible files the maintenance control gives the repack to pack.
const BULK_FILE_COUNT: usize = 32;

/// How many bytes each incompressible file holds.
const BULK_FILE_BYTES: usize = 256 * 1024;

/// Pseudo-random bytes from a xorshift generator seeded with `seed`, which deflate cannot shrink.
fn incompressible(seed: u64) -> Vec<u8> {
    let mut state: u64 = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut bytes: Vec<u8> = Vec::with_capacity(BULK_FILE_BYTES);
    while bytes.len() < BULK_FILE_BYTES {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        bytes.extend_from_slice(&state.to_le_bytes());
    }
    return bytes;
}

/// With detaching off, the maintenance a landing starts has finished when the wrapper returns:
///  the second landing leaves more packs than `gc.autoPackLimit` allows, so Git repacks the loose objects,
///  and that repack is complete, with no temporary file and no maintenance lock, when the wrapper returns.
#[test]
fn a_landing_returns_after_its_automatic_maintenance_finished() {
    let fixture: Fixture = fixture("commit-foreground-maintenance");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    git(&fixture, repo.as_path(), &["config", "gc.autoPackLimit", "1"]);
    // Maximum compression of incompressible files keeps the repack running well past the wrapper's own last steps.
    git(&fixture, repo.as_path(), &["config", "pack.compression", "9"]);
    std::fs::create_dir(repo.join("filler")).expect("filler directory");
    for number in 0..BULK_FILE_COUNT {
        std::fs::write(repo.join(format!("filler/{number}.bin")), incompressible(number as u64)).expect("filler file");
    }
    git(&fixture, repo.as_path(), &["add", "--", "filler"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=filler"]);
    let first: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=second", "a.txt"]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    std::fs::write(repo.join("a.txt"), b"one\ntwo\nthree\n").expect("edit a again");
    let second: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=third", "a.txt"]);
    assert_eq!(second.code, Some(0), "{}", stderr_of(&second));
    let counts: String = git_text(&fixture, repo.as_path(), &["count-objects", "-v"]);
    assert!(counts.contains("count: 0\n"), "the loose objects were packed: {counts}");
    assert!(counts.contains("garbage: 0\n"), "{counts}");
    assert!(!repo.join(".git/objects/maintenance.lock").exists(), "no maintenance still holds its lock");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An explicit-path commit lands the worktree content of the named path once and leaves nothing behind.
#[test]
fn an_explicit_path_commit_lands_once_and_leaves_nothing() {
    let fixture: Fixture = fixture("commit-explicit");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=second", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "3");
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(), "second");
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A final-newline correction lands in the commit and in the worktree copy, and the summary names the path.
#[test]
fn a_final_newline_correction_lands_in_the_commit_and_the_worktree() {
    let fixture: Fixture = fixture("commit-newline");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("a.txt"), b"one\ntwo").expect("edit without the final newline");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=second", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("worktree copy"), b"one\ntwo\n");
    assert!(
        stderr_of(&landed).contains("\"type\":\"fix-summary\",\"trigger\":\"pre-forward\",\"passes\":1,\"changedPaths\":[\"a.txt\"]"),
        "{}",
        stderr_of(&landed)
    );
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A dry run commits nothing and starts no transaction.
#[test]
fn a_dry_run_commits_nothing() {
    let fixture: Fixture = fixture("commit-dry-run");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let dry: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--dry-run", "--message=x", "a.txt"]);
    assert_eq!(dry.code, Some(0), "{}", stderr_of(&dry));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert!(!repo.join(".git/cli-git").exists(), "a dry run creates no shadow area");
    remove_settled(&fixture);
}

/// `commit -a` commits every tracked change, including a deletion.
#[test]
fn a_commit_of_all_tracked_changes_lands_them() {
    let fixture: Fixture = fixture("commit-all");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("b.txt"), b"b\n").expect("b");
    git(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=more"]);
    std::fs::remove_file(repo.join("b.txt")).expect("remove b");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "-a", "--no-only", "--message=all"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["ls-tree", "--name-only", "HEAD"]).trim(),
        "a.txt"
    );
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A commit that names no path while commit-only enforcement is active is rejected with exit status 1.
#[test]
fn a_blocking_finding_stops_the_commit_with_status_one() {
    let fixture: Fixture = fixture("commit-blocked");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let blocked: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=x"]);
    assert_eq!(blocked.code, Some(1), "{}", stderr_of(&blocked));
    assert!(stderr_of(&blocked).contains("commit-only/pathspec-required"), "{}", stderr_of(&blocked));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(porcelain(&fixture, repo.as_path()), "M  a.txt\n");
    remove_settled(&fixture);
}

/// A correction that makes the commit identical to its base creates no commit and reconciles the copies.
#[test]
fn a_correction_that_restores_the_base_is_a_no_change_normalization() {
    let fixture: Fixture = fixture("commit-no-change");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("a.txt"), b"one").expect("remove the final newline");
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let result: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=x", "a.txt"]);
    assert_eq!(result.code, Some(1), "{}", stderr_of(&result));
    assert!(
        stderr_of(&result).contains("\"code\":\"commit-normalization/no-change\""),
        "{}",
        stderr_of(&result)
    );
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("worktree copy"), b"one\n");
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An amend replaces the head commit, and an empty commit lands only with `--allow-empty`.
#[test]
fn an_amend_replaces_the_head_and_an_empty_commit_needs_its_flag() {
    let fixture: Fixture = fixture("commit-amend");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let amended: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--amend", "--message=amended", "a.txt"],
    );
    assert_eq!(amended.code, Some(0), "{}", stderr_of(&amended));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "2");
    assert_eq!(git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(), "amended");
    let empty: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--allow-empty", "--no-only", "--message=empty"],
    );
    assert_eq!(empty.code, Some(0), "{}", stderr_of(&empty));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "3");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An unknown option reaches Git, which refuses it; nothing lands.
#[test]
fn an_unknown_option_is_refused_by_git_and_nothing_lands() {
    let fixture: Fixture = fixture("commit-unknown-option");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--no-such-option", "a.txt"]);
    let direct: Observed = run_direct(&fixture, repo.as_path(), &["commit", "--no-such-option", "a.txt"]);
    assert_eq!(refused.code, direct.code, "{}", stderr_of(&refused));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    remove_settled(&fixture);
}

/// A configured `pre-commit` hook that fails ends the commit with Git's status and leaves nothing behind.
#[test]
fn a_failing_pre_commit_hook_stops_the_commit_with_its_status() {
    let fixture: Fixture = fixture("commit-hook-fails");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    super::support::configure_pre_commit_hook(&fixture, repo.as_path(), b"#!/bin/sh\necho rejected by the hook >&2\nexit 1\n");
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let rejected: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=x", "a.txt"]);
    assert_eq!(rejected.code, Some(1), "{}", stderr_of(&rejected));
    assert!(stderr_of(&rejected).contains("rejected by the hook"), "{}", stderr_of(&rejected));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A configured `pre-commit` hook that re-stages the file through the wrapper has its change kept in the commit.
#[test]
fn a_pre_commit_hook_restaging_a_file_is_kept() {
    let fixture: Fixture = fixture("commit-hook-restages");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    super::support::configure_pre_commit_hook(
        &fixture,
        repo.as_path(),
        b"#!/bin/sh\nprintf 'hooked\\n' >> a.txt\ngit add -- a.txt\n",
    );
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=second", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]),
        "one\ntwo\nhooked\n"
    );
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// Waits until a phase marker exists, or fails the control.
pub(super) fn await_marker(path: &Path) {
    let started: Instant = Instant::now();
    while !path.exists() {
        assert!(started.elapsed() < Duration::from_secs(90), "the marker {} never appeared", path.display());
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A commit paused after preparation is overtaken by another commit; it replays onto the moved branch and lands.
///  `configuration` is the text of a repository configuration file, when the control needs one.
fn replayed_commit_stderr(name: &str, configuration: Option<&str>) -> String {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = edited_repository(&fixture, "repo");
    if let Some(text) = configuration {
        std::fs::write(repo.join("cli-git.config.jsonc"), text).expect("repository configuration");
    }
    std::fs::write(repo.join("b.txt"), b"b\n").expect("b");
    git(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("marker directory");
    let slow: Child = wrapped_commit(&fixture, repo.as_path(), &["commit", "--message=slow", "a.txt"])
        .env("CLI_GIT_TEST_ONLY_PHASE_SIGNAL", format!("preparation-done:pause:{}", markers.display()))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the slow commit starts");
    await_marker(markers.join("preparation-done.reached").as_path());
    let fast: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=fast", "b.txt"]);
    assert_eq!(fast.code, Some(0), "{}", stderr_of(&fast));
    std::fs::write(markers.join("preparation-done.release"), b"").expect("release the slow commit");
    let finished = slow.wait_with_output().expect("the slow commit ends");
    let stderr: String = String::from_utf8_lossy(finished.stderr.as_slice()).into_owned();
    assert_eq!(finished.status.code(), Some(0), "{stderr}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "4");
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:b.txt"]), "b\n");
    remove_settled(&fixture);
    return stderr;
}

/// One lost race replays and lands, and under the default threshold also takes the starvation reservation.
#[test]
fn a_commit_that_loses_the_landing_race_replays_and_lands() {
    let stderr: String = replayed_commit_stderr("commit-replay", None);
    assert!(stderr.contains("\"type\":\"landing-race-lost\",\"attempt\":1"), "{stderr}");
    assert!(stderr.contains("\"type\":\"commit-replayed\""), "{stderr}");
    assert!(stderr.contains("\"type\":\"landing-reserved\",\"lostRaces\":1"), "{stderr}");
}

/// With `landing.reserveAfterLostRaces` at 2, one lost race replays without taking the reservation.
#[test]
fn fewer_lost_races_than_configured_take_no_reservation() {
    let stderr: String = replayed_commit_stderr("commit-no-reserve", Some("{ \"landing\": { \"reserveAfterLostRaces\": 2 } }\n"));
    assert!(stderr.contains("\"type\":\"commit-replayed\""), "{stderr}");
    assert!(!stderr.contains("\"type\":\"landing-reserved\""), "{stderr}");
}

/// A commit that overlaps the commit that won fails with `concurrent-commit/replay-conflict`, and nothing lands.
#[test]
fn a_commit_that_overlaps_the_winner_ends_in_a_replay_conflict() {
    let fixture: Fixture = fixture("commit-conflict");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("marker directory");
    let slow: Child = wrapped_commit(&fixture, repo.as_path(), &["commit", "--message=slow", "a.txt"])
        .env("CLI_GIT_TEST_ONLY_PHASE_SIGNAL", format!("preparation-done:pause:{}", markers.display()))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the slow commit starts");
    await_marker(markers.join("preparation-done.reached").as_path());
    // The winner rewrites the same line the slow commit changes, after the slow commit captured its bytes.
    std::fs::write(repo.join("a.txt"), b"other\n").expect("winner edit");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=winner"]);
    let winning: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(markers.join("preparation-done.release"), b"").expect("release the slow commit");
    let finished = slow.wait_with_output().expect("the slow commit ends");
    let stderr: String = String::from_utf8_lossy(finished.stderr.as_slice()).into_owned();
    assert_eq!(finished.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("\"code\":\"concurrent-commit/replay-conflict\""), "{stderr}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), winning.trim());
    remove_settled(&fixture);
}

/// A termination signal sent to the wrapper while a `pre-commit` hook runs ends the hook's Git, leaves no
///  transaction behind, and exits with 128 plus the signal number.
#[test]
fn a_termination_signal_during_a_hook_ends_the_commit_and_leaves_nothing() {
    let fixture: Fixture = fixture("commit-signal");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let started: PathBuf = fixture.root.join("hook-started");
    super::support::configure_pre_commit_hook(
        &fixture,
        repo.as_path(),
        format!("#!/bin/sh\ntouch '{}'\nsleep 60\n", started.display()).as_bytes(),
    );
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let child: Child = wrapped_commit(&fixture, repo.as_path(), &["commit", "--message=x", "a.txt"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the commit starts");
    await_marker(started.as_path());
    let signalled = Command::new("kill")
        .args(["-s", "TERM"])
        .arg(child.id().to_string())
        .status()
        .expect("kill starts");
    assert!(signalled.success());
    let finished = child.wait_with_output().expect("the commit ends");
    assert_eq!(finished.status.code(), Some(143), "{}", String::from_utf8_lossy(finished.stderr.as_slice()));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// `git commit --patch` stages the hunks chosen on standard input into the private index, and only those land.
#[test]
fn a_patch_selection_lands_the_chosen_hunk() {
    let fixture: Fixture = fixture("commit-patch");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let landed: Observed = observe(
        &mut wrapped_commit(&fixture, repo.as_path(), &["commit", "--patch", "--message=chosen"]),
        b"y\n",
    );
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(), "chosen");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// `git commit --interactive` runs the file menu against the private index the same way.
#[test]
fn an_interactive_selection_lands_the_chosen_file() {
    let fixture: Fixture = fixture("commit-interactive");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let landed: Observed = observe(
        &mut wrapped_commit(&fixture, repo.as_path(), &["commit", "--interactive", "--message=chosen"]),
        b"u\n1\n\nq\n",
    );
    assert_eq!(landed.code, Some(0), "{}{}", stderr_of(&landed), String::from_utf8_lossy(landed.stdout.as_slice()));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// With `--allow-empty`, a correction that restores the base still creates the commit the user asked for.
#[test]
fn an_allowed_empty_commit_is_not_a_no_change_normalization() {
    let fixture: Fixture = fixture("commit-allow-empty-normalized");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("a.txt"), b"one").expect("remove the final newline");
    let before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let landed: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--allow-empty", "--message=empty on purpose", "a.txt"],
    );
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "3");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD^{tree}"]), before);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// `--include` commits the named path's worktree content together with what is already staged.
#[test]
fn include_commits_the_named_path_together_with_the_staged_changes() {
    let fixture: Fixture = fixture("commit-include");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("b.txt"), b"b\n").expect("b");
    git(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--include", "--message=both", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:b.txt"]), "b\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// `--pathspec-from-file` names the committed paths from a file, and `-` reads them from standard input.
#[test]
fn a_pathspec_file_selects_the_committed_paths() {
    let fixture: Fixture = fixture("commit-pathspec-file");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("b.txt"), b"b\n").expect("b");
    std::fs::write(repo.join("list.txt"), b"a.txt\n").expect("list");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=listed", "--pathspec-from-file=list.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert!(!git_text(&fixture, repo.as_path(), &["ls-tree", "--name-only", "HEAD"]).contains("b.txt"));
    std::fs::write(repo.join("a.txt"), b"one\ntwo\nthree\n").expect("edit a again");
    let from_input: Observed = observe(
        &mut wrapped_commit(&fixture, repo.as_path(), &["commit", "--message=piped", "--pathspec-from-file=-"]),
        b"a.txt\n",
    );
    assert_eq!(from_input.code, Some(0), "{}", stderr_of(&from_input));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\nthree\n");
    assert_eq!(git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(), "piped");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An explicit-path commit cannot record a deletion staged for a path that stays on disk: the finding says so and
/// offers the index-backed form, and that form records the deletion and keeps the file (issue 472).
#[test]
fn an_explicit_path_commit_of_a_staged_deletion_that_stays_on_disk_offers_the_index_form() {
    let fixture: Fixture = fixture("commit-staged-deletion");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    git(&fixture, repo.as_path(), &["rm", "--quiet", "--cached", "--", "a.txt"]);
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=untrack", "a.txt"]);
    assert_eq!(refused.code, Some(1), "{}", stderr_of(&refused));
    let text: String = stderr_of(&refused);
    assert!(text.contains("commit-only/staged-deletion-worktree-present"), "{text}");
    assert!(text.contains("[\\\"a.txt\\\"]"), "{text}");
    assert!(text.contains("--no-only"), "{text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(porcelain(&fixture, repo.as_path()), "D  a.txt\n?? a.txt\n");
    assert_nothing_left(&fixture, repo.as_path());
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--no-only", "--message=untrack"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["ls-tree", "--name-only", "HEAD"]).trim(), "");
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("the file stays"), b"one\ntwo\n");
    remove_settled(&fixture);
}

/// A staged deletion below an ignored directory gets the same finding, not Git's hint to force-add the ignored path.
#[test]
fn a_staged_deletion_below_an_ignored_directory_is_not_answered_with_a_force_add_hint() {
    let fixture: Fixture = fixture("commit-staged-deletion-ignored");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::create_dir(repo.join("tool")).expect("tool directory");
    std::fs::write(repo.join("tool").join("name"), b"local\n").expect("tool file");
    git(&fixture, repo.as_path(), &["add", "--", "tool/name"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join(".gitignore"), b"/tool/\n").expect("ignore rule");
    git(&fixture, repo.as_path(), &["add", "--", ".gitignore"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ignore", "--", ".gitignore"]);
    keep_maintenance_in_the_foreground(&fixture, repo.as_path());
    git(&fixture, repo.as_path(), &["rm", "--quiet", "--cached", "--", "tool/name"]);
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=untrack", "tool/name"]);
    assert_eq!(refused.code, Some(1), "{}", stderr_of(&refused));
    let text: String = stderr_of(&refused);
    assert!(text.contains("commit-only/staged-deletion-worktree-present"), "{text}");
    assert!(!text.contains("Use -f"), "{text}");
    assert!(!text.contains("ignored by one of your .gitignore files"), "{text}");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// Files an ignore rule matches but the real index stages are committed by a directory pathspec and by a file
/// pathspec, with the worktree content, as native Git commits them (issue 1807).
#[test]
fn explicit_paths_commit_staged_additions_that_an_ignore_rule_matches() {
    let fixture: Fixture = fixture("commit-staged-ignored");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join(".gitignore"), b"*.js\n").expect("ignore rule");
    git(&fixture, repo.as_path(), &["add", "--", ".gitignore"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ignore", "--", ".gitignore"]);
    std::fs::create_dir(repo.join("lib")).expect("lib");
    std::fs::write(repo.join("lib/_a.js"), b"a\n").expect("_a");
    std::fs::write(repo.join("lib/_b.js"), b"b\n").expect("_b");
    git(&fixture, repo.as_path(), &["add", "--force", "--", "lib/_a.js", "lib/_b.js"]);
    let by_directory: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=lib", "--", "lib"]);
    assert_eq!(by_directory.code, Some(0), "{}", stderr_of(&by_directory));
    assert_eq!(git_text(&fixture, repo.as_path(), &["ls-tree", "--name-only", "-r", "HEAD", "lib"]), "lib/_a.js\nlib/_b.js\n");
    std::fs::write(repo.join("lib/_a.js"), b"a2\n").expect("edit _a");
    git(&fixture, repo.as_path(), &["add", "--force", "--", "lib/_a.js"]);
    std::fs::write(repo.join("lib/_a.js"), b"a3\n").expect("edit _a again");
    let by_file: Observed =
        run_commit(&fixture, repo.as_path(), &["commit", "--message=file", "--", "lib/_a.js"]);
    assert_eq!(by_file.code, Some(0), "{}", stderr_of(&by_file));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:lib/_a.js"]), "a3\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An ignored path nothing stages is refused with the way forward, not with Git's force-add hint alone.
#[test]
fn an_explicit_path_commit_of_an_unstaged_ignored_path_names_the_way_forward() {
    let fixture: Fixture = fixture("commit-unstaged-ignored");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join(".gitignore"), b"*.js\n").expect("ignore rule");
    git(&fixture, repo.as_path(), &["add", "--", ".gitignore"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ignore", "--", ".gitignore"]);
    std::fs::write(repo.join("c.js"), b"c\n").expect("c");
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=c", "--", "c.js"]);
    assert_ne!(refused.code, Some(0), "{}", stderr_of(&refused));
    assert!(stderr_of(&refused).contains("git add --force -- <path>"), "{}", stderr_of(&refused));
    assert_nothing_left(&fixture, repo.as_path());
    let head_before: String = git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]);
    git(&fixture, repo.as_path(), &["add", "--force", "--", "c.js"]);
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=c", "--", "c.js"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_ne!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]), head_before);
    remove_settled(&fixture);
}

/// A repository nothing was staged in has no index file; an explicit-path commit there reads it as empty instead of
///  failing on the missing file.
#[test]
fn an_explicit_path_commit_works_before_any_index_exists() {
    let fixture: Fixture = fixture("commit-no-index");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    // Git writes an index file whenever it commits; a repository nothing was staged in has none, so remove it.
    std::fs::remove_file(repo.join(".git/index")).expect("the index Git wrote for the initial commit");
    assert!(!repo.join(".git/index").exists(), "the premise: no index file yet");
    std::fs::write(repo.join("first.txt"), b"one\n").expect("first file");
    let observed: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=first", "--", "first.txt"],
    );
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert_eq!(
        git_text(&fixture, &repo, &["show", "--name-only", "--format=", "HEAD"]),
        "first.txt\n"
    );
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    remove_settled(&fixture);
}

/// An explicit-path commit leaves every other staged entry (a rename, a new file) exactly as it was and says nothing
///  about staged entries moving.
#[test]
fn an_explicit_path_commit_keeps_other_staged_entries_without_a_warning() {
    let fixture: Fixture = fixture("commit-keeps-staged");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("b.txt"), b"base\n").expect("b");
    git(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=b", "--", "b.txt"]);
    git(&fixture, repo.as_path(), &["mv", "b.txt", "moved.txt"]);
    std::fs::write(repo.join("c.txt"), b"new\n").expect("c");
    git(&fixture, repo.as_path(), &["add", "--", "c.txt"]);
    let staged_before: String = git_text(&fixture, &repo, &["diff", "--cached", "--name-status"]);
    let observed: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=edit a", "--", "a.txt"],
    );
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert!(
        !stderr_of(&observed).contains("outside the paths it committed"),
        "{}",
        stderr_of(&observed)
    );
    assert_eq!(
        git_text(&fixture, &repo, &["diff", "--cached", "--name-status"]),
        staged_before
    );
    remove_settled(&fixture);
}
