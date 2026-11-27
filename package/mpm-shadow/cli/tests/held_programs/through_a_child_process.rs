//! What:
//! The one holder of a program file these test binaries start while it is being written:
//! a child process that opens the file for writing and keeps it open until the value returned
//! here is dropped.
//! Why:
//! `execve` answers `Text file busy` for a file that any process holds open for writing,
//! so a test of that refusal needs a holder.
//! The holder has to be another process:
//! a write descriptor opened by the test process is inherited by every child another test thread
//! forks and stays open there until that child reaches its own start
//! (issue 6527,
//! measured 2026-11-27),
//! so it would hold files of tests that are meant to start one.
//! The file lives in a folder of its own because several test binaries include it and it is not a
//! test binary itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // tests/held-programs/through-a-child-process.ts
//! export function holdOpenForWriting(path: string): Holding
//! ```

/// Imports the buffered reader that reads the holder's report line,
/// and the trait that adds `read_line` to it.
use std::io::{
    BufRead,
    BufReader,
};
/// Imports the borrowed path type the holder takes.
use std::path::Path;
/// Imports the started child,
/// the pipe into it whose close ends the hold,
/// the command that starts it,
/// its standard streams,
/// and the type of its standard output.
use std::process::{
    Child,
    ChildStdin,
    ChildStdout,
    Command,
    Stdio,
};
/// Imports the span of time a timed hold lasts.
use std::time::Duration;

/// What:
/// A child process that holds one file open for writing,
/// released when this value is dropped.
/// Why:
/// `execve` refuses a file that any process holds open for writing,
/// so the refusal needs a holder that outlives the start the test watches and
/// never belongs to the test process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Holding = { holder: ChildProcess };
/// ```
pub struct HeldOpenForWriting {
    /// The `sh` process whose descriptor number three is the file open for writing.
    holder: Child,
}

/// What:
/// `impl Drop for HeldOpenForWriting` closes the pipe into the holder and waits for it to end.
/// Why:
/// Closing the pipe is how the holder learns its work is over,
/// and waiting for it keeps it from outliving the test that started it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // registered in the fixture's teardown: holder.stdin.end(); holder.waitSync();
/// ```
impl Drop for HeldOpenForWriting {
    /// Ends the hold and reaps the holder.
    fn drop(&mut self) {
        // What:     `.stdin.take()` moves the pipe out of the child record, and `drop` closes it here.
        // Why:      Dropping the last handle on the pipe gives the holder's `cat` its end of file,
        //           which is the only thing it waits for.
        let pipe: Option<ChildStdin> = self.holder.stdin.take();
        drop(pipe);
        let _ = self.holder.wait();
    }
}

/// What:
///     `fn hold_open_for_writing(path) -> HeldOpenForWriting` starts a `sh` that opens `path` for
///     appending and keeps it open until the returned value is dropped,
///     and returns once the file really is open.
/// Why:
///     This is the write a start can meet:
///     the system refuses to start a file that is being written,
///     and the test has to be able to say exactly that and nothing else.
///     The holder appends no bytes,
///     so the program in the file stays the one the test wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdOpenForWriting(path: string): HeldOpenForWriting
/// ```
pub fn hold_open_for_writing(path: &Path) -> HeldOpenForWriting {
    // What:     `Command::new("/bin/sh")` starts a shell whose arguments after the script are
    //           `sh` (its own `$0`) and `path` (its `$1`).
    // Why:      `exec 3>>"$1"` opens the file for appending on descriptor three of the shell itself,
    //           so the descriptor belongs to a process of the test's own and never to the test.
    let mut holder: Child = Command::new("/bin/sh")
        .arg("-c")
        .arg("exec 3>>\"$1\" || exit 9; printf 'held\\n'; cat > /dev/null")
        .arg("sh")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("start the holder of the program file");
    // What:     `.stdout.take()` moves the reporting pipe out of the child record.
    // Why:      The line `held` is the proof the file is open before the test starts anything,
    //           so no test races the holder.
    let reporting: ChildStdout = holder.stdout.take().expect("the holder's report");
    let mut reader: BufReader<ChildStdout> = BufReader::new(reporting);
    let mut line: String = String::new();
    reader
        .read_line(&mut line)
        .expect("read the holder's report");
    assert_eq!(line, "held\n", "the holder did not report the open file");
    return HeldOpenForWriting {
        holder,
    };
}

