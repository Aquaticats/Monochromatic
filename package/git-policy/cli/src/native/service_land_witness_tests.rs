//! Controls for a replay of a landing whose index update did not install: it is made only over the index file the landing saw
//!  under its lock, and every other index is left as its writers left it.

use super::tests::{World, built_request, land_with_failed_index, request, world};
use super::*;
use crate::service_fault::hooks;
use crate::service_journal::{DONE_EXTENSION, OPEN_EXTENSION, batch_path};
use crate::test_support::{git, git_text, remove};

/// Commits `path` with `content` as the branch's tip, so a landing starts from an index that has an entry for it.
fn commit_file(world: &World, path: &str, content: &str) {
    std::fs::write(world.repo.join(path), content).expect("write");
    git(world.repo.as_path(), &["add", "--", path]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message", format!("start {path}").as_str()]);
}

/// Stages `content` at `path` the way another session does: the worktree file is written and added.
fn stage(world: &World, path: &str, content: &str) {
    std::fs::create_dir_all(world.repo.join(path).parent().expect("parent")).expect("directories");
    std::fs::write(world.repo.join(path), content).expect("write");
    git(world.repo.as_path(), &["add", "--", path]);
}

fn staged(world: &World, path: &str) -> String {
    return git_text(world.repo.as_path(), &["show", format!(":{path}").as_str()]);
}

/// The control that must move: a replay over the index file the landing saw finishes the index update.
#[test]
fn a_replay_over_the_index_the_landing_saw_finishes_the_update() {
    let world: World = world("service-witness-untouched");
    commit_file(&world, "a.txt", "A\n");
    let landed: CommitRequest = request(&world, "landed", "a.txt", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    assert_eq!(staged(&world, "a.txt"), "A", "the index update did not install");
    recover(&world.context);
    assert_eq!(staged(&world, "a.txt"), "B", "the replay finished the update");
    assert!(batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION).exists());
    remove(world.root.as_path());
}

/// A path staged back to exactly the entry the landing started from was staged by someone: the same entry is not the same
///  history, so the replay does not put the landed content over it.
#[test]
fn a_path_staged_back_to_the_starting_entry_is_kept() {
    let world: World = world("service-witness-aba");
    commit_file(&world, "a.txt", "A\n");
    let landed: CommitRequest = request(&world, "landed", "a.txt", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    // Another session stages another version, then the version the landing started from.
    stage(&world, "a.txt", "C\n");
    stage(&world, "a.txt", "A\n");
    recover(&world.context);
    assert_eq!(staged(&world, "a.txt"), "A", "the later staging survives");
    assert!(batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION).exists(), "and the batch is finished, with its index work left");
    remove(world.root.as_path());
}

/// A commit Git built lists its paths only when it is finished, and it is replayed like any other batch: over the file the
///  landing saw, or not at all.
#[test]
fn a_commit_git_built_is_not_replayed_over_later_staging() {
    let world: World = world("service-witness-built");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "built", "built.txt", &[base.as_str()]);
    land_with_failed_index(&world, 1, &[built]);
    stage(&world, "built.txt", "someone else's version\n");
    recover(&world.context);
    assert_eq!(staged(&world, "built.txt"), "someone else's version", "the later staging survives");
    remove(world.root.as_path());
}

/// A batch with no start record (an older one, or a landing that could not write it) is not replayed: nothing says what the
///  index was.
#[test]
fn a_batch_with_no_start_record_is_not_replayed() {
    let world: World = world("service-witness-no-record");
    commit_file(&world, "a.txt", "A\n");
    let landed: CommitRequest = request(&world, "landed", "a.txt", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    std::fs::remove_file(world.context.journal.join("1.start")).expect("an older batch has no record");
    stage(&world, "a.txt", "C\n");
    recover(&world.context);
    assert_eq!(staged(&world, "a.txt"), "C", "the later staging survives");
    assert!(batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION).exists(), "the batch is finished, its index work left");
    remove(world.root.as_path());
}

/// A record that cannot be read now decides nothing: the batch stays open and the index alone.
#[test]
fn a_start_record_that_cannot_be_read_leaves_the_batch_open() {
    let world: World = world("service-witness-unreadable");
    commit_file(&world, "a.txt", "A\n");
    let landed: CommitRequest = request(&world, "landed", "a.txt", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    std::fs::remove_file(world.context.journal.join("1.start")).expect("remove");
    // A directory where the record belongs is present and cannot be read.
    std::fs::create_dir(world.context.journal.join("1.start")).expect("an unreadable record");
    recover(&world.context);
    assert_eq!(staged(&world, "a.txt"), "A", "the index was left alone");
    assert!(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION).exists(), "the batch stays open");
    remove(world.root.as_path());
}

