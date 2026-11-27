//! What:
//!  The one place this crate reads this session's runtime directory and this session's session-bus address,
//!  which are the two facts a commit's route to its service is chosen from.
//! Why:
//!  Both facts name places outside any fixture,
//!  and a test process shares this machine's account with the commits of every other session.
//!  So a build made for tests answers them only from what a control handed in
//!  (`RUNTIME_DIRECTORY_SWITCH` and `BUS_ADDRESS_SWITCH` below),
//!  never from the environment the test process inherited,
//!  and a body that hands in nothing gets no place and therefore no route
//!  (issue 6586:
//!  a threaded run of the library's tests on the host asked this account's own manager to start commit
//!  services for its fixtures,
//!  and pointed the program link every worktree's service is started through at the test binary).
//!  A build the release ships answers them from the process's own environment,
//!  which is how a command started with another `XDG_RUNTIME_DIR` is judged.
//!  Every read of either name in this crate comes through this file,
//!  which `source_guard/no_release_read_of_the_session_places_tests.rs` refuses to let drift.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // The two facts, each answered by the release's own environment and by a control's switch alone in a test build.
//! const runtimeDirectory = () => sessionPlace('XDG_RUNTIME_DIR');
//! const busAddress = () => sessionPlace('DBUS_SESSION_BUS_ADDRESS');
//! ```

/// What:
///  `OsString` is owned operating-system text,
///  of raw bytes on Linux,
///  whose siblings are `String`,
///  an owned UTF-8 buffer,
///  and `&OsStr`,
///  a borrowed view of one.
/// Why:
///  A runtime directory's name is bytes,
///  and the environment this file reads hands it over as bytes;
///  `String` would refuse every name that is not UTF-8,
///  which would make the person's own runtime directory invisible.
use std::ffi::OsString;
/// What:
///  `PathBuf` is an owned filesystem path,
///  its siblings being `&Path`,
///  a borrowed view,
///  and `OsString`,
///  which is not a path at all.
/// Why:
///  The route keeps the directory after the environment it came from is gone,
///  and every caller builds a socket path or a unit folder under it.
use std::path::PathBuf;

/// What:
///  The switch a build made for tests takes this session's runtime directory from,
///  which no build the release ships reads and no shipped process sets.
/// Why:
///  A control names a runtime directory of its own in the child it runs its body in,
///  so the route a hand-over finds its service by never reads the account's own place,
///  whatever the run's environment holds
///  (issue 6586).
///  The name carries the crate's test-only prefix,
///  so the ban on a release file reading such a variable
///  covers it too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RUNTIME_DIRECTORY_SWITCH = 'CLI_GIT_TEST_ONLY_SESSION_RUNTIME_DIRECTORY';
/// ```
#[cfg(test)]
pub(crate) const RUNTIME_DIRECTORY_SWITCH: &str = "CLI_GIT_TEST_ONLY_SESSION_RUNTIME_DIRECTORY";

/// What:
///  The switch a build made for tests takes this session's session-bus address from,
///  which no build the release ships reads and no shipped process sets.
/// Why:
///  The bus address is the second fact provisioning needs,
///  so a control that hands its body a runtime directory of its own must be able to say that no bus is
///  reachable there,
///  rather than inherit this account's bus from the run's environment
///  (issue 6586).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BUS_ADDRESS_SWITCH = 'CLI_GIT_TEST_ONLY_SESSION_BUS_ADDRESS';
/// ```
#[cfg(test)]
pub(crate) const BUS_ADDRESS_SWITCH: &str = "CLI_GIT_TEST_ONLY_SESSION_BUS_ADDRESS";

/// What:
///  The directory a name holds,
///  or nothing when no name was given or the name is empty.
/// Why:
///  An empty value names no place,
///  and both the route and the loads that read the runtime directory treat an empty one as unset;
///  one function says that once,
///  so no caller can decide it differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namedDirectory(named: string | undefined): string | undefined { return named ? named : undefined; }
/// ```
fn named_directory(named: Option<OsString>) -> Option<PathBuf> {
    // What:     `Some(value)` is the "present" variant of `Option`,
    //           and the `?` after `named` hands the function `None` at once when no name was given.
    // Why:      `XDG_RUNTIME_DIR` unset is a session that names no place, and this function answers `None` for it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (named === undefined) return undefined;
    // ```
    let value: OsString = named?;
    if value.is_empty() {
        return None;
    }
    // What:     `PathBuf::from(value)` moves the owned bytes into an owned path, the type every caller here wants.
    // Why:      The bytes came from the environment, so nothing borrows them once this function returns.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return named;
    // ```
    return Some(PathBuf::from(value));
}

