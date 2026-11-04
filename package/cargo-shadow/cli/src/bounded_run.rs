//! What:
//! Runs a child process in its own process group and collects its output within a time bound.
//! Why:
//! The owner's rule of 2026-11-03 is "nothing should ever wait forever or indefinitely".
//! At the bound the whole process group is stopped,
//! and the caller gets an error that names what was running;
//! a call that returns has no member of the group left alive,
//! so no descendant can write after the caller recorded,
//! verified or rolled back what the child did (review finding 4c F2).
//! Output is returned only when it is complete:
//! a prefix is an error (review finding 4c F4).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded-run.ts
//! export function outputWithin(command: Command, input: Buffer | undefined, bound: Bound): Output  // throws RunError
//! ```

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports `std::cell::RefCell` for this file.
use std::cell::RefCell;
/// Imports `std::io::{self, Read, Write}` for this file.
use std::io::{self, Read, Write};
// Imports `std::os::unix::process::CommandExt`.
/// Imports `std::os::unix::process::CommandExt` for this file.
use std::os::unix::process::CommandExt;
// Imports `std::path::PathBuf`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// Imports `std::path::PathBuf` for this file.
use std::path::PathBuf;
// Imports `std::process::{Child, Command, ExitStatus, ...`.
/// Imports `std::process::{Child, Command, ExitStatus, Output, Stdio}` for this file.
use std::process::{Child, Command, ExitStatus, Output, Stdio};
// Imports `std::sync::mpsc::{self, Receiver, RecvTimeo...`.
/// Imports `std::sync::mpsc::{self, Receiver, RecvTimeoutError}` for this file.
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
/// Imports `std::thread` for this file.
use std::thread;
/// Imports `std::time::{Duration, Instant}` for this file.
use std::time::{Duration, Instant};

// Imports `rustix::io::Errno`.
/// Imports `rustix::io::Errno` for this file.
use rustix::io::Errno;
// Imports `rustix::process::{Pid, Signal, kill_process...`.
/// Imports `rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group}` for this file.
use rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group};

/// How often the child and its group are asked whether they are gone.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// How long the direct child is given to be reaped after the group was killed.
/// Unmeasured:
/// a killed process is reaped within milliseconds unless it is in an uninterruptible wait,
/// which no bound can fix;
/// what must be measured is the time from `kill` to `wait` on the machines the shim runs on.
const REAP_BOUND: Duration = Duration::from_secs(5);

/// How long the members of the group are given to disappear after they were killed,
/// or after the direct child exited normally.
/// Unmeasured,
/// for the same reason as `REAP_BOUND`.
const QUIESCE_BOUND: Duration = Duration::from_secs(5);

/// How long the output readers are given to reach the end of the output once the group is gone.
/// With every writer gone the pipes are at their end at once;
/// a reader that does not finish means a process outside the group holds the pipe.
/// Unmeasured:
/// what must be measured is the drain time of the largest output (`cargo metadata` of this repository).
const READER_GRACE: Duration = Duration::from_secs(2);

