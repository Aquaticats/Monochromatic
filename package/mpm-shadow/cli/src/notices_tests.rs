//! What:
//! The controls of the one writer every word this crate says goes through,
//! and of the rule that no word leaves through a macro that aborts the process.
//! Why:
//! `0.2.7-alpha.20` ended a whole install at a notice a full standard error refused,
//! and `0.2.7-alpha.21` still ended one at a report a refusing standard output met between two of
//! the command's own steps;
//! the writer is only worth having if a control shows it survives each answer a stream gives,
//! if a control shows that a stream answering "not now" is waited out,
//! and if a control shows that no future change puts a `println!` back on a path that runs on this
//! machine.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // notices-test.ts
//! test('a write a full non-blocking stream refuses is made when the reader drains');
//! test('the line end goes out with its line and is not dropped alone');
//! test('a write no reader takes gives up within the bound');
//! test('every error kind a stream answers returns without ending the process');
//! test('an interrupted or short write carries the rest of the words');
//! test('every word this crate says leaves through this module');
//! ```

/// Imports `File` from `std::fs`,
/// the owned handle a pipe's end becomes,
/// so the crate's own `Write` implementation writes into it.
/// Why:
/// The controls write into a real pipe, which is the stream the finding was measured on,
/// and `File` is how a pair of descriptors from `rustix` becomes a stream Rust can write to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No import: `fs.write(fd, bytes)` takes the number directly.
/// ```
use std::fs::File;
/// Imports `ErrorKind`, `Read` and `Write` from `std::io`.
/// Why:
/// The reader threads read the pipe, and the arms of the controls tell a refusal that a wait cures
/// from one that it does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No import: `process.stderr.write` and `fs.readSync` are on the global object.
/// ```
use std::io::{
    ErrorKind,
    Read,
    Write,
};
/// Imports `Path` and `PathBuf` from `std::path`.
/// Why:
/// The source scan of the last control walks the crate's own `src` folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // path.join and path.dirname.
/// ```
use std::path::{
    Path,
    PathBuf,
};
/// Imports `Duration` and `Instant` from `std::time`.
/// Why:
/// A reader thread gives up after a deadline so a control that would never see its bytes fails
/// instead of hanging,
/// and the bound's own control measures how long a write took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now().
/// ```
use std::time::{
    Duration,
    Instant,
};

/// Imports `OFlags` and `fcntl_setfl` from `rustix::fs`,
/// the safe wrapper that leaves a descriptor non-blocking.
/// Why:
/// The state the finding is about is a stream another process left non-blocking,
/// and the crate already calls `rustix` for its other descriptor work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: Node's streams are non-blocking already.
/// ```
use rustix::fs::{
    OFlags,
    fcntl_setfl,
};
/// Imports `fcntl_setpipe_size` from `rustix::pipe`,
/// which makes a pipe hold one page and no more.
/// Why:
/// The one loss the read measured that no filling can show is a pipe with room for a line and not its line
/// end,
/// and that state needs a pipe small enough that the room a reader makes is room the next write can use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent in the standard library.
/// ```
use rustix::pipe::fcntl_setpipe_size;
/// Imports `pipe` from `rustix::pipe`,
/// which makes the two descriptors of one pipe.
/// Why:
/// A real pipe is what refuses a write that does not fit in one piece,
/// which a socket pair does not do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent in the standard library.
/// ```
use rustix::pipe::pipe;

/// Imports the bound under test,
/// and the writer itself.
/// Why:
/// The controls hold what the shipped code does, so they call it and read its own constant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { A_WAIT_FOR_A_STREAM_TO_TAKE_THE_BYTES, writeTheWords } from './notices.ts';
/// ```
use super::{
    A_WAIT_FOR_A_STREAM_TO_TAKE_THE_BYTES,
    write_the_words,
};

/// What:
/// The pause between two reads of a control's reader thread.
/// Why:
/// A non-blocking read answers "not now" in the microseconds before the writer's bytes arrive,
/// and a reader that slept a whole pause per attempt would make every control slower than the wait
/// it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_PAUSE_BETWEEN_TWO_READS = 1;
/// ```
const A_PAUSE_BETWEEN_TWO_READS: Duration = Duration::from_millis(1);

/// What:
/// The deadline a control's reader thread stops reading at.
/// Why:
/// A control whose writer dropped its bytes would wait for them for ever,
/// and a hung control says nothing;
/// 5 s is far above the 1 s bound under test and the 100 ms a reader waits before it drains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_READERS_DEADLINE = 5000;
/// ```
const A_READERS_DEADLINE: Duration = Duration::from_secs(5);

/// What:
/// The size the line-and-its-line-end control gives its pipe: one page,
/// which is the smallest a pipe may be set to.
/// Why:
/// With one page the room a reader makes at the tail is room in the same page a write merges into,
/// so the control can put the pipe in the state where the line fits and its line end does not.
/// A larger pipe keeps its room in a different page than its head,
/// which no partial drain reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_ONE_PAGE_PIPE = 4096;
/// ```
const A_ONE_PAGE_PIPE: usize = 4096;

