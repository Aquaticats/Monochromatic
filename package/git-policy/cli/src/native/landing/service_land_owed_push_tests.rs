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
    land_group(&world.context, None, &mut seq, group, &mut landed, &mut captures);
    let Some(ServiceMessage::Landed { commit, already, .. }) = messages.try_recv().ok() else {
        panic!("the client is answered that the commit landed");
    };
    assert!(!already);
    let git_dir: PathBuf = world.repo.join(".git");
    let work_dir: PathBuf = crate::service_work::work_directory(git_dir.as_path());
    let id: String = format!("{}-{commit}", WorkKind::Push.name());
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

/// A held push of a command that is still deciding is never promoted by another one:
/// the owner is this live process,
/// so a blocked gate still wins.
#[test]
fn a_held_push_of_a_living_command_is_not_promoted() {
    let world: World = world("service-land-owed-push-owner");
    let work_dir: PathBuf = crate::service_work::work_directory(world.repo.join(".git").as_path());
    let item: crate::work_item::WorkItem = crate::work_item::WorkItem {
        kind: WorkKind::Push,
        repository: world.repo.display().to_string(),
        git_dir: world.repo.join(".git").display().to_string(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from("0".repeat(40)),
        parent: String::from("0".repeat(40)),
        created: crate::iso_time::now_iso(),
        tries: 0,
    };
    crate::work_item::hold(work_dir.as_path(), &item).expect("held");
    crate::work_item::claim(work_dir.as_path(), item.id().as_str());
    assert_eq!(crate::work_item::promote_held(work_dir.as_path()), 0, "a living command's gate is not overtaken");
    assert!(!state_directory(work_dir.as_path(), PENDING).join(item.id()).exists(), "nothing runnable was made");
    // The command is gone when no owner names a process: the next command may finish its push.
    let _gone: std::io::Result<()> = std::fs::remove_file(state_directory(work_dir.as_path(), crate::work_item::HELD).join(format!("{}.owner", item.id())));
    assert_eq!(crate::work_item::promote_held(work_dir.as_path()), 1, "an ownerless record is a dead command's");
    assert!(state_directory(work_dir.as_path(), PENDING).join(item.id()).exists(), "the push is runnable");
    remove(world.root.as_path());
}

/// A batch that landed and whose service was killed before it held the pushes leaves recovery to hold them.
#[test]
fn recovery_holds_the_push_of_a_batch_it_finishes() {
    let world: World = world("service-land-owed-push-recovery");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let work_dir: PathBuf = crate::service_work::work_directory(world.repo.join(".git").as_path());
    let record: PathBuf = state_directory(work_dir.as_path(), crate::work_item::HELD).join(format!("push-{commit}"));
    // The kill came before the landing held it.
    let _gone: std::io::Result<()> = std::fs::remove_file(record.as_path());
    assert!(!record.exists());
    let reports: Vec<RecoveryReport> = recover(&world.context);
    assert!(reports.iter().any(|report| { return report.seq == 1 && report.decision == Recovery::Finish; }), "{reports:?}");
    assert!(record.exists(), "the push of the landed commit is held by recovery");
    assert_eq!(git_text(world.repo.as_path(), &["log", "-1", "--format=%s"]), "message of landed");
    remove(world.root.as_path());
}
