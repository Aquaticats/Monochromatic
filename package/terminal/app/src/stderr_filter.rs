//! Stderr filtering for noisy Ghostty debug lines.

/// What:
///      `use std::{...};` imports standard-library modules.
///  `File` owns file
///           descriptors,
///  `BufRead` and `Write` move bytes,
///  `c_int` matches C's
///           integer type,
///  and `thread` starts the forwarding worker.
/// Why:
///       The filter redirects process stderr through a pipe and forwards every
///           line except Ghostty's known noisy OSC debug line.
use std::{
    ffi::c_int,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    thread,
};

/// What:
///      `use std::os::fd::FromRawFd;` imports Unix file-descriptor ownership
///           conversion.
///  Sibling `AsRawFd` only borrows an fd,
///  while `IntoRawFd`
///           gives one away.
/// Why:
///       After `pipe` and `dup`,
///  Rust needs to own those raw descriptors as files.
#[cfg(unix)]
use std::os::fd::FromRawFd;

/// What:
///      `const STDERR_FILE_DESCRIPTOR: c_int = 2;` names Unix stderr's fd.
///           Sibling fd values are stdin `0` and stdout `1`.
/// Why:
///       `dup2` needs the numeric destination fd for process stderr.
#[cfg(unix)]
const STDERR_FILE_DESCRIPTOR: c_int = 2;

/// What:
///      `const SUPPRESSED_GHOSTTY_OSC_LOG: &[u8] = ...` stores bytes,
///  not a
///           `&str`;
///  sibling `&str` would require valid UTF-8 input.
/// Why:
///       Stderr is a byte stream,
///  so filtering should not fail on non-UTF-8 text.
const SUPPRESSED_GHOSTTY_OSC_LOG: &[u8] = b"unimplemented OSC callback";

/// What:
///      `struct PipeFileDescriptors` names the two numeric ends returned by
///           Unix `pipe`.
///  A tuple sibling would hide which fd is read vs write.
/// Why:
///       Error cleanup must close the correct end at each step.
#[cfg(unix)]
struct PipeFileDescriptors {
    /// What:
    ///      `read_fd: c_int` holds the read end of the pipe as a raw OS file
    ///           descriptor:
    ///  a small integer the kernel hands out to name an open
    ///           resource.
    ///  `c_int` is the integer type matching C's `int` across the
    ///           FFI boundary.
    ///  Sibling `i32` is the same 32-bit width on the platforms
    ///           we target;
    ///  owned-wrapper siblings are `OwnedFd` and `File`,
    ///  which close
    ///           the descriptor automatically on drop.
    /// Why:
    ///       `c_int` (not `i32`,
    ///  and not the owning `OwnedFd`/`File`) because this
    ///           number comes straight from the C `pipe` call and is passed back into C
    ///           `close`/`dup2`;
    ///  keeping it a plain integer means there is no GC and no
    ///           automatic cleanup,
    ///  so the code must close it by hand.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// readFd: number; // a raw OS handle integer; no garbage collector frees it
    /// ```
    read_fd: c_int,
    /// What:
    ///      `write_fd: c_int` holds the write end of the pipe as a raw OS file
    ///           descriptor (a kernel-assigned integer naming an open resource).
    ///           `c_int` matches C's `int` for the FFI boundary;
    ///  sibling `i32` is the
    ///           same width here,
    ///  and owning siblings `OwnedFd`/`File` would auto-close.
    /// Why:
    ///       `c_int` (not `i32`,
    ///  not `OwnedFd`/`File`) because this integer is
    ///           produced by C `pipe` and consumed by C `dup2`/`close`;
    ///  a raw integer
    ///           has no GC and no automatic cleanup,
    ///  so the code closes it explicitly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// writeFd: number; // a raw OS handle integer; you must close it yourself
    /// ```
    write_fd: c_int,
}