/// What:
/// `fn a_one_page_pipe_with_both_ends_non_blocking() -> (File, File)` makes one pipe of a single page whose
/// two ends are non-blocking,
/// answering the reading end and then the writing end.
/// Why:
/// The line-end control needs a pipe whose room is reachable by draining a few bytes at the tail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aOnePagePipeWithBothEndsNonBlocking(): [File, File]
/// ```
fn a_one_page_pipe_with_both_ends_non_blocking() -> (File, File) {
    // What: the two descriptors of one pipe.
    // Why: both ends belong to this control.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [reading, writing] = pipe();
    // ```
    let (reading, writing) = pipe().expect("make a pipe");
    // What: the size the kernel gave the pipe, which has to be the one page this control asked for.
    // Why: a larger pipe keeps the room a partial drain makes away from the page a write merges into.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const size = setPipeSize(writing, A_ONE_PAGE_PIPE);
    // ```
    let size: usize = fcntl_setpipe_size(&writing, A_ONE_PAGE_PIPE)
        .expect("the kernel sets a pipe's size to one page");
    assert_eq!(
        size, A_ONE_PAGE_PIPE,
        "the pipe a control gave one page to holds one page"
    );
    // What: leaves the reading end non-blocking.
    // Why: the reader polls, so an empty pipe must answer rather than hold it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setNonBlocking(reading);
    // ```
    fcntl_setfl(&reading, OFlags::NONBLOCK).expect("leave the reading end non-blocking");
    // What: leaves the writing end non-blocking, which is the state the finding is about.
    // Why: a blocking writing end would wait for room instead of answering "not now".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setNonBlocking(writing);
    // ```
    fcntl_setfl(&writing, OFlags::NONBLOCK).expect("leave the writing end non-blocking");
    // What: the two descriptors as files.
    // Why: `File` implements `Write`, which the writer takes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [reading, writing];
    // ```
    return (File::from(reading), File::from(writing));
}

/// What:
/// `fn a_pipe_with_both_ends_non_blocking() -> (File, File)` makes one pipe whose two ends are
/// non-blocking,
/// answering the reading end and then the writing end.
/// Why:
/// The state under test is a stream another process left non-blocking,
/// and a reader that could block would hang the control rather than fail it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aPipeWithBothEndsNonBlocking(): [File, File]
/// ```
fn a_pipe_with_both_ends_non_blocking() -> (File, File) {
    // What: the two descriptors of one pipe: the reading end and the writing end.
    // Why: both ends belong to this control, so nothing the writer does can reach another process.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [reading, writing] = pipe();
    // ```
    let (reading, writing) = pipe().expect("make a pipe");
    // What: leaves the reading end non-blocking. `&reading` lends the descriptor without giving it up.
    // Why: a reader thread that polls must not block when the pipe is empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setNonBlocking(reading);
    // ```
    fcntl_setfl(&reading, OFlags::NONBLOCK).expect("leave the reading end non-blocking");
    // What: leaves the writing end non-blocking, which is the state the finding is about.
    // Why: a blocking writing end would wait for room instead of answering "not now".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // setNonBlocking(writing);
    // ```
    fcntl_setfl(&writing, OFlags::NONBLOCK).expect("leave the writing end non-blocking");
    // What: the two descriptors as files, which is how the crate writes into a stream it did not open.
    // Why: `File` implements `Write`, which the writer takes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [reading, writing];
    // ```
    return (File::from(reading), File::from(writing));
}

/// What:
/// `fn filled_until_it_refuses(writing: &mut File) -> Vec<u8>` writes filler into the pipe until the
/// pipe answers "not now",
/// and answers the filler it took.
/// Why:
/// The control needs the pipe at its capacity, and how many bytes that is belongs to the kernel
/// (64 KiB on this machine), so the filler's length is the measurement rather than a number written
/// here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function filledUntilItRefuses(writing: File): Uint8Array
/// ```
fn filled_until_it_refuses(writing: &mut File) -> Vec<u8> {
    // What: one chunk of filler, the same byte repeated.
    // Why: the filler only has to be recognisable, and one byte per position keeps the length exact.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const chunk = new Uint8Array(4096).fill(0x66);
    // ```
    let chunk: [u8; 4096] = [b'f'; 4096];
    // What: the filler the pipe has taken so far.
    // Why: its length is what the reader has to read before the notice's bytes arrive.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const filler: number[] = [];
    // ```
    let mut filler: Vec<u8> = Vec::new();
    loop {
        // What: one attempt to add a chunk, which takes a prefix while the pipe fills.
        // Why: the same call answers how much room was left.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const answer = writing.write(chunk);
        // ```
        let attempt = writing.write(&chunk);
        match attempt {
            // What: the whole chunk went in, so the loop asks for another.
            // Why: a pipe that took everything is not full yet.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (answer > 0) { filler.push(...chunk.slice(0, answer)); continue; }
            // ```
            Ok(count) if count > 0 => filler.extend_from_slice(&chunk[..count]),
            // What: the pipe is full, which is the state the control wants.
            // Why: the loop's one way out.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return filler;
            // ```
            Err(error) if error.kind() == ErrorKind::WouldBlock => return filler,
            // What: any other answer, which a pipe of this control's own never gives.
            // Why: a control that cannot fill its pipe has to say so rather than read a wrong state.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // throw error;
            // ```
            Ok(_) => panic!("the pipe took nothing and refused nothing part-way"),
            Err(error) => panic!("the pipe refused the filler for good: {error}"),
        }
    }
}

