//! What:
//!  `git cli-git service install`, `uninstall` and `status`: put the commit service's activation in place for the
//!  current account, take it out again, or say whether it is there.
//! Why:
//!  The service is never started by the wrapper: the operating system starts it when the first commit connects to its
//!  socket. Installing means writing two unit files for the account's service manager, naming the service program and
//!  the real Git, and enabling the socket; nothing is started but the socket. The steps are idempotent, and every
//!  failure the person can act on is said in plain words (`service_messages.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = await installService({ unitDirectory, systemctl, serviceBinary, git });
//! ```

use super::service_messages::{ClientFailure, Platform, render};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// The socket unit, with the text the service manager reads.
pub const SOCKET_UNIT: &str = include_str!("service_units/cli-git.socket");

/// The service unit, with `@SERVICE_BINARY@` and `@GIT@` still to fill in.
pub const SERVICE_UNIT: &str = include_str!("service_units/cli-git.service");

/// File name of the socket unit.
pub const SOCKET_UNIT_NAME: &str = "cli-git.socket";

/// File name of the service unit.
pub const SERVICE_UNIT_NAME: &str = "cli-git.service";

/// Name of the service program next to the wrapper.
pub const SERVICE_PROGRAM: &str = "cli-git-service";

/// Placeholder for the service program's absolute path in the service unit.
const BINARY_PLACEHOLDER: &str = "@SERVICE_BINARY@";

/// Placeholder for the real Git's absolute path in the service unit.
const GIT_PLACEHOLDER: &str = "@GIT@";

/// What the install steps need from the machine.
#[derive(Clone, Debug)]
pub struct ServiceHost {
    /// Where the account's service manager reads user units.
    pub unit_directory: PathBuf,
    /// The service manager's control program.
    pub systemctl: PathBuf,
    /// The service program.
    pub service_binary: PathBuf,
    /// The real Git the service runs.
    pub git: PathBuf,
}

/// What a step came to: the text for standard output and the exit status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    /// Exit status of the command.
    pub code: i32,
    /// What went to standard output.
    pub stdout: String,
    /// What went to standard error.
    pub stderr: String,
}

