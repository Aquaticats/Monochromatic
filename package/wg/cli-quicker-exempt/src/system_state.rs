//! Whether the whole system is stopping.
//! A sentinel runs as a service of the system manager and outlives the user session of the watcher it waits for.
//! At a shutdown or a reboot the watcher is ended with everything else:
//! the state in `/run` and the tunnel go with it,
//! so nothing is left unenforced and the death is not a policy failure.
//! A logout while the system keeps running is different:
//! the tunnel stays up with no watcher,
//! so the update traffic of later sessions is no longer exempted,
//! and that death is reported.
//! The system counts as stopping when `shutdown.target` is active or becoming active,
//! or when `/run/nologin` exists,
//! which `systemd-user-sessions stop` creates early in a shutdown and which `systemd-logind` creates
//! shortly before a scheduled one.
//! A probe that cannot tell is treated as a system that keeps running,
//! so a failing probe never hides a death.

/// Non-panicking log.
use crate::log_line::log_line;
/// Commands with a time limit and a fixed search path.
use crate::bounded_command::run_bounded;
/// Paths of the marker files.
use std::path::Path;
/// The time limit.
use std::time::Duration;

/// The file that tells that no new logins are allowed,
/// which exists while a system stops.
pub const NOLOGIN_PATH: &str = "/run/nologin";

/// Arguments of `systemctl` that print the state of `shutdown.target` alone and exit with zero for any state.
const SHOW_ARGUMENTS: [&str; 4] = ["show", "--property=ActiveState", "--value", "shutdown.target"];

/// Time `systemctl` may take before it is ended.
const PROBE_LIMIT: Duration = Duration::from_secs(5);

/// Whether the system is stopping,
/// replaceable in tests.
pub trait SystemState {
    /// Whether the whole system is shutting down or rebooting.
    fn is_stopping(&self) -> bool;
}

/// Whether the state `systemctl` printed for `shutdown.target` means a shutdown is under way.
pub fn shutdown_target_means_stopping(printed: &str) -> bool {
    return matches!(printed.trim(), "active" | "activating");
}

/// The real host.
pub struct HostSystemState<'a> {
    /// The file whose presence means no new logins are allowed.
    pub nologin: &'a Path,
}

/// Reading the real host.
impl SystemState for HostSystemState<'_> {
    /// True when the no-login file exists or `shutdown.target` is active or activating.
    fn is_stopping(&self) -> bool {
        if self.nologin.exists() {
            return true;
        }
        let arguments: Vec<String> = SHOW_ARGUMENTS.iter().map(ToString::to_string).collect();
        return match run_bounded("systemctl", &arguments, PROBE_LIMIT) {
            Ok(output) if output.success => shutdown_target_means_stopping(&output.stdout),
            Ok(output) => {
                log_line(&format!("systemctl could not tell whether the system is stopping ({}: {}), so it is judged to keep running", output.status, output.stderr.trim()));
                false
            }
            Err(error) => {
                log_line(&format!("systemctl could not tell whether the system is stopping ({error}), so it is judged to keep running"));
                false
            }
        };
    }
}
