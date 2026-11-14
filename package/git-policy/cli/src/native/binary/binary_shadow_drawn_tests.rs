//! What:
//!  What a commit does when the finished shadow it was handed goes between its hand-out and its last write,
//!  through the built executable and the real service:
//!  a tree that goes after the commit's own question of it and before its first write,
//!  and one that goes after that write.
//! Why:
//!  The tree is the wrapper's own cache,
//!  so nothing another process of the repository does to it may end a commit with a nonzero status
//!  (`SPEC.md`,
//!  "Merges and commits never fail").
//!  A tree that goes before the first write is a tree the commit has written nothing of its own into,
//!  so it takes the pool's member or makes its own tree and lands;
//!  a tree that goes after that write is one the commit is already done with,
//!  because the request carries the change and the landing reads the commit's own scratch tree.
//!  The two instants are the wrapper's own phase markers (`CLI_GIT_TEST_ONLY_PHASE_SIGNAL`),
//!  so the loss is placed exactly there and nothing here is a stand-in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! pauseAt('shadow-handed'); await rm(handed, { recursive: true }); release(); expect(await commit()).toLand();
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, FixtureService, Observed, fixture, git, observe, repository, stderr_of, start_fixture_service};
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use git_policy_cli::wrapper_names_old_and_new::SHADOW_DRAWN_FOLDER;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// The longest a control waits for the paused wrapper to arrive,
/// far beyond a healthy run.
const PATIENCE: Duration = Duration::from_secs(60);

/// A repository with the service that serves it,
/// an edit the next commit names,
/// and where the paused wrapper leaves its markers.
struct Case {
    fixture: Fixture,
    repo: PathBuf,
    service: FixtureService,
    markers: PathBuf,
}

/// A repository with one commit on `main`,
/// and a file the next commit changes.
fn case(name: &str) -> Case {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(&fixture, repo.as_path(), &["config", "commit.verbose", "false"]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("markers");
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Case { fixture, repo, service, markers };
}

/// The drawn trees the worktree's drawn folder holds now.
fn drawn_trees(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder) else { return Vec::new() };
    return entries
        .flatten()
        .map(|entry| { return entry.path(); })
        .filter(|path| { return path.file_name().is_some_and(|name| { return name.to_string_lossy().starts_with("drawn-"); }); })
        .collect();
}

