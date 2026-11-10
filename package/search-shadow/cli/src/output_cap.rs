//! What:
//! The cap on how much of the real tool's standard output the shadow passes on.
//! When the shadow's own standard output is not a terminal,
//! the real tool runs as a child with its output on a pipe,
//! the shadow copies that output through while it is under the cap,
//! and once the cap is passed the shadow stops the tool,
//! writes one line to standard error
//! and ends with `CAPPED_EXIT_CODE`.
//! A terminal's output is not capped:
//! there the shadow still replaces itself with the real tool.
//! Why:
//! An `rg` over a tree that is not a repository printed 13,328,044,032 bytes into a 16 GiB tmpfs
//! (issue 3474,
//! `doc/handover/questions-for-the-owner.md`),
//! and the owner decided "Cap its output" on 2026-11-10.
//! The refusal policies cannot cover that case:
//! they need a repository root or a widening flag,
//! and an ordinary pattern over an ordinary tree prints gigabytes without either.
//! A terminal is left alone because a cap there would cost the real tool its terminal:
//! no colors,
//! and a full-screen tool such as `ncdu` cannot run at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // output-cap.ts
//! export function runCapped(name: string, real: string, args: Buffer[]): number { ... }
//! ```

/// Imports the owned operating-system text type,
/// the shape of an argument that may not be valid UTF-8.
use std::ffi::OsString;

/// Imports `IsTerminal` (asks whether standard output is a terminal),
/// `Read` (the trait that adds `read` to the pipe),
/// `Write` (the trait that adds `write_all` and `flush` to standard output)
/// and `io` for the error type.
use std::io::{self, IsTerminal, Read, Write};

/// Imports the borrowed path type.
use std::path::Path;

/// Imports `Child` (the started real tool),
/// `Command` (the builder that starts it),
/// `ExitStatus` (how it ended)
/// and `Stdio` (what its standard output is attached to).
use std::process::{Child, Command, ExitStatus, Stdio};

/// Imports `sleep`,
/// which the stop uses between its looks at the tool.
use std::thread;

/// Imports `Duration` and `Instant`,
/// the two clocks the stop reads.
use std::time::{Duration, Instant};

/// Imports the shell's exit code for a finished child.
use crate::child_exit::shell_exit_code;

/// Imports the cap, the exit codes a capped or undeliverable run ends with, and the failed-start code.
use crate::constants::{CAPPED_EXIT_CODE, EXEC_FAILED_EXIT_CODE, MAX_OUTPUT_BYTES, OUTPUT_FAILED_EXIT_CODE};

/// Imports the check of the chosen program and the arrival marker of its start.
use crate::shadow_identity::{StartMode, Vetted, mark_start, vet};

/// How long a stopped tool gets to end by itself before it is killed.
/// It is the same grace this repository's push cap gives a stopped `git push`,
/// and a tool that finds its pipe closed ends at once in every case measured (`rg`, `find`, `du`, `tree`).
const STOP_GRACE: Duration = Duration::from_secs(5);

/// How often the stop looks at the tool while it waits.
const STOP_POLL: Duration = Duration::from_millis(10);

/// The most bytes read from the tool's pipe at once,
/// the same size as a Linux pipe buffer,
/// so one full pipe is one read and one write.
const READ_CHUNK: usize = 64 * 1024;

/// What:
/// `pub fn cap_applies(output_is_terminal: bool) -> bool` declares the one decision this module turns on:
/// whether the cap runs at all.
/// Why:
/// It is a named question with one answer that a test can pin,
/// where `!output_is_terminal` at the call site reads as a mistake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const capApplies = (outputIsTerminal: boolean): boolean => !outputIsTerminal;
/// ```
pub fn cap_applies(output_is_terminal: bool) -> bool {
    return !output_is_terminal;
}

/// What:
/// `pub fn standard_output_is_terminal() -> bool` asks whether this process's standard output is a terminal.
/// Why:
/// A person watching a terminal is not the case the cap protects,
/// and the real tool must keep the terminal itself (colors, and full-screen tools such as `ncdu`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standardOutputIsTerminal = (): boolean => process.stdout.isTTY === true;
/// ```
pub fn standard_output_is_terminal() -> bool {
    return io::stdout().is_terminal();
}

