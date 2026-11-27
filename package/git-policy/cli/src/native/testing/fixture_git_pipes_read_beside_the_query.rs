//! What:
//!  One pipe of a bounded query,
//! read on a thread of its own:
//!  the bytes as they arrive,
//!  the news that its reader reached the end,
//!  the kernel's own answer about that pipe,
//!  and the wait for the reader's word once the kernel says the pipe has ended.
//! Why:
//!  Reading a pipe on the answering thread would wait for its end of file,
//!  which a descendant that inherited the pipe can postpone past every bound;
//!  a reader of its own lets the answer keep what has arrived and leave the rest to whoever holds it,
//!  and asking the kernel is what makes the answer about the query and not about when this process's own threads ran.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { bytes, ended, theKernelsHand } = drain(child.stdout);
//! const finished = hasEndedWithin(theKernelsHand, 250) && await theReaderHandedOverWhatItRead('standard output', program, ended, ended);
//! ```

/// How long a reader may take to hand over what it read.
use super::query_bounds::REPORTING_THE_END;
/// `Read` drains one of the query's pipes on a thread of its own.
use std::io::Read;
/// `AsRawFd` is the descriptor number behind an open pipe end,
/// `FromRawFd` is the one way to take ownership of a number the kernel handed back,
/// and `OwnedFd` is that ownership:
/// it closes the number when it goes out of scope.
use std::os::unix::io::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
};
/// `Path` is a borrowed filesystem path of raw OS bytes,
/// which names the program a failure reports.
use std::path::Path;

/// What:
///  One pipe of a query: the bytes as they arrive,
///  the news that its reader reached the end,
///  and a second reading end of the same pipe for the answer to ask the kernel about.
/// Why:
///  Reading the pipes on this thread would wait for the end of file,
/// which a descendant that inherited the pipe can postpone past every bound;
/// a reader of its own lets the answer keep what has arrived and leave the rest to whoever holds it,
/// and that reader ends when the holder lets go or with the test's process.
///  The second reading end is the kernel's own answer about the same pipe,
/// kept there until someone asks for it,
/// so a verdict about a pipe that something still holds is about the query and not about when this process's
/// own threads ran.
///  A second reading end never keeps a pipe from ending:
/// only a writing end does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { bytes, ended, theKernelsHand } = drain(child.stdout); // ended: a message when the pipe reaches its end
/// ```
fn draining<R: Read + Send + AsRawFd + 'static>(
    mut reader: R
) -> (
    std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    std::sync::mpsc::Receiver<()>,
    OwnedFd,
) {
    // What:     `libc::dup` asks the kernel for a second descriptor number for the same open pipe end;
    //           the number the reader holds keeps working and the new one has to be closed by its owner.
    //           `reader.as_raw_fd()` is the number behind the handle the thread below takes.
    // Why:      the asking outlives that handle, so it needs a number of its own;
    //           a negative answer means the ask failed and nothing can be reported about this pipe.
    // Gotcha:   a duplicated READING end of a pipe does not delay the pipe's end, though a duplicated writing
    //           end would: the kernel counts writing ends alone when it decides that a pipe has ended.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const forTheKernel: number = duplicateHandle(reader.asRawFd()); // a number this code must close
    // if (forTheKernel < 0) throw new Error("a second reading end of a query's pipe");
    // ```
    let doubled: libc::c_int = unsafe { libc::dup(reader.as_raw_fd()) };
    assert!(
        doubled >= 0,
        "a second reading end of one of a query's pipes: {}",
        std::io::Error::last_os_error()
    );
    // What:     `OwnedFd::from_raw_fd` takes ownership of one descriptor number the kernel answered, so the
    //           number is closed when this value goes out of scope.
    // Why:      every return path of the answer then closes the second reading end, including a panic's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const theKernelsHand = new OwnedHandle(forTheKernel); // closed on scope exit
    // ```
    let the_kernel_s_hand: OwnedFd = unsafe { OwnedFd::from_raw_fd(doubled) };
    let kept: std::sync::Arc<std::sync::Mutex<Vec<u8>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    let sink: std::sync::Arc<std::sync::Mutex<Vec<u8>>> = std::sync::Arc::clone(&kept);
    let _reader: std::thread::JoinHandle<()> = std::thread::Builder::new()
        .spawn(move || {
            let mut buffer: [u8; 8192] = [0; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => sink
                        .lock()
                        .expect("the pipe's bytes")
                        .extend_from_slice(&buffer[..read]),
                    Err(_) => break,
                }
            }
            let _ended = sender.send(());
        })
        .expect("the reader of a query's pipe starts");
    return (kept, receiver, the_kernel_s_hand);
}

