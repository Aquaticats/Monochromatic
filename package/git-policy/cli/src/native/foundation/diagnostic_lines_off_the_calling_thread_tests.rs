//! Controls for the diagnostic lines on a thread of their own:
//! a stream that never drains holds no caller for longer than one bound,
//! a stream that drains gets the lines in the order they were said,
//! the queue is bounded and says how many lines it dropped,
//! and the end of a process waits for what it handed over and no longer.
//!
//! These controls need one process per test:
//!  the queue and its thread belong to the process,
//! `give_the_thread` puts a control's stream in the one place they write,
//!  and a control whose reader goes
//! ends that thread for good.
//! A run that shares one process between them therefore fails them whichever order it takes:
//! measured on 2026-11-25,
//!  `cargo test --lib diagnostic_lines_off_the_calling_thread` failed 3 of the 5 on the
//! pristine tree and 4 of the 5 after an unrelated change to the door,
//!  `--test-threads=1` failed 2 of the 5,
//!  and
//! each of the five passed alone in a process of its own.
//! So each control runs its body in a child process of its own
//! (`testing/process_wide_state_child_test_support.rs`),
//!  which is also how the package's gate runs them
//! (`cargo nextest`),
//!  and `cargo test --lib` passes them now (issue 6487).

use super::*;

/// The lines this thread has written,
/// which is where the notice about a stopped reader is read.
use crate::diagnostic_log::take_lines_written;
/// Reading what a stream already holds,
/// which is how a control sees a line without waiting for a thread.
use std::io::Read;
/// The two times a control reads:
/// when a bound has passed,
/// and how long it waited.
use std::time::{
    Duration,
    Instant,
};

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
    let capacity: std::ffi::c_int =
        unsafe { libc::fcntl(std::os::fd::AsRawFd::as_raw_fd(&reader), libc::F_GETPIPE_SZ) };
    drop(writer);
    drop(reader);
    assert!(
        capacity > 0,
        "the kernel did not report what one pipe holds: {capacity}"
    );
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
    filler
        .write_all(vec![b'.'; capacity].as_slice())
        .expect("the stream fills");
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
    let flags: std::ffi::c_int =
        unsafe { libc::fcntl(std::os::fd::AsRawFd::as_raw_fd(reader), libc::F_GETFL) };
    assert!(flags >= 0, "the read end's flags are readable");
    // SAFETY: the same descriptor, with the flags it already had plus the one that makes a read return instead of waiting.
    let set: std::ffi::c_int = unsafe {
        libc::fcntl(
            std::os::fd::AsRawFd::as_raw_fd(reader),
            libc::F_SETFL,
            flags | libc::O_NONBLOCK,
        )
    };
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
fn read_while_it_keeps_coming(
    reader: &mut std::io::PipeReader,
    within: Duration,
) -> String {
    read_instead_of_waiting(reader);
    let deadline: Instant = Instant::now() + within;
    let mut text: Vec<u8> = Vec::new();
    let mut buffer: [u8; 4096] = [0; 4096];
    while Instant::now() < deadline {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => text.extend_from_slice(&buffer[..read]),
            // A stream with nothing more now is read again after a tick, up to the bound of this control.
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(5))
            }
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
/// waits for,
///  and the incident on record is a stop of a push that never returned
/// (`doc/planning/cli-git-no-unbounded-waits.md`,
///  "Writes to the caller's own streams").
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
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_stream_nobody_drains_holds_no_caller",
        || {
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
            assert!(
                queued <= QUEUED_LINES,
                "the queue never grows past its bound: {queued}"
            );
            assert!(
                dropped > 0,
                "a stream nobody drains makes the queue drop lines rather than hold the caller: {queued} queued, {dropped} dropped"
            );
            assert!(
                the_reader_stopped(),
                "a reader that took no line within the bound is remembered as one that is not taking lines"
            );
            // One bound is what the first line costs, and no later line waits at all: two bounds leave room for a loaded host
            //  without leaving room for a wait per line, which the whole loop would then take.
            assert!(
                elapsed < WARNING_WRITE_CEILING.saturating_mul(4),
                "no caller waits for the reader past one bound: {lines} lines took {elapsed:?}"
            );
            drop(reader);
        },
    );
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
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_stream_that_drains_gets_the_lines_in_the_order_they_were_said",
        || {
            let (mut reader, writer) = std::io::pipe().expect("a pipe");
            give_the_thread(Box::new(writer));
            let said: [&str; 3] = [
                "[warn] the first line\n",
                "[debug] the second line\n",
                "[warn] the third line\n",
            ];
            for line in said {
                write(line);
            }
            let text: String = read_while_it_keeps_coming(&mut reader, Duration::from_millis(500));
            assert_eq!(
                text,
                said.concat(),
                "every line reached the stream, in the order it was said"
            );
            assert!(
                !the_reader_stopped(),
                "a reader that takes every line is never taken for one that stopped"
            );
        },
    );
}