thread_local! {
    /// The command the user ran,
    /// for the error of a bound that was exceeded.
    static RERUN: RefCell<Option<String>> = const { RefCell::new(None) };

    /// The file that records the process group of every child started by this thread,
    /// set while a transaction is open.
    static GROUP_LOG: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// What:
/// `pub fn set_group_log(path: Option<PathBuf>)` names the file in which this thread records the process group of each child it
/// starts (`None` stops recording).
/// Why:
/// A recovery that finds a killed run's journal needs to know whether a child of that run may still be running,
/// which the end of the wrapper's lock does not say.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function setGroupLog(path?: string): void
/// ```
pub fn set_group_log(path: Option<PathBuf>) {
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    GROUP_LOG.with(|slot| { *slot.borrow_mut() = path; });
}

/// The bound of one run and the words an error uses.
#[derive(Clone, Debug)]
pub struct Bound {
    /// The longest the child may run.
    pub limit: Duration,
    /// What is running,
    /// for the error.
    pub what: String,
    /// The command to run again,
    /// for the error.
    pub rerun: String,
}

/// Why a bounded run did not produce output.
#[derive(Debug)]
pub enum RunError {
    /// The child could not be started or read.
    Io(
        /// The system error.
        io::Error,
    ),
    /// The child outlived its bound;
    /// its process group was killed.
    TimedOut {
        /// What was running.
        what: String,
        /// The bound it exceeded.
        limit: Duration,
        /// The command to run again.
        rerun: String,
    },
    /// Members of the process group could not be shown to be gone,
    /// so something may still write.
    NotQuiescent {
        /// What was running.
        what: String,
    },
    /// The output could not be read to its end.
    IncompleteOutput {
        /// What was running.
        what: String,
    },
}

/// The methods of the type above.
impl std::fmt::Display for RunError {
    /// What:
    /// `fn fmt(&self, formatter) -> fmt::Result` words the error for the user.
    /// Why:
    /// Each variant says what blocked and what state it leaves.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            RunError::Io(error) => write!(formatter, "{error}"),
            RunError::TimedOut { what, limit, rerun } => write!(
                formatter,
                "{what} did not finish within {limit:?}; it and everything it started were stopped, and no file was restored over or removed by that. Run {rerun} again when it can finish"
            ),
            // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
            // Why:      Text building and printing are written as one call instead of string concatenation.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // `..${x}..`   // template literal
            // console.error(text);   // eprintln!
            // [a, b]            // vec![a, b]
            // ```
            RunError::NotQuiescent { what } => write!(formatter, "{what} left a process running that could not be stopped, so nothing it may still write can be trusted; no file was changed by this tool"),
            // Macro `write!`: builds or prints text (or a list) from a template.
            RunError::IncompleteOutput { what } => write!(formatter, "the output of {what} could not be read to its end because a process outside its process group holds the pipe; the partial output was discarded"),
        };
    }
}

/// What:
/// `fn group_exists(group: Pid) -> bool` says whether any process is still in the group.
/// Why:
/// A process that exists but cannot be signalled (`EPERM`) still exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const groupExists = (group: number): boolean => { try { process.kill(-group, 0); return true; } catch (e) { return e.code !== 'ESRCH'; } };
/// ```
fn group_exists(group: Pid) -> bool {
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    return match test_kill_process_group(group) {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        Ok(()) => true,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(code) => code != Errno::SRCH,
    };
}

/// What:
/// `fn group_gone_within(group: Pid, limit: Duration) -> bool` polls until no process is in the group or the limit passes.
/// Why:
/// Killed members are reaped by their parent or by init a moment later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupGoneWithin(group: number, limit: number): boolean
/// ```
fn group_gone_within(group: Pid, limit: Duration) -> bool {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
    // Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let deadline: Instant = Instant::now() + limit;
    loop {
        if !group_exists(group) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(POLL_INTERVAL);
    }
}

/// What:
/// `fn reap(child: &mut Child) -> Option<ExitStatus>` waits for the direct child,
/// a few milliseconds at a time,
/// up to `REAP_BOUND`.
/// Why:
/// `Child::wait` has no deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reap(child: ChildProcess): ExitStatus | undefined
/// ```
fn reap(child: &mut Child) -> Option<ExitStatus> {
    // Typed binding: `deadline: Instant`.
    let deadline: Instant = Instant::now() + REAP_BOUND;
    loop {
        // `match`: picks the arm whose pattern fits the value.
        match child.try_wait() {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
            // Why:      Absence is a value the caller must check; there is no `null`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Some(value) is  value
            // // None        is  undefined
            // ```
            Ok(Some(status)) => return Some(status),
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `None`: the present or the absent value of an `Option`.
            Ok(None) => {}
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `None`: the present or the absent value of an `Option`.
            Err(_) => return None,
        }
        if Instant::now() >= deadline {
            // `None`: the present or the absent value of an `Option`.
            return None;
        }
        thread::sleep(POLL_INTERVAL);
    }
}

/// What:
/// `fn stop_group(group: Pid, child: &mut Child) -> bool` kills the process group,
/// reaps the direct child and waits for the group to
/// be gone;
/// `true` only if both happened.
/// Why:
/// This is the one way a run is ended early,
/// on timeout and on every error path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopGroup(group: number, child: ChildProcess): boolean
/// ```
fn stop_group(group: Pid, child: &mut Child) -> bool {
    // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
    // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // void expr;   // or try { expr } catch {}
    // ```
    let _ = kill_process_group(group, Signal::KILL);
    // Typed binding: `reaped: bool`.
    let reaped: bool = reap(child).is_some();
    return group_gone_within(group, QUIESCE_BOUND) && reaped;
}

