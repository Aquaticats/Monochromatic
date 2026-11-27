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
//! Each control runs its body in a child of its own:
//! the place of the one note write of a process (`A_NOTE_WRITE_IS_OUTSTANDING`) belongs to the whole process,
//! so a control that holds that write kept the place of every other control of a threaded run,
//! which never reached its own note (issue 6497).
//!
//! A window one of these controls times holds the wait for a note and the answer that follows it,
//! and never a commit's landing:
//! the release code grants a ceiling to the note's write (`LANDING_WATCH_NOTE_CEILING`)
//! and none to a landing,
//! so a wall-clock bound that covered a landing would fail on a host whose disk is busy
//! (measured 2026-11-26 in a threaded `landing::` run:
//! the nine landings that run logged took 140 ms to 22576 ms,
//! and the child that failed here answered its second commit 3893 ms after that commit was submitted,
//! of which 501 ms was the first commit's note ceiling and 3367 ms was that commit's own index update;
//! issue 6525).
//! What each bound rests on is written beside it.
//!
//! The service and its clients are the ones `service_land_hung_write_tests.rs` runs,
//! and the fixtures are the ones the landing's controls make.

use super::hung_write_tests::{
    PATIENCE,
    Serving,
    Submitting,
    client,
    serve_on_a_thread_with_settings,
    submit_on_a_thread,
    wait_until_settled,
};
use super::tests::{
    World,
    request,
    world,
};
use super::*;
use crate::service_claim_watch::{
    ClaimWatch,
    WATCH_DIRECTORY,
    next_service_note_point,
};
use crate::service_client::Submitted;
use crate::service_fault::hooks::{
    HoldOnAnyThread,
    hold_on_any_thread,
};
use crate::service_handoff::{
    ServiceHandoff,
    hand_over,
    promote_the_push,
};
use crate::service_messages::ClientFailure;
use crate::service_server::ServeSettings;
use crate::test_support::{
    git_text,
    remove,
};
use std::time::{
    Duration,
    Instant,
};

/// The ceiling the landings of these controls grant a note's write:
/// short,
/// so a control ends in seconds,
/// and its value plays no part in what a commit is answered.
const NOTE_CEILING: Duration = Duration::from_millis(500);

/// The ceiling the landing of a run whose store answers is granted in these controls:
/// long enough that no landing of this host reaches it,
/// so a commit that waited for it would still be waiting when a control's patience ran out.
/// A landing of this host costs seconds
/// (measured 2026-11-26 in a threaded `landing::` run:
/// the nine landings it logged took 140 ms to 22576 ms),
/// and the release code grants no landing any ceiling.
const A_CEILING_NOBODY_WAITS_FOR: Duration = Duration::from_secs(600);

/// The most a commit may wait beside the ceiling for its own note:
/// what the rest of the answer's path costs is a send on a channel,
/// so anything past this is a wait that is not the ceiling's.
/// The window it bounds starts once that commit has landed,
/// because a landing is granted no ceiling of its own (see the module words).
const BESIDE_THE_CEILING: Duration = Duration::from_millis(2500);

/// A service serving on a thread of its own,
/// whose landings grant `ceiling` for a note.
fn serving_with_ceiling(
    world: &World,
    name: &str,
    ceiling: Duration,
) -> Serving {
    let mut settings: ServeSettings = ServeSettings::new(PATIENCE);
    settings.note_ceiling = ceiling;
    return serve_on_a_thread_with_settings(world.context.clone(), name, settings);
}

/// `serving_with_ceiling` with the ceiling these controls hold a write past.
fn serving(
    world: &World,
    name: &str,
) -> Serving {
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
    /// Arms the hold on the note the landing writes about `git_dir` and gives its two ends.
    fn arm(git_dir: &std::path::Path) -> Self {
        let (reached, reached_wait) = tokio::sync::oneshot::channel();
        let (let_go, held) = std::sync::mpsc::channel();
        hold_on_any_thread(HoldOnAnyThread {
            point: next_service_note_point(git_dir),
            reached,
            let_go: held,
        });
        return Self {
            reached: Some(reached_wait),
            let_go,
        };
    }

    /// Waits,
    /// no longer than the longest a control waits,
    /// for the note's write to begin.
    fn wait_until_written(&mut self) {
        let reached: tokio::sync::oneshot::Receiver<()> =
            self.reached.take().expect("the hold was armed");
        let waited = crate::lazy_runtime::run_on_runtime(async move {
            return tokio::time::timeout(PATIENCE, reached).await;
        });
        assert!(
            matches!(waited, Ok(Ok(Ok(())))),
            "the note's write was not reached within {PATIENCE:?}"
        );
    }

    /// Lets the held write go on,
    /// so the thread it holds ends and the service finishes its work.
    fn let_go(&self) {
        let _released = self.let_go.send(());
    }
}