/// What:
///  A line handed over just before the process ends is in the stream when the end waits for it.
/// Why:
///   `std::process::exit` ends the thread where it stands,
/// so the end waits for what was handed over;
///  a refusal's last line is the whole of what a person gets
/// (`doc/planning/cli-git-no-unbounded-waits.md`,
///  "Writes to the caller's own streams").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// giveTheThread(writer); write(line); expect(writeWhatWaits(bound)).toBe(true); expect(read()).toContain(line)
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_line_handed_before_the_end_waits_and_reaches_the_stream() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_line_handed_before_the_end_waits_and_reaches_the_stream",
        || {
            let (mut reader, writer) = std::io::pipe().expect("a pipe");
            give_the_thread(Box::new(writer));
            let line: &str = "[warn] the last line before this process ends\n";
            write(line);
            assert!(
                write_what_waits(WAIT_BEFORE_AN_EXIT),
                "the queue was empty and nothing was being written"
            );
            let text: String = read_while_it_keeps_coming(&mut reader, Duration::from_millis(500));
            assert_eq!(
                text, line,
                "what the end waited for is what the stream holds"
            );
        },
    );
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
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "the_end_gives_up_at_its_bound_on_a_stream_nobody_drains",
        || {
            let (reader, writer) = a_pipe_that_is_full();
            give_the_thread(Box::new(writer));
            write("[warn] a line that waits for a reader\n");
            let began: Instant = Instant::now();
            let written: bool = write_what_waits(Duration::from_millis(100));
            let elapsed: Duration = began.elapsed();
            assert!(
                !written,
                "a stream nobody drains leaves the line unwritten at the end"
            );
            assert!(
                elapsed >= Duration::from_millis(100),
                "the end waits for its bound before giving up: {elapsed:?}"
            );
            drop(reader);
        },
    );
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
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_line_the_queue_could_not_take_is_counted_and_said_before_the_next",
        || {
            let (mut reader, writer) = a_pipe_that_is_full();
            give_the_thread(Box::new(writer));
            let lines: usize = QUEUED_LINES.saturating_mul(2);
            for index in 0..lines {
                write(format!("a diagnostic line that nobody reads {index}\n").as_str());
            }
            assert!(
                dropped_so_far() > 0,
                "a queue that is full counts its lines rather than growing"
            );
            // The reader comes back: what the thread held reaches the stream, and the first line it takes after the gap says how
            //  many were dropped before it.
            let text: String = read_while_it_keeps_coming(&mut reader, Duration::from_millis(500));
            assert!(
                text.contains(DROPPED_NOTICE),
                "the stream states the gap it took: {text:?}"
            );
        },
    );
}

