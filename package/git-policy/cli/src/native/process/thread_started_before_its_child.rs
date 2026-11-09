//! What:
//!  A thread of this wrapper,
//! named,
//! started before the child it exists to wait for,
//! and a refusal a control can make on demand.
//! Why:
//!  A child is reaped by the thread that waits for it,
//! so a start that happens after the child exists leaves a child nothing
//!  reaps when the thread cannot be started:
//! the child is killed and dropped,
//! and it stays a zombie until the process exits,
//!  which is the shortage of processes (`ulimit -u`,
//! the control group's `pids.max`) that refuses such a thread.
//! Starting the
//!  thread first costs the command its start instead,
//! and starts nothing that cannot be waited for.
//!  The refusal is a control's to make because no real process limit can be reached without starving
//!  the whole test process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const waiting = startThread('cli-git-query-wait', () => child.wait()); // throws before any child exists
//! ```

#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};

/// Whether the next start is refused,
/// as a control asks.
///  Nothing in the release executable reads it.
#[cfg(test)]
static REFUSE_ONE_START: AtomicBool = AtomicBool::new(false);

/// What:
///  Start a thread of this wrapper by name,
/// running `body`,
/// or say why no thread could be started.
/// Why:
///  Every waiting thread of the wrapper takes this one path,
/// so a refusal is reported the same way wherever it happens,
///  and a control can make it happen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startNamedThread(name: string, body: () => void): ThreadHandle;
/// ```
pub fn start_named_thread(
    name: &str,
    body: Box<dyn FnOnce() + Send>,
) -> std::io::Result<std::thread::JoinHandle<()>> {
    #[cfg(test)]
    if REFUSE_ONE_START.swap(false, Ordering::Acquire) {
        // The body is dropped here, so a control sees that nothing ran.
        return Err(std::io::Error::other("the process limit refused a thread"));
    }
    return std::thread::Builder::new().name(name.to_owned()).spawn(body);
}

/// What:
///  Make the next `start_named_thread` fail,
/// as a start under a process limit does.
/// Why:
///  A control has to reach the path that reports a refused thread,
/// and the resource that refuses one cannot be exhausted without
///  starving the test process itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseOneThreadStart(); // the next start throws
/// ```
#[cfg(test)]
pub fn refuse_one_start() {
    REFUSE_ONE_START.store(true, Ordering::Release);
}

/// Controls for a refused start stay out of the release executable.
#[cfg(test)]
#[path = "thread_started_before_its_child_tests.rs"]
mod tests;