// What:     `unsafe extern "C" { ... }` declares C functions supplied by libc.
//           Rust cannot check their memory or fd safety for us.
// Why:      The standard library wraps files but does not expose `pipe` or `dup2`.
#[cfg(unix)]
unsafe extern "C" {
    /// What:
    ///      `fn pipe(pipe_fds: *mut c_int) -> c_int` declares libc's `pipe`
    ///           syscall.
    ///  `*mut c_int` is a raw mutable pointer to the first of two
    ///           C-int slots the kernel fills:
    ///  index 0 with the read fd,
    ///  index 1 with
    ///           the write fd (a raw pointer is an address with no ownership or
    ///           null-safety guarantees,
    ///  unlike a Rust reference).
    ///  It returns a `c_int`
    ///           status:
    ///  `0` on success,
    ///  `-1` on error.
    /// Why:
    ///       Creates an in-memory pipe so stderr bytes can be captured into the read
    ///           end and filtered before reaching the terminal;
    ///  there is no exception on
    ///           failure,
    ///  the caller must test the `-1` sentinel.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // No 1:1 equivalent; closest is creating an OS pipe and reading its two handles.
    /// function pipe(out: number[]): number; // 0 ok, -1 error
    /// ```
    fn pipe(pipe_fds: *mut c_int) -> c_int;
    /// What:
    ///      `fn dup(old_fd: c_int) -> c_int` declares libc's `dup` syscall.
    ///  It
    ///           takes one raw file-descriptor integer and returns a new descriptor
    ///           (a fresh integer) pointing at the same open resource,
    ///  or `-1` on error.
    /// Why:
    ///       Duplicates the original stderr handle so the filter thread keeps a
    ///           stable destination to forward kept lines to even after fd 2 is
    ///           repointed;
    ///  the `-1` return is an error sentinel,
    ///  not an exception.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function dup(oldFd: number): number; // returns a new handle, or -1 on error
    /// ```
    fn dup(old_fd: c_int) -> c_int;
    /// What:
    ///      `fn dup2(old_fd: c_int, new_fd: c_int) -> c_int` declares libc's `dup2`
    ///           syscall.
    ///  It atomically makes `new_fd` refer to the same open resource
    ///           as `old_fd`,
    ///  closing whatever `new_fd` pointed at first;
    ///  both arguments
    ///           are raw fd integers and it returns `new_fd` on success or `-1` on error.
    /// Why:
    ///       Repoints fd 2 (stderr) at the pipe's write end so all later writes to
    ///           stderr flow through the filter without changing any caller;
    ///  failure is
    ///           reported by the `-1` sentinel.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function dup2(oldFd: number, newFd: number): number; // newFd ok, -1 error
    /// ```
    fn dup2(old_fd: c_int, new_fd: c_int) -> c_int;
    /// What:
    ///      `fn close(fd: c_int) -> c_int` declares libc's `close` syscall.
    ///  It
    ///           takes one raw file-descriptor integer and releases the kernel resource
    ///           it names,
    ///  returning `0` on success or `-1` on error.
    /// Why:
    ///       Raw descriptors are not garbage-collected,
    ///  so each fd opened by `pipe`
    ///           or `dup` must be closed by hand once finished;
    ///  the `-1` return is the
    ///           error sentinel rather than a thrown exception.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function close(fd: number): number; // 0 ok, -1 error; no GC frees fds
    /// ```
    fn close(fd: c_int) -> c_int;
}

/// What:
///      `pub fn install_ghostty_stderr_filter() -> io::Result<()>` installs the
///           process-wide stderr filter on Unix and no-ops elsewhere.
/// Why:
///       Ghostty writes the noisy debug line directly to stderr before Rust can
///           handle it at the terminal-engine layer.
#[cfg(unix)]
pub fn install_ghostty_stderr_filter() -> io::Result<()> {
    // What:     `let pipe_fds = create_pipe()?` creates a read end and write end.
    // Why:      Future stderr bytes need somewhere to go before filtering.
    let pipe_fds = create_pipe()?;
    // What:     `match duplicate_fd(...)` copies current stderr before replacing it.
    // Why:      The filter thread needs the original destination to forward kept lines.
    let original_stderr_fd = match duplicate_fd(STDERR_FILE_DESCRIPTOR) {
        Ok(fd) => fd,
        Err(error) => {
            close_fd(pipe_fds.read_fd);
            close_fd(pipe_fds.write_fd);
            return Err(error);
        }
    };
    // What:     `if let Err(error) = replace_stderr_with_pipe(...)` tries to make fd 2
    //           point at the pipe write end.
    // Why:      Anything writing to stderr after this goes through the filter worker.
    if let Err(error) = replace_stderr_with_pipe(pipe_fds.write_fd) {
        close_fd(pipe_fds.read_fd);
        close_fd(pipe_fds.write_fd);
        close_fd(original_stderr_fd);
        return Err(error);
    }
    // What:     `close_fd(pipe_fds.write_fd)` closes the extra write fd after `dup2`.
    // Why:      fd 2 now owns the write side, so this duplicate would only delay EOF.
    close_fd(pipe_fds.write_fd);
    // What:     `file_from_fd(...)` turns raw fds into Rust-owned `File` values.
    // Why:      The worker thread can then read and write with safe stream methods.
    let reader_file = file_from_fd(pipe_fds.read_fd);
    // What:     `file_from_fd(...)` owns the duplicate of original stderr.
    // Why:      Kept lines bypass fd 2 and avoid feeding back into the filter pipe.
    let writer_file = file_from_fd(original_stderr_fd);
    // What:     `thread::spawn(move || ...)` starts a detached worker and moves both
    //           files into it.
    // Why:      Stderr writes must not block the Slint UI thread on filtering work.
    thread::spawn(move || forward_filtered_stderr(reader_file, writer_file));
    // What:     `Ok(())` returns success with no payload.
    // Why:      fd 2 now routes through the filter.
    return Ok(())
}

