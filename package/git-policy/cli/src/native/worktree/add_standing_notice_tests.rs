//! What:
//!  Controls of the line a long-running `git worktree add` prints:
//! its words,
//! when it is printed,
//! that it stops,
//! and that the add's own steps are what it names.
//! Why:
//!  The notice is output only,
//! so what it must not do is as important as what it must:
//! it never ends the add,
//! never names a step the add is not in,
//! and never prints after the add answered.

use super::*;
use crate::add_timing;
use crate::wait_ceilings::ADD_STANDING_NOTICE_AFTER;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

/// What:
///  Everything a notice printed during one control,
/// read until `stop` says how many lines are wanted or a bound passes.
/// Why:
///   The line is printed on the notice's own thread,
/// so a control reads what arrived rather than what it hopes arrived,
/// and never waits for the thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = (lines: mpsc.Receiver<string>, wanted: number, bound: Duration) => { const said = []; while (said.length < wanted && !timedOut) said.push(lines.recvTimeout(bound)); return said; }
/// ```
fn read_lines(lines: &Receiver<String>, wanted: usize, bound: Duration) -> Vec<String> {
    let mut said: Vec<String> = Vec::new();
    while said.len() < wanted {
        let Ok(line) = lines.recv_timeout(bound) else { break };
        said.push(line);
    }
    return said;
}

/// What:
///  What stands published for the notice now,
/// or nothing.
/// Why:
///   The add's own step recording is what a control of the publication reads,
/// so the one fact it needs is read through the same door the notice uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// insideAnAdd() // [step, began] | undefined
/// ```
fn published() -> Option<(Option<String>, Instant)> {
    return inside_an_add();
}

/// A step the add entered and left is published as that step,
/// then as the step that encloses it,
/// and a step entered with no recording publishes nothing.
///
/// This is the control that the line names what the add is doing:
/// before the add's timing published its steps,
/// a waiting add had nothing to name.
#[test]
fn the_add_s_own_steps_are_what_the_notice_names() {
    let root: PathBuf = crate::test_support::fixture("add-standing-published");
    add_timing::begin(root.join("common"), false);
    the_add_ended();
    assert_eq!(published(), None, "no step is published before one is entered");
    let outer: bool = add_timing::begin_step("copyIgnored");
    let inner: bool = add_timing::begin_step("git ls-files");
    assert_eq!(published().map(|(step, _began)| { return step; }), Some(Some(String::from("git ls-files"))), "the innermost step is the one named");
    add_timing::end_step(inner);
    assert_eq!(published().map(|(step, _began)| { return step; }), Some(Some(String::from("copyIgnored"))), "leaving a step names the one that encloses it");
    add_timing::end_step(outer);
    assert_eq!(published().map(|(step, _began)| { return step; }), Some(None), "between two steps nothing is named");
    add_timing::finish(0);
    assert_eq!(published(), None, "an add that answered publishes no step");
    crate::test_support::remove(root.as_path());
}

/// A notice is printed once the add has been in a step longer than the wait,
/// and it stops when the add's notice goes out of scope.
///
/// The wait here is a few milliseconds instead of the shipped seconds,
/// so the control's bound is the machine's speed
/// rather than a person's patience.
#[test]
fn a_notice_is_printed_once_the_wait_is_over_and_stops_with_its_value() {
    let root: PathBuf = crate::test_support::fixture("add-standing-printed");
    add_timing::begin(root.join("common"), false);
    let standing: bool = add_timing::begin_step("gitAdd");
    let (announced, heard): (std::sync::mpsc::Sender<String>, Receiver<String>) = channel();
    let notice: StandingNotice = begin_after(
        Box::new(move |line: String| {
            let _sent = announced.send(line);
        }),
        Duration::from_millis(1),
    );
    let said: Vec<String> = read_lines(&heard, 1, Duration::from_secs(10));
    assert_eq!(said.len(), 1, "one line arrives once the wait is over");
    assert!(said[0].starts_with("monochromatic-git-policy-cli: `git worktree add` is still running after "), "{}", said[0]);
    assert!(said[0].contains("it is in Git's own `git worktree add`"), "the line names what the add is doing: {}", said[0]);
    drop(notice);
    add_timing::end_step(standing);
    add_timing::finish(0);
    crate::test_support::remove(root.as_path());
}