/// What:
///  The runtime directory this session's own environment names,
///  or nothing when it names none.
/// Why:
///  A command started with another runtime directory is judged by that one,
///  so the release reads the process's environment and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runtimeDirectory(): string | undefined { return process.env.XDG_RUNTIME_DIR || undefined; }
/// ```
#[cfg(not(test))]
pub fn runtime_directory() -> Option<PathBuf> {
    // What:     `std::env::var_os` reads one variable of this process's environment as raw bytes.
    // Why:      `std::env::var` answers nothing for a name that is not UTF-8, and the person's own runtime
    //           directory would then name no place at all.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const named = process.env.XDG_RUNTIME_DIR;
    // ```
    let named: Option<OsString> = std::env::var_os("XDG_RUNTIME_DIR");
    return named_directory(named);
}

/// What:
///  The runtime directory the control that started this process handed in,
///  or nothing when it handed in none.
/// Why:
///  A test process's own environment names this account's runtime directory,
///  and a route that read it would hand the account's own service manager a unit for a fixture
///  (issue 6586).
///  So a build made for tests reads the switch alone,
///  and a body whose control named none gets no runtime directory at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runtimeDirectory(): string | undefined { return process.env.CLI_GIT_TEST_ONLY_SESSION_RUNTIME_DIRECTORY || undefined; }
/// ```
#[cfg(test)]
pub fn runtime_directory() -> Option<PathBuf> {
    // What:     `crate::test_only_switches::value` is the one read of a `CLI_GIT_TEST_ONLY_*` variable, and it
    //           answers nothing in a build made without the `test-only-switches` feature.
    // Why:      A build made for tests without that feature reads no switch, so its route answers no runtime
    //           directory either, which is the safe answer for a test process.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const named = testOnlySwitchValue(RUNTIME_DIRECTORY_SWITCH);
    // ```
    let named: Option<OsString> = crate::test_only_switches::value(RUNTIME_DIRECTORY_SWITCH);
    return named_directory(named);
}

/// What:
///  The session-bus address this session's own environment names,
///  or nothing when it names none,
///  which leaves the route with the `bus` of the runtime directory alone.
/// Why:
///  The address is how provisioning reaches the manager,
///  so the release reads the one this session gave it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function busAddress(): string | undefined { return process.env.DBUS_SESSION_BUS_ADDRESS; }
/// ```
#[cfg(not(test))]
pub fn bus_address() -> Option<String> {
    // What:     `std::env::var` reads one variable as UTF-8 text, and `.ok()` turns its "not there or not text"
    //           answer into the "nothing" this function returns.
    // Why:      A bus address is text a parser splits on `;` and `,`, so bytes that are not UTF-8 name no bus this
    //           crate can parse, and the route falls back to the runtime directory's own `bus`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return process.env.DBUS_SESSION_BUS_ADDRESS;
    // ```
    return std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
}

/// What:
///  The session-bus address the control that started this process handed in,
///  or nothing when it handed in none.
/// Why:
///  A test process inherits this account's bus address,
///  and a fixture's own runtime directory does not keep a program off that bus:
///  a control that handed its body a runtime directory of its own and left the address inherited would
///  still let the route answer with this account's manager
///  (issue 6586).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function busAddress(): string | undefined { return process.env.CLI_GIT_TEST_ONLY_SESSION_BUS_ADDRESS; }
/// ```
#[cfg(test)]
pub fn bus_address() -> Option<String> {
    return crate::test_only_switches::text(BUS_ADDRESS_SWITCH);
}

/// Controls for the two answers:
/// what a release reads and what a build made for tests reads.
#[cfg(test)]
#[path = "session_runtime_directory_and_bus_address_tests.rs"]
mod tests;
