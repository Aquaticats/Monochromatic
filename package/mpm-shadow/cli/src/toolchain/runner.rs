//! What:
//! The seam through which every command starts another program (`cargo`,
//! `rustup`,
//! `uv`,
//! `pnpm`,
//! `bin`,
//!           `tar`,
//! `unzip`,
//! `gh`):
//! a `Runner` trait,
//! the real implementation on `std::process::Command`,
//! and the
//!           environment those programs are given.
//! Why:
//! Tests pass a recording fake,
//! so a provider's tests assert the exact arguments it builds without
//!           installing anything or touching the real host.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // runner.ts
//! export type RunRequest = { command: string; args: string[]; env?: Record<string, string>; cwd?: string };
//! export type RunOutcome = { stdout: string; stderr: string; code: number };
//! export interface Runner { run(request: RunRequest): RunOutcome }
//! ```

/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;
/// Imports what this module uses from `std::process`.
use std::process::{
    Child,
    Command,
    Output,
    Stdio,
};
/// Imports what this module uses from `std::sync`,
/// which carries the one wake-up a finished child gives the wait.
use std::sync::{
    Condvar,
    Mutex,
};
/// Imports `Duration`,
/// the length of the wait before the one line is printed.
use std::time::Duration;

/// Imports `manager_native_bin_dirs` from `super::layout`.
use super::layout::{
    manager_native_bin_dirs,
    physical_pnpm_home,
};
/// Imports what this module uses from `super::scope`.
use super::scope::{
    EnvMap,
    Scope,
};

/// The arguments as owned texts.
fn texts(arguments: &[&str]) -> Vec<String> {
    let mut owned: Vec<String> = Vec::new();
    for argument in arguments {
        owned.push((*argument).to_string());
    }
    return owned;
}

/// What:
/// One program start:
/// the program (a name looked up on `PATH`,
/// or a path),
/// its arguments,
/// an optional full
///           environment and an optional working directory.
/// Why:
/// A plain value,
/// so a fake can record it and a test can compare it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RunRequest = { command: string; args: string[]; env?: Record<string, string>; cwd?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunRequest {
    /// The program name or path.
    pub command: String,
    /// The arguments,
    /// one per element,
    /// never split by a shell.
    pub args: Vec<String>,
    /// The complete environment of the child;
    /// `None` means it inherits this process's.
    pub env: Option<BTreeMap<String, String>>,
    /// The working directory;
    /// `None` means this process's.
    pub cwd: Option<PathBuf>,
}

/// The methods of `RunRequest`.
impl RunRequest {
    /// A request with the process's own environment and working directory.
    pub fn new(
        command: &str,
        args: &[&str],
    ) -> RunRequest {
        return RunRequest {
            command: command.to_string(),
            args: texts(args),
            env: None,
            cwd: None,
        };
    }

    /// A copy with the child environment set.
    pub fn with_env(
        mut self,
        env: BTreeMap<String, String>,
    ) -> RunRequest {
        self.env = Some(env);
        return self;
    }

    /// A copy with the working directory set.
    pub fn with_cwd(
        mut self,
        cwd: PathBuf,
    ) -> RunRequest {
        self.cwd = Some(cwd);
        return self;
    }

    /// The request as one line (`cargo binstall --no-confirm ...`),
    /// for logs and test assertions.
    pub fn line(&self) -> String {
        let mut line: String = self.command.clone();
        for argument in &self.args {
            line.push(' ');
            line.push_str(argument);
        }
        return line;
    }
}

/// What a finished program printed and how it ended;
/// a program that could not start has code 1 and the reason in
/// `stderr`,
/// so callers handle one shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunOutcome {
    /// Standard output as text (invalid UTF-8 replaced).
    pub stdout: String,
    /// Standard error as text.
    pub stderr: String,
    /// The exit code;
    /// 1 when the program could not start or was killed by a signal.
    pub code: i32,
}

/// The methods of `RunOutcome`.
impl RunOutcome {
    /// Whether the program exited with code 0.
    pub fn succeeded(&self) -> bool {
        return self.code == 0;
    }

    /// Standard error if it printed any,
    /// otherwise standard output:
    /// the text to show when a command failed.
    pub fn failure_text(&self) -> String {
        if self.stderr.trim().is_empty() {
            return self.stdout.trim().to_string();
        }
        return self.stderr.trim().to_string();
    }
}

/// What:
/// How long a program this command starts may run before one line on standard error names it and the wait.
/// Why:
/// No deadline ends a wait for a program's answer:
/// a cut-off would tell a person a toolchain's parts could not be read when `rustup` was merely slow.
/// What is missing instead is any word while it waits,
/// so a run that is stuck and a run that is working look alike.
/// The bound is where that word becomes useful, and it is not a decision about the wait's end.
/// Measured on 2026-11-28 by `measure_ordinary_rustup_calls.py` in this round's scratch folder,
/// three arms interleaved in one round, 40 rounds:
/// the real `rustup --version` (this machine's `rustup 1.29.3`) took 4.39 ms at the median and 45.61 ms at its slowest,
/// and a stand-in's `which` and `component list` took 12.85 ms and 12.63 ms at their medians
/// and 26.58 ms and 26.28 ms at their slowest,
/// so 2 s is 75 times the slowest ordinary question and 157 times the median of the two questions.
/// It is also above the spread the control arm showed (its slowest sample was 10.4 times its median),
/// and above the whole `list` command the README's own measurement took (20 to 116 ms).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_WAIT_LONGER_THAN_ORDINARY = 2000;
/// ```
pub const A_WAIT_LONGER_THAN_ORDINARY: Duration = Duration::from_secs(2);

