//! What:
//! The one place this program writes its own words:
//! the report a command prints about what it did on standard output,
//! and the lines it says about its own work on standard error
//! (the wait a long answer takes,
//! the warning about a lock older than the tool that runs,
//! the lines naming what a run left in a folder,
//! the cleanup summary after an mpm command,
//! and the sentence that ends a command that failed).
//! Why:
//! A person reading a terminal or a log needs those words,
//! and a stream that refuses the bytes must not decide what the command did or what it answers:
//! a report is written between two of a command's own steps
//! (`install <tool>` prints what it added to the plan and then runs the sync),
//! and a failure's sentence is the command's own account of a status it has already decided,
//! so a refused write must leave both alone.
//! `println!` and `eprintln!` do not answer that need:
//! they panic on every write error but a closed descriptor,
//! and this crate's release build sets `panic = "abort"`,
//! so a full disk or a pipe whose reader has gone turns one line into `SIGABRT`
//! (`0.2.7-alpha.20` did exactly that for the wait line,
//! and `0.2.7-alpha.21` still did it for every report;
//! `doc/second-pass-reads-of-the-alpha-releases-what-each-confirmed.md` holds the findings).
//! Every word this crate says therefore leaves through this module,
//! and `every_word_this_crate_says_leaves_through_this_module` is the control of that.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // A stderr write and a stdout write that ignore a broken stream.
//! export function say(line: string): void {
//!   try { process.stderr.write(`${line}\n`); } catch { /* courtesy line */ }
//! }
//! export function reportLine(line: string): void {
//!   try { process.stdout.write(`${line}\n`); } catch { /* the work is done either way */ }
//! }
//! ```

/// Imports what this module uses from `std::io`:
/// `Write`,
/// the trait that gives a stream its `write` method,
/// and `ErrorKind`,
/// the enum that says why a write was refused.
/// Why:
/// The writer below is one loop over `Write::write`,
/// so a stream that answers "not now" can be told from one that refuses for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No import: `process.stderr.write` is a method on the global object.
/// ```
use std::io::{
    ErrorKind,
    Write,
};
/// Imports `Duration` and `Instant` from `std::time`,
/// the two types that bound how long a refused write is retried.
/// Why:
/// A non-blocking stream that is full answers `EAGAIN` ("not now"),
/// which is a refusal a wait cures,
/// so the writer has to know how long it may wait before it gives up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now() arithmetic; TS has no retry loop in the standard library.
/// ```
use std::time::{
    Duration,
    Instant,
};

/// What:
/// The longest this writer waits for a non-blocking stream to take the bytes it refused.
/// Why:
/// A stream left non-blocking by another process answers `EAGAIN` while it is full,
/// and that answer means "not now",
/// not "never":
/// dropping the notice there loses a warning a retry would have delivered
/// (`0.2.7-alpha.21` did that,
/// and the second-pass read of it named the loss).
/// The wait is bounded because the reader may be a process that stopped reading:
/// an unbounded wait would hold a command that had finished its work.
/// 1 s is far above the time a reader needs to make room for the bytes of one line and far below
/// the wait a blocking stream already imposes,
/// measured on 2026-11-28T08:32:28-05:00 in twenty interleaved rounds of two arms,
/// one write of a
/// 400-byte line each,
/// on this machine:
/// a pipe filled to its capacity,
/// whose writer waits for a reader that drains whole pages,
/// took 0.029 ms at the median and 0.082 ms at its slowest,
/// and the control arm,
/// the same write with the room already there,
/// took 0.024 ms at the median and
/// 0.037 ms at its slowest,
/// so this bound is about twelve thousand times the slowest wait measured and about forty times the
/// control arm's own spread (its slowest sample was 1.6 times its median);
/// a reader that stops altogether costs the command this 1 s and no more.
/// `a_write_a_full_non_blocking_stream_refuses_is_made_when_the_reader_drains` holds the wait and
/// `a_write_no_reader_takes_gives_up_within_the_bound` holds its end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_WAIT_FOR_A_STREAM_TO_TAKE_THE_BYTES = 1000;
/// ```
pub(crate) const A_WAIT_FOR_A_STREAM_TO_TAKE_THE_BYTES: Duration = Duration::from_millis(1000);

/// What:
/// The longest pause between two attempts of that wait.
/// Why:
/// The pause doubles from 1 ms so a stream that frees a byte at a time costs few system calls,
/// and the cap keeps the last attempt from being most of the bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_LONGEST_PAUSE_BETWEEN_TWO_ATTEMPTS = 20;
/// ```
const THE_LONGEST_PAUSE_BETWEEN_TWO_ATTEMPTS: Duration = Duration::from_millis(20);

