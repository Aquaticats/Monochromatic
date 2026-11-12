//! Controls for the worktree's head around a landing:
//! a head that cannot be read is no answer,
//! a head attached since the last
//!  look is not replaced,
//! the late switch settles every obligation of the batch,
//! and a restart reads the head under its lock.

use super::tests::{World, built_request, git_with_index, land_with_failed_index, request, world};
use super::*;
use crate::service_fault::hooks;
use crate::service_journal::{DONE_EXTENSION, OPEN_EXTENSION, batch_path, parse_intent};
use crate::test_support::{fixture, git, git_context, git_text, remove};

/// A context whose Git is pointed at a Git directory that does not exist,
/// so every question it is asked ends with status 128.
pub(super) fn blind_context(root: &Path) -> GitContext {
    let mut context: GitContext = git_context();
    context.overlay.push((std::ffi::OsString::from("GIT_DIR"), root.join("no-such-git-dir").into_os_string()));
    return context;
}

/// A Git that cannot read the repository gives no answer about the head,
/// so a caller never takes its failure for a detached head
///  or for another branch.
#[test]
fn a_git_that_cannot_read_the_repository_is_no_answer_about_the_head() {
    let root: PathBuf = fixture("service-head-no-answer");
    let context = blind_context(root.as_path());
    for reference in ["HEAD", "refs/heads/main"] {
        let asked = still_checked_out(&context, root.as_path(), root.join("no-git-dir").as_path(), root.join("no-common").as_path(), &crate::ref_name::RefName::from_text(reference));
        assert_eq!(asked, Err(FailureCode::Unavailable), "{reference}");
    }
    let moved = follows_move(&context, root.as_path(), root.join("no-git-dir").as_path(), root.join("no-common").as_path(), &crate::ref_name::RefName::from_text("HEAD"), "0".repeat(40).as_str());
    assert_eq!(moved, Err(FailureCode::Unavailable));
    remove(root.as_path());
}

