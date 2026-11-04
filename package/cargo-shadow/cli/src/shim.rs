//! What:
//! The shadow mode:
//! when the binary is started as `cargo`,
//! it warns about manifest problems in the workspace of the
//! current directory,
//! finds the real `cargo` on PATH,
//! runs it as a child with the arguments untouched,
//! waits for it,
//! and exits with the child's own status.
//! Why:
//! The warning never blocks.
//! A scan problem,
//! a missing workspace and a closed standard error all lead to the same
//! thing:
//! cargo runs.
//! Only a missing or unstartable cargo is the shadow's own failure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim.ts
//! export function runShim(args: string[]): number {
//!   warnAboutWorkspace();
//!   const real = findInPath('cargo', process.env.PATH ?? '', ownPath);
//!   const result = spawnSync(real, args, { stdio: 'inherit' });
//!   return result.status ?? 128 + signalNumber(result.signal);
//! }
//! ```

/// Imports the process-environment module (PATH,
/// the current directory).
use std::env;

/// Imports the owned operating-system text type:
/// arguments may be invalid UTF-8 and pass byte for byte.
use std::ffi::OsString;

/// Imports the file-system module (resolves the shadow's own path).
use std::fs;

/// Imports `Write`,
/// the trait that adds `write_all` to standard error.
use std::io::Write;

/// What:
/// `use std::os::unix::process::ExitStatusExt;` imports a trait that adds `.signal()` to a process's exit status.
/// Why:
/// A child killed by a signal has no exit code,
/// and the shadow reports it the way a shell does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Node reports `result.signal` directly.
/// ```
use std::os::unix::process::ExitStatusExt;

/// What:
/// `use std::os::unix::process::CommandExt;` imports a trait that adds `.exec()` to a command:
/// replace this process with the
/// program instead of starting it as a child.
/// Why:
/// A run that needs no notice hands over completely,
/// so the shadow's cost ends before cargo starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: Node cannot replace its own process image.
/// ```
use std::os::unix::process::CommandExt;

/// Imports the borrowed and owned path types.
use std::path::{Path, PathBuf};

/// Imports `Command` (build and start a program) and `ExitStatus` (how a finished program ended).
use std::process::{Command, ExitStatus};

/// Imports the fixed values.
use crate::constants::{EXEC_FAILED_EXIT_CODE, INSTALL_FAILED_EXIT_CODE, NOT_FOUND_EXIT_CODE, SHADOWED_NAME, SIGNAL_EXIT_CODE_OFFSET};
/// Imports the loop guard.
use crate::guard::{GUARD_VARIABLE, Guard, GuardKind, format_guard, is_meant_for, parse_guard};
/// Imports the notice handshake.
use crate::notice_marker::{claim_marker_in, leave_marker_in, runtime_dir};
/// Imports the PATH lookup.
use crate::real_binary::{find_in_path, own_real_path};
/// Imports the notice that follows cargo's version and help.
use crate::shadow_notice::{TopLevelRequest, notice_text, top_level_request};
/// Imports the command-word rules.
use crate::options::asserts_lock;
/// What:
/// `use crate::shadow_step::{Preflight, before_cargo, warn_after_cargo, with_locked};` imports `before_cargo`,
/// `warn_after_cargo`,
/// `with_locked` from another module of this crate so this file can name them unqualified.
/// Why:
/// `Preflight` is used by `run_shim`;
/// `before_cargo` is used by `run_shim`;
/// `warn_after_cargo` is used by `run_shim`;
/// `with_locked` is used by `run_shim`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Preflight, before_cargo, warn_after_cargo, with_locked } from 'crate/shadow_step';
/// ```
use crate::shadow_step::{Preflight, before_cargo, warn_after_cargo, with_locked};
/// What:
/// `use crate::shadow_update::run_update_command;` imports `run_update_command` from another module of this crate so this file can name it unqualified.
/// Why:
/// `run_update_command` is used by `run_shim`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { run_update_command } from 'crate/shadow_update';
/// ```
use crate::shadow_update::run_update_command;
/// What:
/// `use crate::verb::{Treatment, treatment_of};` imports `Treatment`,
/// `treatment_of` from another module of this crate so this file can name them unqualified.
/// Why:
/// `Treatment` is used by `run_shim`;
/// `treatment_of` is used by `run_shim`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Treatment, treatment_of } from 'crate/verb';
/// ```
use crate::verb::{Treatment, treatment_of};

