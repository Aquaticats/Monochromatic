//! Discovers named Ghostty,
//!  Steam,
//!  Helium,
//!  Firefox release,
//!  Firefox Nightly,
//!  ChatGPT,
//!  Interpreter,
//!  Qure,
//!  OdyTTY,
//!  and Pi agent cgroups plus executable-owned application cgroups.

/// Filesystem and process-race failures.
use std::io;
/// Target and fixture roots.
use std::path::{Path, PathBuf};

/// Ghostty main user service prefix.
const GHOSTTY_SERVICE_PREFIX: &str = "app-com.mitchellh.ghostty@";
/// Ghostty terminal surface scope prefix.
const GHOSTTY_SURFACE_PREFIX: &str = "app-ghostty-surface-transient-";
/// What:
///      `STEAM_SERVICE_PREFIX` is immutable process-lifetime text borrowed from binary storage.
///           Rust spells that borrowed text type `&str`;
///  sibling `String` would allocate owned text.
/// Why:
///       Exact stable prefix identifies Steam's systemd service without allocating or matching unrelated names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STEAM_SERVICE_PREFIX = 'app-steam@';
/// ```
const STEAM_SERVICE_PREFIX: &str = "app-steam@";
/// Helium desktop integration service prefix observed from its Chrome application ID.
const HELIUM_SERVICE_PREFIX: &str =
    "app-chrome\\x2dcadlkienfkclaiaibeoongdcgmdikeeg\\x2dDefault@";
/// Firefox Nightly service prefix produced by its configured remoting name.
const FIREFOX_NIGHTLY_SERVICE_PREFIX: &str = "app-firefox\\x2dnightly@";
/// Firefox release service prefix produced by the `firefox.desktop` launcher.
const FIREFOX_RELEASE_SERVICE_PREFIX: &str = "app-firefox@";
/// ChatGPT desktop install tree whose every executable belongs to that application.
const CHATGPT_INSTALL_DIRECTORY: &str = "/usr/lib/chatgpt";
/// ChatGPT desktop service prefix observed from its desktop-entry identifier.
const CHATGPT_SERVICE_PREFIX: &str = "app-chatgpt@";
/// Interpreter executable family prefix shared by its AppImage file and bundled agents.
const INTERPRETER_EXECUTABLE_PREFIX: &str = "interpreter";
/// Interpreter desktop service prefix observed from its desktop-entry identifier.
const INTERPRETER_SERVICE_PREFIX: &str = "app-interpreter@";
/// Qure executable family prefix shared by its AppImage file and mounted Electron image.
const QURE_EXECUTABLE_PREFIX: &str = "qure";
/// Qure desktop service prefix observed from its desktop-entry identifier.
const QURE_SERVICE_PREFIX: &str = "app-qure@";
/// OdyTTY executable family prefix shared by its AppImage file and the native image it runs.
const ODYTTY_EXECUTABLE_PREFIX: &str = "odytty";
/// OdyTTY desktop service prefix observed from its desktop-entry identifier.
const ODYTTY_SERVICE_PREFIX: &str = "app-odytty@";

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

/// Reports Ghostty service or surface cgroup name.
pub fn is_ghostty_cgroup_name(name: &str) -> bool {
    return (name.starts_with(GHOSTTY_SERVICE_PREFIX) && name.ends_with(".service"))
        || (name.starts_with(GHOSTTY_SURFACE_PREFIX) && name.ends_with(".scope"));
}

/// What:
///      `is_steam_service_name` borrows candidate text as `&str` and returns primitive `bool`.
///           Borrowing avoids ownership transfer;
///  sibling `String` would require caller-owned allocation.
/// Why:
///       Watcher must select Steam's complete service cgroup while rejecting similarly named scopes and services.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isSteamServiceName(name: string): boolean {
///   return name.startsWith(STEAM_SERVICE_PREFIX) && name.endsWith('.service');
/// }
/// ```
fn is_steam_service_name(name: &str) -> bool {
    return name.starts_with(STEAM_SERVICE_PREFIX) && name.ends_with(".service");
}

/// Reports Helium desktop-integration service name before executable scan catches children.
pub fn is_helium_service_name(name: &str) -> bool {
    return name.starts_with(HELIUM_SERVICE_PREFIX) && name.ends_with(".service");
}

/// Reports Firefox Nightly desktop-integration service name.
pub fn is_firefox_nightly_service_name(name: &str) -> bool {
    return name.starts_with(FIREFOX_NIGHTLY_SERVICE_PREFIX)
        && name.ends_with(".service");
}

/// What:
/// Borrows candidate service text without allocating an owned `String`.
/// Why:
/// Select the release launcher while keeping ESR and similarly named scopes on the tunnel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isFirefoxReleaseServiceName(name: string): boolean {
///   return name.startsWith('app-firefox@') && name.endsWith('.service');
/// }
/// ```
pub fn is_firefox_release_service_name(name: &str) -> bool {
    return name.starts_with(FIREFOX_RELEASE_SERVICE_PREFIX)
        && name.ends_with(".service");
}

