//! What:
//! Mechanism F,
//! the systemd user service that keeps `cargo-shadow lean run` running,
//! and the command that installs it.
//! Why:
//! Dormant worktrees,
//! busy-skipped directories and expired grace periods are retried without a cargo run
//! (`doc/planning/lean-build-dirs.md`,
//! option F).
//! The owner,
//! 2026-11-11:
//! "Make the cleanups a systemd service that runs continuously."
//! That replaced option F's once-a-day timer:
//! on 2026-11-11 the build volume held 6.5 MB free at 13:13,
//! hours before the daily pass,
//! and a manual pass hit its own deadline before it reached every profile directory.
//! The owner's earlier rule still bounds how it starts:
//! "Were you implying our wrapper or something in it auto starts itself on system startup or user login?
//! It should never do that! Follow Bazel."
//! then "Systemd units are fine."
//! So nothing in cargo-shadow starts itself:
//! the installer writes the unit,
//! `systemctl --user enable` links it into the user manager,
//! and only then does the manager start it.
//! Each pass ends on its own deadline,
//! the unit kills a pass that outlasts `RuntimeMaxSec`,
//! and the next pass starts `PAUSE_BETWEEN_PASSES_SECONDS` after the last one ended,
//! for as long as the user manager runs:
//! `Restart=always` with `StartLimitIntervalSec=0`,
//! so a pass that finds nothing never stops the loop.
//! The service shows nothing on the desktop (no bus address, no desktop message) and writes to the journal.
//! The once-a-day timer this installer wrote before is stopped,
//! disabled and removed, and no timer is written again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function renderService(repository: string, binary: string): string
//! ```

/// Files,
/// paths and child processes.
use std::{fs, path::{Path, PathBuf}, process::Command, time::Duration};

/// Imports the names this file uses from `crate::bounded_run`.
use crate::bounded_run::{Bound, RunError, output_within};
/// Imports the names this file uses from `crate::lean_policy`.
use crate::lean_policy::{PAUSE_BETWEEN_PASSES_SECONDS, RUN_DEADLINE_SECONDS};

/// The service unit's file name.
pub const SERVICE_NAME: &str = "cargo-shadow-lean.service";

/// The file name of the once-a-day timer this installer wrote before 2026-11-11.
/// It is no longer written:
/// `install_service` stops and disables it and removes its file.
pub const TIMER_NAME: &str = "cargo-shadow-lean.timer";

/// How long `systemctl` may take to answer.
const SYSTEMCTL_LIMIT: Duration = Duration::from_secs(60);

/// The time `RuntimeMaxSec` grants one pass above the run's own deadline:
/// thirty minutes,
/// for the one large tree whose deletion outlasts the run's own bound
/// (`doc/planning/lean-build-dirs.md`).
pub const RUNTIME_SLACK_SECONDS: u64 = 1_800;

/// What:
/// `pub fn render_service(repository: &Path, binary: &Path) -> String` is the service unit.
/// Why:
/// It runs the installed binary from the main checkout in a loop under limits sized for an I/O-bound job:
/// disk weight 5 (never above 10),
/// two threads of CPU,
/// 4 GiB of memory,
/// and a runtime bound above the run's own deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const renderService = (repository: string, binary: string): string
/// ```
pub fn render_service(repository: &Path, binary: &Path) -> String {
    return format!(
        "# Keeps the build folders lean continuously: every pass of `cargo-shadow lean run` ends on its own deadline,\n\
         # and the next one starts {pause} s later, for as long as the user manager runs.\n\
         # Installed and enabled by `cargo-shadow lean service-install`. It replaced the once-a-day timer of\n\
         # doc/planning/lean-build-dirs.md option F (owner, 2026-11-11: \"Make the cleanups a systemd service that runs continuously.\").\n\
         [Unit]\n\
         Description=Remove build folders, run records, toolchains and container images nothing needs, continuously (cargo-shadow lean run)\n\
         Documentation=file://{repository}/doc/planning/lean-build-dirs.md\n\
         # A pass that finds nothing ends quickly, and it must never stop the loop: the pause below is many times the\n\
         # measured cost of such a pass, so the loop cannot become a busy loop, and a restart is never given up on.\n\
         StartLimitIntervalSec=0\n\n\
         [Service]\n\
         Type=simple\n\
         WorkingDirectory={repository}\n\
         Environment=PATH=%h/.local/bin:%h/.cargo/bin:/usr/local/bin:/usr/bin:/bin\n\
         ExecStart={binary} lean run\n\
         # Nothing here may wait forever: each pass obeys the run's own deadline, and this kills one that outlasts it.\n\
         RuntimeMaxSec={runtime_max}\n\
         Restart=always\n\
         RestartSec={pause}s\n\
         CPUQuota=200%\n\
         MemoryMax=4G\n\
         TasksMax=512\n\
         IOWeight=5\n\
         Nice=15\n\
         IOSchedulingClass=idle\n\n\
         # The user manager starts it at login, and only because a person enabled it: nothing here starts itself.\n\
         [Install]\n\
         WantedBy=default.target\n",
        pause = PAUSE_BETWEEN_PASSES_SECONDS,
        repository = repository.display(),
        binary = binary.display(),
        runtime_max = RUN_DEADLINE_SECONDS + RUNTIME_SLACK_SECONDS,
    );
}