/// What:
/// `fn wrote_exactly(writing: &mut File, count: usize) -> Vec<u8>` writes exactly `count` filler bytes
/// into a pipe that has room for them,
/// and answers what it wrote.
/// Why:
/// The line-end control's state is one exact amount of free room,
/// and filling a pipe until it refuses gives none of it;
/// this helper leaves the room the control chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wroteExactly(writing: File, count: number): Uint8Array
/// ```
fn wrote_exactly(
    writing: &mut File,
    count: usize,
) -> Vec<u8> {
    // What: one chunk of filler.
    // Why: a chunk smaller than a page lets the loop stop at the exact count.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const chunk = new Uint8Array(1024).fill(0x66);
    // ```
    let chunk: [u8; 1024] = [b'f'; 1024];
    // What: the filler written so far.
    // Why: the loop stops at the count the control asked for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const filler: number[] = [];
    // ```
    let mut filler: Vec<u8> = Vec::with_capacity(count);
    while filler.len() < count {
        // What: how much of the next chunk to write, which is the rest of the count.
        // Why: the last write stops exactly at the count.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const room = Math.min(count - filler.length, chunk.length);
        // ```
        let room: usize = (count - filler.len()).min(chunk.len());
        // What: one write of that part.
        // Why: a one-page pipe with room takes it whole.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const answer = writing.write(chunk.slice(0, room));
        // ```
        let attempt = writing.write(&chunk[..room]);
        match attempt {
            // What: the bytes this write took.
            // Why: they are the next part of the filler.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (answer > 0) { filler.push(...chunk.slice(0, answer)); continue; }
            // ```
            Ok(taken) if taken > 0 => filler.extend_from_slice(&chunk[..taken]),
            // What: a pipe that took nothing while the control still had room for it.
            // Why: a control that cannot reach its state has to say so rather than measure a wrong one.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // throw new Error('the pipe took nothing');
            // ```
            Ok(_) => panic!("the pipe took nothing while the control still had room for filler"),
            // What: a pipe that refused filler before the control reached its count.
            // Why: the state the control wants is not the one it measured.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // throw error;
            // ```
            Err(error) => panic!("the pipe refused filler before the control reached its count: {error}"),
        }
    }
    // What: the filler, whose length is the count the control asked for.
    // Why: the room left in the pipe is the page's size minus this length.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return filler;
    // ```
    return filler;
}

/// What:
/// `fn everything_the_pipe_holds(reading: &mut File) -> Vec<u8>` reads until the pipe answers
/// "not now" and answers what it took.
/// Why:
/// A control that wants what is in the pipe at this instant must not wait for bytes nobody is going to
/// write,
/// and a reader's own deadline would make it wait for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function everythingThePipeHolds(reading: File): Uint8Array
/// ```
fn everything_the_pipe_holds(reading: &mut File) -> Vec<u8> {
    // What: the bytes read so far.
    // Why: the loop adds one read's worth at a time until the pipe is empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const seen: number[] = [];
    // ```
    let mut seen: Vec<u8> = Vec::new();
    loop {
        // What: the room the next read fills, one pipe's worth at a time.
        // Why: a fixed chunk keeps the loop simple and never asks the kernel for more than a pipe holds.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const room = new Uint8Array(65536);
        // ```
        let mut room: [u8; 65536] = [0u8; 65536];
        // What: one read.
        // Why: "not now" is the loop's one way out.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const answer = reading.read(room);
        // ```
        let attempt = reading.read(&mut room);
        match attempt {
            // What: the bytes this read took.
            // Why: they are the next part of the pipe's content.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (answer > 0) { seen.push(...room.slice(0, answer)); continue; }
            // ```
            Ok(count) if count > 0 => seen.extend_from_slice(&room[..count]),
            // What: the pipe is empty, which is as much as this control can read now.
            // Why: the loop's one way out.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return seen;
            // ```
            Err(error) if error.kind() == ErrorKind::WouldBlock => return seen,
            // What: a signal arrived before the read was made.
            // Why: nothing was taken, so the same read is made again.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (isInterrupted) continue;
            // ```
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            // What: the writing end is gone, so the pipe holds nothing more.
            // Why: what was read is the whole answer.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return seen;
            // ```
            Ok(_) => return seen,
            Err(error) => panic!("the pipe refused the read for good: {error}"),
        }
    }
}

