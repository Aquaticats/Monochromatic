//! What:
//!  The removal of the markers a service made ahead, done by a thread the stop starts,
//!  and asked for once the work in flight has settled,
//!  with a bound on the waiting.
//! Why:
//!  The sweep is the only step between the line that says the service ends and the raise of the signal that ends it
//!  that can wait in the kernel:
//!  it takes the step a prestage job excludes itself from and calls the file system.
//!  A kernel or another thread that does not answer for the grace the last-resort alarm leaves
//!  would end the service through that alarm instead of its own shutdown,
//!  which the release gate met on 2026-11-15
//!  (`a_stop_during_a_hung_branch_write_ends_the_service_within_the_bound_and_keeps_the_commit_on_its_side_ref`).
//!  A thread started at the moment the sweep is asked for would put the start's own wait
//!  (a stack to map, which can wait in the kernel while the container reclaims its memory) inside that grace,
//!  so the thread is started when the stop is told,
//!  a settle bound earlier,
//!  and waits there on one message.
//!  A sweep that gives no answer within the bound is left behind:
//!  it ends with the process,
//!  and what it did not reach is the next acquisition's to remove,
//!  which the shutdown's last line says (`service_server_shutdown_after_a_stop.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sweep = startTheSweepAtTheStop(); // the thread waits for one ask
//! // … the work in flight settles …
//! if (sweep.ask(SWEEP_CEILING) !== 'completely') sayWhichRemovalWasLeft();
//! ```
//!
//! Limits:
//!  A marker the sweep does not reach stays in the Git directory under a staging name that no Git command reads,
//!  and the next acquisition of that index removes it (`SPEC.md`, "Removing the lock and what is made durable").

use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

/// The thread name the sweep carries in a process listing,
/// so a sweep a stop left behind is recognisable there.
const SWEEP_THREAD_NAME: &str = "cli-git-stop-sweep";

/// How the removal of the markers ended for the stop that asked for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Swept {
    /// The thread removed the markers the service had made ahead.
    Completely,
    /// The thread had not answered at the bound,
    /// so the markers it did not reach are left where they are.
    AtTheBound,
    /// No thread could be started for the sweep,
    /// so nothing was removed.
    NoThread,
}

/// What:
///  The thread a stop's sweep runs on,
///  and the two messages of its one hand-over.
/// Why:
///   The thread is started at the stop and asked at the end,
/// so nothing about starting it stands between the service's own line and its end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TheSweepOfMarkersAtTheStop { ask(ceiling): 'completely' | 'at the bound' | 'no thread' }
/// ```
pub struct TheSweepOfMarkersAtTheStop {
    /// The message that asks for the sweep.
    ask: Sender<()>,
    /// The message that says the sweep is done.
    done: Receiver<()>,
    /// Whether a thread was started:
    /// a system that refused one has nothing to ask,
    /// and no marker is removed.
    started: bool,
}

impl TheSweepOfMarkersAtTheStop {
    /// What:
    ///  Asks the thread to remove the markers made ahead,
    ///  and waits for its answer no longer than `ceiling`.
    /// Why:
    ///   The stop may not wait on the sweep for ever,
    /// and a sweep the system refused must be told apart from one that did not answer in time:
    /// the first removed nothing at all and the second may have removed some of it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// sweep.ask(SWEEP_CEILING)
    /// ```
    ///
    /// @param ceiling - the longest the stop waits for the answer
    ///
    /// @returns whether the sweep finished, gave no answer at the bound, or never ran
    ///
    /// @example
    /// ```ts
    /// sweep.ask(SWEEP_CEILING);
    /// ```
    pub fn ask(&self, ceiling: Duration) -> Swept {
        if !self.started {
            return Swept::NoThread;
        }
        if self.ask.send(()).is_err() {
            // The thread is gone before it was asked, which is the same answer as a system that refused it.
            return Swept::NoThread;
        }
        return match self.done.recv_timeout(ceiling) {
            Ok(()) => Swept::Completely,
            Err(_unanswered) => Swept::AtTheBound,
        };
    }
}

/// What:
///  Starts the thread that will remove the markers made ahead when the stop asks,
///  and gives back the two ends of that one hand-over.
/// Why:
///   The thread must exist before the grace begins,
/// and a system that refuses one is answered as such rather than made to fail the stop:
/// the markers are then left for the next acquisition, which the shutdown says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [ask, done] = startTheThread() // the body waits for ask, then removes, then answers
/// ```
///
/// @returns the thread's two ends, and whether there is a thread at all
///
/// @example
/// ```ts
/// const sweep = start_the_sweep_at_the_stop();
/// ```
pub fn start_the_sweep_at_the_stop() -> TheSweepOfMarkersAtTheStop {
    let (ask, asked): (Sender<()>, Receiver<()>) = channel();
    let (answered, done): (Sender<()>, Receiver<()>) = channel();
    let spawned = std::thread::Builder::new()
        .name(String::from(SWEEP_THREAD_NAME))
        .spawn(move || {
            // A service whose stop never asks for the sweep leaves this thread waiting here until the process ends,
            //  and nothing else waits for it.
            if asked.recv().is_err() {
                return;
            }
            crate::service_sweep_control_window::the_sweep_waits_where_a_control_says();
            crate::service_index_prestage::remove_the_file_made_ahead();
            // A caller that stopped waiting is gone with its receiver, and the answer goes with the send.
            let _answered = answered.send(());
        });
    return TheSweepOfMarkersAtTheStop { ask, done, started: spawned.is_ok() };
}

/// The two ends of the hand-over are a control in the test module.
#[cfg(test)]
#[path = "service_sweep_started_at_the_stop_tests.rs"]
mod tests;
