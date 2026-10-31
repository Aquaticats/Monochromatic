//! What:
//! Moves `sync` and `update` into the clearnet updates slice.
//! Why:
//! `wg-quicker-exempt` attaches its socket-marking cgroup programs to that slice,
//! while a WireGuard tunnel with `ExemptMark` is up,
//! so every socket a run and its children open takes the physical route instead of the tunnel.
//! Those children are cargo-binstall,
//! cargo,
//! rustup,
//! uv,
//! pnpm,
//! node fetch
//! and curl.
//! The TypeScript entry points in `package/file-enforcer/core/src/toolchain/updates-slice.ts` do the same,
//! and share the slice name and both environment variables,
//! so a run that already moved is never moved twice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // updates-slice.ts
//! export function relaunchIntoUpdatesSlice(env: EnvMap): number | undefined
//! ```

/// Imports `ExitStatusExt` from `std::os::unix::process`,
/// which reads the signal that ended a child.
use std::os::unix::process::ExitStatusExt;
/// Imports `Command`,
/// `ExitStatus` and `Stdio` from `std::process`.
use std::process::{Command, ExitStatus, Stdio};

/// Imports the environment type from `super::scope`.
use super::scope::EnvMap;

/// What:
/// The slice `wg-quicker-exempt` matches exactly.
/// Its constant is `TOOLCHAIN_UPDATES_SLICE` in `package/wg/cli-quicker-exempt/src/application_targets.rs`.
/// Why:
/// The `app-` prefix nests it directly under the user's `app.slice`,
/// where the watcher scans.
/// The `\x2d` is the escaped literal dash,
/// so the name is one hierarchy level.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const TOOLCHAIN_UPDATES_SLICE = String.raw`app-monochromatic\x2dupdates.slice`;
/// ```
pub const TOOLCHAIN_UPDATES_SLICE: &str = "app-monochromatic\\x2dupdates.slice";

/// The variable a re-executed child carries,
/// so it never re-executes again.
/// It is the TypeScript entry points' guard too.
pub const GUARD_ENVIRONMENT: &str = "MONOCHROMATIC_TOOLCHAIN_IN_UPDATES_SLICE";

/// The variable whose value `off` keeps a run in place.
pub const OPT_OUT_ENVIRONMENT: &str = "MONOCHROMATIC_TOOLCHAIN_CLEARNET";

/// The exit code a command that could not be run reports,
/// by shell convention.
const COMMAND_NOT_RUNNABLE_EXIT_CODE: i32 = 127;

/// The offset a shell adds to a signal number to report death by signal.
const SIGNAL_EXIT_CODE_OFFSET: i32 = 128;

/// What:
/// The facts the placement decision reads.
/// Why:
/// A decision over plain values is testable without a user manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UpdatesSliceFacts = { onLinux: boolean; env: EnvMap; cgroupText: string; userManagerPresent: boolean };
/// ```
pub struct Facts<'a> {
    /// Whether this process runs on Linux.
    pub on_linux: bool,
    /// The process environment.
    pub env: &'a EnvMap,
    /// The contents of `/proc/self/cgroup`,
    /// empty when unreadable.
    pub cgroup_text: &'a str,
    /// Whether the user's systemd manager socket exists.
    pub user_manager_present: bool,
}

/// What:
/// Whether the unified cgroup path of a `/proc/self/cgroup` text already contains the slice as one segment.
/// Why:
/// A run already inside the slice must not move again,
/// and a lookalike segment must not count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function insideUpdatesSlice(cgroupText: string): boolean
/// ```
pub fn inside_updates_slice(cgroup_text: &str) -> bool {
    for line in cgroup_text.lines() {
        if let Some(path) = line.strip_prefix("0::")
            && path.split('/').any(is_updates_slice)
        {
            return true;
        }
    }
    return false;
}

/// Whether a cgroup path segment is the updates slice.
fn is_updates_slice(segment: &str) -> bool {
    return segment == TOOLCHAIN_UPDATES_SLICE;
}

