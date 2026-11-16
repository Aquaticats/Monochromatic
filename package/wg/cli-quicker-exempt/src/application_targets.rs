//! Discovers the cgroups a socket mark belongs on:
//! the units a desktop application's name claims while the application is in them,
//! the cgroups of the applications' own programs,
//! the Pi agent units and the updates slice.
//! What each application is called and which programs belong to it lives in `desktop_applications`.

/// The applications a name claims and the programs that prove one is there,
/// which the scan reads and the rest of the program reaches through this module.
pub use crate::desktop_applications::{DesktopApplication, desktop_application_of_name, is_application_executable};
/// The programs of the applications that carry no cgroup name of their own.
use crate::desktop_applications::is_exempt_application_executable;
/// Filesystem and process-race failures.
use std::io;
/// Target and fixture roots.
use std::path::{Path, PathBuf};

/// Prefix of the transient user units the repository's Pi agents run in,
/// one unit per agent run,
/// named `pi-<name>.service` after the run.
const PI_AGENT_SERVICE_PREFIX: &str = "pi-";

/// Suffix of every systemd service unit.
const SERVICE_SUFFIX: &str = ".service";

/// Slice that carries the toolchain's update runs.
/// The `app-` prefix makes systemd nest it directly under `app.slice`,
///  where the watcher scans and watches.
/// A name such as `monochromatic-updates.slice` would sit beside `app.slice` instead,
///  and the watcher would never see it.
/// `\x2d` is the escaped literal dash,
///  so the name is one hierarchy level and not `app-monochromatic.slice`.
/// The watcher itself creates the slice and moves update entry points into it;
///  the toolchain knows nothing about it.
pub const TOOLCHAIN_UPDATES_SLICE: &str = "app-monochromatic\\x2dupdates.slice";

/// Roots make process and cgroup discovery testable without real host state.
pub struct ScanRoots<'a> {
    /// User's systemd `app.slice` directory.
    pub app_slice: &'a Path,
    /// Procfs root containing numeric process directories.
    pub proc_root: &'a Path,
    /// Cgroup v2 mount root joined with unified paths.
    pub cgroup_root: &'a Path,
}

/// Reports the slice the toolchain's update runs execute in.
/// Cgroup programs apply to every descendant cgroup,
///  so attaching to the slice marks each run scope created below it,
///  with no attach per run.
pub fn is_toolchain_updates_slice_name(name: &str) -> bool {
    return name == TOOLCHAIN_UPDATES_SLICE;
}

/// The cgroup path of the updates slice of `uid`,
/// as `/proc/<pid>/cgroup` spells it.
pub fn updates_slice_cgroup(uid: u32) -> String {
    return format!("/user.slice/user-{uid}.slice/user@{uid}.service/app.slice/{TOOLCHAIN_UPDATES_SLICE}");
}

/// Reports whether `cgroup` is the updates slice itself or lies below it.
/// Whole path components are compared:
/// a unit the user named after the slice is somewhere else,
/// and a process in it is moved like any other.
pub fn is_in_updates_slice(cgroup: &str, uid: u32) -> bool {
    let slice = updates_slice_cgroup(uid);
    return cgroup == slice || cgroup.starts_with(&format!("{slice}/"));
}

/// Reports a `pi-<name>.service` agent unit,
/// the unit a Pi agent run of this repository lives in.
/// The name must carry something after the prefix:
/// `pi-.service` names no run.
pub fn is_pi_service_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix(PI_AGENT_SERVICE_PREFIX) else {
        return false;
    };
    let Some(agent) = rest.strip_suffix(SERVICE_SUFFIX) else {
        return false;
    };
    return !agent.is_empty();
}