/// What:
/// The line printed once a wait for a program has passed the bound,
/// naming the program and every argument it was started with.
/// Why:
/// The reader has to learn which question is outstanding,
/// because the answer belongs to that command and the run prints nothing else until it comes.
/// It says that nothing ends the wait,
/// so a person does not wait for a cut-off that does not exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theWaitNotice(request: RunRequest, waited: Duration): string {
///   return `mpm-shadow: still waiting for \`${request.line()}\` to answer, ${seconds} s after this command started it; no deadline ends that wait`;
/// }
/// ```
fn the_wait_notice(
    request: &RunRequest,
    waited: Duration,
) -> String {
    return format!(
        "mpm-shadow: still waiting for `{}` to answer, {:.1} s after this command started it; no deadline ends that wait",
        request.line(),
        waited.as_secs_f64()
    );
}

/// What:
/// Waits for `child`,
/// saying `the_wait_notice` once through `say` when the wait has passed `bound`,
/// and reading both of its pipes while it runs.
/// Why:
/// The child's answer is what the caller acts on,
/// so its output is read as it comes and the wait has no end of its own;
/// the one line is all this adds, and it is added once however long the wait lasts.
/// `bound` and `say` are arguments so a control can pass a bound it can wait out and read what was said;
/// the shipped call passes `A_WAIT_LONGER_THAN_ORDINARY` and writes to standard error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function waitForTheProgram(child: Child, request: RunRequest, bound: number, say: (line: string) => void): Promise<Output>
/// ```
pub(crate) fn wait_for_the_program(
    child: Child,
    request: &RunRequest,
    bound: Duration,
    say: &dyn Fn(&str),
) -> std::io::Result<Output> {
    // What:
    // whether the child has ended, for the two waits below,
    // and the wake-up the reading thread gives them.
    // Why:
    // The thread that reads the pipes is the only one that knows the child ended,
    // so a condition beats a poll loop: the waiting side wakes as soon as the answer is in,
    // and no ordinary answer is slowed by the bound's grain.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ended = { held: false };
    // const woke = new Condvar();
    // ```
    let ended: Mutex<bool> = Mutex::new(false);
    let woke: Condvar = Condvar::new();
    return std::thread::scope(|scope| {
        let reading = scope.spawn(|| {
            let answer: std::io::Result<Output> = child.wait_with_output();
            let mut finished = ended
                .lock()
                .unwrap_or_else(|poisoned| return poisoned.into_inner());
            *finished = true;
            drop(finished);
            woke.notify_all();
            return answer;
        });
        // What: whether the bound passed with the child still running.
        // Why: the line is printed once, and only for a wait that outlasted an ordinary answer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!waitUntilTheChildEnded(ended, woke, bound)) say(theWaitNotice(request, bound));
        // ```
        if !wait_until_the_child_ended(&ended, &woke, Some(bound)) {
            say(&the_wait_notice(request, bound));
            wait_until_the_child_ended(&ended, &woke, None);
        }
        return reading.join().unwrap_or_else(|_| {
            return Err(std::io::Error::other(
                "the thread reading the program's output ended without an answer",
            ));
        });
    });
}

/// What:
/// Waits until the reading thread has said the child ended,
/// for at most `bound` when one is given,
/// and answers whether the child had ended.
/// Why:
/// Both waits in `wait_for_the_program` are the same question with and without a bound,
/// and the answer is what decides whether the one line is printed;
/// a bound of `None` is the wait that has no end and no deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitUntilTheChildEnded(ended: {held: boolean}, woke: Condvar, bound?: number): boolean
/// ```
fn wait_until_the_child_ended(
    ended: &Mutex<bool>,
    woke: &Condvar,
    bound: Option<Duration>,
) -> bool {
    let finished = ended
        .lock()
        .unwrap_or_else(|poisoned| return poisoned.into_inner());
    if let Some(bounded) = bound {
        let (held, _timeout) = woke
            .wait_timeout_while(finished, bounded, |done| return !*done)
            .unwrap_or_else(|poisoned| return poisoned.into_inner());
        return *held;
    }
    let held = woke
        .wait_while(finished, |done| return !*done)
        .unwrap_or_else(|poisoned| return poisoned.into_inner());
    return *held;
}