/// A detached head that was attached to a branch at the same commit after the last look is not replaced by a detached head:
/// the
///  commit goes to the branch the head names,
/// as it does for native Git's commit,
/// and the index is updated for it.
#[test]
fn a_head_attached_after_the_last_look_stays_attached() {
    let world: World = world("service-head-attached");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["checkout", "--quiet", "--detach"]);
    git(world.repo.as_path(), &["branch", "attached", base.as_str()]);
    let mut built: CommitRequest = built_request(&world, "loose", "loose.txt", &[base.as_str()]);
    built.reference = crate::ref_name::RefName::from_text("HEAD");
    let commit: String = built.commit.clone().unwrap_or_default();
    let repo: PathBuf = world.repo.clone();
    hooks::set("before-ref", std::rc::Rc::new(move || { git(repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/attached"]); }));
    let outcomes = land_batch(&world.context, 1, &[built]).expect("lands");
    assert_eq!(outcomes, vec![Outcome::Landed { commit: commit.clone() }]);
    assert_eq!(git_text(world.repo.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/attached", "the head is still attached");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "attached"]), commit, "the branch the head names took the commit");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), base, "the other branch did not move");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short", "--", "loose.txt"]), "", "the index shows the commit");
    remove(world.root.as_path());
}

/// A switch after the last look leaves the worktree's default index to its new branch but still finishes an alternate index
///  (`GIT_INDEX_FILE`),
/// which belongs to no branch.
#[test]
fn a_late_switch_still_finishes_an_alternate_index() {
    let world: World = world("service-head-late-alternate");
    git(world.repo.as_path(), &["branch", "other"]);
    let alternate: PathBuf = world.repo.join(".git/alternate-index");
    std::fs::copy(world.repo.join(".git/index"), &alternate).expect("an alternate index");
    let mut mine: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    mine.index = alternate.clone();
    let repo: PathBuf = world.repo.clone();
    hooks::set("before-ref", std::rc::Rc::new(move || { git(repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/other"]); }));
    let outcome = land_batch(&world.context, 1, std::slice::from_ref(&mine)).expect("lands");
    assert!(matches!(outcome.as_slice(), [Outcome::Landed { .. }]), "{outcome:?}");
    let listed: String = git_with_index(world.repo.as_path(), alternate.as_path(), &["ls-files", "--stage", "--", "mine.txt"]);
    assert!(!listed.is_empty(), "the alternate index shows the landing");
    assert!(batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION).exists() && !batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION).exists(), "the batch is finished");
    remove(world.root.as_path());
}

/// A switch after the last look does not finish the batch while its conclusion state is still there:
/// the batch stays open,
/// so a
///  restart removes the state.
#[test]
fn a_late_switch_keeps_the_batch_open_while_its_cleanup_is_pending() {
    let world: World = world("service-head-late-cleanup");
    git(world.repo.as_path(), &["branch", "other"]);
    let scratch: PathBuf = world.root.join("scratch");
    let copies: PathBuf = scratch.join(crate::conclusion_cleanup::CONCLUSION_COPY_DIRECTORY);
    std::fs::create_dir_all(copies.as_path()).expect("a copy directory");
    // A record that is not JSON is damage in every backend, so this batch's cleanup cannot be done.
    std::fs::write(copies.join(crate::conclusion_cleanup::STORE_RECORD_FILENAME), b"{").expect("a torn record");
    let mut mine: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    mine.cleanup = Some(crate::service_protocol::Cleanup { git_dir: world.root.join("no-such-git-dir"), scratch, reftable: false });
    let repo: PathBuf = world.repo.clone();
    hooks::set("before-ref", std::rc::Rc::new(move || { git(repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/other"]); }));
    let outcome = land_batch(&world.context, 1, std::slice::from_ref(&mine)).expect("lands");
    assert!(matches!(outcome.as_slice(), [Outcome::Landed { .. }]), "{outcome:?}");
    assert!(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION).exists(), "the batch stays open while its cleanup cannot be done");
    assert!(!world.repo.join(".git/index.lock").exists());
    remove(world.root.as_path());
}

/// A restart reads the head again under the lock it takes:
/// a worktree that left the branch between its first look and the lock
///  keeps the index of the branch it is on.
#[test]
fn a_restart_reads_the_head_under_its_lock() {
    let world: World = world("service-head-under-lock");
    git(world.repo.as_path(), &["branch", "other"]);
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    land_with_failed_index(&world, 1, &[landed]);
    let before: Vec<u8> = std::fs::read(world.repo.join(".git/index")).expect("index");
    let repo: PathBuf = world.repo.clone();
    hooks::set("restart-locked", std::rc::Rc::new(move || { git(repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/other"]); }));
    recover(&world.context);
    assert_eq!(std::fs::read(world.repo.join(".git/index")).expect("index"), before, "the index of the branch checked out was not touched");
    assert!(batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION).exists(), "the batch is finished");
    remove(world.root.as_path());
}

/// A head that cannot be read stops a restart before it touches anything:
/// the batch stays open and the index is as it was.
#[test]
fn a_restart_that_cannot_read_the_head_leaves_the_batch_open() {
    let world: World = world("service-head-unreadable");
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    land_with_failed_index(&world, 1, &[landed]);
    let open: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    let mut intent: Intent = parse_intent(std::fs::read_to_string(&open).expect("read").as_str()).expect("parse");
    // The worktree's Git directory is not a Git directory, and Git is asked in a directory that is no repository.
    intent.git_dir = world.root.join("no-git-dir");
    intent.index = intent.git_dir.join("index");
    intent.repository = world.root.clone();
    let settled = crate::service_land_finish::settle(&blind_context(world.root.as_path()), &intent, world.context.journal.as_path());
    assert!(matches!(settled, crate::service_land_finish::Settled::Open), "the batch stays open");
    assert!(!world.root.join("no-git-dir").exists(), "nothing was made where the index would be");
    remove(world.root.as_path());
}
