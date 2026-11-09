//! Controls for what the service leaves for itself when a commit lands:
//! the push record,
//! written before the client is answered,
//! so a wrapper killed between the answer and its hand-off
//!  loses no push.

use super::tests::{World, land_with_failed_index, request, world};
use super::*;
use crate::service_captures::CaptureMemory;
use crate::service_protocol::ServiceMessage;
use crate::service_reply::Reply;
use crate::service_server_land_group::land_group;
use crate::service_timing::Arrival;
use crate::test_support::{git_text, remove};
use crate::work_item::{PENDING, WorkKind, state_directory};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::unbounded_channel;

/// The post-commit gate runs in the wrapper,
/// after the answer,
/// and only its verdict may let a landed commit be pushed:
/// a blocked gate keeps the commit local.
/// The service therefore holds the push record and leaves it unrunnable,
/// so no round can publish a commit whose gate blocks it,
/// while a client killed between the answer and its own record loses nothing.
#[test]
fn a_landed_commit_holds_its_push_where_no_round_runs_it() {
    let world: World = world("service-land-owed-push");
    let asked: CommitRequest = request(&world, "pushed", "pushed.txt", "pushed\n");
    let (reply, mut messages) = Reply::new();
    let (_wake, mut wakes) = unbounded_channel::<crate::service_work::Wake>();
    let group = vec![(Box::new(asked.clone()), reply, Arrival { accepted: Instant::now(), read: Duration::ZERO })];
    let mut seq: u64 = 0;
    let mut landed: HashMap<String, String> = HashMap::new();
    let mut captures: CaptureMemory = CaptureMemory::default();
    let _kept: Vec<String> = land_group(&world.context, None, &mut seq, group, &mut landed, &mut captures, crate::wait_ceilings::WAITING_NOTICE_AFTER);
    let Some(ServiceMessage::Landed { commit, already, .. }) = messages.try_recv().ok() else {
        panic!("the client is answered that the commit landed");
    };
    assert!(!already);
    let git_dir: PathBuf = world.repo.join(".git");
    let work_dir: PathBuf = crate::service_work::work_directory(git_dir.as_path());
    let id: String = crate::work_item::WorkItem {
        kind: WorkKind::Push,
        repository: world.repo.clone(),
        git_dir: world.repo.join(".git"),
        branch_ref: String::from("refs/heads/main"),
        oid: commit.clone(),
        parent: String::new(),
        created: String::new(),
        tries: 0,
    }
    .id();
    assert!(
        !state_directory(work_dir.as_path(), PENDING).join(id.as_str()).exists(),
        "no runnable push record waits for a slot: another round cannot push the commit before its gate"
    );
    assert!(
        state_directory(work_dir.as_path(), crate::work_item::HELD).join(id.as_str()).exists(),
        "the push waits where nothing drains it, until the wrapper's gate promotes it"
    );
    // The producer's decision is one rename; a refused one unlinks the held record and leaves nothing runnable.
    assert!(crate::work_item::promote(work_dir.as_path(), id.as_str()), "the gate's approval makes it runnable");
    assert!(state_directory(work_dir.as_path(), PENDING).join(id.as_str()).exists());
    crate::work_item::forget_held(work_dir.as_path(), id.as_str());
    assert!(wakes.try_recv().is_err(), "the service wakes no work of its own: the wrapper's hand-off is what asks for the push");
    let _ = &world;
    remove(world.root.as_path());
}

/// A push whose command died before its verdict stays held through a service restart:
/// nothing releases it by judging a process,
/// and nothing drains it,
/// so nothing is pushed.
#[test]
fn a_held_push_stays_held_when_the_service_restarts() {
    let world: World = world("service-land-owed-push-restart");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let work_dir: PathBuf = crate::service_work::work_directory(world.repo.join(".git").as_path());
    let held_name: String = crate::work_item::WorkItem {
        kind: WorkKind::Push,
        repository: world.repo.clone(),
        git_dir: world.repo.join(".git"),
        branch_ref: String::from("refs/heads/main"),
        oid: commit.clone(),
        parent: String::new(),
        created: String::new(),
        tries: 0,
    }
    .id();
    let held: PathBuf = state_directory(work_dir.as_path(), crate::work_item::HELD).join(held_name.as_str());
    assert!(held.exists(), "the landing held the push");
    // The restart: the queue is drained as a service start does, and nothing about an owner is judged.
    let handlers: crate::service_work::GitHandlers = crate::service_work::GitHandlers::new(crate::test_support::git_context());
    let _drained = crate::service_work::drain(world.repo.join(".git").as_path(), &handlers, &crate::claim_time::ClaimTime::read());
    assert!(held.exists(), "the held push is still held after the restart");
    assert!(!state_directory(work_dir.as_path(), PENDING).join(held_name.as_str()).exists(), "nothing was made runnable");
    remove(world.root.as_path());
}

/// Every command in the worktree says once,
/// in one line,
/// which commit is held:
/// the notice is recorded beside the push failures and taken as it is shown.
#[test]
fn every_command_says_which_commit_is_held() {
    let world: World = world("service-land-owed-push-notice");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let shown: Vec<String> = crate::service_handoff::held_push_notices(world.repo.join(".git").as_path(), &crate::held_push_commit_on_the_remote::NothingAsked);
    assert_eq!(shown.len(), 1, "one line: {shown:?}");
    assert!(shown[0].starts_with(format!("the commit {commit} is held").as_str()), "the line names the commit, not the record's file name: {}", shown[0]);
    assert!(shown[0].contains("held") && shown[0].contains("has not been released yet") && shown[0].contains("next push"), "{}", shown[0]);
    // What is known is said; whether the command is still running cannot be told, so the line does not claim it stopped.
    assert!(!shown[0].contains("stopped"), "{}", shown[0]);
    // The state stands, so the line stands while the record does: nothing has to be written or cleared for it.
    assert_eq!(crate::service_handoff::held_push_notices(world.repo.join(".git").as_path(), &crate::held_push_commit_on_the_remote::NothingAsked), shown, "the line is the record's");
    // The record itself is still held: a notice is not a release.
    let work_dir: PathBuf = crate::service_work::work_directory(world.repo.join(".git").as_path());
    let held_name: String = crate::work_item::WorkItem {
        kind: WorkKind::Push,
        repository: world.repo.clone(),
        git_dir: world.repo.join(".git"),
        branch_ref: String::from("refs/heads/main"),
        oid: commit.clone(),
        parent: String::new(),
        created: String::new(),
        tries: 0,
    }
    .id();
    assert!(state_directory(work_dir.as_path(), crate::work_item::HELD).join(held_name.as_str()).exists(), "the push is still held");
    remove(world.root.as_path());
}
