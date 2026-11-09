//! What:
//!  The fixture of a binary-level control and the resident service process it may start,
//! with the groups of helper processes that die with the test thread.
//! Why:
//!  The fixture and its service are one lifecycle:
//! the fixture owns the service and stops it when it is dropped,
//! so they live together in a file that depends on the limits file and on the readiness checks,
//! which say when a service is at work and when it serves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = fixture(); const service = startFixtureService(fixture);
//! ```

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use crate::support_fixture_service_readiness::{SERVICE_QUIET_WINDOW, SignsOfWork, service_is_serving, wait_until_nothing_listens};
use crate::support_resource_limits::{ServiceLaunch, restrict_to_one_cpu};

/// One disposable directory with a `bin/git` link to the wrapper.
pub struct Fixture {
    /// Canonical fixture root.
    pub root: PathBuf,
    /// The wrapper path every link and hard link in this fixture points at:
    ///  a hard link of the built executable,
    /// so no control may write to it.
    pub wrapper: PathBuf,
    /// The commit service of this fixture,
    /// started by the first wrapped command.
    pub service: std::sync::Mutex<Option<FixtureService>>,
    /// Variables the fixture's service starts with besides its fixed ones:
    /// what the user manager's environment would hold.
    pub service_environment: std::sync::Mutex<Vec<(OsString, OsString)>>,
    /// How the fixture's services are started when a fault control needs more than the plain program.
    pub service_launch: std::sync::Mutex<ServiceLaunch>,
}

/// The process groups the helper processes of one test thread lead,
/// killed when the thread ends however its test ended.
pub struct OwnedGroups {
    /// The group numbers,
    /// which are the numbers of the group leaders.
    groups: std::cell::RefCell<Vec<libc::pid_t>>,
}

/// A panic unwinds the test thread,
/// and a thread that ends runs this.
impl Drop for OwnedGroups {
    fn drop(&mut self) {
        for group in self.groups.borrow().iter() {
            // SAFETY: `killpg` takes a group number and a signal and touches no memory; a group with no member is `ESRCH`.
            unsafe { libc::killpg(*group, libc::SIGKILL) };
        }
    }
}

thread_local! {
    /// The helper groups of the running test thread.
    static OWNED_GROUPS: OwnedGroups = const { OwnedGroups { groups: std::cell::RefCell::new(Vec::new()) } };
}

/// What:
///  Make the process group a child leads die with the test thread that started it,
/// whether the test passes,
/// fails or panics.
/// Why:
///   `Child` does not kill on drop,
/// so a helper that a failing assertion never reached the end of (a service whose idle exit is
///  hours away,
/// a shell loop) kept running after its test.
/// The child must lead a group of its own already (`process_group(0)`
///  before the start),
/// so what it starts is killed with it.
pub fn own_group(child: &Child) {
    let group: libc::pid_t = libc::pid_t::try_from(child.id()).expect("a pid fits the pid type");
    OWNED_GROUPS.with(|owned| { owned.groups.borrow_mut().push(group); });
}

/// What:
///  Start a helper process in a group of its own that dies with the test thread that started it.
/// Why:
///   The same reason as `own_group`,
/// for a child started here.
pub fn spawn_owned(command: &mut Command) -> std::io::Result<Child> {
    use std::os::unix::process::CommandExt as _;
    command.process_group(0);
    let child: Child = command.spawn()?;
    own_group(&child);
    return Ok(child);
}

/// The commit services of one fixture,
/// one per worktree as in production,
/// stopped with the fixture.
pub struct FixtureService {
    /// The running service of each worktree,
    /// by the worktree's own Git directory.
    children: std::sync::Mutex<std::collections::HashMap<PathBuf, Child>>,
    /// The runtime directory the wrapper finds the services' sockets in.
    pub runtime: PathBuf,
}

impl Drop for FixtureService {
    fn drop(&mut self) {
        if let Ok(mut children) = self.children.lock() {
            for child in children.values_mut() {
                kill_with_everything_it_started(child);
            }
        }
        let _removed = std::fs::remove_dir_all(&self.runtime);
    }
}

