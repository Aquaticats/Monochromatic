//! What:
//!  Controls for writing output to a stream that may close early or fail,
//!  and for the panic hook.
//! Why:
//!  A closed pipe is an ordinary end of output;
//!  any other write failure must change the exit status and name the stream and the error,
//! while a run that wrote both streams keeps the status its findings earned.
//! The panic hook is process-wide,
//!  so its control runs one invocation in a child process.
//! Argument parsing and real streams are exercised by the binary-level container tests.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('emit', () => { /* bytes written, closed pipe tolerated, other failures reported */ });
//! describe('writeOutputs', () => { /* refusal named on the surviving stream */ });
//! ```

/// Import the stream writer,
///  the output writer,
///  the panic-hook decision and the invocation entry under test.
use super::{emit, run_process_with, silences_panics, write_outputs};
use crate::{cli_options::CliOptions, run_finish::PROGRAM, run_output::RunOutput};
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

/// The text the stream accepted,
///  read as UTF-8 because every caller writes UTF-8.
impl Stream {
    /// The accepted bytes as text.
    fn text(&self) -> String {
        return String::from_utf8(self.written.clone()).expect("streams carry UTF-8");
    }
}

/// An output whose two streams carry text and whose status comes from the findings.
fn output(stdout: &str, stderr: &str, exit_code: u8) -> RunOutput {
    return RunOutput {
        stdout: String::from(stdout),
        stderr: String::from(stderr),
        exit_code,
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
    assert!(emit(&mut open, "{\"code\":\"x\"}\n🚀\n").is_ok());
    assert_eq!(open.written, "{\"code\":\"x\"}\n🚀\n".as_bytes());
    assert_eq!(open.flushes, 1);
    let mut untouched: Stream = stream(Some(ErrorKind::Other), Some(ErrorKind::Other));
    assert!(emit(&mut untouched, "").is_ok());
    assert!(untouched.written.is_empty());
    assert_eq!(untouched.flushes, 0);
}

/// A closed pipe on write or flush is tolerated;
///  any other failure is reported with its own kind.
#[test]
fn only_a_closed_pipe_is_tolerated() {
    assert!(emit(&mut stream(Some(ErrorKind::BrokenPipe), None), "x").is_ok());
    assert!(emit(&mut stream(None, Some(ErrorKind::BrokenPipe)), "x").is_ok());
    assert_eq!(
        emit(&mut stream(Some(ErrorKind::PermissionDenied), None), "x")
            .expect_err("a refused write")
            .kind(),
        ErrorKind::PermissionDenied
    );
    assert_eq!(
        emit(&mut stream(None, Some(ErrorKind::StorageFull)), "x")
            .expect_err("a refused flush")
            .kind(),
        ErrorKind::StorageFull
    );
    // A failed write is not followed by a flush.
    let mut failed: Stream = stream(Some(ErrorKind::PermissionDenied), None);
    assert!(emit(&mut failed, "x").is_err());
    assert_eq!(failed.flushes, 0);
}

/// Both streams accepting their text keeps the status the findings earned;
///  a refusal exits 2 and is named,
///  with the operating system's error,
///  on the stream that still works.
#[test]
fn a_refused_write_is_named_on_the_surviving_stream() {
    let written: RunOutput = output("finding\n", "note\n", 1);
    let mut stdout: Stream = stream(None, None);
    let mut stderr: Stream = stream(None, None);
    assert_eq!(write_outputs(&mut stdout, &mut stderr, &written), 1);
    assert_eq!(stdout.text(), "finding\n");
    assert_eq!(stderr.text(), "note\n");
    // Standard output refuses its text: the program's own standard-error text and then one line
    // naming the refused stream and the error reach standard error.
    let mut refused_stdout: Stream = stream(Some(ErrorKind::StorageFull), None);
    let mut surviving_stderr: Stream = stream(None, None);
    assert_eq!(
        write_outputs(&mut refused_stdout, &mut surviving_stderr, &written),
        2
    );
    assert_eq!(
        surviving_stderr.text(),
        format!(
            "note\n{PROGRAM}: Cannot write standard output: {}.\n",
            std::io::Error::from(ErrorKind::StorageFull)
        )
    );
    // Standard error refuses its text: the findings and then the explanation reach standard output.
    let mut surviving_stdout: Stream = stream(None, None);
    let mut refused_stderr: Stream = stream(None, Some(ErrorKind::StorageFull));
    assert_eq!(
        write_outputs(&mut surviving_stdout, &mut refused_stderr, &written),
        2
    );
    assert_eq!(
        surviving_stdout.text(),
        format!(
            "finding\n{PROGRAM}: Cannot write standard error: {}.\n",
            std::io::Error::from(ErrorKind::StorageFull)
        )
    );
    // Both streams refuse: no stream is left to carry the explanation, so the status is all there is.
    let mut refused_both_stdout: Stream = stream(Some(ErrorKind::StorageFull), None);
    let mut refused_both_stderr: Stream = stream(Some(ErrorKind::StorageFull), None);
    assert_eq!(
        write_outputs(&mut refused_both_stdout, &mut refused_both_stderr, &written),
        2
    );
    assert_eq!(refused_both_stdout.text(), "");
    assert_eq!(refused_both_stderr.text(), "");
    // A closed pipe on either stream is an ordinary end of output, not a refusal.
    let mut closed_stdout: Stream = stream(None, Some(ErrorKind::BrokenPipe));
    let mut open_stderr: Stream = stream(None, None);
    assert_eq!(
        write_outputs(&mut closed_stdout, &mut open_stderr, &written),
        1
    );
    assert_eq!(open_stderr.text(), "note\n");
    let mut open_stdout: Stream = stream(None, None);
    let mut closed_stderr: Stream = stream(Some(ErrorKind::BrokenPipe), None);
    assert_eq!(
        write_outputs(&mut open_stdout, &mut closed_stderr, &written),
        1
    );
    assert_eq!(open_stdout.text(), "finding\n");
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
