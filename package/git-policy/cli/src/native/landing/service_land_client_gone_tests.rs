//! Controls for a commit whose client is gone before it lands:
//! the commit is not made,
//! and the rest of the batch still lands.
//! Why:
//!  The person's own words for a commit that is not made are "nothing lands later by itself",
//!  and native Git makes none when its command was ended,
//!  so a request nobody is waiting for answers nothing and makes nothing
//!  (`SPEC.md`, "The commit protocol").

use super::tests::{World, request, world};
use super::*;
use crate::service_index_lock::{Announce, Announcer};
use crate::service_journal::OPEN_EXTENSION;
use crate::test_support::{git_text, remove};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

/// What:
///  An announcer whose client is gone from the first ask on.
/// Why:
///  It is the state a killed command leaves: the service's channel to it is closed,
///  so nothing reads an answer.
struct Gone;

impl Announcer for Gone {
    fn announce(&self, _note: Announce) {}

    fn client_is_gone(&self, _place: usize) -> bool {
        return true;
    }
}

/// What:
///  An announcer whose first request is gone and whose other requests are there.
/// Why:
///  It is the batch whose first client was killed and whose second one still waits.
struct GoneFirst;

impl Announcer for GoneFirst {
    fn announce(&self, _note: Announce) {}

    fn client_is_gone(&self, place: usize) -> bool {
        return place == 0;
    }
}

/// What:
///  An announcer whose client goes away once the landing has started,
///  which is the person's interrupt while a commit waits,
///  and that lets another program's index lock go when the landing says what it waits for.
/// Why:
///  The interrupt lands in the wait for the lock,
///  which is the stretch the landing has to ask again in:
///  the first ask happens before the objects are written and answers that the client is there,
///  and every ask after it answers that it is gone.
///  The lock is let go at the notice,
///  so the control ends by itself and waits for no sleep.
struct GoingAwayAfterTheFirstAsk {
    /// How many times the landing asked whether the client is there.
    asked: AtomicUsize,
    /// The file whose appearance tells the holder of the lock to let go.
    release: PathBuf,
}

impl Announcer for GoingAwayAfterTheFirstAsk {
    fn announce(&self, note: Announce) {
        if matches!(note, Announce::Waiting(_)) {
            let _told = std::fs::write(self.release.as_path(), b"let go\n");
        }
    }

    fn client_is_gone(&self, _place: usize) -> bool {
        return self.asked.fetch_add(1, Ordering::SeqCst) > 0;
    }
}

/// What:
///  `hold_the_index`, for the landing of these controls.
/// Why:
///   A lock nothing holds is removed by the landing's own rule,
/// so only a lock a live program holds makes the wait last.
fn hold_the_index(world: &World, release: &Path) -> PathBuf {
    let lock: PathBuf = world.repo.join(".git/index.lock");
    let script: PathBuf = world.root.join("hold-the-index.sh");
    crate::test_support::executable(
        script.as_path(),
        b"#!/bin/sh\nexec 9>\"$1\"\nwhile [ ! -e \"$2\" ]; do /usr/bin/sleep 0.02; done\nrm -f \"$1\"\n".as_slice(),
    );
    let _holder = crate::test_support::spawn_owned(std::process::Command::new(script.as_path()).arg(lock.as_path()).arg(release))
        .expect("the holder of the index runs");
    let began: std::time::Instant = std::time::Instant::now();
    while !lock.exists() {
        assert!(began.elapsed() < std::time::Duration::from_secs(60), "the holder did not take {}", lock.display());
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    return lock;
}

/// A commit whose client is gone before anything is written is not made:
/// the worktree,
/// the index and the branch are as they were,
/// and no record of the batch is left for the next command to read.
#[test]
fn a_commit_whose_client_is_gone_is_not_made() {
    let world: World = world("land-client-gone");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let asked: CommitRequest = request(&world, "gone", "gone.txt", "gone\n");
    let outcomes = land_batch_announcing(&world.context, 1, std::slice::from_ref(&asked), &Gone).expect("nothing fails: nothing lands");
    assert_eq!(outcomes, vec![Outcome::ClientGone], "the request makes no commit");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), base, "the branch did not move");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "?? gone.txt", "the worktree is as it was");
    assert!(!crate::service_journal::batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION).exists(), "no batch is left open");
    assert!(!crate::owed_index_update::path_of(world.repo.join(".git").as_path(), 1).exists(), "a batch that did not land owes nothing");
    remove(world.root.as_path());
}