/// What:
///  Whether the kernel says one of the query's pipes has reached its end within `bound`:
///  no process is left that holds a writing end of it.
/// Why:
///  A pipe's end is the kernel's news:
/// it arrives when the last writing end closes and stays there until
///  someone asks,
/// so a thread of this process that a loaded host wakes late still finds it instead of missing it.
/// Asking the pipe is what makes this answer about the query and whatever it started,
/// and not about when this process's own threads ran.
///  A window that has run out still asks:
/// a pipe whose end has come answers at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = hasEndedWithin(pipe.theKernelsHand, 250); // false while something holds a writing end
/// ```
fn has_ended_within(
    pipe: &OwnedFd,
    bound: std::time::Duration,
) -> bool {
    // What:     `libc::pollfd` is one question about one descriptor: `events` is what to wait for and `revents`
    //           is what the kernel answered.
    // Why:      no event is asked for here, because the bytes are the reader's job;
    //           `POLLHUP` and `POLLERR` are reported whether or not they are asked for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const question = { fd: pipe.number, asked: 0, answered: 0 };
    // ```
    let mut asked: libc::pollfd = libc::pollfd {
        fd: pipe.as_raw_fd(),
        events: 0,
        revents: 0,
    };
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        // The window left, as the whole milliseconds `libc::poll` takes; a bound past that is asked in full.
        let left: std::time::Duration = bound.saturating_sub(began.elapsed());
        let milliseconds: libc::c_int =
            libc::c_int::try_from(left.as_millis()).unwrap_or(libc::c_int::MAX);
        // SAFETY: `poll` reads the one question it is given, writes its answer into the same entry, and touches
        //  no memory beyond it; `asked.fd` is a descriptor this answer owns through `pipe`.
        let answered: libc::c_int = unsafe { libc::poll(&mut asked, 1, milliseconds) };
        if answered < 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            // A signal this process received ends the wait early and says nothing about the pipe,
            // so the same question is asked again with the window that is left.
            if error.kind() == std::io::ErrorKind::Interrupted && began.elapsed() < bound {
                continue;
            }
            panic!("one of a query's pipes could not be waited for: {error}");
        }
        // Nothing was answered within the window: something outside the query still holds a writing end.
        if answered == 0 {
            return false;
        }
        // `POLLHUP` is the kernel's news that no writing end is left; `POLLERR` is a pipe it reports as failed.
        return asked.revents & (libc::POLLHUP | libc::POLLERR) != 0;
    }
}

/// What:
///  Wait for the reader of one of the query's pipes to hand over what it read,
///  and answer whether that pipe reached its end at all.
/// Why:
///  Once the kernel says a pipe has no writing end left,
/// every byte the query wrote is in the pipe,
/// so what is left is the reader's own wakes;
/// the window that bounds them is long enough to stop a hang and nothing else,
/// and a reader that misses it is a defect of this helper,
/// said here with what was waited for and how long.
/// A pipe that something outside the query still holds needs no wait:
/// its reader ends when that holder lets go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = await theReaderHandedOverWhatItRead("standard output", "git", pipeEnded, readerWord);
/// ```
fn the_reader_handed_over_what_it_read(
    what: &str,
    program: &Path,
    the_pipe_has_ended: bool,
    the_reader_s_word: std::sync::mpsc::Receiver<()>,
) -> bool {
    if !the_pipe_has_ended {
        return false;
    }
    if the_reader_s_word.recv_timeout(REPORTING_THE_END).is_err() {
        panic!(
            "the query {program:?} reached the end of its {what}, and the thread that read it did not hand over what it read within {REPORTING_THE_END:?}"
        );
    }
    return true;
}