/// Reports whether a cgroup name is marked without any evidence of its application:
/// a Pi agent's unit and the updates slice,
/// the two names the owner accepted as names alone.
/// Every desktop application's name needs the application's own program in the cgroup it names.
pub fn is_unconditionally_marked_cgroup(cgroup: &str, uid: u32) -> bool {
    let prefix = format!("/user.slice/user-{uid}.slice/user@{uid}.service/app.slice/");
    let Some(leaf) = cgroup.strip_prefix(&prefix) else {
        return false;
    };
    if leaf.is_empty() || leaf.contains('/') {
        return false;
    }
    return is_pi_service_name(leaf) || is_toolchain_updates_slice_name(leaf);
}

/// Reports numeric procfs directory name without regular expression parsing.
fn is_process_id(name: &str) -> bool {
    return !name.is_empty() && name.bytes().all(is_ascii_digit_byte);
}

/// Reports one ASCII decimal digit byte.
fn is_ascii_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// Extracts unified cgroup path from one procfs cgroup file.
fn unified_cgroup_path(text: &str) -> Option<&str> {
    for line in text.lines() {
        if let Some(path) = line.strip_prefix("0::") {
            return Some(path);
        }
    }
    return None;
}

/// Adds path once while preserving simple linear collection semantics.
fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}

/// Whether an error means the process is gone,
/// which the shared reader of process facts also decides.
use crate::process_info::means_vanished;

/// One process of the watched user:
/// the cgroup it runs in and the program it runs.
struct ApplicationProcess {
    /// Cgroup directory of the process,
    /// under the app slice the scan reads.
    cgroup: PathBuf,
    /// Resolved executable path of the process.
    executable: PathBuf,
}

/// Reads every process that runs in the watched user's app slice.
///
/// One pass serves both halves of the scan:
/// a program that owns a cgroup,
/// and the program a cgroup name has to hold before the name is believed.
fn read_application_processes(roots: &ScanRoots<'_>) -> io::Result<Vec<ApplicationProcess>> {
    let mut processes = Vec::new();
    for entry_result in std::fs::read_dir(roots.proc_root)? {
        let entry = entry_result?;
        let name = entry.file_name();
        let Some(name_text) = name.to_str() else {
            continue;
        };
        if !is_process_id(name_text) {
            continue;
        }
        let process = entry.path();
        let Some(executable) = read_process_executable(&process.join("exe"))? else {
            continue;
        };
        let Some(cgroup_text) = read_process_file(&process.join("cgroup"))? else {
            continue;
        };
        let Some(relative) = unified_cgroup_path(&cgroup_text) else {
            continue;
        };
        let target = roots.cgroup_root.join(relative.trim_start_matches('/'));
        if target.starts_with(roots.app_slice) && target.is_dir() {
            processes.push(ApplicationProcess { cgroup: target, executable });
        }
    }
    return Ok(processes);
}

/// Reports whether a cgroup,
/// or a cgroup below it,
/// holds a process running `application`'s program.
fn holds_application(application: DesktopApplication, directory: &Path, processes: &[ApplicationProcess]) -> bool {
    return processes.iter().any(|process| {
        return process.cgroup.starts_with(directory) && is_application_executable(application, &process.executable);
    });
}

/// Scans direct app slice entries identifying Ghostty,
///  Steam,
///  Helium,
///  Firefox release,
///  Firefox Nightly,
/// ChatGPT,
///  Interpreter,
///  Qure,
///  OdyTTY
///  or Pi services,
///  and the toolchain updates slice.
///
/// A desktop application's name is believed only while the cgroup it names holds that application's own program:
/// a unit a user named `app-firefox@anything.service` and filled with another program is not marked,
/// where the name alone used to be enough.
/// Pi units and the updates slice are marked by name,
/// which the owner accepted because a unit name is a name the user can choose.
fn scan_named_cgroups(
    roots: &ScanRoots<'_>,
    processes: &[ApplicationProcess],
    targets: &mut Vec<PathBuf>,
    without_application: &mut Vec<PathBuf>,
) -> io::Result<()> {
    for entry_result in std::fs::read_dir(roots.app_slice)? {
        let entry = entry_result?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let Some(name_text) = name.to_str() else {
            continue;
        };
        let directory = entry.path();
        if is_pi_service_name(name_text) || is_toolchain_updates_slice_name(name_text) {
            push_unique(targets, directory);
            continue;
        }
        let Some(application) = desktop_application_of_name(name_text) else {
            continue;
        };
        if holds_application(application, &directory, processes) {
            push_unique(targets, directory);
        } else {
            push_unique(without_application, directory);
        }
    }
    return Ok(());
}