/// What:
/// `fn read_until(reading: &mut File, wanted: usize) -> Vec<u8>` reads the pipe until it has
/// `wanted` bytes or the reader's deadline passes,
/// and answers what it read.
/// Why:
/// A control whose writer dropped its bytes must fail on a short answer rather than wait for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readUntil(reading: File, wanted: number): Uint8Array
/// ```
fn read_until(
    reading: &mut File,
    wanted: usize,
) -> Vec<u8> {
    // What: the bytes read so far, and the instant the reading stops.
    // Why: the deadline is what keeps a lost notice from hanging the control.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const seen: number[] = []; const deadline = Date.now() + A_READERS_DEADLINE;
    // ```
    let mut seen: Vec<u8> = Vec::with_capacity(wanted);
    let deadline: Instant = Instant::now() + A_READERS_DEADLINE;
    while seen.len() < wanted && Instant::now() < deadline {
        // What: where the next read puts its bytes: the room the answer has not filled yet.
        // Why: reading into the same buffer from its end keeps the order the pipe gave.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const room = new Uint8Array(wanted - seen.length);
        // ```
        let mut room: Vec<u8> = vec![0u8; wanted - seen.len()];
        // What: one read, which takes what the pipe holds up to the room given.
        // Why: a non-blocking pipe answers "not now" while it is empty.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const answer = reading.read(room);
        // ```
        let attempt = reading.read(&mut room);
        match attempt {
            // What: the bytes this read took.
            // Why: they are the next part of the pipe's content.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (answer > 0) { seen.push(...room.slice(0, answer)); continue; }
            // ```
            Ok(count) if count > 0 => seen.extend_from_slice(&room[..count]),
            // What: nothing is in the pipe yet.
            // Why: the writer may still be waiting for its own attempt, so the loop waits a little.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (isEmpty) { sleep(A_PAUSE_BETWEEN_TWO_READS); continue; }
            // ```
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(A_PAUSE_BETWEEN_TWO_READS);
            }
            // What: a signal arrived before the read was made.
            // Why: nothing was taken, so the same read is made again.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (isInterrupted) continue;
            // ```
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            // What: the writing end is gone, so the pipe can hold nothing more.
            // Why: the control reports what it read, and its assertion says what is missing.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // break;
            // ```
            Ok(_) => break,
            Err(error) => panic!("the pipe refused the read for good: {error}"),
        }
    }
    // What: what the pipe gave this control before its deadline, which may be short.
    // Why: a short answer is the failure a lost notice shows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return seen;
    // ```
    return seen;
}

/// What:
/// A write a full non-blocking pipe refuses is made once a reader drains it.
/// Why:
/// The second-pass read of `0.2.7-alpha.21` measured exactly this state on a real pipe:
/// the notice never arrived and the command exited 0,
/// because the writer dropped the `EAGAIN` a reader would have cured 100 ms later.
/// `EAGAIN` says "not now" and not "never", so the writer waits for room within its bound.
#[test]
fn a_write_a_full_non_blocking_stream_refuses_is_made_when_the_reader_drains() {
    // What: the pipe, full to its capacity.
    // Why: the notice cannot go in until the reader takes filler out.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [reading, writing] = aPipeWithBothEndsNonBlocking(); const filler = filledUntilItRefuses(writing);
    // ```
    let (mut reading, mut writing) = a_pipe_with_both_ends_non_blocking();
    let filler: Vec<u8> = filled_until_it_refuses(&mut writing);
    // What: the line the writer is given, with its line end.
    // Why: it has to be recognised at the end of what the reader takes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const notice = 'mpm-shadow: a line a full pipe takes once the reader drains\n';
    // ```
    let notice: &str = "mpm-shadow: a line a full pipe takes once the reader drains\n";
    // What: how many bytes the reader has to take for the whole pipe's content to be out.
    // Why: an exact count lets the reader stop at the notice's last byte.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const expected = filler.length + notice.length;
    // ```
    let expected: usize = filler.len() + notice.len();
    // What: the reader, which waits past the writer's first refusal and then drains everything.
    // Why: this is the reader the read measured as alive and slow, which the dropped `EAGAIN` lost the
    //  notice of.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const reader = new Thread(() => { sleep(100); return readUntil(reading, expected); });
    // ```
    let reader = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        return read_until(&mut reading, expected);
    });
    // What: the write under test, on the pipe that has no room yet.
    // Why: the writer waits, the reader drains, and the notice arrives.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeTheWords(writing, Buffer.from(notice));
    // ```
    write_the_words(&mut writing, notice.as_bytes());
    // What: what the reader thread took.
    // Why: its tail is the notice, if the writer waited for room.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const seen = reader.join();
    // ```
    let seen: Vec<u8> = reader.join().expect("the reader thread ended");
    assert_eq!(
        seen.len(),
        expected,
        "the reader took the filler and the whole line after it, not the filler alone"
    );
    assert_eq!(
        &seen[filler.len()..],
        notice.as_bytes(),
        "the bytes after the filler are the line with its line end"
    );
}

