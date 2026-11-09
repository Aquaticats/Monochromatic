//! What:
//!  The thread that gives a second-pass command its request,
//! how far it got,
//! and the two bounds it runs under.
//! Why:
//!  The write of the bytes and the count an answer reads are two steps,
//! and a command may answer before it has read all of its input,
//! so the writer needs a bound of its own and a way to tell the answer that its
//!  last count is published.
//! It sits beside the query that starts it,
//! because the query's own file passes the line limit with it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const writer = feedStandardInput(pipe, bytes, windowMs); // { finished: Promise<void>; cutShort(): string | undefined }
//! ```

use std::io::Write;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;
use std::process::ChildStdin;
use std::time::Duration;

/// How long the writer of a second-pass command's input waits while the command takes none of it.
const FEEDING_PAUSE: Duration = std::time::Duration::from_millis(10);

/// How long an answer waits for the writer to publish its last count once the command has ended.
/// Why:
///   The write of the last bytes and the count the answer reads are two steps,
/// so a writer descheduled between them
///  would make a request that arrived whole look cut.
/// The writer keeps one `FEEDING_PAUSE` (10 ms) between two attempts,
/// so a turn of
///  that pause is all a writer with its bytes out needs;
/// fifty of them absorb a loaded host's scheduling of one thread and stay far
///  under the query window the answer was given (30 s by default).
/// A writer still held after that is the blocked case `cut_short` names:
///  its pipe is full with bytes unwritten,
/// and the answer is the prefix count.
/// The coordinator's pick,
/// open to the owner's veto.
pub(crate) const WRITER_SETTLE_CEILING: Duration = Duration::from_millis(500);

/// What:
///  Write `bytes` to the standard input of a started command from a thread of its own.
/// Why:
///  The command may answer before it has read all of its input,
/// so writing here could block on a full pipe while nothing reads the answer;
/// the thread's write fails once the command is stopped and its pipe closes.
///
/// # Errors
///
/// A thread that cannot be started.
pub(crate) fn feed_standard_input(pipe: Option<ChildStdin>, bytes: Vec<u8>, window: Duration, hold: Option<Box<dyn Fn() + Send + 'static>>) -> Result<InputWriter, String> {
    let progress: InputProgress = InputProgress::default();
    let reported: InputProgress = progress.clone();
    reported.started(bytes.len());
    let (finished, completion) = tokio::sync::oneshot::channel::<()>();
    let written: Result<std::thread::JoinHandle<()>, std::io::Error> = match pipe {
        // No input was asked of the command, so there is nothing to write and its standard input is not piped.
        None => crate::scoped_start::spawn_named("cli-git-second-pass-input", move || {
            let _finished = finished.send(());
        }),
        Some(open) => crate::scoped_start::spawn_named("cli-git-second-pass-input", move || {
            write_within_the_window(open, bytes.as_slice(), window, reported, hold);
            // The count is published by now, so a reader that waits for this reads the writer's own finished result.
            let _finished = finished.send(());
        }),
    };
    return match written {
        Ok(handle) => Ok(InputWriter { handle, progress, completion }),
        Err(error) => Err(crate::scoped_start::refused_thread_sentence("give the command its input", &error)),
    };
}
/// What:
///  The thread that gives a command its input,
/// and what it has to report about it.
/// Why:
///   A request that could not be written whole leaves a command reading a prefix,
/// so the query must learn that from the
///  writer and not pass the answer on as one made from the whole request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InputWriter = { finished: Promise<void>; cutShort(): string | undefined };
/// ```
pub(crate) struct InputWriter {
    /// The thread that writes the input,
    /// and closes the command's end when it ends.
    /// A query waits for its child and lets this thread end by itself;
    /// a control joins it to see that it did.
    #[cfg_attr(not(test), expect(dead_code, reason = "only a control joins the writer; a query waits for its child"))]
    handle: std::thread::JoinHandle<()>,
    /// What the thread has written of the request,
    /// and why it stopped short when it did.
    progress: InputProgress,
    /// Resolves once the writer has published its last count.
    completion: tokio::sync::oneshot::Receiver<()>,
}

/// What a writer has written of its request,
/// and why it stopped short when it did,
/// shared with the thread that writes it.
/// A cell behind a mutex is enough:
/// one thread writes it and one reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Progress = { written: number; total: number; failure?: string };
/// ```
#[derive(Clone, Default)]
pub(crate) struct InputProgress(std::sync::Arc<std::sync::Mutex<Progress>>);

/// How far a writer got.
#[derive(Clone, Default)]
struct Progress {
    /// The bytes handed to the kernel so far.
    written: usize,
    /// The bytes the request holds.
    total: usize,
    /// Why the writer stopped before the end,
    /// once it knows of it.
    failure: Option<String>,
}

impl InputProgress {
    /// The request this writer was given,
    /// so a reader can count from the start.
    fn started(&self, total: usize) {
        if let Ok(mut slot) = self.0.lock() {
            slot.total = total;
        }
    }

    /// Records the bytes handed to the kernel so far.
    fn advanced(&self, written: usize) {
        if let Ok(mut slot) = self.0.lock() {
            slot.written = written;
        }
    }

    /// Records why the request could not be written whole.
    fn record(&self, reason: String) {
        if let Ok(mut slot) = self.0.lock() {
            slot.failure = Some(reason);
        }
    }