/// What:
/// Why a run stays in place,
/// or `None` when it should move into the slice.
/// Why:
/// The reason lets a run that stays in place explain itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function stayInPlaceReason(facts: UpdatesSliceFacts): string | undefined
/// ```
pub fn stay_in_place_reason(facts: &Facts<'_>) -> Option<&'static str> {
    if !facts.on_linux {
        return Some("this platform has no user slices");
    }
    if facts.env.get(GUARD_ENVIRONMENT).is_some() {
        return Some("this run was already re-executed into the slice");
    }
    if facts.env.get(OPT_OUT_ENVIRONMENT) == Some("off") {
        return Some("the clearnet opt-out is set");
    }
    if inside_updates_slice(facts.cgroup_text) {
        return Some("already inside the updates slice");
    }
    if !facts.user_manager_present {
        return Some("no user systemd manager socket, so this is a container or a host without systemd");
    }
    return None;
}

/// What:
/// The `systemd-run` arguments that place a command in a fresh user scope below the slice.
/// Why:
/// One builder serves the probe and the real run,
/// so they cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function systemdRunArguments(command: string[]): string[]
/// ```
pub fn systemd_run_arguments(command: &[String]) -> Vec<String> {
    let mut arguments: Vec<String> = vec![
        "--user".to_string(),
        "--scope".to_string(),
        "--quiet".to_string(),
        "--collect".to_string(),
        format!("--slice={TOOLCHAIN_UPDATES_SLICE}"),
        "--".to_string(),
    ];
    arguments.extend(command.iter().cloned());
    return arguments;
}

/// Converts a child's exit status into one exit code:
/// the code,
/// or 128 plus the signal number after a signal.
fn exit_code_of(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    return SIGNAL_EXIT_CODE_OFFSET + status.signal().unwrap_or(0);
}

/// What:
/// Probes `program`,
/// then re-runs `arguments` through it inside the slice and returns that run's exit code.
/// Why:
/// `systemd-run` reports its own failures with the exit code a failing sync uses,
/// so a silent probe first proves a user manager accepts the scope.
/// A failed probe leaves the run in place,
/// and nothing has been run twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runThroughSystemd(program: string, executable: string, args: string[], env: EnvMap): number | undefined
/// ```
pub fn run_through_systemd(program: &str, executable: &str, arguments: &[String], env: &EnvMap) -> Option<i32> {
    let mut command: Vec<String> = vec![executable.to_string()];
    command.extend(arguments.iter().cloned());
    let probe = Command::new(program)
        .args(systemd_run_arguments(&[executable.to_string(), "--version".to_string()]))
        .envs(env.all())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match probe {
        Ok(status) if status.success() => {}
        Ok(status) => {
            eprintln!(
                "mpm-shadow: systemd-run could not place a scope in {TOOLCHAIN_UPDATES_SLICE}, exit {}; update traffic will follow the default route",
                exit_code_of(status)
            );
            return None;
        }
        Err(error) => {
            eprintln!("mpm-shadow: cannot run {program}: {error}; update traffic will follow the default route");
            return None;
        }
    }
    return match Command::new(program)
        .args(systemd_run_arguments(&command))
        .envs(env.all())
        .env(GUARD_ENVIRONMENT, "1")
        .status()
    {
        Ok(status) => Some(exit_code_of(status)),
        Err(error) => {
            eprintln!("mpm-shadow: cannot run {program}: {error}");
            Some(COMMAND_NOT_RUNNABLE_EXIT_CODE)
        }
    };
}

/// What:
/// Re-executes this process through `systemd-run` when that can work.
/// Why:
/// `dispatch` calls it before `sync` and `update`.
/// `Some(code)` means the run already happened in the slice,
/// and `None` means the caller does the work itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function relaunchIntoUpdatesSlice(env: EnvMap): number | undefined
/// ```
pub fn relaunch_into_updates_slice(env: &EnvMap) -> Option<i32> {
    let cgroup_text: String = std::fs::read_to_string("/proc/self/cgroup").unwrap_or_default();
    let user_manager_present: bool = match env.get("XDG_RUNTIME_DIR") {
        Some(directory) => std::path::Path::new(directory).join("systemd").join("private").exists(),
        None => false,
    };
    let facts = Facts {
        on_linux: cfg!(target_os = "linux"),
        env,
        cgroup_text: &cgroup_text,
        user_manager_present,
    };
    if stay_in_place_reason(&facts).is_some() {
        return None;
    }
    let Ok(executable) = std::env::current_exe() else {
        return None;
    };
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    return run_through_systemd("systemd-run", &executable.to_string_lossy(), &arguments, env);
}
