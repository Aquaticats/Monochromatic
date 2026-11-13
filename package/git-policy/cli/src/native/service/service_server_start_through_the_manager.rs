//! What:
//!  The enhancement: where a service manager of this account is there and is the account's own,
//!  the server is started as a transient scope of it,
//!  so it gets a cgroup of its own and is neither killed with nor charged to the command that
//!  happened to start it.
//! Why:
//!   The base needs no manager
//! (`service_server_start_by_the_wrapper.rs`), so this is added on top of it and never
//!  under it:
//!  every way this file can fail or be absent leaves the base's start exactly as it was,
//!  within a bound, with no message a person has to act on.
//! Bazel's client does the same (`src/main/tools/daemonize.cc`:
//!  `systemd-run --user --scope -- /bin/true` as the probe,
//!  then `systemd-run --user --scope -- /bin/bash <script>`,
//!  and the plain `execv` in every other case),
//!  and its two numbers are adopted:
//!  a probe whose answer decides,
//!  and a start that falls back to the base when it does not answer in time.
//!  The gate is `ServiceHost::named_runtime_dir`:
//!  the manager is asked only when the environment named the runtime directory the account's own bus
//!  belongs to,
//!  because `systemd-run --user` would otherwise ask *another* session's manager for a scope
//!  (issue 6293:
//!  a command given a runtime directory of its own must leave nothing in the account's manager),
//!  and only when a bus address resolves to a socket inside that directory.
//! A manager that is busy re-reading its units answers late or not at all
//!  (systemd defers `Reload()`'s reply until the reload ends,
//!  `src/core/dbus-manager.c` in systemd at `92e51dd8a5`,
//!  and one reload on this machine took 139,770 ms),
//!  which is why nothing here is on a path a commit waits on beyond `PROBE_WAIT`:
//!  a probe that has not answered by then is a manager that is not there for us.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const manager = ManagerStart.inThisSession(host); // undefined: the base starts the server
//! if (manager?.probeAtMost(PROBE_WAIT)) manager.start(server) else spawnDetached(server);
//! ```

use crate::service_host_of_a_worktree::ServiceHost;
use crate::service_server_start_by_the_wrapper::ServerToStart;
use crate::service_server_start_spawns_a_detached_process::{Detached, spawn};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The longest the probe may take before the manager is treated as not there for this command.
/// Why:
///   A manager re-reading its units answers after the reload, which was measured at 139,770 ms on this
///  machine on 2026-11-12;
///   a commit may not pay that, so a probe that outlasts this bound is a manager this command does without.
///  It is also what a person pays at most for the enhancement's absence: two seconds on a first commit in a worktree,
///  and nothing on any later one.
pub const PROBE_WAIT: Duration = Duration::from_secs(2);

/// What:
///  What is done with a probe child the bound ended, on a thread of its own.
/// Why:
///  A probe child that is inside an uninterruptible read or write cannot be ended by a signal until that call returns
///  (a stuck mount, a kernel stall on the filesystem holding the program),
///  and a wait for it on this path would wait for as long as that takes:
/// the review of `3d5c1a0d8c` found `kill()` then `wait()` with no bound after `PROBE_WAIT`.
///  So the ending and the reaping happen here, never on the command's path, which goes on without the probe's answer.
///  A thread that waits for ever is the price of a child that cannot be ended; it holds no lock, no descriptor of the
///  command's, and dies with the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // after the bound: child.kill(); thread(() => child.wait()); return false
/// ```
pub fn end_and_reap_off_the_path(mut probe_child: std::process::Child) {
    let _ended = probe_child.kill();
    let _reaped = probe_child.wait();
}

/// The extra a start through the manager is given past the base's own start wait,
/// after which the scope is ended and the base's start is used instead.
/// Why:
///   One second pays for the extra process the manager runs;
///  a start that needs more than that is a manager or a scope in a state this command does not wait for.
pub const MANAGER_START_EXTRA: Duration = Duration::from_secs(1);

/// The manager's program, as the account's session has it.
pub const PROGRAM: &str = "systemd-run";

/// One service manager this command may ask for a scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagerStart {
    /// The `systemd-run` to run.
    pub program: PathBuf,
}

impl ManagerStart {
    /// What:
    ///  The manager this command may ask,
    ///  when the environment names the runtime directory its bus belongs to and that bus is inside it.
    /// Why:
    ///   `systemd-run --user` finds the manager by the session bus,
    /// so a bus that belongs to another directory is another session's manager:
    /// asking it would put this run's service in the account's manager and its cgroup,
    /// which is what a private runtime directory must never cause (issue 6293).
    /// A folder the wrapper fell back to (`~/.run`) is not a session either:
    /// no manager runs with it, and asking would reach whatever bus the account happens to have.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ManagerStart.inThisSession(host) // undefined when the bus is not ours or systemd-run is not there
    /// ```
    pub fn in_this_session(host: &ServiceHost) -> Option<Self> {
        let named: &Path = host.named_runtime_dir.as_deref()?;
        if named != host.runtime_dir.as_path() {
            return None;
        }
        if !bus_is_ours(named, host.bus_address.as_deref()) {
            return None;
        }
        let program: PathBuf = find_on_path(PROGRAM)?;
        return Some(Self { program });
    }

