//! What:     The seam through which every command starts another program (`cargo`, `rustup`, `uv`, `pnpm`, `bin`,
//!           `tar`, `unzip`, `gh`): a `Runner` trait, the real implementation on `std::process::Command`, and the
//!           environment those programs are given.
//! Why:      Tests pass a recording fake, so a provider's tests assert the exact arguments it builds without
//!           installing anything or touching the real host.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // runner.ts
//! export type RunRequest = { command: string; args: string[]; env?: Record<string, string>; cwd?: string };
//! export type RunOutcome = { stdout: string; stderr: string; code: number };
//! export interface Runner { run(request: RunRequest): RunOutcome }
//! ```

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::layout::manager_native_bin_dirs;
use super::scope::{EnvMap, Scope};

/// What:     One program start: the program (a name looked up on `PATH`, or a path), its arguments, an optional full
///           environment and an optional working directory.
/// Why:      A plain value, so a fake can record it and a test can compare it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RunRequest = { command: string; args: string[]; env?: Record<string, string>; cwd?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunRequest {
    /// The program name or path.
    pub command: String,
    /// The arguments, one per element, never split by a shell.
    pub args: Vec<String>,
    /// The complete environment of the child; `None` means it inherits this process's.
    pub env: Option<BTreeMap<String, String>>,
    /// The working directory; `None` means this process's.
    pub cwd: Option<PathBuf>,
}

impl RunRequest {
    /// A request with the process's own environment and working directory.
    pub fn new(command: &str, args: &[&str]) -> RunRequest {
        return RunRequest {
            command: command.to_string(),
            args: args.iter().map(|argument| return (*argument).to_string()).collect(),
            env: None,
            cwd: None,
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

    /// The request as one line (`cargo binstall --no-confirm ...`), for logs and test assertions.
    pub fn line(&self) -> String {
        let mut line: String = self.command.clone();
        for argument in &self.args {
            line.push(' ');
            line.push_str(argument);
        }
        return line;
    }
}

/// What a finished program printed and how it ended; a program that could not start has code 1 and the reason in
/// `stderr`, so callers handle one shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunOutcome {
    /// Standard output as text (invalid UTF-8 replaced).
    pub stdout: String,
    /// Standard error as text.
    pub stderr: String,
    /// The exit code; 1 when the program could not start or was killed by a signal.
    pub code: i32,
}

impl RunOutcome {
    /// Whether the program exited with code 0.
    pub fn succeeded(&self) -> bool {
        return self.code == 0;
    }

    /// Standard error if it printed any, otherwise standard output: the text to show when a command failed.
    pub fn failure_text(&self) -> String {
        if self.stderr.trim().is_empty() {
            return self.stdout.trim().to_string();
        }
        return self.stderr.trim().to_string();
    }
}

/// What:     The seam: something that can start a program and report what it did.
/// Why:      `&self` and not `&mut self` so one runner is shared by every provider; the fake records through interior
///           mutability.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Runner { run(request: RunRequest): RunOutcome }
/// ```
pub trait Runner {
    /// Starts the program, waits for it and reports the outcome.
    fn run(&self, request: &RunRequest) -> RunOutcome;
}

/// The real runner: `std::process::Command` with captured output and no standard input.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessRunner;

impl Runner for ProcessRunner {
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
        return match command.output() {
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

/// What:     The environment tool installs run with: the whole given environment, `HOME` set to the scope's home, and
///           `PATH` led by the scope's exposure, toolchain, cargo, uv, pnpm and npm bin directories.
/// Why:      `cargo binstall`, `pnpm add --global` and `uv tool install` find each other and the freshly installed
///           tools through `PATH`, and must write into the scope's home.
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
