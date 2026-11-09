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

use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

/// The thread name the jobs of this module carry in a process listing,
/// so a stuck read is recognisable.
const WORK_THREAD_NAME: &str = "cli-git-bounded-work";

/// What:
///  Run `work` on a thread of its own and wait for its value no longer than `ceiling`.
/// Why:
///  The caller stops waiting at the ceiling whatever the job does,
/// so a call the kernel never returns cannot hold a command;
///  a thread the system refuses is "no answer" too,
/// because nothing ran.
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
    let (sender, receiver): (std::sync::mpsc::Sender<Value>, Receiver<Value>) = channel::<Value>();
    let started = std::thread::Builder::new().name(String::from(WORK_THREAD_NAME)).spawn(move || {
        let value: Value = work();
        // A caller that stopped waiting is gone with its receiver, and the value goes with the send.
        let _sent = sender.send(value);
    });
    if started.is_err() {
        crate::diagnostic_log::debug("workWithin", "the system refused a thread for the job, so it did not run");
        return None;
    }
    return match receiver.recv_timeout(ceiling) {
        Ok(value) => Some(value),
        Err(RecvTimeoutError::Timeout) => {
            crate::diagnostic_log::debug("workWithin", format!("the job gave no answer within {ceiling:?} and is left behind").as_str());
            None
        }
        Err(RecvTimeoutError::Disconnected) => None,
    };
}

/// The bounded job is controlled in the test module.
#[cfg(test)]
#[path = "work_awaited_within_a_ceiling_tests.rs"]
mod tests;
