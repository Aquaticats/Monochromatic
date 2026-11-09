//! What:
//!  A child waited for by the thread that owns it,
//! with a ceiling,
//! so a query that never answers is ended by its owner.
//! Why:
//!  A number is a safe address only while its process is this process's own unreaped child.
//! A caller that
//!  stopped waiting at its ceiling and signalled the number later could reach a successor of the child,
//! because the
//!  thread that waits for the child reaps it as soon as it ends
//! (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
//!  the numeric-signal findings of the second and third reviews).
//! Here the thread that waits is the thread that kills:
//! while it waits it has not
//!  reaped the child,
//! so the number still names it,
//! and nothing else can have it.
//! A child that cannot be ended
//!  therefore cannot exist on this path,
//! and no descriptor and no pin is needed to end one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const status = await withDeadline(child.exited, ceiling) ?? (child.kill(), await child.exited, throw ceilingReached());
//! ```

use std::io::Read as _;
use std::process::{Child, ExitStatus, Output};
use std::time::{Duration, Instant};

/// How often a waiting thread looks at its child.
///  A child that ends is noticed within this,
/// and the wait costs one look per interval.
pub const LOOK_INTERVAL: Duration = Duration::from_millis(10);

/// How long the read of a query's stream may go on after the query itself has ended.
///  The same bound the runtime reader gives a step's pipes
/// (`src/native/process/step_output_read_whole_through_pipes.rs`):
/// a program the query started that still holds a stream one second after the query ended is holding it,
///  not writing the query's output.
/// Labelled unmeasured there,
/// and unmeasured here for the same reason.
const DRAIN_AFTER_EXIT: Duration = Duration::from_secs(1);

/// What:
///  The sentence for a query that did not answer within its ceiling and was ended.
/// Why:
///  The person has to know the query is what was stopped,
/// the limit it met,
/// and that nothing of the repository was changed by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${program} did not answer within ${seconds} seconds ... and was ended ...`
/// ```
pub fn ceiling_reached_text(program: &str, ceiling: Duration) -> String {
    return format!(
        "{program} did not answer within {} seconds, the longest a read-only query of this wrapper waits, and was ended. Such a query \
         changes nothing of the repository. Hint: check with `ps` what that program waits for, look at the disk it reads, and run \
         the command again.",
        ceiling.as_secs()
    );
}

/// What:
///  Waits for `child` to end,
/// but no longer than `ceiling`;
/// past it the child is killed and reaped,
/// and `TimedOut` is returned.
/// Why:
///   The caller owns the child and has not reaped it,
/// so the kill reaches the child and never a number that was reused
///  after it was reaped;
/// the reap here means no zombie is left behind either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitWithin(child: Child, ceiling: Duration): ExitStatus;
/// ```
pub fn wait_within(child: &mut Child, ceiling: Duration) -> std::io::Result<ExitStatus> {
    let began: Instant = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        let left: Duration = ceiling.saturating_sub(began.elapsed());
        if left.is_zero() {
            end_and_reap(child);
            return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "the child did not end within its ceiling"));
        }
        std::thread::sleep(left.min(LOOK_INTERVAL));
    }
}

