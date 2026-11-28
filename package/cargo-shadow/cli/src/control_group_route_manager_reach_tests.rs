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
/// `use crate::control_group_route_discovery::ASKING_AGAIN_ATTEMPTS;` imports how many times the discovery puts the question
/// before a failure is taken for this host's answer.
/// Why:
/// The controls below that read a transport on the host require the asking again to be kept there,
/// and the count of attempts is what "kept" means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ASKING_AGAIN_ATTEMPTS } from 'crate/control_group_route_discovery'
/// ```
use crate::control_group_route_discovery::ASKING_AGAIN_ATTEMPTS;
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
/// `use crate::control_group_route_manager_reach::{MANAGER_FILES_DIRECTORY, MANAGER_SOCKET, SESSION_BUS_SOCKET};` imports the
/// names the read looks for,
/// so a control builds the state the read is about.
/// Why:
/// A control that wrote `"systemd/private"` itself would pass after the read's own constant moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { MANAGER_FILES_DIRECTORY, MANAGER_SOCKET, SESSION_BUS_SOCKET } from 'crate/control_group_route_manager_reach'
/// ```
use crate::control_group_route_manager_reach::{
    MANAGER_FILES_DIRECTORY,
    MANAGER_SOCKET,
    SESSION_BUS_SOCKET,
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

/// What:
/// `fn a_socket_node(path: &Path)` makes a socket file of the file system at a path,
/// with no program listening on it.
/// Why:
/// The read judges a transport by the file's kind alone,
/// so the node is the whole state;
/// binding a listener would need a socket address,
/// and these fixture paths are longer than one holds (`SUN_LEN` is 108 bytes on this host,
/// and every path under this package's scratch folder is longer).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aSocketNode(path: string): void  // mknod(path, S_IFSOCK | 0o600)
/// ```
fn a_socket_node(path: &Path) {
    // What:     `rustix::fs::mknodat(rustix::fs::CWD, path, FileType::Socket, ..., 0)` makes a node of the given kind at
    //           `path`, with no device number, in this process's own directory.
    // Why:      It is the only way to a socket node without a bound address, which is what the read looks at.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // mknodSync(path, S_IFSOCK | 0o600)
    // ```
    rustix::fs::mknodat(
        rustix::fs::CWD,
        path,
        rustix::fs::FileType::Socket,
        rustix::fs::Mode::from_bits_truncate(0o600),
        0,
    )
    .expect("make a socket file");
}

/// What:
/// `fn asked_again(settings: &RouteSettings, log: &Path) -> String` runs the discovery,
/// requires that the question was put once per attempt,
/// and answers the refusal's own words.
/// Why:
/// A transport the read counts must keep the asking again,
/// and the count of asks is what "kept" means (issue 6629).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function askedAgain(settings: RouteSettings, log: string): string
/// ```
fn asked_again(
    settings: &RouteSettings,
    log: &Path,
) -> String {
    // The refusal, which a host that cannot reach a manager must give.
    let refused = detect_route(settings)
        .expect_err("a host with no manager to reach is a missing capability");
    // Every ask the stand-in was given.
    let calls: String = fs::read_to_string(log).expect("the stand-in's log");
    // What:     `usize::try_from(ASKING_AGAIN_ATTEMPTS)` is the attempt count as the width `lines().count()` answers in.
    // Why:      The count of attempts is a `u32`, and the count of lines is a `usize`; comparing them needs one width.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const wanted = ASKING_AGAIN_ATTEMPTS
    // ```
    let wanted: usize = usize::try_from(ASKING_AGAIN_ATTEMPTS)
        .expect("an attempt count that fits a count of lines");
    assert_eq!(
        calls.lines().count(),
        wanted,
        "a transport an ask could be carried over was read on this host, so the asking again must be kept: {calls:?}"
    );
    return refused.to_string();
}

/// The control of the first transport the read counts:
/// a socket at the manager's own name inside the runtime directory,
/// which is where `systemctl --user` looks first and where systemd 262 looks only.
/// The asking again is kept there,
/// and the socket carries nothing at all.
#[test]
fn a_manager_socket_at_the_runtime_directory_keeps_the_asking_again() {
    this_test_needs_a_delegated_scope();
    // The fixture directory of this control.
    let directory = scratch();
    // Where the stand-in writes down each ask.
    let log: PathBuf = directory.path().join("calls");
    // The runtime directory this control names.
    let runtime_directory: PathBuf = directory.path().join("a-runtime-directory");
    fs::create_dir_all(runtime_directory.join(MANAGER_FILES_DIRECTORY))
        .expect("make the manager's directory");
    // What:     `a_socket_node(...)` plants the manager's socket where an ask would look for it.
    // Why:      The read judges a transport by the file's kind, and this is the one systemd 262 takes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // aSocketNode(join(runtimeDirectory, MANAGER_SOCKET))
    // ```
    a_socket_node(&runtime_directory.join(MANAGER_SOCKET));
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

    let refused: String = asked_again(&settings, &log);

    assert!(
        refused.contains("the user manager cannot be asked where app.slice is ("),
        "the refusal does not name the ask that failed: {refused}"
    );
}

/// The control of the second transport the read counts:
/// a socket at the user bus's name inside the runtime directory and no bus address,
/// which is where a systemd 255 falls back to.
#[test]
fn a_bus_socket_at_the_runtime_directory_keeps_the_asking_again_where_no_bus_address_is_set() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    let runtime_directory: PathBuf = directory.path().join("a-runtime-directory");
    fs::create_dir_all(&runtime_directory).expect("make the runtime directory");
    // What:     `a_socket_node(...)` plants the user bus's socket at its own name.
    // Why:      A systemd 255 answers over it where the manager's own socket cannot be connected, so the read counts it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // aSocketNode(join(runtimeDirectory, SESSION_BUS_SOCKET))
    // ```
    a_socket_node(&runtime_directory.join(SESSION_BUS_SOCKET));
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

    asked_again(&settings, &log);
}