/// What:
/// `pub fn say(line: &str)` writes `line` and one newline to standard error
/// and drops whatever that write answered.
/// Why:
/// Every notice this program says about its own work goes through this function,
/// so the rule "a stream that refuses the line does not change the command" is stated once,
/// and a caller cannot accidentally say a notice with a write that aborts the process.
/// The answer is dropped rather than reported because there is nowhere left to report it:
/// the only channel it would go to is the one that just refused the bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function say(line: string): void
/// ```
pub fn say(line: &str) {
    // What: `std::io::stderr()` borrows the process's standard error stream
    //  (the sibling `std::io::stdout()` is the other one).
    // Why: the notices belong on standard error, beside a command's failures,
    //  while the command's report stays on standard output for whatever parses it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const standardError = process.stderr;
    // ```
    let mut standard_error = std::io::stderr();
    // What: hands the line to the function below, which is where the text and the write live.
    // Why: a control gives that function a stream of its own, so a plant in the text or the write is held
    //  by a control and not only by reading.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // sayOn(standardError, line);
    // ```
    say_on(&mut standard_error, line);
}

/// What:
/// `pub(crate) fn say_on(stream: &mut dyn Write, line: &str)` writes `line` and one newline into `stream`
/// and drops whatever that write answered.
/// Why:
/// This is where a notice's text and its one write live,
/// so a control can hand it a pipe another process filled or a stand-in that answers a chosen error,
/// which the process's own standard error cannot be made to do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sayOn(stream: { write(b: Uint8Array): number }, line: string): void
/// ```
pub(crate) fn say_on(
    stream: &mut dyn Write,
    line: &str,
) {
    // What: the line and its line end as one text, so one write carries both.
    // Why: two writes let a stream take the line and refuse the newline, which joins the next output
    //  to this line; one write cannot be split that way on a stream that answers atomically.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const words = `${line}\n`;
    // ```
    let words: String = format!("{line}\n");
    // What: hands both to the one writer, which drops a refusal.
    // Why: the wait for a non-blocking stream and the handling of every error kind live in that one place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeTheWords(stream, words);
    // ```
    write_the_words(stream, words.as_bytes());
}

/// What:
/// `pub fn report(text: &str)` writes `text` to standard output exactly as given
/// (its own line ends included)
/// and drops whatever that write answered.
/// Why:
/// A command's report is the toolchain lock's text,
/// the lock check's own words or the IDE launcher's
/// JSON document,
/// all of which already carry their line ends;
/// writing the text as it is keeps one line's bytes in one write
/// (`say` is for a single line),
/// and a stream that refuses it must not end the command,
/// which for `install <tool>` would stop it between the plan write and the sync.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function report(text: string): void
/// ```
pub fn report(text: &str) {
    // What: `std::io::stdout()` borrows the process's standard output stream.
    // Why: a report is the command's result and belongs where a caller reads pipes and files from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const standardOutput = process.stdout;
    // ```
    let mut standard_output = std::io::stdout();
    // What: the same one writer standard error uses, with the same dropping rule.
    // Why: no report's write may end a command between two of its own steps.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeTheWords(standardOutput, text);
    // ```
    write_the_words(&mut standard_output, text.as_bytes());
}

/// What:
/// `pub fn report_line(line: &str)` writes `line` and one newline to standard output
/// and drops whatever that write answered.
/// Why:
/// A command that prints one sentence of its result has one line to say,
/// and the line end belongs with the line in one write for the reason `say` gives;
/// the refusal is dropped for the reason `report` gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function reportLine(line: string): void
/// ```
pub fn report_line(line: &str) {
    // What: `std::io::stdout()` borrows the process's standard output stream.
    // Why: one sentence of a command's result goes where its other lines go.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const standardOutput = process.stdout;
    // ```
    let mut standard_output = std::io::stdout();
    // What: hands the line to the function below, which is where the text and the write live.
    // Why: a control gives that function a stream of its own, as it does for a notice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // reportLineOn(standardOutput, line);
    // ```
    report_line_on(&mut standard_output, line);
}

/// What:
/// `pub(crate) fn report_line_on(stream: &mut dyn Write, line: &str)` writes `line` and one newline into
/// `stream`
/// and drops whatever that write answered.
/// Why:
/// This is where the line end is added to the line in one text,
/// which is what keeps a stream from taking
/// the line and refusing its end under a control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportLineOn(stream: { write(b: Uint8Array): number }, line: string): void
/// ```
pub(crate) fn report_line_on(
    stream: &mut dyn Write,
    line: &str,
) {
    // What: the line and its line end as one text.
    // Why: `report` writes text as given, so the newline is added here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const words = `${line}\n`;
    // ```
    let words: String = format!("{line}\n");
    // What: the one writer, with the same dropping rule.
    // Why: the same reason `say` gives: the stream cannot change what the command did.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeTheWords(stream, words);
    // ```
    write_the_words(stream, words.as_bytes());
}

