//! Controls for the wait a landing gives the note that names its repository to the next service:
//! one write at a time for the whole process,
//! and the place of that one write given back when it ends.

use super::*;
use crate::service_claim_watch::{ClaimWatch, IN_THE_NEXT_SERVICE_NOTE, WATCH_DIRECTORY};
use crate::service_fault::hooks::{HoldOnAnyThread, hold_on_any_thread};
use crate::work_item::WORK_DIRECTORY;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The most a control waits for a note write that is not held to answer.
const A_NOTE_THAT_ANSWERS: Duration = Duration::from_secs(5);

/// The ceiling the held write of this control is granted:
/// short,
/// so the control ends in seconds.
const A_HELD_NOTES_CEILING: Duration = Duration::from_millis(400);

/// A scratch directory under the package's own build directory,
/// removed by the caller.
fn scratch(name: &str) -> PathBuf {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("claim-watch-tests").join(format!("{name}-{}", std::process::id()));
    let _cleared = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the scratch directory");
    return directory;
}

/// A Git directory with the work directory the wrapper makes.
fn git_dir_with_work(root: &Path, name: &str) -> PathBuf {
    let git_dir: PathBuf = root.join(name);
    std::fs::create_dir_all(git_dir.join(WORK_DIRECTORY)).expect("the work directory");
    return git_dir;
}

/// What:
///  A write that has not ended keeps the store from being handed another,
///  and the place of that one write is given back when it ends.
/// Why:
///   A journal on a mount that stopped answering would otherwise cost one thread for every commit that lands,
/// and the place is what holds that to one;
///   a store that answers is not slowed by it,
/// because the place is given back in microseconds.
/// The first write is held at the point the landing reaches,
/// so "while a write has not ended" is a state and not a duration.
#[test]
fn a_second_note_is_not_handed_over_while_the_first_has_not_ended() {
    let root: PathBuf = scratch("one-write-at-a-time");
    let git_dir: PathBuf = git_dir_with_work(root.as_path(), "repo.git");
    let store: PathBuf = root.join("journal").join(WATCH_DIRECTORY);
    let (reached, _reached_wait) = tokio::sync::oneshot::channel::<()>();
    let (let_go, held) = std::sync::mpsc::channel::<()>();
    hold_on_any_thread(HoldOnAnyThread { point: String::from(IN_THE_NEXT_SERVICE_NOTE), reached, let_go: held });
    // Held on the thread that writes it, this one gives no answer within its ceiling.
    let first: NoteForTheNextService = note_for_the_next_service_within(A_HELD_NOTES_CEILING, store.as_path(), None, git_dir.as_path());
    assert_eq!(first, NoteForTheNextService::NoAnswer, "a held write is left behind at its ceiling");
    let began: Instant = Instant::now();
    let second: NoteForTheNextService = note_for_the_next_service_within(A_HELD_NOTES_CEILING, store.as_path(), None, git_dir.as_path());
    let second_waited: Duration = began.elapsed();
    assert_eq!(second, NoteForTheNextService::AnEarlierWriteHasNotEnded, "the store is handed no second write");
    assert!(second_waited < A_HELD_NOTES_CEILING / 2, "a commit is told at once that the store is not asked, not after {second_waited:?}");
    // The write ends, and the place goes with it: the store is asked again, and answers.
    let _released = let_go.send(());
    let retried: Instant = Instant::now();
    loop {
        let again: NoteForTheNextService = note_for_the_next_service_within(A_NOTE_THAT_ANSWERS, store.as_path(), None, git_dir.as_path());
        if again == NoteForTheNextService::Answered {
            break;
        }
        assert_eq!(again, NoteForTheNextService::AnEarlierWriteHasNotEnded, "nothing but the place of the write may hold a note back");
        assert!(retried.elapsed() < A_NOTE_THAT_ANSWERS, "the place of the write was not given back when that write ended");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(ClaimWatch::restored(store.as_path(), None).directories(), vec![git_dir], "the note the store answered with is there");
    let _removed = std::fs::remove_dir_all(&root);
}