/// The write footprint of a reset is more than the names it was given: a file landed at `a` is reset by replacing every entry
///  under `a/`. A directory another session staged where the landed file is must survive the replay.
#[test]
fn a_directory_staged_where_the_landed_file_is_survives_the_replay() {
    let world: World = world("service-witness-footprint");
    let landed: CommitRequest = request(&world, "landed", "a", "a file\n");
    land_with_failed_index(&world, 1, &[landed]);
    std::fs::remove_file(world.repo.join("a")).expect("remove the file");
    stage(&world, "a/b", "below\n");
    recover(&world.context);
    assert_eq!(git_text(world.repo.as_path(), &["ls-files", "--stage", "--", "a", "a/b"]).split_whitespace().last(), Some("a/b"), "the staged descendant is still staged");
    remove(world.root.as_path());
}

/// The inverse: a file staged where the landed directory's entries are.
#[test]
fn a_file_staged_where_the_landed_directory_is_survives_the_replay() {
    let world: World = world("service-witness-footprint-inverse");
    commit_file(&world, "a", "a file\n");
    std::fs::remove_file(world.repo.join("a")).expect("remove the file");
    git(world.repo.as_path(), &["rm", "--cached", "--quiet", "--", "a"]);
    let landed: CommitRequest = request(&world, "landed", "a/b", "below\n");
    land_with_failed_index(&world, 1, &[landed]);
    std::fs::remove_dir_all(world.repo.join("a")).expect("remove the directory");
    stage(&world, "a", "a file again\n");
    recover(&world.context);
    assert_eq!(staged(&world, "a"), "a file again", "the staged file is still staged");
    remove(world.root.as_path());
}

/// A landing stopped after its start record and before the branch moved leaves the branch and the index as they were, and the
///  restart forgets the batch with its record.
#[test]
fn a_landing_stopped_after_the_start_record_and_before_the_move_is_dropped() {
    let world: World = world("service-witness-before-ref");
    commit_file(&world, "a.txt", "A\n");
    let before: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let landed: CommitRequest = request(&world, "landed", "a.txt", "B\n");
    hooks::set("before-ref", std::rc::Rc::new(|| { panic!("stopped before the branch moved"); }));
    let stopped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { return land_batch(&world.context, 1, &[landed]); }));
    assert!(stopped.is_err());
    assert!(world.context.journal.join("1.start").exists(), "the record was written before the branch moved");
    recover(&world.context);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), before, "the branch did not move");
    assert_eq!(staged(&world, "a.txt"), "A");
    assert!(!world.context.journal.join("1.start").exists() && !batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION).exists(), "nothing of the batch is left");
    remove(world.root.as_path());
}

/// A landing stopped after the branch moved and before the index was built is finished by the restart over the index it saw.
#[test]
fn a_landing_stopped_after_the_move_is_finished_by_the_restart() {
    let world: World = world("service-witness-after-ref");
    commit_file(&world, "a.txt", "A\n");
    let landed: CommitRequest = request(&world, "landed", "a.txt", "B\n");
    hooks::set("after-ref", std::rc::Rc::new(|| { panic!("stopped after the branch moved"); }));
    let stopped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { return land_batch(&world.context, 1, &[landed]); }));
    assert!(stopped.is_err());
    assert_eq!(staged(&world, "a.txt"), "A");
    recover(&world.context);
    assert_eq!(staged(&world, "a.txt"), "B");
    remove(world.root.as_path());
}

/// A path staged since leaves every path of the batch as it is, the untouched ones too: whether a path was touched cannot be
///  told from its entry, and a reset of one path can rewrite the entries around it.
#[test]
fn a_changed_index_resets_no_path_of_the_batch() {
    let world: World = world("service-witness-whole-batch");
    commit_file(&world, "kept.txt", "A\n");
    commit_file(&world, "other.txt", "A\n");
    let batch: Vec<CommitRequest> = vec![request(&world, "kept", "kept.txt", "B\n"), request(&world, "other", "other.txt", "B\n")];
    land_with_failed_index(&world, 1, &batch);
    stage(&world, "kept.txt", "staged after the landing\n");
    recover(&world.context);
    assert_eq!(staged(&world, "kept.txt"), "staged after the landing", "the later staging is kept");
    assert_eq!(staged(&world, "other.txt"), "A", "no path of the batch is reset over an index that may hold staging");
    assert!(batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION).exists(), "the batch is finished");
    remove(world.root.as_path());
}
