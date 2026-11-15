//! Controls for the service's log lines on a thread of their own:
//! a stream that never drains holds no caller,
//! the queue is bounded and says how many lines it dropped.

use super::*;

/// The two times a control reads: when a bound has passed, and how long it waited.
use std::time::{Duration, Instant};
/// The descriptor behind a pipe end, which `fcntl` and `poll` take.
use std::os::fd::AsRawFd;

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
///  A stream nobody drains holds no caller,
///  the queue stops growing at its bound,
///  and the lines it could not take are counted.
/// Why:
///  A landing that wrote its own log line straight to standard error was held 7.35 seconds by a full FIFO
/// (`pi-runs/kimi-audit-alpha-14-commit-path-1`),
/// and the same write against a slow journal is a wait of unknowable length.
///  This control never drains the stream,
/// so a caller that waited for it would never return and the control itself would not finish:
/// handing twice the queue's length of lines and reading the count is what shows the caller does not wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = pipe(); fill(writer, capacity); giveTheThread(writer); for (n) hand(line); expect(dropped()).toBeGreaterThan(0);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_stream_nobody_drains_holds_no_caller() {
    let capacity: usize = one_pipe_s_holding();
    let (reader, writer) = std::io::pipe().expect("a pipe");
    // The stream is full before the thread is given it, so the thread's first write is the one that waits.
    let mut filled: Box<dyn Write + Send> = Box::new(writer.try_clone().expect("a second write end"));
    let block: Vec<u8> = vec![b'.'; capacity];
    filled.write_all(block.as_slice()).expect("the stream fills");
    give_the_thread(Box::new(writer));
    // Twice the queue's length: whatever the thread has taken, the rest cannot all fit, and every line that does not fit
    //  is counted rather than waited for.
    let lines: usize = QUEUED_LINES.saturating_mul(2);
    for index in 0..lines {
        let line: String = format!("a log line that nobody reads {index}\n");
        let _taken = hand(line.as_str());
    }
    let queued: usize = queued();
    let dropped: u64 = dropped_so_far();
    // The thread holds at most one line in its own write, so the queue and the count account for every line handed over.
    assert!(queued <= QUEUED_LINES, "the queue never grows past its bound: {queued}");
    assert!(dropped > 0, "a stream nobody drains makes the queue drop lines rather than hold the caller: {queued} queued, {dropped} dropped");
    assert!(
        u64::try_from(queued).unwrap_or(u64::MAX).saturating_add(dropped) <= u64::try_from(lines).unwrap_or(u64::MAX),
        "no line is counted twice: {queued} queued and {dropped} dropped of {lines}"
    );
    // The stream is closed, so the waiting thread finds it closed instead of standing in a write for the life of this process.
    drop(reader);
    drop(filled);
}