/// The person interrupts a commit that waits for another program's index lock:
/// the commit is not made,
/// so nothing lands after the interrupt,
/// and every record the attempt took is given back.
#[test]
fn a_commit_whose_client_goes_away_while_the_index_lock_is_waited_for_is_not_made() {
    let world: World = world("land-client-gone-while-waiting");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let asked: CommitRequest = request(&world, "interrupted", "interrupted.txt", "interrupted\n");
    let release: PathBuf = world.root.join("release-the-index");
    let _lock: PathBuf = hold_the_index(&world, release.as_path());
    let announce: GoingAwayAfterTheFirstAsk = GoingAwayAfterTheFirstAsk { asked: AtomicUsize::new(0), release };
    let outcomes = land_batch_announcing(&world.context, 1, std::slice::from_ref(&asked), &announce).expect("nothing fails: nothing lands");
    assert_eq!(outcomes, vec![Outcome::ClientGone], "the wait ended with the client gone, and no commit was made");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), base, "the branch did not move");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "?? interrupted.txt", "the worktree is as it was");
    assert!(!crate::service_journal::batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION).exists(), "the batch the wait took is given back");
    assert!(!crate::owed_index_update::path_of(world.repo.join(".git").as_path(), 1).exists(), "a batch that did not land owes nothing");
    remove(world.root.as_path());
}

/// What:
///  The rest of a batch lands when one of its clients is gone.
/// Why:
///  One person's interrupt must not cost another person's commit:
///  the batch is built again from the requests whose clients are still there.
///  The commit written for a gone client would also be published by the next push of the branch,
///  so no branch may name it.
#[test]
fn the_rest_of_a_batch_lands_when_one_of_its_clients_is_gone() {
    let world: World = world("land-client-gone-among-others");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let first: CommitRequest = request(&world, "first", "first.txt", "first\n");
    let second: CommitRequest = request(&world, "second", "second.txt", "second\n");
    let outcomes = land_batch_announcing(&world.context, 1, &[first, second], &GoneFirst).expect("the second lands");
    assert_eq!(outcomes[0], Outcome::ClientGone, "the commit of the client that is gone is not made");
    let Outcome::Landed { commit } = &outcomes[1] else { panic!("{:?}", outcomes[1]) };
    assert_eq!(git_text(world.repo.as_path(), &["log", "--format=%s", format!("{base}..HEAD").as_str()]), "message of second", "the rest of the batch is one commit");
    assert_eq!(git_text(world.repo.as_path(), &["ls-tree", "--name-only", commit.as_str()]), "second.txt", "the gone client's file is in no commit");
    remove(world.root.as_path());
}

/// What:
///  A batch whose only request belongs to a client that is gone makes no commit,
///  answers nothing,
///  and leaves that request's tree to the sweep.
/// Why:
///   The group drops such a request before it numbers a batch (`land_group`),
///  so a client that went away costs no landing at all;
///  the tree stays because a client that reconnects sends the same request again and reads it,
///  and the sweep takes it by age otherwise.
#[test]
fn a_batch_whose_client_is_gone_makes_no_commit() {
    use crate::service_captures::CaptureMemory;
    use crate::service_reply::Reply;
    use crate::service_server_land_group::land_group;
    use crate::service_timing::Arrival;
    let world: World = world("land-group-client-gone");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let asked: CommitRequest = request(&world, "gone", "gone.txt", "gone\n");
    let (reply, messages) = Reply::new();
    // The client is gone: the task that writes its connection ended, so nothing reads what the landing would answer.
    drop(messages);
    let group: Vec<crate::service_server_queue_types_and_send::Queued> = vec![(Box::new(asked), reply, Arrival { accepted: std::time::Instant::now(), read: std::time::Duration::ZERO })];
    let mut seq: u64 = 0;
    let mut landed: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut captures: CaptureMemory = CaptureMemory::default();
    let mut handed_back: std::collections::HashSet<String> = std::collections::HashSet::new();
    land_group(&world.context, None, &mut seq, group, &mut landed, &mut captures, crate::wait_ceilings::WAITING_NOTICE_AFTER, &mut handed_back);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), base, "the branch did not move");
    assert_eq!(seq, 0, "no batch number is taken for a request nobody waits for");
    assert!(landed.is_empty(), "no commit of a client that is gone is remembered as landed: {landed:?}");
    assert!(handed_back.contains("gone"), "the tree stays for a client that reconnects: {handed_back:?}");
    remove(world.root.as_path());
}