/// What:
///  A process whose process limit refuses the thread that writes its lines says which limit refused it,
///  and every line the process says after that refusal reaches the stream.
/// Why:
///   A commit at its process limit failed on 2026-11-25 saying nothing at all (issue 6476):
/// its refusal waited in a queue that no thread would ever write.
///  The body runs in a child whose process limit is one,
/// which is the one state in which the thread is refused,
///  and the two lines after the refusal are a command's first word and its last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inChild(() => { const [reader, writer] = pipe(); giveTheThread(writer); write(first); write(last); expect(read()).toContain('ulimit -u'); expect(read()).toContain(first); expect(read()).toContain(last) })
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_process_refused_the_writer_thread_says_which_limit_it_was_and_says_its_lines() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_process_refused_the_writer_thread_says_which_limit_it_was_and_says_its_lines",
        || {
            crate::no_threads_child_test_support::run_the_body_where_no_thread_can_start(
                module_path!(),
                "a_process_refused_the_writer_thread_says_which_limit_it_was_and_says_its_lines",
                || {
                    let (mut reader, writer) = std::io::pipe().expect("a pipe");
                    give_the_thread(Box::new(writer));
                    let first: &str =
                        "[warn] the line a process at its process limit would have lost\n";
                    let last: &str = "monochromatic-git-policy-cli: the line after it, which a queue no thread writes would also have lost\n";
                    write(first);
                    write(last);
                    let said: String =
                        read_while_it_keeps_coming(&mut reader, Duration::from_millis(500));
                    assert!(
                        said.contains("ulimit -u"),
                        "the refusal names the process limit that refused the thread, and not only the system's error: {said}"
                    );
                    assert!(
                        said.contains(first),
                        "the line said as the refusal is written where it is said: {said}"
                    );
                    assert!(
                        said.contains(last),
                        "the line said after the refusal is written where it is said too: {said}"
                    );
                },
            );
        },
    );
}

/// What:
///  A pipe filled to its capacity and then read from until `pages` whole pages are free,
///  with the read end a control drains when it wants the words.
/// Why:
///   A line written where it is said is bounded by the room a poll reports,
///  and the room a pipe reports is one free slot,
///  so a control that wants a partly full stream makes exactly the pages it chooses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = filledPipe(); reader.readExact(pages * 4096); return [reader, writer];
/// ```
#[cfg(target_os = "linux")]
fn a_pipe_with_whole_pages_free(pages: usize) -> (std::io::PipeReader, std::io::PipeWriter) {
    let (mut reader, writer) = a_pipe_that_is_full();
    let mut room: Vec<u8> = vec![
        0;
        pages.saturating_mul(
            crate::lines_written_by_a_thread::THE_MOST_A_PIPE_TAKES_WHEN_IT_SHOWS_ROOM,
        )
    ];
    reader
        .read_exact(room.as_mut_slice())
        .expect("the room this control makes");
    return (reader, writer);
}