/// What:
///  A line handed over just before the process ends is in the stream when the end waits for it,
///  and a stream nobody drains holds the end for its bound and no longer.
/// Why:
///   `std::process::exit` ends the thread where it stands,
/// so before the end waited for it,
/// a refusal's one line was out of the queue and gone in most runs:
/// the `service_mode` controls of 2026-11-14 read no byte at all from such a start,
/// and the wrapper owed the person a reason.
///  The second arm is the promise the queue was built on:
/// a stream that never drains costs the process one bounded wait at its end,
/// and is never waited for per line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = pipe(); giveTheThread(writer); hand(line); expect(writeWhatWaits(bound)).toBe(true); expect(readNonBlocking(reader)).toContain(line);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_line_handed_before_the_end_waits_and_reaches_the_stream() {
    let (mut reader, writer) = std::io::pipe().expect("a pipe");
    give_the_thread(Box::new(writer));
    let line: &str = "the last line before this process ends\n";
    assert!(hand(line), "the queue has room for one line");
    assert!(write_what_waits(WAIT_BEFORE_AN_EXIT), "the line reached the stream within the bound");
    // The stream is read the moment nothing is waiting to write it, so a non-blocking read has the whole line or nothing.
    // SAFETY: `fcntl` takes a descriptor this process owns and an integer command, and writes no memory here.
    let flags: std::ffi::c_int = unsafe { libc::fcntl(std::os::fd::AsRawFd::as_raw_fd(&reader), libc::F_GETFL) };
    assert!(flags >= 0, "the read end's flags are readable");
    // SAFETY: the same descriptor, with the flags it already had plus the one that makes a read return instead of waiting.
    let set: std::ffi::c_int = unsafe { libc::fcntl(std::os::fd::AsRawFd::as_raw_fd(&reader), libc::F_SETFL, flags | libc::O_NONBLOCK) };
    assert_eq!(set, 0, "the read end is made non-blocking");
    let mut text: String = String::new();
    let _taken = std::io::Read::read_to_string(&mut reader, &mut text);
    assert_eq!(text, line, "what the end waited for is what the stream holds");

    // The arm that proves the bound: a stream already full, and a line the thread takes and stands in.
    let capacity: usize = one_pipe_s_holding();
    let (unread_end, write_end) = std::io::pipe().expect("a pipe");
    let mut filled: Box<dyn Write + Send> = Box::new(write_end.try_clone().expect("a second write end"));
    filled.write_all(vec![b'.'; capacity].as_slice()).expect("the stream fills");
    give_the_thread(Box::new(write_end));
    assert!(hand("a line nobody can read\n"), "the queue has room");
    let began: std::time::Instant = std::time::Instant::now();
    let written: bool = write_what_waits(WAIT_BEFORE_AN_EXIT);
    assert!(!written, "a stream nobody drains is not written, and the end is told so rather than waiting forever");
    assert!(began.elapsed() < WAIT_BEFORE_AN_EXIT.saturating_mul(8), "the end waited its bound ({} ms) and no longer: {:?}", WAIT_BEFORE_AN_EXIT.as_millis(), began.elapsed());
    // The stream is closed behind the standing write, so the thread that waits there ends with this stream rather than
    //  with the process.
    drop(unread_end);
    drop(filled);
}

/// What:
///  Makes one end of a pipe nonblocking,
///  so that a write to a full stream answers rather than waiting for room.
/// Why:
///  A caller that opened standard error without blocking is a real arrangement,
/// and it is the one in which a line is refused instead of waited for,
/// so it is the one the four controls of this file need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// makeNonblocking(pipeWriter, 'write end')
/// ```
#[cfg(target_os = "linux")]
fn make_nonblocking(end: &impl std::os::fd::AsRawFd, what: &str) {
    // SAFETY: `fcntl` reads the flags of a descriptor this process owns and writes no memory here.
    let flags: std::ffi::c_int = unsafe { libc::fcntl(end.as_raw_fd(), libc::F_GETFL) };
    assert!(flags >= 0, "the {what}'s flags are readable");
    // SAFETY: this adds only `O_NONBLOCK` to that descriptor's own flags.
    let set: std::ffi::c_int = unsafe { libc::fcntl(end.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) };
    assert_eq!(set, 0, "the {what} is made nonblocking");
}

/// What:
///  Reads what a stream gives until `within` has passed,
///  and gives the bytes in the order they arrived.
/// Why:
///  A control has to read what actually reached the stream,
/// and it must not wait past its own bound for bytes that never come:
/// `poll` waits on the descriptor and each read takes only what is there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const captured = whatTheStreamGivesWithin(reader, bound);
/// ```
#[cfg(target_os = "linux")]
fn what_the_stream_gives_within(reader: &mut std::io::PipeReader, within: Duration) -> Vec<u8> {
    make_nonblocking(reader, "read end");
    let began: Instant = Instant::now();
    let mut bytes: Vec<u8> = Vec::new();
    loop {
        let Some(left) = within.checked_sub(began.elapsed()) else {
            return bytes;
        };
        let millis: std::ffi::c_int = std::ffi::c_int::try_from(left.as_millis()).unwrap_or(std::ffi::c_int::MAX).max(1);
        let mut waiting: libc::pollfd = libc::pollfd { fd: reader.as_raw_fd(), events: libc::POLLIN, revents: 0 };
        // SAFETY: `poll` reads and writes one `pollfd` of this frame, and waits at most `millis` milliseconds.
        let ready: std::ffi::c_int = unsafe { libc::poll(&mut waiting, 1, millis) };
        if ready < 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            // A signal ended the wait and nothing about the stream changed, so the wait is taken again within the bound.
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return bytes;
        }
        if ready == 0 {
            return bytes;
        }
        let mut buffer: [u8; 8192] = [0; 8192];
        match std::io::Read::read(reader, &mut buffer) {
            // The far end is closed and everything it wrote has been read.
            Ok(0) => return bytes,
            Ok(read) => bytes.extend_from_slice(&buffer[..read]),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_unreadable) => return bytes,
        }
    }
}

