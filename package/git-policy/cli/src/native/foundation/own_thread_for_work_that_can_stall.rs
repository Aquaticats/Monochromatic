//! What:
//!  A kind of work that can stop on storage runs on a thread of its own,
//!  awaited under a ceiling,
//!  with a stated number of such threads alive at once for that kind.
//! Why:
//!  A call into storage cannot be taken back once it is made:
//!  a pool job whose caller stops waiting keeps the thread it runs on until its work ends
//! (`blocking_jobs_within_the_budget.rs`),
//! so work that repeatedly stalls can take threads a commit needs and hold them with no end
//! (the reviews of `0b847be249` and `e1a615aab0`).
//!  A thread of this kind is not a place of the process's bound,
//! so a commit's own path and a withdrawal's answer never stand behind one;
//! and each kind has its own small count,
//!  so a stuck read of a beat's note cannot take the look that routes a commit,
//! and no kind can take more threads than it states however long its work runs.
//! A place of a kind goes back when that thread's work ends,
//!  or when a thread the system refuses drops the closure it was given,
//! and never when the caller stops waiting:
//!  that is the whole point of the count.
//!  The caller is told which of the two happened:
//! the work ran,
//! no place came within the ceiling (nothing ran, nothing was
//!  changed),
//! or the work started and gave no answer within it (it may have done its work, and its thread stays alive until it ends).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = await runOnItsOwnThread('routingALook', ceiling, () => look(dir)); // ran | noPlace | noAnswer
//! ```
use crate::blocking_jobs_within_the_budget::TheAnswerOfAJobAwaited;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::Semaphore;

/// What a kind of work that can stop on storage is called,
/// and how many threads of it may be alive at once.
/// Why:
///  Each kind's count is its own,
/// so one kind's stuck threads never take another kind's,
/// and the count is the bound on how many storage calls of that kind can be alive at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorkThatCanStall = 'routingALook' | 'readingABeatNote';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkThatCanStall {
    /// The look that says whose a commit's request is.
    /// Two, because two commits can arrive at once and each needs its own look.
    RoutingALook,
    /// The journal read of a waiting client's beat.
    /// One, because a note is what a client reads between two beats and the beat behind it asks again.
    ReadingABeatNote,
}

impl WorkThatCanStall {
    /// How many threads of this kind may be alive at once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const aliveAtOnce = (kind) => kind === 'routingALook' ? 2 : 1;
    /// ```
    pub fn alive_at_once(self) -> usize {
        return match self {
            Self::RoutingALook => 2,
            Self::ReadingABeatNote => 1,
        };
    }

    /// The name a thread of this kind carries in a process listing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const threadName = (kind) => `cli-git-${kind}`;
    /// ```
    fn thread_name(self) -> &'static str {
        return match self {
            Self::RoutingALook => "cli-git-routing-look",
            Self::ReadingABeatNote => "cli-git-beat-note",
        };
    }

    /// The places of this kind,
    /// made by the first thread of it that needs one.
    fn places(self) -> &'static Arc<Semaphore> {
        static LOOKS: OnceLock<Arc<Semaphore>> = OnceLock::new();
        static NOTES: OnceLock<Arc<Semaphore>> = OnceLock::new();
        let kept: &'static OnceLock<Arc<Semaphore>> = match self {
            Self::RoutingALook => &LOOKS,
            Self::ReadingABeatNote => &NOTES,
        };
        return kept.get_or_init(|| { return Arc::new(Semaphore::new(self.alive_at_once())); });
    }
}

/// What:
///  The work of this kind that is alive now,
///  which is its count less the places it has given out.
/// Why:
///  A caller that asks again must not add a thread while the one before it is still alive,
///  and the only holders of these places are threads of this kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aliveNow = (kind) => kind.aliveAtOnce() - kind.places().availablePermits();
/// ```
pub fn alive_now(kind: WorkThatCanStall) -> usize {
    return kind.alive_at_once().saturating_sub(kind.places().available_permits());
}

/// What:
///  Whether every thread of this kind has ended.
/// Why:
///  A caller that retried while one of its own kind was still alive would add a thread beside it,
/// and a look that stopped on storage would fill the kind's count one retry at a time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noneAlive = (kind) => aliveNow(kind) === 0;
/// ```
pub fn none_alive(kind: WorkThatCanStall) -> bool {
    return alive_now(kind) == 0;
}

/// What:
///  Run `work` on a thread of its own of this kind and await it no longer than `ceiling`.
/// Why:
///  See the module description.
///  A place that no place came for within the ceiling proves nothing ran and nothing was changed;
/// a work that outlasted the ceiling after it started may have done its work,
///  and its thread stays alive, holding its place, until it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await runOnItsOwnThread(kind, ceiling, work);
/// ```
pub async fn run_on_its_own_thread<T, F>(kind: WorkThatCanStall, ceiling: Duration, work: F) -> TheAnswerOfAJobAwaited<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let place: tokio::sync::OwnedSemaphorePermit = match tokio::time::timeout(ceiling, Arc::clone(kind.places()).acquire_owned()).await {
        Ok(Ok(taken)) => taken,
        // The semaphore lives as long as the process, so a closed one is a defect rather than a state: nothing is run.
        Ok(Err(_closed)) => return TheAnswerOfAJobAwaited::NotRun,
        // Every thread of this kind is alive, so this work did not start and nothing it would have changed was changed.
        Err(_no_place) => return TheAnswerOfAJobAwaited::NotRun,
    };
    let (sender, answered): (tokio::sync::oneshot::Sender<T>, tokio::sync::oneshot::Receiver<T>) = tokio::sync::oneshot::channel::<T>();
    let started = std::thread::Builder::new().name(String::from(kind.thread_name())).spawn(move || {
        // The place is this thread's and goes back when its work ends, however long that takes.
        let _place: tokio::sync::OwnedSemaphorePermit = place;
        let answer: T = work();
        // A caller that stopped waiting is gone with its receiver, and the answer goes with the send.
        let _sent = sender.send(answer);
    });
    if started.is_err() {
        // The closure went with the refused thread, and its place with the closure: nothing ran.
        return TheAnswerOfAJobAwaited::NotRun;
    }
    return match tokio::time::timeout(ceiling, answered).await {
        Ok(Ok(value)) => TheAnswerOfAJobAwaited::Ran(value),
        // The thread went without an answer, which this work cannot do while it runs.
        Ok(Err(_dropped)) => TheAnswerOfAJobAwaited::NoAnswer,
        // The work outlasted the ceiling: it may have done its work, and its thread stays alive until it ends.
        Err(_outlasted) => TheAnswerOfAJobAwaited::NoAnswer,
    };
}
