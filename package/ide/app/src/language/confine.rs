//! The production launch policy:
//!  every language server runs inside bubblewrap,
//!  so neither it
//! nor anything it starts can write project files.
//!  If confinement cannot be set up,
//!  the server
//! is refused;
//!  there is no fallback to an unconfined launch.

/// The seam's input and output types,
///  and the state-path checks shared with `prepare`.
use super::launch::{LaunchRequest, ServerLaunch, check_state_directory, resolve_existing};
/// The shared private cache directory rule and the fixed FNV-1a digest that names per-project state.
use crate::{app_cache::application_cache, content_digest::fnv1a};
/// What:
///  `Command` starts a child process and waits for it;
///  `OnceLock` holds a value computed once.
/// Why:
///  The namespace probe runs bubblewrap once per process and remembers the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawnSync } from 'node:child_process';
/// ```
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

/// How the project is mounted inside the sandbox.
pub mod project;
/// The bubblewrap command line,
///  without system access.
pub mod recipe;
/// The settings overrides a confined server gets.
mod settings;

/// The only bubblewrap the policy uses:
///  the host's packaged copy,
///  never one found on `PATH`.
pub const BUBBLEWRAP: &str = "/usr/bin/bwrap";

/// A program that exists on the host and does nothing,
///  used to test sandbox creation.
const PROBE_PROGRAM: &str = "/usr/bin/true";

/// Result of the sandbox probe with a process-id namespace,
///  computed once per process.
static PROBE_WITH_PID: OnceLock<Result<(), String>> = OnceLock::new();

/// Result of the sandbox probe without a process-id namespace,
///  computed once per process.
static PROBE_WITHOUT_PID: OnceLock<Result<(), String>> = OnceLock::new();

/// What:
///  Turn text into a safe single path component.
///  `char` is one Unicode scalar value.
/// Why:
///  Project and server names become directory names below private state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const component = (text: string) => text.replace(/[^A-Za-z0-9._-]/g, '_').replace(/^\.+$/, '_') || '_';
/// ```
fn component(text: &str) -> String {
    let mut safe = String::new();
    for character in text.chars() {
        if character.is_ascii_alphanumeric()
            || character == '.'
            || character == '_'
            || character == '-'
        {
            safe.push(character);
        } else {
            safe.push('_');
        }
    }
    let mut only_dots = true;
    for character in safe.chars() {
        if character != '.' {
            only_dots = false;
            break;
        }
    }
    if safe.is_empty() || only_dots {
        return "_".to_string();
    }
    return safe;
}

/// What:
///  The private state directory of one server for one project:
///       `<state root>/<project name>-<hash of the project path>/<server>`.
/// Why:
///  The adopted shape keeps state per project and per server,
///  in the private cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stateDirectory(stateRoot: string, project: string, server: string): string
/// ```
pub fn state_directory(state_root: &Path, project_root: &Path, server: &str) -> PathBuf {
    // `file_name` is the last path component; `to_string_lossy` renders it as text.
    let name = match project_root.file_name() {
        Some(found) => found.to_string_lossy().into_owned(),
        None => "root".to_string(),
    };
    let shortened: String = component(&name).chars().take(40).collect();
    let hash = fnv1a(project_root.as_os_str().as_encoded_bytes());
    return state_root
        .join(format!("{shortened}-{hash:016x}"))
        .join(component(server));
}

/// What:
///  The application's private state root:
///  `$XDG_CACHE_HOME/monochromatic-ide/language`,
///       or `$HOME/.cache/monochromatic-ide/language`;
///  nothing when neither is an absolute path.
/// Why:
///  A private application cache is allowed by the accepted scope;
///  it lies inside the project
///      only when the home folder (or a folder above it) is opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function defaultStateRoot(): string | undefined
/// ```
pub fn default_state_root() -> Option<PathBuf> {
    // `application_cache()` is the shared `$XDG_CACHE_HOME/monochromatic-ide` rule; the trailing `?`
    // keeps `None` as it is, and `language` is joined onto a present value.
    let cache = application_cache()?;
    return Some(cache.join("language"));
}

/// What:
///  Refuse a path below `/proc`.
///  `what` names the path and `remedy` the fix in the message.
/// Why:
///  No directory a user creates can lie there,
///  and the sandbox mounts a fresh process file
///      system;
///  every other location works,
///  because the project and private state are bound
///      again after the replacements.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkOutsideProc(path: string, what: string, remedy: string): void // throws to refuse
/// ```
fn check_outside_proc(path: &Path, what: &str, remedy: &str) -> Result<(), String> {
    if path.starts_with(project::PROCESS_FILE_SYSTEM) {
        return Err(format!(
            "{what} {} is below /proc, where no directory a user creates can exist and the language-server sandbox mounts a fresh process file system. {remedy}",
            path.display()
        ));
    }
    return Ok(());
}

