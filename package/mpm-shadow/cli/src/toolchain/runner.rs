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
//! the
//!           bound every start runs under,
//! and the environment those programs are given.
//! Why:
//! Tests pass a recording fake,
//! so a provider's tests assert the exact arguments it builds without
//!           installing anything or touching the real host.
//! Nothing may wait forever,
//! so the real runner stops a program that outlives its bound instead of waiting on it,
//! and a caller whose read must not stall states a shorter bound than the default.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // runner.ts
//! export type RunRequest = { command: string; args: string[]; env?: Record<string, string>; cwd?: string; bound: number };
//! export type RunOutcome = { stdout: string; stderr: string; code: number };
//! export interface Runner { run(request: RunRequest): RunOutcome }
//! ```

/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;
/// Imports what this module uses from `std::process`.
use std::process::{Command, Stdio};
/// Imports `Duration`,
/// `Instant` from `std::time`.
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
/// The longest one program the toolchain starts may run before the real runner stops it:
/// one hour.
/// Why:
/// Nothing may wait forever,
/// so a program that hangs on a network read or on a lock of its own ends instead of holding the install:
/// the review of 2026-11-14 found `cargo binstall` waiting on a read with no timeout,
/// and the answer has to be a bound on every start rather than on the one that hung.
/// An hour is the agent's choice,
/// open to the owner's veto:
/// the longest a real install takes here is a third-party crate compiled from source
/// (the store held a 3.1 GiB `llama.cpp` build on 2026-11-13,
/// and a release build of this package's own 8 000-line crate took 18 s in this run's scope),
/// so an hour covers a source build several times over and still ends a hang.
/// A program stopped at its bound loses nothing a retry cannot do:
/// its version directory stays unfinished,
/// which the size- and write-time cleanup keeps because it is recent,
/// and the next install replaces it.
pub const DEFAULT_PROGRAM_BOUND: Duration = Duration::from_secs(3600);

/// How long the runner waits between two looks at a program that is still running.
/// It starts here so a program that finishes in milliseconds costs almost nothing,
/// and doubles up to `LONGEST_PAUSE` so an hour's wait is a few thousand checks rather than an hour of them.
const FIRST_PAUSE: Duration = Duration::from_millis(1);

/// The longest pause between two looks at a running program.
const LONGEST_PAUSE: Duration = Duration::from_millis(25);

/// How many bytes one read of a captured stream takes.
const BYTES_PER_READ: usize = 8192;

/// How long the runner waits for a stopped program's captured output once it is gone.
/// A program whose own child still holds the pipe never closes it,
/// so the output is taken as it stands rather than waited for.
const STOPPED_OUTPUT_GRACE: Duration = Duration::from_millis(500);

/// How long the runner waits for a killed program to be reaped.
/// Why:
/// A program held in an uninterruptible call cannot be reaped at all,
/// and the bound belongs to the runner,
/// so the reap has one of its own:
/// a program still there after this is left to the kernel rather than waited for.
const REAP_GRACE: Duration = Duration::from_secs(2);

/// How long the process table is read for,
/// to find what a stopped program started:
/// 500 ms.
/// Why:
/// A process that left the program's own process group is reached through its parent and by nothing else,
/// so the tree is recorded before the group is signalled;
/// this machine's table holds about a thousand processes and reading it costs about 0.18 s
/// (`doc/toolchain-version-cleanup.md`),
/// so half a second covers it with room.
/// A tree this does not reach is not stopped,
/// and the caller is told so.
const DESCENDANT_SCAN_DEADLINE: Duration = Duration::from_millis(500);

/// How long the runner waits for the shell that signals a stopped program's group:
/// 2 s.
const SIGNAL_HELPER_BOUND: Duration = Duration::from_secs(2);

/// How long the runner waits,
/// after signalling,
/// for the processes it signalled to be gone:
/// 1 s.
/// Why:
/// A signalled process leaves the table at once,
/// unless it is a zombie waiting for the parent it was given,
/// and the wait must not hold a command that has already given up.
const STOP_SETTLE_GRACE: Duration = Duration::from_secs(1);