/// The subjects on the branch after `base`,
/// oldest first.
fn subjects_since(
    world: &World,
    base: &str,
) -> String {
    return git_text(
        world.repo.as_path(),
        &[
            "log",
            "--reverse",
            "--format=%s",
            format!("{base}..HEAD").as_str(),
        ],
    );
}

/// Whether a watch of `store` names `git_dir`,
/// which is what the service that starts next would look at.
fn the_store_names(
    store: &Path,
    git_dir: &Path,
) -> bool {
    return ClaimWatch::restored(store, None)
        .directories()
        .iter()
        .any(|named| {
            return named == git_dir;
        });
}

/// Waits,
/// no longer than the longest a control waits,
/// until a watch of `store` names `git_dir`,
/// which is what the service that starts next would look at.
fn wait_until_the_next_service_would_look(
    store: &Path,
    git_dir: &Path,
) {
    let began: Instant = Instant::now();
    loop {
        if the_store_names(store, git_dir) {
            return;
        }
        assert!(
            began.elapsed() < PATIENCE,
            "no note in {} names {}",
            store.display(),
            git_dir.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// One client's answer,
/// with the words it printed.
fn answered(
    submitting: Submitting,
    what: &str,
) -> (Result<Submitted, ClientFailure>, String) {
    return match submitting.join() {
        Ok(answered) => answered,
        Err(_) => panic!("{what}: still waiting after {PATIENCE:?}"),
    };
}

/// The commit an answer of `Landed` carries,
/// or a failure naming the answer the client was given instead.
fn landed_commit(
    answer: Result<Submitted, ClientFailure>,
    notices: &str,
) -> String {
    return match answer {
        Ok(Submitted::Landed {
            commit,
            ..
        }) => commit,
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
fn after_the_gate(
    world: &World,
    serving: &Serving,
    landed: &str,
    parent: &str,
) {
    let handoff: ServiceHandoff = ServiceHandoff {
        // The wake and the promotion answer within the longest a control waits, and never within a shorter ceiling:
        //  the wait for either holds the service's own work in the repository, and no ceiling of the release code
        //  covers that work.
        config: client(serving, PATIENCE, PATIENCE),
        repository: world.repo.clone(),
        git_dir: world.repo.join(".git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        parent: String::from(parent),
    };
    promote_the_push(&handoff, landed).expect("the push the landing held is promoted");
    let mut printed: Vec<u8> = Vec::new();
    hand_over(&handoff, landed, &mut printed)
        .expect("the service is woken for the commit's after-commit work");
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
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "the_commit_whose_note_the_store_has_not_answered_is_answered_within_the_ceiling",
        || {
            let world: World = world("service-watch-note-held");
            let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
            let serving: Serving = serving(&world, "note-held");
            let mut held: HeldNoteWrite = HeldNoteWrite::arm(world.repo.join(".git").as_path());
            let only: Submitting = submit_on_a_thread(
                // The client's wait holds this commit's landing, and the release code grants a landing no ceiling, so
                //  only the longest a control waits bounds it; the window the control times starts once that landing
                //  has ended.
                client(&serving, PATIENCE, PATIENCE),
                request(&world, "only", "only.txt", "only\n"),
            );
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
            assert_eq!(
                subjects_since(&world, base.as_str()),
                "message of only",
                "the commit is on the branch"
            );
            // What the note is for: the service that starts next looks at this repository, and the note that says so was written
            //  by the work loop's own look, because the landing's own write has still not returned.
            after_the_gate(&world, &serving, landed.as_str(), base.as_str());
            let store: PathBuf = store_of(&world);
            wait_until_the_next_service_would_look(
                store.as_path(),
                world.repo.join(".git").as_path(),
            );
            held.let_go();
            wait_until_settled(&world);
            remove(world.root.as_path());
        },
    );
}

/// A commit submitted while another commit's note write is held is not queued behind it:
/// both are answered while the store has answered nothing,
/// the commit whose note is held is answered inside that write's ceiling,
/// and both stay on the branch in their order.
/// The store is handed no second write while the first has not ended,
/// so the note the first gave up on is written by the work loop's look instead,
/// which a watch of the store reads once that answer's wake has been taken.
#[test]
fn the_commit_behind_a_note_the_store_has_not_answered_is_not_queued_behind_it() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "the_commit_behind_a_note_the_store_has_not_answered_is_not_queued_behind_it",
        || {
            let world: World = world("service-watch-note-queued");
            let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
            let serving: Serving = serving(&world, "note-queued");
            let mut held: HeldNoteWrite = HeldNoteWrite::arm(world.repo.join(".git").as_path());
            // The clients here give the service's answer the longest a control waits, and never a shorter ceiling:
            //  a landing is granted no ceiling of its own, so a landing that takes seconds is a wait this control must
            //  not read as a failure, and only a hang ends one of these waits.
            let first: Submitting = submit_on_a_thread(
                client(&serving, PATIENCE, PATIENCE),
                request(&world, "first", "first.txt", "first\n"),
            );
            held.wait_until_written();
            // The second commit is prepared here and submitted after the moment below is taken, so the window the first
            //  answer is timed in holds that commit's note wait and the answer that follows it, and nothing this
            //  control's own thread does.
            let second_request: CommitRequest = request(&world, "second", "second.txt", "second\n");
            let note_began: Instant = Instant::now();
            let second: Submitting =
                submit_on_a_thread(client(&serving, PATIENCE, PATIENCE), second_request);
            let (first_answer, first_notices) = answered(first, "the client of the first commit");
            let first_waited: Duration = note_began.elapsed();
            let first_landed: String = landed_commit(first_answer, first_notices.as_str());
            assert!(
                first_waited < NOTE_CEILING + BESIDE_THE_CEILING,
                "the commit whose note's write began was answered {first_waited:?} after that write began, and the write is granted {NOTE_CEILING:?}"
            );
            let (second_answer, second_notices) =
                answered(second, "the client of the second commit");
            assert!(
                matches!(second_answer, Ok(Submitted::Landed { .. })),
                "{second_answer:?} {second_notices}"
            );
            // What:
            //  The store names no repository once both commits are answered,
            //  so neither answer came from it.
            // Why:
            //   The second commit is answered while the store has answered nothing,
            //  which is the whole of "not queued behind it":
            //  an answer that had waited for the store would have come only after the held write is let go, below.
            //  No wall-clock bound stands in for this,
            //  because the window it would time holds the second commit's own landing,
            //  which the release code grants no ceiling and this host does in seconds (see the module words).
            assert!(
                !the_store_names(
                    store_of(&world).as_path(),
                    world.repo.join(".git").as_path()
                ),
                "the store already names {}, so an answer here may have come from it",
                world.repo.join(".git").display()
            );
            assert_eq!(
                subjects_since(&world, base.as_str()),
                "message of first\nmessage of second",
                "both commits are on the branch, in their order"
            );
            // What the note is for: the work loop's look writes the note the first landing gave up on, and the held write has
            //  still not returned.
            after_the_gate(&world, &serving, first_landed.as_str(), base.as_str());
            let store: PathBuf = store_of(&world);
            wait_until_the_next_service_would_look(
                store.as_path(),
                world.repo.join(".git").as_path(),
            );
            held.let_go();
            wait_until_settled(&world);
            remove(world.root.as_path());
        },
    );
}

/// A commit whose note the store answers never waits for that ceiling:
/// it is answered within the longest a control waits,
/// while that write is granted ten times that,
/// so the bound costs a commit nothing where storage answers.
#[test]
fn a_commit_whose_note_the_store_answers_does_not_wait_for_that_ceiling() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_commit_whose_note_the_store_answers_does_not_wait_for_that_ceiling",
        || {
            let world: World = world("service-watch-note-answered");
            let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
            let serving: Serving =
                serving_with_ceiling(&world, "note-answered", A_CEILING_NOBODY_WAITS_FOR);
            // What:
            //  The commit is answered, and the note the landing wrote names its repository,
            //  while that write was granted `A_CEILING_NOBODY_WAITS_FOR`.
            // Why:
            //   That ceiling is what a commit which waited for the store would have spent,
            //  and it is ten times the longest a control waits,
            //  so an answer that arrives is one the ceiling did not hold.
            //  No other bound is stated here:
            //  the window one would time holds this commit's own landing,
            //  which the release code grants no ceiling and this host does in seconds (see the module words).
            let only: Submitting = submit_on_a_thread(
                client(&serving, PATIENCE, PATIENCE),
                request(&world, "only", "only.txt", "only\n"),
            );
            let (answer, notices) = answered(only, "the client of the answered note");
            assert!(
                matches!(answer, Ok(Submitted::Landed { .. })),
                "{answer:?} {notices}"
            );
            assert_eq!(
                subjects_since(&world, base.as_str()),
                "message of only",
                "the commit is on the branch"
            );
            // No wake is taken here, so the landing's own write is the only writer of this store: its note appearing is
            //  that write having answered.
            let store: PathBuf = store_of(&world);
            wait_until_the_next_service_would_look(
                store.as_path(),
                world.repo.join(".git").as_path(),
            );
            wait_until_settled(&world);
            remove(world.root.as_path());
        },
    );
}
