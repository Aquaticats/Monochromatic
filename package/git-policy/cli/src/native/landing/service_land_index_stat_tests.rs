//! Controls for the stat data of the index a landing installs:
//! a stale entry the batch does not change is refreshed there,
//! a refresh that fails or is interrupted costs the landing its time and never its result,
//! a refresh never hides an unstaged change,
//! and an index that already describes the worktree costs no Git at all.

use super::tests::{World, request, world};
use super::*;
use crate::test_support::{REAL_GIT, executable, git, git_context, git_text, remove, renew_inode};

/// The file a control makes stale before it lands:
/// a committed path no request of the batch names.
const STALE_FILE: &str = "stale.txt";

/// The ceiling a control gives every step of its landing:
/// short enough that a Git which never returns is ended in a moment,
/// and long enough for the steps of a fixture this size.
const STEP_CEILING: std::time::Duration = std::time::Duration::from_secs(3);

/// Commits `name` with `content` to the world's branch, through Git alone.
fn commit_into(world: &World, name: &str, content: &str) {
    std::fs::write(world.repo.join(name), content).expect("write");
    git(world.repo.as_path(), &["add", "--", name]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=before"]);
}

/// What `git ls-files --debug` prints for `name` in the world's real index.
fn recording(world: &World, name: &str) -> String {
    return git_text(
        world.repo.as_path(),
        &["--no-optional-locks", "ls-files", "--debug", "--", name],
    );
}

/// The inode `name` has now, as `ls-files --debug` prints it.
fn inode_of(world: &World, name: &str) -> u64 {
    return std::os::unix::fs::MetadataExt::ino(&std::fs::metadata(world.repo.join(name)).expect("the file"));
}

/// The world's index made to record stat data its worktree no longer has,
/// as the move to another disk left every worktree,
/// with `STALE_FILE` as the path no request names.
fn make_the_index_stale(world: &World) {
    commit_into(world, STALE_FILE, "stale\n");
    renew_inode(world.repo.join(STALE_FILE).as_path());
    let before: String = recording(world, STALE_FILE);
    assert!(
        !before.contains(format!("ino: {}", inode_of(world, STALE_FILE)).as_str()),
        "the fixture's index must be stale, or this control proves nothing: {before}"
    );
}

/// What:
///  The same world whose landings run a Git that answers the stat refresh with `answering`,
///  passing every other command to the real Git.
/// Why:
///  Git cannot be made to fail at one step on demand,
/// and a shell is the only way to name that step's outcome.
fn with_a_shimmed_git(world: World, answering: &str) -> World {
    let shim: PathBuf = world.root.join("shim-git");
    let script: String = format!("#!/bin/sh\ncase \" $* \" in\n  *\" update-index --refresh \"*) {answering} ;;\n  *) exec '{REAL_GIT}' \"$@\";;\nesac\n");
    executable(shim.as_path(), script.as_bytes());
    let context: LandContext = LandContext {
        git: GitContext { real_git: shim, ..git_context() },
        journal: world.context.journal.clone(),
        on_objects_written: None,
        lock_ceiling: world.context.lock_ceiling,
    };
    return World { root: world.root, repo: world.repo, context };
}

/// The stat data of an entry the batch does not change is refreshed in the index the landing installs,
/// so the commit after this one copies an index that already describes the worktree:
///  the landing holds the real index lock anyway,
/// so this is the one place a batch may pay for the refresh.
#[test]
fn a_landing_keeps_the_stat_data_of_a_stale_entry_it_does_not_change() {
    let world: World = world("service-land-index-stat-kept");
    make_the_index_stale(&world);
    let inode: u64 = inode_of(&world, STALE_FILE);
    let outcomes = land_batch(&world.context, 1, &[request(&world, "one", "one.txt", "one\n")]).expect("landed");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{outcomes:?}");
    let after: String = recording(&world, STALE_FILE);
    assert!(
        after.contains(format!("ino: {inode}").as_str()),
        "the installed index must record the inode the file has now, {inode}: {after}"
    );
    remove(world.root.as_path());
}

/// A refresh Git refuses is a refresh that was not kept:
/// the landing installs the index it built anyway,
/// and the commit lands.
#[test]
fn a_landing_whose_refresh_fails_still_installs_its_index() {
    let world: World = world("service-land-index-stat-refused");
    make_the_index_stale(&world);
    let world: World = with_a_shimmed_git(world, "exit 128");
    let outcomes = land_batch(&world.context, 1, &[request(&world, "one", "one.txt", "one\n")]).expect("a refresh Git refused must not fail the landing");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{outcomes:?}");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing");
    remove(world.root.as_path());
}

