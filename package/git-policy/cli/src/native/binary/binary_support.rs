//! What:
//!  Disposable fixtures and bounded process helpers for binary-level controls.
//! Why:
//!  Every control runs the built executable through a `git`-named link on a
//!      fixture PATH,
//!  with no real home or global Git configuration,
//!  and a hard time
//!      bound so a forwarding loop fails the test instead of hanging it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const fixture = await mkdtemp(...); await symlink(wrapper, join(fixture, 'bin/git'));
//! ```

/// Native string,
///  path and process types used by the helpers.
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

/// The native executable Cargo built for this test run.
pub const WRAPPER: &str = env!("CARGO_BIN_EXE_cli-git-native");

/// The real Git 2.56.0 executable installed in the verification image.
pub const REAL_GIT: &str = "/usr/bin/git";

/// The wall-clock backstop, in seconds, of the shared bound: a hang guard of the test harness, not a requirement on the
///  wrapper, which has no time limit on a wrapped command;
///  the one latency requirement is that a commit made through the wrapper takes less than 5 seconds longer than the
///  same commit made with native Git.
/// Real commands here finish in milliseconds. The bound that fails a hung command quickly is not this one but the
///  supervisor's (`binary_supervisor.rs`): it ends a command whose process tree shows no progress for five seconds or has
///  used ten seconds of processor time, and it does not count time spent waiting for the disk or for the processor, which
///  other sessions on a shared host stretch without limit. A mutant that loops or deadlocks still fails every affected
///  control within seconds.
pub const TIME_BOUND_SECONDS: &str = super::supervisor::SHARED_BACKSTOP_SECONDS;

/// The signal that ends `timeout --signal=KILL` itself when the bound was hit:
///  it kills
/// the command,
///  then re-raises the same signal on itself.
pub const TIMED_OUT_SIGNAL: i32 = 9;

/// What one finished process showed its caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observed {
    /// Exit code,
    ///  absent when a signal ended the process.
    pub code: Option<i32>,
    /// Exact standard output bytes.
    pub stdout: Vec<u8>,
    /// Exact standard error bytes.
    pub stderr: Vec<u8>,
}

/// One disposable directory with a `bin/git` link to the wrapper.
pub struct Fixture {
    /// Canonical fixture root.
    pub root: PathBuf,
    /// The wrapper path every link and hard link in this fixture points at:
    ///  a hard link of the built executable, so no control may write to it.
    pub wrapper: PathBuf,
    /// The commit service of this fixture, started by the first wrapped command.
    pub service: std::sync::Mutex<Option<FixtureService>>,
    /// Variables the fixture's service starts with besides its fixed ones: what the user manager's environment would hold.
    pub service_environment: std::sync::Mutex<Vec<(OsString, OsString)>>,
    /// How the fixture's services are started when a fault control needs more than the plain program.
    pub service_launch: std::sync::Mutex<ServiceLaunch>,
}

/// A resource limit a service starts with, set between `fork` and `exec` with no privilege.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LimitKind {
    /// Open descriptors (`RLIMIT_NOFILE`).
    OpenFiles,
    /// Processes of the real user (`RLIMIT_NPROC`).
    Processes,
    /// Address space (`RLIMIT_AS`).
    AddressSpace,
    /// Size of a core file (`RLIMIT_CORE`); zero writes none.
    CoreFile,
}