/// What:
///  `wait_within` for a child whose two streams are pipes the caller wants every byte of.
/// Why:
///  A child that fills a pipe before it exits would wait for a reader for good,
/// so both streams are read on threads of
///  their own while the child is waited for.
///  A standard output that a program the query started still holds open one drain after the
/// query ended is not the answer,
/// so the read fails and says so;
/// the standard error of such a query comes
///  back as what was read of it,
/// with a line naming the program,
/// because the answer itself was read whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [out, err, status] = await withDeadline(readAll(child.stdout), readAll(child.stderr), child.exited);
/// ```
pub fn output_within(mut child: Child, ceiling: Duration) -> std::io::Result<Output> {
    let (answers, answered) = std::sync::mpsc::channel::<OneStream>();
    let (complaints, complained) = std::sync::mpsc::channel::<OneStream>();
    let mut readers: Vec<std::thread::JoinHandle<()>> = Vec::new();
    if let Some(pipe) = child.stdout.take() {
        readers.push(start_reader(Box::new(pipe), "standard output", answers, &mut child)?);
    }
    if let Some(pipe) = child.stderr.take() {
        readers.push(start_reader(Box::new(pipe), "standard error", complaints, &mut child)?);
    }
    let status: ExitStatus = wait_within(&mut child, ceiling)?;
    // The streams end when the child's write ends close; a program the child started can hold one past that, so each read is
    //  taken with a bound of its own and what it read is judged by where it stopped.
    let stdout: Vec<u8> = match answered.recv_timeout(DRAIN_AFTER_EXIT) {
        Ok(ended) => match ended.failed {
            Some(error) => return Err(error),
            None => ended.bytes,
        },
        Err(_cut) => return Err(std::io::Error::other(cut_short_text("standard output"))),
    };
    let stderr: Vec<u8> = match complained.recv_timeout(DRAIN_AFTER_EXIT) {
        Ok(ended) => match ended.failed {
            Some(error) => return Err(error),
            None => ended.bytes,
        },
        Err(_cut) => {
            crate::diagnostic_log::warn("queryOutput", cut_short_text("standard error").as_str());
            Vec::new()
        }
    };
    // A reader whose stream no program holds open has sent what it read and ended. One whose stream a program the query
    //  started still holds is still blocked in its read, and joining it would wait for that program's life: nothing joins the
    //  readers, and each ends when its stream does.
    drop(readers);
    return Ok(Output { status, stdout, stderr });
}

/// What:
///  Starts the thread that reads one stream of the query to its end,
/// or ends and reaps the child and says why none could start.
/// Why:
///   A pipe a child fills before it exits needs a reader of its own,
/// and a refusal must cost the query its run rather than
///  leave a child nothing reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reader = startReader(pipe, 'standard output', send);
/// ```
fn start_reader(
    pipe: Box<dyn std::io::Read + Send>,
    name: &'static str,
    reporting: std::sync::mpsc::Sender<OneStream>,
    child: &mut Child,
) -> std::io::Result<std::thread::JoinHandle<()>> {
    let started = std::thread::Builder::new().name(format!("cli-git-query-{name}")).spawn(move || {
        let mut pipe = pipe;
        let mut bytes: Vec<u8> = Vec::new();
        let failed: Option<std::io::Error> = pipe.read_to_end(&mut bytes).err();
        // Nobody reads this when the wait has gone; the bytes it read are lost with it.
        let _abandoned = reporting.send(OneStream { bytes, failed });
    });
    return match started {
        Ok(reader) => Ok(reader),
        Err(error) => {
            // Nothing reads this child's output now, and the caller is told rather than the child being left to run.
            end_and_reap(child);
            Err(std::io::Error::other(format!("no thread could read the {name} of the query: {error}")))
        }
    };
}

/// One stream of a query,
/// read to its end or to the drain that ended it.
struct OneStream {
    /// The bytes read before the stream ended.
    bytes: Vec<u8>,
    /// Why the read stopped, when it did not stop at the end of the stream.
    failed: Option<std::io::Error>,
}

/// What:
///  The sentence for a query whose stream a program it started still holds open after it ended.
/// Why:
///  The person has to know that what was read may be incomplete,
///  which stream it was,
/// and which program to look for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the query ended, but a program it started still holds its ${stream} open ...`
/// ```
fn cut_short_text(stream: &str) -> String {
    return format!(
        "the query ended, but a program it started still holds its {stream} open, so what this command read may be incomplete: \
         the read ended {} seconds after the query did. Check with `ps` what that program is, and run the command again.",
        DRAIN_AFTER_EXIT.as_secs()
    );
}

/// Ends a child this value owns and has not reaped,
/// and reaps it,
/// so nothing is left running and no zombie is left behind.
fn end_and_reap(child: &mut Child) {
    // A kill of a child that has already exited fails harmlessly; the wait reaps either way.
    let _ = child.kill();
    let _ = child.wait();
}

/// Controls for a child its own thread waits for stay out of the release executable.
#[cfg(test)]
#[path = "child_waited_for_within_a_ceiling_tests.rs"]
mod tests;