/// The line says how long the add has run and what it is doing,
/// in words a person reads rather than the timing record's step names.
#[test]
fn the_line_counts_the_seconds_and_says_the_step_in_words() {
    let line: String = notice_line(Some("snapshotIoctl"), Duration::from_secs(7));
    assert!(line.starts_with("monochromatic-git-policy-cli: `git worktree add` is still running after 7 seconds; "), "{line}");
    assert!(line.contains("it is taking the snapshot of the main checkout"), "the step is said in the person's words: {line}");
    assert!(!line.contains("snapshotIoctl"), "the timing record's name for the step is not the person's: {line}");
    assert!(line.contains("Nothing here waits for another command to finish"), "the line says what the wait is not: {line}");
    assert!(notice_line(None, Duration::from_secs(9)).contains("it is between two of its steps"), "a step that has just ended is named as such");
    assert_eq!(step_sentence(Some("a step of a later release")), "the add is working in a step its timing record names", "a step this release does not know is still named as work");
}

/// The wait a control sets is the shipped one unless a positive number of seconds is set.
#[test]
fn the_notice_takes_a_positive_setting_and_the_shipped_wait_otherwise() {
    assert_eq!(crate::wait_ceilings::notice_after_from(Some("30"), ADD_STANDING_NOTICE_AFTER), Duration::from_secs(30));
    for meaningless in [None, Some(""), Some("abc"), Some("0"), Some("-5"), Some("1.5")] {
        assert_eq!(crate::wait_ceilings::notice_after_from(meaningless, ADD_STANDING_NOTICE_AFTER), ADD_STANDING_NOTICE_AFTER, "{meaningless:?}");
    }
    assert_eq!(crate::wait_ceilings::ADD_STANDING_NOTICE_VARIABLE, "CLI_GIT_ADD_NOTICE_SECONDS");
}

/// An add whose person asked for `--quiet` keeps the notice and prints nothing,
/// where the same add without it prints the line.
///
/// Both notices run in one process with a wait of one tick,
/// so the difference the control reads is the quietness and not the wait.
#[test]
fn a_quiet_add_prints_no_line_and_a_loud_one_does() {
    let root: PathBuf = crate::test_support::fixture("add-standing-quiet");
    add_timing::begin(root.join("common"), false);
    let standing: bool = add_timing::begin_step("gitAdd");
    let (quiet_lines, quiet_heard): (std::sync::mpsc::Sender<String>, Receiver<String>) = channel();
    let quiet: StandingNotice = begin_quiet_after(
        true,
        Box::new(move |line: String| {
            let _sent = quiet_lines.send(line);
        }),
        Duration::from_millis(1),
    );
    assert_eq!(read_lines(&quiet_heard, 1, Duration::from_millis(500)), Vec::<String>::new(), "a quiet add prints no line");
    drop(quiet);
    let (loud_lines, loud_heard): (std::sync::mpsc::Sender<String>, Receiver<String>) = channel();
    let loud: StandingNotice = begin_quiet_after(
        false,
        Box::new(move |line: String| {
            let _sent = loud_lines.send(line);
        }),
        Duration::from_millis(1),
    );
    let said: Vec<String> = read_lines(&loud_heard, 1, Duration::from_secs(10));
    assert_eq!(said.len(), 1, "the same add without `--quiet` prints the line: {said:?}");
    drop(loud);
    add_timing::end_step(standing);
    add_timing::finish(0);
    crate::test_support::remove(root.as_path());
}
/// What:
///  Standard error as a pipe whose reader takes nothing,
/// with `room` bytes left for this process to write.
/// Why:
///  A reader that stops draining is the case the notice must survive,
/// and the room is what tells a write that fits from one that does not:
/// the pipe accepts `room` bytes and refuses a longer write without writing any of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pipe = fullPipeButFor(room); // the room sits in the one page a write may merge into
/// ```
struct StandardErrorWithRoom {
    /// The pipe's read end,
    /// kept open and never read,
    /// so writes stop at the room left.
    _read_end: std::os::fd::OwnedFd,
}

/// What:
///  The descriptor standard error was,
/// put back when the control ends.
/// Why:
///  The control makes descriptor 2 a pipe,
/// and every later write to it in this process,
/// the test harness's included,
///  would go into that pipe instead,
/// so it is put back whichever way the control ends,
/// a panic included.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RestoredStandardError { drop() { dup2(saved, 2); close(saved); } }
/// ```
struct RestoredStandardError(std::os::fd::RawFd);

