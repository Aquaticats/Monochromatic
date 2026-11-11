//! The desktop applications a cgroup name claims,
//! and the program families that prove one is there.
//!
//! A socket mark is exemption from the tunnel,
//! so a name is believed only while the cgroup it names holds that application's own program,
//! except for the two names the owner accepted as names alone:
//! a Pi agent's unit and the updates slice.
//! The families here are the one place that knows what an application is,
//! so a name and a program can never disagree.

/// The file name of an executable path.
use std::path::Path;

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
pub fn is_steam_service_name(name: &str) -> bool {
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

/// The desktop application a socket-marked cgroup name claims.
///
/// The name alone is a name the user can choose,
/// so each of these is marked only while the cgroup it names holds a process running that application's own program.
/// Pi units and the updates slice are the two names the owner accepted as names alone.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DesktopApplication {
    /// The Ghostty terminal,
    /// its user service or one of its surface scopes.
    Ghostty,
    /// Steam's user service.
    Steam,
    /// Helium's desktop integration service.
    Helium,
    /// Firefox Nightly's service.
    FirefoxNightly,
    /// Firefox release's service,
    /// the launcher `firefox.desktop` starts.
    FirefoxRelease,
    /// ChatGPT's desktop integration service.
    Chatgpt,
    /// Interpreter's desktop integration service.
    Interpreter,
    /// Qure's desktop integration service.
    Qure,
    /// OdyTTY's desktop integration service.
    OdyTty,
}

/// The file name of an executable path,
/// or nothing when the path has no final component.
fn executable_name(path: &Path) -> Option<std::borrow::Cow<'_, str>> {
    let name = path.file_name()?;
    return Some(name.to_string_lossy());
}

/// Reports whether an executable path is the program of the application a cgroup name claims.
///
/// The families are the same ones `is_exempt_application_executable` matches,
/// so a name and a program can never disagree about what an application is.
pub fn is_application_executable(application: DesktopApplication, path: &Path) -> bool {
    let Some(name) = executable_name(path) else {
        return false;
    };
    return match application {
        DesktopApplication::Ghostty => is_ghostty_executable(&name),
        DesktopApplication::Steam => is_steam_executable(&name),
        DesktopApplication::Helium => is_helium_executable(&name),
        DesktopApplication::FirefoxNightly => is_firefox_nightly_executable(path),
        DesktopApplication::FirefoxRelease => is_firefox_release_executable(path),
        DesktopApplication::Chatgpt => is_chatgpt_executable(path),
        DesktopApplication::Interpreter => is_interpreter_executable(&name),
        DesktopApplication::Qure => is_qure_executable(&name),
        DesktopApplication::OdyTty => is_odytty_executable(&name),
    };
}

/// Reports whether which desktop application a cgroup name claims,
/// or nothing for a name that claims none.
pub fn desktop_application_of_name(name: &str) -> Option<DesktopApplication> {
    if is_ghostty_cgroup_name(name) {
        return Some(DesktopApplication::Ghostty);
    }
    if is_steam_service_name(name) {
        return Some(DesktopApplication::Steam);
    }
    if is_helium_service_name(name) {
        return Some(DesktopApplication::Helium);
    }
    if is_firefox_nightly_service_name(name) {
        return Some(DesktopApplication::FirefoxNightly);
    }
    if is_firefox_release_service_name(name) {
        return Some(DesktopApplication::FirefoxRelease);
    }
    if is_chatgpt_service_name(name) {
        return Some(DesktopApplication::Chatgpt);
    }
    if is_interpreter_service_name(name) {
        return Some(DesktopApplication::Interpreter);
    }
    if is_qure_service_name(name) {
        return Some(DesktopApplication::Qure);
    }
    if is_odytty_service_name(name) {
        return Some(DesktopApplication::OdyTty);
    }
    return None;
}

/// Reports Ghostty's program,
/// whose service and surface cgroups carry the terminal's name.
fn is_ghostty_executable(name: &str) -> bool {
    return name == "ghostty";
}

/// Reports Steam's launcher,
/// the program its user service runs.
fn is_steam_executable(name: &str) -> bool {
    return name == "steam";
}

/// Reports Helium's executable family by its case-insensitive prefix.
fn is_helium_executable(name: &str) -> bool {
    return name.to_ascii_lowercase().starts_with("helium");
}

/// Reports Pale Moon's two installed executable names.
fn is_palemoon_executable(name: &str) -> bool {
    return (name == "palemoon") || (name == "palemoon-bin");
}

/// Reports Firefox Nightly's image,
/// which only its own install directory carries.
fn is_firefox_nightly_executable(path: &Path) -> bool {
    return path.ends_with("firefox-nightly/firefox")
        || path.ends_with("firefox-nightly/firefox-bin");
}

/// Reports release Firefox's image,
/// which only its own install directory carries,
/// leaving ESR and backup directories unmatched.
fn is_firefox_release_executable(path: &Path) -> bool {
    return path.ends_with("firefox/firefox") || path.ends_with("firefox/firefox-bin");
}

/// Reports any program inside ChatGPT's root-owned package tree,
/// which includes the subprocess agents that application spawns.
fn is_chatgpt_executable(path: &Path) -> bool {
    return path.starts_with(CHATGPT_INSTALL_DIRECTORY);
}

/// Reports Interpreter's executable family by its exact lowercase prefix.
fn is_interpreter_executable(name: &str) -> bool {
    return name.starts_with(INTERPRETER_EXECUTABLE_PREFIX);
}

/// Reports Qure's executable family by its exact lowercase prefix.
fn is_qure_executable(name: &str) -> bool {
    return name.starts_with(QURE_EXECUTABLE_PREFIX);
}

/// Reports OdyTTY's executable family by its exact lowercase prefix.
fn is_odytty_executable(name: &str) -> bool {
    return name.starts_with(ODYTTY_EXECUTABLE_PREFIX);
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
///   return name.toLowerCase().startsWith('helium')
///     || name === 'palemoon'
///     || name === 'palemoon-bin'
///     || isFirefoxNightly(path)
///     || isFirefoxRelease(path)
///     || isChatgpt(path)
///     || name.startsWith('interpreter')
///     || name.startsWith('qure')
///     || name.startsWith('odytty');
/// }
/// ```
pub fn is_exempt_application_executable(path: &Path) -> bool {
    let Some(name) = executable_name(path) else {
        return false;
    };
    return is_helium_executable(&name)
        || is_palemoon_executable(&name)
        || is_firefox_nightly_executable(path)
        || is_firefox_release_executable(path)
        || is_chatgpt_executable(path)
        || is_interpreter_executable(&name)
        || is_qure_executable(&name)
        || is_odytty_executable(&name);
}
