//! What:
//! The controls of the read that tells a host with no user manager from a host whose manager was there and did not answer for a
//! moment:
//! one control for each way a host can have no manager,
//! and each of them requires one ask and no wait (issue 6629).
//! Why:
//! A host with no manager refuses every ask at once,
//! so the asking again sized to a manager's reexec spends its whole span there,
//! and the cost is paid by every call in a container,
//! a CI job or a session with no user manager.
//! The stand-in here is a stand-in:
//! it writes down each ask and hands it to the real `systemctl`,
//! so the refusal is the system's own words about the host this control builds,
//! and no claim about a real manager rests on the stand-in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('a host where no user manager can be reached', () => { it('is asked once', ...) })
//! ```

/// What:
/// `use std::fs;` imports the write of a stand-in and the read of the log it keeps.
/// Why:
/// Each control writes the program it plants and then reads what that program was asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync, writeFileSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::os::unix::fs::PermissionsExt;` imports the trait that sets a file's mode.
/// Why:
/// A stand-in that is not executable is not on the way to the manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { chmodSync } from 'node:fs';
/// ```
use std::os::unix::fs::PermissionsExt;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// The stand-in of a control,
/// the log it writes and the runtime directory it names are paths of that control's own scratch directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a path is a plain string in TS
/// ```
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `use std::time::{Duration, Instant};` imports a span of time and a reading of the monotonic clock.
/// Why:
/// Each control measures how long the discovery took,
/// which is how "does not wait" is checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds, and performance.now()
/// ```
use std::time::{
    Duration,
    Instant,
};

/// What:
/// `use crate::cgroup_test_support::this_test_needs_a_delegated_scope;` imports the line that says a control cannot run where this
/// host has no manager.
/// Why:
/// These controls build a host with no manager out of settings,
/// which is only evidence about that host where this one has a manager to lose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { thisTestNeedsADelegatedScope } from 'crate/cgroup_test_support';
/// ```
use crate::cgroup_test_support::this_test_needs_a_delegated_scope;
/// What:
/// `use crate::control_group_route_discovery::ASKING_AGAIN_PAUSE;` imports the wait between two asks.
/// Why:
/// What "does not wait" means is measured against the wait the discovery makes between two asks,
/// so the control reads that value rather than writing a second copy of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ASKING_AGAIN_PAUSE } from 'crate/control_group_route_discovery';
/// ```
use crate::control_group_route_discovery::ASKING_AGAIN_PAUSE;
/// What:
/// `use crate::control_group_route_discovery::{RouteSettings, detect_route};` imports the discovery and the settings that replace
/// the program it asks.
/// Why:
/// The question under control is the discovery's own,
/// asked through a program the control wrote and against a host the control built.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RouteSettings, detectRoute } from 'crate/control_group_route_discovery';
/// ```
use crate::control_group_route_discovery::{
    RouteSettings,
    detect_route,
};
/// What:
/// `use crate::control_group_route_manager_reach::{RUNTIME_DIRECTORY_VARIABLE, SESSION_BUS_VARIABLE};` imports the two variable
/// names a control plants a host with no manager through.
/// Why:
/// The control and the read have to name the same variables,
/// or the control would build a host the read never looks at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RUNTIME_DIRECTORY_VARIABLE, SESSION_BUS_VARIABLE } from 'crate/control_group_route_manager_reach';
/// ```
use crate::control_group_route_manager_reach::{
    RUNTIME_DIRECTORY_VARIABLE,
    SESSION_BUS_VARIABLE,
};
/// What:
/// `use crate::test_scratch::scratch;` imports the disposable directory of one control.
/// Why:
/// The stand-in,
/// its log and the runtime directory a control names live in a fixture of the control's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { scratch } from 'crate/test_scratch';
/// ```
use crate::test_scratch::scratch;

/// What:
/// `fn a_program(directory: &Path, name: &str, text: &str) -> PathBuf` writes an executable stand-in into a control's directory
/// and answers its path.
/// Why:
/// Each control plants one program in place of `systemctl`,
/// and a stand-in says it is one where it is defined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aProgram(dir: string, name: string, text: string): string
/// ```
fn a_program(
    directory: &Path,
    name: &str,
    text: &str,
) -> PathBuf {
    // The path the stand-in is written to.
    let program: PathBuf = directory.join(name);
    fs::write(&program, text).expect("write the stand-in");
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("make it executable");
    // The stand-in's path, for the settings that ask through it.
    return program;
}

/// What:
/// `fn real_systemctl() -> String` is the first `systemctl` on `PATH`.
/// Why:
/// The stand-in hands every ask to the real program,
/// so the words of the refusal are the system's own about the host the control built.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function realSystemctl(): string
/// ```
fn real_systemctl() -> String {
    // The search path, as one string of directories.
    let search: String = std::env::var("PATH").unwrap_or_default();
    for directory in search.split(':') {
        if directory.is_empty() {
            continue;
        }
        // One candidate program in that directory.
        let candidate: PathBuf = Path::new(directory).join("systemctl");
        if crate::control_group::is_executable_file(&candidate) {
            return candidate.display().to_string();
        }
    }
    panic!("no systemctl is on PATH, so this host has no user manager to ask");
}