/// What:
/// `pub enum Forwarding { ... }` declares the three ways passing the tool's output on can end.
/// Why:
/// The caller stops a capped run,
/// reports a failed one,
/// and only waits for a complete one,
/// so the three must be told apart rather than collapsed into one number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Forwarding = { kind: 'complete' } | { kind: 'capped' } | { kind: 'failed'; error: Error };
/// ```
#[derive(Debug)]
pub enum Forwarding {
    /// Every byte the tool printed was passed on.
    Complete,
    /// The output passed the cap;
    /// exactly the cap was passed on.
    Capped,
    /// The output could not be written to the shadow's own standard output.
    Failed(io::Error),
}

/// What:
/// `fn forward(...) -> Forwarding` declares the copy loop:
/// it reads from the tool's pipe,
/// writes to the shadow's standard output,
/// and stops at the cap.
/// It is generic over source and target so a unit test can drive it with a byte slice and a buffer instead of a pipe.
/// Why:
/// This is where the whole cap is decided,
/// and a byte boundary is exactly what a process-level test cannot show precisely.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forward(from: Readable, to: Writable, cap: number): Forwarding { ... }
/// ```
fn forward<Source: Read, Target: Write>(from: &mut Source, to: &mut Target, cap: u64) -> Forwarding {
    let outcome: Forwarding = pass_on(from, to, cap);
    // What:     `to.flush()` asks the writer to hand over what it still holds.
    // Why:      Standard output is line buffered, so a complete answer that does not end in a newline would otherwise be
    // cut short in the shadow's own buffer, and a flush that fails is an output that did not arrive.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!to.flush()) return { kind: 'failed', error };
    // ```
    if let Err(error) = to.flush() {
        return Forwarding::Failed(error);
    }
    return outcome;
}

/// What:
/// `fn pass_on(...) -> Forwarding` is the copy loop without the flush.
/// Why:
/// The flush must happen on one path,
/// and keeping the loop flat keeps every `return` about one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function passOn(from: Readable, to: Writable, cap: number): Forwarding { ... }
/// ```
fn pass_on<Source: Read, Target: Write>(from: &mut Source, to: &mut Target, cap: u64) -> Forwarding {
    let mut buffer: [u8; READ_CHUNK] = [0; READ_CHUNK];
    let mut passed: u64 = 0;
    while passed < cap {
        // What:     `usize::try_from(cap - passed)` turns the bytes still allowed into the length of this read; on a
        // machine whose counts are narrower, the chunk length is the bound anyway.
        // Why:      The last read before the cap must stop at the cap exactly, or the shadow would print past it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const wanted = Math.min(cap - passed, buffer.length);
        // ```
        let wanted: usize = usize::try_from(cap - passed).unwrap_or(READ_CHUNK).min(READ_CHUNK);
        let read: usize = match from.read(&mut buffer[..wanted]) {
            // End of the tool's output: it printed less than the cap and nothing is cut.
            Ok(0) => return Forwarding::Complete,
            Ok(taken) => taken,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Forwarding::Failed(error),
        };
        if let Err(error) = to.write_all(&buffer[..read]) {
            return Forwarding::Failed(error);
        }
        passed += read as u64;
    }
    // The cap is exactly full, so one more byte decides between an output that ended here and one that did not.
    let mut one: [u8; 1] = [0; 1];
    loop {
        match from.read(&mut one) {
            Ok(0) => return Forwarding::Complete,
            Ok(_) => return Forwarding::Capped,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Forwarding::Failed(error),
        }
    }
}

/// What:
/// `fn stop(child: &mut Child) -> Option<ExitStatus>` declares the stop:
/// it waits up to `STOP_GRACE` for the tool to end by itself and kills it after that.
/// Why:
/// Closing the reading end of the pipe is what ends a tool that is writing (its next write fails),
/// but a tool can be busy elsewhere,
/// and the shadow must not wait for it forever.
/// The status is `None` for a tool that had to be killed,
/// whose status says nothing about the work it did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stop(child: ChildProcess): ExitStatus | undefined { ... }
/// ```
fn stop(child: &mut Child) -> Option<ExitStatus> {
    let deadline: Instant = Instant::now() + STOP_GRACE;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {}
            Err(_) => return None,
        }
        if Instant::now() >= deadline {
            break;
        }
        thread::sleep(STOP_POLL);
    }
    let _ = child.kill();
    let _ = child.wait();
    return None;
}

