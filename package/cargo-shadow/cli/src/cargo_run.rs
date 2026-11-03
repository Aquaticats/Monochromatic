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

/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fs } from 'std';
/// ```
use std::fs;
/// What:
/// `use std::path::{Path, PathBuf};` imports `Path`,
/// `PathBuf` from the standard library so this file can name them unqualified.
/// Why:
/// `Path` is used by `run`,
/// the `impl` of `F`;
/// `PathBuf` is used by `RealCargo`,
/// `run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path, PathBuf } from 'std/path';
/// ```
use std::path::{Path, PathBuf};
/// What:
/// `use std::process::Command;` imports `Command` from the standard library so this file can name it unqualified.
/// Why:
/// `Command` is used by `run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command } from 'std/process';
/// ```
use std::process::Command;

/// What:
/// `use crate::guard::{GUARD_VARIABLE, Guard, GuardKind, format_guard};` imports `GUARD_VARIABLE`,
/// `Guard`,
/// `GuardKind`,
/// `format_guard` from another module of this crate so this file can name them unqualified.
/// Why:
/// `GUARD_VARIABLE` is used by `run`;
/// `Guard` is used by `run`;
/// `GuardKind` is used by `run`;
/// `format_guard` is used by `run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { GUARD_VARIABLE, Guard, GuardKind, format_guard } from 'crate/guard';
/// ```
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

/// What:
/// `impl CargoRunner for RealCargo { ... }` makes the real cargo usable wherever a `CargoRunner` is asked for (a trait is a TS interface,
/// an `impl` block the class that implements it).
/// Why:
/// The install and update steps take any runner,
/// so the same code runs the real cargo in production and a fake in the tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RealCargo implements CargoRunner { run(cwd, args) { ... } }
/// ```
impl CargoRunner for RealCargo {
    /// What:
    /// `fn run(&self, cwd, args)` starts the real cargo in `cwd` with `args` and returns whether it exited 0 and what it printed;
    /// `&self` borrows the runner read-only.
    /// Why:
    /// Every cargo call of the install and the update goes through it,
    /// so the pinned toolchain and the loop guard are set in one place.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// run(cwd: string, args: string[]): CargoOutcome  // throws if cargo cannot start
    /// ```
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
        let output = match command.output() {
            Ok(found) => found,
            Err(error) => return Err(format!("could not start {}: {error}", self.program.display())),
        };
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
    /// What:
    /// `fn run(&self, cwd, args)` of the closure runner calls the closure itself with the same arguments.
    /// Why:
    /// A test passes `|cwd, args| ...` as the cargo to use,
    /// which is shorter than defining a struct per fake.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// run(cwd: string, args: string[]) { return fn(cwd, args); }
    /// ```
    fn run(&self, cwd: &Path, args: &[String]) -> Result<CargoOutcome, String> {
        return self(cwd, args);
    }
}
