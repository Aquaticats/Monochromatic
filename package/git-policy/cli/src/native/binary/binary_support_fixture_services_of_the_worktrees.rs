//! What:
//!  The services a fixture holds,
//!  one for each worktree under it as in production,
//!  and the acts that start one,
//!  stop them all and forget the ones that ended.
//! Why:
//!  A fixture has no user manager:
//!  in production that manager starts one instance per worktree at its first connection,
//!  so the acts that reproduce that work for a control describe how a service is started
//!  rather than the fixture's own lifecycle,
//!  and the lifecycle file stays inside the repository's per-file line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const service = startFixtureServiceIn(runtime); service.serve(fixture, gitDir); // one per worktree, on demand
//! ```

use crate::support_fixture_service::{
    Fixture,
    FixtureService,
    kill_with_everything_it_started,
};
use crate::support_fixture_service_readiness::{
    SERVICE_QUIET_WINDOW,
    SignsOfWork,
    service_is_serving,
    wait_until_nothing_listens,
};
use crate::support_fixture_service_start::{
    SERVICE_PROGRAM,
    spawn_for_the_process_lifetime,
};
use crate::support_resource_limits::{
    ServiceLaunch,
    restrict_to_one_cpu,
};
use git_policy_cli::service_server::Bind;
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
    Stdio,
};

/// What:
///  Makes the runtime directory `runtime` the one a fixture's services listen in.
/// Why:
///   A runtime directory too long for a socket path is the case this package's address form exists for,
///  and only a control that chooses the directory can make one that long.
///  The service is started at whatever address `socket_address_of` gives that directory,
///  which is the address the wrapper's own client computes.
pub fn start_fixture_service_in(runtime: PathBuf) -> FixtureService {
    // The socket file stands in a folder of its own, which a socket unit's `DirectoryMode=` makes in production.
    std::fs::create_dir_all(runtime.join("monochromatic-git-policy-cli"))
        .expect("the folder of the sockets");
    return FixtureService {
        children: std::sync::Mutex::new(std::collections::HashMap::new()),
        log_copies: std::sync::Mutex::new(Vec::new()),
        runtime,
    };
}

