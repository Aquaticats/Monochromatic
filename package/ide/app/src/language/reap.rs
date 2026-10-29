//! Waiting until every server process the worker thread started has been reaped.
//!
//! A process that has ended stays in the kernel's process table,
//!  as a "zombie",
//!  until its parent
//! collects its exit status ("reaps" it).
//!  helix-lsp owns the process handles and kills on drop.
//! tokio reaps the process of a dropped handle only while one of its runtimes keeps running,
//!  and
//! only after the kernel reported that a child ended.
//!  Two things follow for the worker:
//!
//! - Its own runtime must be dropped first.
//!    helix-lsp keeps a client alive inside the task that
//!   awaits `initialize`,
//!    so a server that never answered is killed only when that task is
//!   dropped,
//!    which happens when the runtime is dropped.
//! - Something must then keep reaping until no child is left.
//!    A killed process is not gone at
//!   once;
//!    under load it can take longer than any fixed pause.
//!
//! So after its runtime is gone,
//!  the worker thread runs a second,
//!  short-lived runtime here and
//! waits until the kernel lists no child of this thread any more.
//!  The kernel's list needs no
//! process handle,
//!  which is why it works although helix-lsp owns them all.

/// What:
///  `Duration` is a time span.
/// Why:
///  The wait and the pauses inside it are bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Duration = number; // milliseconds
/// ```
use std::time::Duration;
/// What:
///  `Instant` is a point on the clock;
///  `sleep` returns a future (a promise) that resolves
///       after a time span.
/// Why:
///  While the wait sleeps,
///  the runtime runs its drivers,
///  and reaping happens there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sleep = (ms: number) => new Promise(done => setTimeout(done, ms));
/// ```
use tokio::time::{Instant, sleep};

/// The kernel's list of processes that the reading thread started and has not reaped yet,
/// zombies included:
///  process numbers separated by spaces.
///  The worker thread starts language
/// servers and nothing else that outlives a call,
///  so this list is exactly its servers.
const CHILDREN_OF_THIS_THREAD: &str = "/proc/thread-self/children";

/// Pause between two looks at the list.
const POLL: Duration = Duration::from_millis(2);

/// What:
///  The numbers of this thread's unreaped child processes,
///  or nothing when the kernel does
///       not offer the list.
///  `Option<Vec<u32>>` is "a growable list of unsigned 32-bit numbers,
///       or nothing" (siblings of `u32`:
///  `i32`,
///  `u64`;
///  a process number is a positive 32-bit value).
/// Why:
///  An empty list is the one reliable sign that no server process is left to reap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unreaped(): number[] | undefined {
///   try { return readFileSync('/proc/thread-self/children', 'utf8').split(/\s+/).filter(Boolean).map(Number); }
///   catch { return undefined; }
/// }
/// ```
fn unreaped() -> Option<Vec<u32>> {
    // What: `read_to_string` returns `Result`; `match` separates the text from the failure.
    // Why: A kernel built without this list, or another operating system, has no such file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let text: string; try { text = readFileSync(path, 'utf8'); } catch (error) { log.debug(error); return undefined; }
    // ```
    let text = match std::fs::read_to_string(CHILDREN_OF_THIS_THREAD) {
        Ok(text) => text,
        Err(error) => {
            tracing::debug!(%error, "the kernel does not list this thread's child processes");
            // `None` is the "nothing" variant of `Option`.
            return None;
        }
    };
    // `Vec::new()` creates an empty list; `mut` allows pushing to it.
    let mut found: Vec<u32> = Vec::new();
    for word in text.split_whitespace() {
        // What: `parse::<u32>()` converts text to a number and returns `Result`; `if let Ok(...)`
        //       runs the block only for a number.
        // Why: The kernel writes only numbers; anything else would not name a process.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const number = Number(word); if (Number.isInteger(number)) found.push(number);
        // ```
        if let Ok(number) = word.parse::<u32>() {
            found.push(number);
        }
    }
    // `Some(...)` is the "value present" variant of `Option`.
    return Some(found);
}

/// What:
///  Wait until this thread has no unreaped child process,
///  at most `grace`.
///       `async fn` returns a future (a promise).
/// Why:
///  Each pause lets the runtime that awaits this reap;
///  an empty list ends the wait early,
///      which is at once when every server had ended by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function untilReaped(grace: number): Promise<void> {
///   const started = Date.now();
///   for (;;) {
///     const left = unreaped();
///     if (left === undefined) { await sleep(grace); return; }
///     if (left.length === 0) return;
///     if (Date.now() - started >= grace) { log.warn(left); return; }
///     await sleep(POLL);
///   }
/// }
/// ```
async fn until_reaped(grace: Duration) {
    let started = Instant::now();
    loop {
        // `let Some(x) = option else { ... }` binds the list or runs the block for "nothing".
        let Some(left) = unreaped() else {
            // Without the list there is no sign of completion; the whole allowance is used.
            sleep(grace).await;
            return;
        };
        if left.is_empty() {
            tracing::debug!(
                waited_ms = started.elapsed().as_millis(),
                "every language server process was reaped"
            );
            return;
        }
        if started.elapsed() >= grace {
            tracing::warn!(
                ?left,
                waited_ms = started.elapsed().as_millis(),
                "language server processes were not reaped in time; they stay in the process table until the application exits"
            );
            return;
        }
        sleep(POLL).await;
    }
}

/// What:
///  Reap the worker thread's server processes on a runtime of its own,
///  waiting at most
///       `grace` for the last of them.
/// Why:
///  Call this on the worker thread after its runtime was dropped:
///  dropping the runtime
///      drops every task and with it the last process handles,
///  and this runtime then reaps what
///      those drops killed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finish(grace: number): void { runToCompletion(untilReaped(grace)); }
/// ```
pub(super) fn finish(grace: Duration) {
    // What: A current-thread runtime drives every task on the thread that calls `block_on`;
    //       `enable_all` turns on its drivers, which are where tokio reaps. `build` returns `Result`.
    // Why: The worker's own runtime is gone by now; reaping needs a running one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // a second, short-lived event loop on the same thread
    // ```
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!(
                %error,
                "cannot start the runtime that reaps language server processes; ended servers stay in the process table until the application exits"
            );
            return;
        }
    };
    runtime.block_on(until_reaped(grace));
}
