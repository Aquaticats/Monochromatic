//! Controls for the service's log lines on a thread of their own:
//! a stream that never drains holds no caller,
//! the queue is bounded and says how many lines it dropped.

use super::*;

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
    let (reader, writer) = std::io::pipe().expect("a pipe");
    let mut filled: Box<dyn Write + Send> = Box::new(writer.try_clone().expect("a second write end"));
    filled.write_all(vec![b'.'; capacity].as_slice()).expect("the stream fills");
    give_the_thread(Box::new(writer));
    assert!(hand("a line nobody can read\n"), "the queue has room");
    let began: std::time::Instant = std::time::Instant::now();
    let written: bool = write_what_waits(WAIT_BEFORE_AN_EXIT);
    assert!(!written, "a stream nobody drains is not written, and the end is told so rather than waiting forever");
    assert!(began.elapsed() < WAIT_BEFORE_AN_EXIT.saturating_mul(8), "the end waited its bound ({} ms) and no longer: {:?}", WAIT_BEFORE_AN_EXIT.as_millis(), began.elapsed());
    // The stream is closed behind the standing write, so the thread that waits there ends with this stream rather than
    //  with the process.
    drop(reader);
    drop(filled);
}