impl FixtureService {
    /// What:
    ///  Starts the service of the worktree whose own Git directory is `git_dir`,
    /// listening at the address the wrapper looks for,
    ///  unless it already runs.
    /// Why:
    ///   In production the manager starts one instance per worktree on its first connection;
    /// the fixture has no manager,
    /// so
    ///  it starts the instance itself and waits until it listens.
    /// The address is the one `socket_address_of` gives this runtime directory,
    /// which is a socket file where a path fits one socket address and an abstract name where it does not,
    ///  and the service is given the one or the other on its command line.
    pub fn serve(
        &self,
        fixture: &Fixture,
        git_dir: &Path,
    ) {
        use std::os::unix::process::CommandExt;
        let key: PathBuf = std::fs::canonicalize(git_dir).unwrap_or_else(|_| {
            return git_dir.to_path_buf();
        });
        let mut children = self.children.lock().expect("the fixture's services");
        if children.contains_key(&key) {
            return;
        }
        let instance: String = git_policy_cli::service_instance::instance_of(key.as_path());
        let address: Bind = git_policy_cli::service_address_of_a_worktree::socket_address_of(
            self.runtime.as_path(),
            instance.as_str(),
        );
        // A service ended a moment ago may still hold the socket; the new one would end with its address in use while the poll below
        //  connected to the old one and called the new one ready.
        wait_until_nothing_listens(&address);
        let launch: ServiceLaunch = fixture
            .service_launch
            .lock()
            .expect("the service launch")
            .clone();
        // A helper that an older program started has the earlier command word in its unit file.
        let words: &[&str] = if launch.earlier_command_word {
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
            .arg(fixture.root.join("journal"));
        match &address {
            Bind::Path(path) => {
                command.arg("--path").arg(path);
            }
            Bind::Abstract(name) => {
                command.arg("--abstract").arg(name);
            }
            Bind::Inherited => panic!("the fixture's service names the address it listens at"),
        }
        let (kept_log, when_the_copy_ends): (std::os::fd::OwnedFd, std::sync::mpsc::Receiver<()>) =
            crate::support_fixture_service_log::kept_stderr_pipe(fixture.root.as_path())
                .expect("a pipe for the service's log");
        command.stderr(Stdio::from(kept_log));
        self.log_copies
            .lock()
            .expect("the fixture's log copies")
            .push(when_the_copy_ends);
        command
            .arg("--instance")
            .arg(instance.as_str())
            .env_clear()
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", fixture.root.join("bin").display()),
            )
            .env("HOME", fixture.root.join("home"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            // The identity the person's own configuration gives their Git, which a push that merges (a lost race) needs.
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .stdin(Stdio::null())
            .stdout(Stdio::null());
        for (name, value) in fixture
            .service_environment
            .lock()
            .expect("the service environment")
            .iter()
        {
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
                command.pre_exec(move || {
                    return kind.apply(soft, hard);
                });
            }
        }
        let child: Child =
            spawn_for_the_process_lifetime(command).expect("the fixture's commit service starts");
        let service_pid: u32 = child.id();
        children.insert(key.clone(), child);
        let mut signs: SignsOfWork = SignsOfWork::of(service_pid);
        let began: std::time::Instant = std::time::Instant::now();
        let mut last_change: std::time::Instant = std::time::Instant::now();
        while !service_is_serving(&address) {
            // A service that has ended will never listen, so waiting out the patience for it only hides how it ended. A
            //  control whose fault may keep the service from starting (a disk that is full from its first write) goes on
            //  once the service has ended or a short wait has passed: what the client then says is the control's to check.
            let ended: Option<std::process::ExitStatus> =
                children.get_mut(&key).and_then(|started| {
                    return started.try_wait().ok().flatten();
                });
            if launch.may_fail_to_start {
                if ended.is_some() || began.elapsed() > std::time::Duration::from_secs(20) {
                    return;
                }
            } else if let Some(status) = ended {
                children.remove(&key);
                panic!(
                    "the fixture's commit service ended ({status}) before it listened at {}; what it wrote to its log is in this test's output",
                    git_policy_cli::service_server::address_text(&address)
                );
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
                git_policy_cli::service_server::address_text(&address),
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
            let instance: String = git_policy_cli::service_instance::instance_of(key.as_path());
            wait_until_nothing_listens(
                &git_policy_cli::service_address_of_a_worktree::socket_address_of(
                    self.runtime.as_path(),
                    instance.as_str(),
                ),
            );
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
        let ended: Vec<PathBuf> = children
            .iter_mut()
            .filter_map(|(key, child)| {
                return child.try_wait().ok().flatten().map(|_| {
                    return key.clone();
                });
            })
            .collect();
        for key in &ended {
            children.remove(key);
        }
        return !ended.is_empty();
    }

    /// Starts the service of every worktree under the fixture root that has none yet:
    /// the repositories and the linked
    ///  worktrees a control made,
    /// found as Git finds them.
    pub fn serve_worktrees(
        &self,
        fixture: &Fixture,
    ) {
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
pub fn worktree_git_dirs(
    directory: &Path,
    depth: usize,
) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        let name = entry.file_name();
        if name == ".git" {
            if path.is_dir() {
                found.push(path);
            } else if let Ok(text) = std::fs::read_to_string(&path)
                && let Some(target) = text.lines().find_map(|line| {
                    return line.strip_prefix("gitdir: ");
                })
            {
                found.push(directory.join(target));
            }
        } else if depth > 0 && name != "node_modules" && path.is_dir() && !path.is_symlink() {
            found.extend(worktree_git_dirs(path.as_path(), depth - 1));
        }
    }
    return found;
}