/// What:
/// `pub fn exit_code_of(status: ExitStatus) -> i32` turns a finished child's status into the number the shadow exits with.
/// Why:
/// A normal exit passes its code through;
/// death by signal becomes `128 + signal`,
/// the shell convention.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function exitCodeOf(result: SpawnSyncReturns): number { return result.status ?? 128 + os.constants.signals[result.signal]; }
/// ```
pub fn exit_code_of(status: ExitStatus) -> i32 {
    // `status.code()` is `Some(code)` for a normal exit and `None` after a signal.
    if let Some(code) = status.code() {
        return code;
    }
    if let Some(signal) = status.signal() {
        return SIGNAL_EXIT_CODE_OFFSET + signal;
    }
    return EXEC_FAILED_EXIT_CODE;
}

/// What:
/// `pub fn run_program(program: &Path, arguments: &[OsString]) -> std::io::Result<i32>` returns `Ok(code)` when the
/// program ran to the end and `Err(error)` when it could not be started.
/// Why:
/// The caller reports a start failure with the right exit code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runProgram(program: string, args: string[]): number  // throws when the spawn fails
/// ```
pub fn run_program(program: &Path, arguments: &[OsString], guard_value: Option<&str>) -> std::io::Result<i32> {
    // `Command::new(program).args(arguments)` builds the command; the guard, when there is one, is added to its environment
    // only (this process's own environment is untouched). `.status()` starts it with the standard streams inherited from
    // this process and waits for it; `?` leaves the function with the `Err` when the start failed.
    crate::real_binary::refuse_if_ourselves(program).map_err(std::io::Error::other)?;
    let mut command: Command = Command::new(program);
    command.args(arguments);
    if let Some(value) = guard_value {
        command.env(GUARD_VARIABLE, value);
    }
    let status: ExitStatus = command.status()?;
    return Ok(exit_code_of(status));
}

