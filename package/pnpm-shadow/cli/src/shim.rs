//! What:
//! The shadow mode:
//! when the binary is started as `pnpm`,
//! it warns about manifest problems in the workspace of the
//! current directory,
//! finds the real `pnpm` on PATH,
//! runs it as a child with the arguments untouched,
//! waits for it,
//! and exits with the child's own status.
//! Why:
//! The warning never blocks.
//! A scan problem,
//! a missing workspace and a closed standard error all lead to the same
//! thing:
//! pnpm runs.
//! Only a missing or unstartable pnpm is the shadow's own failure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim.ts
//! export function runShim(args: string[]): number {
//!   warnAboutWorkspace();
//!   const real = findInPath('pnpm', process.env.PATH ?? '', ownPath);
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
/// so the shadow's cost ends before pnpm starts.
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
use crate::constants::{EXEC_FAILED_EXIT_CODE, NOT_FOUND_EXIT_CODE, SHADOWED_NAME, SIGNAL_EXIT_CODE_OFFSET};
/// Imports the loop guard.
use crate::guard::{GUARD_VARIABLE, Guard, GuardKind, MAX_HOPS, format_guard, is_meant_for, parse_guard};
/// Imports the notice handshake.
use crate::notice_marker::{claim_marker_in, leave_marker_in, runtime_dir};
/// Imports the PATH lookup.
use crate::real_binary::find_in_path;
/// Imports the report renderer.
use crate::report::render_report;
/// Imports the workspace scan.
use crate::scan::{find_workspace_root, scan_workspace};
use crate::shadow_identity::{ARRIVAL_EXIT_CODE, StartMode, Vetted, arrival_at_this_process, arrival_line, mark_start, vet};
/// Imports the notice that follows pnpm's version and help.
use crate::shadow_notice::{TopLevelRequest, notice_text, top_level_request};
/// Imports the rule that says whether the manifest warning makes sense for a command line.
use crate::verb::warning_makes_sense;

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
    // The program is checked while open and started through that file, and the child carries the arrival marker (`shadow_identity.rs`).
    let vetted: Vetted = vet(program).map_err(std::io::Error::other)?;
    let mut command: Command = vetted.command();
    command.args(arguments);
    if let Some(value) = guard_value {
        command.env(GUARD_VARIABLE, value);
    }
    mark_start(&mut command, StartMode::Spawn);
    let status: ExitStatus = command.status()?;
    return Ok(exit_code_of(status));
}

/// What:
/// `fn own_real_path() -> PathBuf` is the shadow's own resolved executable path,
/// or an empty path when the system cannot say.
/// Why:
/// The PATH search skips every entry that resolves to it;
/// an empty path matches nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownRealPath(): string { return fs.realpathSync(process.execPath); }
/// ```
fn own_real_path() -> PathBuf {
    if let Ok(executable) = env::current_exe() {
        if let Ok(resolved) = fs::canonicalize(&executable) {
            return resolved;
        }
        return executable;
    }
    return PathBuf::new();
}

/// What:
/// `pub fn warn_about_workspace(start: &Path, sink: &mut dyn Write)` scans the workspace that contains `start` and
/// writes the report to `sink` when there is one.
/// Why:
/// The sink is a parameter so tests read the text;
/// the shadow passes standard error.
/// `&mut dyn Write` is a borrowed
/// writer of any type (a TS `Writable`).
/// A write error is ignored on purpose:
/// a closed standard error must not stop pnpm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function warnAboutWorkspace(start: string, sink: Writable): void
/// ```
pub fn warn_about_workspace(start: &Path, sink: &mut dyn Write) {
    // No `pnpm-workspace.yaml` above `start` means there is no workspace to check; pnpm runs without a word.
    let root: PathBuf = match find_workspace_root(start) {
        Some(found) => found,
        None => return,
    };
    let report = scan_workspace(&root);
    if let Some(text) = render_report(&root, &report) {
        // `let _ =` discards the write result on purpose.
        let _ = sink.write_all(text.as_bytes());
    }
}

