//! What:
//!  The process boundary:
//!  real arguments,
//!  working directory,
//!  standard streams and exit status.
//! Why:
//!  Everything above this module is testable with injected values;
//!  this module only connects
//! them to the operating system and makes sure no failure leaves the process without the
//! program's own explanation and one of the documented statuses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // process.exitCode = runProcess();
//! ```

/// Import the command grammar,
///  the injected-environment runner,
///  output model and lint threads.
use crate::{
    cli_options::CliOptions, run_command::run_command, run_failure::panic_text,
    run_finish::PROGRAM, run_output::RunOutput, run_workers::lint_thread,
};
/// Import the argument parser trait that provides `parse`.
use clap::Parser;
/// Import stream reading and writing,
///  panic containment and scoped threads.
use std::{
    io::{ErrorKind, Read, Write},
    panic::{AssertUnwindSafe, PanicHookInfo, catch_unwind},
    path::{Path, PathBuf},
    thread::{Builder, Scope, ScopedJoinHandle},
};

/// What:
///  The signature of one complete invocation given its options,
///  working directory and input.
/// Why:
///  The executable passes `run_command`;
///  a test passes a runner that panics,
///  to observe the
/// process-wide panic hook from a separate process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Runner = (options: CliOptions, cwd: string, stdin: Readable) => RunOutput;
/// ```
pub(crate) type Runner = fn(&CliOptions, &Path, &mut dyn Read) -> RunOutput;

/// What:
///  A panic hook that prints nothing.
/// Why:
///  The default hook writes a message to standard error for every panic,
///  including ones this
/// program catches and reports as findings.
///  In `--stdin --fix` mode standard error carries JSONL,
/// so that extra text would corrupt the record stream.
///  Every panic is caught and reported in the
/// program's own format instead;
///  `--debug` keeps the default hook for its location and backtrace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.on('uncaughtException', () => {}); // reporting happens where the error is caught
/// ```
fn silent_hook(_info: &PanicHookInfo<'_>) {}

/// What:
///  Whether this invocation silences panics:
///  every run except one with `--debug`.
/// Why:
///  Naming the decision lets a unit test pin both answers;
///  `run_process_with` installs the
/// silent hook exactly when this returns true,
///  which a test observes from a child process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function silencesPanics(debug: boolean): boolean { return !debug; }
/// ```
fn silences_panics(debug: bool) -> bool {
    return !debug;
}

/// What:
///  Write bytes to a stream,
///  treating a closed pipe as an ordinary end of output.
/// Why:
///  `monochromatic-lint . | head` closes the pipe early;
///  that is not an error worth reporting.
/// Returns false when some other write failure occurred.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function emit(stream: Writable, text: string): boolean;
/// ```
fn emit(stream: &mut dyn Write, text: &str) -> bool {
    if text.is_empty() {
        return true;
    }
    let mut outcome: std::io::Result<()> = stream.write_all(text.as_bytes());
    if outcome.is_ok() {
        outcome = stream.flush();
    }
    match outcome {
        Ok(()) => return true,
        Err(error) => return error.kind() == ErrorKind::BrokenPipe,
    }
}

/// What:
///  The output reported when a panic escapes per-file containment.
/// Why:
///  The process must still end with its own explanation and status 2,
///  never a bare runtime abort.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function internalError(payload: unknown): RunOutput;
/// ```
fn internal_error(payload: &(dyn std::any::Any + Send)) -> RunOutput {
    return RunOutput {
        stdout: String::new(),
        stderr: format!(
            "{PROGRAM}: internal error: {}. No result was produced; rerun with --debug for the panic location.\n",
            panic_text(payload)
        ),
        exit_code: 2,
    };
}

/// What:
///  Run one invocation with the real working directory and standard input.
/// Why:
///  An unreadable working directory is a setup error with status 2,
///  like any other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runWithEnvironment(options: CliOptions, run: Runner): RunOutput;
/// ```
fn run_with_environment(options: &CliOptions, run: Runner) -> RunOutput {
    let cwd: PathBuf = match std::env::current_dir() {
        Ok(directory) => directory,
        Err(error) => {
            return RunOutput {
                stdout: String::new(),
                stderr: format!("{PROGRAM}: Cannot determine the working directory: {error}.\n"),
                exit_code: 2,
            };
        }
    };
    let mut stdin: std::io::StdinLock<'static> = std::io::stdin().lock();
    return run(options, &cwd, &mut stdin);
}

/// What:
///  Run one invocation,
///  converting a panic into the internal-error output.
/// Why:
///  `catch_unwind` needs a callable;
///  the closure only forwards to the named `run_with_environment`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function containedRun(options, run): RunOutput { try { return runWithEnvironment(options, run); } catch (error) { return internalError(error); } }
/// ```
fn contained_run(options: &CliOptions, run: Runner) -> RunOutput {
    let attempt: Result<RunOutput, Box<dyn std::any::Any + Send>> = catch_unwind(AssertUnwindSafe(
        #[cfg_attr(
            dylint_lib = "monochromatic_lint_closure",
            expect(
                avoidable_closure,
                reason = "catch_unwind accepts only a callable without parameters, so the borrowed options and the runner must be moved into a closure"
            )
        )]
        || {
            return run_with_environment(options, run);
        },
    ));
    match attempt {
        Ok(output) => return output,
        Err(payload) => return internal_error(payload.as_ref()),
    }
}