/// What:
/// One program start:
/// the program (a name looked up on `PATH`,
/// or a path),
/// its arguments,
/// an optional full
///           environment,
/// an optional working directory and
/// how long it may run.
/// Why:
/// A plain value,
/// so a fake can record it and a test can compare it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RunRequest = { command: string; args: string[]; env?: Record<string, string>; cwd?: string; bound: number };
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
    /// The longest it may run;
    /// the real runner stops it after this and reports a failure naming the program and the bound.
    pub bound: Duration,
}

/// The methods of `RunRequest`.
impl RunRequest {
    /// A request with the process's own environment and working directory,
    /// bounded by `DEFAULT_PROGRAM_BOUND`.
    pub fn new(command: &str, args: &[&str]) -> RunRequest {
        return RunRequest {
            command: command.to_string(),
            args: texts(args),
            env: None,
            cwd: None,
            bound: DEFAULT_PROGRAM_BOUND,
        };
    }

    /// A copy with a shorter (or longer) bound than the default.
    pub fn bounded(mut self, bound: Duration) -> RunRequest {
        self.bound = bound;
        return self;
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

/// Reads one captured stream and sends what it reads as it arrives,
/// then `None` when the stream ended.
/// Why:
/// The bytes must reach the caller before the stream ends:
/// a program can exit while one of its own children still holds the pipe,
/// and then the stream never ends.
/// A reader still waiting when the runner gives up on it is left behind rather than joined,
/// for the same reason.
fn collect(source: Option<impl std::io::Read + Send + 'static>, stdout: bool, sender: std::sync::mpsc::Sender<(bool, Option<Vec<u8>>)>) {
    let Some(mut stream) = source else {
        let _sent = sender.send((stdout, None));
        return;
    };
    let mut buffer: [u8; BYTES_PER_READ] = [0; BYTES_PER_READ];
    loop {
        match std::io::Read::read(&mut stream, &mut buffer) {
            Ok(0) => {
                let _sent = sender.send((stdout, None));
                return;
            }
            Ok(read) => {
                if sender.send((stdout, Some(buffer[..read].to_vec()))).is_err() {
                    return;
                }
            }
            Err(_) => {
                let _sent = sender.send((stdout, None));
                return;
            }
        }
    }
}

/// Waits for a started program to end,
/// or kills it once `bound` passed,
/// and returns its exit status when it ended.
/// Why:
/// A helper the runner itself starts is waited for with this,
/// so the stop path can never reach itself through another stop.
fn wait_for_exit(child: &mut std::process::Child, bound: Duration) -> Option<std::process::ExitStatus> {
    let give_up_at: Instant = Instant::now() + bound;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {}
            Err(_unknown) => return None,
        }
        if Instant::now() >= give_up_at {
            let _killed = child.kill();
            let _reaped = child.try_wait();
            return None;
        }
        std::thread::sleep(FIRST_PAUSE);
    }
}

/// What:
/// Sends SIGKILL to the process group `leader` leads and to each process in `more`.
/// Why:
/// The standard library can signal only a child it started itself,
/// so the group is signalled through the shell's own `kill`,
/// whose negative number names the group;
/// the group's number is the leader's,
/// because the program is started in a group of its own.
/// A signal that cannot be sent costs the stop and not the command:
/// the caller is told what is left running.
#[cfg(unix)]
fn signal_dead(leader: u32, more: &[u32]) -> bool {
    let mut line: String = format!("kill -s KILL -- -{leader}");
    for pid in more {
        line.push_str(&format!(" {pid}"));
    }
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg(&line).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    let Ok(mut helper) = command.spawn() else {
        return false;
    };
    return match wait_for_exit(&mut helper, SIGNAL_HELPER_BOUND) {
        Some(status) => status.success(),
        None => false,
    };
}

/// Sends SIGKILL to a program and the processes it started;
/// no system but Unix has a process group to name.
#[cfg(not(unix))]
fn signal_dead(_leader: u32, _more: &[u32]) -> bool {
    return false;
}

