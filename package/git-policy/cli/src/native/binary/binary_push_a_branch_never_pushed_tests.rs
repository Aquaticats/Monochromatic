//! What:
//!  Where the automatic push of a branch that has never been pushed goes,
//! read from the bare remote and from the branch's own configuration after one commit through the wrapper:
//! a branch with no tracking setting at all,
//! one with only `branch.<name>.remote`,
//! one with only `branch.<name>.merge`,
//! one whose upstream is another remote,
//! one in a linked worktree,
//! and one in a repository with no `origin`.
//! Why:
//!  `SPEC.md` line 9204 says a branch without an upstream is pushed to `origin` with `--set-upstream` when that
//! remote exists, and line 9205 says it is skipped silently when it does not.
//! The remote is a real bare repository,
//! the service a real process,
//! the command the built wrapper,
//! so every row is what a person meets.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await wrapped(['commit', '-m', 'work']); await untilSettled();
//! expect(remoteTip('fresh')).toBe(localTip('fresh')); expect(config('branch.fresh.merge')).toBe('refs/heads/fresh');
//! ```

use super::commit::{remove_settled, wrapped_commit};
use super::support::{
    Fixture, FixtureService, Observed, SERVICE_WORK_PATIENCE, fixture, git, native_git, observe, repository,
    start_fixture_service, stderr_of,
};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// A repository on `main` that tracks a bare remote's `main`,
/// a second bare remote named `alt`,
/// and a service.
struct World {
    /// The disposable folder this control owns.
    fixture: Fixture,
    /// The worktree the commits land in.
    repo: PathBuf,
    /// The bare repository `origin` names.
    remote: PathBuf,
    /// The bare repository `alt` names.
    alt: PathBuf,
    /// The service that pushes.
    service: FixtureService,
}

/// The world whose `origin` holds `main`.
fn world(name: &str) -> World {
    return world_with_origin(name, true);
}