/// Ends a service and the processes of its group:
/// the tracer in front of it,
/// if any,
/// and the Git it started.
/// A tracer that is
///  killed does not take the traced program with it,
/// so ending only the child would leave a service running.
/// The child is asked
///  to end first,
/// because a tracer that is killed outright loses the part of its log it has not written yet,
/// and a control reads
///  that log;
/// what is still there after a short wait is killed.
pub fn kill_with_everything_it_started(child: &mut Child) {
    let group: i32 = -i32::try_from(child.id()).expect("a process ID fits");
    // SAFETY: `kill` takes two integers and touches no memory of this process.
    unsafe { libc::kill(child.id().try_into().expect("a process ID fits"), libc::SIGTERM) };
    let asked: std::time::Instant = std::time::Instant::now();
    while child.try_wait().is_ok_and(|status| { return status.is_none(); }) && asked.elapsed() < std::time::Duration::from_secs(2) {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    // SAFETY: as above.
    unsafe { libc::kill(group, libc::SIGKILL) };
    let _killed = child.kill();
    let _waited = child.wait();
}

/// The executable Cargo built for this test run,
/// which is the wrapper and,
/// started with the service mode words,
/// the resident
///  service.
pub const SERVICE_PROGRAM: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// What:
///  The command that starts the resident service of this build:
/// the one executable followed by the words of its service mode.
/// Why:
///   The service is a mode of the wrapper's executable (`service_main.rs`),
/// so every control that starts one names these words
///  first and its own options after them.
pub fn service_command() -> Command {
    let mut command: Command = Command::new(SERVICE_PROGRAM);
    command.args(git_policy_cli::service_main::SERVICE_WORDS);
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
/// Each service gets the parent-death signal,
/// so a control that panics
///  or is killed leaves no service behind.
pub fn start_fixture_service(fixture: &Fixture) -> FixtureService {
    let runtime: PathBuf = short_runtime_directory(fixture);
    return FixtureService { children: std::sync::Mutex::new(std::collections::HashMap::new()), runtime };
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
///  so a test depended on how deep the repository sat.
/// The service's own socket already lives in such a directory,
/// and this is the one place that makes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runtime = shortRuntimeDirectory(fixture); // `${HOME}/temp/agent/rt-${pid}-${n}`
/// ```
pub fn short_runtime_directory(fixture: &Fixture) -> PathBuf {
    let _ = fixture;
    let home: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory for the service's runtime directory"));
    let number: usize = RUNTIME_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let runtime: PathBuf = home.join("temp/agent").join(format!("rt-{}-{number}", std::process::id()));
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


impl FixtureService {
    /// What:
    ///  Starts the service of the worktree whose own Git directory is `git_dir`,
    /// listening at the socket the wrapper looks for,
    ///  unless it already runs.
    /// Why:
    ///   In production the manager starts one instance per worktree on its first connection;
    /// the fixture has no manager,
    /// so
    ///  it starts the instance itself and waits until it listens.
    pub fn serve(&self, fixture: &Fixture, git_dir: &Path) {
        use std::os::unix::process::CommandExt;
        let key: PathBuf = std::fs::canonicalize(git_dir).unwrap_or_else(|_| { return git_dir.to_path_buf(); });
        let mut children = self.children.lock().expect("the fixture's services");
        if children.contains_key(&key) {
            return;
        }
        let socket: PathBuf = git_policy_cli::service_instance::socket_for(self.runtime.as_path(), key.as_path());
        // A service ended a moment ago may still hold the socket; the new one would end with its address in use while the poll below
        //  connected to the old one and called the new one ready.
        wait_until_nothing_listens(&socket);
        let launch: ServiceLaunch = fixture.service_launch.lock().expect("the service launch").clone();
        // A helper that an older program started has the earlier command word in its unit file.
        let words: [&str; 3] = if launch.earlier_command_word {
            git_policy_cli::service_main::EARLIER_SERVICE_WORDS
        } else {
            git_policy_cli::service_main::SERVICE_WORDS
        };
        let mut command: Command = match launch.prefix.split_first() {
            Some((program, rest)) => {
                let mut prefixed: Command = Command::new(program);
                prefixed.args(rest).arg(SERVICE_PROGRAM).args(words);
                prefixed
            }
            None => {
                let mut plain: Command = Command::new(SERVICE_PROGRAM);
                plain.args(words);
                plain
            }
        };
        command
            .args(["--journal"])
            .arg(fixture.root.join("journal"))
            .arg("--path")
            .arg(&socket)
            .arg("--instance")
            .arg(git_policy_cli::service_instance::instance_of(key.as_path()))
            .env_clear()
            .env("PATH", format!("{}:/usr/bin:/bin", fixture.root.join("bin").display()))
            .env("HOME", fixture.root.join("home"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            // The identity the person's own configuration gives their Git, which a push that merges (a lost race) needs.
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(crate::support_fixture_service_log::kept_stderr_pipe(fixture.root.as_path()).expect("a pipe for the service's log")));
        for (name, value) in fixture.service_environment.lock().expect("the service environment").iter() {
            command.env(name, value);
        }
        // Its own process group, so that `kill_with_everything_it_started` reaches a traced service and its children.
        command.process_group(0);
        // SAFETY: the closure runs in the child between fork and exec and calls only `prctl`, which is async-signal-safe.
        unsafe {
            command.pre_exec(|| {
                libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
                return Ok(());
            });
        }
        if launch.single_cpu {
            // SAFETY: the closure runs in the child between fork and exec and calls only `sched_getaffinity` and `sched_setaffinity`.
            unsafe {
                command.pre_exec(restrict_to_one_cpu);
            }
        }
        for (kind, soft, hard) in launch.limits {
            // SAFETY: the closure runs in the child between fork and exec and calls only `setrlimit`, which is async-signal-safe.
            unsafe {
                command.pre_exec(move || { return kind.apply(soft, hard); });
            }
        }
        let child: Child = spawn_for_the_process_lifetime(command).expect("the fixture's commit service starts");
        let service_pid: u32 = child.id();
        children.insert(key.clone(), child);
        let mut signs: SignsOfWork = SignsOfWork::of(service_pid);
        let began: std::time::Instant = std::time::Instant::now();
        let mut last_change: std::time::Instant = std::time::Instant::now();
        while !service_is_serving(&socket) {
            // A service that has ended will never listen, so waiting out the patience for it only hides how it ended. A
            //  control whose fault may keep the service from starting (a disk that is full from its first write) goes on
            //  once the service has ended or a short wait has passed: what the client then says is the control's to check.
            let ended: Option<std::process::ExitStatus> = children.get_mut(&key).and_then(|started| { return started.try_wait().ok().flatten(); });
            if launch.may_fail_to_start {
                if ended.is_some() || began.elapsed() > std::time::Duration::from_secs(20) {
                    return;
                }
            } else if let Some(status) = ended {
                children.remove(&key);
                panic!("the fixture's commit service ended ({status}) before it listened at {}; what it wrote to its log is in this test's output", socket.display());
            }
            // A service that is starting on a loaded host is working: it reads its configuration, opens its log and binds
            //  its socket. The fixture ends the wait on the service's own silence, never on the clock, because six
            //  concurrent cases once reached a stated minute while every service was still starting (report
            //  kill-sweeps-reach-the-calls-they-name-1, 2026-11-09).
            if signs.changed(service_pid) {
                last_change = std::time::Instant::now();
            }
            assert!(
                last_change.elapsed() < SERVICE_QUIET_WINDOW,
                "the fixture's commit service at {} showed no sign of work for {:?} while it was starting",
                socket.display(),
                SERVICE_QUIET_WINDOW
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    /// The process IDs of the running services,
    /// for a control that changes a limit in place or signals one.
    pub fn process_ids(&self) -> Vec<u32> {
        let children = self.children.lock().expect("the fixture's services");
        return children.values().map(Child::id).collect();
    }

    /// Ends every running service,
    /// so the next wrapped command starts one by the launch the fixture holds then:
    ///  what the user manager does when a service has gone.
    pub fn stop_all(&self) {
        let mut children = self.children.lock().expect("the fixture's services");
        for (key, child) in children.iter_mut() {
            kill_with_everything_it_started(child);
            // Reaping the child is not the service's end when a tracer is the child: the socket answers until the service is gone.
            wait_until_nothing_listens(git_policy_cli::service_instance::socket_for(self.runtime.as_path(), key.as_path()).as_path());
        }
        children.clear();
    }

    /// What:
    ///  Forgets the services that ended on their own,
    /// so that the next start of their worktree's service runs again.
    /// Why:
    ///   The fixture has no manager,
    /// and a service that is recorded as running is never started again,
    /// ended or not.
    ///
    /// Returns whether any service had ended.
    pub fn forget_ended(&self) -> bool {
        let mut children = self.children.lock().expect("the fixture's services");
        let ended: Vec<PathBuf> =
            children.iter_mut().filter_map(|(key, child)| { return child.try_wait().ok().flatten().map(|_| { return key.clone(); }); }).collect();
        for key in &ended {
            children.remove(key);
        }
        return !ended.is_empty();
    }

    /// Starts the service of every worktree under the fixture root that has none yet:
    /// the repositories and the linked
    ///  worktrees a control made,
    /// found as Git finds them.
    pub fn serve_worktrees(&self, fixture: &Fixture) {
        for git_dir in worktree_git_dirs(fixture.root.as_path(), 4) {
            self.serve(fixture, git_dir.as_path());
        }
    }
}

/// The own Git directories of the worktrees under `directory`,
/// down to `depth` levels:
/// a `.git` directory is its worktree's
///  Git directory,
/// and a `.git` file names a linked worktree's.
/// Bare repositories have no worktree and are not listed.
pub fn worktree_git_dirs(directory: &Path, depth: usize) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let Ok(entries) = std::fs::read_dir(directory) else { return found };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        let name = entry.file_name();
        if name == ".git" {
            if path.is_dir() {
                found.push(path);
            } else if let Ok(text) = std::fs::read_to_string(&path)
                && let Some(target) = text.lines().find_map(|line| { return line.strip_prefix("gitdir: "); })
            {
                found.push(directory.join(target));
            }
        } else if depth > 0 && name != "node_modules" && path.is_dir() && !path.is_symlink() {
            found.extend(worktree_git_dirs(path.as_path(), depth - 1));
        }
    }
    return found;
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

/// Environment variable that keeps every fixture on disk after its test ends,
/// for looking at a failure.
pub const KEEP_FIXTURES_VARIABLE: &str = "CLI_GIT_KEEP_FIXTURES";

/// What:
///  Remove a fixture tree unless the person asked to keep it,
/// trying again for up to five seconds.
/// Why:
///   A landed commit may leave a maintenance child finishing for a moment,
/// and removing a directory that still gains
///  entries fails;
/// a tree already gone is the explicit `remove` having run first,
/// not a failure.
pub fn settle_fixture(root: &Path, keep: bool) {
    if keep {
        return;
    }
    for _ in 0..50 {
        match std::fs::remove_dir_all(root) {
            Ok(()) => return,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
        }
    }
}

/// A fixture goes away however its test ends:
/// a panic unwinds through this too.
impl Drop for Fixture {
    fn drop(&mut self) {
        // The service goes first: it writes into the fixture.
        let _stopped = self.service.lock().map(|mut service| { return service.take(); });
        // Then whatever product code started here and a failed control never ended (a hook that waits with one `sleep`).
        end_processes_working_in(self.root.as_path());
        settle_fixture(self.root.as_path(), std::env::var_os(KEEP_FIXTURES_VARIABLE).is_some());
    }
}

/// The shell condition that holds while the test that started a fixture script is still there.
#[path = "../testing/test_alive.rs"]
mod test_alive;

pub use test_alive::{end_processes_working_in, while_test_lives};