/// What:
/// `fn collect(mut stream: impl Read + Send + 'static) -> Receiver<io::Result<Vec<u8>>>` reads a pipe to its end on its own thread.
/// Why:
/// A child that fills a pipe nobody reads blocks,
/// and the result tells whether the end of the output was reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collect(stream: Readable): Promise<Buffer>
/// ```
fn collect(mut stream: impl Read + Send + 'static) -> Receiver<io::Result<Vec<u8>>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
        // Why:      The value is built up or replaced step by step below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let x = ...;   // plain Rust `let` is TS `const`
        // ```
        // Typed binding: `bytes: Vec<u8>`.
        // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
        // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const list: string[] = [];   // or new Set() / new Map()
        // ```
        // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
        // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: T[] / Set / Map / number
        // ```
        // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
        // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: number
        // ```
        let mut bytes: Vec<u8> = Vec::new();
        // Closure `|_|`: an anonymous function, like a TS arrow function.
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
        // Why:      They express one step of a pipeline without an index loop.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // list.map(f).filter(g)
        // ```
        // Typed binding: `outcome: io::Result<Vec<u8>>`.
        let outcome: io::Result<Vec<u8>> = stream.read_to_end(&mut bytes).map(|_| { return bytes; });
        // `let _ =`: discards the result on purpose.
        let _ = sender.send(outcome);
    });
    return receiver;
}

/// What:
/// `fn finished(receiver: &Receiver<io::Result<Vec<u8>>>, what: &str) -> Result<Vec<u8>, RunError>` takes a reader's result within
/// `READER_GRACE`.
/// Why:
/// A reader that has not reached the end means the output is incomplete,
/// which is an error and never a prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finished(reader: Promise<Buffer>, what: string): Buffer  // throws
/// ```
fn finished(receiver: &Receiver<io::Result<Vec<u8>>>, what: &str) -> Result<Vec<u8>, RunError> {
    // `match`: picks the arm whose pattern fits the value.
    return match receiver.recv_timeout(READER_GRACE) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(Ok(bytes)) => Ok(bytes),
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(Err(error)) => Err(RunError::Io(error)),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => Err(RunError::IncompleteOutput { what: what.to_string() }),
    };
}

