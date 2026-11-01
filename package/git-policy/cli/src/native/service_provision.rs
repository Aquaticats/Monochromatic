//! What:
//!  The wrapper makes socket activation for the commit service exist the first time a commit needs it: it writes the two
//!  unit files into the user's runtime unit directory, asks the user manager (through the session bus) to reload and to
//!  start the socket, and returns. The manager, not the wrapper, starts the service when the first connection arrives.
//! Why:
//!  There is no install step to remember (`doc/planning/cli-git-resident-service.md`, "Provisioning"): like Bazel's
//!  client, the wrapper sets up what it needs and tells nobody. The units live under `$XDG_RUNTIME_DIR/systemd/user`, which
//!  disappears at logout, so nothing is left behind in the home directory and the next first use provisions again. Every
//!  step is idempotent (a file is written, by temporary file and rename, only when its content differs), so several commits
//!  racing to provision agree without any lock. Units this wrapper's earlier releases wrote into `~/.config/systemd/user`
//!  outrank the runtime ones in the unit search path, so they are recognized by their content and removed; a unit of the
//!  same name that something else wrote is never touched, and the commit stops with a message naming the file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await provision({ runtimeDir, configHome, busPath, serviceBinary, git }) // writes units, reloads, starts the socket
//! ```

use super::service_messages::{ClientFailure, Platform};
use super::systemd_dbus::{Bus, CallError, EXCHANGE_WAIT};
use std::path::{Path, PathBuf};

/// The socket unit's text, with the marker on its first line.
pub const SOCKET_UNIT: &str = include_str!("service_units/cli-git.socket");

/// The service unit's text, with `@SERVICE_BINARY@` and `@GIT@` still to fill in.
pub const SERVICE_UNIT: &str = include_str!("service_units/cli-git.service");

/// File name of the socket unit.
pub const SOCKET_UNIT_NAME: &str = "cli-git.socket";

/// File name of the service unit.
pub const SERVICE_UNIT_NAME: &str = "cli-git.service";

/// Name of the service program, next to the wrapper.
pub const SERVICE_PROGRAM: &str = "cli-git-service";

/// The words that mark a unit file as written by this wrapper: the marker its units carry now, and the descriptions the
///  units of earlier releases carried.
pub const OWN_MARKERS: [&str; 3] =
    ["(cli-git-unit)", "Description=Commit service socket for the git wrapper", "Description=Commit service for the git wrapper"];

const BINARY_PLACEHOLDER: &str = "@SERVICE_BINARY@";
const GIT_PLACEHOLDER: &str = "@GIT@";

/// What provisioning needs to know about the machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionHost {
    /// The user's runtime directory, which holds the units and the service's socket.
    pub runtime_dir: PathBuf,
    /// The directory a user's own unit files live in (`~/.config/systemd/user`), where earlier releases wrote theirs.
    pub user_unit_dir: Option<PathBuf>,
    /// The session bus socket.
    pub bus: PathBuf,
    /// The service program.
    pub service_binary: PathBuf,
    /// The real Git the service runs.
    pub git: PathBuf,
}

/// What the manager is asked to do, so a control can record it instead of reaching a manager.
pub trait Manager {
    /// Reloads the unit files.
    fn reload(&mut self) -> Result<(), CallError>;
    /// Starts a unit.
    fn start(&mut self, unit: &str) -> Result<(), CallError>;
}

impl Manager for Bus {
    fn reload(&mut self) -> Result<(), CallError> {
        return Bus::reload(self);
    }

