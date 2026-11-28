//! What:
//! The read that tells a host where no user manager can be reached from a host whose manager is there and did not answer
//! for a moment,
//! and the controls of that read.
//! Why:
//! The discovery asks its question again when an ask fails,
//! five times and two hundred milliseconds apart,
//! and that span is sized to a manager's reexec (issue 6623).
//! Where no manager can be reached at all,
//! every ask refuses at once and the whole span is spent,
//! which was about 0.88 s against a released binary's about 0.04 s for one `cargo-shadow install --dry-run`
//! (README,
//! "Development";
//! issue 6629).
//! A manager that is there and did not answer for a moment must still be asked again,
//! so the two are told apart by what this process reads before it asks again:
//! the runtime directory and the bus address the ask would see,
//! and the names they hold.
//! Never by the words of a refusal,
//! which come from the system's own message catalogue and follow the locale.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control_group_route_manager_reach.ts: aRefusalThatEndsTheAsking(settings, lastReason): string | undefined
//! ```

/// What:
/// `use std::ffi::OsString;` imports the owned operating-system string type (its siblings are `String`,
/// which must be UTF-8,
/// and `&OsStr`,
/// which is borrowed).
/// Why:
/// An environment variable's value is not promised to be UTF-8,
/// and a path read from it is used as it stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::ffi::OsString;
/// What:
/// `use std::fs;` imports the standard library's file-system module so this file can name it unqualified.
/// Why:
/// Whether one of the places an ask could be carried over is there is one `stat`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { statSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::os::unix::fs::FileTypeExt;` imports the trait that adds `is_socket` to a file's type,
/// which the plain type leaves out on purpose.
/// Why:
/// Only a socket carries an ask,
/// so a file's kind,
/// not only its name,
/// is what the read looks at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { statSync } from 'node:fs';   // statSync(...).isSocket()
/// ```
use std::os::unix::fs::FileTypeExt;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types (their sibling is `&Path`,
/// which is the same borrowed type under another name).
/// Why:
/// Each place a transport names is built from a directory and a name,
/// so each one is owned here,
/// while the check of what kind of thing it is lends it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::{
    Path,
    PathBuf,
};

/// What:
/// `use crate::control_group_route_discovery::RouteSettings;` imports the settings an ask of the manager is made with.
/// Why:
/// Those settings carry the environment a control plants,
/// and the read must see the one the ask would see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RouteSettings } from 'crate/control_group_route_discovery';
/// ```
use crate::control_group_route_discovery::RouteSettings;

/// The environment variable that names the directory where a user's manager keeps its own files and sockets.
/// Why:
/// `systemctl --user` looks there for the transport to its manager,
/// so its absence is the first of the two ways a host has no manager to reach.
pub(crate) const RUNTIME_DIRECTORY_VARIABLE: &str = "XDG_RUNTIME_DIR";

/// The environment variable that names the bus a user's programs ask their manager over.
/// Why:
/// It is the other half of the address `systemctl --user` builds,
/// and what the manager's own words name where neither is set.
/// Where it is set it is preferred over `$XDG_RUNTIME_DIR/bus` for the session bus
/// (`bus_set_address_user` at `src/libsystemd/sd-bus/sd-bus.c:1373-1398` at v262).
pub(crate) const SESSION_BUS_VARIABLE: &str = "DBUS_SESSION_BUS_ADDRESS";

/// Where inside the runtime directory the manager keeps its own socket (`unix:path=$XDG_RUNTIME_DIR/systemd/private`),
/// which is the transport `systemctl --user` takes first.
/// Why:
/// It is the answer to whether a manager is there where the runtime directory is set,
/// and it is the only transport systemd 262 takes there at all.
/// `bus_connect_transport_systemd` sends a user-scope ask to `bus_connect_user_systemd` (`src/shared/bus-util.c:519-524` at
/// v262), which connects `unix:path=$XDG_RUNTIME_DIR/systemd/private` and returns its failure rather than falling back
/// (`src/shared/bus-util.c:266-289`),
/// and systemd 255's copy of the same function falls back to the session bus where that connect fails
/// (`src/shared/bus-util.c:234-262`).
pub(crate) const MANAGER_SOCKET: &str = "systemd/private";

/// Where inside the runtime directory the socket of the user bus is (`unix:path=$XDG_RUNTIME_DIR/bus`, sd-bus's
/// `DEFAULT_USER_BUS_ADDRESS_FMT`):
/// the transport a systemd older than 262 falls back to where the manager's own socket cannot be connected and no bus address
/// is set.
/// Why:
/// A host that has a bus has this,
/// and a systemd 255 that cannot connect the private socket answers over it (`src/shared/bus-util.c:234-262` at v255,
/// `return sd_bus_default_user(ret_bus)` at `:255`),
/// while a systemd 262 never takes it (`src/shared/bus-util.c:266-289` at v262).
/// The read counts it because the package runs beside both.
/// Where `$DBUS_SESSION_BUS_ADDRESS` is set, sd-bus takes the address rather than this name
/// (`bus_set_address_user` at `src/libsystemd/sd-bus/sd-bus.c:1373-1398` at v262 and `:1336-1361` at v255),
/// so this name is not a transport such a host can take.
pub(crate) const SESSION_BUS_SOCKET: &str = "bus";