/// What:
///  A line longer than the stream's free room is settled at once,
///  and it is counted as a line the stream could not say.
/// Why:
///   A process whose process limit refused the queue's thread writes every line where it is said.
///  `POLLOUT` promises one free pipe page rather than room for any line:
///  a line of two or three pages written in one call waits inside the kernel for a reader that never reads,
///  which is the wait the fallback exists to avoid
///  (issue 6476;
///  the release pass of 2026-11-26).
///  So each write is bounded by one page and a line that does not fit costs its tail.
///  This process's own standard error is the stream under test here,
///  because a control's stream is memory and the fallback writes it whole:
///  this child gives its own descriptor 2 the pipe below,
///  and puts its first standard error back before it judges,
///  so a failed assertion can still be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inNoThreadChild(() => { const first = dup(2); const [reader, writer] = pipeWithPagesFree(2); dup2(writer.fd, 2); const line = 'x'.repeat(3 * 4096) + '\n'; const began = now(); write(line); const took = now() - began; const dropped = droppedSoFar(); dup2(first, 2); expect(took).toBeLessThan(5s); expect(dropped).toBe(1) })
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_line_longer_than_the_stream_s_room_is_settled_at_once() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_line_longer_than_the_stream_s_room_is_settled_at_once",
        || {
            crate::no_threads_child_test_support::run_the_body_where_no_thread_can_start(
                module_path!(),
                "a_line_longer_than_the_stream_s_room_is_settled_at_once",
                || {
                    let page: usize =
                        crate::lines_written_by_a_thread::THE_MOST_A_PIPE_TAKES_WHEN_IT_SHOWS_ROOM;
                    // What: The child's first standard error, kept under a number of its own.
                    // Why:  A judgement this control makes after the write must be readable, so descriptor 2
                    //       goes back to this before any assertion.
                    // SAFETY: `dup` copies one open descriptor of this child and writes no memory here.
                    let first: libc::c_int = unsafe { libc::dup(libc::STDERR_FILENO) };
                    assert!(first >= 0, "this child's own standard error is copied");
                    // What: Two whole pages of room beyond a full pipe, so the refusal's own sentence cannot take it all.
                    // Why:  The queue says which limit refused its thread before it says the first line.
                    let (reader, writer) = a_pipe_with_whole_pages_free(2);
                    // What: This child's standard error becomes the pipe, with `dup2` on descriptor 2.
                    // Why:  The queue's default stream is this process's standard error descriptor,
                    //       and a control's own stream is memory the fallback writes whole, so the bounded write
                    //       is only reached through descriptor 2.
                    let write_end: std::os::fd::RawFd = std::os::fd::AsRawFd::as_raw_fd(&writer);
                    // SAFETY: `dup2` copies one open descriptor of this child onto another number it names.
                    let copied: std::os::fd::RawFd =
                        unsafe { libc::dup2(write_end, libc::STDERR_FILENO) };
                    assert_eq!(
                        copied,
                        libc::STDERR_FILENO,
                        "this child's standard error is the pipe of this control"
                    );
                    // The pipe stays open at descriptor 2 once this end is dropped, and the read end keeps it
                    //  a stream nobody takes from.
                    drop(writer);
                    // What: A line of three pages, so a write of the whole of it is more than the room a poll reports.
                    // Why:  The fallback promises that a caller never waits for the reader, and this is the line that
                    //       broke that promise.
                    let line: String = format!("{}\n", "x".repeat(3usize.saturating_mul(page)));
                    let began: Instant = Instant::now();
                    write(line.as_str());
                    let took: Duration = began.elapsed();
                    let dropped: u64 = dropped_so_far();
                    // SAFETY: `dup2` puts the child's first standard error back on descriptor 2.
                    let restored: libc::c_int = unsafe { libc::dup2(first, libc::STDERR_FILENO) };
                    assert_eq!(
                        restored,
                        libc::STDERR_FILENO,
                        "standard error is readable again"
                    );
                    // SAFETY: the copy is closed here, and no other call uses its number.
                    let _closed: libc::c_int = unsafe { libc::close(first) };
                    drop(reader);
                    assert!(
                        took < Duration::from_secs(5),
                        "a line longer than the stream's room was settled in {took:?}, so the caller waited for a reader"
                    );
                    assert_eq!(
                        dropped, 1,
                        "the line the stream could not take whole is counted, as a line the queue's bound dropped is"
                    );
                },
            );
        },
    );
}

/// What:
///  A line the queue put in a stream that takes it is answered written.
/// Why:
///  A caller that must not lose what its line says keeps its record on this answer,
///  so a line the person did read must be told apart from one they did not.
#[cfg(target_os = "linux")]
#[test]
fn a_line_the_stream_took_is_answered_written() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_line_the_stream_took_is_answered_written",
        || {
            let (mut reader, writer) = std::io::pipe().expect("a pipe");
            give_the_thread(Box::new(writer));
            let answered: HandedOver = write_bytes(b"[warn] a line the reader takes\n");
            assert_eq!(
                answered,
                HandedOver::Written,
                "the line is in the stream, so the caller is told it is"
            );
            let text: String = read_while_it_keeps_coming(&mut reader, Duration::from_millis(500));
            assert_eq!(
                text, "[warn] a line the reader takes\n",
                "the line the answer is about is the line the stream holds"
            );
        },
    );
}

/// What:
///  A line a closed stream refused is answered unwritten.
/// Why:
///   The write of the line answered nothing,
///  because the thread had not written it yet;
///  this answer is what a record that must outlive its line is kept by
///  (`Notice::show`).
#[cfg(target_os = "linux")]
#[test]
fn a_line_a_closed_stream_refused_is_answered_unwritten() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_line_a_closed_stream_refused_is_answered_unwritten",
        || {
            give_the_thread(Box::new(crate::test_support::AStreamThatRefusesEveryByte));
            let answered: HandedOver = write_bytes(b"[warn] a line nobody can read\n");
            assert_eq!(
                answered,
                HandedOver::Unwritten,
                "a stream that refused the line leaves it out of the stream"
            );
        },
    );
}