/// What:
///  A stream with no room when the line is handed over,
///  and a reader that makes room inside the wait,
///  carries the line and the end of the process is told it did.
/// Why:
///  A full nonblocking standard error is a real arrangement,
/// and a write that answers `WouldBlock` wrote nothing at all:
/// the line has to stay the thread's and be written when the room appears,
/// or the person gets the status without the reason
/// (the review of 2026-11-15, `nonblocking_stream_is_reported_written_before_a_reader_within_the_bound`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = pipe(); fill(writer, capacity); makeNonblocking(writer); giveTheThread(writer); hand(line); drainer.readsAfter(50ms); expect(writeWhatWaits(bound)).toBe(true); expect(drainer.bytes).toEndWith(line);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_line_a_stream_has_no_room_for_is_written_when_the_room_appears() {
    let capacity: usize = one_pipe_s_holding();
    let (mut reader, writer) = std::io::pipe().expect("a pipe");
    let mut filling: std::io::PipeWriter = writer.try_clone().expect("a second write end");
    // The empty pipe takes exactly what the kernel says it holds, which leaves no room for one more byte.
    filling.write_all(vec![b'.'; capacity].as_slice()).expect("the stream fills");
    drop(filling);
    make_nonblocking(&writer, "write end");
    give_the_thread(Box::new(writer));
    let line: &str = "the refusal a reader makes room for\n";
    assert!(hand(line), "the queue has room for one line");
    // The room appears well inside the bound the end of the process is given.
    let drainer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        return what_the_stream_gives_within(&mut reader, WAIT_BEFORE_AN_EXIT.saturating_mul(2));
    });
    let written: bool = write_what_waits(WAIT_BEFORE_AN_EXIT);
    let captured: Vec<u8> = drainer.join().expect("the reader's thread");
    assert!(written, "the end is told the line reached the stream");
    assert!(captured.len() >= capacity, "the bytes that filled the stream came back: {} of {capacity}", captured.len());
    let after: &[u8] = &captured[capacity..];
    assert_eq!(after, line.as_bytes(), "the line the reader made room for follows the bytes that filled the stream");
}

/// What:
///  A stream nobody makes room for is not reported written,
///  and the end of the process is told so at its bound and not after it.
/// Why:
///  An answer of `true` promises that every line reached the stream,
/// so a line the stream never took has to be answered `false`,
/// and the person's command has to end anyway:
/// the wait is a promise about the words, never a wait for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fill(writer, capacity); makeNonblocking(writer); giveTheThread(writer); hand(line); const began = now(); expect(writeWhatWaits(bound)).toBe(false); expect(now() - began).toBeAtLeast(bound);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_line_no_stream_will_take_is_not_reported_written() {
    let capacity: usize = one_pipe_s_holding();
    let (_reader, writer) = std::io::pipe().expect("a pipe");
    let mut filling: std::io::PipeWriter = writer.try_clone().expect("a second write end");
    filling.write_all(vec![b'.'; capacity].as_slice()).expect("the stream fills");
    drop(filling);
    make_nonblocking(&writer, "write end");
    give_the_thread(Box::new(writer));
    assert!(hand("a line the stream never takes\n"), "the queue has room for one line");
    let began: Instant = Instant::now();
    let written: bool = write_what_waits(WAIT_BEFORE_AN_EXIT);
    let took: Duration = began.elapsed();
    assert!(!written, "the end is told the line did not reach the stream");
    assert!(took >= WAIT_BEFORE_AN_EXIT, "the end waited its bound for room that never came: {took:?}");
    assert!(took < WAIT_BEFORE_AN_EXIT.saturating_mul(5), "the end waited its bound and no longer: {took:?}");
}

