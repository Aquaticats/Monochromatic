//! What:
//!  The executable a fixture's resident service is started from,
//!  its words and its runtime directory,
//!  the thread that starts it so the process's parent-death signal is the one meant,
//!  and the restart the user manager would make when a service has ended.
//! Why:
//!  A service is started for a worktree and restarted at the next connection in production;
//!  a fixture has no manager,
//!  so the words, the address form and the restart are the fixture's own to reproduce,
//!  and they describe how a service is started rather than the fixture's lifecycle,
//!  which keeps the lifecycle file inside the repository's per-file line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const service = spawn(SERVICE_PROGRAM, [...SERVICE_WORDS, '--journal', journal, '--path', socket]);
//! ```

use std::path::PathBuf;
use std::process::{Child, Command};
use crate::support_fixture_service::{Fixture, FixtureService, start_fixture_service_in};
use crate::support_resource_limits::ServiceLaunch;

/// The executable Cargo built for this test run,
/// which is the wrapper and,
/// started with the service mode words,
/// the resident
///  service.
pub const SERVICE_PROGRAM: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// What:
///  The command that starts the resident service of this build:
/// the one executable followed by the words of its service mode,
/// with no runtime directory and no session bus inherited from the process that runs the tests.
/// Why:
///   The service is a mode of the wrapper's executable (`service_main.rs`),
/// so every control that starts one names these words
///  first and its own options after them.
/// A control must never reach the owner's real user manager:
/// a service that inherited a developer's `XDG_RUNTIME_DIR` and `DBUS_SESSION_BUS_ADDRESS` could make that manager start,
/// or stop,
///  a unit for a throwaway worktree,
/// and hundreds of units of deleted throwaway repositories were measured there on 2026-11-10
/// (`doc/troubleshooting/git-policy-cli-service-units-of-deleted-worktrees.md`).
/// A control that needs a bus or a runtime directory gives its own after this call,
/// so nothing a control starts can find one by inheritance.
pub fn service_command() -> Command {
    let mut command: Command = Command::new(SERVICE_PROGRAM);
    command
        .args(git_policy_cli::service_main::SERVICE_WORDS)
        .env_remove("XDG_RUNTIME_DIR")
        .env_remove("DBUS_SESSION_BUS_ADDRESS");
    return command;
}

/// The path of the executable that runs as the service when started with the service mode words.
pub const fn service_executable() -> &'static str {
    return SERVICE_PROGRAM;
}

/// Numbers the runtime directories of one test process.
pub static RUNTIME_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// What:
///  Makes the runtime directory a fixture's services listen in;
/// no service runs until a worktree asks for one.
/// Why:
///  A socket path holds about a hundred bytes,
/// which the fixture tree's depth does not allow,
/// so the runtime directory is
///  in the short agent scratch directory under the home.
/// A control of the address a long runtime directory takes gives its own instead
///  (`start_fixture_service_in`).
/// Each service gets the parent-death signal,
/// so a control that panics
///  or is killed leaves no service behind.
pub fn start_fixture_service(fixture: &Fixture) -> FixtureService {
    return start_fixture_service_in(short_runtime_directory(fixture));
}

/// What:
///  A directory of this fixture's own,
///  short enough to hold a socket name.
/// Why:
///   A Unix socket's name must fit `SUN_LEN`,
/// which is 108 bytes,
/// and a fixture root under a deep checkout does not.
/// A control that bound its socket beside the fixture failed with "path must be shorter than SUN_LEN"
///  on a long checkout and passed on a short one (2026-11-08),
///  so a test depended on how deep the repository sat;
/// and it then depended on the length of the home directory as well (2026-11-15),
///  so the folder now comes from `socket_folder`,
/// which asks every place this account offers and names them when none
///  is short enough.
/// The service's own socket already lives in such a directory,
/// and this is the one place that makes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runtime = shortRuntimeDirectory(fixture); // `${HOME}/temp/agent/rt3f9a1c/rt-0`
/// ```
pub fn short_runtime_directory(fixture: &Fixture) -> PathBuf {
    let number: usize = RUNTIME_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let name: String = format!("rt-{number}");
    // The runtime directory is one socket path less its name and the socket's own suffix, and every instance of this
    //  package is `INSTANCE_LENGTH` hexadecimal digits, so the suffix is the same whatever the worktree is.
    let instance: String = "0".repeat(git_policy_cli::service_instance::INSTANCE_LENGTH);
    let longest_folder: usize = git_policy_cli::service_instance::longest_runtime_directory(instance.as_str()).saturating_sub(1 + name.len());
    let folder: PathBuf = fixture.socket_folders.lock().expect("the fixture's socket folders").folder("rt", longest_folder);
    let runtime: PathBuf = folder.join(name.as_str());
    std::fs::create_dir_all(runtime.join("monochromatic-git-policy-cli")).expect("runtime directory");
    return runtime;
}

/// What:
///  Starts `command` from a thread that lives as long as this test process,
/// and returns the child.
/// Why:
///   `PR_SET_PDEATHSIG` is sent when the thread that spawned the child ends,
/// not when the process does.
/// Controls start their
///  services from whichever of their threads gets there first (two worktrees committing at once),
/// so a child spawned there
///  died when that thread's closure returned,
/// and the other thread's commit found its work claimed by a dead service.
/// This
///  thread ends only with the process,
/// which is the moment the signal is meant for.
pub fn spawn_for_the_process_lifetime(command: Command) -> std::io::Result<Child> {
    type Job = (Command, std::sync::mpsc::Sender<std::io::Result<Child>>);
    static SPAWNER: std::sync::OnceLock<std::sync::Mutex<std::sync::mpsc::Sender<Job>>> = std::sync::OnceLock::new();
    let spawner = SPAWNER.get_or_init(|| {
        let (sender, receiver) = std::sync::mpsc::channel::<Job>();
        std::thread::spawn(move || {
            for (mut job, reply) in receiver {
                let _sent = reply.send(job.spawn());
            }
        });
        return std::sync::Mutex::new(sender);
    });
    let (reply, answer) = std::sync::mpsc::channel();
    spawner.lock().expect("the spawner's sender").send((command, reply)).expect("the spawner thread runs");
    return answer.recv().expect("the spawner thread answers");
}

/// What:
///  Does what the user manager does when a service has ended:
/// starts the worktree's service again,
/// as production runs it,
///  with no tracer and no fault.
/// Why:
///   The manager owns the socket and starts the service at the next connection.
/// A control that kills the service during a
///  commit relies on that start for the client's resend,
/// so it calls this while the commit runs.
pub fn restart_ended_services(fixture: &Fixture) {
    let service = fixture.service.lock().expect("the fixture's service lock");
    let Some(services) = service.as_ref() else { return };
    if services.forget_ended() {
        *fixture.service_launch.lock().expect("the service launch") = ServiceLaunch::default();
        services.serve_worktrees(fixture);
    }
}

/// The runtime directory of the fixture's services,
/// with a service running for every worktree of the fixture,
/// so a wrapped
///  commit takes the route a person's commit takes:
/// the commit path has no other writer.
pub fn service_runtime(fixture: &Fixture) -> Option<PathBuf> {
    let mut service = fixture.service.lock().expect("the fixture's service lock");
    if service.is_none() {
        *service = Some(start_fixture_service(fixture));
    }
    let started = service.as_ref()?;
    started.serve_worktrees(fixture);
    return Some(started.runtime.clone());
}