/// The control of the place a host with no runtime directory asks over:
/// a bus address naming a socket,
/// which is the transport both systemd versions take there.
#[test]
fn a_bus_address_naming_a_socket_keeps_the_asking_again_where_no_runtime_directory_is_set() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    let bus_socket: PathBuf = directory.path().join("a-bus-socket");
    // What:     `a_socket_node(&bus_socket)` plants the socket the address names.
    // Why:      An ask over a bus address is carried over the socket the address names, and that is what the read looks for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // aSocketNode(busSocket)
    // ```
    a_socket_node(&bus_socket);
    let settings: RouteSettings = RouteSettings {
        systemctl: a_stand_in_that_only_writes_down_each_ask(directory.path(), &log)
            .display()
            .to_string(),
        environment: vec![
            (RUNTIME_DIRECTORY_VARIABLE.to_string(), None),
            (
                SESSION_BUS_VARIABLE.to_string(),
                Some(format!("unix:path={}", bus_socket.display())),
            ),
        ],
        ..RouteSettings::default()
    };

    asked_again(&settings, &log);
}

/// The control of the address a runtime directory does not replace:
/// a runtime directory that holds nothing,
/// and a bus address naming a socket with nothing listening on it,
/// keeps the asking again,
/// because an ask can be carried over the address.
/// Why it is a control and not a comment:
/// with `$XDG_RUNTIME_DIR` set,
/// systemd 262 takes `$XDG_RUNTIME_DIR/systemd/private` alone,
/// so a read that counted the address only where no runtime directory is set would end the asking on a host where a
/// systemd 255 can still reach a manager there
/// (`src/shared/bus-util.c:244` and `:260` at v255).
/// Measured 2026-11-27 by a same-vendor second read of this release,
/// which named that edit and the state it changes:
/// 5 asks in both releases,
/// and 1 under the edit.
#[test]
fn a_bus_address_naming_a_socket_keeps_the_asking_again_where_a_runtime_directory_is_set() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    // The runtime directory this control names: made, and holding nothing at all.
    let runtime_directory: PathBuf = directory.path().join("a-runtime-directory-holding-nothing");
    fs::create_dir_all(&runtime_directory).expect("make the runtime directory");
    // The socket the address names, with nothing listening on it.
    let bus_socket: PathBuf = directory.path().join("a-bus-socket-with-nothing-listening");
    // What:     `a_socket_node(&bus_socket)` plants the socket the address names.
    // Why:      The read judges the transport by the file's kind, and an address naming a socket is one it keeps the asking for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // aSocketNode(busSocket)
    // ```
    a_socket_node(&bus_socket);
    let settings: RouteSettings = RouteSettings {
        systemctl: a_stand_in_that_only_writes_down_each_ask(directory.path(), &log)
            .display()
            .to_string(),
        environment: vec![
            (
                RUNTIME_DIRECTORY_VARIABLE.to_string(),
                Some(runtime_directory.display().to_string()),
            ),
            (
                SESSION_BUS_VARIABLE.to_string(),
                Some(format!("unix:path={}", bus_socket.display())),
            ),
        ],
        ..RouteSettings::default()
    };

    asked_again(&settings, &log);
}