/// What:
/// `fn capped_line(name: &str, arguments: &[OsString], real: &Path) -> String` builds the one line a capped run writes
/// to standard error.
/// Why:
/// A shortened answer that does not say so is the failure this crate exists to prevent,
/// so the line names the command,
/// the cap,
/// the status,
/// both ways to read the whole thing,
/// and the way to read less.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cappedLine = (name, args, real) => `search-shadow: stopped ...`;
/// ```
fn capped_line(name: &str, arguments: &[OsString], real: &Path) -> String {
    let mut command: String = String::from(name);
    for argument in arguments {
        command.push(' ');
        command.push_str(argument.to_string_lossy().as_ref());
    }
    return format!(
        "search-shadow: stopped `{command}` after {MAX_OUTPUT_BYTES} bytes of output (limit=output-cap, exit status {CAPPED_EXIT_CODE}). Everything the real tool printed past that limit was dropped, so the output above is only its beginning. To read all of it, run the real tool itself: {}. To read less, narrow the search with a path, --max-count or --max-depth. An output a terminal shows is never capped.",
        real.display(),
    );
}

/// What:
/// `pub fn run_capped(name: &str, real: &Path, arguments: &[OsString]) -> i32` runs the real tool as a child with its
/// standard output on a pipe,
/// passes the output on up to the cap,
/// and returns the exit code.
/// Why:
/// The output of a run nothing is watching is the case that filled a tmpfs,
/// and passing it on through the shadow is the only way to stop it at a byte count.
/// `name` is the shadowed name the message quotes and `arguments` are the tool's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runCapped(name: string, real: string, args: Buffer[]): number { ... }
/// ```
pub fn run_capped(name: &str, real: &Path, arguments: &[OsString]) -> i32 {
    // The program is checked while open and started through that file, and the child carries the arrival marker
    // (`shadow_identity.rs`); the message and code of a failed start are the same as the normal path's.
    let vetted: Vetted = match vet(real) {
        Ok(checked) => checked,
        Err(reason) => {
            eprintln!("search-shadow: could not start {}: {reason}", real.display());
            return EXEC_FAILED_EXIT_CODE;
        }
    };
    let mut command: Command = vetted.command();
    command.args(arguments);
    mark_start(&mut command, StartMode::Spawn);
    command.stdout(Stdio::piped());
    let mut child: Child = match command.spawn() {
        Ok(started) => started,
        Err(error) => {
            eprintln!("search-shadow: could not start {}: {error}", real.display());
            return EXEC_FAILED_EXIT_CODE;
        }
    };

    // What:     `child.stdout.take()` takes the reading end of the pipe out of the child record.
    // Why:      The tool's output is read here; a missing pipe cannot happen after asking for one, and must still be
    // reported rather than panicked over.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fromTool = child.stdout;
    // ```
    let mut from_tool = match child.stdout.take() {
        Some(pipe) => pipe,
        None => {
            eprintln!("search-shadow: could not read {}'s output: the pipe is missing", real.display());
            let _ = stop(&mut child);
            return OUTPUT_FAILED_EXIT_CODE;
        }
    };

    let mut to_caller = io::stdout().lock();
    let outcome: Forwarding = forward(&mut from_tool, &mut to_caller, MAX_OUTPUT_BYTES);
    // Dropping the reading end closes it, so the tool's next write fails instead of filling a pipe nobody reads.
    drop(from_tool);

    return match outcome {
        Forwarding::Complete => match child.wait() {
            Ok(status) => shell_exit_code(status),
            Err(error) => {
                eprintln!("search-shadow: could not wait for {}: {error}", real.display());
                OUTPUT_FAILED_EXIT_CODE
            }
        },
        Forwarding::Capped => {
            eprintln!("{}", capped_line(name, arguments, real));
            let _ = stop(&mut child);
            CAPPED_EXIT_CODE
        }
        // A reader that stopped listening (`rg PATTERN | head`) is the ordinary close the tool would see without the
        // shadow, so it keeps the tool's own status and adds no line.
        Forwarding::Failed(error) if error.kind() == io::ErrorKind::BrokenPipe => match stop(&mut child) {
            Some(status) => shell_exit_code(status),
            None => OUTPUT_FAILED_EXIT_CODE,
        },
        Forwarding::Failed(error) => {
            eprintln!("search-shadow: could not pass {}'s output on: {error}", real.display());
            let _ = stop(&mut child);
            OUTPUT_FAILED_EXIT_CODE
        }
    };
}

/// The unit tests of the decision and the copy loop.
#[cfg(test)]
#[path = "output_cap_tests.rs"]
mod tests;
