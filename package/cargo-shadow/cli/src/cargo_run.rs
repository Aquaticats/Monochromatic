//! What:
//! Runs the real cargo for the install and update steps:
//! a directory,
//! arguments,
//! a toolchain,
//! and the combined output.
//! Why:
//! The steps need to know whether cargo exited 0 and what it said,
//! and the tests need to replace cargo with a fake,
//! so the
//! runner is a trait.
//! The real runner never runs the shadow:
//! it takes the first `cargo` on PATH that is not this binary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cargo.ts
//! export type RunCargo = (request: { cwd: string; args: string[] }) => Promise<{ ok: boolean; output: string }>;
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::guard::{GUARD_VARIABLE, Guard, GuardKind, format_guard};

/// The outcome of one cargo run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoOutcome {
    /// Whether cargo exited 0.
    pub ok: bool,
    /// Standard output,
    /// then standard error.
    pub output: String,
    /// Standard output alone,
    /// for the callers that parse it.
    pub stdout: String,
}

/// What:
/// `pub trait CargoRunner { fn run(...) }` is a named bundle of one method (like a TS interface) that both the real cargo
/// and a test's fake implement.
/// Why:
/// The install algorithm is tested against a fake that records the commands it was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface CargoRunner { run(cwd: string, args: string[]): CargoOutcome }
/// ```
pub trait CargoRunner {
    /// Runs cargo in `cwd`;
    /// `Err` means cargo could not be started at all.
    fn run(&self, cwd: &Path, args: &[String]) -> Result<CargoOutcome, String>;
}

/// The real cargo under one rustup toolchain.
pub struct RealCargo {
    /// The cargo to start,
    /// as found on PATH (its own name is kept for rustup's sake).
    pub program: PathBuf,
    /// `RUSTUP_TOOLCHAIN` for the child;
    /// empty to inherit the caller's.
    pub toolchain: String,
}

impl CargoRunner for RealCargo {
    fn run(&self, cwd: &Path, args: &[String]) -> Result<CargoOutcome, String> {
        let mut command: Command = Command::new(&self.program);
        command.args(args).current_dir(cwd);
        // The loop guard (`guard.rs`): the candidate's resolved path joins the skip list of the instance it may start, so a
        // `cargo` that is a launcher leading back to the shadow is skipped there and the next `cargo` on PATH runs instead.
        // The value is bound to this process: only the direct child honors it, never a script the real cargo runs.
        let chosen: PathBuf = fs::canonicalize(&self.program).unwrap_or_else(|_| { return self.program.clone(); });
        let guard: Guard = Guard { kind: GuardKind::Spawn, pid: std::process::id(), hops: 1, skipped: vec![chosen] };
        command.env(GUARD_VARIABLE, format_guard(&guard));
        if !self.toolchain.is_empty() {
            command.env("RUSTUP_TOOLCHAIN", &self.toolchain);
        }
        let output = command.output().map_err(|error| { return format!("could not start {}: {error}", self.program.display()); })?;
        let stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
        let mut text: String = stdout.clone();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        return Ok(CargoOutcome { ok: output.status.success(), output: text, stdout });
    }
}

/// A closure with the runner's shape is a runner,
/// so a test passes `|cwd, args| ...`.
impl<F> CargoRunner for F
where
    F: Fn(&Path, &[String]) -> Result<CargoOutcome, String>,
{
    fn run(&self, cwd: &Path, args: &[String]) -> Result<CargoOutcome, String> {
        return self(cwd, args);
    }
}
