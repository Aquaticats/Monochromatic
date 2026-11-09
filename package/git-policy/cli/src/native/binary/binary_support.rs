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
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

/// The helpers that moved to files of their own,
/// named here so every `support::name` path stays the same.
pub use crate::support_btrfs_controls::*;
pub use crate::support_fixture_directories::*;
pub use crate::support_fixture_service::*;
pub use crate::support_fixture_service_log::*;
pub use crate::support_fixture_service_readiness::*;
pub use crate::support_observation::*;
pub use crate::support_program_names_and_time_bound::*;
pub use crate::support_resource_limits::*;
pub use crate::support_scratch_left_behind::*;




























/// A pipe whose two ends are closed on `exec`:
/// `[read end, write end]`.
/// Linux makes both ends close-on-exec in the one `pipe2` call;
///  macOS has no `pipe2`,
/// so there the flag is set on each end right after `pipe`.
#[cfg(target_os = "linux")]
pub fn close_on_exec_pipe() -> [libc::c_int; 2] {
    let mut ends: [libc::c_int; 2] = [0, 0];
    // SAFETY: `pipe2` fills the two integers it is given.
    let made: libc::c_int = unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) };
    assert_eq!(made, 0, "pipe2 failed: {}", std::io::Error::last_os_error());
    return ends;
}

/// A pipe whose two ends are closed on `exec`:
/// `[read end, write end]`.
/// Linux makes both ends close-on-exec in the one `pipe2` call;
///  macOS has no `pipe2`,
/// so there the flag is set on each end right after `pipe`.
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
///  macOS has none,
/// and `TIOCPTYGNAME` returns the name into the caller's buffer just as safely.
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
///  macOS has none,
/// and `TIOCPTYGNAME` returns the name into the caller's buffer just as safely.
#[cfg(all(unix, not(target_os = "linux")))]
pub fn pseudo_terminal_slave_name(descriptor: libc::c_int) -> std::ffi::CString {
    let mut buffer: [libc::c_char; 128] = [0; 128];
    // SAFETY: `TIOCPTYGNAME` writes a NUL-terminated name of at most 128 bytes into the buffer it is given.
    let named: libc::c_int = unsafe { libc::ioctl(descriptor, libc::TIOCPTYGNAME as libc::c_ulong, buffer.as_mut_ptr()) };
    assert_eq!(named, 0, "the slave has a name");
    // SAFETY: the call above wrote a NUL-terminated name into `buffer`.
    return unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }.to_owned();
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
    // The place a waiting commit is told is a line a healthy commit must not print, so it is set past the bound that
    //  ends this command: a control that checks the line itself sets its own.
    command.env(git_policy_cli::wait_ceilings::WAITING_NOTICE_VARIABLE, seconds);
    return command;
}

/// A time-bounded command for `program` (a wrapper path) under the given PATH.
pub fn bounded(fixture: &Fixture, program: &Path, path: &OsStr) -> Command {
    return bounded_for(fixture, program, path, TIME_BOUND_SECONDS);
}

/// What:
///  Put a `man` in the fixture's `bin` directory that prints one marker line naming the page and exits 0.
/// Why:
///  Git renders `--help` by running `man`,
/// which starts `preconv`,
/// `tbl`,
/// `nroff`,
/// `col`,
/// `groff`,
/// `troff` and
///  `grotty` (11 process starts in all,
/// 18 ms warm and 0.7 s cold on the development host,
/// 3.2 s once under load),
///  none of it the wrapper's work.
/// Git looks `man` up on the command's `PATH`,
/// whose first entry is this directory,
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
///  Make the in-process object readers decline the repository,
/// with a replace ref on two blobs nothing uses.
/// Why:
///   The controls with a stand-in `git-lfs` check what the wrapper does around the program it starts (order,
/// status,
///  explanation,
/// deadlines).
/// The wrapper starts the program only when it finds a pointer in the pushed commits or when
///  the readers decline and `git-lfs` has to scan;
/// these repositories hold no pointer,
/// so they take the second path,
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












