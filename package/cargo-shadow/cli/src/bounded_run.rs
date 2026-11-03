//! What:
//! Runs a child process and collects its output within a time bound.
//! Why:
//! The owner's rule of 2026-11-03 is "nothing should ever wait forever or indefinitely".
//! At the bound the child is stopped and the caller gets an error that names what was running and the command to run again;
//! every file stays where it is.
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
/// Imports `std::io::{self, Read, Write}` for this file.
use std::io::{self, Read, Write};
// Imports `std::process::{Command, Output, Stdio}`.
/// Imports `std::process::{Command, Output, Stdio}` for this file.
use std::process::{Command, Output, Stdio};
// Imports `std::sync::{Arc, Mutex}`.
/// Imports `std::sync::{Arc, Mutex}` for this file.
use std::sync::{Arc, Mutex};
// Imports `std::thread`.
/// Imports `std::thread` for this file.
use std::thread;
// Imports `std::time::{Duration, Instant}`.
// What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
// Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: T[] / Set / Map / number
// ```
/// Imports `std::time::{Duration, Instant}` for this file.
use std::time::{Duration, Instant};

/// How often the child is asked whether it has finished.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// How long output readers are given to finish after the child exited.
/// A grandchild that inherited the pipes (cargo's compiler processes) can keep them open after the child is gone;
/// what was read by then is the output.
/// A bound chosen for the owner's rule above,
/// not measured.
const READER_GRACE: Duration = Duration::from_millis(1_000);

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
    /// The child outlived its bound and was killed.
    TimedOut {
        /// What was running.
        what: String,
        /// The bound it exceeded.
        limit: Duration,
        /// The command to run again.
        rerun: String,
    },
}

/// The methods of the type above.
impl std::fmt::Display for RunError {
    /// What:
    /// `fn fmt(&self, formatter) -> fmt::Result` words the error for the user.
    /// Why:
    /// A timeout names what blocked,
    /// says nothing was lost and names the command to run again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
        // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
        // ```
        return match self {
            // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
            // Why:      Text building and printing are written as one call instead of string concatenation.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // `..${x}..`   // template literal
            // console.error(text);   // eprintln!
            // [a, b]            // vec![a, b]
            // ```
            RunError::Io(error) => write!(formatter, "{error}"),
            // Macro `write!`: builds or prints text (or a list) from a template.
            RunError::TimedOut { what, limit, rerun } => write!(
                formatter,
                "{what} did not finish within {limit:?} and was stopped; every file is kept as it was. Run `{rerun}` again when it can finish"
            ),
        };
    }
}

/// What:
/// `fn collect(mut stream: impl Read + Send + 'static) -> Arc<Mutex<Vec<u8>>>` reads a pipe on its own thread into a shared buffer.
/// Why:
/// A child that fills a pipe nobody reads blocks,
/// and a reader that never finishes must not block the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collect(stream: Readable): { bytes: Buffer }  // appended to by a background reader
/// ```
fn collect(mut stream: impl Read + Send + 'static) -> Arc<Mutex<Vec<u8>>> {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
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
    let buffer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    // Borrow `&buffer`: lends the value without copying or giving it away.
    // Typed binding: `shared: Arc<Mutex<Vec<u8>>>`.
    let shared: Arc<Mutex<Vec<u8>>> = Arc::clone(&buffer);
    thread::spawn(move || {
        // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
        // Why:      The value is built up or replaced step by step below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let x = ...;   // plain Rust `let` is TS `const`
        // ```
        // Typed binding: `chunk: [u8; 8192]`.
        let mut chunk: [u8; 8192] = [0; 8192];
        loop {
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&mut chunk`: lends the value without copying or giving it away.
            match stream.read(&mut chunk) {
                // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
                // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // // Ok(value)  is  return value;
                // // Err(error) is  throw error;   (the caller sees it as a returned value)
                // ```
                Ok(0) | Err(_) => return,
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                Ok(count) => {
                    // `Ok(...)`: builds the success or the failure variant of a `Result`.
                    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
                    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // if (value !== undefined) { const inner = value; ... }
                    // ```
                    if let Ok(mut held) = shared.lock() {
                        // Borrow `&chunk`: lends the value without copying or giving it away.
                        held.extend_from_slice(&chunk[..count]);
                    }
                }
            }
        }
    });
    return buffer;
}

