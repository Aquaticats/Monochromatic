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
/// Imports the bounded child run that every cargo call goes through.
use crate::bounded_run::{Bound, RunError, output_within};

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
    /// The longest one cargo call may take (owner rule of 2026-11-03:
    /// "nothing should ever wait forever or indefinitely");
    /// at the bound cargo is stopped and the error names the call and the command to run again.
    pub bound: std::time::Duration,
}

/// The default bound of one cargo call,
/// in seconds.
/// Measured 2026-11-04 on this repository (load average 6.2 to 6.9 on 32 CPUs,
/// capped scopes,
/// 3 runs each,
/// `~/temp/agent/bench_bounds.py` and `bench_online.py`):
/// `cargo update --workspace --locked --offline` took 0.11 s to 0.91 s and `cargo metadata --locked --offline` 0.05 s to 0.65 s
/// (slowest:
/// `package/music-player/desktop-app`,
/// the largest lockfile here);
/// an online `cargo update --workspace --dry-run` took 0.09 s to 0.12 s.
/// Half an hour is about 2,000 times the slowest measured call and is meant for a cold index fetch over a slow network,
/// which was not measured:
/// a run that resolves with an empty registry cache (and so shows the real upper end) is what must be measured to set this number.
/// The owner's rule of 2026-11-03 ("nothing should ever wait forever or indefinitely") justifies that a bound exists;
/// the number is a Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open.
pub const CARGO_CALL_SECONDS: u64 = 1_800;

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
        let vetted = crate::real_binary::vet(&self.program)?;
        // The runner below puts the program inside a scope whose launcher starts it from its own process, so only a path names it there.
        let mut command: Command = vetted.command_by_path()?;
        command.args(args).current_dir(cwd);
        // The loop guard (`guard.rs`): the candidate's resolved path joins the skip list of the instance it may start, so a
        // `cargo` that is a launcher leading back to the shadow is skipped there and the next `cargo` on PATH runs instead.
        // The value is bound to this process: only the direct child honors it, never a script the real cargo runs.
        let chosen: PathBuf = match fs::canonicalize(&self.program) {
            Ok(resolved) => resolved,
            Err(_) => self.program.clone(),
        };
        let guard: Guard = Guard { kind: GuardKind::Spawn, pid: std::process::id(), hops: 1, skipped: vec![chosen] };
        command.env(GUARD_VARIABLE, format_guard(&guard));
        if !self.toolchain.is_empty() {
            command.env("RUSTUP_TOOLCHAIN", &self.toolchain);
        }
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        // Typed binding: `bound: Bound`.
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        let bound: Bound = Bound { limit: self.bound, what: format!("`cargo {}`", args.join(" ")), rerun: crate::bounded_run::rerun_text() };
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&mut command`: lends the value without copying or giving it away.
        let output = match output_within(&mut command, None, &bound) {
            Ok(found) => found,
            // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
            // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Ok(value)  is  return value;
            // // Err(error) is  throw error;   (the caller sees it as a returned value)
            // ```
            // Macro `format!`: builds or prints text (or a list) from a template.
            Err(RunError::Io(error)) => return Err(format!("could not start {}: {error}", self.program.display())),
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            Err(timed_out) => return Err(timed_out.to_string()),
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