/// The control of a transport the read no longer counts:
/// the user bus's socket beside a bus address,
/// which no systemd met here takes,
/// because sd-bus prefers the address to the bus socket.
/// The address names nothing that is there,
/// so the discovery ends at its first ask.
#[test]
fn a_bus_socket_beside_a_bus_address_is_not_a_transport_the_ask_takes() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    let runtime_directory: PathBuf = directory.path().join("a-runtime-directory");
    fs::create_dir_all(&runtime_directory).expect("make the runtime directory");
    // What:     `a_socket_node(...)` plants the user bus's socket at its own name, beside the address.
    // Why:      The state is the one the read must judge as having no transport: the address replaces this name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // aSocketNode(join(runtimeDirectory, SESSION_BUS_SOCKET))
    // ```
    a_socket_node(&runtime_directory.join(SESSION_BUS_SOCKET));
    let settings: RouteSettings = RouteSettings {
        systemctl: a_stand_in_that_only_writes_down_each_ask(directory.path(), &log)
            .display()
            .to_string(),
        environment: vec![
            (
                RUNTIME_DIRECTORY_VARIABLE.to_string(),
                Some(runtime_directory.display().to_string()),
            ),
            (
                SESSION_BUS_VARIABLE.to_string(),
                Some(format!(
                    "unix:path={}",
                    directory
                        .path()
                        .join("a-socket-that-is-not-there")
                        .display()
                )),
            ),
        ],
        ..RouteSettings::default()
    };

    let refused: String = asked_once_and_not_waited_for(&settings, &log);

    assert!(
        refused.contains("nothing an ask could be carried over is there:"),
        "the refusal does not say what this process read: {refused}"
    );
}

/// The control of the addresses the read cannot judge:
/// an abstract socket,
/// an escaped byte and another transport each name no path this process can look for,
/// so nothing is claimed about them and the asking again is kept.
#[test]
fn an_address_this_process_cannot_judge_keeps_the_asking_again() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    // The three addresses, each of a transport or a form the read cannot look for.
    let unjudged: [&str; 3] = [
        "unix:abstract=/cargo-shadow-a-control-no-manager",
        "unix:path=/a-directory/a%74-socket",
        "unix:tmpdir=/tmp",
    ];
    for (index, address) in unjudged.iter().enumerate() {
        // What:     `log` is that address's own log, so the asks of the address before it are not counted again.
        // Why:      One stand-in writes every ask it is given to this path, and a count across three addresses would be the
        //           sum of theirs.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const log = join(directory, `calls-${index}`)
        // ```
        let log: PathBuf = directory.path().join(format!("calls-{index}"));
        // What:     `RouteSettings { .. }` asks through the stand-in with only that address set.
        // Why:      Each address is a host of its own: the read must keep the asking again for each.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const settings = { systemctl, environment: [[RUNTIME_DIRECTORY_VARIABLE, undefined], [SESSION_BUS_VARIABLE, address]] }
        // ```
        let settings: RouteSettings = RouteSettings {
            systemctl: a_stand_in_that_only_writes_down_each_ask(directory.path(), &log)
                .display()
                .to_string(),
            environment: vec![
                (RUNTIME_DIRECTORY_VARIABLE.to_string(), None),
                (SESSION_BUS_VARIABLE.to_string(), Some(address.to_string())),
            ],
            ..RouteSettings::default()
        };

        asked_again(&settings, &log);
    }
}