    /// What:
    ///  Whether the manager answers at all within `PROBE_WAIT`.
    /// Why:
    ///   This is Bazel's probe (`systemd-run --user --scope -- /bin/true`):
    /// the manager's own program decides whether a scope can be made,
    /// and a refusal, a missing manager, a bus that is not there or a busy manager all answer the same way, in time.
    /// The probe is not detached, unlike a start:
    /// it is a child of this command for the length of one `true`,
    /// so its own exit status is what answers, and a probe still running at the bound is ended and reaped by number.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// waitAtMost(PROBE_WAIT, () => manager.probe())
    /// ```
    pub fn probe(&self) -> bool {
        return self.probe_with(Box::new(end_and_reap_off_the_path));
    }

    /// What:
    ///  The probe, with how a child the bound ended is dealt with given by the caller.
    /// Why:
    ///  The control of this file hands in a way that never returns, which is the state a probe child in an uninterruptible
    /// call is in, and asserts that the probe still answers within its bound; a wait on this path would fail it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// probeWith(endAndReap) // endAndReap runs on a thread of its own
    /// ```
    fn probe_with(&self, end_and_reap: Box<dyn FnOnce(std::process::Child) + Send>) -> bool {
        let mut command: std::process::Command = std::process::Command::new(&self.program);
        command.args(["--user", "--scope", "--quiet", "--", "/bin/true"]).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
        let Ok(mut child) = command.spawn() else { return false };
        let deadline: std::time::Instant = std::time::Instant::now() + PROBE_WAIT;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return status.success(),
                Ok(None) if std::time::Instant::now() >= deadline => {
                    // What: The child is ended and reaped on a thread of its own, and this command goes on without its answer.
                    // Why:  A signal cannot end a child inside an uninterruptible call, and waiting for such a child here was
                    //       the review's finding 8: a commit could wait for ever inside a probe.
                    std::thread::spawn(move || {
                        end_and_reap(child);
                    });
                    return false;
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                Err(_unreadable) => return false,
            }
        }
    }

    /// What:
    ///  Starts the server as a transient scope of the manager,
    ///  and gives the scope leader's process number.
    /// Why:
    ///   `systemd-run --user --scope -- <program> <words>` runs the program itself as its child in the caller's
    ///  environment and returns when the program ends (`systemd-run(1)`),
    /// so the wrapper's own detaching (a session of its own, `/dev/null`, one log) applies to `systemd-run`,
    /// and the server is the leader's child in the scope the manager made.
    /// Nothing of the manager's own bookkeeping is named: no unit file is written and no reload is asked for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// spawnDetached({ program: 'systemd-run', arguments: ['--user', '--scope', '--', server.program, ...server.arguments()], log: server.log() })
    /// ```
    pub fn start(&self, server: &ServerToStart) -> std::io::Result<Option<i32>> {
        let mut arguments: Vec<std::ffi::OsString> = vec![
            crate::service_server_start_spawns_a_detached_process::text_argument("--user"),
            crate::service_server_start_spawns_a_detached_process::text_argument("--scope"),
            crate::service_server_start_spawns_a_detached_process::text_argument("--quiet"),
            crate::service_server_start_spawns_a_detached_process::text_argument("--"),
            crate::service_server_start_spawns_a_detached_process::argument(server.program.as_path()),
        ];
        arguments.extend(server.arguments());
        let detached: Detached = Detached { program: self.program.clone(), arguments, log: server.log() };
        return spawn(&detached);
    }
}

/// What:
///  Whether the session bus this command inherited is the one inside `runtime_dir`.
/// Why:
///   The address is the fact,
/// and its socket's directory is the runtime directory its session uses:
/// `/run/user/1000/bus` is the bus of the session whose runtime directory is `/run/user/1000`.
/// An abstract address has no directory, so it cannot be proven ours;
/// a list of addresses is read in order and the first that names a path decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// busAddresses(env.DBUS_SESSION_BUS_ADDRESS).some((a) => path.dirname(a.path) === runtimeDir)
/// ```
pub fn bus_is_ours(runtime_dir: &Path, address: Option<&str>) -> bool {
    let Some(spelled) = address else { return false };
    for one in spelled.split(';') {
        let Some(path) = one.strip_prefix("unix:path=") else { continue };
        let socket: &Path = Path::new(path);
        let Some(directory) = socket.parent() else { continue };
        return directory == runtime_dir;
    }
    return false;
}

/// Whether a program of this name is on `PATH`,
/// as the process running this command has it.
pub fn find_on_path(name: &str) -> Option<PathBuf> {
    let path: std::ffi::OsString = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path) {
        let candidate: PathBuf = directory.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    return None;
}

/// Ends a leader and everything in its group, which is the scope's own group.
pub fn end_the_group_of(pid: i32) {
    // SAFETY: a negative number names the process group the leader was made the head of (`setsid` in the spawn).
    let _signalled: libc::c_int = unsafe { libc::kill(-pid, libc::SIGKILL) };
}

/// Controls of the enhancement stay out of the release executable.
#[cfg(test)]
#[path = "service_server_start_through_the_manager_tests.rs"]
mod tests;