/// What:
/// The state letter and the parent's number one process's `/proc/<pid>/stat` line holds,
/// or `None` where it cannot be read.
/// Why:
/// The kernel writes the process's own name in parentheses in that line,
/// and a name may hold spaces or parentheses,
/// so the fields after it are counted from the last `)`:
/// the first is the state and the second is the parent.
#[cfg(target_os = "linux")]
fn state_and_parent(pid: u32) -> Option<(char, u32)> {
    let line: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after_name: &str = line.rsplit_once(')')?.1;
    let mut fields = after_name.split_whitespace();
    let state: char = fields.next()?.chars().next()?;
    let parent: u32 = fields.next()?.parse().ok()?;
    return Some((state, parent));
}

/// The state letter and the parent's number of one process;
/// no system but Linux keeps the file that says.
#[cfg(not(target_os = "linux"))]
fn state_and_parent(_pid: u32) -> Option<(char, u32)> {
    return None;
}

/// Whether the process `pid` is in the process table and not a zombie waiting to be reaped.
fn is_running(pid: u32) -> bool {
    return match state_and_parent(pid) {
        Some((state, _parent)) => state != 'Z',
        None => false,
    };
}

/// What:
/// Every process whose parent chain leads to `root`,
/// as the table shows it now.
/// Why:
/// A process that called `setsid` or `setpgid` itself is no longer in the group a stop signals,
/// and its parent is the only thing that still names it;
/// recording the tree before the group is signalled is what lets the stop reach those too,
/// and say which ones it could not.
fn descendants_of(root: u32, deadline: Instant) -> Vec<u32> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    let mut parents: Vec<(u32, u32)> = Vec::new();
    for entry in entries.flatten() {
        if Instant::now() >= deadline {
            break;
        }
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        if let Some((_state, parent)) = state_and_parent(pid) {
            parents.push((pid, parent));
        }
    }
    let mut found: Vec<u32> = Vec::new();
    let mut frontier: Vec<u32> = vec![root];
    while let Some(parent) = frontier.pop() {
        for (pid, candidate_parent) in &parents {
            if *candidate_parent == parent && *pid != root {
                found.push(*pid);
                frontier.push(*pid);
            }
        }
    }
    return found;
}

/// What:
/// Stops a program whole:
/// the program,
/// the process group it leads,
/// and every process it started,
/// and answers with what is still running when that is not nothing.
/// Why:
/// A program the runner stops at its bound is stopped so that no part of its work goes on beside the next command;
/// a descendant still writing into an install would meet the retry.
/// The group is signalled,
/// which is one step for everything still in it;
/// the tree recorded before the signal is signalled too,
/// for a process that left the group;
/// and a process that left both (a double fork that reparented before the tree was read) is beyond reach,
/// which is why the caller is told what is still running rather than promised silence.
fn stop(child: &mut std::process::Child) -> Option<String> {
    let leader: u32 = child.id();
    let tree: Vec<u32> = descendants_of(leader, Instant::now() + DESCENDANT_SCAN_DEADLINE);
    if !signal_dead(leader, &tree) {
        // Best effort: the group could not be signalled, so at least the program the runner started is killed.
        let _killed = child.kill();
    }
    let give_up_at: Instant = Instant::now() + REAP_GRACE;
    while Instant::now() < give_up_at {
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) => std::thread::sleep(FIRST_PAUSE),
            Err(_unknown) => break,
        }
    }
    let settle_until: Instant = Instant::now() + STOP_SETTLE_GRACE;
    loop {
        let mut alive: Vec<u32> = tree.iter().copied().filter(|pid| return is_running(*pid)).collect();
        if alive.is_empty() {
            return None;
        }
        if Instant::now() >= settle_until {
            alive.sort_unstable();
            return Some(format!("these processes it started are still running: {alive:?}"));
        }
        std::thread::sleep(FIRST_PAUSE);
    }
}