    /// What the request could not be written whole for,
    /// or nothing when every byte reached the command.
    /// Why:
    ///   A command that ended while bytes remain cannot have read them,
    /// so the answer was made from a prefix of the
    ///  request even where the writer had not yet noticed the broken pipe.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// progress.failure ?? (progress.written < progress.total ? `after ${progress.written} of ${progress.total} bytes` : undefined)
    /// ```
    fn cut_short(&self) -> Option<String> {
        let Ok(slot) = self.0.lock() else {
            return Some(String::from("how much of its request was written could not be read"));
        };
        if let Some(reason) = slot.failure.as_ref() {
            return Some(reason.clone());
        }
        if slot.written >= slot.total {
            return None;
        }
        return Some(format!(
            "it stopped taking its input after {} of {} bytes, so {} of them were not written",
            slot.written,
            slot.total,
            slot.total.saturating_sub(slot.written)
        ));
    }
}

impl InputWriter {
    /// Waits for the writer to end.
    #[cfg(test)]
    pub(crate) fn join(self) -> std::thread::Result<()> {
        return self.handle.join();
    }

    /// What:
    ///  Waits for the writer to publish its last count,
    /// no longer than `ceiling`,
    /// and answers whether it did.
    /// Why:
    ///   A writer descheduled between its last write and that count has the request out already,
    /// and the answer must
    ///  not read the count that is still to come.
    /// A writer still held at the ceiling is the blocked case the count names.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await Promise.race([writer.finished, delay(ceiling)]).then(() => writer.finishedRan)
    /// ```
    pub(crate) async fn settled(&mut self, ceiling: Duration) -> bool {
        return tokio::time::timeout(ceiling, &mut self.completion).await.is_ok();
    }

    /// Why the request could not be written whole,
    /// or nothing when every byte reached the command.
    pub(crate) fn cut_short(&self) -> Option<String> {
        return self.progress.cut_short();
    }
}


/// What:
///  Writes `bytes` into `open` while the command keeps taking them,
/// and gives up when it takes none for `window`.
/// Why:
///   The command may answer before it has read all of its input,
/// so a blocking write can wait on a full pipe for as long as
///  the command holds its end open,
/// and a command the kernel does not release would keep that thread for good.
/// The pipe's
///  writing end is made non-blocking,
/// so the thread sees a full pipe instead of waiting in it,
/// and it holds the
///  request only until nothing has been taken for the window the query itself is judged by;
/// a request of ours that a
///  stopped command never read has nothing left to say.
///  The writing end is closed when this returns,
/// so the command reads the end of its input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// setNonBlocking(fd); while (written < bytes.length) { const n = tryWrite(fd, bytes.subarray(written)); if (n > 0) { written += n; last = Date.now() } else if (Date.now() - last > windowMs) return; else await sleep(POLL) }
/// ```
fn write_within_the_window(mut open: ChildStdin, bytes: &[u8], window: Duration, progress: InputProgress, hold: Option<Box<dyn Fn() + Send + 'static>>) {
    // The writing end is the only place that waits: making it non-blocking turns a full pipe into a report of it, and the
    //  command's own end is a different description of the same pipe.
    let descriptor: std::os::fd::RawFd = open.as_fd().as_raw_fd();
    // SAFETY: `fcntl` reads and writes an integer of the descriptor's own flags and touches no memory of this process.
    let flags: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        crate::diagnostic_log::debug("monochromatic-git-policy-cli", "the input of a second-pass command could not be written without blocking");
        progress.record(String::from("its input could not be written without blocking"));
        let _closed = open.flush();
        return;
    }
    let mut written: usize = 0;
    let mut last_taken: std::time::Instant = std::time::Instant::now();
    while written < bytes.len() {
        match open.write(&bytes[written..]) {
            // A write of nothing with bytes left is a request the command will not take; the count says how far it got.
            Ok(0) => break,
            Ok(taken) => {
                written += taken;
                // A control holds the writer here, between its write and the count that write stands for.
                if let Some(step) = hold.as_ref() {
                    step();
                }
                progress.advanced(written);
                last_taken = std::time::Instant::now();
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if last_taken.elapsed() >= window {
                    let reason: String = format!(
                        "it took none of its input for {} seconds, so {} of {} bytes were not written",
                        window.as_secs(),
                        bytes.len().saturating_sub(written),
                        bytes.len()
                    );
                    crate::diagnostic_log::debug("monochromatic-git-policy-cli", format!("a second-pass command {reason}").as_str());
                    progress.record(reason);
                    break;
                }
                std::thread::sleep(FEEDING_PAUSE);
            }
            // A command that closed its end before every byte was written answered from a prefix of the request: the loop runs
            //  only while bytes remain, so nothing here is "what it wanted".
            Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => {
                let reason: String = format!(
                    "it closed its input after {} of {} bytes, so {} of them were not written",
                    written,
                    bytes.len(),
                    bytes.len().saturating_sub(written)
                );
                crate::diagnostic_log::debug("monochromatic-git-policy-cli", format!("a second-pass command {reason}").as_str());
                progress.record(reason);
                break;
            }
            Err(error) => {
                let reason: String = format!("its input could not be written: {error}");
                crate::diagnostic_log::debug("monochromatic-git-policy-cli", format!("a second-pass command {reason}").as_str());
                progress.record(reason);
                break;
            }
        }
    }
    // Closing our end tells the command that its input has ended.
    let _closed = open.flush();
}