/// Reports ChatGPT desktop-integration service name.
pub fn is_chatgpt_service_name(name: &str) -> bool {
    return name.starts_with(CHATGPT_SERVICE_PREFIX) && name.ends_with(".service");
}

/// Reports Interpreter desktop-integration service name,
///  leaving the application's
/// own executable-named scope to executable discovery.
pub fn is_interpreter_service_name(name: &str) -> bool {
    return name.starts_with(INTERPRETER_SERVICE_PREFIX) && name.ends_with(".service");
}

/// Reports Qure desktop-integration service name,
///  leaving the application's
/// own executable-named scope to executable discovery.
pub fn is_qure_service_name(name: &str) -> bool {
    return name.starts_with(QURE_SERVICE_PREFIX) && name.ends_with(".service");
}

/// Reports OdyTTY desktop-integration service name,
///  leaving the application's
/// own executable-named scope to executable discovery.
pub fn is_odytty_service_name(name: &str) -> bool {
    return name.starts_with(ODYTTY_SERVICE_PREFIX) && name.ends_with(".service");
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

/// Reports whether the watcher attaches the socket mark to a cgroup of this name itself,
/// which is the cgroup a named service or scope keeps,
/// or the updates slice.
fn is_named_application_cgroup_name(name: &str) -> bool {
    return is_ghostty_cgroup_name(name)
        || is_steam_service_name(name)
        || is_helium_service_name(name)
        || is_firefox_nightly_service_name(name)
        || is_firefox_release_service_name(name)
        || is_chatgpt_service_name(name)
        || is_interpreter_service_name(name)
        || is_qure_service_name(name)
        || is_odytty_service_name(name)
        || is_pi_service_name(name)
        || is_toolchain_updates_slice_name(name);
}

/// Reports whether the watcher attaches the socket mark to this very cgroup of the watched user,
/// because it is a direct `app.slice` entry whose name one of the watcher's rules selects.
/// Such a cgroup needs no moved process:
/// the mark applies to every process in it and to every descendant cgroup the user's manager
/// creates below it,
/// which covers a service's main process,
/// the one thing the manager cannot move,
/// and every process Pi or a terminal starts,
/// the same rule either way.
pub fn is_socket_marked_cgroup(cgroup: &str, uid: u32) -> bool {
    let prefix = format!("/user.slice/user-{uid}.slice/user@{uid}.service/app.slice/");
    let Some(leaf) = cgroup.strip_prefix(&prefix) else {
        return false;
    };
    if leaf.is_empty() || leaf.contains('/') {
        return false;
    }
    return is_named_application_cgroup_name(leaf);
}

/// Reports numeric procfs directory name without regular expression parsing.
fn is_process_id(name: &str) -> bool {
    return !name.is_empty() && name.bytes().all(is_ascii_digit_byte);
}

/// Reports one ASCII decimal digit byte.
fn is_ascii_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// What:
///      `is_exempt_application_executable` borrows executable path as `&Path` and returns primitive `bool`.
///           Borrowing avoids ownership transfer;
///  sibling `PathBuf` would require caller-owned allocation.
/// Why:
///       Process scan must recognize Helium,
///  Pale Moon,
///  Firefox release,
///  Firefox Nightly,
///  ChatGPT,
///  Interpreter,
///           Qure,
///  and OdyTTY without matching other Firefox channels or unrelated install trees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isExemptApplicationExecutable(path: string): boolean {
///   const name = basename(path);
///   const isFirefoxNightly = path.endsWith('/firefox-nightly/firefox')
///     || path.endsWith('/firefox-nightly/firefox-bin');
///   const isFirefoxRelease = path.endsWith('/firefox/firefox')
///     || path.endsWith('/firefox/firefox-bin');
///   const isChatGpt = path.startsWith('/usr/lib/chatgpt/');
///   return name.toLowerCase().startsWith('helium')
///     || name === 'palemoon'
///     || name === 'palemoon-bin'
///     || isFirefoxNightly
///     || isFirefoxRelease
///     || isChatGpt
///     || name.startsWith('interpreter')
///     || name.startsWith('qure')
///     || name.startsWith('odytty');
/// }
/// ```
fn is_exempt_application_executable(path: &Path) -> bool {
    // What:     `let Some(name) = ... else` extracts present filename from Rust's `Option` wrapper.
    //           `Some` carries filename; sibling `None` means path has no final component.
    // Why:      Matching requires filename, while root-like paths must safely return false.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name = basename(path);
    // if (name === undefined) return false;
    // ```
    let Some(name) = path.file_name() else {
        return false;
    };
    // What:     `.to_string_lossy()` creates readable text from an OS filename, replacing invalid UTF-8.
    // Why:      Installed executable names are ASCII, while malformed names must remain safe to inspect.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const nameText = String(name);
    // ```
    let name_text = name.to_string_lossy();
    // What:     `.to_ascii_lowercase()` allocates lowercase owned text for Helium's case-insensitive prefix comparison.
    // Why:      Preserve existing Helium matching without widening Pale Moon beyond exact installed filenames.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const normalizedHeliumName = nameText.toLowerCase();
    // ```
    let normalized_helium_name = name_text.to_ascii_lowercase();
    // What:     `Path::ends_with` compares complete path components rather than a text suffix.
    // Why:      Exact install-directory and binary names distinguish Firefox Nightly
    //           from Firefox ESR and release Firefox.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const isFirefoxNightly = path.endsWith('/firefox-nightly/firefox')
    //   || path.endsWith('/firefox-nightly/firefox-bin');
    // ```
    let is_firefox_nightly = path.ends_with("firefox-nightly/firefox")
        || path.ends_with("firefox-nightly/firefox-bin");
    // Compare complete release-directory components, leaving ESR and backup directories unmatched.
    let is_firefox_release = path.ends_with("firefox/firefox")
        || path.ends_with("firefox/firefox-bin");
    // What:     `Path::starts_with` compares complete leading path components instead of a text prefix.
    // Why:      ChatGPT spawns every bundled agent from its own root-owned package tree,
    //           so that whole tree identifies the application even when a subprocess
    //           runs in a cgroup of its own, while `/usr/lib/chatgpt-backup` stays tunnel-routed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const isChatGpt = path.startsWith('/usr/lib/chatgpt/');
    // ```
    let is_chatgpt = path.starts_with(CHATGPT_INSTALL_DIRECTORY);
    // Interpreter's AppImage runtime file, its mounted inner binary, and its bundled agents
    // all carry the lowercase `interpreter` name prefix.
    // Unlike Helium's rule this comparison keeps the exact lowercase spelling,
    // so a capitalized lookalike stays tunnel-routed, while a renamed AppImage file still matches
    // through the lowercase inner binary it mounts.
    let is_interpreter = name_text.starts_with(INTERPRETER_EXECUTABLE_PREFIX);
    // Qure's AppImage runtime file, its mounted Electron image, its bundled pytest runner,
    // and every CLI wrapper it spawns all exec an image whose name begins with `qure`.
    // Its Chromium-generic helpers, such as `chrome_crashpad_handler`, carry no such name and
    // stay tunnel-routed on their own; they bypass only while sharing a Qure cgroup.
    let is_qure = name_text.starts_with(QURE_EXECUTABLE_PREFIX);
    // OdyTTY's AppImage runtime file and the native image it runs both carry the lowercase `odytty` prefix.
    // The comparison keeps the exact lowercase spelling, so a capitalized lookalike stays tunnel-routed.
    let is_odytty = name_text.starts_with(ODYTTY_EXECUTABLE_PREFIX);
    return normalized_helium_name.starts_with("helium")
        || name_text == "palemoon"
        || name_text == "palemoon-bin"
        || is_firefox_nightly
        || is_firefox_release
        || is_chatgpt
        || is_interpreter
        || is_qure
        || is_odytty;
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
/// The same names answer `is_socket_marked_cgroup`,
///  so the attaching and the process watch never disagree about which cgroup carries the mark.
fn scan_named_cgroups(roots: &ScanRoots<'_>, targets: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry_result in std::fs::read_dir(roots.app_slice)? {
        let entry = entry_result?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let Some(name_text) = name.to_str() else {
            continue;
        };
        if is_named_application_cgroup_name(name_text) {
            push_unique(targets, entry.path());
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
fn scan_exempt_application_processes(
    roots: &ScanRoots<'_>,
    targets: &mut Vec<PathBuf>,
) -> io::Result<()> {
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
        if !is_exempt_application_executable(&executable) {
            continue;
        }
        let Some(cgroup_text) = read_process_file(&process.join("cgroup"))? else {
            continue;
        };
        let Some(relative) = unified_cgroup_path(&cgroup_text) else {
            continue;
        };
        let target = roots.cgroup_root.join(relative.trim_start_matches('/'));
        if target.starts_with(roots.app_slice) && target.is_dir() {
            push_unique(targets, target);
        }
    }
    return Ok(());
}

/// Returns deduplicated sorted cgroups requiring socket exemption.
pub fn scan_application_targets(roots: &ScanRoots<'_>) -> io::Result<Vec<PathBuf>> {
    let mut targets = Vec::new();
    scan_named_cgroups(roots, &mut targets)?;
    scan_exempt_application_processes(roots, &mut targets)?;
    targets.sort();
    targets.dedup();
    return Ok(targets);
}

/// Builds production user `app.slice` path from numeric UID.
pub fn user_app_slice(uid: u32) -> PathBuf {
    return Path::new("/sys/fs/cgroup/user.slice")
        .join(format!("user-{uid}.slice"))
        .join(format!("user@{uid}.service"))
        .join("app.slice");
}

/// Scans production procfs and cgroup mount for one desktop user.
pub fn scan_user_application_targets(uid: u32) -> io::Result<Vec<PathBuf>> {
    let app_slice = user_app_slice(uid);
    return scan_application_targets(&ScanRoots {
        app_slice: &app_slice,
        proc_root: Path::new("/proc"),
        cgroup_root: Path::new("/sys/fs/cgroup"),
    });
}