/// What:
/// `pub fn run_shim(arguments: Vec<OsString>) -> i32` takes the arguments cargo was started with (an owned list) and
/// returns the exit code.
/// Why:
/// `main.rs` calls it when the binary was started under the name `cargo`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runShim(args: Buffer[]): number
/// ```
pub fn run_shim(mut arguments: Vec<OsString>) -> i32 {
    // The loop guard (`guard.rs`): a value an earlier shadow instance left for the process its candidate started. It counts
    // only when it was left for this very process, so a script that the real cargo runs never inherits it.
    let mut skipped: Vec<PathBuf> = Vec::new();
    let hops: u32 = 0;
    // The process id of the shadow instance that started this one as a child, when there is one (a spawn guard).
    let outer_pid: Option<u32> = None;
    // A guard meant for this process says that another cargo-shadow chose the file this process runs as the real cargo: this one is
    // that second start. It stops at once, before doing any work, with the message that tells a reader with no context what happened and
    // what to type (owner decision of 2026-11-04, option A); there is no second try with another file.
    if let Ok(text) = env::var(GUARD_VARIABLE)
        && let Some(guard) = parse_guard(&text)
        && is_meant_for(&guard, std::process::id(), std::os::unix::process::parent_id())
    {
        let own: PathBuf = env::current_exe().unwrap_or_default();
        eprint!("{}", crate::arrival::arrival_message(&guard, &own, &env::var("PATH").unwrap_or_default()));
        return crate::arrival::ARRIVAL_EXIT_CODE;
    }

    // The treatment of the command line (`verb.rs`). Only the first instance acts: an instance started by an earlier one
    // (hops above zero) has already had its install step run. The install step comes first and its messages go to standard
    // error, so cargo's standard output stays exactly cargo's.
    let treatment: Treatment = treatment_of(&arguments);
    if hops == 0
        && treatment == Treatment::PnpmUpdate
        && let Some(code) = run_update_command(&arguments)
    {
        return code;
    }
    // The install step failed for an operational reason (cargo or the registry could not be asked, or a frozen check found a
    // stale lockfile): pnpm warns and runs the command anyway ("If that install fails ... pnpm prints a warning and runs the
    // command anyway", https://pnpm.io/settings/build). Cargo rewrites a stale lock, so the command runs with `--locked` added
    // (unless the user already passed it), and cargo stops with its own lock error where it would have changed the lock. A
    // policy refusal, frozen or local, is different: `--locked` would not stop the build script of a refused release, so
    // `Preflight::Refused` stops the command (below). `arguments` is replaced for the rest of the run, so the notice and
    // guard logic see the same line.
    // The command the user ran, so an error that says "run it again" names it (review finding 4c P3).
    crate::bounded_run::set_rerun(Some(rerun_command(&arguments)));
    if hops == 0
        && treatment == Treatment::InstallStep
        // `Some`: the present or the absent value of an `Option`.
        // Borrow `&arguments`: lends the value without copying or giving it away.
        && let Some(preflight) = before_cargo(&arguments)
    {
        // A local policy refusal stops the command: a release the policy refuses must never run, and `--locked` would not
        // stop its build script (the owner's decision of 2026-11-03).
        let message: String = match preflight {
            Preflight::Refused(text) => {
                // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
                // Why:      Text building and printing are written as one call instead of string concatenation.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // `..${x}..`   // template literal
                // console.error(text);   // eprintln!
                // [a, b]            // vec![a, b]
                // ```
                eprintln!("{text}");
                // Macro `eprintln!`: builds or prints text (or a list) from a template.
                eprintln!("cargo-shadow: the command was not run: the install step stopped it, for the reason given above");
                return INSTALL_FAILED_EXIT_CODE;
            }
            Preflight::Failed(text) => text,
        };
        eprintln!("{message}");
        if !asserts_lock(&arguments) {
            eprintln!("cargo-shadow: warning: the install step did not complete, so `--locked` is added to your cargo command; cargo stops with its own lock error where it would have changed Cargo.lock");
        }
        arguments = with_locked(&arguments);
    }

    let path_variable: String = env::var("PATH").unwrap_or_default();
    let mut skip_list: Vec<PathBuf> = vec![own_real_path()];
    skip_list.extend(skipped.iter().cloned());
    let program: PathBuf = match find_in_path(SHADOWED_NAME, &path_variable, &skip_list) {
        Some(found) => found,
        None => {
            eprintln!("cargo-shadow: no real {SHADOWED_NAME} found on PATH");
            return NOT_FOUND_EXIT_CODE;
        }
    };
    let request: TopLevelRequest = top_level_request(&arguments);

    // The candidate's resolved path joins the skip list for the next instance: if the candidate is a launcher or another
    // link that leads back to the shadow, that instance skips it and finds the next `cargo`, and if it is the real cargo,
    // the value is bound to this process and reaches nobody else.
    let chosen: PathBuf = match fs::canonicalize(&program) {
        Ok(resolved) => resolved,
        Err(_) => program.clone(),
    };
    skipped.push(chosen);
    // Review finding (recursion guard kind): the guard kind must follow the way the program is really launched below, not the
    // notice classification. An `Other` request that has a catalog notice to print afterwards (`add`, `remove`) is started as a
    // child, so its guard is the spawn kind; only the exec branch below leaves an exec guard.
    let will_exec: bool = request == TopLevelRequest::Other && treatment != Treatment::CatalogWarning;
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Typed binding: `guard_kind: GuardKind`.
    let guard_kind: GuardKind = if will_exec { GuardKind::Exec } else { GuardKind::Spawn };
    let guard_value: String = format_guard(&Guard { kind: guard_kind, pid: std::process::id(), hops: hops + 1, skipped });

    // What:     `Command::new(&program).args(&arguments).exec()` replaces this process with cargo: same process id, same
    //           standard streams, and the exit status and signals are cargo's own because nothing of the shadow is left to
    //           report them. `exec()` returns only when it failed, so the value it returns is always an error.
    // Why:      The shadow adds nothing after a run that needs no notice, so it must not stay resident to wait for cargo.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // execvp(program, args);  // never returns on success; Node has no exec, this is the POSIX call
    // ```
    if will_exec {
        if let Err(reason) = crate::real_binary::refuse_if_ourselves(&program) {
            eprintln!("cargo-shadow: {reason}");
            return EXEC_FAILED_EXIT_CODE;
        }
        let error: std::io::Error = Command::new(&program).args(&arguments).env(GUARD_VARIABLE, &guard_value).exec();
        eprintln!("cargo-shadow: could not start {}: {error}", program.display());
        return EXEC_FAILED_EXIT_CODE;
    }

    // A version or help request keeps the shadow alive until cargo finishes, because the notice follows cargo's text.
    // The notice handshake (`notice_marker.rs`): clear a marker an earlier run with this process id may have left, then
    // after cargo finishes check whether an instance below this one already printed the notice.
    let own_pid: u32 = std::process::id();
    let runtime: Option<PathBuf> = runtime_dir();
    if let Some(dir) = &runtime {
        let _ = claim_marker_in(dir, own_pid);
    }
    match run_program(&program, &arguments, Some(&guard_value)) {
        Ok(code) => {
            // After `cargo add` or `cargo remove` succeeded, warn about dependencies that do not use the catalog; the exit
            // code stays cargo's.
            if code == 0 && treatment == Treatment::CatalogWarning && hops == 0 {
                warn_after_cargo(&arguments, &mut std::io::stderr());
            }
            // Like hub after git's version: only after a successful top-level `--version` or `--help` of cargo itself,
            // so the notice follows cargo's own text and is the one place the shadow names itself. A closed standard
            // output is ignored on purpose: cargo's output already went out. When the candidate was a launcher that
            // started another shadow instance, that inner instance knows which cargo really ran and printed the notice;
            // this instance then stays silent, and passes the fact on to the instance above it, if there is one.
            if code == 0
                && let Some(text) = notice_text(request, &program)
            {
                let printed_below: bool = match &runtime {
                    Some(dir) => claim_marker_in(dir, own_pid),
                    None => false,
                };
                if !printed_below {
                    let _ = std::io::stdout().write_all(text.as_bytes());
                }
                if let Some(dir) = &runtime
                    && let Some(outer) = outer_pid
                {
                    leave_marker_in(dir, outer);
                }
            }
            return code;
        }
        Err(error) => {
            eprintln!("cargo-shadow: could not start {}: {error}", program.display());
            return EXEC_FAILED_EXIT_CODE;
        }
    }
}

/// What:
/// `pub(crate) fn rerun_command(arguments: &[OsString]) -> String` is the command line the user ran,
/// for errors that say "run it again".
/// Why:
/// Round 6 (5c N5):
/// arguments keep their boundaries,
/// so the text can be copied into a shell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function rerunCommand(args: string[]): string
/// ```
pub(crate) fn rerun_command(arguments: &[std::ffi::OsString]) -> String {
    // POSIX shell quoting: a word of plain characters stays as it is; anything else is single-quoted, with each single quote
    // written as close-quote, escaped quote, open-quote. Bytes that are not text are shown as the replacement character.
    let word = |argument: &std::ffi::OsString| -> String {
        let text: String = argument.to_string_lossy().to_string();
        let plain: bool = !text.is_empty() && text.chars().all(|character| { return character.is_ascii_alphanumeric() || "_@%+=:,./-".contains(character); });
        return if plain { text } else { format!("'{}'", text.replace('\'', "'\\''")) };
    };
    return format!("cargo {}", arguments.iter().map(word).collect::<Vec<String>>().join(" "));
}