/// What:
/// Waits for a started program until its bound,
/// and returns its exit status when it ended,
/// or why something it started may still run when the bound passed and it was stopped.
/// Why:
/// The pause grows from `FIRST_PAUSE` to `LONGEST_PAUSE`,
/// so a program that answers in milliseconds is not held up by the check
/// and an hour's wait costs a few thousand checks rather than an hour of them.
fn wait_until_stopped(child: &mut std::process::Child, bound: Duration) -> Result<std::process::ExitStatus, Option<String>> {
    let give_up_at: Instant = Instant::now() + bound;
    let mut pause: Duration = FIRST_PAUSE;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {}
            Err(_unknown) => return Err(None),
        }
        if Instant::now() >= give_up_at {
            let note: Option<String> = stop(child);
            return Err(note);
        }
        std::thread::sleep(pause);
        pause = pause.saturating_mul(2).min(LONGEST_PAUSE);
    }
}

/// The text of both captured streams,
/// taken as it stands once both ended or `STOPPED_OUTPUT_GRACE` passed.
fn captured_output(receiver: &std::sync::mpsc::Receiver<(bool, Option<Vec<u8>>)>) -> (String, String) {
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let mut open: usize = 2;
    let give_up_at: Instant = Instant::now() + STOPPED_OUTPUT_GRACE;
    while open > 0 {
        let left: Duration = give_up_at.saturating_duration_since(Instant::now());
        if left.is_zero() {
            break;
        }
        let Ok((is_stdout, chunk)) = receiver.recv_timeout(left) else {
            break;
        };
        match chunk {
            Some(bytes) if is_stdout => stdout.extend_from_slice(&bytes),
            Some(bytes) => stderr.extend_from_slice(&bytes),
            None => open -= 1,
        }
    }
    return (String::from_utf8_lossy(&stdout).to_string(), String::from_utf8_lossy(&stderr).to_string());
}

/// Implements `Runner` for `ProcessRunner`.
impl Runner for ProcessRunner {
    /// Starts the program with captured output,
    /// and stops it once it has run for its bound.
    fn run(&self, request: &RunRequest) -> RunOutcome {
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
        // What: the program is started in a process group of its own.
        // Why: a stopped program is stopped whole through that group,
        // and a signal aimed at this process's group (a terminal's interrupt) cannot take the program away from
        // under the runner that is waiting for it.
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let mut child: std::process::Child = match command.spawn() {
            Ok(started) => started,
            Err(error) => {
                return RunOutcome {
                    stdout: String::new(),
                    stderr: format!("cannot start {}: {error}", request.command),
                    code: 1,
                };
            }
        };
        // What: each captured stream is drained by its own thread.
        // Why: a program that writes more than a pipe holds would block before it exits,
        // and one whose own child keeps the pipe open would hold the reader for ever.
        let (sender, receiver) = std::sync::mpsc::channel::<(bool, Option<Vec<u8>>)>();
        let out = child.stdout.take();
        let err = child.stderr.take();
        let out_sender = sender.clone();
        let err_sender = sender.clone();
        drop(sender);
        let out_thread = std::thread::spawn(move || return collect(out, true, out_sender));
        let err_thread = std::thread::spawn(move || return collect(err, false, err_sender));
        let waited: Result<std::process::ExitStatus, Option<String>> = wait_until_stopped(&mut child, request.bound);
        let (stdout, stderr) = captured_output(&receiver);
        // The reader threads end with their streams; a join here would wait on a pipe another program still holds.
        drop(out_thread);
        drop(err_thread);
        let exit: std::process::ExitStatus = match waited {
            Ok(status) => status,
            Err(left_running) => {
                let mut message: String = format!(
                    "{} was still running after its bound of {} s and was stopped, so nothing it was doing finished",
                    request.line(),
                    request.bound.as_secs()
                );
                if let Some(note) = left_running {
                    message.push_str(&format!("; {note}"));
                }
                if !stderr.trim().is_empty() {
                    message.push_str(&format!(": {}", stderr.trim()));
                }
                return RunOutcome { stdout, stderr: message, code: 1 };
            }
        };
        return RunOutcome { stdout, stderr, code: exit.code().unwrap_or(1) };
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
