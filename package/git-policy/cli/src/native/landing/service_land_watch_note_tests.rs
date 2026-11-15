//! Controls for the note a landing writes for the service that starts next
//! (`service_claim_watch.rs`,
//! `note_for_the_next_service_within`):
//! the note's write is held on the thread that makes it,
//! which is what a journal directory on a filesystem that stopped answering does to it,
//! and the commit that waits for that note,
//! and the commit submitted behind it,
//! are answered inside the ceiling that write is granted.
//! The held write is let go by each control before it ends,
//! so every control ends in seconds and leaves no thread waiting.
//!
//! The service and its clients are the ones `service_land_hung_write_tests.rs` runs,
//! and the fixtures are the ones the landing's controls make.

use super::hung_write_tests::{PATIENCE, Serving, Submitting, client, serve_on_a_thread_with_settings, submit_on_a_thread, wait_until_settled};
use super::tests::{World, request, world};
use super::*;
use crate::service_claim_watch::{ClaimWatch, IN_THE_NEXT_SERVICE_NOTE, WATCH_DIRECTORY};
use crate::service_client::Submitted;
use crate::service_fault::hooks::{HoldOnAnyThread, hold_on_any_thread};
use crate::service_handoff::{ServiceHandoff, hand_over, promote_the_push};
use crate::service_messages::ClientFailure;
use crate::service_server::ServeSettings;
use crate::test_support::{git_text, remove};
use std::time::{Duration, Instant};

/// The ceiling the landings of these controls grant a note's write:
/// short,
/// so a control ends in seconds,
/// and its value plays no part in what a commit is answered.
const NOTE_CEILING: Duration = Duration::from_millis(500);

/// The ceiling the landing of a run whose store answers is granted in these controls:
/// long,
/// so a commit that waited for it would be seen waiting.
const A_CEILING_NOBODY_WAITS_FOR: Duration = Duration::from_secs(30);

/// The most a commit may wait beside the ceiling for its own note:
/// what the rest of the answer's path costs is a send on a channel,
/// so anything past this is a wait that is not the ceiling's.
const BESIDE_THE_CEILING: Duration = Duration::from_millis(2500);

/// The ceiling the clients of these controls give the service's answer:
/// past the bound each control asserts,
/// so an answer that comes is the bound's and never the client's ceiling.
const ANSWER_CEILING: Duration = Duration::from_millis(6000);

/// A service serving on a thread of its own,
/// whose landings grant `ceiling` for a note.
fn serving_with_ceiling(world: &World, name: &str, ceiling: Duration) -> Serving {
    let mut settings: ServeSettings = ServeSettings::new(PATIENCE);
    settings.note_ceiling = ceiling;
    return serve_on_a_thread_with_settings(world.context.clone(), name, settings);
}

/// `serving_with_ceiling` with the ceiling these controls hold a write past.
fn serving(world: &World, name: &str) -> Serving {
    return serving_with_ceiling(world, name, NOTE_CEILING);
}

/// What:
///  A control's hold of the landing's note write,
///  and the two ends of it.
/// Why:
///   The write is held on whichever thread makes it,
/// so the control learns the write has begun and lets it go when it is done,
/// and a write that was never reached fails the control instead of waiting for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const write = heldNoteWrite(); await write.reached; // the commit is answered meanwhile
/// write.letGo();
/// ```
struct HeldNoteWrite {
    /// What the thread that reached the write says,
    /// until the control takes it.
    reached: Option<tokio::sync::oneshot::Receiver<()>>,
    /// What lets the write go on.
    let_go: std::sync::mpsc::Sender<()>,
}

impl HeldNoteWrite {
    /// Arms the hold and gives its two ends.
    fn arm() -> Self {
        let (reached, reached_wait) = tokio::sync::oneshot::channel();
        let (let_go, held) = std::sync::mpsc::channel();
        hold_on_any_thread(HoldOnAnyThread { point: String::from(IN_THE_NEXT_SERVICE_NOTE), reached, let_go: held });
        return Self { reached: Some(reached_wait), let_go };
    }

    /// Waits,
    /// no longer than the longest a control waits,
    /// for the note's write to begin.
    fn wait_until_written(&mut self) {
        let reached: tokio::sync::oneshot::Receiver<()> = self.reached.take().expect("the hold was armed");
        let waited = crate::lazy_runtime::run_on_runtime(async move { return tokio::time::timeout(PATIENCE, reached).await; });
        assert!(matches!(waited, Ok(Ok(Ok(())))), "the note's write was not reached within {PATIENCE:?}");
    }