/// What:
///  Gives the fixture's commit service a variable,
/// as the user manager's environment would.
/// Why:
///   The push runs in the service's environment,
/// not in the terminal's,
/// so a control that sets a variable for the push (a
///  deadline,
/// a marker,
/// a temporary directory) must set it where the push runs.
/// The service reads it at its start,
/// so it is
///  given before the fixture's first wrapped command.
///
/// # Panics
///
/// When the service already started with another value of the variable.
pub fn give_service_environment(fixture: &Fixture, name: &str, value: &str) {
    // The service's lock first and the environment's second: `service_runtime` holds the service's lock while it starts a service,
    //  and the start reads the environment, so the other order let two threads wait for each other for ever.
    let service = fixture.service.lock().expect("the fixture's service lock");
    let mut given = fixture.service_environment.lock().expect("the service environment");
    let present = service.is_some();
    let already: bool = given.iter().any(|(held, text)| { return held == name && text == value; });
    assert!(already || !present, "the fixture's service already started without {name}={value}");
    if !already {
        given.push((OsString::from(name), OsString::from(value)));
    }
}

/// The words that say a backup push failed:
/// the command's own line when the wrapper pushes,
/// the notice the service leaves
///  for the next commit when the service does.
pub fn push_failed_words() -> &'static str {
    // The notice the service leaves ends with the line the wrapper prints itself after a push it ran.
    return git_policy_cli::auto_push::PUSH_FAILED_NOTE;
}

/// What a person is told about the push of the commit that `stderr` belongs to:
/// the command's own report,
/// and the notices the
///  service left in the repository at `directory`,
/// worded as the next command shows them (and taken as it takes them).
pub fn push_report(directory: &Path, stderr: &str) -> String {
    let mut report: String = stderr.to_string();
    settle_service_work(directory);
    let git_dir: Option<PathBuf> = directory.ancestors().map(|candidate| { return candidate.join(".git"); }).find(|candidate| { return candidate.is_dir(); });
    if let Some(found) = git_dir {
        // The notices are worded the way the next command in the worktree shows them, and are shown and removed as that
        //  command does.
        for notice in git_policy_cli::push_failure_notice::push_failures(found.as_path()) {
            let mut shown: Vec<u8> = Vec::new();
            let _written = notice.show(&mut shown);
            report.push_str(String::from_utf8_lossy(shown.as_slice()).as_ref());
        }
    }
    return report;
}

/// An editor that records that it started and then waits for a minute,
/// so a commit that opens it sits inside native
///  `git commit` until a signal arrives.
/// Hooks are unsupported,
/// so controls that need the commit to stand still use this.
///
/// The editor catches the signals the controls send and ends with the signal's status,
/// so no timing decides the
///  verdict.
/// A shell that waits for a foreground child ignores `SIGINT` and `SIGQUIT` unless that child dies of the
///  same signal,
/// and a shell that is about to `exec` can lose one too (measured:
/// the editor with `touch` then
///  `sleep` lost an interrupt,
/// and with `exec sleep` it lost a quit,
/// at the instant after it announced itself).
///  The editor then slept its full minute and ended the commit with status 1 and "Aborting commit due to empty
///  commit message."
/// instead of the signal's status.
/// The handlers are set before the announcement,
/// so every signal
///  that can follow the announcement is caught;
/// `wait` returns at once on a caught signal;
/// the sleep has no
///  terminal and no pipe of the test,
/// so a sleep left behind cannot hold the commit's standard error open.
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






/// The transaction ID registry fixtures use.
pub const TRANSACTION_ID: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// What:
///  Leave a commit transaction directory of an earlier release in a repository's registry.
/// Why:
///   The wrapper no longer reads such a directory;
/// a control that proves it goes on beside one,
/// and only names it,
/// needs one there.
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
///  The owner of a process that really started,
/// had its birth identity read while it ran,
/// and was then killed and reaped.
/// Why:
///   An interrupted copy is a journal or a claim whose owner is gone;
/// a control builds one with this owner instead of
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