/// What:
/// `pub(crate) fn write_the_words(stream: &mut dyn Write, words: &[u8])` writes every byte of `words`
/// into `stream`,
/// waits out a stream that answers "not now" for at most `A_WAIT_FOR_A_STREAM_TO_TAKE_THE_BYTES`,
/// and drops the rest whatever else the stream answers.
/// Why:
/// One loop covers the three answers a stream gives:
/// a short write (take the rest next time),
/// `EINTR` (the write was not made,
/// so make it again),
/// and `EAGAIN` on a stream another process left non-blocking (wait for room),
/// while every other error is a stream that refuses for good,
/// which is dropped.
/// `&mut dyn Write` rather than a concrete stream because the controls pass a stand-in that answers
/// a chosen error,
/// and the two real streams are `Stderr` and `Stdout`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeTheWords(stream: { write(b: Uint8Array): number }, words: Uint8Array): void
/// ```
pub(crate) fn write_the_words(
    stream: &mut dyn Write,
    words: &[u8],
) {
    // What: how many of the bytes the stream has taken so far.
    // Why: a short write takes a prefix, and the next attempt carries the rest.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let written = 0;
    // ```
    let mut written: usize = 0;
    // What: when the first "not now" came, or nothing while the stream has not refused that way.
    // Why: the bound is counted from the first refusal, so a stream that takes bytes slowly but never
    //  refuses them is never cut off.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let refusedAt: number | undefined = undefined;
    // ```
    let mut refused_at: Option<Instant> = None;
    // What: how long the next attempt waits.
    // Why: it doubles after each refusal so a stream that stays full costs few system calls.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let pause = 1;
    // ```
    let mut pause: Duration = Duration::from_millis(1);
    while written < words.len() {
        // What: one attempt, which takes a prefix of the bytes left and answers how many.
        // Why: the same call serves every stream, and its answer is what the arms below read.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const answer = stream.write(words.subarray(written));
        // ```
        let attempt = stream.write(&words[written..]);
        match attempt {
            // What: the stream took the rest, so the words are all out.
            // Why: this is the one arm that finishes the loop.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (answer === words.length - written) return;
            // ```
            Ok(count) if count > 0 => {
                written += count;
            }
            // What: a stream that took nothing and answered nothing more to try.
            // Why: no error came with it, so there is nothing to classify; the bytes are dropped rather
            //  than retried for ever.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (answer === 0) return;
            // ```
            Ok(_) => return,
            // What: the write was not made because a signal arrived first.
            // Why: the stream said nothing about the bytes, so the attempt is made again as it is.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (isInterrupted) continue;
            // ```
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            // What: a non-blocking stream with no room, which the next attempt may find room in.
            // Why: this is the one refusal a wait cures, so it is waited out until the bound passes.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (isWouldBlock) { sleep(pause); pause = Math.min(pause * 2, 20); if (Date.now() - refusedAt >= BOUND) return; continue; }
            // ```
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                // What: the instant of this stream's first refusal, kept from the first time this arm runs.
                // Why: the bound is one wait per call, however many refusals it takes.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // refusedAt ??= Date.now();
                // ```
                let started: Instant = *refused_at.get_or_insert_with(Instant::now);
                // What: whether the wait has used its bound.
                // Why: a reader that never reads must not hold a command that has finished its work.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (Date.now() - started >= BOUND) return;
                // ```
                if started.elapsed() >= A_WAIT_FOR_A_STREAM_TO_TAKE_THE_BYTES {
                    return;
                }
                // What: one sleep before the next attempt.
                // Why: an immediate retry of a full pipe only burns system calls.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // sleep(pause);
                // ```
                std::thread::sleep(pause);
                // What: the doubled pause, up to the cap.
                // Why: the first refusals of a reader that drains soon are still answered within a
                //  couple of milliseconds.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // pause = Math.min(pause * 2, THE_LONGEST_PAUSE);
                // ```
                pause = (pause * 2).min(THE_LONGEST_PAUSE_BETWEEN_TWO_ATTEMPTS);
            }
            // What: any other refusal, such as a full disk or a reader that is gone.
            // Why: the stream refuses for good, and there is nowhere else to send the bytes, so the
            //  command goes on without them.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return;
            // ```
            Err(_) => return,
        }
    }
}

/// The unit tests of the writer and of the rule that every word leaves through it.
#[cfg(test)]
#[path = "notices_tests.rs"]
mod tests;