/// What:
/// A line is never written without its line end.
/// Why:
/// `0.2.7-alpha.21` wrote a line and its newline as two calls,
/// so a pipe with room for the line took the line and refused the newline,
/// and the newline was dropped:
/// the next line written then joined the line whose end was lost,
/// which corrupts two lines where one was refused.
/// One write carries both, so a stream that cannot take the line with its end takes neither,
/// and what a person reads is never a line without its end.
/// The two arms below are one pipe state apart, by one byte of room:
/// the room for the line alone shows that neither the line nor its end goes in,
/// and the room for both shows that this writer does deliver them together,
/// so the first arm cannot be passed by a writer that writes nothing at all.
#[test]
fn a_line_is_never_written_without_its_line_end() {
    // What: a line long enough that the room made for it is easy to hit exactly.
    // Why: the pipe's answer differs by one byte, and that byte is the line end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const line = 'y'.repeat(200);
    // ```
    let line: String = "y".repeat(200);
    // What: the two rooms the arms give the writer, one byte apart.
    // Why: the first fits the line alone and the second fits the line with its end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rooms = [line.length, line.length + 1];
    // ```
    let rooms: [usize; 2] = [line.len(), line.len() + 1];
    for room in rooms {
        // What: a one-page pipe holding as much filler as leaves `room` bytes free in its one page.
        // Why: a page's free end is where a write merges, so the filler's length is what decides the room,
        //  and a one-page pipe is the smallest state that has such a free end.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const [reading, writing] = aOnePagePipeWithBothEndsNonBlocking();
        // const filler = wroteExactly(writing, A_ONE_PAGE_PIPE - room);
        // ```
        let (mut reading, mut writing) = a_one_page_pipe_with_both_ends_non_blocking();
        let filler: Vec<u8> = wrote_exactly(&mut writing, A_ONE_PAGE_PIPE - room);
        // What: the write of the line and its line end as one text, the way `say` makes it.
        // Why: this is the call whose one write cannot lose the line end alone.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // writeTheWords(writing, Buffer.from(`${line}\n`));
        // ```
        write_the_words(&mut writing, format!("{line}\n").as_bytes());
        // What: what the pipe holds after the write.
        // Why: the arms' whole difference is what this holds.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const seen = everythingThePipeHolds(reading);
        // ```
        let seen: Vec<u8> = everything_the_pipe_holds(&mut reading);
        if room == line.len() {
            assert_eq!(
                seen, filler,
                "with room for the line alone, neither the line nor its end went in"
            );
            continue;
        }
        assert_eq!(
            seen.len(),
            filler.len() + line.len() + 1,
            "with room for both, the line and its end are in the pipe"
        );
        assert_eq!(
            &seen[filler.len()..],
            format!("{line}\n").as_bytes(),
            "the line is there with its line end"
        );
    }
}

/// What:
/// A write into a full non-blocking pipe that no reader drains gives up within the bound.
/// Why:
/// The wait is bounded so a reader that stopped reading cannot hold a command that has finished its
/// work;
/// this control holds the end of the wait,
/// and that the dropping it ends with leaves the pipe's content as it was.
#[test]
fn a_write_no_reader_takes_gives_up_within_the_bound() {
    // What: the pipe, full to its capacity, with its reader kept by this control and never reading.
    // Why: this is the reader that stopped reading.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [reading, writing] = aPipeWithBothEndsNonBlocking(); const filler = filledUntilItRefuses(writing);
    // ```
    let (mut reading, mut writing) = a_pipe_with_both_ends_non_blocking();
    let filler: Vec<u8> = filled_until_it_refuses(&mut writing);
    // What: the instant the write is made, so its wait can be measured.
    // Why: the bound is a property of the shipped constant, and the control reads the time it took.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const started = Date.now();
    // ```
    let started: Instant = Instant::now();
    // What: one line the pipe has no room for.
    // Why: its write is the call that has to give up.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeTheWords(writing, Buffer.from('a line nobody reads\n'));
    // ```
    write_the_words(&mut writing, b"a line nobody reads\n");
    // What: how long it waited.
    // Why: shorter than the bound would mean it gave up early, longer would mean the bound is not held.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const waited = Date.now() - started;
    // ```
    let waited: Duration = started.elapsed();
    assert!(
        waited >= A_WAIT_FOR_A_STREAM_TO_TAKE_THE_BYTES,
        "the writer waits for the bound before it drops the bytes, and it waited {waited:?}"
    );
    assert!(
        waited < A_WAIT_FOR_A_STREAM_TO_TAKE_THE_BYTES + Duration::from_secs(2),
        "the writer waits no longer than the bound, and it waited {waited:?}"
    );
    // What: the bound the record promises, as one second.
    // Why: the module and `README.md` say 1 s, so a constant that moved without them would leave every
    //  sentence about this wait wrong while the timings above still agreed with the constant's new value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(A_WAIT_FOR_A_STREAM_TO_TAKE_THE_BYTES).toBe(1000);
    // ```
    assert_eq!(
        A_WAIT_FOR_A_STREAM_TO_TAKE_THE_BYTES,
        Duration::from_millis(1000),
        "the bound the record promises is the one that ships"
    );
    // What: what the pipe holds now, which is everything it held before the refused write.
    // Why: the refused line has to be absent, so the drop left the stream as it found it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const seen = everythingThePipeHolds(reading);
    // ```
    let seen: Vec<u8> = everything_the_pipe_holds(&mut reading);
    assert_eq!(
        seen, filler,
        "the pipe holds the filler alone, so no byte of the refused line went in"
    );
}