/// What:
///      `pub fn install_ghostty_stderr_filter() -> io::Result<()>` is the
///           non-Unix sibling of the real installer.
/// Why:
///       The terminal package currently targets Linux,
///  but this keeps the crate
///           compiling if Cargo checks it on another platform.
#[cfg(not(unix))]
pub fn install_ghostty_stderr_filter() -> io::Result<()> {
    // What:     `Ok(())` returns success with no payload.
    // Why:      There is no Unix fd 2 to filter on this platform path.
    Ok(())
}

/// What:
///      `fn create_pipe() -> io::Result<PipeFileDescriptors>` wraps Unix `pipe`.
/// Why:
///       Callers get normal Rust error handling instead of raw `-1` checks.
#[cfg(unix)]
fn create_pipe() -> io::Result<PipeFileDescriptors> {
    // What:     `let mut pipe_fds = [0; 2]` creates two C-int slots for libc to fill.
    // Why:      `pipe` writes the read fd into index 0 and write fd into index 1.
    let mut pipe_fds = [0; 2];
    // What:     `unsafe { pipe(...) }` calls C code with a pointer to the two slots.
    // Why:      Rust has no safe wrapper for creating an anonymous pipe here.
    let result = unsafe { pipe(pipe_fds.as_mut_ptr()) };
    // What:     `if result == -1` checks libc's error sentinel.
    // Why:      Failed syscalls report details through `errno`, read by `last_os_error`.
    if result == -1 {
        return Err(io::Error::last_os_error());
    }
    // What:     `Ok(PipeFileDescriptors { ... })` wraps both raw fd numbers.
    // Why:      The caller can now close or transfer them by name.
    return Ok(PipeFileDescriptors {
        read_fd: pipe_fds[0],
        write_fd: pipe_fds[1],
    })
}

/// What:
///      `fn duplicate_fd(fd: c_int) -> io::Result<c_int>` wraps Unix `dup`.
/// Why:
///       The original stderr destination must survive after fd 2 is replaced.
#[cfg(unix)]
fn duplicate_fd(fd: c_int) -> io::Result<c_int> {
    // What:     `unsafe { dup(fd) }` asks libc to duplicate one open fd.
    // Why:      The duplicate gives the filter thread a stable output destination.
    let result = unsafe { dup(fd) };
    // What:     `if result == -1` checks libc's error sentinel.
    // Why:      `dup` can fail if the process is out of file descriptors.
    if result == -1 {
        return Err(io::Error::last_os_error());
    }
    // What:     `Ok(result)` returns the duplicated fd number.
    // Why:      The caller owns that fd and must eventually close it.
    return Ok(result)
}

/// What:
///      `fn replace_stderr_with_pipe(write_fd: c_int) -> io::Result<()>` wraps
///           Unix `dup2`.
/// Why:
///       Repointing fd 2 makes existing C/Zig stderr writes enter our pipe.
#[cfg(unix)]
fn replace_stderr_with_pipe(write_fd: c_int) -> io::Result<()> {
    // What:     `unsafe { dup2(...) }` atomically copies `write_fd` onto fd 2.
    // Why:      Future writes to stderr should use the pipe without changing callers.
    let result = unsafe { dup2(write_fd, STDERR_FILE_DESCRIPTOR) };
    // What:     `if result == -1` checks libc's error sentinel.
    // Why:      The filter cannot work if fd 2 was not replaced.
    if result == -1 {
        return Err(io::Error::last_os_error());
    }
    // What:     `Ok(())` returns success with no payload.
    // Why:      stderr now points at the pipe write end.
    return Ok(())
}

