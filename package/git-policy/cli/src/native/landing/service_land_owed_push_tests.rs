//! Controls for what the service leaves for itself when a commit lands:
//! the push record,
//! written before the client is answered,
//! so a wrapper killed between the answer and its hand-off
//!  loses no push.

use super::tests::{World, request, world};
use super::*;
use crate::service_captures::CaptureMemory;
use crate::service_protocol::ServiceMessage;
use crate::service_reply::Reply;
use crate::service_server_land_group::land_group;
use crate::service_timing::Arrival;
use crate::test_support::remove;
use crate::work_item::{PENDING, WorkKind, state_directory};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::unbounded_channel;

/// Owner's rule "Commits should never fail" and the standing expectation that a landed commit is pushed:
/// the wrapper hands the push over after the answer,
/// so a wrapper killed or closed between the two loses it.
/// The service knows the commit landed,
/// and leaves the push record before it answers.
#[test]
fn a_landed_commit_leaves_its_push_record_before_the_client_is_answered() {
    let world: World = world("service-land-owed-push");
    let asked: CommitRequest = request(&world, "pushed", "pushed.txt", "pushed\n");
    let (reply, mut messages) = Reply::new();
    let (wake, mut wakes) = unbounded_channel();
    let group = vec![(Box::new(asked.clone()), reply, Arrival { accepted: Instant::now(), read: Duration::ZERO })];
    let mut seq: u64 = 0;
    let mut landed: HashMap<String, String> = HashMap::new();
    let mut captures: CaptureMemory = CaptureMemory::default();
    land_group(&world.context, None, &mut seq, group, &mut landed, &mut captures, &wake);
    let Some(ServiceMessage::Landed { commit, already, .. }) = messages.try_recv().ok() else {
        panic!("the client is answered that the commit landed");
    };
    assert!(!already);
    let git_dir: PathBuf = world.repo.join(".git");
    let work_dir: PathBuf = crate::service_work::work_directory(git_dir.as_path());
    let record: PathBuf = state_directory(work_dir.as_path(), PENDING).join(format!("{}-{commit}", WorkKind::Push.name()));
    assert!(record.exists(), "the service left the push of the commit it landed at {}", record.display());
    let woken: (PathBuf, Option<crate::service_review_key::ReviewKey>) = wakes.try_recv().expect("the work thread is woken for the record");
    assert_eq!(woken.0, git_dir, "the wake names the worktree whose commit landed");
    remove(world.root.as_path());
}