/// What:
/// A stream that refuses with one chosen error, which is what a stand-in for a real stream gives.
/// Why:
/// A control cannot make a real stream answer `EIO` or a quota error on demand,
/// and what the writer does with each answer is the rule under test;
/// this stand-in is not a real stream and no claim about a real one rests on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AStreamThatRefuses { write() { throw Object.assign(new Error('planted'), { errno: this.code }); } }
/// ```
struct AStreamThatRefuses {
    /// What: the operating system's number for the error this stream answers with.
    /// Why: each kind is one arm of the writer's rule.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// code: number;
    /// ```
    code: i32,
}

/// The stream's own `Write` implementation: every write answers the chosen error.
impl Write for AStreamThatRefuses {
    /// What: answers the chosen error and takes no bytes.
    /// Why: this is a stream that refuses for good.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// write() { throw errnoError(this.code); }
    /// ```
    fn write(
        &mut self,
        _words: &[u8],
    ) -> std::io::Result<usize> {
        return Err(std::io::Error::from_raw_os_error(self.code));
    }

    /// What: the required `flush`, which has nothing buffered to push out.
    /// Why: `Write` needs it, and this stand-in keeps no buffer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// flush() { return 0; }
    /// ```
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
/// Every error kind a stream can answer with returns from the writer without ending the process.
/// Why:
/// `0.2.7-alpha.21`'s controls covered `ENOSPC` and `EPIPE` alone,
/// so a writer that aborted on any other kind kept them green and brought the defect back for a full
/// quota, an `EIO` or a closed descriptor.
/// The writer drops all of them, which is the rule this holds.
#[test]
fn every_error_kind_a_stream_answers_returns_without_ending_the_process() {
    // What: the error numbers a stream this program writes to can answer with, one per line.
    // Why: each is an arm the writer must survive; the list is the kernel's numbers and not this
    //  control's invention.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const kinds = [EIO, ENOSPC, EPIPE, EDQUOT, EACCES, ENOMEM, EBADF];
    // ```
    let kinds: [i32; 7] = [
        5,   // EIO: an input/output error on the stream
        28,  // ENOSPC: the disk the stream writes to is full
        32,  // EPIPE: the reader is gone
        122, // EDQUOT: the account's disk quota is used up
        13,  // EACCES: the stream is not writable by this process
        12,  // ENOMEM: the kernel had no room for the write
        9,   // EBADF: the descriptor is not open for writing
    ];
    for code in kinds {
        // What: the writer's call with that kind of refusal.
        // Why: returning at all is the assertion, because a panic ends the whole test binary.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // writeTheWords(new AStreamThatRefuses(code), Buffer.from('a line\n'));
        // ```
        let mut refusing = AStreamThatRefuses {
            code,
        };
        write_the_words(&mut refusing, b"a line\n");
    }
}

/// What:
/// A stream that answers "not now" twice and then takes the bytes, and one that takes them a few at a
/// time.
/// Why:
/// These are the two answers that are not refusals,
/// and a writer that treated either as a refusal would lose a line on a stream that was willing to
/// take it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AStreamThatWaitsTwice { write(b) { if (this.refusals-- > 0) throw wouldBlock(); return b.length; } }
/// class AStreamThatTakesBytesSlowly { write(b) { return Math.min(b.length, this.atATime); } }
/// ```
struct AStreamThatWaitsTwice {
    /// What: how many more writes answer "not now".
    /// Why: the writer has to make a third attempt and take the bytes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refusals: number;
    /// ```
    refusals: u32,
    /// What: the words the stream took, in the order it took them.
    /// Why: the control reads them back to see the whole line arrived.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// taken: number[];
    /// ```
    taken: Vec<u8>,
}

/// The waiting stream's own `Write` implementation.
impl Write for AStreamThatWaitsTwice {
    /// What: answers "not now" while it has refusals left and takes the rest after that.
    /// Why: the state a slow reader's pipe is in.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// write(b) { if (this.refusals > 0) { this.refusals--; throw wouldBlock(); } this.taken.push(...b); return b.length; }
    /// ```
    fn write(
        &mut self,
        words: &[u8],
    ) -> std::io::Result<usize> {
        if self.refusals > 0 {
            self.refusals -= 1;
            return Err(std::io::Error::from(ErrorKind::WouldBlock));
        }
        self.taken.extend_from_slice(words);
        return Ok(words.len());
    }

