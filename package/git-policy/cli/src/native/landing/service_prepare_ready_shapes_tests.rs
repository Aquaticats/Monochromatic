//! What:
//!  The commit's scratch directory as the preparation hands it on:
//!  where it is made,
//!  and that the command that made it never removes it.
//! Why:
//!  A recursive removal after the commit landed waited on the person
//!  (`doc/planning/cli-git-commit-latency-under-many-agents.md`),
//!  so what the command must not do is remove it,
//!  and what must be true is that something else does:
//!  the service that held the connection,
//!  or the sweep of the repository scratch directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const scratch = makeScratch(common, git, name); drop(scratch); expect(exists(scratch.path)).toBe(true);
//! ```

/// The directory under control,
/// and the place it is made.
use super::Scratch;
/// The marker a later command reads,
/// and the rule it decides with.
use crate::service_index_marker::{holder_is_running, marker_of};
use crate::service_scratch_abandonment::{OWNER_MARKER, UNMARKED_SCRATCH_TREE_AGE, remove_abandoned_trees};
use crate::scratch_location::REPOSITORY_SCRATCH_NAME;
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Set an entry's modification time to `back` before now.
fn age(path: &Path, back: Duration) {
    std::fs::File::open(path).expect("open").set_modified(SystemTime::now() - back).expect("set modification time");
}

/// The names of this process's threads.
fn thread_names() -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    if let Ok(tasks) = std::fs::read_dir("/proc/self/task") {
        for task in tasks.flatten() {
            if let Ok(name) = std::fs::read_to_string(task.path().join("comm")) {
                names.push(name.trim_end().to_string());
            }
        }
    }
    return names;
}

/// A tree the command handed to the service is the service's to remove,
/// and a tree nothing took is the command's own to remove,
/// so a declined commit leaves nothing behind.
#[test]
fn the_commit_removes_only_the_tree_no_service_took() {
    let root: PathBuf = fixture("ready-shapes-scratch");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let name: &str = "monochromatic-git-policy-cli-service-0a1b2c3d";
    let mut handed: Scratch = Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree");
    let tree: PathBuf = handed.0.clone();
    handed.held_by_the_service(true);
    drop(handed);
    assert!(tree.exists(), "the command that handed it over must not remove it: {}", tree.display());
    drop(Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree again"));
    assert!(!tree.exists(), "a tree no service took is removed by the command: {}", tree.display());
    remove(root.as_path());
}

/// The tree is made inside the repository scratch directory,
/// where the sweep covers it.
#[test]
fn the_scratch_tree_is_made_where_the_sweep_covers_it() {
    let root: PathBuf = fixture("ready-shapes-scratch-made");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let scratch: Scratch = Scratch::make(common.as_path(), common.as_path(), "monochromatic-git-policy-cli-service-0a1b2c3d").expect("the scratch tree");
    assert_eq!(scratch.0, common.join(REPOSITORY_SCRATCH_NAME).join("monochromatic-git-policy-cli-service-0a1b2c3d"));
    drop(scratch);
    remove(root.as_path());
}

/// A second attempt of one command gets an empty tree,
/// so nothing an earlier attempt wrote is read as this attempt's.
#[test]
fn a_second_attempt_starts_from_an_empty_tree() {
    let root: PathBuf = fixture("ready-shapes-scratch-again");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let name: &str = "monochromatic-git-policy-cli-service-0a1b2c3d";
    let first: Scratch = Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree");
    std::fs::write(first.0.join("commit.index"), b"stale").expect("a file in the tree");
    std::fs::create_dir_all(first.0.join("shadow")).expect("a shadow of the earlier attempt");
    drop(first);
    let again: Scratch = Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree again");
    assert!(!again.0.join("commit.index").exists(), "the earlier attempt's index is gone");
    assert!(!again.0.join("shadow").exists(), "the earlier attempt's shadow is gone");
    remove(root.as_path());
}

/// A repository whose scratch directory cannot be used still gets a tree,
/// in the Git directory,
/// so a commit never fails over where its scratch files go.
#[test]
fn an_unusable_repository_scratch_directory_falls_back_to_the_git_directory() {
    let root: PathBuf = fixture("ready-shapes-scratch-fallback");
    let git_dir: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&git_dir).expect("git directory");
    // A link where the repository scratch directory goes is refused by the privacy check.
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");
    std::os::unix::fs::symlink(&elsewhere, git_dir.join(REPOSITORY_SCRATCH_NAME)).expect("a link in place of the directory");
    let name: &str = "monochromatic-git-policy-cli-service-0a1b2c3d";
    let scratch: Scratch = Scratch::make(git_dir.as_path(), git_dir.as_path(), name).expect("the scratch tree");
    assert_eq!(scratch.0, git_dir.join(name));
    drop(scratch);
    remove(root.as_path());
}

/// The tree a command makes carries the marker of the process that made it,
/// so a later command leaves it alone however old it is,
/// and takes it once that process is gone.
#[test]
fn a_made_tree_names_the_command_that_made_it() {
    let root: PathBuf = fixture("ready-shapes-scratch-marked");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let name: &str = "monochromatic-git-policy-cli-service-0a1b2c3d";
    let scratch: Scratch = Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree");
    let marker: PathBuf = scratch.0.join(OWNER_MARKER);
    let marked = marker_of(marker.as_path()).expect("the tree names the command that made it");
    assert!(holder_is_running(marked.holder.as_ref()), "the marker names this live process");
    // A live command can hold its tree for longer than any age rule allows for, so the age never decides the tree's fate.
    age(scratch.0.as_path(), UNMARKED_SCRATCH_TREE_AGE * 2);
    assert_eq!(remove_abandoned_trees(common.join(REPOSITORY_SCRATCH_NAME).as_path()), 0);
    assert!(scratch.0.exists(), "a later command keeps a live command's tree");
    drop(scratch);
    remove(root.as_path());
}

/// A tree a command makes is renewed while that command lives,
/// and the renewal stops when the command lets the tree go,
/// so a sweep of the place by age never takes a live tree
/// and a killed command's tree ages out as it did before.
#[test]
fn a_made_tree_is_renewed_while_its_command_lives() {
    let root: PathBuf = fixture("ready-shapes-scratch-renewed");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let scratch: Scratch = Scratch::make(common.as_path(), common.as_path(), "monochromatic-git-policy-cli-service-0a1b2c3d").expect("the scratch tree");
    assert!(thread_names().iter().any(|name| { return name == "scratch-renewal"; }), "a made tree is renewed while the command lives: {:?}", thread_names());
    drop(scratch);
    let began: std::time::Instant = std::time::Instant::now();
    while thread_names().iter().any(|name| { return name == "scratch-renewal"; }) {
        assert!(began.elapsed() < Duration::from_secs(15), "the renewal of a released tree stops");
        std::thread::sleep(Duration::from_millis(20));
    }
    remove(root.as_path());
}