/// What:
/// `fn take(buffer: &Arc<Mutex<Vec<u8>>>, until: Instant) -> Vec<u8>` returns what a reader has gathered once it has stopped
/// growing or the time is up.
/// Why:
/// The child exited,
/// so a reader that is still going is held open by a grandchild;
/// the bytes read so far are the output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function take(buffer: { bytes: Buffer }, until: number): Buffer
/// ```
fn take(buffer: &Arc<Mutex<Vec<u8>>>, until: Instant) -> Vec<u8> {
    // `let mut`: a binding that may change.
    // Typed binding: `last: usize`.
    let mut last: usize = usize::MAX;
    loop {
        // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
        // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (a, b) => body
        // ```
        // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
        // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
        // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // value ?? fallback
        // ```
        // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
        // Why:      They express one step of a pipeline without an index loop.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // list.map(f).filter(g)
        // ```
        // Typed binding: `length: usize`.
        let length: usize = buffer.lock().map(|held| { return held.len(); }).unwrap_or(0);
        if Instant::now() >= until || (length == last && Arc::strong_count(buffer) == 1) {
            break;
        }
        last = length;
        thread::sleep(POLL_INTERVAL);
    }
    // Closure `|held|`: an anonymous function, like a TS arrow function.
    // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
    // Why:      The copy can be stored or moved elsewhere while the original stays in place.
    // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // structuredClone(value)  // or [...list] / {...object}
    // ```
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.map(`: transforms or selects items with a closure.
    return buffer.lock().map(|held| { return held.clone(); }).unwrap_or_default();
}

/// What:
/// `pub fn output_within(command, input, bound) -> Result<Output, RunError>` runs the command,
/// feeds it `input` on standard input
/// and returns its output,
/// or stops it at the bound.
/// Why:
/// Every wait for a child process in cargo-shadow goes through here,
/// so each has a bound and a named blocker.
/// The child is always waited for,
/// killed at the bound and then waited for again,
/// so none is left unreaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function outputWithin(command: Command, input: Buffer | undefined, bound: Bound): Output  // throws RunError
/// ```
pub fn output_within(command: &mut Command, input: Option<&[u8]>, bound: &Bound) -> Result<Output, RunError> {
    command.stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
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
    let mut child = command.spawn().map_err(RunError::Io)?;
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    if let (Some(bytes), Some(mut stdin)) = (input, child.stdin.take()) {
        // What:     `.to_vec()` copies a borrowed slice (`&[T]`) into a new owned growable `Vec<T>`.
        // Why:      The result can be stored or modified independently of the slice it came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // [...slice]
        // ```
        // Typed binding: `owned: Vec<u8>`.
        let owned: Vec<u8> = bytes.to_vec();
        thread::spawn(move || {
            // A child that exits early closes the pipe; the failed write is the child's answer, and it is waited for below.
            let _ = stdin.write_all(&owned);
        });
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `.ok()` turns a `Result` into an `Option` (dropping the error); `.ok_or(e)` turns an `Option` into a `Result` with the given error.
    // Why:      The next step wants a value-or-nothing, or a value-or-error, not the other form.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return compute(); } catch { return undefined; }
    // ```
    let stdout = collect(child.stdout.take().ok_or_else(|| { return RunError::Io(io::Error::other("no standard output pipe")); })?);
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
    let stderr = collect(child.stderr.take().ok_or_else(|| { return RunError::Io(io::Error::other("no standard error pipe")); })?);
    // Typed binding: `deadline: Instant`.
    let deadline: Instant = Instant::now() + bound.limit;
    let status = loop {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `match`: picks the arm whose pattern fits the value.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        match child.try_wait().map_err(RunError::Io)? {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => break found,
            // `None`: the present or the absent value of an `Option`.
            None => {
                if Instant::now() >= deadline {
                    // `let _ =`: discards the result on purpose.
                    let _ = child.kill();
                    // `let _ =`: discards the result on purpose.
                    let _ = child.wait();
                    // `Err(...)`: builds the success or the failure variant of a `Result`.
                    // `.clone()`: an independent owned copy.
                    return Err(RunError::TimedOut { what: bound.what.clone(), limit: bound.limit, rerun: bound.rerun.clone() });
                }
                thread::sleep(POLL_INTERVAL);
            }
        }
    };
    // Typed binding: `until: Instant`.
    let until: Instant = Instant::now() + READER_GRACE;
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&stdout`: lends the value without copying or giving it away.
    return Ok(Output { status, stdout: take(&stdout, until), stderr: take(&stderr, until) });
}
