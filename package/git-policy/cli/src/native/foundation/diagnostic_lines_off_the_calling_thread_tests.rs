//! Controls for the diagnostic lines on a thread of their own:
//! a stream that never drains holds no caller for longer than one bound,
//! a stream that drains gets the lines in the order they were said,
//! the queue is bounded and says how many lines it dropped,
//! and the end of a process waits for what it handed over and no longer.

use super::*;

/// The two times a control reads:
/// when a bound has passed,
/// and how long it waited.
use std::time::{Duration, Instant};
/// Reading what a stream already holds,
/// which is how a control sees a line without waiting for a thread.
use std::io::Read;

/// What:
///  The capacity of one pipe this process makes,
///  as the kernel reports it.
/// Why:
///   The control fills the stream before it hands it over,
/// and how much fills it belongs to the kernel rather than to a number in this file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capacity = fcntlGetPipeSize(pipe);
/// ```
#[cfg(target_os = "linux")]
fn one_pipe_s_holding() -> usize {
    let (reader, writer) = std::io::pipe().expect("a pipe");
    // SAFETY: `fcntl` takes a descriptor this process owns and an integer command, and writes no memory here.
    let capacity: std::ffi::c_int = unsafe { libc::fcntl(std::os::fd::AsRawFd::as_raw_fd(&reader), libc::F_GETPIPE_SZ) };
    drop(writer);
    drop(reader);
    assert!(capacity > 0, "the kernel did not report what one pipe holds: {capacity}");
    return usize::try_from(capacity).expect("a positive capacity");
}

/// What:
///  A pipe whose every byte of room is already taken,
///  and the read end a control drains when it wants the words.
/// Why:
///   The stream is full before the thread is given it,
/// so the thread's first write is the one that waits,
///  which is the state a caller must not be held by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = filledPipe();
/// ```
#[cfg(target_os = "linux")]
fn a_pipe_that_is_full() -> (std::io::PipeReader, std::io::PipeWriter) {
    let (reader, writer) = std::io::pipe().expect("a pipe");
    let capacity: usize = one_pipe_s_holding();
    let mut filler: std::io::PipeWriter = writer.try_clone().expect("a second write end");
    filler.write_all(vec![b'.'; capacity].as_slice()).expect("the stream fills");
    return (reader, writer);
}

/// What:
///  Makes a read end answer instead of waiting,
///  so a control reads what a stream holds now and never waits on it.
/// Why:
///   A control that read a stream nobody writes any more would wait for the end of the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fcntl(reader, F_SETFL, fcntl(reader, F_GETFL) | O_NONBLOCK)
/// ```
#[cfg(target_os = "linux")]
fn read_instead_of_waiting(reader: &std::io::PipeReader) {
    // SAFETY: `fcntl` takes a descriptor this process owns and an integer command, and writes no memory here.
    let flags: std::ffi::c_int = unsafe { libc::fcntl(std::os::fd::AsRawFd::as_raw_fd(reader), libc::F_GETFL) };
    assert!(flags >= 0, "the read end's flags are readable");
    // SAFETY: the same descriptor, with the flags it already had plus the one that makes a read return instead of waiting.
    let set: std::ffi::c_int = unsafe { libc::fcntl(std::os::fd::AsRawFd::as_raw_fd(reader), libc::F_SETFL, flags | libc::O_NONBLOCK) };
    assert_eq!(set, 0, "the read end is made non-blocking");
}

/// What:
///  What a read end holds now,
///  and nothing more once the stream has been quiet for `quiet`.
/// Why:
///   A control reads the lines a thread wrote without waiting for the end of a stream nothing closes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = readWhileItKeepsComing(reader, 2_000)
/// ```
#[cfg(target_os = "linux")]
fn read_while_it_keeps_coming(reader: &mut std::io::PipeReader, within: Duration) -> String {
    read_instead_of_waiting(reader);
    let deadline: Instant = Instant::now() + within;
    let mut text: Vec<u8> = Vec::new();
    let mut buffer: [u8; 4096] = [0; 4096];
    while Instant::now() < deadline {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => text.extend_from_slice(&buffer[..read]),
            // A stream with nothing more now is read again after a tick, up to the bound of this control.
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(5)),
            Err(other) => panic!("the stream of the control could not be read: {other}"),
        }
    }
    return String::from_utf8_lossy(text.as_slice()).into_owned();
}