/// What inside the runtime directory shows that a manager keeps its files there,
/// whether or not its socket is there at this instant.
/// Why:
/// A reexec of the manager unlinks and rebinds its socket for a moment,
/// 0.17 ms to 0.28 ms of one measured in a container of its own on 2026-11-27,
/// and a read that took that moment for "no manager" would break the asking again across a reexec.
/// The directory stays for the whole reexec,
/// so it is what says a manager was there and keeps its files,
/// and a host that never had one has neither.
pub(crate) const MANAGER_FILES_DIRECTORY: &str = "systemd";

/// What:
/// `pub(crate) fn a_refusal_that_ends_the_asking(settings, last_reason) -> Option<String>` says what a failure should refuse
/// with where no user manager can be reached,
/// and `None` where one can be or where this process cannot tell.
/// Why:
/// A host with no manager refuses the same way every time,
/// so putting the question again spends the whole span on a host that has already answered (issue 6629),
/// while a moment of a manager that is there must still be asked through.
/// The words are the failure's own,
/// and then what this read found,
/// so a run that goes on in the weaker path still names the manager's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aRefusalThatEndsTheAsking(settings: RouteSettings, lastReason: string): string | undefined
/// ```
pub(crate) fn a_refusal_that_ends_the_asking(
    settings: &RouteSettings,
    last_reason: &str,
) -> Option<String> {
    // Why this process reads that no manager can be reached, where it does.
    let why: String = why_no_manager_can_be_reached(settings)?;
    // `Some(...)`: the refusal to end the asking with, which names the failure and then the read.
    return Some(format!(
        "{last_reason}; no user manager can be reached ({why})"
    ));
}

/// What:
/// `fn why_no_manager_can_be_reached(settings) -> Option<String>` names what was read where nothing on this host could carry an
/// ask to a user manager,
/// and answers `None` where something could,
/// or where what is named cannot be looked for.
/// Why:
/// The two cases are told apart by what is on disk and in the environment,
/// and `None` is the answer that keeps the asking again,
/// because a wrong "nothing here" would turn a reexec into a refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whyNoManagerCanBeReached(settings: RouteSettings): string | undefined
/// ```
fn why_no_manager_can_be_reached(settings: &RouteSettings) -> Option<String> {
    // The runtime directory the ask would see, where it names one.
    let runtime_directory: Option<OsString> =
        a_variable_the_ask_would_see(settings, RUNTIME_DIRECTORY_VARIABLE);
    // The bus address the ask would see, where it names one.
    let bus_address: Option<OsString> =
        a_variable_the_ask_would_see(settings, SESSION_BUS_VARIABLE);
    if runtime_directory.is_none() && bus_address.is_none() {
        // Neither is set: `systemctl --user` refuses with the words that say so, and nothing here names a transport at all.
        return Some(format!(
            "neither ${RUNTIME_DIRECTORY_VARIABLE} nor ${SESSION_BUS_VARIABLE} is set"
        ));
    }
    // Every socket an ask could be carried over.
    let mut sockets: Vec<PathBuf> = Vec::new();
    // Where the manager keeps its files, where a runtime directory names a place for them.
    let mut manager_files: Option<PathBuf> = None;
    if let Some(directory) = &runtime_directory {
        // The directory the variable names, as a path this process can look at.
        let root: PathBuf = PathBuf::from(directory);
        // What:     `root.join(MANAGER_SOCKET)` is the manager's own socket, the transport `systemctl --user` takes first where
        //           `$XDG_RUNTIME_DIR` is set.
        // Why:      Where it connects, that is the transport the ask went over, on systemd 262 and 255 alike.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // sockets.push(path.join(root, 'systemd/private'))
        // ```
        sockets.push(root.join(MANAGER_SOCKET));
        // What:     `bus_address.is_none()` is the condition under which `root.join(SESSION_BUS_SOCKET)` is a transport an ask
        //           can take: sd-bus prefers `$DBUS_SESSION_BUS_ADDRESS` to the user bus socket where the address is set.
        // Why:      A read that counted the bus socket beside a bus address would keep the asking again on a host whose only
        //           named transport is the address (measured 2026-11-27 in a container: with both variables set, systemd 262
        //           takes `systemd/private` alone and systemd 255 takes the address, and neither takes the bus socket).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (busAddress === undefined) sockets.push(path.join(root, 'bus'))
        // ```
        if bus_address.is_none() {
            // What:     `sockets.push(root.join(SESSION_BUS_SOCKET))` adds the user bus socket as a place an ask could be carried
            //           over.
            // Why:      A systemd 255 falls back to it where the private socket cannot be connected and no address is set.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // sockets.push(path.join(root, 'bus'))
            // ```
            sockets.push(root.join(SESSION_BUS_SOCKET));
        }
        manager_files = Some(root.join(MANAGER_FILES_DIRECTORY));
    }
    if let Some(address) = &bus_address {
        // `?`: an address this process cannot read as socket paths is a transport it cannot judge, so nothing is claimed about
        // it and the answer is that a manager may be there.
        let found: Vec<PathBuf> = paths_in_a_bus_address(address)?;
        for path in found {
            sockets.push(path);
        }
    }
    // Whether one of those places is there, which is all it takes to keep asking.
    let mut one_is_there: bool = false;
    // Every place that was read, in the words the refusal names them with.
    let mut read: Vec<String> = Vec::new();
    for path in &sockets {
        read.push(path.display().to_string());
        if is_a_socket(path) {
            one_is_there = true;
        }
    }
    if let Some(path) = &manager_files {
        read.push(path.display().to_string());
        if is_a_directory(path) {
            one_is_there = true;
        }
    }
    if one_is_there {
        return None;
    }
    // `Some(...)`: nothing an ask could be carried over is there, and every place that was read is named.
    return Some(format!(
        "nothing an ask could be carried over is there: {}",
        read.join(", ")
    ));
}