/// What:
/// `pub fn output_within(command, input, bound) -> Result<Output, RunError>` runs the command in its own process group,
/// feeds it
/// `input` and returns its complete output,
/// or stops the whole group at the bound.
/// Why:
/// Every wait for a child process in cargo-shadow goes through here,
/// so each has a bound and a named blocker.
/// The direct child is always reaped;
/// on timeout and on every error the group is killed and shown to be gone;
/// after a normal exit any member left in the group is killed too,
/// because it could still write after the caller moved on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function outputWithin(command: Command, input: Buffer | undefined, bound: Bound): Output  // throws RunError
/// ```
pub fn output_within(command: &mut Command, input: Option<&[u8]>, bound: &Bound) -> Result<Output, RunError> {
    command.process_group(0).stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
    // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
    // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
    // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const value = await expr;   // throws to the caller on failure
    // ```
    // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
    // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { ... } catch (error) { throw new MyError(String(error)); }
    // ```
    // `let mut`: a binding that may change.
    // Typed binding: `child: Child`.
    let mut child: Child = command.spawn().map_err(RunError::Io)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // What:     `.ok()` turns a `Result` into an `Option` (dropping the error); `.ok_or(e)` turns an `Option` into a `Result` with the given error.
    // Why:      The next step wants a value-or-nothing, or a value-or-error, not the other form.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return compute(); } catch { return undefined; }
    // ```
    // Typed binding: `group: Pid`.
    let group: Pid = Pid::from_raw(i32::try_from(child.id()).unwrap_or(0)).ok_or_else(|| { return RunError::Io(io::Error::other("the child has no process id")); })?;
    // Closure `|slot|`: an anonymous function, like a TS arrow function.
    GROUP_LOG.with(|slot| {
        // `Some`: the present or the absent value of an `Option`.
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        if let Some(path) = slot.borrow().as_ref()
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            && let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path)
        {
            // Macro `writeln!`: builds or prints text (or a list) from a template.
            // `let _ =`: discards the result on purpose.
            let _ = writeln!(file, "{}", group.as_raw_nonzero());
        }
    });
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `writer: Option<Receiver<io::Result<()>>>`.
    let writer: Option<Receiver<io::Result<()>>> = match (input, child.stdin.take()) {
        // `Some`: the present or the absent value of an `Option`.
        (Some(bytes), Some(mut stdin)) => {
            // What:     `.to_vec()` copies a borrowed slice (`&[T]`) into a new owned growable `Vec<T>`.
            // Why:      The result can be stored or modified independently of the slice it came from.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // [...slice]
            // ```
            // Typed binding: `owned: Vec<u8>`.
            let owned: Vec<u8> = bytes.to_vec();
            let (sender, receiver) = mpsc::channel();
            thread::spawn(move || {
                // Borrow `&owned`: lends the value without copying or giving it away.
                // `let _ =`: discards the result on purpose.
                let _ = sender.send(stdin.write_all(&owned));
            });
            // `Some`: the present or the absent value of an `Option`.
            Some(receiver)
        }
        // `None`: the present or the absent value of an `Option`.
        _ => None,
    };
    let stdout = collect(child.stdout.take().ok_or_else(|| { return RunError::Io(io::Error::other("no standard output pipe")); })?);
    let stderr = collect(child.stderr.take().ok_or_else(|| { return RunError::Io(io::Error::other("no standard error pipe")); })?);
    let deadline: Instant = Instant::now() + bound.limit;
    // Typed binding: `status: ExitStatus`.
    let status: ExitStatus = loop {
        // `match`: picks the arm whose pattern fits the value.
        match child.try_wait() {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `Some`: the present or the absent value of an `Option`.
            Ok(Some(found)) => break found,
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `None`: the present or the absent value of an `Option`.
            Ok(None) => {}
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) => {
                // Borrow `&mut child`: lends the value without copying or giving it away.
                stop_group(group, &mut child);
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                return Err(RunError::Io(error));
            }
        }
        if Instant::now() >= deadline {
            // Borrow `&mut child`: lends the value without copying or giving it away.
            if !stop_group(group, &mut child) {
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
                // Why:      The copy can be stored or moved elsewhere while the original stays in place.
                // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // structuredClone(value)  // or [...list] / {...object}
                // ```
                return Err(RunError::NotQuiescent { what: bound.what.clone() });
            }
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `.clone()`: an independent owned copy.
            return Err(RunError::TimedOut { what: bound.what.clone(), limit: bound.limit, rerun: bound.rerun.clone() });
        }
        thread::sleep(POLL_INTERVAL);
    };
    // The parent is gone; whatever it left in its group could still write, so it is stopped before the caller moves on.
    if group_exists(group) {
        // `let _ =`: discards the result on purpose.
        let _ = kill_process_group(group, Signal::KILL);
    }
    if !group_gone_within(group, QUIESCE_BOUND) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.clone()`: an independent owned copy.
        return Err(RunError::NotQuiescent { what: bound.what.clone() });
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&stdout`: lends the value without copying or giving it away.
    // Typed binding: `out: Vec<u8>`.
    let out: Vec<u8> = finished(&stdout, &bound.what)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&stderr`: lends the value without copying or giving it away.
    // Typed binding: `err: Vec<u8>`.
    let err: Vec<u8> = finished(&stderr, &bound.what)?;
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    if let Some(receiver) = writer
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        && let Ok(Err(error)) = receiver.recv_timeout(READER_GRACE)
        && error.kind() != io::ErrorKind::BrokenPipe
    {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(RunError::Io(error));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(Output { status, stdout: out, stderr: err });
}

/// What:
/// `pub fn set_rerun(command: Option<String>)` records the command the user ran,
/// for the errors of this thread.
/// Why:
/// Review finding (4c P3):
/// an error that says "run the command again" must name it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function setRerun(command?: string): void
/// ```
pub fn set_rerun(command: Option<String>) {
    // Closure `|slot|`: an anonymous function, like a TS arrow function.
    RERUN.with(|slot| { *slot.borrow_mut() = command; });
}

/// What:
/// `pub fn rerun_text() -> String` is the command to name in an error:
/// the recorded one,
/// or a plain description when none was.
/// Why:
/// Callers build their `Bound` with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function rerunText(): string
/// ```
pub fn rerun_text() -> String {
    // Closure `|slot|`: an anonymous function, like a TS arrow function.
    // `.clone()`: an independent owned copy.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return RERUN.with(|slot| { return slot.borrow().clone().map(|command| { return format!("`{command}`"); }).unwrap_or_else(|| { return "the command you ran".to_string(); }); });
}