/// The control of the read's tolerance for a reexec:
/// a runtime directory that holds the manager's own directory and no socket at this instant,
/// which is what the manager's unlink and rebind looks like between two samples,
/// keeps the asking again.
#[test]
fn the_manager_s_own_directory_keeps_the_asking_again() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    let runtime_directory: PathBuf = directory.path().join("a-runtime-directory");
    // What:     `fs::create_dir_all(runtime_directory.join(MANAGER_FILES_DIRECTORY))` makes the directory the manager keeps its
    //           files in, and nothing else.
    // Why:      A reexec unlinks and rebinds its socket, so the directory is what says a manager was there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // mkdirSync(join(runtimeDirectory, MANAGER_FILES_DIRECTORY), { recursive: true })
    // ```
    fs::create_dir_all(runtime_directory.join(MANAGER_FILES_DIRECTORY))
        .expect("make the manager's directory");
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

    asked_again(&settings, &log);
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

/// The control of what kind of file a bus address names:
/// an address naming a plain file is no transport an ask can be carried over,
/// so the discovery ends at its first ask rather than spending the span of the asking again.
/// Why it is a control and not a comment:
/// no control planted anything but a socket node at an address's path before 2026-11-28,
/// so `is_a_socket` replaced by a check that the path exists left the whole suite green
/// on a host where a file sits at a socket's name
/// (a same-vendor second read of `0.1.5-alpha.18`).
#[test]
fn a_plain_file_at_a_bus_address_is_not_a_transport_the_ask_takes() {
    this_test_needs_a_delegated_scope();
    // The fixture directory of this control.
    let directory = scratch();
    // Where the stand-in writes down each ask.
    let log: PathBuf = directory.path().join("calls");
    // What:     `named` is the path the bus address names: a plain file, which carries no ask.
    // Why:      A read that counted it would keep the asking again on a host where nothing can be reached.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const named = path.join(dir, 'a-bus-address-that-is-a-plain-file')
    // ```
    let named: PathBuf = directory.path().join("a-bus-address-that-is-a-plain-file");
    fs::write(&named, "a plain file where a driver would find a socket").expect("write the plain file");
    // The settings of this control: the stand-in for `systemctl`, no runtime directory, and the address above.
    let settings: RouteSettings = RouteSettings {
        systemctl: a_stand_in_that_only_writes_down_each_ask(directory.path(), &log)
            .display()
            .to_string(),
        environment: vec![
            (RUNTIME_DIRECTORY_VARIABLE.to_string(), None),
            (
                SESSION_BUS_VARIABLE.to_string(),
                Some(format!("unix:path={}", named.display())),
            ),
        ],
        ..RouteSettings::default()
    };

    // The words of the refusal, which a host with no transport must give after one ask.
    let refused: String = asked_once_and_not_waited_for(&settings, &log);

    assert!(
        refused.contains("nothing an ask could be carried over is there:")
            && refused.contains(&named.display().to_string()),
        "the refusal does not say that the plain file at the address's path is no transport: {refused}"
    );
}

/// The control of the other kind of file a socket's name can hold:
/// a directory at a bus address's path is no transport either,
/// so the discovery ends at its first ask.
/// Why it is a control and not a comment:
/// the same read that judged a socket node by its kind is the one that tells a directory from a transport,
/// and until 2026-11-28 no control planted a directory at an address's path
/// (a same-vendor second read of `0.1.5-alpha.18`).
#[test]
fn a_directory_at_a_bus_address_is_not_a_transport_the_ask_takes() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    // What:     `named` is the path the bus address names: a directory, which carries no ask.
    // Why:      A directory's mode is not a socket's, so a read that judges the file's kind counts nothing here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const named = path.join(dir, 'a-bus-address-that-is-a-directory')
    // ```
    let named: PathBuf = directory.path().join("a-bus-address-that-is-a-directory");
    fs::create_dir(&named).expect("make the directory");
    let settings: RouteSettings = RouteSettings {
        systemctl: a_stand_in_that_only_writes_down_each_ask(directory.path(), &log)
            .display()
            .to_string(),
        environment: vec![
            (RUNTIME_DIRECTORY_VARIABLE.to_string(), None),
            (
                SESSION_BUS_VARIABLE.to_string(),
                Some(format!("unix:path={}", named.display())),
            ),
        ],
        ..RouteSettings::default()
    };

    // The words of the refusal, which a host with no transport must give after one ask.
    let refused: String = asked_once_and_not_waited_for(&settings, &log);

    assert!(
        refused.contains("nothing an ask could be carried over is there:")
            && refused.contains(&named.display().to_string()),
        "the refusal does not say that the directory at the address's path is no transport: {refused}"
    );
}
