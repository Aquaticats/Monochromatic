//! What:
//!  A blocking job run on a thread of its own,
//! awaited for no longer than a ceiling.
//! Why:
//!  A file system call can wait in the kernel with no end:
//! a directory listing,
//! a `statfs`,
//! or the read of a
//!  shim file on a mount whose server stopped answering.
//! The owner's rule is "nothing should ever wait forever or indefinitely",
//! so a caller
//!  that can say less without the answer waits only to the ceiling and goes on.
//!  A timer around a future that blocks inside such a call cannot end it,
//! because the timer is polled by the same task that
//!  is stuck;
//! only work on another thread can be left behind,
//! which is what `landing/scan_awaited_under_a_ceiling.rs` does
//!  for the process look.
//!  The job is abandoned,
//! never cancelled:
//! it ends when the kernel lets the call go,
//! and the thread it holds is gone only then.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = await Promise.race([new Promise((resolve) => resolve(work())), sleep(ceiling).then(() => undefined)]);
//! ```

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

/// The thread name the jobs of this module carry in a process listing,
/// so a stuck read is recognisable.
const WORK_THREAD_NAME: &str = "cli-git-bounded-work";

/// What:
///  What a job handed to a thread of its own answered,
///  when the caller may not wait for it without an end.
/// Why:
///   Three things can happen and two of them are not the job's answer:
/// the job ran and gave its value,
/// or the system refused the thread so that nothing ran and nothing the job would have changed was changed,
/// or the job started and gave no answer within the ceiling and is left behind with whatever it holds.
/// A caller that holds something of its own until the work ends must tell the refusal from the wait
/// (`landing/service_claim_watch_note_within_a_ceiling.rs`,
/// the one note write),
/// because a refusal releases what it holds at once and a wait does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AnswerOfAThreadOfWork<T> = { ran: T } | 'noThreadToStart' | 'noAnswer';
/// ```
pub enum TheAnswerOfAThreadOfWork<T> {
    /// The job ran to its end and gave this.
    Ran(T),
    /// The system refused a thread,
    /// so the job never ran and nothing it would have changed was changed.
    NoThreadToStart,
    /// The job started and gave no answer within the ceiling,
    /// or it ended without a value because it panicked:
    /// either way what it did is not known here,
    /// and it is left behind to end when its work does.
    NoAnswer,
}

/// What:
///  Run `work` on a thread of its own and wait for its value no longer than `ceiling`,
///  giving which of the three things happened.
/// Why:
///  The caller stops waiting at the ceiling whatever the job does,
/// so a call the kernel never returns cannot hold a command;
/// the refusal is told apart from the wait
/// so a caller that holds something until the work ends can give it back on the path where nothing ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = workWithinReporting(ceiling, () => listDirectory(directory));
/// ```
pub fn work_within_reporting<Work, Value>(ceiling: Duration, work: Work) -> TheAnswerOfAThreadOfWork<Value>
where
    Work: FnOnce() -> Value + Send + 'static,
    Value: Send + 'static,
{
    #[cfg(test)]
    if A_CONTROL_REFUSES_THREADS.load(Ordering::SeqCst) {
        // The stand-in of a system that refused the thread: the job does not run, and the closure goes with it.
        return TheAnswerOfAThreadOfWork::NoThreadToStart;
    }
    let (sender, receiver): (std::sync::mpsc::Sender<Value>, Receiver<Value>) = channel::<Value>();
    let started = std::thread::Builder::new().name(String::from(WORK_THREAD_NAME)).spawn(move || {
        let value: Value = work();
        // A caller that stopped waiting is gone with its receiver, and the value goes with the send.
        let _sent = sender.send(value);
    });
    if started.is_err() {
        crate::diagnostic_log::debug("workWithin", "the system refused a thread for the job, so it did not run");
        return TheAnswerOfAThreadOfWork::NoThreadToStart;
    }
    return match receiver.recv_timeout(ceiling) {
        Ok(value) => TheAnswerOfAThreadOfWork::Ran(value),
        Err(RecvTimeoutError::Timeout) => {
            crate::diagnostic_log::debug("workWithin", format!("the job gave no answer within {ceiling:?} and is left behind").as_str());
            TheAnswerOfAThreadOfWork::NoAnswer
        }
        // The job ended without sending: it panicked, and what it did before that is not known here.
        Err(RecvTimeoutError::Disconnected) => TheAnswerOfAThreadOfWork::NoAnswer,
    };
}