/// The command that commits the next edit,
/// under the service's runtime and with the marker variable armed for `phase`.
fn a_commit_paused_at(case: &Case, phase: &str, message: &str) -> Command {
    let mut command: Command = wrapped_commit(&case.fixture, case.repo.as_path(), &["commit", "--quiet", &format!("--message={message}"), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &case.service.runtime);
    command.env(PHASE_SIGNAL_VARIABLE, format!("{phase}:pause:{}", case.markers.display()));
    return command;
}

/// What:
///  A repository whose service drew the next commit's tree,
///  with the edit that commit names,
///  and where the drawn tree is.
/// Why:
///   The draw happens beside the landing that answers a commit,
///  so a control needs one commit made before the one it means to hold.
fn a_repository_with_a_drawn_tree(name: &str) -> (Case, PathBuf) {
    let case: Case = case(name);
    std::fs::write(case.repo.join("a.txt"), "base\none\n").expect("the first edit");
    git(&case.fixture, case.repo.as_path(), &["add", "--", "a.txt"]);
    let mut first: Command = wrapped_commit(&case.fixture, case.repo.as_path(), &["commit", "--quiet", "--message=one", "--", "a.txt"]);
    first.env("XDG_RUNTIME_DIR", &case.service.runtime);
    let landed: Observed = observe(&mut first, b"");
    assert_eq!(landed.code, Some(0), "the commit that makes the service draw: {}", stderr_of(&landed));
    let folder: PathBuf = case.repo.join(".git").join(SHADOW_DRAWN_FOLDER).join("main");
    let began: Instant = Instant::now();
    let mut drawn: Vec<PathBuf> = drawn_trees(folder.as_path());
    while drawn.is_empty() {
        assert!(began.elapsed() < PATIENCE, "the service drew the next commit's tree ahead of it");
        std::thread::sleep(Duration::from_millis(10));
        drawn = drawn_trees(folder.as_path());
    }
    assert_eq!(drawn.len(), 1, "a stock of one is what the draw keeps: {drawn:?}");
    let handed: PathBuf = drawn.remove(0);
    std::fs::write(case.repo.join("a.txt"), "base\none\ntwo\n").expect("the second edit");
    git(&case.fixture, case.repo.as_path(), &["add", "--", "a.txt"]);
    return (case, handed);
}

/// What:
///  A commit that is handed a drawn tree and loses it after its own question of it,
///  and before its first write into it,
///  lands, in a tree of its own or one from the pool.
/// Why:
///   The tree the service answered with is the wrapper's own cache,
///  and a tree that goes at that instant is not something the person did or can fix.
///  Nothing has been sent to the service yet and what was written went into the tree the command abandons,
///  so the commit takes the pool's member or makes its own tree and lands as though it had never been handed one.
#[test]
fn a_commit_that_loses_its_handed_tree_after_its_question_still_lands() {
    let (case, handed): (Case, PathBuf) = a_repository_with_a_drawn_tree("handed-tree-taken-away");
    let mut command: Command = a_commit_paused_at(&case, "shadow-handed", "two");
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join("shadow-handed-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(
            began.elapsed() < PATIENCE,
            "the wrapper reached the instant between its question of the handed tree and its first write into it"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    // The tree goes, as another program of the repository's removal would take it.
    std::fs::remove_dir_all(handed.as_path()).expect("the handed tree goes");
    std::fs::write(markers.join("shadow-handed-1.release"), b"").expect("release the paused command");
    let seen: Observed = waiting.join().expect("the paused command");
    let words: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "a commit whose handed tree went must still land: {words}");
    assert!(!words.contains("packed-refs"), "nothing about the wrapper's own cache reaches the person: {words}");
    let subjects: String = git_text(&case.fixture, case.repo.as_path(), &["log", "--format=%s"]);
    assert_eq!(
        subjects.lines().collect::<Vec<&str>>(),
        vec!["two", "one", "base", "initial"],
        "the commit that lost its tree is on the branch, above the one the draw followed"
    );
    remove_settled(&case.fixture);
}

/// What:
///  A commit whose handed tree goes after the last write into it still lands.
/// Why:
///   After that write the commit is done with the tree:
///  its request carries the change and the landing reads the commit's own scratch tree,
///  so the tree going is a removal the commit never notices,
///  and the service's own removal of that tree then finds nothing to remove.
#[test]
fn a_commit_whose_handed_tree_goes_after_its_last_write_still_lands() {
    let (case, handed): (Case, PathBuf) = a_repository_with_a_drawn_tree("handed-tree-taken-away-after");
    let mut command: Command = a_commit_paused_at(&case, "service-prepared", "two");
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join("service-prepared-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(
            began.elapsed() < PATIENCE,
            "the wrapper reached the instant after its last write into the handed tree and before it sent the commit"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    std::fs::remove_dir_all(handed.as_path()).expect("the handed tree goes");
    std::fs::write(markers.join("service-prepared-1.release"), b"").expect("release the paused command");
    let seen: Observed = waiting.join().expect("the paused command");
    let words: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "a commit whose handed tree went after its last write still lands: {words}");
    let subjects: String = git_text(&case.fixture, case.repo.as_path(), &["log", "--format=%s"]);
    assert_eq!(
        subjects.lines().collect::<Vec<&str>>(),
        vec!["two", "one", "base", "initial"],
        "the commit that lost its tree is on the branch, above the one the draw followed"
    );
    remove_settled(&case.fixture);
}
