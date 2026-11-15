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
use std::process::{Command, Stdio};
/// Imports `Duration`, `Instant` from `std::time`.
use std::time::{Duration, Instant};

/// Imports `manager_native_bin_dirs` from `super::layout`.
use super::layout::{manager_native_bin_dirs, physical_pnpm_home};
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope};

/// The arguments as owned texts.
fn texts(arguments: &[&str]) -> Vec<String> {
    let mut owned: Vec<String> = Vec::new();
    for argument in arguments {
        owned.push((*argument).to_string());
    }
    return owned;
}

/// What:
/// The moment a program a request starts is ended, and the sentence that says what sized that moment.
/// Why:
/// No request of this program may wait for good, and a bare optional deadline let a call site say "no bound" in one
/// word; a request now names a ceiling, and the value is named where it is chosen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ceiling = { at: Instant; why: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ceiling {
    /// When the program is ended.
    pub at: Instant,
    /// What sized that moment, in the words a report quotes.
    pub why: &'static str,
}

/// The methods of `Ceiling`.
impl Ceiling {
    /// A ceiling that long from now, with the sentence that says what sized it.
    pub fn after(span: Duration, why: &'static str) -> Ceiling {
        return Ceiling { at: Instant::now() + span, why };
    }

    /// A ceiling at a moment another bound already set.
    pub fn at(at: Instant, why: &'static str) -> Ceiling {
        return Ceiling { at, why };
    }
}

/// How long listing the repository's worktrees may take: sixty seconds.
/// Why:
/// `git worktree list --porcelain` of this repository answered in 30 ms on 2026-11-13 (the cleanup's measurement);
/// sixty seconds is a ceiling for a repository of many worktrees on a slow drive, and a listing that runs past it is a
/// failed listing, which keeps every version.
pub const WORKTREE_LIST_CEILING: Duration = Duration::from_secs(60);

/// How long one install step may take: one hour.
/// Why:
/// A step installs a package, runs a postinstall command that may compile, or fetches an archive; a release build of
/// this package took about a minute on 2026-11-15, and a cold compile of a large crate is the long case this is sized
/// for. At the ceiling the program is ended, the step is reported as failed, and the sync says which step it was.
pub const STEP_CEILING: Duration = Duration::from_secs(60 * 60);

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
    /// When the program is ended, and what sized that moment.
    /// There is no value that means "no bound": every request names one.
    pub deadline: Ceiling,
}

/// The methods of `RunRequest`.
impl RunRequest {
    /// A request with the process's own environment and working directory.
    pub fn new(command: &str, args: &[&str]) -> RunRequest {
        return RunRequest {
            command: command.to_string(),
            args: texts(args),
            env: None,
            cwd: None,
            deadline: Ceiling::after(STEP_CEILING, "a step that installs a package or runs a postinstall command"),
        };
    }

    /// A copy with the child environment set.
    pub fn with_env(mut self, env: BTreeMap<String, String>) -> RunRequest {
        self.env = Some(env);
        return self;
    }

    /// A copy with the working directory set.
    pub fn with_cwd(mut self, cwd: PathBuf) -> RunRequest {
        self.cwd = Some(cwd);
        return self;
    }