/// What:
///  A stream whose reader is gone is answered at once and not waited for.
/// Why:
///  No wait brings a reader back:
/// a closed pipe answers `EPIPE` for every write from then on,
/// so the line cannot reach anyone and waiting for it would spend the person's bound for nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = pipe(); drop(reader); giveTheThread(writer); hand(line); const began = now(); expect(writeWhatWaits(bound)).toBe(false); expect(now() - began).toBeLessThan(bound / 5);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_stream_whose_reader_is_gone_is_answered_at_once() {
    // SAFETY: this process ignores `SIGPIPE` for the rest of its life, so the write answers `EPIPE` instead of ending the
    //  process that runs the controls. A test binary starts with that same disposition, and the call is stated here
    //  because this control depends on it.
    let _ignored = unsafe { libc::signal(libc::SIGPIPE, libc::SIG_IGN) };
    let (reader, writer) = std::io::pipe().expect("a pipe");
    drop(reader);
    give_the_thread(Box::new(writer));
    assert!(hand("a line no reader can take\n"), "the queue has room for one line");
    let began: Instant = Instant::now();
    let written: bool = write_what_waits(WAIT_BEFORE_AN_EXIT);
    let took: Duration = began.elapsed();
    assert!(!written, "a stream that is gone does not report a line written");
    assert!(took < WAIT_BEFORE_AN_EXIT / 5, "a stream no reader can take is answered at once, not after the bound: {took:?}");
}

/// What:
///  A line longer than the stream's room is written whole,
///  and the line handed over after it follows it and never overtakes it.
/// Why:
///  `write_all` on a nonblocking stream can write part of a line and then fail,
/// so a torn line is the second way a write ends short:
/// what the stream did not take has to stay the thread's,
/// and no other line may be written into the middle of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const long = 'a'.repeat(capacity * 1.5) + '\n'; hand(long); hand(next); drainer.readsAfter(50ms); expect(writeWhatWaits(bound)).toBe(true); expect(captured).toBe(long + next);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_line_longer_than_the_room_is_written_whole_before_the_next_begins() {
    let capacity: usize = one_pipe_s_holding();
    let (mut reader, writer) = std::io::pipe().expect("a pipe");
    make_nonblocking(&writer, "write end");
    give_the_thread(Box::new(writer));
    // One byte more than the pipe holds, so that the write of this one line has to end short.
    let long: String = format!("{}\n", "a".repeat(capacity.saturating_add(1)));
    let next: &str = "the line that waits for the long one\n";
    assert!(hand(long.as_str()), "the queue has room for the long line");
    assert!(hand(next), "the queue has room for the next line");
    let drainer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        return what_the_stream_gives_within(&mut reader, WAIT_BEFORE_AN_EXIT.saturating_mul(2));
    });
    let written: bool = write_what_waits(WAIT_BEFORE_AN_EXIT);
    let captured: Vec<u8> = drainer.join().expect("the reader's thread");
    assert!(written, "the end is told both lines reached the stream");
    assert!(
        captured.len() == long.len().saturating_add(next.len()),
        "the stream holds the whole of both lines: {} bytes of {}",
        captured.len(),
        long.len().saturating_add(next.len())
    );
    assert_eq!(&captured[..long.len()], long.as_bytes(), "the long line is whole before the next one begins");
    assert_eq!(&captured[long.len()..], next.as_bytes(), "the next line follows the long one");
}