/// What:
///  The private state directory of the requested server,
///  below the resolved state root,
///       or the refusal reason.
/// Why:
///  The state root is resolved first,
///  so a variable that reaches `/tmp` or the project
///      through a symbolic link is compared and bound by its real location.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function privateState(request: LaunchRequest): string // throws to refuse
/// ```
pub fn private_state(request: &LaunchRequest) -> Result<PathBuf, String> {
    let server = &request.server;
    let Some(state_root) = request.state_root.as_ref() else {
        return Err(format!(
            "no private state directory is available for {server} (neither XDG_CACHE_HOME nor HOME is an absolute path), so it is not started. Set HOME or XDG_CACHE_HOME and restart the application"
        ));
    };
    let resolved = resolve_existing(state_root);
    let remedy = format!(
        "{server} is not started. Point XDG_CACHE_HOME or HOME at a directory that does not contain the project and is outside /proc, then restart the application"
    );
    check_outside_proc(&resolved, "the private state directory", &remedy)?;
    if let Err(reason) = check_state_directory(&resolved, &request.project_root) {
        return Err(format!("{reason}, so {remedy}"));
    }
    return Ok(state_directory(&resolved, &request.project_root, server));
}

/// What:
///  Start bubblewrap once with the server's namespaces around a program that does nothing.
/// Why:
///  A host without user namespaces makes bubblewrap fail;
///  the probe turns that into a
///      refusal with the cause,
///  before any server process exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function probe(bubblewrap: string, pidNamespace: boolean): void // throws with the cause
/// ```
fn probe(bubblewrap: &str, pid_namespace: bool) -> Result<(), String> {
    let mut arguments = recipe::isolation(pid_namespace);
    arguments.push("--".to_string());
    arguments.push(PROBE_PROGRAM.to_string());
    let ran = Command::new(bubblewrap)
        .args(&arguments)
        .env_clear()
        .output();
    let output = match ran {
        Ok(finished) => finished,
        Err(error) => return Err(format!("cannot run {bubblewrap}: {error}")),
    };
    if output.status.success() {
        return Ok(());
    }
    // `from_utf8_lossy` renders bubblewrap's message even if it is not valid UTF-8.
    let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
    return Err(format!(
        "{bubblewrap} could not create the language-server sandbox ({}: {message}). Language servers are not started without write confinement. Allow unprivileged user namespaces for this application (sysctl user.max_user_namespaces above 0, and no security policy denying them), then restart the application",
        output.status
    ));
}

/// What:
///  The confined launch with an explicit bubblewrap path.
/// Why:
///  Tests pass a missing path to prove that an absent bubblewrap refuses the server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function confineWith(request: LaunchRequest, bubblewrap: string): ServerLaunch // throws to refuse
/// ```
pub fn confine_with(request: &LaunchRequest, bubblewrap: &str) -> Result<ServerLaunch, String> {
    let reopen = format!(
        "{} is not started. Open the project from a directory outside /proc",
        request.server
    );
    check_outside_proc(&request.project_root, "the project", &reopen)?;
    for spelling in &request.project_spellings {
        check_outside_proc(spelling, "the project spelling", &reopen)?;
    }
    let executable = match std::fs::metadata(bubblewrap) {
        Ok(found) => found.is_file() && found.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    };
    if !executable {
        return Err(format!(
            "{bubblewrap} is not installed, so {} cannot be confined and is not started. Install bubblewrap (the bubblewrap package) and restart the application",
            request.server
        ));
    }
    let state = private_state(request)?;
    let pid_namespace = !recipe::NO_PID_NAMESPACE.contains(&request.server.as_str());
    let cell = if pid_namespace {
        &PROBE_WITH_PID
    } else {
        &PROBE_WITHOUT_PID
    };
    // The probe runs on first use; the stored answer is read afterwards.
    if cell.get().is_none() {
        let _stored = cell.set(probe(bubblewrap, pid_namespace));
    }
    let Some(probed) = cell.get() else {
        return Err("the sandbox probe did not store its answer".to_string());
    };
    probed.clone()?;
    let mut inherited: Vec<(String, String)> = Vec::new();
    for name in recipe::ALLOWED_ENVIRONMENT {
        // `var` fails for unset and non-Unicode values; both are left out.
        if let Ok(value) = std::env::var(name) {
            inherited.push((name.to_string(), value));
        }
    }
    return recipe::recipe(request, &state, bubblewrap, &inherited);
}

/// What:
///  The production launch policy.
///  Its signature is `LaunchPolicy`.
/// Why:
///  One function decides how every server starts;
///  it refuses rather than run unconfined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const launchConfined: LaunchPolicy = request => confineWith(request, '/usr/bin/bwrap');
/// ```
pub fn launch_confined(request: &LaunchRequest) -> Result<ServerLaunch, String> {
    return confine_with(request, BUBBLEWRAP);
}

/// Command-line shapes,
///  settings overrides,
///  state naming,
///  and refusals,
///  without starting bubblewrap.
#[cfg(test)]
#[path = "confine_tests.rs"]
mod tests;