fn failed(failure: &ClientFailure) -> Report {
    return Report { code: 1, stdout: String::new(), stderr: format!("{}\n", render(failure)) };
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
pub fn unit_texts(host: &ServiceHost) -> [(&'static str, String); 2] {
    let service: String = SERVICE_UNIT
        .replace(BINARY_PLACEHOLDER, host.service_binary.to_string_lossy().as_ref())
        .replace(GIT_PLACEHOLDER, host.git.to_string_lossy().as_ref());
    return [(SOCKET_UNIT_NAME, SOCKET_UNIT.to_string()), (SERVICE_UNIT_NAME, service)];
}

fn systemctl(host: &ServiceHost, arguments: &[&str]) -> std::io::Result<Output> {
    return Command::new(&host.systemctl).arg("--user").args(arguments).stdin(Stdio::null()).output();
}

/// Whether the account has a service manager that answers.
fn manager_answers(host: &ServiceHost) -> bool {
    return systemctl(host, &["show-environment"]).is_ok_and(|output| { return output.status.success(); });
}

fn command_text(output: &Output) -> String {
    let text: String = String::from_utf8_lossy(output.stderr.as_slice()).trim().replace('\n', " ");
    return if text.is_empty() { String::from("it gave no reason") } else { text };
}

/// What:
///  Writes the unit files that differ, enables the socket, and says what it did.
/// Why:
///   A second run changes nothing and says so; the socket is enabled and started, the service is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await install(host)
/// ```
pub fn install(host: &ServiceHost) -> Report {
    if !host.service_binary.is_file() {
        return failed(&ClientFailure::InstallProgramMissing(host.service_binary.display().to_string()));
    }
    if !manager_answers(host) {
        return failed(&ClientFailure::NoActivation(platform_without_manager()));
    }
    if let Err(error) = std::fs::create_dir_all(&host.unit_directory) {
        return failed(&ClientFailure::InstallCommandFailed(format!("{} cannot be created: {error}", host.unit_directory.display())));
    }
    let mut written: Vec<String> = Vec::new();
    for (name, text) in unit_texts(host) {
        let path: PathBuf = host.unit_directory.join(name);
        if std::fs::read_to_string(&path).is_ok_and(|current| { return current == text; }) {
            continue;
        }
        let temporary: PathBuf = host.unit_directory.join(format!("{name}.new"));
        if let Err(error) = std::fs::write(&temporary, text.as_bytes()).and_then(|()| { return std::fs::rename(&temporary, &path); }) {
            return failed(&ClientFailure::InstallCommandFailed(format!("{} cannot be written: {error}", path.display())));
        }
        written.push(path.display().to_string());
    }
    for arguments in [&["daemon-reload"][..], &["enable", "--now", SOCKET_UNIT_NAME][..]] {
        match systemctl(host, arguments) {
            Ok(output) if output.status.success() => {}
            Ok(output) => return failed(&ClientFailure::InstallCommandFailed(command_text(&output))),
            Err(error) => return failed(&ClientFailure::InstallCommandFailed(error.to_string())),
        }
    }
    let mut lines: Vec<String> = Vec::new();
    if written.is_empty() {
        lines.push(String::from("The commit service was already installed for this account; nothing changed."));
    } else {
        lines.push(String::from("Installed the commit service for this account."));
        for path in written {
            lines.push(format!("  Wrote `{path}`."));
        }
    }
    lines.push(format!("  Enabled `{SOCKET_UNIT_NAME}`; the service itself starts when the first commit asks for it."));
    lines.push(String::from("  Plain scoped commits go through it from now on; `CLI_GIT_COMMIT_SERVICE=0` keeps one commit out of it."));
    return Report { code: 0, stdout: format!("{}\n", lines.join("\n")), stderr: String::new() };
}

/// What:
///  Stops and removes the activation, and says what it did.
/// Why:
///   Units that were never there are not an error; the journal of landed commits is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await uninstall(host)
/// ```
pub fn uninstall(host: &ServiceHost) -> Report {
    if !manager_answers(host) {
        return failed(&ClientFailure::NoActivation(platform_without_manager()));
    }
    // A unit the manager does not know makes `disable` fail, which is the state being asked for.
    let _disabled = systemctl(host, &["disable", "--now", SOCKET_UNIT_NAME, SERVICE_UNIT_NAME]);
    let mut removed: Vec<String> = Vec::new();
    for name in [SOCKET_UNIT_NAME, SERVICE_UNIT_NAME] {
        let path: PathBuf = host.unit_directory.join(name);
        if path.exists() {
            if let Err(error) = std::fs::remove_file(&path) {
                return failed(&ClientFailure::InstallCommandFailed(format!("{} cannot be removed: {error}", path.display())));
            }
            removed.push(path.display().to_string());
        }
    }
    let _reloaded = systemctl(host, &["daemon-reload"]);
    let text: String = if removed.is_empty() {
        String::from("The commit service was not installed for this account; nothing changed.\n")
    } else {
        format!("Removed the commit service for this account.\n{}\n", removed.iter().map(|path| { return format!("  Removed `{path}`."); }).collect::<Vec<String>>().join("\n"))
    };
    return Report { code: 0, stdout: text, stderr: String::new() };
}

/// What:
///  Says whether the activation is installed and listening.
/// Why:
///   Exit status 0 when the socket is listening, 1 when it is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await status(host)
/// ```
pub fn status(host: &ServiceHost) -> Report {
    if !manager_answers(host) {
        return failed(&ClientFailure::NoActivation(platform_without_manager()));
    }
    let listening: bool = systemctl(host, &["is-active", SOCKET_UNIT_NAME]).is_ok_and(|output| { return output.status.success(); });
    return if listening {
        Report { code: 0, stdout: String::from("The commit service is installed and ready for commits.\n"), stderr: String::new() }
    } else {
        Report { code: 1, stdout: String::new(), stderr: format!("{}\n", render(&ClientFailure::NotInstalled)) }
    };
}

/// The directory the account's service manager reads user units from.
pub fn unit_directory(config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    return config_home
        .map(|directory| { return directory.join("systemd").join("user"); })
        .or_else(|| { return home.map(|directory| { return directory.join(".config").join("systemd").join("user"); }); });
}

/// Install controls stay out of the release executable.
#[cfg(test)]
#[path = "service_install_tests.rs"]
mod tests;