    /// A copy that is ended at `ceiling`.
    pub fn with_deadline(mut self, ceiling: Ceiling) -> RunRequest {
        self.deadline = ceiling;
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
    fn run(&self, request: &RunRequest) -> RunOutcome;
}

/// The real runner:
/// `std::process::Command` with captured output and no standard input.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessRunner;

/// How long the bounded wait sleeps between two reads of the child.
const WAIT_STEP: Duration = Duration::from_millis(2);

/// Implements `Runner` for `ProcessRunner`.
impl Runner for ProcessRunner {
    /// Starts the program with captured output,
    /// ends it at the request's deadline when it names one,
    /// and reports that it was ended.
    fn run(&self, request: &RunRequest) -> RunOutcome {
        let mut command: Command = Command::new(&request.command);
        command.args(&request.args);
        command.stdin(Stdio::null());
        if let Some(env) = &request.env {
            command.env_clear();
            command.envs(env);
        }
        if let Some(directory) = &request.cwd {
            command.current_dir(directory);
        }
        return bounded(command, request.deadline);
    }
}

/// What:
/// What:
/// Starts `command` and waits for it only until `deadline`, then ends it and reaps it.
/// Why:
/// The cleanup starts Git, and a Git that never ends held an install for good; the pass's work limit has to bound the
/// programs the pass starts, not only the work around them.
/// The two streams are read on threads of their own, so a full pipe cannot block the wait either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bounded(command: Command, deadline: Instant): RunOutcome
/// ```
fn bounded(mut command: Command, ceiling: Ceiling) -> RunOutcome {
    let deadline: Instant = ceiling.at;
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child: std::process::Child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return RunOutcome {
                stdout: String::new(),
                stderr: format!("cannot start {}: {error}", command.get_program().to_string_lossy()),
                code: 1,
            };
        }
    };
    // What: the two streams are read on threads of their own, and each one sends what it read back.
    // Why: a reader blocked on a pipe must not be joined by the request; the request takes what has arrived by its
    // ceiling and leaves the thread to end when the pipe does.
    let (stdout_sent, stdout_text) = std::sync::mpsc::channel::<String>();
    let (stderr_sent, stderr_text) = std::sync::mpsc::channel::<String>();
    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();
    std::thread::spawn(move || {
        let mut text: String = String::new();
        if let Some(pipe) = stdout_pipe.as_mut() {
            let _ = std::io::Read::read_to_string(pipe, &mut text);
        }
        let _ = stdout_sent.send(text);
    });
    std::thread::spawn(move || {
        let mut text: String = String::new();
        if let Some(pipe) = stderr_pipe.as_mut() {
            let _ = std::io::Read::read_to_string(pipe, &mut text);
        }
        let _ = stderr_sent.send(text);
    });
    let mut status: Option<std::process::ExitStatus> = None;
    let mut killed: bool = false;
    loop {
        match child.try_wait() {
            Ok(Some(ended)) => {
                status = Some(ended);
                break;
            }
            Ok(None) => {}
            Err(_) => {
                let _ = child.wait();
                break;
            }
        }
        if Instant::now() >= deadline {
            killed = true;
            let _ = child.kill();
            let _ = child.wait();
            break;
        }
        std::thread::sleep(WAIT_STEP);
    }
    // What: the answer waits for the output only until the ceiling.
    // Why: a descendant that inherited the pipe and outlived the direct child would otherwise hold the request for
    // good.
    let mut stdout: String = String::new();
    let mut stderr: String = String::new();
    let mut held_past_the_ceiling: bool = false;
    for (text, from_stdout) in [(&mut stdout, true), (&mut stderr, false)] {
        let left: Duration = deadline.saturating_duration_since(Instant::now());
        let arrived = if from_stdout { stdout_text.recv_timeout(left) } else { stderr_text.recv_timeout(left) };
        match arrived {
            Ok(read) => *text = read,
            Err(_) => held_past_the_ceiling = true,
        }
    }
    let mut note: String = String::new();
    if killed {
        note.push_str(&format!("the program was ended at its ceiling: {}", ceiling.why));
    } else if held_past_the_ceiling {
        note.push_str(&format!("a descendant still held the output past the ceiling: {}", ceiling.why));
    }
    if !note.is_empty() {
        if !stderr.ends_with('\n') && !stderr.is_empty() {
            stderr.push('\n');
        }
        stderr.push_str(&note);
    }
    let code: i32 = match status {
        Some(ended) => ended.code().unwrap_or(1),
        None => 1,
    };
    return RunOutcome { stdout, stderr, code };
}

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
pub fn toolchain_path_env(scope: &Scope, env: &EnvMap) -> BTreeMap<String, String> {
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
pub fn pnpm_env(scope: &Scope, env: &EnvMap) -> BTreeMap<String, String> {
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
    merged.insert("PNPM_HOME".to_string(), physical_home.to_string_lossy().to_string());
    return merged;
}