/// What:
///  A stream nobody drains holds no caller for longer than one bound,
///  the queue stops growing at its bound,
///  and the lines it could not take are counted.
/// Why:
///  The harm this file exists to remove is a process that waits in a write while it holds something another command
/// waits for, and the incident on record is a stop of a push that never returned
/// (`doc/planning/cli-git-no-unbounded-waits.md`, "Writes to the caller's own streams").
///  This control never drains the stream,
/// so a caller that waited for each line would take the bound per line:
/// handing twice the queue's length and measuring the whole loop is what shows the wait is paid once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = full(); giveTheThread(writer); certify(elapsed(() => { for (2 * QUEUED_LINES) write(line) }) <= 2 * bound)
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_stream_nobody_drains_holds_no_caller() {
    let (reader, writer) = a_pipe_that_is_full();
    give_the_thread(Box::new(writer));
    let lines: usize = QUEUED_LINES.saturating_mul(2);
    let began: Instant = Instant::now();
    for index in 0..lines {
        write(format!("a diagnostic line that nobody reads {index}\n").as_str());
    }
    let elapsed: Duration = began.elapsed();
    let queued: usize = queued();
    let dropped: u64 = dropped_so_far();
    // The thread holds at most one line in its own write, so the queue and the count account for every line handed over.
    assert!(queued <= QUEUED_LINES, "the queue never grows past its bound: {queued}");
    assert!(dropped > 0, "a stream nobody drains makes the queue drop lines rather than hold the caller: {queued} queued, {dropped} dropped");
    assert!(the_reader_stopped(), "a reader that took no line within the bound is remembered as one that is not taking lines");
    // One bound is what the first line costs, and no later line waits at all: two bounds leave room for a loaded host
    //  without leaving room for a wait per line, which the whole loop would then take.
    assert!(
        elapsed < WARNING_WRITE_CEILING.saturating_mul(4),
        "no caller waits for the reader past one bound: {lines} lines took {elapsed:?}"
    );
    drop(reader);
}

/// What:
///  A stream that drains gets every line,
///  in the order the callers said them.
/// Why:
///  A diagnostic is read beside the lines a command writes by hand,
/// so the door must keep the order of what it is given;
///  the wait for the reader that keeps up is what keeps it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// giveTheThread(writer); write('one'); write('two'); write('three'); expect(read()).toBe('onetwothree')
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_stream_that_drains_gets_the_lines_in_the_order_they_were_said() {
    let (mut reader, writer) = std::io::pipe().expect("a pipe");
    give_the_thread(Box::new(writer));
    let said: [&str; 3] = ["[warn] the first line\n", "[debug] the second line\n", "[warn] the third line\n"];
    for line in said {
        write(line);
    }
    let text: String = read_while_it_keeps_coming(&mut reader, Duration::from_millis(500));
    assert_eq!(text, said.concat(), "every line reached the stream, in the order it was said");
    assert!(!the_reader_stopped(), "a reader that takes every line is never taken for one that stopped");
}

/// What:
///  A line handed over just before the process ends is in the stream when the end waits for it.
/// Why:
///   `std::process::exit` ends the thread where it stands,
/// so the end waits for what was handed over;
///  a refusal's last line is the whole of what a person gets
/// (`doc/planning/cli-git-no-unbounded-waits.md`, "Writes to the caller's own streams").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// giveTheThread(writer); write(line); expect(writeWhatWaits(bound)).toBe(true); expect(read()).toContain(line)
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_line_handed_before_the_end_waits_and_reaches_the_stream() {
    let (mut reader, writer) = std::io::pipe().expect("a pipe");
    give_the_thread(Box::new(writer));
    let line: &str = "[warn] the last line before this process ends\n";
    write(line);
    assert!(write_what_waits(WAIT_BEFORE_AN_EXIT), "the queue was empty and nothing was being written");
    let text: String = read_while_it_keeps_coming(&mut reader, Duration::from_millis(500));
    assert_eq!(text, line, "what the end waited for is what the stream holds");
}

/// What:
///  The end of a process gives up on a stream nobody drains at its bound,
///  and says nothing that would wait for that stream.
/// Why:
///   The end waits once and bounded,
/// never for as long as a reader does not read,
///  and a process that is ending is the one place a wait is allowed to fail silently:
///  there is no stream left to say it on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = full(); giveTheThread(writer); write(line); expect(writeWhatWaits(bound)).toBe(false); expect(elapsed).toBeAtLeast(bound)
/// ```
#[cfg(target_os = "linux")]
#[test]
fn the_end_gives_up_at_its_bound_on_a_stream_nobody_drains() {
    let (reader, writer) = a_pipe_that_is_full();
    give_the_thread(Box::new(writer));
    write("[warn] a line that waits for a reader\n");
    let began: Instant = Instant::now();
    let written: bool = write_what_waits(Duration::from_millis(100));
    let elapsed: Duration = began.elapsed();
    assert!(!written, "a stream nobody drains leaves the line unwritten at the end");
    assert!(elapsed >= Duration::from_millis(100), "the end waits for its bound before giving up: {elapsed:?}");
    drop(reader);
}

/// What:
///  A line the queue could not take is counted,
///  and the next line that gets through says how many were dropped before it.
/// Why:
///  A gap in the diagnostics must be a gap the diagnostics state,
/// so a person reading a log is never left believing that a command said nothing
/// when what happened is that its lines found no room.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = full(); giveTheThread(writer); for (2 * QUEUED_LINES) write(line); drain(reader); expect(read()).toContain('were dropped')
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_line_the_queue_could_not_take_is_counted_and_said_before_the_next() {
    let (mut reader, writer) = a_pipe_that_is_full();
    give_the_thread(Box::new(writer));
    let lines: usize = QUEUED_LINES.saturating_mul(2);
    for index in 0..lines {
        write(format!("a diagnostic line that nobody reads {index}\n").as_str());
    }
    assert!(dropped_so_far() > 0, "a queue that is full counts its lines rather than growing");
    // The reader comes back: what the thread held reaches the stream, and the first line it takes after the gap says how
    //  many were dropped before it.
    let text: String = read_while_it_keeps_coming(&mut reader, Duration::from_millis(500));
    assert!(text.contains(DROPPED_NOTICE), "the stream states the gap it took: {text:?}");
}