    /// Lets the held write go on,
    /// so the thread it holds ends and the service finishes its work.
    fn let_go(&self) {
        let _released = self.let_go.send(());
    }
}

/// The subjects on the branch after `base`,
/// oldest first.
fn subjects_since(world: &World, base: &str) -> String {
    return git_text(world.repo.as_path(), &["log", "--reverse", "--format=%s", format!("{base}..HEAD").as_str()]);
}

/// Waits,
/// no longer than the longest a control waits,
/// until a watch of `store` names `git_dir`,
/// which is what the service that starts next would look at.
fn wait_until_the_next_service_would_look(store: &Path, git_dir: &Path) {
    let began: Instant = Instant::now();
    loop {
        if ClaimWatch::restored(store, None).directories().iter().any(|named| { return named == git_dir; }) {
            return;
        }
        assert!(began.elapsed() < PATIENCE, "no note in {} names {}", store.display(), git_dir.display());
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// One client's answer,
/// with the words it printed.
fn answered(submitting: Submitting, what: &str) -> (Result<Submitted, ClientFailure>, String) {
    return match submitting.join() {
        Ok(answered) => answered,
        Err(_) => panic!("{what}: still waiting after {PATIENCE:?}"),
    };
}

/// The commit an answer of `Landed` carries,
/// or a failure naming the answer the client was given instead.
fn landed_commit(answer: Result<Submitted, ClientFailure>, notices: &str) -> String {
    return match answer {
        Ok(Submitted::Landed { commit, .. }) => commit,
        other => panic!("the commit did not land: {other:?} {notices}"),
    };
}

/// The store a service of `world` keeps the notes for the next service in.
fn store_of(world: &World) -> PathBuf {
    return world.context.journal.join(WATCH_DIRECTORY);
}

/// What:
///  Does what a commit command does with a commit the service landed,
///  once its post-commit gate has allowed the push.
/// Why:
///   The wake is the step that sends the work loop to look at the repository,
/// which is what makes up for a note the landing gave up on,
/// so a control of that promise has to take the step,
///  and the promotion is the step beside it in the same command (`commit/commit_command.rs`,
/// `post_commit`).
/// The gate itself is not run:
/// the controls here are of the note and not of the checks,
/// and this is the path a gate that allows takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await promoteThePush(handoff, landed); await handOver(handoff, landed, notices);
/// ```
fn after_the_gate(world: &World, serving: &Serving, landed: &str, parent: &str) {
    let handoff: ServiceHandoff = ServiceHandoff {
        config: client(serving, ANSWER_CEILING, PATIENCE),
        repository: world.repo.clone(),
        git_dir: world.repo.join(".git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        parent: String::from(parent),
    };
    promote_the_push(&handoff, landed).expect("the push the landing held is promoted");
    let mut printed: Vec<u8> = Vec::new();
    hand_over(&handoff, landed, &mut printed).expect("the service is woken for the commit's after-commit work");
}

/// What:
///  A note whose write does not return costs that note and never the commit that waits for it,
///  and the note is written anyway by the work loop's own look.
/// Why:
///   The landing writes the note between the commit landing and its client's answer,
/// so a journal directory that stopped answering would otherwise hold that answer and every commit behind it.
/// The store is the only thing that wait is for,
/// and the work loop writes the same note under a ceiling of its own when the answer's wake sends it to look,
/// which a watch of the store reads here while the held write has still not returned.
#[test]
fn the_commit_whose_note_the_store_has_not_answered_is_answered_within_the_ceiling() {
    let world: World = world("service-watch-note-held");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let serving: Serving = serving(&world, "note-held");
    let mut held: HeldNoteWrite = HeldNoteWrite::arm();
    let only: Submitting = submit_on_a_thread(client(&serving, ANSWER_CEILING, PATIENCE), request(&world, "only", "only.txt", "only\n"));
    held.wait_until_written();
    let write_began: Instant = Instant::now();
    let (answer, notices) = answered(only, "the client of the held note");
    let waited: Duration = write_began.elapsed();
    let landed: String = landed_commit(answer, notices.as_str());
    assert!(
        waited < NOTE_CEILING + BESIDE_THE_CEILING,
        "the commit was answered {waited:?} after its note's write began, and that write is granted {NOTE_CEILING:?}"
    );
    // The commit that landed stays landed, and is reported so.
    assert_eq!(subjects_since(&world, base.as_str()), "message of only", "the commit is on the branch");
    // What the note is for: the service that starts next looks at this repository, and the note that says so was written
    //  by the work loop's own look, because the landing's own write has still not returned.
    after_the_gate(&world, &serving, landed.as_str(), base.as_str());
    let store: PathBuf = store_of(&world);
    wait_until_the_next_service_would_look(store.as_path(), world.repo.join(".git").as_path());
    held.let_go();
    wait_until_settled(&world);
    remove(serving.directory.as_path());
    remove(world.root.as_path());
}

/// A commit submitted while another commit's note write is held is not queued behind it:
/// both are answered,
/// the second without waiting for a ceiling of its own,
/// and both stay on the branch in their order.
/// The store is handed no second write while the first has not ended,
/// so the note the first gave up on is written by the work loop's look instead,
/// which a watch of the store reads once that answer's wake has been taken.
#[test]
fn the_commit_behind_a_note_the_store_has_not_answered_is_not_queued_behind_it() {
    let world: World = world("service-watch-note-queued");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let serving: Serving = serving(&world, "note-queued");
    let mut held: HeldNoteWrite = HeldNoteWrite::arm();
    let first: Submitting = submit_on_a_thread(client(&serving, ANSWER_CEILING, PATIENCE), request(&world, "first", "first.txt", "first\n"));
    held.wait_until_written();
    let second_began: Instant = Instant::now();
    let second: Submitting = submit_on_a_thread(client(&serving, ANSWER_CEILING, PATIENCE), request(&world, "second", "second.txt", "second\n"));
    let (first_answer, first_notices) = answered(first, "the client of the first commit");
    let (second_answer, second_notices) = answered(second, "the client of the second commit");
    let second_waited: Duration = second_began.elapsed();
    let first_landed: String = landed_commit(first_answer, first_notices.as_str());
    assert!(matches!(second_answer, Ok(Submitted::Landed { .. })), "{second_answer:?} {second_notices}");
    assert!(
        second_waited < NOTE_CEILING + BESIDE_THE_CEILING,
        "the commit behind the held note was answered after {second_waited:?}, and that note's write is granted {NOTE_CEILING:?}"
    );
    assert_eq!(subjects_since(&world, base.as_str()), "message of first\nmessage of second", "both commits are on the branch, in their order");
    // What the note is for: the work loop's look writes the note the first landing gave up on, and the held write has
    //  still not returned.
    after_the_gate(&world, &serving, first_landed.as_str(), base.as_str());
    let store: PathBuf = store_of(&world);
    wait_until_the_next_service_would_look(store.as_path(), world.repo.join(".git").as_path());
    held.let_go();
    wait_until_settled(&world);
    remove(serving.directory.as_path());
    remove(world.root.as_path());
}

/// A commit whose note the store answers never waits for that ceiling,
/// so the bound costs a commit nothing where storage answers.
#[test]
fn a_commit_whose_note_the_store_answers_does_not_wait_for_that_ceiling() {
    let world: World = world("service-watch-note-answered");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let serving: Serving = serving_with_ceiling(&world, "note-answered", A_CEILING_NOBODY_WAITS_FOR);
    let began: Instant = Instant::now();
    let only: Submitting = submit_on_a_thread(client(&serving, ANSWER_CEILING, PATIENCE), request(&world, "only", "only.txt", "only\n"));
    let (answer, notices) = answered(only, "the client of the answered note");
    let waited: Duration = began.elapsed();
    assert!(matches!(answer, Ok(Submitted::Landed { .. })), "{answer:?} {notices}");
    assert!(
        waited < A_CEILING_NOBODY_WAITS_FOR / 4,
        "the commit took {waited:?} where its note answered at once, and the landing grants that write {A_CEILING_NOBODY_WAITS_FOR:?}"
    );
    assert_eq!(subjects_since(&world, base.as_str()), "message of only", "the commit is on the branch");
    let store: PathBuf = store_of(&world);
    wait_until_the_next_service_would_look(store.as_path(), world.repo.join(".git").as_path());
    wait_until_settled(&world);
    remove(serving.directory.as_path());
    remove(world.root.as_path());
}