/// What:
///      `fn close_fd(fd: c_int)` wraps Unix `close` and discards errors.
/// Why:
///       Cleanup paths cannot fix a close failure and should preserve the earlier error.
#[cfg(unix)]
fn close_fd(fd: c_int) {
    // What:     `let _ = unsafe { close(fd) }` calls C `close` and explicitly discards
    //           its return code.
    // Why:      These calls are best-effort cleanup after success or another failure.
    let _ = unsafe { close(fd) };
}

/// What:
///      `fn file_from_fd(fd: c_int) -> File` converts a raw fd into a `File`.
/// Why:
///       Rust should close the fd automatically when the worker exits.
#[cfg(unix)]
fn file_from_fd(fd: c_int) -> File {
    // What:     `unsafe { File::from_raw_fd(fd) }` tells Rust it now owns `fd`.
    // Why:      The fd came from `pipe` or `dup`, so no other Rust `File` owns it.
    unsafe { return File::from_raw_fd(fd) }
}

/// What:
///      `fn forward_filtered_stderr(...)` reads redirected stderr and writes kept
///           lines to the original stderr destination.
/// Why:
///       Only Ghostty's known noisy OSC debug line should disappear.
fn forward_filtered_stderr(reader_file: File, mut writer_file: File) {
    // What:     `BufReader::new(reader_file)` buffers reads from the pipe.
    // Why:      Line-oriented filtering should not make one syscall per byte.
    let mut reader = BufReader::new(reader_file);
    // What:     `let mut line = Vec::new()` creates a growable byte buffer. Sibling
    //           `String` would require UTF-8; fixed arrays cannot grow for long logs.
    // Why:      Ghostty debug records can be long because they dump OSC metadata bytes.
    let mut line = Vec::new();
    // What:     `loop { ... }` repeats until stderr closes or reading fails.
    // Why:      The app can write diagnostics for the whole GUI lifetime.
    loop {
        line.clear();
        // What:     `reader.read_until(...)` reads one newline-terminated byte record.
        // Why:      Dropping whole lines preserves every non-matching diagnostic intact.
        let read_count = reader.read_until(b'\n', &mut line);
        // What:     `match read_count` branches on bytes read, EOF, or read error.
        // Why:      Each case has a different control-flow result.
        match read_count {
            Ok(0) => return,
            Ok(_) => {
                // What:     `if should_suppress_stderr_line(...)` checks the byte record.
                // Why:      Matching records are exactly the Ghostty OSC noise to hide.
                if should_suppress_stderr_line(line.as_slice()) {
                    continue;
                }
                // What:     `let _ = writer_file.write_all(...)` forwards all kept bytes
                //           and explicitly discards write errors.
                // Why:      If original stderr closes, there is nowhere useful to report it.
                let _ = writer_file.write_all(line.as_slice());
                // What:     `let _ = writer_file.flush()` pushes diagnostics promptly and
                //           explicitly discards flush errors.
                // Why:      Error logs should remain interactive, but failure is unrecoverable.
                let _ = writer_file.flush();
            }
            Err(_) => return,
        }
    }
}

/// What:
///      `fn should_suppress_stderr_line(line: &[u8]) -> bool` checks one borrowed
///           stderr byte line for the Ghostty noise marker.
/// Why:
///       Tests can lock the exact filtering rule without touching process stderr.
fn should_suppress_stderr_line(line: &[u8]) -> bool {
    // What:     `.windows(...).any(|window| ...)` scans overlapping byte windows with
    //           a Rust closure. It is the byte-slice sibling of string `includes`.
    // Why:      This finds the marker without regex or UTF-8 assumptions.
    return line.windows(SUPPRESSED_GHOSTTY_OSC_LOG.len())
        .any(|window| return window == SUPPRESSED_GHOSTTY_OSC_LOG)
}

/// What:
///      `#[cfg(test)] #[path = "stderr_filter_tests.rs"] mod tests;`
///           declares a test-only submodule whose code lives in the sibling
///           file `stderr_filter_tests.rs`.
///  `#[cfg(test)]` gates it to test
///           builds only;
///  `#[path = "..."]` aims the module at a flat sibling
///           file instead of the default `stderr_filter/tests.rs`
///           subdirectory lookup.
///  The file stays the `tests` CHILD of
///           stderr_filter,
///  so its `use super::*` reaches the module items
///           (including private ones) unchanged.
/// Why:
///       Keep `stderr_filter.rs` to production code;
///  the tests live
///           beside it without inflating this file or its max-lines budget
///           (sibling `*_tests.rs` files are exempt from the linter).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // stderr_filter.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "stderr_filter_tests.rs"]
mod tests;