/// A refresh whose Git was killed is the same outcome reached the other way:
/// the landing installs its index and the commit lands.
#[test]
fn a_landing_whose_refresh_is_killed_still_installs_its_index() {
    let world: World = world("service-land-index-stat-killed");
    make_the_index_stale(&world);
    let world: World = with_a_shimmed_git(world, "kill -9 $$");
    let outcomes = land_batch(&world.context, 1, &[request(&world, "one", "one.txt", "one\n")]).expect("a refresh that was killed must not fail the landing");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{outcomes:?}");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing");
    remove(world.root.as_path());
}

/// A refresh the step's ceiling ends is a refresh that was not kept:
/// the landing installs the index it built and the commit lands.
///  The completion is the assertion:
/// the Git the shim starts sleeps for longer than the whole run may show no progress,
/// so a landing that waited for it instead of ending it at the ceiling could not finish.
#[test]
fn a_landing_whose_refresh_is_ended_at_its_ceiling_still_installs_its_index() {
    let world: World = world("service-land-index-stat-ceiling");
    make_the_index_stale(&world);
    let world: World = with_a_shimmed_git(world, "sleep 600");
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(STEP_CEILING)); });
    let outcomes = land_batch(&world.context, 1, &[request(&world, "one", "one.txt", "one\n")]);
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    let outcomes = outcomes.expect("a refresh the ceiling ended must not fail the landing");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{outcomes:?}");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing");
    remove(world.root.as_path());
}

/// A refresh never records stat data for a path whose bytes the commit does not hold:
/// the person's unstaged change is still a change after the landing,
/// in `git status` and in `git diff`.
#[test]
fn a_landing_whose_refresh_meets_an_unstaged_change_keeps_it_visible() {
    let world: World = world("service-land-index-stat-unstaged");
    make_the_index_stale(&world);
    std::fs::write(world.repo.join(STALE_FILE), "edited but not staged\n").expect("an unstaged change");
    let outcomes = land_batch(&world.context, 1, &[request(&world, "one", "one.txt", "one\n")]).expect("landed");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{outcomes:?}");
    let status: String = git_text(world.repo.as_path(), &["status", "--short"]);
    assert!(status.contains(STALE_FILE), "the unstaged change is still a change: {status}");
    let difference: String = git_text(world.repo.as_path(), &["diff", "--", STALE_FILE]);
    assert!(difference.contains("edited but not staged"), "the worktree bytes are the person's: {difference}");
    remove(world.root.as_path());
}

/// An index that already describes the worktree costs the landing no Git start at all:
/// the Git of the world leaves a file behind when it is asked for a refresh,
/// and a landing that asked would leave it.
#[test]
fn a_landing_over_an_index_that_describes_the_worktree_starts_no_refresh() {
    let world: World = world("service-land-index-stat-fresh");
    commit_into(&world, STALE_FILE, "stale\n");
    let asked: PathBuf = world.root.join("refresh-asked");
    let world: World = with_a_shimmed_git(world, format!(": > '{}'", asked.display()).as_str());
    let outcomes = land_batch(&world.context, 1, &[request(&world, "one", "one.txt", "one\n")]).expect("landed");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{outcomes:?}");
    assert!(!asked.exists(), "an index that describes the worktree must cost no Git start, but the refresh was asked for");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing");
    remove(world.root.as_path());
}