/// What:
///  Run one invocation on a scoped thread with the lint stack and wait for its output.
/// Why:
///  The main thread's stack is set by the platform,
///  and deeply nested input recurses in the
/// parser.
///  If the operating system refuses the thread,
///  the invocation runs on the calling thread
/// instead of failing.
///  `'scope` is the lifetime of the thread scope and `'env` of the borrowed options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runInScope(scope, options, run): RunOutput;
/// ```
fn run_in_scope<'scope, 'env>(
    scope: &'scope Scope<'scope, 'env>,
    options: &'env CliOptions,
    run: Runner,
) -> RunOutput {
    let builder: Builder = lint_thread().name(String::from(PROGRAM));
    // The thread API needs a callable; the closure only forwards to the named `contained_run`,
    // and `move` copies the borrowed options and the function pointer into it.
    let spawned: std::io::Result<ScopedJoinHandle<'scope, RunOutput>> = builder.spawn_scoped(
        scope,
        #[cfg_attr(
            dylint_lib = "monochromatic_lint_closure",
            expect(
                avoidable_closure,
                reason = "spawn_scoped runs the closure on the spawned thread after this call returns, so the borrowed options and the runner must travel inside it"
            )
        )]
        move || return contained_run(options, run),
    );
    match spawned {
        Ok(handle) => match handle.join() {
            Ok(output) => return output,
            Err(payload) => return internal_error(payload.as_ref()),
        },
        Err(_) => return contained_run(options, run),
    }
}

/// What:
///  Install the panic hook for these options,
///  run the invocation on a lint thread,
///  write both
/// streams and return the exit status.
/// Why:
///  Every file,
///  including a single file,
///  a `--concurrency 1` run and standard input,
///  is linted
/// on a thread whose stack size is explicit.
///  The runner is a parameter so a test can raise a panic
/// inside a real invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runProcessWith(options: CliOptions, run: Runner): number;
/// ```
pub(crate) fn run_process_with(options: &CliOptions, run: Runner) -> u8 {
    if silences_panics(options.debug) {
        // Box::new moves the function pointer to the heap, as the hook API requires an owned callable.
        std::panic::set_hook(Box::new(silent_hook));
    }
    // The scope API needs a callable; the closure only forwards to the named `run_in_scope`.
    let output: RunOutput = std::thread::scope(
        #[cfg_attr(
            dylint_lib = "monochromatic_lint_closure",
            expect(
                avoidable_closure,
                reason = "std::thread::scope hands its callback only the scope reference, so the borrowed options and the runner can reach run_in_scope only through a closure"
            )
        )]
        |scope| return run_in_scope(scope, options, run),
    );
    let wrote_stdout: bool = emit(&mut std::io::stdout().lock(), output.stdout.as_str());
    let wrote_stderr: bool = emit(&mut std::io::stderr().lock(), output.stderr.as_str());
    if !wrote_stdout || !wrote_stderr {
        return 2;
    }
    return output.exit_code;
}

/// What:
///  Parse the real command line,
///  run it and return the exit status.
/// Why:
///  The argument parser prints help,
///  version and usage errors itself and exits with 0 or 2.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runProcess(): number;
/// ```
pub fn run_process() -> u8 {
    let options: CliOptions = CliOptions::parse();
    return run_process_with(&options, run_command);
}

/// Stream-writing and panic-hook controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_process_tests.rs"]
mod tests;
