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