/// What:
/// `fn a_stand_in_that_only_writes_down_each_ask(directory: &Path, log: &Path) -> PathBuf` writes a `systemctl` that logs each ask
/// and hands it to the real one.
/// Why:
/// The count of asks is the subject,
/// and the answer each ask gets is the host's own,
/// so the stand-in is a log and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aStandInThatOnlyWritesDownEachAsk(dir: string, log: string): string
/// ```
fn a_stand_in_that_only_writes_down_each_ask(
    directory: &Path,
    log: &Path,
) -> PathBuf {
    return a_program(
        directory,
        "systemctl-counting-the-asks",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nexec '{real}' \"$@\"\n",
            log = log.display(),
            real = real_systemctl()
        ),
    );
}

/// What:
/// `fn asked_once_and_not_waited_for(settings: &RouteSettings, log: &Path) -> String` runs the discovery,
/// requires that it refused after one ask and before two waits,
/// and answers the refusal's own words.
/// Why:
/// Both ways of having no manager must give the same answer about the asking,
/// and a function replaces a second copy of the three assertions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function askedOnceAndNotWaitedFor(settings: RouteSettings, log: string): string
/// ```
fn asked_once_and_not_waited_for(
    settings: &RouteSettings,
    log: &Path,
) -> String {
    // When the discovery began, so the time it took can be read from the clock.
    let began: Instant = Instant::now();
    // The refusal, which a host with no manager must give.
    let refused = detect_route(settings)
        .expect_err("a host with no manager to reach is a missing capability");
    // How long the whole discovery took.
    let took: Duration = began.elapsed();
    // Every ask the stand-in was given.
    let calls: String = fs::read_to_string(log).expect("the stand-in's log");
    assert_eq!(
        calls.lines().count(),
        1,
        "the question was put again where no manager can be reached: {calls:?}"
    );
    // Two waits are the most one ask and a loaded host can take: four of them are what the asking again always spends, so a
    // discovery that waited cannot answer under this bound.
    assert!(
        took < ASKING_AGAIN_PAUSE * 2,
        "the discovery answered after {took:?}, which is inside the span of waits it must not spend here ({ASKING_AGAIN_PAUSE:?} apart)"
    );
    return refused.to_string();
}

/// The control of the first way a host has no manager:
/// neither the runtime directory nor the bus address is set,
/// which is what a container,
/// a CI job or a session started without a manager gives.
#[test]
fn a_host_with_no_runtime_directory_and_no_bus_address_is_asked_once() {
    this_test_needs_a_delegated_scope();
    // The fixture directory of this control.
    let directory = scratch();
    // Where the stand-in writes down each ask.
    let log: PathBuf = directory.path().join("calls");
    // The settings of this control: the stand-in for `systemctl`, and the two variables removed as the ask would see them.
    let settings: RouteSettings = RouteSettings {
        systemctl: a_stand_in_that_only_writes_down_each_ask(directory.path(), &log)
            .display()
            .to_string(),
        environment: vec![
            (RUNTIME_DIRECTORY_VARIABLE.to_string(), None),
            (SESSION_BUS_VARIABLE.to_string(), None),
        ],
        ..RouteSettings::default()
    };

    // The words of the refusal.
    let refused: String = asked_once_and_not_waited_for(&settings, &log);

    assert!(
        refused.contains("no user manager can be reached (neither $XDG_RUNTIME_DIR nor $DBUS_SESSION_BUS_ADDRESS is set)"),
        "the refusal does not say what this process read: {refused}"
    );
}

/// The control of the second way a host has no manager:
/// the runtime directory is named and holds no manager socket,
/// which is what a container gives whose runtime directory was made and left empty.
#[test]
fn a_host_whose_runtime_directory_holds_no_manager_socket_is_asked_once() {
    this_test_needs_a_delegated_scope();
    // The fixture directory of this control.
    let directory = scratch();
    // Where the stand-in writes down each ask.
    let log: PathBuf = directory.path().join("calls");
    // The runtime directory this control names: made, and holding nothing at all.
    let runtime_directory: PathBuf = directory.path().join("a-runtime-directory-holding-nothing");
    fs::create_dir_all(&runtime_directory).expect("make the runtime directory");
    // The settings of this control: the stand-in for `systemctl`, that runtime directory, and no bus address.
    let settings: RouteSettings = RouteSettings {
        systemctl: a_stand_in_that_only_writes_down_each_ask(directory.path(), &log)
            .display()
            .to_string(),
        environment: vec![
            (
                RUNTIME_DIRECTORY_VARIABLE.to_string(),
                Some(runtime_directory.display().to_string()),
            ),
            (SESSION_BUS_VARIABLE.to_string(), None),
        ],
        ..RouteSettings::default()
    };

    // The words of the refusal.
    let refused: String = asked_once_and_not_waited_for(&settings, &log);

    assert!(
        refused.contains(
            "no user manager can be reached (nothing an ask could be carried over is there:"
        ) && refused.contains(&format!("{}/systemd/private", runtime_directory.display())),
        "the refusal does not say what this process read: {refused}"
    );
}