/// What:
/// `fn systemctl(arguments: &[&str]) -> Result<String, String>` runs `systemctl --user` with a deadline.
/// Why:
/// The user manager can be slow to answer under load,
/// and a wait must end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function systemctl(args: string[]): string  // throws
/// ```
fn systemctl(arguments: &[&str]) -> Result<String, String> {
    let bound: Bound = Bound { limit: SYSTEMCTL_LIMIT, what: format!("systemctl --user {}", arguments.join(" ")), rerun: String::from("cargo-shadow lean service-install") };
    return match output_within(Command::new("systemctl").arg("--user").args(arguments), None, &bound) {
        Ok(output) if output.status.success() => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
        Ok(output) => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        Err(RunError::TimedOut { what, limit, .. }) => Err(format!("{what} did not answer within {} s", limit.as_secs())),
        Err(other) => Err(format!("{other:?}")),
    };
}

/// What:
/// `fn absent_unit(reason: &str) -> bool` says whether `systemctl` refused because it does not know the unit.
/// Why:
/// The once-a-day timer was never enabled on most machines,
/// so `systemctl --user disable --now` answers that it does not exist,
/// which is exactly the state this install wants.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const absentUnit = (reason: string): boolean => ['does not exist', 'not loaded', 'not found', 'No such file'].some((needle) => reason.includes(needle))
/// ```
fn absent_unit(reason: &str) -> bool {
    return ["does not exist", "not loaded", "not found", "No such file"].iter().any(|needle| { return reason.contains(needle); });
}

/// What:
/// `fn stop_old_timer() -> Result<String, String>` stops and disables the once-a-day timer an older install enabled.
/// Why:
/// A running timer would start a second pass beside the continuous service,
/// and the link `enable` left in the manager's `timers.target.wants` would come back after the file is gone,
/// so the timer is stopped and disabled before its file is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopOldTimer(): string  // throws
/// ```
fn stop_old_timer() -> Result<String, String> {
    return match systemctl(&["disable", "--now", TIMER_NAME]) {
        Ok(_) => Ok(format!("stopped and disabled {TIMER_NAME}")),
        Err(reason) if absent_unit(&reason) => Ok(format!("{TIMER_NAME} was not enabled in the user manager, so there was nothing to stop")),
        Err(reason) => Err(format!("could not stop {TIMER_NAME}, so the daily pass would still run: {reason}")),
    };
}

/// What:
/// `pub fn install_service(repository: &Path, binary: &Path, unit_directory: &Path, enable: bool) -> Result<Vec<String>, String>`
/// writes the service,
/// stops and removes the once-a-day timer it once wrote,
/// and,
/// when asked,
/// reloads the user manager and enables the service.
/// Why:
/// It is the only place anything of this tool is made to start by itself,
/// and only because the user ran it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installService(repository: string, binary: string, unitDirectory: string, enable: boolean): string[]  // throws
/// ```
pub fn install_service(repository: &Path, binary: &Path, unit_directory: &Path, enable: bool) -> Result<Vec<String>, String> {
    if !binary.is_file() {
        return Err(format!("{} does not exist; run `mise run toolchain:install` first so the installed cargo-shadow is linked there", binary.display()));
    }
    if let Err(error) = fs::create_dir_all(unit_directory) {
        return Err(format!("{}: {error}", unit_directory.display()));
    }
    let service: PathBuf = unit_directory.join(SERVICE_NAME);
    let timer: PathBuf = unit_directory.join(TIMER_NAME);
    let mut lines: Vec<String> = Vec::new();
    // The daily pass must not outlive this install, so it is stopped before anything else,
    // and nothing is removed until the file that replaces it is written.
    if enable {
        match stop_old_timer() {
            Ok(line) => lines.push(line),
            Err(reason) => return Err(reason),
        }
    }
    if let Err(error) = fs::write(&service, render_service(repository, binary)) {
        return Err(format!("{}: {error}", service.display()));
    }
    lines.push(format!("wrote {}", service.display()));
    match fs::remove_file(&timer) {
        Ok(()) => lines.push(format!("removed {}, the once-a-day timer; the run is continuous now", timer.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("{}: {error}", timer.display())),
    }
    if enable {
        systemctl(&["daemon-reload"])?;
        systemctl(&["enable", "--now", SERVICE_NAME])?;
        lines.push(format!("enabled {SERVICE_NAME}: a pass runs now, each pass ends on its own deadline, and the next starts {PAUSE_BETWEEN_PASSES_SECONDS} s after it"));
        if let Ok(state) = systemctl(&["is-active", SERVICE_NAME]) {
            lines.push(format!("{SERVICE_NAME} is {}", state.trim()));
        }
    } else {
        lines.push(format!("{SERVICE_NAME} was written and not enabled (--no-enable); if {TIMER_NAME} is still enabled in the user manager, `systemctl --user disable --now {TIMER_NAME}` removes the link it left"));
    }
    return Ok(lines);
}