/// What:
/// Writes one line about a wait to standard error,
/// which is where this program's messages about what it is doing belong.
/// Why:
/// The reports go to standard output and the wait is not part of one,
/// so a reader piping the report nowhere sees the line and a parser of the report does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sayOnStandardError(line: string): void { process.stderr.write(line + '\n'); }
/// ```
fn say_on_standard_error(line: &str) {
    eprintln!("{line}");
}

/// What:
/// The seam:
/// something that can start a program and report what it did.
/// Why:
/// `&self` and not `&mut self` so one runner is shared by every provider;
/// the fake records through interior
///           mutability.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Runner { run(request: RunRequest): RunOutcome }
/// ```
pub trait Runner {
    /// Starts the program,
    /// waits for it and reports the outcome.
    fn run(
        &self,
        request: &RunRequest,
    ) -> RunOutcome;
}

/// The real runner:
/// `std::process::Command` with captured output and no standard input.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessRunner;

/// Implements `Runner` for `ProcessRunner`.
impl Runner for ProcessRunner {
    /// Starts the program with captured output,
    /// and says one line on standard error when the wait for its answer passes `A_WAIT_LONGER_THAN_ORDINARY`.
    fn run(
        &self,
        request: &RunRequest,
    ) -> RunOutcome {
        let mut command: Command = Command::new(&request.command);
        command.args(&request.args);
        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        if let Some(env) = &request.env {
            command.env_clear();
            command.envs(env);
        }
        if let Some(directory) = &request.cwd {
            command.current_dir(directory);
        }
        // What: the started program, or the failure to start it.
        // Why: `spawn` and the wait below are two steps here where `output` was one,
        //  so a child that has not answered can be named while the caller still waits for it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const started = command.spawn();
        // ```
        let child: Child = match command.spawn() {
            Ok(started) => started,
            Err(error) => {
                return RunOutcome {
                    stdout: String::new(),
                    stderr: format!("cannot start {}: {error}", request.command),
                    code: 1,
                };
            }
        };
        return match wait_for_the_program(
            child,
            request,
            A_WAIT_LONGER_THAN_ORDINARY,
            &say_on_standard_error,
        ) {
            Ok(output) => RunOutcome {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                code: output.status.code().unwrap_or(1),
            },
            Err(error) => RunOutcome {
                stdout: String::new(),
                stderr: format!("cannot start {}: {error}", request.command),
                code: 1,
            },
        };
    }
}

/// What:
/// The environment tool installs run with:
/// the whole given environment,
/// `HOME` set to the scope's home,
/// and
///           `PATH` led by the scope's exposure,
/// toolchain,
/// cargo,
/// uv,
/// pnpm and npm bin directories.
/// Why:
/// `cargo binstall`,
/// `pnpm add --global` and `uv tool install` find each other and the freshly installed
///           tools through `PATH`,
/// and must write into the scope's home.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function toolchainPathEnv(scope: Scope, env: EnvMap): Record<string, string>
/// ```
pub fn toolchain_path_env(
    scope: &Scope,
    env: &EnvMap,
) -> BTreeMap<String, String> {
    let mut merged: BTreeMap<String, String> = env.all().clone();
    merged.insert("HOME".to_string(), scope.home.to_string_lossy().to_string());
    let mut directories: Vec<PathBuf> = manager_native_bin_dirs(scope, env);
    if let Some(existing) = env.get("PATH") {
        directories.extend(std::env::split_paths(existing));
    }
    let joined: String = match std::env::join_paths(&directories) {
        Ok(path) => path.to_string_lossy().to_string(),
        // A directory name holding the separator cannot be joined; fall back to the inherited PATH alone.
        Err(_) => env.get("PATH").unwrap_or("").to_string(),
    };
    merged.insert("PATH".to_string(), joined);
    return merged;
}

/// What:
/// The environment `pnpm` runs with:
/// the toolchain environment with `PNPM_HOME` set to the physical spelling of the pnpm home,
/// and that home's `bin` directory first on `PATH` in the same spelling.
/// Why:
/// pnpm links a global package relative to the spelling of its home,
/// so a home reached through a symbolic link makes `pnpm add --global` write links `pnpm list --global` cannot read,
/// and pnpm refuses a global bin directory that `PATH` holds in another spelling than `PNPM_HOME`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pnpmEnv(scope: Scope, env: EnvMap): Record<string, string>
/// ```
pub fn pnpm_env(
    scope: &Scope,
    env: &EnvMap,
) -> BTreeMap<String, String> {
    let mut merged: BTreeMap<String, String> = toolchain_path_env(scope, env);
    let physical_home: PathBuf = physical_pnpm_home(env, &scope.home);
    let mut directories: Vec<PathBuf> = vec![physical_home.join("bin")];
    if let Some(existing) = merged.get("PATH") {
        directories.extend(std::env::split_paths(existing));
    }
    // A directory name holding the separator cannot be joined; the toolchain PATH then stands as it was.
    if let Ok(path) = std::env::join_paths(&directories) {
        merged.insert("PATH".to_string(), path.to_string_lossy().to_string());
    }
    merged.insert(
        "PNPM_HOME".to_string(),
        physical_home.to_string_lossy().to_string(),
    );
    return merged;
}
