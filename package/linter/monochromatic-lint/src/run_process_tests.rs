//! What:
//!  Controls for writing output to a stream that may close early or fail,
//!  and for the panic hook.
//! Why:
//!  A closed pipe is an ordinary end of output;
//!  any other write failure must change the exit status.
//! The panic hook is process-wide,
//!  so its control runs one invocation in a child process.
//! Argument parsing and real streams are exercised by the binary-level container tests.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('emit', () => { /* bytes written, closed pipe tolerated, other failures reported */ });
//! ```

/// Import the stream writer,
///  the panic-hook decision and the invocation entry under test.
use super::{emit, run_process_with, silences_panics};
use crate::{cli_options::CliOptions, run_output::RunOutput};
use clap::Parser;
use std::{
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

/// Environment variable that makes `panic_hook_child` run,
///  naming its mode:
///  `plain` or `debug`.
const CHILD_MODE: &str = "MONOCHROMATIC_LINT_PANIC_HOOK_CHILD";

/// The full name of the child half,
///  as libtest selects it with `--exact`.
const CHILD_TEST: &str = "run_process::tests::panic_hook_child";

/// What the program writes on standard error for the planted panic,
///  after any hook output.
const INTERNAL_ERROR: &str = "monochromatic-lint: internal error: planted runner panic. No result was produced; rerun with --debug for the panic location.\n";

/// A stream that records what it receives and can fail on write or on flush with a chosen error.
struct Stream {
    /// Bytes accepted so far.
    written: Vec<u8>,
    /// Error every write returns,
    ///  if any.
    write_failure: Option<ErrorKind>,
    /// Error every flush returns,
    ///  if any.
    flush_failure: Option<ErrorKind>,
    /// How many times flush was called.
    flushes: usize,
}

/// Build a stream with the given failure behavior.
fn stream(write_failure: Option<ErrorKind>, flush_failure: Option<ErrorKind>) -> Stream {
    return Stream {
        written: Vec::<u8>::new(),
        write_failure,
        flush_failure,
        flushes: 0,
    };
}

/// Accept or refuse bytes as configured.
impl Write for Stream {
    /// Record the bytes,
    ///  or fail with the configured error.
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        if let Some(kind) = self.write_failure {
            return Err(std::io::Error::from(kind));
        }
        self.written.extend_from_slice(buffer);
        return Ok(buffer.len());
    }

    /// Count the flush,
    ///  or fail with the configured error.
    fn flush(&mut self) -> std::io::Result<()> {
        self.flushes += 1;
        if let Some(kind) = self.flush_failure {
            return Err(std::io::Error::from(kind));
        }
        return Ok(());
    }
}

/// Text is written whole and flushed once;
///  empty text touches the stream not at all.
#[test]
fn text_is_written_whole_and_flushed() {
    let mut open: Stream = stream(None, None);
    assert!(emit(&mut open, "{\"code\":\"x\"}\n🚀\n"));
    assert_eq!(open.written, "{\"code\":\"x\"}\n🚀\n".as_bytes());
    assert_eq!(open.flushes, 1);
    let mut untouched: Stream = stream(Some(ErrorKind::Other), Some(ErrorKind::Other));
    assert!(emit(&mut untouched, ""));
    assert!(untouched.written.is_empty());
    assert_eq!(untouched.flushes, 0);
}

/// A closed pipe on write or flush is tolerated;
///  any other failure is reported.
#[test]
fn only_a_closed_pipe_is_tolerated() {
    assert!(emit(&mut stream(Some(ErrorKind::BrokenPipe), None), "x"));
    assert!(emit(&mut stream(None, Some(ErrorKind::BrokenPipe)), "x"));
    assert!(!emit(
        &mut stream(Some(ErrorKind::PermissionDenied), None),
        "x"
    ));
    assert!(!emit(&mut stream(None, Some(ErrorKind::StorageFull)), "x"));
    // A failed write is not followed by a flush.
    let mut failed: Stream = stream(Some(ErrorKind::PermissionDenied), None);
    assert!(!emit(&mut failed, "x"));
    assert_eq!(failed.flushes, 0);
}

/// Panics are silenced on every run except a `--debug` run,
///  which keeps the default hook's location and backtrace.
#[test]
fn only_debug_runs_keep_the_default_panic_hook() {
    assert!(silences_panics(false));
    assert!(!silences_panics(true));
}

/// A runner that panics inside the invocation,
///  standing in for a defect that no known input reaches.
fn panicking_runner(_options: &CliOptions, _cwd: &Path, _stdin: &mut dyn Read) -> RunOutput {
    panic!("planted runner panic");
}

/// The child half of the panic-hook control:
///  one real invocation whose runner panics.
/// It does nothing unless `CHILD_MODE` is set,
///  and it is ignored unless selected with `--ignored`,
/// because installing a hook in a shared test binary would silence every other test's panics.
#[test]
#[ignore = "runs only as the child process of the panic-hook control"]
fn panic_hook_child() {
    let Ok(mode): Result<String, std::env::VarError> = std::env::var(CHILD_MODE) else {
        return;
    };
    let mut arguments: Vec<&str> = vec!["monochromatic-lint"];
    if mode == "debug" {
        arguments.push("--debug");
    }
    let options: CliOptions = CliOptions::parse_from(arguments);
    assert_eq!(run_process_with(&options, panicking_runner), 2);
}

/// Run this test binary again with only `panic_hook_child` selected,
///  in one mode.
fn panic_hook_child_run(mode: &str) -> Output {
    let binary: PathBuf = std::env::current_exe().expect("locate the running test binary");
    return Command::new(binary)
        .args([
            "--exact",
            CHILD_TEST,
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_MODE, mode)
        .env_remove("RUST_BACKTRACE")
        .stdin(Stdio::null())
        .output()
        .expect("run the panic-hook child");
}

/// An invocation installs the silent hook unless `--debug` is given:
///  a panic inside a plain run leaves only the
/// program's own internal-error line on standard error,
///  while a `--debug` run keeps the default hook's message and
/// location before that line.
///  Both runs end with status 2.
///  Deleting the `set_hook` call,
///  or inverting the decision,
/// changes standard error in one of the two runs.
#[test]
fn only_debug_invocations_print_the_default_panic_message() {
    let plain: Output = panic_hook_child_run("plain");
    let plain_stderr: String = String::from_utf8(plain.stderr).expect("UTF-8 standard error");
    assert!(plain.status.success(), "{plain_stderr}");
    assert_eq!(plain_stderr, INTERNAL_ERROR);
    let debug: Output = panic_hook_child_run("debug");
    let debug_stderr: String = String::from_utf8(debug.stderr).expect("UTF-8 standard error");
    assert!(debug.status.success(), "{debug_stderr}");
    // The default hook names the invocation thread, may add its numeric id, and gives the panic location.
    assert!(
        debug_stderr
            .trim_start_matches('\n')
            .starts_with("thread 'monochromatic-lint' "),
        "{debug_stderr}"
    );
    assert!(
        debug_stderr.contains(" panicked at src/run_process_tests.rs:"),
        "{debug_stderr}"
    );
    assert!(
        debug_stderr.contains("\nplanted runner panic\n"),
        "{debug_stderr}"
    );
    assert!(debug_stderr.ends_with(INTERNAL_ERROR), "{debug_stderr}");
    // The child really ran the selected test: libtest reports one passed test on standard output.
    for stdout in [&plain.stdout, &debug.stdout] {
        let text: &str = std::str::from_utf8(stdout).expect("UTF-8 standard output");
        assert!(text.contains("test result: ok. 1 passed"), "{text}");
    }
}