/// Reads process file while treating process disappearance as ordinary scan race.
fn read_process_file(path: &Path) -> io::Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(text) => return Ok(Some(text)),
        Err(error) if means_vanished(&error) || error.kind() == io::ErrorKind::PermissionDenied => return Ok(None),
        Err(error) => return Err(error),
    }
}

/// Reads process executable while treating process disappearance as ordinary scan race.
fn read_process_executable(path: &Path) -> io::Result<Option<PathBuf>> {
    match std::fs::read_link(path) {
        Ok(executable) => return Ok(Some(executable)),
        Err(error) if means_vanished(&error) || error.kind() == io::ErrorKind::PermissionDenied => return Ok(None),
        Err(error) => return Err(error),
    }
}

/// What:
///      `scan_exempt_application_processes` borrows scan roots and mutable target list,
///           then returns `io::Result<()>`,
///  Rust's success-or-I/O-error wrapper with no success payload.
/// Why:
///       Live executables identify Helium,
///  Pale Moon,
///  Firefox release,
///  Firefox Nightly,
///  ChatGPT,
///           Interpreter,
///  Qure,
///  and OdyTTY cgroups beyond named-service coverage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanExemptApplicationProcesses(roots: ScanRoots, targets: string[]): void {
///   // Add live Helium, Pale Moon, Firefox release, and Nightly cgroups or throw an I/O error.
/// }
/// ```
fn scan_exempt_application_processes(processes: &[ApplicationProcess], targets: &mut Vec<PathBuf>) {
    for process in processes {
        if is_exempt_application_executable(&process.executable) {
            push_unique(targets, process.cgroup.clone());
        }
    }
}

/// What one scan found.
///
/// `without_application` names the cgroups a desktop application's name selected
/// while no process of that application was in them:
/// the mark is not attached there,
/// and the watcher says so,
/// because an application whose program the scan cannot recognize would otherwise lose its exemption in silence.
#[derive(Debug, PartialEq, Eq)]
pub struct ApplicationScan {
    /// Deduplicated,
    /// sorted cgroups that carry the socket mark.
    pub targets: Vec<PathBuf>,
    /// Sorted cgroups a desktop application's name selected that hold no program of it.
    pub without_application: Vec<PathBuf>,
}

/// Returns the cgroups requiring socket exemption,
/// and the names that were not believed.
pub fn scan_application_targets(roots: &ScanRoots<'_>) -> io::Result<ApplicationScan> {
    let processes = read_application_processes(roots)?;
    let mut targets = Vec::new();
    let mut without_application = Vec::new();
    scan_named_cgroups(roots, &processes, &mut targets, &mut without_application)?;
    scan_exempt_application_processes(&processes, &mut targets);
    targets.sort();
    targets.dedup();
    without_application.sort();
    without_application.dedup();
    return Ok(ApplicationScan { targets, without_application });
}

/// Builds production user `app.slice` path from numeric UID.
pub fn user_app_slice(uid: u32) -> PathBuf {
    return Path::new("/sys/fs/cgroup/user.slice")
        .join(format!("user-{uid}.slice"))
        .join(format!("user@{uid}.service"))
        .join("app.slice");
}

/// Scans production procfs and cgroup mount for one desktop user.
pub fn scan_user_application_targets(uid: u32) -> io::Result<ApplicationScan> {
    let app_slice = user_app_slice(uid);
    return scan_application_targets(&ScanRoots {
        app_slice: &app_slice,
        proc_root: Path::new("/proc"),
        cgroup_root: Path::new("/sys/fs/cgroup"),
    });
}