/// What:
/// `fn is_a_socket(path: &Path) -> bool` says whether a path is a socket of the file system (its siblings are
/// `is_dir` and `is_file`,
/// and a plain file at a socket's name is neither a transport nor a manager).
/// Why:
/// A file that is not a socket carries no ask,
/// so reading its name as a transport would keep the asking again on a host where nothing can be reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isASocket(path: string): boolean
/// ```
fn is_a_socket(path: &Path) -> bool {
    return fs::metadata(path).is_ok_and(|found| {
        return found.file_type().is_socket();
    });
}

/// What:
/// `fn is_a_directory(path: &Path) -> bool` says whether a path is a directory of the file system.
/// Why:
/// The manager's own directory is what says a manager keeps its files there,
/// and a file at that name is not one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isADirectory(path: string): boolean
/// ```
fn is_a_directory(path: &Path) -> bool {
    return fs::metadata(path).is_ok_and(|found| {
        return found.is_dir();
    });
}

/// What:
/// `fn a_variable_the_ask_would_see(settings, name) -> Option<OsString>` is the value of one environment variable as the program
/// an ask starts would see it,
/// and `None` where it is unset or empty.
/// Why:
/// A control plants a host with no manager by removing or replacing these variables for the ask,
/// so the read takes this process's own value,
/// lets the settings change it as the child's environment does,
/// and reads what the ask would see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aVariableTheAskWouldSee(settings: RouteSettings, name: string): string | undefined
/// ```
fn a_variable_the_ask_would_see(
    settings: &RouteSettings,
    name: &str,
) -> Option<OsString> {
    // What this process itself holds for that name, where it holds anything.
    let mut found: Option<OsString> = std::env::var_os(name);
    for (key, value) in &settings.environment {
        if key != name {
            continue;
        }
        // `value.clone().map(OsString::from)`: the settings hold a `String` and a removed one is a `None`, and this read keeps
        // the same shape as what the process holds.
        found = value.clone().map(OsString::from);
    }
    if let Some(text) = &found
        && text.is_empty()
    {
        // An empty variable names nothing, which is what the ask sees when it is empty too.
        return None;
    }
    return found;
}

/// What:
/// `fn paths_in_a_bus_address(address) -> Option<Vec<PathBuf>>` is every socket path a bus address names,
/// or `None` where one of its entries is not a socket path in the file system.
/// Why:
/// The address `systemctl --user` is given is a list of transports,
/// and only the ones that name a path can be looked for;
/// a transport that cannot be looked for must not be read as absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathsInABusAddress(address: string): string[] | undefined
/// ```
fn paths_in_a_bus_address(address: &OsString) -> Option<Vec<PathBuf>> {
    // `?`: an address that is not text names no socket path this read can look for.
    let text: &str = address.to_str()?;
    // The paths the address names, one for each of its entries.
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in text.split(';') {
        if entry.is_empty() {
            continue;
        }
        // `?`: an entry of another transport is no path in this file system.
        let rest: &str = entry.strip_prefix("unix:")?;
        // The path this entry names, where it names one.
        let mut named: Option<&str> = None;
        for key_and_value in rest.split(',') {
            if let Some(path) = key_and_value.strip_prefix("path=") {
                named = Some(path);
            }
        }
        // `?`: `unix:abstract=` names a socket with no path in the file system, so it cannot be looked for here.
        let path: &str = named?;
        // `?`: a bus address escapes each byte it may not write plainly as `%` and two hex digits, and this read does not undo
        // that, so a name that holds a `%` at all is one it cannot look for as written.
        // sd-bus escapes a literal `%` as `%25`, so a `%` here is always an escape and never a name's own character.
        if path.contains('%') {
            return None;
        }
        paths.push(PathBuf::from(path));
    }
    if paths.is_empty() {
        return None;
    }
    return Some(paths);
}

/// The controls of the read:
/// a host with no runtime directory and no bus address,
/// and a host whose runtime directory holds no manager socket (issue 6629).
#[cfg(test)]
#[path = "control_group_route_manager_reach_tests.rs"]
mod manager_reach_tests;