/// What:
///  A line a stream nobody drains could not take is answered unwritten,
///  and the caller is not held for the reader.
/// Why:
///   Two promises meet at this line:
///  the person must not be told the words reached them when they did not,
///  and the command must never wait for a reader that stopped draining (issue 6421).
///  One bound is what the first line of such a stream costs,
///  so this control reads both.
#[cfg(target_os = "linux")]
#[test]
fn a_line_a_stream_nobody_drains_is_answered_unwritten_and_holds_no_caller() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_line_a_stream_nobody_drains_is_answered_unwritten_and_holds_no_caller",
        || {
            let (reader, writer) = a_pipe_that_is_full();
            give_the_thread(Box::new(writer));
            let began: Instant = Instant::now();
            let answered: HandedOver = write_bytes(b"[warn] a line nobody drains reads\n");
            let elapsed: Duration = began.elapsed();
            assert_eq!(
                answered,
                HandedOver::Unwritten,
                "a line still waiting for a reader is not in the stream"
            );
            // One bound is what the first line of a stream nobody drains costs, and four leave room for a loaded
            //  host without leaving room for a wait that has no end.
            assert!(
                elapsed < WARNING_WRITE_CEILING.saturating_mul(4),
                "the caller waits one bound and not for the reader: {elapsed:?}"
            );
            drop(reader);
        },
    );
}