impl LimitKind {
    /// Sets this limit on the calling process; it is async-signal-safe, so a `pre_exec` closure may call it.
    pub fn apply(self, soft: u64, hard: u64) -> std::io::Result<()> {
        let cap: libc::rlimit = libc::rlimit { rlim_cur: soft, rlim_max: hard };
        let resource = match self {
            LimitKind::OpenFiles => libc::RLIMIT_NOFILE,
            LimitKind::Processes => libc::RLIMIT_NPROC,
            LimitKind::AddressSpace => libc::RLIMIT_AS,
            LimitKind::CoreFile => libc::RLIMIT_CORE,
        };
        // SAFETY: `setrlimit` reads the one `rlimit` this function owns.
        if unsafe { libc::setrlimit(resource, &raw const cap) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        return Ok(());
    }

    /// The hard limit of this process, which a child inherits and which an unprivileged process can lower but never raise.
    pub fn current_hard(self) -> u64 {
        let mut held: libc::rlimit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        let resource = match self {
            LimitKind::OpenFiles => libc::RLIMIT_NOFILE,
            LimitKind::Processes => libc::RLIMIT_NPROC,
            LimitKind::AddressSpace => libc::RLIMIT_AS,
            LimitKind::CoreFile => libc::RLIMIT_CORE,
        };
        // SAFETY: `getrlimit` writes the one `rlimit` this function owns.
        let read: libc::c_int = unsafe { libc::getrlimit(resource, &raw mut held) };
        assert_eq!(read, 0, "getrlimit failed: {}", std::io::Error::last_os_error());
        return held.rlim_max;
    }

    /// Changes the soft limit of the running process `pid` of the same user, as the end of the pressure that caused it
    ///  would: the hard limit stays, so raising the soft limit up to it needs no privilege.
    #[cfg(target_os = "linux")]
    pub fn lift_for(self, pid: u32, soft: u64, hard: u64) -> std::io::Result<()> {
        let cap: libc::rlimit = libc::rlimit { rlim_cur: soft, rlim_max: hard };
        let resource = match self {
            LimitKind::OpenFiles => libc::RLIMIT_NOFILE,
            LimitKind::Processes => libc::RLIMIT_NPROC,
            LimitKind::AddressSpace => libc::RLIMIT_AS,
            LimitKind::CoreFile => libc::RLIMIT_CORE,
        };
        // SAFETY: `prlimit` reads the one `rlimit` this function owns and is given no old-value pointer.
        if unsafe { libc::prlimit(i32::try_from(pid).expect("a process ID fits"), resource, &raw const cap, std::ptr::null_mut()) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        return Ok(());
    }
}

/// How a fixture starts its commit services: a program in front of the service (a tracer), and limits on the service and
///  everything it starts. The default is the plain service of production.
#[derive(Clone, Debug, Default)]
pub struct ServiceLaunch {
    /// The program and arguments placed in front of the service executable; empty starts the service itself.
    pub prefix: Vec<OsString>,
    /// Soft and hard limits applied to the service before it runs.
    pub limits: Vec<(LimitKind, u64, u64)>,
    /// Whether the service and everything it starts may run on one processor only.
    pub single_cpu: bool,
    /// Whether a service that ends or never listens is the control's subject, not a failure of the fixture.
    pub may_fail_to_start: bool,
}

/// Restricts the calling process, and so everything it starts, to the one processor it may use first.
///  It is async-signal-safe, so a `pre_exec` closure may call it.
#[cfg(target_os = "linux")]
pub fn restrict_to_one_cpu() -> std::io::Result<()> {
    // SAFETY: `cpu_set_t` is plain data, and a zeroed set is the empty one.
    let mut allowed: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    // SAFETY: `sched_getaffinity` writes the set it is given, whose size is passed.
    if unsafe { libc::sched_getaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &raw mut allowed) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let first: usize = (0..libc::CPU_SETSIZE as usize)
        .find(|cpu| { return unsafe { libc::CPU_ISSET(*cpu, &allowed) }; })
        .ok_or_else(|| { return std::io::Error::other("no allowed processor"); })?;
    // SAFETY: as above.
    let mut one: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    // SAFETY: `first` is below `CPU_SETSIZE`, so the set holds the bit.
    unsafe { libc::CPU_SET(first, &mut one) };
    // SAFETY: `sched_setaffinity` reads the set it is given, whose size is passed.
    if unsafe { libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &raw const one) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// The process groups the helper processes of one test thread lead, killed when the thread ends however its test ended.
struct OwnedGroups {
    /// The group numbers, which are the numbers of the group leaders.
    groups: std::cell::RefCell<Vec<libc::pid_t>>,
}

/// A panic unwinds the test thread, and a thread that ends runs this.
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
///  Make the process group a child leads die with the test thread that started it, whether the test passes, fails or panics.
/// Why:
///   `Child` does not kill on drop, so a helper that a failing assertion never reached the end of (a service whose idle exit is
///  hours away, a shell loop) kept running after its test. The child must lead a group of its own already (`process_group(0)`
///  before the start), so what it starts is killed with it.
pub fn own_group(child: &Child) {
    let group: libc::pid_t = libc::pid_t::try_from(child.id()).expect("a pid fits the pid type");
    OWNED_GROUPS.with(|owned| { owned.groups.borrow_mut().push(group); });
}

/// What:
///  Start a helper process in a group of its own that dies with the test thread that started it.
/// Why:
///   The same reason as `own_group`, for a child started here.
pub fn spawn_owned(command: &mut Command) -> std::io::Result<Child> {
    use std::os::unix::process::CommandExt as _;
    command.process_group(0);
    let child: Child = command.spawn()?;
    own_group(&child);
    return Ok(child);
}

/// The commit services of one fixture, one per worktree as in production, stopped with the fixture.
pub struct FixtureService {
    /// The running service of each worktree, by the worktree's own Git directory.
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

/// Ends a service and the processes of its group: the tracer in front of it, if any, and the Git it started. A tracer that is
///  killed does not take the traced program with it, so ending only the child would leave a service running. The child is asked
///  to end first, because a tracer that is killed outright loses the part of its log it has not written yet, and a control reads
///  that log; what is still there after a short wait is killed.
fn kill_with_everything_it_started(child: &mut Child) {
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

/// The executable Cargo built for this test run, which is the wrapper and, started with the service mode words, the resident
///  service.
const SERVICE_PROGRAM: &str = env!("CARGO_BIN_EXE_cli-git-native");

/// What:
///  The command that starts the resident service of this build: the one executable followed by the words of its service mode.
/// Why:
///   The service is a mode of the wrapper's executable (`service_main.rs`), so every control that starts one names these words
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
static RUNTIME_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// What:
///  Makes the runtime directory a fixture's services listen in; no service runs until a worktree asks for one.
/// Why:
///  A socket path holds about a hundred bytes, which the fixture tree's depth does not allow, so the runtime directory is
///  in the short agent scratch directory under the home. Each service gets the parent-death signal, so a control that panics
///  or is killed leaves no service behind.
pub fn start_fixture_service(fixture: &Fixture) -> FixtureService {
    let _ = fixture;
    let home: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory for the service's runtime directory"));
    let number: usize = RUNTIME_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let runtime: PathBuf = home.join("temp/agent").join(format!("rt-{}-{number}", std::process::id()));
    std::fs::create_dir_all(runtime.join("cli-git")).expect("runtime directory");
    return FixtureService { children: std::sync::Mutex::new(std::collections::HashMap::new()), runtime };
}

/// What:
///  Starts `command` from a thread that lives as long as this test process, and returns the child.
/// Why:
///   `PR_SET_PDEATHSIG` is sent when the thread that spawned the child ends, not when the process does. Controls start their
///  services from whichever of their threads gets there first (two worktrees committing at once), so a child spawned there
///  died when that thread's closure returned, and the other thread's commit found its work claimed by a dead service. This
///  thread ends only with the process, which is the moment the signal is meant for.
fn spawn_for_the_process_lifetime(command: Command) -> std::io::Result<Child> {
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

/// How long a fixture waits for an ended service's socket to stop answering before it says the service is still there.
const SOCKET_RELEASE_PATIENCE: std::time::Duration = std::time::Duration::from_secs(60);

/// What:
///  Waits until nothing answers at `socket`: the file is absent, or it is a stale socket whose listener is gone.
/// Why:
///   Ending a service that runs under a tracer reaps the tracer, and the traced service dies a moment later, so its socket can
///  still answer after the fixture counts the service as ended. A new service cannot take such a socket, and a readiness poll
///  that connects to it reports a service that is not the new one.
///
/// # Panics
/// When something still answers after `SOCKET_RELEASE_PATIENCE`, which is a service the fixture did not end.
fn wait_until_nothing_listens(socket: &Path) {
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        match std::os::unix::net::UnixStream::connect(socket) {
            Err(error) if matches!(error.kind(), std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound) => return,
            Ok(_) | Err(_) => {}
        }
        assert!(began.elapsed() < SOCKET_RELEASE_PATIENCE, "something still answers at {} although the fixture ended its service", socket.display());
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

impl FixtureService {
    /// What:
    ///  Starts the service of the worktree whose own Git directory is `git_dir`, listening at the socket the wrapper looks for,
    ///  unless it already runs.
    /// Why:
    ///   In production the manager starts one instance per worktree on its first connection; the fixture has no manager, so
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
        let mut command: Command = match launch.prefix.split_first() {
            Some((program, rest)) => {
                let mut prefixed: Command = Command::new(program);
                prefixed.args(rest).arg(SERVICE_PROGRAM).args(git_policy_cli::service_main::SERVICE_WORDS);
                prefixed
            }
            None => service_command(),
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
            .stderr(Stdio::inherit());
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
        children.insert(key.clone(), child);
        let began: std::time::Instant = std::time::Instant::now();
        while std::os::unix::net::UnixStream::connect(&socket).is_err() {
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
            assert!(began.elapsed() < std::time::Duration::from_secs(60), "the fixture's commit service never listened");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    /// The process IDs of the running services, for a control that changes a limit in place or signals one.
    pub fn process_ids(&self) -> Vec<u32> {
        let children = self.children.lock().expect("the fixture's services");
        return children.values().map(Child::id).collect();
    }

    /// Ends every running service, so the next wrapped command starts one by the launch the fixture holds then:
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
    ///  Forgets the services that ended on their own, so that the next start of their worktree's service runs again.
    /// Why:
    ///   The fixture has no manager, and a service that is recorded as running is never started again, ended or not.
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

    /// Starts the service of every worktree under the fixture root that has none yet: the repositories and the linked
    ///  worktrees a control made, found as Git finds them.
    pub fn serve_worktrees(&self, fixture: &Fixture) {
        for git_dir in worktree_git_dirs(fixture.root.as_path(), 4) {
            self.serve(fixture, git_dir.as_path());
        }
    }
}

/// The own Git directories of the worktrees under `directory`, down to `depth` levels: a `.git` directory is its worktree's
///  Git directory, and a `.git` file names a linked worktree's. Bare repositories have no worktree and are not listed.
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
///  Does what the user manager does when a service has ended: starts the worktree's service again, as production runs it,
///  with no tracer and no fault.
/// Why:
///   The manager owns the socket and starts the service at the next connection. A control that kills the service during a
///  commit relies on that start for the client's resend, so it calls this while the commit runs.
pub fn restart_ended_services(fixture: &Fixture) {
    let service = fixture.service.lock().expect("the fixture's service lock");
    let Some(services) = service.as_ref() else { return };
    if services.forget_ended() {
        *fixture.service_launch.lock().expect("the service launch") = ServiceLaunch::default();
        services.serve_worktrees(fixture);
    }
}

/// The runtime directory of the fixture's services, with a service running for every worktree of the fixture, so a wrapped
///  commit takes the route a person's commit takes: the commit path has no other writer.
pub fn service_runtime(fixture: &Fixture) -> Option<PathBuf> {
    let mut service = fixture.service.lock().expect("the fixture's service lock");
    if service.is_none() {
        *service = Some(start_fixture_service(fixture));
    }
    let started = service.as_ref()?;
    started.serve_worktrees(fixture);
    return Some(started.runtime.clone());
}

/// A pipe whose two ends are closed on `exec`: `[read end, write end]`.
/// Linux makes both ends close-on-exec in the one `pipe2` call;
///  macOS has no `pipe2`, so there the flag is set on each end right after `pipe`.
#[cfg(target_os = "linux")]
pub fn close_on_exec_pipe() -> [libc::c_int; 2] {
    let mut ends: [libc::c_int; 2] = [0, 0];
    // SAFETY: `pipe2` fills the two integers it is given.
    let made: libc::c_int = unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) };
    assert_eq!(made, 0, "pipe2 failed: {}", std::io::Error::last_os_error());
    return ends;
}

/// A pipe whose two ends are closed on `exec`: `[read end, write end]`.
/// Linux makes both ends close-on-exec in the one `pipe2` call;
///  macOS has no `pipe2`, so there the flag is set on each end right after `pipe`.
#[cfg(all(unix, not(target_os = "linux")))]
pub fn close_on_exec_pipe() -> [libc::c_int; 2] {
    let mut ends: [libc::c_int; 2] = [0, 0];
    // SAFETY: `pipe` fills the two integers it is given.
    let made: libc::c_int = unsafe { libc::pipe(ends.as_mut_ptr()) };
    assert_eq!(made, 0, "pipe failed: {}", std::io::Error::last_os_error());
    for end in ends {
        // SAFETY: `end` was just created here; `fcntl` with `F_SETFD` only changes that descriptor's flags.
        let flagged: libc::c_int = unsafe { libc::fcntl(end, libc::F_SETFD, libc::FD_CLOEXEC) };
        assert_eq!(flagged, 0, "fcntl failed: {}", std::io::Error::last_os_error());
    }
    return ends;
}

/// The path of the slave side of the pseudo-terminal whose master is `descriptor`.
/// Linux has `ptsname_r`;
///  macOS has none, and `TIOCPTYGNAME` returns the name into the caller's buffer just as safely.
#[cfg(target_os = "linux")]
pub fn pseudo_terminal_slave_name(descriptor: libc::c_int) -> std::ffi::CString {
    let mut buffer: [libc::c_char; 256] = [0; 256];
    // SAFETY: `ptsname_r` writes at most the buffer's length.
    let named: libc::c_int = unsafe { libc::ptsname_r(descriptor, buffer.as_mut_ptr(), buffer.len()) };
    assert_eq!(named, 0, "the slave has a name");
    // SAFETY: the call above wrote a NUL-terminated name into `buffer`.
    return unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }.to_owned();
}

/// The path of the slave side of the pseudo-terminal whose master is `descriptor`.
/// Linux has `ptsname_r`;
///  macOS has none, and `TIOCPTYGNAME` returns the name into the caller's buffer just as safely.
#[cfg(all(unix, not(target_os = "linux")))]
pub fn pseudo_terminal_slave_name(descriptor: libc::c_int) -> std::ffi::CString {
    let mut buffer: [libc::c_char; 128] = [0; 128];
    // SAFETY: `TIOCPTYGNAME` writes a NUL-terminated name of at most 128 bytes into the buffer it is given.
    let named: libc::c_int = unsafe { libc::ioctl(descriptor, libc::TIOCPTYGNAME as libc::c_ulong, buffer.as_mut_ptr()) };
    assert_eq!(named, 0, "the slave has a name");
    // SAFETY: the call above wrote a NUL-terminated name into `buffer`.
    return unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }.to_owned();
}

/// Environment variable that keeps every fixture on disk after its test ends, for looking at a failure.
pub const KEEP_FIXTURES_VARIABLE: &str = "CLI_GIT_KEEP_FIXTURES";

/// Environment variable that moves the fixture root, for example onto a tmpfs for speed.
pub const FIXTURE_ROOT_VARIABLE: &str = "CLI_GIT_TEST_ROOT";

/// The directory fixtures are created in.
///  Cargo's per-target scratch directory, never `/tmp`: a shared tmpfs with a per-user quota fills up
///  when failing tests leave fixtures behind (issue 1809), and then every command of every session fails.
pub fn fixture_root() -> PathBuf {
    let root: PathBuf = std::env::var_os(FIXTURE_ROOT_VARIABLE)
        .map_or_else(|| { return PathBuf::from(env!("CARGO_TARGET_TMPDIR")); }, PathBuf::from);
    std::fs::create_dir_all(&root).expect("fixture root");
    return root;
}

/// What:
///  Remove a fixture tree unless the person asked to keep it, trying again for up to five seconds.
/// Why:
///   A landed commit may leave a maintenance child finishing for a moment, and removing a directory that still gains
///  entries fails; a tree already gone is the explicit `remove` having run first, not a failure.
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

/// A fixture goes away however its test ends: a panic unwinds through this too.
impl Drop for Fixture {
    fn drop(&mut self) {
        // The service goes first: it writes into the fixture.
        let _stopped = self.service.lock().map(|mut service| { return service.take(); });
        // Then whatever product code started here and a failed control never ended (a hook that waits with one `sleep`).
        end_processes_working_in(self.root.as_path());
        settle_fixture(self.root.as_path(), std::env::var_os(KEEP_FIXTURES_VARIABLE).is_some());
    }
}

/// What:
///  Read the fixture's wrapper once per test process, so its pages are in the page cache before any bounded command
///  starts it, and the wait for a slow disk happens here and not under the 5-second test hang guard.
/// Why:
///  Every fixture's wrapper is a hard link of the built executable, so all fixtures of all test processes share one
///  set of cached pages. Starting an executable that is not cached reads most of the file in its first fault (a 22 MB
///  debug build read 16 MiB; `doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`).
fn warm_wrapper(linked: &Path) {
    /// Set once the executable has been read by this process.
    static WARMED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    WARMED.get_or_init(|| {
        use std::io::Read;
        let mut file: std::fs::File = std::fs::File::open(linked).expect("open the fixture wrapper");
        let mut chunk: Vec<u8> = vec![0; 1 << 20];
        while file.read(chunk.as_mut_slice()).expect("read the built wrapper") > 0 {}
    });
}

/// What:
///  Give a fixture its wrapper path: a hard link of the built executable; when the link fails because the fixture is on
///  another filesystem (`EXDEV`), a hard link of the one stored copy of the build in `store_root`; and a data copy
///  only where neither link works.
/// Why:
///  The wrapper never reads its own path except to recognise itself by file identity and to find hook entries by
///  the name it was started under (`real_git_candidate::same_file`, `main.rs`), and no control writes to it, so a
///  private copy bought nothing but 22 MB of writes per fixture and a cold first start of a new inode per test.
///  The fallback follows the link's own failure and not a comparison of device numbers, which bind mounts defeat.
///  169 fixtures that each copied wrote about 3.7 GB (169 times 21.9 MB) in one wave
///  (`doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`).
fn link_built_wrapper(built: &Path, to: &Path, store_root: &Path) {
    if let Err(error) = std::fs::hard_link(built, to) {
        let stored: std::io::Result<()> = if error.raw_os_error() == Some(libc::EXDEV) {
            super::wrapper_store::link_stored(built, store_root, to, &mut |_| {})
        } else {
            Err(error)
        };
        if let Err(reason) = stored {
            eprintln!("fixture wrapper: no hard link of {} at {} ({reason}); copying it instead", built.display(), to.display());
            copy_executable(built, to);
        }
    }
    // After the link, not before: on overlay storage a hard link of an image-layer file copies the file up to a
    // new inode, whose pages are the ones every fixture then shares and the ones a start needs.
    warm_wrapper(to);
}

/// Create a fixture in the fixture root:
///  `<root>/wrapper/cli-git-native` (hard link of the built executable) and `<root>/bin/git` (symbolic link to it).
pub fn fixture(name: &str) -> Fixture {
    return fixture_in(fixture_root().as_path(), name);
}

/// Create a fixture in `directory`, which may be on another filesystem than the build; see `fixture`.
pub fn fixture_in(directory: &Path, name: &str) -> Fixture {
    forbid_core_dumps();
    std::fs::create_dir_all(directory).expect("fixture directory root");
    let created: PathBuf = directory.join(format!("native-binary-{}-{name}", std::process::id()));
    if created.exists() {
        std::fs::remove_dir_all(&created).expect("remove stale fixture");
    }
    std::fs::create_dir(&created).expect("fresh fixture");
    let root: PathBuf = std::fs::canonicalize(&created).expect("canonical fixture");
    for subdirectory in ["wrapper", "bin", "home"] {
        std::fs::create_dir(root.join(subdirectory)).expect("fixture directory");
    }
    let wrapper: PathBuf = root.join("wrapper/cli-git-native");
    link_built_wrapper(Path::new(WRAPPER), wrapper.as_path(), directory);
    std::os::unix::fs::symlink(&wrapper, root.join("bin/git")).expect("git-named link");
    return Fixture { root, wrapper, service: std::sync::Mutex::new(None), service_environment: std::sync::Mutex::new(Vec::new()), service_launch: std::sync::Mutex::new(ServiceLaunch::default()) };
}

/// How many times a removal that finds the directory still gaining entries is tried,
/// and the pause between tries.
const REMOVAL_TRIES: usize = 50;

/// The pause between two tries of a removal.
const REMOVAL_PAUSE: std::time::Duration = std::time::Duration::from_millis(100);

/// What:
///  Remove the tree at `root` with `attempt`,
/// trying again while the directory is not empty.
/// Why:
///   A landed commit or a command that stopped a service leaves a child finishing for a moment,
/// and removing a directory that
///  still gains entries fails with "not empty";
/// that is an event that ends by itself,
/// so the removal waits for it.
/// A tree that
///  is already gone is not a failure,
/// and any other error is returned at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(root, { recursive: true, maxRetries: tries, retryDelay: pause })
/// ```
pub fn remove_patiently(root: &Path, attempt: &dyn Fn(&Path) -> std::io::Result<()>, tries: usize, pause: std::time::Duration) -> std::io::Result<()> {
    let mut last: std::io::Result<()> = Ok(());
    for _ in 0..tries {
        match attempt(root) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => {
                last = Err(error);
                std::thread::sleep(pause);
            }
            Err(error) => return Err(error),
        }
    }
    return last;
}

/// Remove only the fixture directory a control created.
pub fn remove(fixture: &Fixture) {
    remove_patiently(&fixture.root, &|root| { return std::fs::remove_dir_all(root); }, REMOVAL_TRIES, REMOVAL_PAUSE).expect("remove only the fixture");
}

/// Apply the hermetic environment every wrapped and direct run shares.
pub fn hermetic(command: &mut Command, fixture: &Fixture, path: &OsStr) {
    command
        .env_clear()
        .env("PATH", path)
        .env("HOME", fixture.root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        // No run reaches the host's journal: the entries the wrapper sends go to an abstract name nobody listens at, unless a
        //  control binds the name it passes itself.
        .env(git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, "@cli-git-test-journal-nobody-hermetic")
        .current_dir(&fixture.root);
    // A developer who sets the wrapper's verbose switch for a run gets its debug log in every wrapped command's
    //  output; no run sets it otherwise, so the environment stays hermetic.
    if let Some(verbose) = std::env::var_os("MONOCHROMATIC_VERBOSE") {
        command.env("MONOCHROMATIC_VERBOSE", verbose);
    }
}

/// The PATH most controls use:
///  the wrapper's link directory ahead of real Git.
pub fn wrapper_first_path(fixture: &Fixture) -> OsString {
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    return path;
}

/// A command for `program` (a wrapper path) under the given PATH,
///  killed after `seconds`.
pub fn bounded_for(fixture: &Fixture, program: &Path, path: &OsStr, seconds: &str) -> Command {
    if seconds == TIME_BOUND_SECONDS {
        // The shared bound is the supervisor's; its thread finds the command among the test's children once it runs.
        super::supervisor::ensure_monitor();
    }
    let mut command: Command = Command::new("/usr/bin/timeout");
    command.arg("--signal=KILL").arg(seconds).arg(program);
    hermetic(&mut command, fixture, path);
    if let Some(runtime) = service_runtime(fixture) {
        command.env("XDG_RUNTIME_DIR", runtime);
    }
    return command;
}

/// A time-bounded command for `program` (a wrapper path) under the given PATH.
pub fn bounded(fixture: &Fixture, program: &Path, path: &OsStr) -> Command {
    return bounded_for(fixture, program, path, TIME_BOUND_SECONDS);
}

/// What:
///  Put a `man` in the fixture's `bin` directory that prints one marker line naming the page and exits 0.
/// Why:
///  Git renders `--help` by running `man`, which starts `preconv`, `tbl`, `nroff`, `col`, `groff`, `troff` and
///  `grotty` (11 process starts in all, 18 ms warm and 0.7 s cold on the development host, 3.2 s once under load),
///  none of it the wrapper's work. Git looks `man` up on the command's `PATH`, whose first entry is this directory,
///  so the wrapper still receives and forwards the real `--help` and Git still takes its manual-page path.
pub fn stub_manual_viewer(fixture: &Fixture) {
    executable(
        fixture.root.join("bin/man").as_path(),
        b"#!/bin/sh\necho \"manual page requested: $*\"\n",
    );
}

/// Expose real Git at `<root>/real/git`,
///  a location that is not a conventional one.
/// Conventional locations are promoted to the front of the candidate list,
///  which would
/// hide whether earlier PATH entries were examined and skipped.
pub fn link_real_git(fixture: &Fixture) {
    std::fs::create_dir(fixture.root.join("real")).expect("real directory");
    std::os::unix::fs::symlink(REAL_GIT, fixture.root.join("real/git")).expect("real Git link");
}

/// A time-bounded command for the fixture's `bin/git` link with the wrapper first on PATH.
pub fn wrapped(fixture: &Fixture) -> Command {
    return bounded(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
    );
}

/// The same invocation sent straight to real Git,
///  as the comparison oracle.
pub fn direct(fixture: &Fixture) -> Command {
    let mut command: Command = Command::new(REAL_GIT);
    hermetic(&mut command, fixture, OsStr::new("/usr/bin:/bin"));
    return command;
}

/// What:
///  Make the in-process object readers decline the repository, with a replace ref on two blobs nothing uses.
/// Why:
///   The controls with a stand-in `git-lfs` check what the wrapper does around the program it starts (order, status,
///  explanation, deadlines). The wrapper starts the program only when it finds a pointer in the pushed commits or when
///  the readers decline and `git-lfs` has to scan; these repositories hold no pointer, so they take the second path,
///  which is the fallback the readers' declines lead to in production.
pub fn decline_in_process_lfs_scan(fixture: &Fixture, repo: &Path) {
    let mut hashes: Vec<String> = Vec::new();
    for (name, text) in [("replace-from.txt", "from\n"), ("replace-to.txt", "to\n")] {
        let file: PathBuf = fixture.root.join(name);
        std::fs::write(&file, text).expect("blob source");
        let output = direct(fixture).current_dir(repo).args(["hash-object", "-w"]).arg(&file).output().expect("hash-object starts");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(output.stderr.as_slice()));
        hashes.push(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned());
    }
    let replaced = direct(fixture).current_dir(repo).args(["replace", hashes[0].as_str(), hashes[1].as_str()]).output().expect("replace starts");
    assert!(replaced.status.success(), "{}", String::from_utf8_lossy(replaced.stderr.as_slice()));
}

/// The bytes without their additive `"hint"` fields.
///  Most binary tests pin the incumbent's event shape, which has no hint;
///  the hint text is checked by tests that use `observe_raw`.
pub fn without_hint_fields(bytes: &[u8]) -> Vec<u8> {
    let marker: &[u8] = b",\"hint\":\"";
    let mut kept: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index: usize = 0;
    while index < bytes.len() {
        if bytes[index..].starts_with(marker) {
            index += marker.len();
            while index < bytes.len() && bytes[index] != b'"' {
                index += if bytes[index] == b'\\' { 2 } else { 1 };
            }
            index += 1;
        } else {
            kept.push(bytes[index]);
            index += 1;
        }
    }
    return kept;
}

/// How every wait notice of the wrapper begins (`wait_progress.rs`, `service_index_lock.rs`).
pub const WAIT_NOTICE_PREFIX: &[u8] = b"cli-git: still waiting for ";

/// What:
///  The bytes without the lines that are wait notices.
/// Why:
///  A step that outlasts its notice grace (5 seconds for a query, 10 for a bulk step) prints a notice on standard
///  error however healthy it is, and a loaded host stretches any step past that. A control that compares the
///  wrapper's diagnostics with native Git's words means the words, not that the host was fast, so it compares what
///  is left. Only a line that starts with the notice prefix goes; every other line stays, and a control that
///  asserts a notice reads the raw bytes.
pub fn without_wait_notices(bytes: &[u8]) -> Vec<u8> {
    return bytes
        .split_inclusive(|byte| { return *byte == b'\n'; })
        .filter(|line| { return !line.starts_with(WAIT_NOTICE_PREFIX); })
        .flatten()
        .copied()
        .collect();
}

/// Run a command to completion and observe it without its hint fields,
///  feeding `input` to its standard input.
pub fn observe(command: &mut Command, input: &[u8]) -> Observed {
    let raw: Observed = observe_raw(command, input);
    return Observed {
        code: raw.code,
        stdout: without_hint_fields(raw.stdout.as_slice()),
        stderr: without_hint_fields(raw.stderr.as_slice()),
    };
}

/// What a command run under the shared test hang guard (`TIME_BOUND_SECONDS`) asks the wrapper to do, read the way the
///  wrapper reads it.
enum SharedBoundMisuse {
    /// A commit the wrapper lands through its transaction.
    CommitTransaction,
    /// A `--help` that makes Git render a manual page through `man` and the groff programs.
    ManualPage,
}

/// What:
///  Whether `command` runs under the shared bound (`timeout --signal=KILL 5 <program> <arguments>`) with
///  arguments that need a longer bound or a stub instead; absent for any other command.
/// Why:
///  The shared hang guard is for single Git commands, and is not a limit on how long the wrapper may take.
///  A landed commit starts 40 Git processes and makes 36
///  sync calls (`binary_budget_tests.rs`), against one process for direct Git, and a manual page starts 9 more
///  (`man`, `preconv`, `tbl`, `nroff`, `col`, `groff`, `troff`, `grotty`), so under host load either one can pass
///  the bound while the policy it is there to test works. This check does not depend on timing, so a misuse fails
///  on every host, loaded or quiet, instead of once a day on a busy one.
fn shared_bound_misuse(command: &Command) -> Option<SharedBoundMisuse> {
    let arguments: Vec<OsString> = command.get_args().map(OsStr::to_os_string).collect();
    let [signal, bound, program, wrapped @ ..] = arguments.as_slice() else {
        return None;
    };
    if signal != "--signal=KILL" || bound != TIME_BOUND_SECONDS {
        return None;
    }
    let stripped = git_policy_cli::wrapper_invocation::strip_wrapper_controls(wrapped);
    if stripped.layout.outcome != git_policy_cli::global_arguments::GlobalOutcome::Command {
        return None;
    }
    let region: &[OsString] = git_policy_cli::wrapper_invocation::command_region(&stripped);
    let word: &[u8] = git_policy_cli::wrapper_invocation::command_word(&stripped);
    if word == b"commit"
        && git_policy_cli::command_commit::parse_commit_region(region, &[]).is_ok_and(|facts| { return !facts.dry_run; })
    {
        return Some(SharedBoundMisuse::CommitTransaction);
    }
    let has_manual_stub: bool = Path::new(program)
        .parent()
        .is_some_and(|directory| { return directory.join("man").exists(); });
    let asks_for_help: bool = region
        .iter()
        .take_while(|token| { return *token != "--"; })
        .any(|token| { return token == "--help"; });
    if word != b"cli-git" && asks_for_help && !has_manual_stub {
        return Some(SharedBoundMisuse::ManualPage);
    }
    return None;
}

/// What:
///  Waits for a spawned child with its input and its output moving from the first moment,
/// claims the supervisor's kill of it
///  if there is one,
/// and only then runs `cleanup`.
/// Why:
///   The input is written by a thread of its own,
/// so a child that fills its output before it reads all its input is not
///  deadlocked by the test,
/// and `wait_with_output` drains both pipes from then on:
/// a command that writes much is not idle.
///  The kill is claimed before the cleanup because the cleanup can take longer than the grace in which an unclaimed kill
///  aborts the process.
///
/// # Panics
///  With the supervisor's message when it ended the command.
pub fn supervised_wait(mut child: Child, input: &[u8], cleanup: impl FnOnce()) -> Output {
    let child_id: u32 = child.id();
    let mut stdin = child.stdin.take().expect("piped stdin");
    let owned: Vec<u8> = input.to_vec();
    // A child that exits without reading its input closes the pipe;
    // that is not a test failure.
    let writer = std::thread::spawn(move || { let _ = stdin.write_all(owned.as_slice()); });
    let output: Output = child.wait_with_output().expect("process finishes");
    let _joined = writer.join();
    // The supervisor ended it:
    // a loop or a deadlock,
    // with the reason and the tasks of the tree in the message.
    if let Some(message) = super::supervisor::take_kill(child_id) {
        panic!("{message}");
    }
    cleanup();
    return output;
}

/// Run a command to completion with every byte of its output kept,
///  feeding `input` to its standard input.
/// # Panics
///  When the command is a commit transaction or a manual page under the shared bound; see `shared_bound_misuse`.
pub fn observe_raw(command: &mut Command, input: &[u8]) -> Observed {
    match shared_bound_misuse(command) {
        Some(SharedBoundMisuse::CommitTransaction) => panic!(
            "a commit under the shared bound: the work of a landed commit happens in the commit service, outside the command's \
             process tree, so the supervisor would see a tree that waits and end it; run it with `run_commit` (the {}-second \
             wall-clock bound of binary_commit_tests.rs)",
            super::commit::COMMIT_BOUND_SECONDS
        ),
        Some(SharedBoundMisuse::ManualPage) => panic!(
            "a `--help` under the shared bound renders a manual page through man and groff (9 process starts) that is Git's \
             work and the host's `man`; put an executable `man` stub in the fixture's bin directory first (`stub_manual_viewer`)"
        ),
        None => {}
    }
    let child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("process starts");
    let output: Output = supervised_wait(child, input, || {
        if let Some(directory) = command.get_current_dir() {
            settle_service_work(directory);
        }
    });
    assert_ne!(
        output.status.signal(),
        Some(TIMED_OUT_SIGNAL),
        // Built only on failure: a bound hit under host load must name the command that stalled.
        "the command hit its wall-clock bound: {:?} in {:?}",
        command.get_args().collect::<Vec<&OsStr>>(),
        command.get_current_dir()
    );
    return Observed {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    };
}

/// How long a control waits for the after-commit work (the push, the upkeep) the commit service runs once the commit returned.
const SERVICE_WORK_PATIENCE: std::time::Duration = std::time::Duration::from_secs(60);

/// Every state directory of a work directory with the names it holds, for the failure message of a wait that ran out.
fn work_listing(work: &Path) -> String {
    return ["pending", "claimed", "recovering", "done", "failed"]
        .iter()
        .map(|state| {
            let mut names: Vec<String> = std::fs::read_dir(work.join(state))
                .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
                .unwrap_or_default();
            names.sort();
            return format!("{state}={names:?}");
        })
        .collect::<Vec<String>>()
        .join(" ");
}

/// What:
///  Waits until the commit service has settled the after-commit work of the repository around `directory`.
/// Why:
///   A commit that landed through the service returns before its push and its upkeep run, so a control that inspects the
///  origin, the packs or the push's failure after the command must first wait for the records to leave `pending`,
///  `claimed` and `recovering`; it waits only where a work directory exists.
pub fn settle_service_work(directory: &Path) {
    let Some(git_dir) = directory.ancestors().map(|candidate| { return candidate.join(".git"); }).find(|candidate| { return candidate.is_dir(); }) else {
        return;
    };
    let work: PathBuf = git_dir.join(git_policy_cli::work_item::WORK_DIRECTORY);
    let open = |state: &str| -> usize { return std::fs::read_dir(work.join(state)).map_or(0, Iterator::count); };
    let began: std::time::Instant = std::time::Instant::now();
    while open("pending") + open("claimed") + open("recovering") > 0 {
        assert!(began.elapsed() < SERVICE_WORK_PATIENCE, "the service never settled the after-commit work in {}: {}", work.display(), work_listing(work.as_path()));
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// What:
///  Gives the fixture's commit service a variable, as the user manager's environment would.
/// Why:
///   The push runs in the service's environment, not in the terminal's, so a control that sets a variable for the push (a
///  deadline, a marker, a temporary directory) must set it where the push runs. The service reads it at its start, so it is
///  given before the fixture's first wrapped command.
///
/// # Panics
///
/// When the service already started with another value of the variable.
pub fn give_service_environment(fixture: &Fixture, name: &str, value: &str) {
    let mut given = fixture.service_environment.lock().expect("the service environment");
    let present = fixture.service.lock().expect("the fixture's service lock").is_some();
    let already: bool = given.iter().any(|(held, text)| { return held == name && text == value; });
    assert!(already || !present, "the fixture's service already started without {name}={value}");
    if !already {
        given.push((OsString::from(name), OsString::from(value)));
    }
}

/// The words that say a backup push failed: the command's own line when the wrapper pushes, the notice the service leaves
///  for the next commit when the service does.
pub fn push_failed_words() -> &'static str {
    // The notice the service leaves ends with the line the wrapper prints itself after a push it ran.
    return git_policy_cli::auto_push::PUSH_FAILED_NOTE;
}

/// What a person is told about the push of the commit that `stderr` belongs to: the command's own report, and the notices the
///  service left in the repository at `directory`, worded as the next command shows them (and taken as it takes them).
pub fn push_report(directory: &Path, stderr: &str) -> String {
    let mut report: String = stderr.to_string();
    settle_service_work(directory);
    let git_dir: Option<PathBuf> = directory.ancestors().map(|candidate| { return candidate.join(".git"); }).find(|candidate| { return candidate.is_dir(); });
    if let Some(found) = git_dir {
        // The notices are worded the way the next command in the worktree shows them, and are taken as that command takes them.
        for notice in git_policy_cli::push_failure_notice::take_push_failures(found.as_path()) {
            report.push_str(notice.as_str());
            report.push('\n');
        }
    }
    return report;
}

/// An editor that records that it started and then waits for a minute, so a commit that opens it sits inside native
///  `git commit` until a signal arrives. Hooks are unsupported, so controls that need the commit to stand still use this.
///
/// The editor catches the signals the controls send and ends with the signal's status, so no timing decides the
///  verdict. A shell that waits for a foreground child ignores `SIGINT` and `SIGQUIT` unless that child dies of the
///  same signal, and a shell that is about to `exec` can lose one too (measured: the editor with `touch` then
///  `sleep` lost an interrupt, and with `exec sleep` it lost a quit, at the instant after it announced itself).
///  The editor then slept its full minute and ended the commit with status 1 and "Aborting commit due to empty
///  commit message." instead of the signal's status. The handlers are set before the announcement, so every signal
///  that can follow the announcement is caught; `wait` returns at once on a caught signal; the sleep has no
///  terminal and no pipe of the test, so a sleep left behind cannot hold the commit's standard error open.
pub fn stalling_editor(fixture: &Fixture, started: &Path) -> PathBuf {
    let path: PathBuf = fixture.root.join("stalling-editor");
    executable(
        path.as_path(),
        format!(
            "#!/bin/sh\nstall=\ntrap 'kill $stall 2>/dev/null; exit 130' INT\ntrap 'kill $stall 2>/dev/null; exit 131' QUIT\nprintf started > '{}'\nsleep 60 </dev/null >/dev/null 2>&1 &\nstall=$!\nwait $stall\n",
            started.display()
        )
        .as_bytes(),
    );
    return path;
}

/// Run real Git directly inside a fixture and require success.
pub fn git<S: AsRef<OsStr>>(fixture: &Fixture, directory: &Path, arguments: &[S]) -> Output {
    let output: Output = direct(fixture)
        .current_dir(directory)
        .args(arguments)
        .output()
        .expect("native Git fixture command");
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    return output;
}

/// Create a repository with one empty commit on `main`,
///  using real Git directly.
pub fn repository(fixture: &Fixture, name: &OsStr) -> PathBuf {
    let root: PathBuf = fixture.root.join(name);
    std::fs::create_dir(&root).expect("repository directory");
    git(
        fixture,
        root.as_path(),
        &["init", "--quiet", "--initial-branch=main"],
    );
    git(
        fixture,
        root.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message=initial"],
    );
    return root;
}

/// Run a wrapped command in a directory,
///  with empty standard input,
///  and return what the caller saw.
pub fn run_wrapped(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Observed {
    return observe(wrapped(fixture).current_dir(directory).args(arguments), b"");
}

/// Run real Git directly in a directory,
///  with empty standard input,
///  and return what the caller saw.
pub fn run_direct(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Observed {
    return observe(direct(fixture).current_dir(directory).args(arguments), b"");
}

/// Real Git's porcelain status of a repository,
///  untracked files included,
///  as text.
pub fn porcelain(fixture: &Fixture, repo: &Path) -> String {
    let output: Output = git(
        fixture,
        repo,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    );
    return String::from_utf8_lossy(&output.stdout).into_owned();
}

/// What a caller sees when a command stops with exit status 2 and this text on standard error.
pub fn stopped_with(stderr: &str) -> Observed {
    return Observed {
        code: Some(2),
        stdout: Vec::<u8>::new(),
        stderr: stderr.as_bytes().to_vec(),
    };
}

/// What a caller sees when a command succeeds without printing anything.
pub fn silent_success() -> Observed {
    return Observed {
        code: Some(0),
        stdout: Vec::<u8>::new(),
        stderr: Vec::<u8>::new(),
    };
}

/// Standard error of an observed command as text.
pub fn stderr_of(observed: &Observed) -> String {
    return String::from_utf8_lossy(&observed.stderr).into_owned();
}

/// The one writer of executable fixture files, shared with the unit tests.
#[path = "../testing/fixture_executable.rs"]
mod fixture_executable;

pub use fixture_executable::executable;

/// The shell condition that holds while the test that started a fixture script is still there.
#[path = "../testing/test_alive.rs"]
mod test_alive;

pub use test_alive::{end_processes_working_in, while_test_lives};

/// What:
///  Copy an executable through a child `cp` process,
///  keeping its mode.
/// Why:
///   A copy made by this process would be open for writing here,
///  with the same
///       "Text file busy" window as `executable` describes.
///  Fixtures reach the wrapper through a hard link (`fixture`), so this is for controls that need a second file with
///  the wrapper's bytes and a different identity.
///  The copy is a real data copy (`--reflink=never`): on btrfs, which also backs overlay storage, the default
///  `cp` clones the extents, and the clone has no pages in the page cache, so the first start of the copy reads
///  most of the executable from disk inside the 5-second test hang guard (a 22 MB debug build read 16 MiB in 4 major faults;
///  `doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`). A data copy leaves the pages
///  it wrote in the cache, and a slow read of the original happens here, outside any bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync('cp', ['--reflink=never', '--', from, to]);
/// ```
pub fn copy_executable(from: &Path, to: &Path) {
    let copied: bool = Command::new("cp")
        .arg("--reflink=never")
        .arg("--")
        .arg(from)
        .arg(to)
        .status()
        .expect("start fixture copy")
        .success();
    assert!(copied, "fixture copy failed: {from:?} to {to:?}");
}

/// The transaction ID registry fixtures use.
pub const TRANSACTION_ID: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// What:
///  Leave a commit transaction directory of an earlier release in a repository's registry.
/// Why:
///   The wrapper no longer reads such a directory; a control that proves it goes on beside one, and only names it, needs one there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(join(registry, transactionId), { recursive: true });
/// ```
pub fn registered_transaction(repo: &Path) -> PathBuf {
    let registry: PathBuf = repo.join(".git/cli-git-transactions");
    std::fs::create_dir_all(registry.join(TRANSACTION_ID)).expect("a transaction directory an earlier release left");
    return registry;
}

/// What:
///  The owner of a process that really started, had its birth identity read while it ran, and was then killed and reaped.
/// Why:
///   An interrupted copy is a journal or a claim whose owner is gone; a control builds one with this owner instead of
///  waiting for a real crash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dead = await deadCopyOwner(); // { pid, birthIdentity, pidNamespace } of a killed child
/// ```
pub fn dead_copy_owner() -> git_policy_cli::worktree_copy_owner::CopyOwner {
    let mut child: Child = Command::new("sleep")
        .arg("60")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sleep starts");
    let pid: i64 = i64::from(child.id());
    let birth: String = git_policy_cli::process_identity::process_birth_identity(pid)
        .expect("identity readable")
        .expect("the child runs");
    child.kill().expect("child is killed");
    child.wait().expect("child is reaped");
    return git_policy_cli::worktree_copy_owner::CopyOwner {
        pid,
        birth_identity: birth,
        pid_namespace: git_policy_cli::pid_namespace::current_pid_namespace(),
    };
}

/// What:
///  Forbid core dumps for this test process and everything it starts, once.
/// Why:
///   The binary tests kill services and abort wrappers on purpose, and each dump costs the system's core-dump handler processor
///  time and disk writes on a disk other sessions share. The limit is inherited across `fork` and `exec`, and the hard limit
///  stops a child from raising it; it is the harness's, and the product sets none.
fn forbid_core_dumps() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let none: libc::rlimit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        // SAFETY: `setrlimit` reads the one `rlimit` it is given and touches no memory of ours.
        let _set = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &none) };
    });
}