/// What:
///  Run `work` on a thread of its own and wait for its value no longer than `ceiling`,
///  with a value for its answer and nothing for either a refusal or a wait.
/// Why:
///  The caller stops waiting at the ceiling whatever the job does,
/// so a call the kernel never returns cannot hold a command;
///  a thread the system refuses is "no answer" too,
/// because nothing ran.
/// That is all a caller whose answer is optional data needs;
/// a caller that holds something until the work ends takes `work_within_reporting`, which tells the two apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = workWithin(ceiling, () => listDirectory(directory));
/// ```
pub fn work_within<Work, Value>(ceiling: Duration, work: Work) -> Option<Value>
where
    Work: FnOnce() -> Value + Send + 'static,
    Value: Send + 'static,
{
    return match work_within_reporting(ceiling, work) {
        TheAnswerOfAThreadOfWork::Ran(value) => Some(value),
        TheAnswerOfAThreadOfWork::NoThreadToStart | TheAnswerOfAThreadOfWork::NoAnswer => None,
    };
}

/// Whether a control stands in for a system that refuses every thread,
/// so `work_within_reporting` answers `NoThreadToStart` and its job never runs.
/// Why:
///  A control cannot exhaust this process's threads reliably,
/// and a process limit (`TasksMax`, `RLIMIT_NPROC`) refuses other threads too,
///  which no control of one step should reach;
/// the flag is the seam,
/// and nextest gives every control its own process,
/// so it cannot reach another.
#[cfg(test)]
static A_CONTROL_REFUSES_THREADS: AtomicBool = AtomicBool::new(false);

/// What:
///  Makes every thread of `work_within_reporting` refused until the returned value is dropped.
/// Why:
///  The seam of the control above;
/// the flag is cleared however the control ends,
/// a panic included.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = aControlRefusesEveryThread(); // every thread refused until refused.release()
/// ```
#[cfg(test)]
pub fn a_control_refuses_every_thread() -> ThreadsRefusedForTheControl {
    A_CONTROL_REFUSES_THREADS.store(true, Ordering::SeqCst);
    return ThreadsRefusedForTheControl;
}

/// The stand-in of `a_control_refuses_every_thread`,
/// cleared when it is dropped.
#[cfg(test)]
pub struct ThreadsRefusedForTheControl;

/// Clears the control's stand-in.
#[cfg(test)]
impl Drop for ThreadsRefusedForTheControl {
    fn drop(&mut self) {
        A_CONTROL_REFUSES_THREADS.store(false, Ordering::SeqCst);
    }
}

/// What:
///  Give `text` to `stream` on a thread of its own and wait for it no longer than `ceiling`.
/// Why:
///  A write to a descriptor whose reader has stopped draining waits in the kernel with no end,
/// and a line a person
///  may read is never worth a command that never returns:
/// the job is left behind at the ceiling,
/// and the caller says
///  what that means for its own line.
///  `Some(Ok(()))` is a line delivered,
/// `Some(Err(..))` one the stream refused,
/// and `None` one still waiting at the ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const written = writeWithin(WARNING_WRITE_CEILING, standardErrorDescriptor(), line);
/// ```
pub fn write_within<Stream: Write + Send + 'static>(ceiling: Duration, stream: Stream, text: String) -> Option<std::io::Result<()>> {
    return work_within(ceiling, move || {
        let mut target = stream;
        // The stream is flushed only after the line was written, as the callers of this helper always did.
        return match target.write_all(text.as_bytes()) {
            Ok(()) => target.flush(),
            Err(error) => Err(error),
        };
    });
}

/// The bounded job is controlled in the test module.
#[cfg(test)]
#[path = "work_awaited_within_a_ceiling_tests.rs"]
mod tests;