/// What:
///  The same world,
/// with or without an `origin` remote.
/// Why:
///  The rule that skips a branch silently is decided by whether `origin` exists,
/// so the two rows differ in that one setting.
fn world_with_origin(name: &str, with_origin: bool) -> World {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let remote: PathBuf = bare(&fixture, "remote.git");
    let alt: PathBuf = bare(&fixture, "alt.git");
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    git(&fixture, &repo, &["remote", "add", "alt", alt.to_str().expect("UTF-8")]);
    if with_origin {
        git(&fixture, &repo, &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
        git(&fixture, &repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    }
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return World { fixture, repo, remote, alt, service };
}

/// A bare repository of the fixture's own.
fn bare(fixture: &Fixture, name: &str) -> PathBuf {
    let path: PathBuf = fixture.root.join(name);
    git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", path.to_str().expect("UTF-8")]);
    return path;
}

/// Run a wrapped command in `directory` with the fixture's service reachable.
fn wrapped_in(world: &World, directory: &Path, arguments: &[&str]) -> Observed {
    let mut command = wrapped_commit(&world.fixture, directory, arguments);
    command.env("XDG_RUNTIME_DIR", &world.service.runtime);
    return observe(&mut command, b"");
}

/// Add `name` to the index and commit it through the wrapper.
fn commit_a_file(world: &World, directory: &Path, name: &str) -> Observed {
    std::fs::write(directory.join(name), format!("{name}\n")).expect("a file to commit");
    // The commit-only policy refuses a commit that names no path, so the file is named on the command line.
    return wrapped_in(world, directory, &["commit", "--quiet", "--message", name, "--", name]);
}

/// What a ref holds in a repository,
/// trimmed,
/// and nothing when it does not exist.
fn tip(world: &World, repository: &Path, name: &str) -> Option<String> {
    let asked = native_git(&world.fixture, repository, &["rev-parse", "--verify", "--quiet", name]);
    if asked.status.code() != Some(0) {
        return None;
    }
    return Some(String::from_utf8_lossy(asked.stdout.as_slice()).trim().to_owned());
}

/// The commit a branch names where it is checked out.
fn local_tip(world: &World, directory: &Path) -> String {
    return tip(world, directory, "HEAD").expect("the branch names a commit");
}

/// One configuration value of the repository,
/// and nothing when it is not set.
fn configured(world: &World, directory: &Path, key: &str) -> Option<String> {
    let asked = native_git(&world.fixture, directory, &["config", "--get", key]);
    if asked.status.code() != Some(0) {
        return None;
    }
    return Some(String::from_utf8_lossy(asked.stdout.as_slice()).trim().to_owned());
}

/// How many push records in these states the service holds for one Git directory.
fn push_records_in(git_dir: &Path, states: &[&str]) -> usize {
    let work: PathBuf = git_policy_cli::service_work::work_directory(git_dir);
    return states
        .iter()
        .map(|state| {
            return std::fs::read_dir(work.join(state)).map_or(0, |listing| {
                return listing.flatten().filter(|entry| { return entry.file_name().to_string_lossy().starts_with("push-"); }).count();
            });
        })
        .sum();
}

/// How many push records of any state the service still has to act on for one Git directory.
fn open_push_records(git_dir: &Path) -> usize {
    return push_records_in(git_dir, &["pending", "claimed", "recovering"]);
}

/// What:
///  Wait until the service has settled the after-commit work of one Git directory.
/// Why:
///  `settle_service_work` climbs to the nearest ancestor holding a `.git` folder,
/// and a linked worktree's `.git` is a file,
/// so its climb stops at the main checkout's,
/// which is another worktree's work directory.
fn settle(git_dir: &Path) {
    let work: PathBuf = git_policy_cli::service_work::work_directory(git_dir);
    let began: Instant = Instant::now();
    let open = |state: &str| -> usize { return std::fs::read_dir(work.join(state)).map_or(0, Iterator::count); };
    while open("pending") + open("claimed") + open("recovering") > 0 {
        assert!(began.elapsed() < SERVICE_WORK_PATIENCE, "the service never settled the after-commit work in {}", work.display());
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Assert the command landed and the service settled its work.
fn landed(observed: &Observed, git_dir: &Path) {
    let text: String = stderr_of(observed);
    assert_eq!(observed.code, Some(0), "{text}");
    settle(git_dir);
}

/// What a skipped push says on the terminal,
/// and what it must not say.
fn assert_nothing_was_said(observed: &Observed) {
    let text: String = stderr_of(observed);
    assert!(!text.contains("auto-push"), "a skipped push said something: {text}");
    assert!(!text.contains("not pushed"), "a skipped push said something: {text}");
    assert!(!text.contains("upstream"), "a skipped push said something: {text}");
}

/// A branch nothing has set up is published to `origin` under its own name and tracks it.
#[test]
fn a_branch_with_no_upstream_is_published_to_origin_and_tracks_it() {
    let world: World = world("never-pushed-no-settings");
    git(&world.fixture, &world.repo, &["switch", "--quiet", "--create", "fresh"]);
    let observed: Observed = commit_a_file(&world, world.repo.as_path(), "fresh.txt");
    landed(&observed, world.repo.join(".git").as_path());
    let local: String = local_tip(&world, world.repo.as_path());
    assert_eq!(tip(&world, world.remote.as_path(), "refs/heads/fresh").as_deref(), Some(local.as_str()), "origin holds the branch: {}", stderr_of(&observed));
    assert_eq!(tip(&world, world.remote.as_path(), "refs/heads/main").as_deref(), tip(&world, world.repo.as_path(), "refs/heads/main").as_deref(), "the checked-out branch before it did not move");
    assert_eq!(configured(&world, world.repo.as_path(), "branch.fresh.remote").as_deref(), Some("origin"));
    assert_eq!(configured(&world, world.repo.as_path(), "branch.fresh.merge").as_deref(), Some("refs/heads/fresh"));
    assert_eq!(
        push_records_in(world.repo.join(".git").as_path(), &["done"]),
        1,
        "the service made the push after the command returned, not the command itself"
    );
    remove_settled(&world.fixture);
}

/// A branch whose remote is set and whose merge ref is not is published to `origin` and tracks it.
#[test]
fn a_branch_with_only_its_remote_set_is_published_to_origin_and_tracks_it() {
    let world: World = world("never-pushed-only-remote");
    git(&world.fixture, &world.repo, &["switch", "--quiet", "--create", "fresh"]);
    git(&world.fixture, &world.repo, &["config", "branch.fresh.remote", "origin"]);
    let observed: Observed = commit_a_file(&world, world.repo.as_path(), "fresh.txt");
    landed(&observed, world.repo.join(".git").as_path());
    let local: String = local_tip(&world, world.repo.as_path());
    assert_eq!(tip(&world, world.remote.as_path(), "refs/heads/fresh").as_deref(), Some(local.as_str()), "origin holds the branch: {}", stderr_of(&observed));
    assert_eq!(configured(&world, world.repo.as_path(), "branch.fresh.merge").as_deref(), Some("refs/heads/fresh"));
    remove_settled(&world.fixture);
}

/// A branch whose merge ref is set and whose remote is not is published to `origin` and tracks it.
#[test]
fn a_branch_with_only_its_merge_ref_set_is_published_to_origin_and_tracks_it() {
    let world: World = world("never-pushed-only-merge");
    git(&world.fixture, &world.repo, &["switch", "--quiet", "--create", "fresh"]);
    git(&world.fixture, &world.repo, &["config", "branch.fresh.merge", "refs/heads/fresh"]);
    let observed: Observed = commit_a_file(&world, world.repo.as_path(), "fresh.txt");
    landed(&observed, world.repo.join(".git").as_path());
    let local: String = local_tip(&world, world.repo.as_path());
    assert_eq!(tip(&world, world.remote.as_path(), "refs/heads/fresh").as_deref(), Some(local.as_str()), "origin holds the branch: {}", stderr_of(&observed));
    assert_eq!(configured(&world, world.repo.as_path(), "branch.fresh.remote").as_deref(), Some("origin"));
    remove_settled(&world.fixture);
}

/// A branch that already tracks another remote is published there,
/// and not to `origin`.
#[test]
fn a_branch_that_tracks_another_remote_is_published_there() {
    let world: World = world("never-pushed-other-remote");
    git(&world.fixture, &world.repo, &["push", "--quiet", "alt", "main"]);
    git(&world.fixture, &world.repo, &["switch", "--quiet", "--create", "fresh"]);
    git(&world.fixture, &world.repo, &["config", "branch.fresh.remote", "alt"]);
    git(&world.fixture, &world.repo, &["config", "branch.fresh.merge", "refs/heads/main"]);
    let observed: Observed = commit_a_file(&world, world.repo.as_path(), "fresh.txt");
    landed(&observed, world.repo.join(".git").as_path());
    let local: String = local_tip(&world, world.repo.as_path());
    assert_eq!(tip(&world, world.alt.as_path(), "refs/heads/main").as_deref(), Some(local.as_str()), "alt holds the branch: {}", stderr_of(&observed));
    assert_eq!(tip(&world, world.remote.as_path(), "refs/heads/fresh"), None, "origin was not given the branch");
    assert_eq!(configured(&world, world.repo.as_path(), "branch.fresh.remote").as_deref(), Some("alt"), "the tracking remote did not move");
    remove_settled(&world.fixture);
}

/// A branch of a linked worktree that nothing has set up is published to `origin` and tracks it.
#[test]
fn a_branch_of_a_linked_worktree_with_no_upstream_is_published_to_origin_and_tracks_it() {
    let world: World = world("never-pushed-linked-worktree");
    let linked: PathBuf = world.fixture.root.join("linked");
    git(&world.fixture, &world.repo, &["worktree", "add", "--quiet", "--no-track", "-b", "fresh", linked.to_str().expect("UTF-8"), "main"]);
    let linked_git_dir: PathBuf = world.repo.join(".git/worktrees/linked");
    world.service.serve(&world.fixture, linked_git_dir.as_path());
    assert_eq!(configured(&world, linked.as_path(), "branch.fresh.remote"), None, "the worktree branch starts with no upstream");
    let observed: Observed = commit_a_file(&world, linked.as_path(), "fresh.txt");
    landed(&observed, linked_git_dir.as_path());
    let local: String = local_tip(&world, linked.as_path());
    assert_eq!(tip(&world, world.remote.as_path(), "refs/heads/fresh").as_deref(), Some(local.as_str()), "origin holds the branch: {}", stderr_of(&observed));
    assert_eq!(configured(&world, linked.as_path(), "branch.fresh.remote").as_deref(), Some("origin"));
    assert_eq!(configured(&world, linked.as_path(), "branch.fresh.merge").as_deref(), Some("refs/heads/fresh"));
    remove_settled(&world.fixture);
}

/// A branch with no upstream in a repository with no `origin` is skipped,
/// nothing is queued or pushed,
/// and the command says nothing about it.
#[test]
fn a_branch_with_no_upstream_and_no_origin_is_skipped_without_a_word() {
    let world: World = world_with_origin("never-pushed-no-origin", false);
    git(&world.fixture, &world.repo, &["switch", "--quiet", "--create", "fresh"]);
    let observed: Observed = commit_a_file(&world, world.repo.as_path(), "fresh.txt");
    landed(&observed, world.repo.join(".git").as_path());
    assert_nothing_was_said(&observed);
    assert_eq!(open_push_records(world.repo.join(".git").as_path()), 0, "no push was queued: {}", stderr_of(&observed));
    assert_eq!(tip(&world, world.remote.as_path(), "refs/heads/fresh"), None, "the remote nothing names was not given the branch");
    assert_eq!(configured(&world, world.repo.as_path(), "branch.fresh.remote"), None, "no upstream was set");
    remove_settled(&world.fixture);
}