/// What:
///  The line that says this queue's reader stopped taking lines is said once,
///  at the bound the caller's wait gives the reader up at,
///  through the door a diagnostic line takes:
///  the words name the queue and the bound,
///  and the line reaches the stream the queue's thread writes.
/// Why:
///  The notice is the only account a person gets of diagnostics that stopped reaching them,
///  so a change that moved where it is said must not move when it is said,
///  what it says,
///  or whether it is said twice.
///  A reader that takes its line at once owes no notice,
///  so the first arm is the positive control for the arm that owes one:
///  without it,
///  a notice said for every line would pass.
///  The notice is read through `take_lines_written`,
///  which only the door over the words calls,
///  so a notice that reached the queue some other way would leave that list empty.
///  The bound is the code's own,
///  and a loaded host may wake this control's thread late,
///  so the wait is allowed thirty times it (issue 6519).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inChildWith({ MONOCHROMATIC_VERBOSE: 'true' }, () => {
///   giveTheThread(draining()); expect(writeBytes(line)).toBe('Written'); expect(takenLines()).toEqual([])
///   giveTheThread(full()); const began = now(); expect(writeBytes(line2)).toBe('Unwritten')
///   expect(now() - began).toBeAtLeast(250); expect(takenLines()).toEqual([noticeLine])
///   expect(writeBytes(line3)).toBe('Unwritten'); expect(takenLines()).toEqual([])
///   expect(drain(reader)).toContain(noticeLine)
/// })
/// ```
#[cfg(target_os = "linux")]
#[test]
fn the_notice_about_a_reader_that_stopped_is_said_once_at_its_bound() {
    crate::process_wide_state_child_test_support::run_the_body_with_an_environment_of_its_own(
        module_path!(),
        "the_notice_about_a_reader_that_stopped_is_said_once_at_its_bound",
        &[("MONOCHROMATIC_VERBOSE", std::ffi::OsStr::new("true"))],
        &["MONOCHROMATIC_WARN"],
        || {
            // What: every word of the notice after the time it carries, which is the one part of a line this control
            //       cannot know.
            // Why:  a debug line is stamped when it is said, so the words the control is about are the rest of it.
            let owed: String = format!(
                "[monochromatic-git-policy-cli] [{TAG}] the reader of this queue's stream took no line within \
                 {within:?}, so the command stops waiting for it\n",
                within = WARNING_WRITE_CEILING
            );
            let _before: Vec<String> = take_lines_written();
            // What: a reader that takes every line at once, which owes no notice.
            // Why:  the arm below reads a notice, and this one proves the notice is not simply every line's.
            let (mut keeping_up, writer) = std::io::pipe().expect("a pipe");
            give_the_thread(Box::new(writer));
            let taken: HandedOver = write_bytes(b"[warn] a line a reader takes at once\n");
            assert_eq!(
                taken,
                HandedOver::Written,
                "a reader that is taking lines takes this one, so nothing is owed the person about it"
            );
            assert!(
                take_lines_written().is_empty(),
                "a reader that took the line is not one the queue says stopped"
            );
            let _taken: String =
                read_while_it_keeps_coming(&mut keeping_up, Duration::from_millis(500));
            // What: the same queue with a reader that has stopped, which is what the notice is about.
            // Why:  the wait gives the reader up at its bound, and the notice is said at that moment.
            let (mut stopped, full) = a_pipe_that_is_full();
            give_the_thread(Box::new(full));
            let before_the_bound_passed: Instant = Instant::now();
            let left_out: HandedOver =
                write_bytes(b"[warn] a line the stopped reader never takes\n");
            let waited: Duration = before_the_bound_passed.elapsed();
            assert_eq!(
                left_out,
                HandedOver::Unwritten,
                "a reader that stopped taking lines leaves the line out of the stream"
            );
            assert!(
                waited >= WARNING_WRITE_CEILING,
                "the notice waits for the reader until the bound passes, and this call waited {waited:?}"
            );
            assert!(
                waited < WARNING_WRITE_CEILING.saturating_mul(30),
                "the notice is said at the bound and not at the end of a wait with no end: {waited:?}"
            );
            let said: Vec<String> = take_lines_written();
            assert_eq!(
                said.len(),
                1,
                "one notice is said for one reader that stopped: {said:?}"
            );
            assert!(
                said[0].starts_with("[debug] [") && said[0].ends_with(owed.as_str()),
                "the notice is a debug line saying which stream's reader stopped and the bound it gave up at: {}",
                said[0]
            );
            // What: a second line for the same stopped reader, which pays no bound and says no second notice.
            // Why:  the notice is owed once, and no later caller waits for a reader the queue already gave up on.
            let handed_again: Instant = Instant::now();
            let left_out_again: HandedOver =
                write_bytes(b"[warn] a second line the stopped reader never takes\n");
            let again: Duration = handed_again.elapsed();
            assert_eq!(
                left_out_again,
                HandedOver::Unwritten,
                "a reader that stopped taking lines leaves every later line out of the stream"
            );
            assert!(
                again < WARNING_WRITE_CEILING,
                "a later line waits for no reader at all: {again:?}"
            );
            assert!(
                take_lines_written().is_empty(),
                "the notice is said once and not for every line the stopped reader is owed"
            );
            // What: the stream the queue's thread writes, drained so the words it holds can be read.
            // Why:  the notice is a diagnostic line like any other, so it reaches the person by that route.
            let text: String = read_while_it_keeps_coming(&mut stopped, Duration::from_millis(500));
            assert!(
                text.contains(owed.as_str()),
                "the notice reaches the stream the queue writes: {text:?}"
            );
            drop(stopped);
            // What: a reader replaced by one that stops in its turn, which is owed a notice of its own.
            // Why:  the notice follows the queue's state and not a latch a process keeps, so a control that arms a fresh
            //       reader reads a fresh notice; a process whose reader stopped once and never came back says one line.
            let (stopped_again, replaced_by_one_that_stopped) = a_pipe_that_is_full();
            give_the_thread(Box::new(replaced_by_one_that_stopped));
            let began: Instant = Instant::now();
            let left_out_after_the_replacement: HandedOver =
                write_bytes(b"[warn] a line of the reader that replaced the first one\n");
            let waited_again: Duration = began.elapsed();
            assert_eq!(
                left_out_after_the_replacement,
                HandedOver::Unwritten,
                "the reader that replaced the first one took nothing either"
            );
            assert!(
                waited_again >= WARNING_WRITE_CEILING,
                "the notice for the second reader waits for it until the bound passes: {waited_again:?}"
            );
            let said_again: Vec<String> = take_lines_written();
            assert_eq!(
                said_again.len(),
                1,
                "a reader that stopped is owed one notice of its own: {said_again:?}"
            );
            assert!(
                said_again[0].starts_with("[debug] [") && said_again[0].ends_with(owed.as_str()),
                "the second notice says the same words as the first: {}",
                said_again[0]
            );
            drop(stopped_again);
        },
    );
}