/// What:
/// A child process that holds one file open for writing for a stated time and then lets it go,
/// released early when this value is dropped.
/// Why:
/// A control that bounds what a refused start does needs a hold with an end it can state:
/// a start that repeated the attempt would still be holding the file when the program became
/// startable, so it would start the program instead of being refused again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HoldingUntil = { holder: ChildProcess; stillHolding(): boolean };
/// ```
pub struct HeldOpenUntil {
    /// The `sh` process whose descriptor number three is the file open for writing.
    holder: Child,
}

impl HeldOpenUntil {
    /// What:
    /// `still_holding()` answers whether the holder is still running,
    /// and so still holds the file open.
    /// Why:
    /// A control that asks this after a refused start proves the start gave up while the file was
    /// still being written, never that it waited for the write to end.
    pub fn still_holding(&mut self) -> bool {
        // What:     `try_wait()` returns `Some(status)` once the child ended and `None` while it runs.
        // Why:      The holder ends when its `sleep` finishes, which closes its descriptor.
        return self.holder.try_wait().expect("ask the holder").is_none();
    }
}

/// What:
/// `impl Drop for HeldOpenUntil` waits for the holder to end.
/// Why:
/// A holder that outlived its test would keep the program unstartable for every later test that
/// starts the same file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // registered in the fixture's teardown: holder.waitSync();
/// ```
impl Drop for HeldOpenUntil {
    /// Waits for the holder's own `sleep` to end.
    fn drop(&mut self) {
        let _ = self.holder.wait();
    }
}

/// What:
///     `fn hold_open_until(path, hold)` starts a `sh` that opens `path` for appending,
///     keeps it open for `hold` seconds and then exits,
///     and returns once the file really is open.
/// Why:
///     The control states its bound from this hold:
///     a refused start has to end while the hold lasts,
///     and a start that waited for the file to be free would end after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdOpenUntil(path: string, hold: Duration): HoldingUntil
/// ```
pub fn hold_open_until(
    path: &Path,
    hold: Duration,
) -> HeldOpenUntil {
    // What:     The holder's own `sleep` takes whole seconds in every `sh`, so a fractional hold
    //           is refused here rather than silently shortened.
    // Why:      A silently shortened hold would weaken the bound the control states from it.
    assert_eq!(
        hold.subsec_nanos(),
        0,
        "the hold is stated in whole seconds: {hold:?}"
    );
    // What:     `Command::new("/bin/sh")` starts a shell whose arguments after the script are
    //           `sh` (its own `$0`), `path` (its `$1`) and the number of seconds (its `$2`).
    // Why:      `exec 3>>"$1"` opens the file for appending on descriptor three of the shell itself,
    //           so the descriptor belongs to a process of the test's own and never to the test.
    let mut holder: Child = Command::new("/bin/sh")
        .arg("-c")
        .arg("exec 3>>\"$1\" || exit 9; printf 'held\\n'; sleep \"$2\"")
        .arg("sh")
        .arg(path)
        .arg(hold.as_secs().to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .expect("start the holder of the program file for a stated time");
    // What:     `.stdout.take()` moves the reporting pipe out of the child record.
    // Why:      The line `held` is the proof the file is open before the test starts anything,
    //           so no test races the holder.
    let reporting: ChildStdout = holder.stdout.take().expect("the holder's report");
    let mut reader: BufReader<ChildStdout> = BufReader::new(reporting);
    let mut line: String = String::new();
    reader
        .read_line(&mut line)
        .expect("read the holder's report");
    assert_eq!(line, "held\n", "the holder did not report the open file");
    return HeldOpenUntil {
        holder,
    };
}