/// What:
/// `pub fn run_shim(arguments: Vec<OsString>) -> i32` takes the arguments pnpm was started with (an owned list) and
/// returns the exit code.
/// Why:
/// `main.rs` calls it when the binary was started under the name `pnpm`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runShim(args: Buffer[]): number
/// ```
pub fn run_shim(arguments: Vec<OsString>) -> i32 {
    // The arrival stop comes before anything else: a shadow that a shadow started as its real pnpm does nothing and says so in one line
    // (`shadow_identity.rs`), whatever the loop guard below would have said.
    if let Some(arrival) = arrival_at_this_process() {
        eprintln!("{}", arrival_line(SHADOWED_NAME, &arrival));
        return ARRIVAL_EXIT_CODE;
    }
    // The loop guard (`guard.rs`): a value an earlier shadow instance left for the process its candidate started. It counts
    // only when it was left for this very process, so a script that the real pnpm runs never inherits it.
    let mut skipped: Vec<PathBuf> = Vec::new();
    let mut hops: u32 = 0;
    // The process id of the shadow instance that started this one as a child, when there is one (a spawn guard).
    let mut outer_pid: Option<u32> = None;
    if let Ok(text) = env::var(GUARD_VARIABLE)
        && let Some(guard) = parse_guard(&text)
        && is_meant_for(&guard, std::process::id(), std::os::unix::process::parent_id())
    {
        if guard.kind == GuardKind::Spawn {
            outer_pid = Some(guard.pid);
        }
        skipped = guard.skipped;
        hops = guard.hops;
    }
    if hops >= MAX_HOPS {
        eprintln!(
            "pnpm-shadow: stopped after {hops} hops: the {SHADOWED_NAME} found on PATH keeps leading back to this shadow (tried {})",
            skipped.iter().map(|path| { return path.display().to_string(); }).collect::<Vec<String>>().join(", ")
        );
        return EXEC_FAILED_EXIT_CODE;
    }

    // The scope is the user's: the warning only where it makes sense, which is a command that resolves or changes
    // dependencies (`verb.rs`). Every other command line, `pnpm --version` and `pnpm run` among them, does no scan at all,
    // so the wrapper adds only its fixed cost. The warning comes first and goes to standard error, so pnpm's standard
    // output stays exactly pnpm's. An instance started by an earlier instance (hops above zero) does not warn again.
    if hops == 0
        && warning_makes_sense(&arguments)
        && let Ok(current) = env::current_dir()
    {
        warn_about_workspace(&current, &mut std::io::stderr());
    }

    let path_variable: String = env::var("PATH").unwrap_or_default();
    let mut skip_list: Vec<PathBuf> = vec![own_real_path()];
    skip_list.extend(skipped.iter().cloned());
    let program: PathBuf = match find_in_path(SHADOWED_NAME, &path_variable, &skip_list) {
        Some(found) => found,
        None => {
            eprintln!("pnpm-shadow: no real {SHADOWED_NAME} found on PATH");
            return NOT_FOUND_EXIT_CODE;
        }
    };
    let request: TopLevelRequest = top_level_request(&arguments);

    // The candidate's resolved path joins the skip list for the next instance: if the candidate is a launcher or another
    // link that leads back to the shadow, that instance skips it and finds the next `pnpm`, and if it is the real pnpm,
    // the value is bound to this process and reaches nobody else.
    let chosen: PathBuf = match fs::canonicalize(&program) {
        Ok(resolved) => resolved,
        Err(_) => program.clone(),
    };
    skipped.push(chosen);
    let guard_kind: GuardKind = if request == TopLevelRequest::Other { GuardKind::Exec } else { GuardKind::Spawn };
    let guard_value: String = format_guard(&Guard { kind: guard_kind, pid: std::process::id(), hops: hops + 1, skipped });

    // What:     `Command::new(&program).args(&arguments).exec()` replaces this process with pnpm: same process id, same
    //           standard streams, and the exit status and signals are pnpm's own because nothing of the shadow is left to
    //           report them. `exec()` returns only when it failed, so the value it returns is always an error.
    // Why:      The shadow adds nothing after a run that needs no notice, so it must not stay resident to wait for pnpm.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // execvp(program, args);  // never returns on success; Node has no exec, this is the POSIX call
    // ```
    if request == TopLevelRequest::Other {
        let vetted: Vetted = match vet(&program) {
            Ok(checked) => checked,
            Err(reason) => {
                eprintln!("pnpm-shadow: could not start {}: {reason}", program.display());
                return EXEC_FAILED_EXIT_CODE;
            }
        };
        let mut command: Command = vetted.command();
        command.args(&arguments).env(GUARD_VARIABLE, &guard_value);
        mark_start(&mut command, StartMode::Exec);
        let error: std::io::Error = command.exec();
        eprintln!("pnpm-shadow: could not start {}: {error}", program.display());
        return EXEC_FAILED_EXIT_CODE;
    }

    // A version or help request keeps the shadow alive until pnpm finishes, because the notice follows pnpm's text.
    // The notice handshake (`notice_marker.rs`): clear a marker an earlier run with this process id may have left, then
    // after pnpm finishes check whether an instance below this one already printed the notice.
    let own_pid: u32 = std::process::id();
    let runtime: Option<PathBuf> = runtime_dir();
    if let Some(dir) = &runtime {
        let _ = claim_marker_in(dir, own_pid);
    }
    match run_program(&program, &arguments, Some(&guard_value)) {
        Ok(code) => {
            // Like hub after git's version: only after a successful top-level `--version` or `--help` of pnpm itself,
            // so the notice follows pnpm's own text and is the one place the shadow names itself. A closed standard
            // output is ignored on purpose: pnpm's output already went out. When the candidate was a launcher that
            // started another shadow instance, that inner instance knows which pnpm really ran and printed the notice;
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
            eprintln!("pnpm-shadow: could not start {}: {error}", program.display());
            return EXEC_FAILED_EXIT_CODE;
        }
    }
}