    /// What: the required `flush`, which has nothing buffered.
    /// Why: `Write` needs it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// flush() { return 0; }
    /// ```
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
/// A stream that takes a few bytes per write, and one that takes everything.
/// Why:
/// A short write is a prefix taken, not a refusal, so the rest has to follow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AStreamThatTakesABitAtATime { write(b) { const n = Math.max(1, Math.floor(b.length / 4)); this.taken.push(...b.slice(0, n)); return n; } }
/// ```
struct AStreamThatTakesABitAtATime {
    /// What: the words the stream took, in the order it took them.
    /// Why: the control reads them back to see the whole line arrived.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// taken: number[];
    /// ```
    taken: Vec<u8>,
}

/// The slow stream's own `Write` implementation.
impl Write for AStreamThatTakesABitAtATime {
    /// What: takes a quarter of the bytes, and never fewer than one.
    /// Why: a short answer is what a partial write looks like, and the writer has to carry the rest.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// write(b) { const n = Math.max(1, Math.floor(b.length / 4)); this.taken.push(...b.slice(0, n)); return n; }
    /// ```
    fn write(
        &mut self,
        words: &[u8],
    ) -> std::io::Result<usize> {
        let count: usize = (words.len() / 4).max(1);
        self.taken.extend_from_slice(&words[..count]);
        return Ok(count);
    }

    /// What: the required `flush`, which has nothing buffered.
    /// Why: `Write` needs it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// flush() { return 0; }
    /// ```
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
/// A stream whose first write is interrupted and whose later writes take the bytes.
/// Why:
/// `EINTR` says nothing about the bytes, so the attempt is made again;
/// a writer that dropped it would lose the line to a signal it can see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AStreamThatIsInterruptedOnce { write(b) { if (!this.interrupted) { this.interrupted = true; throw interrupted(); } this.taken.push(...b); return b.length; } }
/// ```
struct AStreamThatIsInterruptedOnce {
    /// What: whether the one interrupt has already been answered.
    /// Why: only the first write is refused, so the retry is what the control sees.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// interrupted: boolean;
    /// ```
    interrupted: bool,
    /// What: the words the stream took.
    /// Why: the control reads them back to see the whole line arrived.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// taken: number[];
    /// ```
    taken: Vec<u8>,
}

/// The interrupted stream's own `Write` implementation.
impl Write for AStreamThatIsInterruptedOnce {
    /// What: refuses the first write with an interrupt and takes every later one.
    /// Why: this is a signal arriving between the bytes and the stream, not a refusal.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// write(b) { if (!this.interrupted) { this.interrupted = true; throw interrupted(); } this.taken.push(...b); return b.length; }
    /// ```
    fn write(
        &mut self,
        words: &[u8],
    ) -> std::io::Result<usize> {
        if !self.interrupted {
            self.interrupted = true;
            return Err(std::io::Error::from(ErrorKind::Interrupted));
        }
        self.taken.extend_from_slice(words);
        return Ok(words.len());
    }