    fn start(&mut self, unit: &str) -> Result<(), CallError> {
        return Bus::start_unit(self, unit);
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

/// The two unit files for a host, as written.
pub fn unit_texts(host: &ProvisionHost) -> [(&'static str, String); 2] {
    let service: String = SERVICE_UNIT
        .replace(BINARY_PLACEHOLDER, host.service_binary.to_string_lossy().as_ref())
        .replace(GIT_PLACEHOLDER, host.git.to_string_lossy().as_ref());
    return [(SOCKET_UNIT_NAME, SOCKET_UNIT.to_string()), (SERVICE_UNIT_NAME, service)];
}

/// The directory the runtime units are written to.
pub fn runtime_unit_dir(runtime_dir: &Path) -> PathBuf {
    return runtime_dir.join("systemd").join("user");
}

static TEMPORARY_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Writes `text` to `path` through a temporary file and a rename, so a reader sees the old file or the new one whole.
fn write_atomically(path: &Path, text: &str) -> std::io::Result<()> {
    let number: u64 = TEMPORARY_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temporary: PathBuf = path.with_extension(format!("tmp-{}-{number}", std::process::id()));
    std::fs::write(&temporary, text.as_bytes())?;
    let renamed: std::io::Result<()> = std::fs::rename(&temporary, path);
    if renamed.is_err() {
        let _removed = std::fs::remove_file(&temporary);
    }
    return renamed;
}

/// What the user's own unit directory holds under the names the wrapper uses.
fn clear_earlier_units(host: &ProvisionHost) -> Result<bool, ClientFailure> {
    let Some(directory) = host.user_unit_dir.as_deref() else { return Ok(false) };
    let mut removed: bool = false;
    for name in [SOCKET_UNIT_NAME, SERVICE_UNIT_NAME] {
        let path: PathBuf = directory.join(name);
        let Ok(text) = std::fs::read_to_string(&path) else {
            if path.exists() {
                return Err(ClientFailure::UnitInTheWay(path.display().to_string()));
            }
            continue;
        };
        if !OWN_MARKERS.iter().any(|marker| { return text.contains(marker); }) {
            return Err(ClientFailure::UnitInTheWay(path.display().to_string()));
        }
        std::fs::remove_file(&path).map_err(|error| { return ClientFailure::CouldNotStart(format!("{} cannot be removed: {error}", path.display())); })?;
        removed = true;
    }
    // The enabling link an earlier release made points at the file just removed.
    let link: PathBuf = directory.join("sockets.target.wants").join(SOCKET_UNIT_NAME);
    if std::fs::symlink_metadata(&link).is_ok_and(|meta| { return meta.file_type().is_symlink(); }) {
        let _removed = std::fs::remove_file(&link);
        removed = true;
    }
    return Ok(removed);
}

/// What:
///  Makes socket activation exist: earlier units removed, current units written where they differ, the manager reloaded when
///  anything changed, and the socket started.
/// Why:
///   Idempotent by construction: a second call with nothing to change only starts the socket, which is a no-op when it is
///  already listening; two calls at once write identical content through renames and both succeed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await provisionWith(host, manager)
/// ```
pub fn provision_with(host: &ProvisionHost, manager: &mut dyn Manager) -> Result<(), ClientFailure> {
    let mut changed: bool = clear_earlier_units(host)?;
    let directory: PathBuf = runtime_unit_dir(host.runtime_dir.as_path());
    std::fs::create_dir_all(&directory).map_err(|error| { return ClientFailure::CouldNotStart(format!("{} cannot be created: {error}", directory.display())); })?;
    for (name, text) in unit_texts(host) {
        let path: PathBuf = directory.join(name);
        if std::fs::read_to_string(&path).is_ok_and(|current| { return current == text; }) {
            continue;
        }
        write_atomically(&path, text.as_str())
            .map_err(|error| { return ClientFailure::CouldNotStart(format!("{} cannot be written: {error}", path.display())); })?;
        changed = true;
    }
    let refused = |error: CallError| -> ClientFailure {
        return match error {
            // The error's name is the manager's internal vocabulary; the person gets its words, or a plain statement.
            CallError::Manager { text, .. } if text.is_empty() => ClientFailure::CouldNotStart(String::from("the service manager refused the request")),
            CallError::Manager { text, .. } => ClientFailure::CouldNotStart(format!("the service manager refused the request ({text})")),
            CallError::Connection(_) | CallError::Authentication(_) | CallError::Malformed(_) => ClientFailure::NoActivation(platform_without_manager()),
        };
    };
    if changed {
        manager.reload().map_err(refused)?;
    }
    return manager.start(SOCKET_UNIT_NAME).map_err(refused);
}

/// Provisions through the session bus.
pub fn provision(host: &ProvisionHost) -> Result<(), ClientFailure> {
    let mut bus: Bus = Bus::connect(host.bus.as_path(), EXCHANGE_WAIT).map_err(|_| { return ClientFailure::NoActivation(platform_without_manager()); })?;
    return provision_with(host, &mut bus);
}

/// Provisioning controls stay out of the release executable.
#[cfg(test)]
#[path = "service_provision_tests.rs"]
mod tests;
