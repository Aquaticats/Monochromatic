//! What:
//!  The snapshot rung of `git worktree add` decides on the filesystem and not on the reader when the add's own environment
//!  names an object store,
//! alternate object directories or a replacement base.
//! Why:
//!  The object reader declines those variables,
//! so that every question a landing asks Git reads the store Git reads.
//!  A proof of *this worktree's own files* is not such a question:
//! it reads the repository's own store,
//! as `main`'s reader
//!  did,
//! and a harmless value of either variable must not turn a snapshot into a copy.
//!  The control counts the routes the add states:
//! the snapshot line where the filesystem can make one,
//! its own structural
//!  refusal where it cannot,
//! and never the reader's refusal of the source.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // held('GIT_OBJECT_DIRECTORY')({ status: 0 }, stderrOf(add)).routes === ['snapshot' | 'filesystem']
//! ```

use super::support::{
    Fixture,
    Observed,
    fixture,
    git,
    observe,
    repository,
    wrapped,
};
use git_policy_cli::worktree_slow_route::LEAD;
use git_policy_cli::worktree_trash::MAIN_SNAPSHOT_NOTE;
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;

/// The words a source the reader refused leaves in the add's explanation.
/// Why:
///  They are the regression this control holds:
/// a proof of this worktree's own files must answer under these variables.
const THE_READERS_REFUSAL: &str = "could not be read to prove it unchanged";

/// The route lines one add stated:
/// the snapshot it made,
/// and the slow route it explained.
/// Why:
///  An add on a filesystem that cannot make a snapshot states nothing where the filesystem's own limit is expected,
///  and states its own reason where it is not;
///  either way the lines are what the route came to,
///  and they must not change because a variable is held.
fn route_lines_of(seen: &Observed) -> Vec<String> {
    return String::from_utf8_lossy(seen.stderr.as_slice())
        .lines()
        .filter(|line| {
            return line.trim_end() == MAIN_SNAPSHOT_NOTE.trim_end()
                || line.starts_with(LEAD.trim_end());
        })
        .map(str::to_owned)
        .collect();
}

/// What:
///  One `git worktree add` through the wrapper,
///  with one setting held or none.
/// Why:
///  The control and its arm differ in the setting alone,
///  so the command line and the environment are made in one place.
fn add_through_the_wrapper(
    fixture: &Fixture,
    repo: &Path,
    branch: &str,
    held: Option<(&str, &str)>,
) -> Observed {
    let mut command: Command = wrapped(fixture);
    command.current_dir(repo).args([
        "worktree",
        "add",
        format!("../{branch}").as_str(),
        "-b",
        format!("topic-{branch}").as_str(),
    ]);
    if let Some((variable, value)) = held {
        command.env(variable, value);
    }
    return observe(&mut command, b"");
}

/// What:
///  One add through the wrapper with `setting` held,
///  and one with nothing held as its control,
///  both stated by the same route and neither because of the reader.
/// Why:
///  The route is the filesystem's where a snapshot cannot be made,
/// and the snapshot's where one can;
///  what must never appear is the reader's refusal of the source,
///  which is what turned a snapshot into a copy under a harmless value.
fn assert_the_snapshot_rung_decides(
    name: &'static str,
    setting: fn(&Fixture, &Path) -> (&'static str, String),
) {
    let fixture: Fixture = fixture(format!("worktree-add-source-{name}").as_str());
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=base"],
    );
    let (variable, value): (&'static str, String) = setting(&fixture, repo.as_path());
    // The control first: the same add with nothing held states the route this filesystem gives an add.
    let nothing: Observed = add_through_the_wrapper(&fixture, repo.as_path(), "with-nothing", None);
    let nothing_said: String = String::from_utf8_lossy(nothing.stderr.as_slice()).into_owned();
    assert_eq!(
        nothing.code,
        Some(0),
        "{name}: the add with nothing held lands: {nothing_said}"
    );
    let nothing_routes: Vec<String> = route_lines_of(&nothing);
    assert!(
        nothing_routes.len() <= 1,
        "{name}: the add with nothing held states one route: {nothing_said}"
    );
    // The arm: the same add with the variable held states the same route, and never the reader's refusal of the source.
    let held: Observed = add_through_the_wrapper(
        &fixture,
        repo.as_path(),
        "with-the-setting",
        Some((variable, value.as_str())),
    );
    let said: String = String::from_utf8_lossy(held.stderr.as_slice()).into_owned();
    let routes: Vec<String> = route_lines_of(&held);
    assert_eq!(
        held.code,
        Some(0),
        "{name} with {variable}: the add lands: {said}"
    );
    assert!(
        routes.len() <= 1,
        "{name} with {variable}: the add states one route: {said}"
    );
    assert!(
        !said.contains(THE_READERS_REFUSAL),
        "{name} with {variable}: the source is proved, so the route is not the reader's: {said}"
    );
    assert_eq!(
        routes, nothing_routes,
        "{name} with {variable}: the route the add states is the one it states with nothing held, which is the snapshot where the filesystem can make one: {said}"
    );
    // What the counts are on this filesystem: the guest run beside this control counts the same five adds on btrfs.
    let snapshots: usize = routes
        .iter()
        .filter(|line| {
            return line.trim_end() == MAIN_SNAPSHOT_NOTE.trim_end();
        })
        .count();
    assert!(
        snapshots <= 1,
        "{name}: at most one snapshot is stated: {said}"
    );
    super::support::remove(&fixture);
}

/// `GIT_OBJECT_DIRECTORY` naming the repository's own store.
#[test]
fn a_worktree_add_proves_its_source_under_the_object_store_a_command_names() {
    return assert_the_snapshot_rung_decides("object-store", |_fixture, repo: &Path| {
        return (
            "GIT_OBJECT_DIRECTORY",
            repo.join(".git/objects").display().to_string(),
        );
    });
}

/// `GIT_ALTERNATE_OBJECT_DIRECTORIES` naming an empty directory.
#[test]
fn a_worktree_add_proves_its_source_under_the_alternate_directories_a_command_names() {
    return assert_the_snapshot_rung_decides("alternate-directories", |fixture, _repo: &Path| {
        let empty: PathBuf = fixture.root.join("empty-cache");
        std::fs::create_dir_all(empty.as_path()).expect("an empty directory");
        return (
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
            empty.display().to_string(),
        );
    });
}

/// `GIT_REPLACE_REF_BASE` naming a base with nothing under it.
#[test]
fn a_worktree_add_proves_its_source_under_the_replacement_base_a_command_names() {
    return assert_the_snapshot_rung_decides("replacement-base", |_fixture, _repo: &Path| {
        return ("GIT_REPLACE_REF_BASE", String::from("refs/probe/"));
    });
}