impl Drop for RestoredStandardError {
    fn drop(&mut self) {
        // SAFETY: `self.0` is the duplicate of descriptor 2 this control made, and nothing else closes it.
        unsafe {
            libc::dup2(self.0, 2);
            libc::close(self.0);
        }
    }
}

/// The pipe's own capacity, in bytes.
fn pipe_capacity(descriptor: std::os::fd::RawFd) -> usize {
    // SAFETY: `F_GETPIPE_SZ` reads the pipe's capacity and writes nothing into this process.
    let size: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_GETPIPE_SZ) };
    return usize::try_from(size).unwrap_or_default();
}

/// The descriptor's file status flags.
fn descriptor_flags(descriptor: std::os::fd::RawFd) -> libc::c_int {
    // SAFETY: `F_GETFL` reads the descriptor's flags and writes nothing into this process.
    return unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
}

/// Whether a write to the descriptor waits for room.
fn wait_for_room(descriptor: std::os::fd::RawFd, waiting: bool) {
    let flags: libc::c_int = descriptor_flags(descriptor);
    let wanted: libc::c_int = if waiting { flags & !libc::O_NONBLOCK } else { flags | libc::O_NONBLOCK };
    // SAFETY: `F_SETFL` sets the flags of an open descriptor of this process.
    assert_eq!(unsafe { libc::fcntl(descriptor, libc::F_SETFL, wanted) }, 0, "the descriptor's flags");
}

/// How many bytes one write took,
/// or nothing when it was refused for want of room.
fn one_write(descriptor: std::os::fd::RawFd, bytes: &[u8]) -> usize {
    // SAFETY: `bytes` is read for its own length, and the descriptor is open.
    let written: libc::ssize_t = unsafe { libc::write(descriptor, bytes.as_ptr().cast(), bytes.len()) };
    return usize::try_from(written).unwrap_or_default();
}

/// How many bytes one read took.
fn one_read(descriptor: std::os::fd::RawFd, into: &mut [u8]) -> usize {
    // SAFETY: `into` is written for its own length, and the descriptor is open.
    let read: libc::ssize_t = unsafe { libc::read(descriptor, into.as_mut_ptr().cast(), into.len()) };
    return usize::try_from(read).unwrap_or_default();
}

/// What:
///  Make descriptor 2 a pipe with `room` bytes free and no reader that takes any,
/// answering the value that holds the pipe open and the guard that puts descriptor 2 back.
/// Why:
///  The notice writes to descriptor 2,
/// so the pipe is where a stuck reader lives,
/// and the room left is the
///  whole difference between a write that fits and one that waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fullPipeButFor = (room) => { const pipe = os.pipe(); fill(pipe.write, capacity); read(pipe.read, 4096); write(pipe.write, bufferOf(4096 - room)); };
/// ```
fn standard_error_with_room(room: usize) -> (StandardErrorWithRoom, RestoredStandardError) {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    let mut ends: [libc::c_int; 2] = [0; 2];
    // SAFETY: `ends` has room for the two descriptors `pipe` writes, and nothing else touches it.
    assert_eq!(unsafe { libc::pipe(ends.as_mut_ptr()) }, 0, "a pipe to stand in for standard error");
    // SAFETY: both descriptors are the ones `pipe2` just made, and neither number is used again.
    let (read_end, write_end): (OwnedFd, OwnedFd) = unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) };
    let capacity: usize = pipe_capacity(write_end.as_raw_fd());
    assert!(room < capacity, "the room {room} must be less than the pipe's {capacity}");
    wait_for_room(write_end.as_raw_fd(), false);
    // A whole buffer at a time until the pipe refuses one: it is then as full as it can be.
    let block: [u8; 4096] = [b'p'; 4096];
    while one_write(write_end.as_raw_fd(), block.as_slice()) == block.len() {}
    // One whole buffer out of the head, then all but `room` of it back into the tail, so the free room
    // is one page a shorter write merges into and a longer write is refused rather than partly written.
    let mut taken: [u8; 4096] = [0; 4096];
    assert_eq!(one_read(read_end.as_raw_fd(), taken.as_mut_slice()), taken.len(), "one whole buffer leaves the pipe");
    let refill: usize = taken.len() - room;
    if refill > 0 {
        assert_eq!(one_write(write_end.as_raw_fd(), &block[..refill]), refill, "the room left in the pipe");
    }
    wait_for_room(write_end.as_raw_fd(), true);
    // SAFETY: duplicating descriptor 2 has no preconditions, and the guard puts it back.
    let saved: std::os::fd::RawFd = unsafe { libc::dup(2) };
    assert!(saved >= 0, "descriptor 2 is open");
    let restored: RestoredStandardError = RestoredStandardError(saved);
    // SAFETY: both descriptors are open, and the guard restores descriptor 2 whichever way this ends.
    assert_eq!(unsafe { libc::dup2(write_end.as_raw_fd(), 2) }, 2, "descriptor 2 is the pipe");
    return (StandardErrorWithRoom { _read_end: read_end }, restored);
}

