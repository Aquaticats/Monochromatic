//! What:
//!  The note a landing writes for the service that starts next,
//!  written on a thread of its own and waited for no longer than its ceiling.
//! Why:
//!   The note itself,
//! what it holds and how it is published,
//! belongs to `service_claim_watch.rs`;
//! this file is only the wait the thread that lands commits may give it.
//! A landing writes the note between a commit landing and the answer its client waits for,
//! and the write is a directory creation,
//! an exclusive file and a rename inside the instance's journal:
//! any of them waits in the kernel with no end on a filesystem that stopped answering,
//! and a write that never ends there would hold that commit's answer and every commit queued behind it.
//! The commit has landed by then,
//! with its push held and its batch recorded,
//! so the wait is for the note alone,
//! and a store that has not answered within `ceiling`
//! (`wait_ceilings.rs`,
//! `LANDING_WATCH_NOTE_CEILING`)
//! costs the note and never the commit:
//! the thread is left behind and ends when the kernel lets the call go,
//! and nothing here reads whether it wrote.
//!   What the note is for happens without it:
//! the work loop writes the same note,
//! for the same directory and under `AFTER_COMMIT_STATE_CEILING`,
//! when the wake of this commit's answer sends it to look at the repository
//! (`service_server_work_loop_awaiting.rs`,
//! `remember_awaiting`),
//! and a landing writes it again once no earlier write is outstanding,
//! which a store that answered late reaches by itself.
//! A service that starts looks at every note already in the journal.
//! What a landing that gave up on the note loses is the service's own look for this repository:
//! the repository is then found by its next commit,
//! or by the work loop when this answer's wake arrives,
//! and nothing is left to be found if this process is killed before that look.
//!   The publish is the same exclusive one the work loop uses,
//! so a note written twice is one note,
//! and a store that answers late writes nothing another writer has not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const theNote = noteForTheNextServiceWithin(ceiling, store, instance, gitDir);
//! ```

use crate::service_claim_watch::note_for_the_next_service;
use crate::work_awaited_within_a_ceiling::work_within;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// What became of a landing's note for the service that starts next.
/// Why:
///  The three answers are three different things for the person reading the service's log and for what the
/// landing does next,
///  so they are not one boolean.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoteForTheNextService {
    /// The store answered:
    /// the note is written,
    /// or its write was refused and the refusal is said.
    Answered,
    /// A note write an earlier landing handed over has not ended,
    /// so this store was not asked.
    AnEarlierWriteHasNotEnded,
    /// The store gave no answer within the ceiling,
    /// and the write is left behind on a thread of its own.
    NoAnswer,
}

/// Whether a note write of this process has been handed over and has not ended.
/// Why:
///   One journal store belongs to one service,
/// so one write at a time is all a store that stopped answering may be given;
/// a place taken per landing would leave one thread per commit on a mount that never returns.
static A_NOTE_WRITE_IS_OUTSTANDING: AtomicBool = AtomicBool::new(false);

/// The place of the one note write,
/// given back when the work ends or when the system refused the thread that would have run it.
struct TheOneNoteWrite;

impl Drop for TheOneNoteWrite {
    fn drop(&mut self) {
        A_NOTE_WRITE_IS_OUTSTANDING.store(false, Ordering::SeqCst);
    }
}

/// What:
///  `note_for_the_next_service` written on a thread of its own and waited for no longer than `ceiling`,
///  giving what became of it.
/// Why:
///   See the module words:
/// what this wait is for,
/// what one write at a time keeps a stopped store from costing,
/// and what makes up for a note that is given up on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theNote = noteForTheNextServiceWithin(ceiling, store, instance, gitDir);
/// ```
pub fn note_for_the_next_service_within(ceiling: Duration, store: &Path, instance: Option<&str>, git_dir: &Path) -> NoteForTheNextService {
    // The place of the one write is taken here and given back when that work ends, whatever the store does.
    if A_NOTE_WRITE_IS_OUTSTANDING.swap(true, Ordering::SeqCst) {
        return NoteForTheNextService::AnEarlierWriteHasNotEnded;
    }
    let asked: (PathBuf, Option<String>, PathBuf) = (store.to_path_buf(), instance.map(str::to_string), git_dir.to_path_buf());
    let answered: Option<()> = work_within(ceiling, move || {
        // The place is this job's from here on: it is given back when this body ends,
        // and when the system refused the thread, the closure that captured it is dropped instead.
        let _place: TheOneNoteWrite = TheOneNoteWrite;
        let (owned_store, owned_instance, owned_git_dir): (PathBuf, Option<String>, PathBuf) = asked;
        note_for_the_next_service(owned_store.as_path(), owned_instance.as_deref(), owned_git_dir.as_path());
    });
    return match answered {
        Some(()) => NoteForTheNextService::Answered,
        None => NoteForTheNextService::NoAnswer,
    };
}

/// Controls for the wait a landing gives the note's write stay out of the release executable.
#[cfg(test)]
#[path = "service_claim_watch_note_within_a_ceiling_tests.rs"]
mod tests;
