//! What:
//! Mechanism F,
//! the systemd user timer that runs `cargo-shadow lean run` once a day,
//! and the command that installs it.
//! Why:
//! Dormant worktrees,
//! busy-skipped directories and expired grace periods are retried without a cargo run
//! (`doc/planning/lean-build-dirs.md`,
//! option F).
//! The owner:
//! "Were you implying our wrapper or something in it auto starts itself on system startup or user login?
//! It should never do that!
//! Follow Bazel." then "Systemd units are fine."
//! So nothing in cargo-shadow starts itself;
//! this timer is a systemd unit the user installs on purpose.
//! It fires once a day at 04:30 local time and at no other moment:
//! `Persistent=false` (a missed run is not made up at boot or login),
//! the service has no `[Install]` section,
//! and the timer's `WantedBy=timers.target` only makes the timer itself available when the user's systemd starts,
//! never the service.
//! The service shows nothing on the desktop (no notification,
//! no bus address) and writes to the journal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function renderTimer(repository: string, binary: string): { service: string; timer: string }
//! ```

/// Files,
/// paths and child processes.
use std::{fs, path::{Path, PathBuf}, process::Command, time::Duration};

use crate::bounded_run::{Bound, RunError, output_within};
use crate::lean_policy::RUN_DEADLINE_SECONDS;

/// The service unit's file name.
pub const SERVICE_NAME: &str = "cargo-shadow-lean.service";

/// The timer unit's file name.
pub const TIMER_NAME: &str = "cargo-shadow-lean.timer";

/// When the timer fires,
/// in the user's local time zone:
/// every day at 04:30.
pub const SCHEDULE: &str = "*-*-* 04:30:00";

/// How long `systemctl` may take to answer.
const SYSTEMCTL_LIMIT: Duration = Duration::from_secs(60);

/// What:
/// `pub fn render_service(repository: &Path, binary: &Path) -> String` is the service unit.
/// Why:
/// It runs the installed binary from the main checkout under limits sized for an I/O-bound job:
/// disk weight 5 (never above 10),
/// two threads of CPU,
/// 4 GiB of memory,
/// and a start timeout above the run's own deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const renderService = (repository: string, binary: string): string
/// ```
pub fn render_service(repository: &Path, binary: &Path) -> String {
    return format!(
        "# Runs the lean-folder tools once. Started only by {TIMER_NAME}, never by a login or a boot:\n\
         # there is deliberately no install section. Rationale: doc/planning/lean-build-dirs.md, option F.\n\
         [Unit]\n\
         Description=Remove build folders, run records, toolchains and container images nothing needs (cargo-shadow lean run)\n\
         Documentation=file://{repository}/doc/planning/lean-build-dirs.md\n\n\
         [Service]\n\
         Type=oneshot\n\
         WorkingDirectory={repository}\n\
         Environment=PATH=%h/.local/bin:%h/.cargo/bin:/usr/local/bin:/usr/bin:/bin\n\
         ExecStart={binary} lean run\n\
         # Nothing here may wait forever: the run has its own deadline and this bounds the whole unit above it.\n\
         TimeoutStartSec={timeout}\n\
         CPUQuota=200%\n\
         MemoryMax=4G\n\
         TasksMax=512\n\
         IOWeight=5\n\
         Nice=15\n\
         IOSchedulingClass=idle\n",
        repository = repository.display(),
        binary = binary.display(),
        timeout = RUN_DEADLINE_SECONDS + 1_800,
    );
}

/// What:
/// `pub fn render_timer() -> String` is the timer unit.
/// Why:
/// It fires once a day at the time in `SCHEDULE` and at no other moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const renderTimer = (): string
/// ```
pub fn render_timer() -> String {
    return format!(
        "# Starts {SERVICE_NAME} once a day. `cargo-shadow lean timer-install` copies it to ~/.config/systemd/user and enables it.\n\
         [Unit]\n\
         Description=Daily lean-folder run (cargo-shadow lean run)\n\n\
         [Timer]\n\
         OnCalendar={SCHEDULE}\n\
         # A run missed because the machine was off or asleep is not made up at boot or at login.\n\
         Persistent=false\n\
         AccuracySec=1min\n\
         Unit={SERVICE_NAME}\n\n\
         [Install]\n\
         WantedBy=timers.target\n"
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
    let bound: Bound = Bound { limit: SYSTEMCTL_LIMIT, what: format!("systemctl --user {}", arguments.join(" ")), rerun: String::from("cargo-shadow lean timer-install") };
    return match output_within(Command::new("systemctl").arg("--user").args(arguments), None, &bound) {
        Ok(output) if output.status.success() => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
        Ok(output) => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        Err(RunError::TimedOut { what, limit, .. }) => Err(format!("{what} did not answer within {} s", limit.as_secs())),
        Err(other) => Err(format!("{other:?}")),
    };
}

/// What:
/// `pub fn install_timer(repository: &Path, binary: &Path, unit_directory: &Path, enable: bool) -> Result<Vec<String>, String>`
/// writes both units and,
/// when asked,
/// reloads the user manager and enables the timer.
/// Why:
/// It is the only place anything of this tool is made to start by itself,
/// and only because the user ran it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installTimer(repository: string, binary: string, unitDirectory: string, enable: boolean): string[]  // throws
/// ```
pub fn install_timer(repository: &Path, binary: &Path, unit_directory: &Path, enable: bool) -> Result<Vec<String>, String> {
    if !binary.is_file() {
        return Err(format!("{} does not exist; run `mise run toolchain:install` first so the installed cargo-shadow is linked there", binary.display()));
    }
    fs::create_dir_all(unit_directory).map_err(|error| { return format!("{}: {error}", unit_directory.display()); })?;
    let service: PathBuf = unit_directory.join(SERVICE_NAME);
    let timer: PathBuf = unit_directory.join(TIMER_NAME);
    fs::write(&service, render_service(repository, binary)).map_err(|error| { return format!("{}: {error}", service.display()); })?;
    fs::write(&timer, render_timer()).map_err(|error| { return format!("{}: {error}", timer.display()); })?;
    let mut lines: Vec<String> = vec![format!("wrote {} and {}", service.display(), timer.display())];
    if enable {
        systemctl(&["daemon-reload"])?;
        systemctl(&["enable", "--now", TIMER_NAME])?;
        lines.push(format!("enabled {TIMER_NAME}: it fires daily at 04:30 local time and at no other moment"));
        if let Ok(listing) = systemctl(&["list-timers", TIMER_NAME, "--no-pager"]) {
            lines.push(listing.trim_end().to_string());
        }
    } else {
        lines.push(format!("{TIMER_NAME} was not enabled (--no-enable)"));
    }
    return Ok(lines);
}