    /// What: the required `flush`, which has nothing buffered.
    /// Why: `Write` needs it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// flush() { return 0; }
    /// ```
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
/// The two answers that are not refusals carry the whole line: a wait for room, and a short write.
/// Why:
/// The read measured `0.2.7-alpha.21` dropping only the answers a retry cures,
/// so the retries are what this control holds;
/// and a loss here would be a line the stream was willing to take.
#[test]
fn a_wait_for_room_and_a_short_write_both_carry_the_whole_line() {
    // What: the line every arm of this control writes.
    // Why: it has to be read back whole.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const line = 'mpm-shadow: one line\n';
    // ```
    let line: &str = "mpm-shadow: one line\n";
    // What: the stream that refuses twice and then takes the bytes.
    // Why: this is a full pipe whose reader catches up.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const waiting = { refusals: 2, taken: [] };
    // ```
    let mut waiting = AStreamThatWaitsTwice {
        refusals: 2,
        taken: Vec::new(),
    };
    // What: the write under test.
    // Why: it has to end with the whole line taken.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeTheWords(waiting, Buffer.from(line));
    // ```
    write_the_words(&mut waiting, line.as_bytes());
    assert_eq!(
        waiting.taken,
        line.as_bytes(),
        "the line arrived whole after two refusals that a wait cures"
    );
    // What: the stream that takes a quarter of the bytes per write.
    // Why: this is a short write, which the writer carries the rest of.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const slowly = { taken: [] };
    // ```
    let mut slowly = AStreamThatTakesABitAtATime {
        taken: Vec::new(),
    };
    // What: the write under test, on the stream that takes a few bytes at a time.
    // Why: the rest of the line has to follow the prefix.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeTheWords(slowly, Buffer.from(line));
    // ```
    write_the_words(&mut slowly, line.as_bytes());
    assert_eq!(
        slowly.taken,
        line.as_bytes(),
        "the line arrived whole through writes that took a few bytes each"
    );
    // What: the stream whose first write is interrupted.
    // Why: a signal is not a refusal, so the same attempt is made again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const interrupted = { interrupted: false, taken: [] };
    // ```
    let mut interrupted = AStreamThatIsInterruptedOnce {
        interrupted: false,
        taken: Vec::new(),
    };
    // What: the write under test.
    // Why: the line has to arrive despite the interrupt.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeTheWords(interrupted, Buffer.from(line));
    // ```
    write_the_words(&mut interrupted, line.as_bytes());
    assert_eq!(
        interrupted.taken,
        line.as_bytes(),
        "the line arrived whole after an interrupt that took no bytes"
    );
}

/// What:
/// `fn source_files_under(directory: &Path, found: &mut Vec<PathBuf>)` collects every `.rs` file
/// under `directory`.
/// Why:
/// The last control reads the crate's own source,
/// and naming the files by walking the folder keeps it from writing the list down.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sourceFilesUnder(directory: string, found: string[]): void
/// ```
fn source_files_under(
    directory: &Path,
    found: &mut Vec<PathBuf>,
) {
    // What: the folder's entries, or the answer that it cannot be read.
    // Why: a folder the control cannot read would hide the very files it is about.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const entries = readdirSync(directory, { withFileTypes: true });
    // ```
    let listed = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()));
    for entry in listed {
        // What: one entry of the folder, or the answer that it cannot be read.
        // Why: a control that skipped an unreadable entry would pass on a folder it never saw.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const entry = entries.next().value;
        // ```
        let listed_entry = entry.unwrap_or_else(|error| {
            panic!("read an entry of {}: {error}", directory.display())
        });
        // What: where the entry is.
        // Why: a folder is walked into and a file is kept when it is Rust source.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const path = join(directory, entry.name);
        // ```
        let path: PathBuf = listed_entry.path();
        if path.is_dir() {
            source_files_under(&path, found);
            continue;
        }
        if path.extension().and_then(|extension| return extension.to_str()) == Some("rs") {
            found.push(path);
        }
    }
}

/// What:
/// Every word this crate says leaves through `notices`, so no source file calls one of the print macros.
/// Why:
/// The second-pass read of `0.2.7-alpha.21` found that ten of the eleven notice sites had no control at
/// all: putting one back to `eprintln!` left every test green, and the defect of `0.2.7-alpha.20`
/// would come back site by site.
/// This one control covers every site of the crate, present and future:
/// any `print!`, `println!`, `eprint!` or `eprintln!` in a source file that is not a test file fails it,
/// because the writer in `notices` is the only place that may write and it writes through `Write`.
/// A mention in a comment is not a call, so the control reads each line up to its own `//`.
#[test]
fn every_word_this_crate_says_leaves_through_this_module() {
    // What: the crate's own folder, which cargo pasted in at compile time.
    // Why: the control reads the source rather than a copy of the list of files.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const crate = import.meta.dirname;
    // ```
    let crate_folder: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // What: every Rust source file under `src`.
    // Why: a file outside `src` is another crate (an integration test, whose own prints are not shipped).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const files: string[] = []; sourceFilesUnder(join(crate, 'src'), files);
    // ```
    let mut files: Vec<PathBuf> = Vec::new();
    source_files_under(&crate_folder.join("src"), &mut files);
    assert!(
        files.len() > 100,
        "the walk reached this crate's source files, and it found {}",
        files.len()
    );
    // What: the print macros, longest name first so a finding names the macro that is there.
    // Why: each one panics on a write error in a release build, which is the defect under test.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const macros = ['eprintln!', 'println!', 'eprint!', 'print!'];
    // ```
    let macros: [&str; 4] = ["eprintln!", "println!", "eprint!", "print!"];
    // What: the calls the control found, one text per finding.
    // Why: a failure names every site at once, so one run tells the whole story.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const findings: string[] = [];
    // ```
    let mut findings: Vec<String> = Vec::new();
    for file in &files {
        // What: the file's name, and whether it is a test file.
        // Why: a test file's own prints never ship, and its lines are not the program's words.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name = basename(file); if (name.endsWith('_tests.rs')) continue;
        // ```
        let name: String = file
            .file_name()
            .map(|file_name| return file_name.to_string_lossy().to_string())
            .unwrap_or_default();
        if name.ends_with("_tests.rs") {
            continue;
        }
        // What: the file's text.
        // Why: a call is a call wherever in the file it sits.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const text = readFileSync(file, 'utf8');
        // ```
        let text: String = std::fs::read_to_string(file)
            .unwrap_or_else(|error| panic!("read {}: {error}", file.display()));
        for (index, line) in text.lines().enumerate() {
            // What: the line up to its own comment, which is where a mention is not a call.
            // Why: this crate's doc comments name the macros when they explain why this module exists.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const code = line.split('//')[0];
            // ```
            let code: &str = line.split("//").next().unwrap_or(line);
            for macro_name in macros {
                if code.contains(macro_name) {
                    findings.push(format!(
                        "{}:{}: {}",
                        file.display(),
                        index + 1,
                        macro_name
                    ));
                    break;
                }
            }
        }
    }
    assert!(
        findings.is_empty(),
        "every word this crate says goes through `notices`, so a file of `src` calls no print macro; these do: {}",
        findings.join(", ")
    );
}
