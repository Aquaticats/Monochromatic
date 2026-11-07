//! What:
//!  What provisioning knows about the machine and the manager it asks:
//! the host's folders and programs,
//! the calls a manager takes,
//! the names of the earlier release's units,
//! and what a refused call is to the person.
//! Why:
//!  `service_provision.rs` writes the units and starts the socket,
//! and `service_provision_earlier_service_retirement.rs` retires the earlier service;
//! both use these and these use neither,
//! so they sit in a file of their own and `service_provision.rs` re-exports the public ones.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ProvisionHost = { runtimeDir: string; userUnitDir?: string; bus: string; serviceBinary: string; git: string; instance: string; retirementWait: number };
//! interface Manager { reload(): void; start(unit: string): void; stop(unit: string): void; activeState(unit: string): string | undefined }
//! ```

use crate::service_messages::{ClientFailure, Platform};
use crate::systemd_dbus::{Bus, CallError};
use std::path::{Path, PathBuf};

/// File name of the socket unit of the single service every worktree shared until one service per worktree.
pub const LEGACY_SOCKET_UNIT_NAME: &str = "cli-git.socket";

/// File name of the service unit of the single service every worktree shared until one service per worktree.
pub const LEGACY_SERVICE_UNIT_NAME: &str = "cli-git.service";

/// What provisioning needs to know about the machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionHost {
    /// The user's runtime directory,
    /// which holds the units and the service's socket.
    pub runtime_dir: PathBuf,
    /// The directory a user's own unit files live in (`~/.config/systemd/user`),
    /// where earlier releases wrote theirs.
    pub user_unit_dir: Option<PathBuf>,
    /// The session bus socket.
    pub bus: PathBuf,
    /// The program the service unit starts:
    /// the wrapper's own executable,
    /// which runs the service when the words after it ask
    ///  for that (`service_main.rs`).
    pub service_binary: PathBuf,
    /// The real Git the service runs.
    pub git: PathBuf,
    /// The instance of the worktree the commit is made in (`service_instance::instance_of`).
    pub instance: String,
    /// How long retiring the earlier single service may take before the commit fails instead of starting an instance beside it.
    pub retirement_wait: std::time::Duration,
}

/// How long the earlier single service may take to stop;
/// it finishes the batch it holds first.
/// The wait and every question to
///  the manager in it share this one budget,
/// set apart from the exchange's own,
/// so a slow stop is reported as that and not as
///  an unreachable manager.
pub const RETIREMENT_WAIT: std::time::Duration = std::time::Duration::from_secs(10);

/// What the manager is asked to do,
/// so a control can record it instead of reaching a manager.
pub trait Manager {
    /// Gives the calls that follow `wait` from now in all,
    /// so that each phase of provisioning has a time of its own.
    fn within(&mut self, wait: std::time::Duration);
    /// Reloads the unit files.
    fn reload(&mut self) -> Result<(), CallError>;
    /// Clears a unit's failed state and start-rate counter.
    fn reset_failed(&mut self, unit: &str) -> Result<(), CallError>;
    /// Starts a unit.
    fn start(&mut self, unit: &str) -> Result<(), CallError>;
    /// Stops a unit.
    fn stop(&mut self, unit: &str) -> Result<(), CallError>;
    /// Stops a unit and none of the units that require it.
    fn stop_alone(&mut self, unit: &str) -> Result<(), CallError>;
    /// The unit's state as the manager reports it (`active`,
    /// `deactivating`,
    /// `inactive`,
    /// `failed`),
    /// or nothing when the
    ///  manager holds no such unit.
    fn active_state(&mut self, unit: &str) -> Result<Option<String>, CallError>;
}

impl Manager for Bus {
    fn within(&mut self, wait: std::time::Duration) {
        Bus::within(self, wait);
    }

    fn reload(&mut self) -> Result<(), CallError> {
        return Bus::reload(self);
    }

    fn reset_failed(&mut self, unit: &str) -> Result<(), CallError> {
        return Bus::reset_failed(self, unit);
    }

    fn start(&mut self, unit: &str) -> Result<(), CallError> {
        return Bus::start_unit(self, unit);
    }

    fn stop(&mut self, unit: &str) -> Result<(), CallError> {
        return Bus::stop_unit(self, unit);
    }

    fn stop_alone(&mut self, unit: &str) -> Result<(), CallError> {
        return Bus::stop_unit_alone(self, unit);
    }

    fn active_state(&mut self, unit: &str) -> Result<Option<String>, CallError> {
        return Bus::active_state(self, unit);
    }
}

/// The platform whose remedy applies when no user service manager answers.
pub fn platform_without_manager() -> Platform {
    if cfg!(windows) {
        return Platform::Windows;
    }
    if cfg!(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "dragonfly")) {
        return Platform::Bsd;
    }
    if cfg!(target_os = "linux") {
        return Platform::LinuxWithoutUserServices;
    }
    return Platform::Other;
}

/// The directory the runtime units are written to.
pub fn runtime_unit_dir(runtime_dir: &Path) -> PathBuf {
    return runtime_dir.join("systemd").join("user");
}

/// What a refused call is to the person:
/// the manager's own words,
/// or a plain statement;
/// its error name is internal vocabulary.
pub(crate) fn refusal(error: CallError) -> ClientFailure {
    return match error {
        CallError::Manager { text, .. } if text.is_empty() => ClientFailure::CouldNotStart(String::from("the service manager refused the request")),
        CallError::Manager { text, .. } => ClientFailure::CouldNotStart(format!("the service manager refused the request ({text})")),
        CallError::Connection(_) | CallError::Authentication(_) | CallError::Malformed(_) | CallError::TimedOut => {
            ClientFailure::NoActivation(platform_without_manager())
        }
    };
}