/// How long one write of a line longer than the room a pipe has left takes to be refused.
/// Why:
///  The notice's thread is given this long to reach its write,
/// so the control asks about a write that
///  is under way rather than one it hopes has started.
/// A refused write returns nothing to wait for,
/// so the only
///  time spent is the thread's own start.
const TIME_FOR_ONE_REFUSED_WRITE: Duration = Duration::from_millis(200);

/// What:
///  A notice stuck in its own write holds no lock the add needs.
/// Why:
///  `std::io::stderr()` is one handle behind one lock for the whole process,
/// and a write through it
///  holds that lock for as long as the write takes.
///  The notice is printed on a thread of its own,
///  and a reader that stopped draining leaves its write waiting,
/// so a notice written through the shared
///  handle would hold the lock the add's own reports need,
/// and the add would wait behind its own notice
///  with room in the pipe for what it has to say.
/// The owner's rule is "nothing should ever wait forever or
///  indefinitely",
/// and the notice's own comment says it must not hold the add's stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const room = noticeLine.length - 1; fullPipeButFor(room); const notice = begin(writeToStandardError);
/// expect(addWritesOneByteWithin(5_000)).toBe('wrote'); // the notice's line is still stuck
/// ```
#[test]
fn a_notice_stuck_in_its_write_holds_no_lock_the_add_needs() {
    use std::sync::mpsc::Sender;
    use std::time::Instant;
    let line: String = notice_line(Some("gitAdd"), ADD_STANDING_NOTICE_AFTER);
    // One byte less than the notice's own line, so the notice's write cannot fit and a shorter one can.
    let room: usize = line.len() + "\n".len() - 1;
    let (pipe, restored): (StandardErrorWithRoom, RestoredStandardError) = standard_error_with_room(room);
    a_step_began(Some("gitAdd"), Instant::now());
    let (began, beginning): (Sender<()>, Receiver<()>) = channel();
    let sink = Box::new(move |line: String| {
        // The sink says its write is about to start, so the control knows the notice is in it rather
        // than hoping a sleep was long enough.
        let _sent = began.send(());
        write_to_standard_error(line);
    });
    let notice: StandingNotice = begin_quiet_after(false, sink, Duration::from_millis(1));
    assert!(beginning.recv_timeout(Duration::from_secs(10)).is_ok(), "the notice began its own write");
    // One write of a line longer than the room cannot return, so by this sleep the notice's thread is
    // inside its write rather than about to make it.
    std::thread::sleep(TIME_FOR_ONE_REFUSED_WRITE);
    // The add's own report, written after the notice is inside a write the pipe refuses.
    let (written, wrote): (Sender<std::io::Result<usize>>, Receiver<std::io::Result<usize>>) = channel();
    let add: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let _sent = written.send(std::io::Write::write(&mut std::io::stderr(), b"x"));
    });
    let answer: Option<std::io::Result<usize>> = wrote.recv_timeout(Duration::from_secs(5)).ok();
    // Closing the read end releases the notice's write, and with it any lock the add's write waits on.
    drop(pipe);
    add.join().expect("the add's thread");
    drop(notice);
    the_add_ended();
    // Descriptor 2 is the test harness's again before the control answers, so its own words reach the
    // reader that would otherwise be the pipe this control filled.
    drop(restored);
    assert_eq!(answer.map(|written| { return written.ok(); }), Some(Some(b"x".len())), "the add's own byte is written while the notice's line waits in the pipe");
}
