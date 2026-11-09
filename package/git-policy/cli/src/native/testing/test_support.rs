//! What:
//!  Disposable directories and real-Git fixtures shared by unit tests.
//! Why:
//!  Every test that touches the filesystem or a repository owns a fresh directory
//!      under the temporary directory and never reads the real home or global Git configuration.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const root = await mkdtemp(join(tmpdir(), 'native-')); await git(root, ['init']);
//! ```

/// Native string,
///  path and process types used by the fixtures.
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The real Git 2.56.0 executable installed in the verification image.
pub(crate) const REAL_GIT: &str = "/usr/bin/git";

/// How long the whole unit-test run may last before the watchdog ends it.
/// The run takes under a minute,
/// so a run past this limit has a test stuck behind something that never happens.
const RUN_LIMIT: std::time::Duration = std::time::Duration::from_secs(120);

/// Ends the test process with a message once the run has lasted `RUN_LIMIT`.
/// Without it a mutant that leaves a lock held or ignores a deadline hangs the run until an outside timeout,
/// which the mutation tool records as a timeout instead of a failed test.
/// The first fixture of the run arms it;
/// later calls do nothing.
pub(crate) fn arm_watchdog() {
    static ARMED: std::sync::Once = std::sync::Once::new();
    ARMED.call_once(|| {
        // A process that is given no thread (a control of a site that starts threads) runs without the watchdog; its parent has its own.
        let _armed = std::thread::Builder::new().spawn(|| {
            std::thread::sleep(RUN_LIMIT);
            eprintln!("the unit-test run exceeded {RUN_LIMIT:?}: a test is stuck");
            std::process::abort();
        });
    });
}

/// What:
///  Forbid core dumps for this test process and everything it starts,
/// once.
/// Why:
///   Tests make services abort or kill them on purpose,
/// and each dump costs the system's core-dump handler processor time and
///  disk writes on a disk other sessions share (several hundred dumps of `cli-git-service` from test runs on 2026-11-03).
/// A
///  process limit is inherited across `fork` and `exec` and the hard limit stops a child from raising it;
/// this is the
///  harness's,
/// and the product sets none.
pub(crate) fn forbid_core_dumps() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let none: libc::rlimit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        // SAFETY: `setrlimit` reads the one `rlimit` it is given and touches no memory of ours.
        let _set = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &none) };
    });
}

/// Environment variable that keeps every fixture on disk after its test ends,
/// for looking at a failure.
pub(crate) const KEEP_FIXTURES_VARIABLE: &str = "CLI_GIT_KEEP_FIXTURES";

/// Environment variable that moves the fixture root,
/// for example onto a tmpfs for speed.
pub(crate) const FIXTURE_ROOT_VARIABLE: &str = "CLI_GIT_TEST_ROOT";

/// The directory fixtures are created in:
/// `test-fixtures` beside the build output of the running test executable
///  (`<target>/debug/deps/<test>` has the target directory three levels up),
/// never `/tmp`.
/// A shared tmpfs with a per-user quota fills up when failing tests leave fixtures behind (issue 1809),
/// and then
/// every command of every session fails.
pub(crate) fn fixture_root() -> PathBuf {
    let root: PathBuf = std::env::var_os(FIXTURE_ROOT_VARIABLE).map_or_else(
        || {
            let target: Option<PathBuf> = this_test_binary()
                .ok()
                .and_then(|executable| { return executable.ancestors().nth(3).map(Path::to_path_buf); });
            return target
                .unwrap_or_else(|| { return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"); })
                .join("test-fixtures");
        },
        PathBuf::from,
    );
    std::fs::create_dir_all(&root).expect("fixture root");
    return root;
}

/// What:
///  Remove a fixture tree unless the person asked to keep it,
/// trying again for up to five seconds.
/// Why:
///   A landed commit may leave a maintenance child finishing for a moment;
/// a tree already gone is the explicit
///  `remove` having run first,
/// not a failure.
pub(crate) fn settle_fixture(root: &Path, keep: bool) {
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

/// The fixtures one test thread created,
/// removed when the thread ends however the test ended.
struct ThreadFixtures {
    /// Roots to remove.
    roots: std::cell::RefCell<Vec<PathBuf>>,
    /// Process groups of the helper processes the thread started with `spawn_owned`,
    /// killed when the thread ends.
    groups: std::cell::RefCell<Vec<libc::pid_t>>,
}

/// A panic unwinds the test thread,
/// which runs this.
impl Drop for ThreadFixtures {
    fn drop(&mut self) {
        // The helpers die first, so no process is left in a directory that is about to be removed.
        for group in self.groups.borrow().iter() {
            // SAFETY: `killpg` takes a group number and a signal and touches no memory; a group with no member is `ESRCH`.
            unsafe { libc::killpg(*group, libc::SIGKILL) };
        }
        let keep: bool = std::env::var_os(KEEP_FIXTURES_VARIABLE).is_some();
        for root in self.roots.borrow().iter() {
            end_processes_working_in(root.as_path());
            settle_fixture(root.as_path(), keep);
        }
    }
}

thread_local! {
    /// The fixtures of the running test thread.
    static THREAD_FIXTURES: ThreadFixtures = const { ThreadFixtures { roots: std::cell::RefCell::new(Vec::new()), groups: std::cell::RefCell::new(Vec::new()) } };
}

/// What:
///  Start a helper process that dies with the test thread that started it,
/// whether the test passes,
/// fails or panics.
/// Why:
///   A shell loop that a failing push-progress test never reached the end of ran for 87 minutes in a deleted fixture.
/// The
///  helper leads a process group of its own,
/// so what it starts (a `sleep` of the loop) is killed with it,
/// and the group is
///  killed when the thread's fixtures are dropped,
/// the same moment the fixture directories go.
pub(crate) fn spawn_owned(command: &mut Command) -> std::io::Result<std::process::Child> {
    use std::os::unix::process::CommandExt as _;
    command.process_group(0);
    let child: std::process::Child = command.spawn()?;
    own_group(&child);
    return Ok(child);
}

/// What:
///  Make the process group a child leads die with the test thread,
/// as `spawn_owned` does for the children it starts.
/// Why:
///   A helper that must become a session leader (a terminal helper calls `setsid` between `fork` and `exec`) cannot also be put in
///  a group by `spawn_owned`,
/// because a group leader cannot start a session.
/// Such a child leads a group of its own already,
///  so only the thread's list of groups to end needs its number.
pub(crate) fn own_group(child: &std::process::Child) {
    let group: libc::pid_t = libc::pid_t::try_from(child.id()).expect("a pid fits the pid type");
    THREAD_FIXTURES.with(|fixtures| { fixtures.groups.borrow_mut().push(group); });
}

/// This test process's own executable,
/// for the helper processes a control starts from the test binary.
///  A test starts the test binary,
/// never the wrapper:
/// the ban on `std::env::current_exe` (`clippy.toml`) is about the
///  wrapper starting itself,
/// so this is the one place the lint is excused for tests.
#[allow(clippy::disallowed_methods, reason = "identifies the running test binary so a control can start a helper copy of it; the wrapper is never started this way")]
pub(crate) fn this_test_binary() -> std::io::Result<PathBuf> {
    return std::env::current_exe();
}

/// Create one empty fixture directory owned by this test process and test name,
///  under a prefix that keeps the fixtures of different support modules apart.
pub(crate) fn fixture_with_prefix(prefix: &str, name: &str) -> PathBuf {
    arm_watchdog();
    forbid_core_dumps();
    let root: PathBuf = fixture_root().join(format!("{prefix}-{}-{name}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("remove stale fixture");
    }
    std::fs::create_dir(&root).expect("fresh fixture");
    // Canonical form so comparisons with Git's canonical output are exact.
    let canonical: PathBuf = std::fs::canonicalize(&root).expect("canonical fixture");
    THREAD_FIXTURES.with(|fixtures| { fixtures.roots.borrow_mut().push(canonical.clone()); });
    return canonical;
}

/// Create one empty fixture directory owned by this test process and test name.
pub(crate) fn fixture(name: &str) -> PathBuf {
    return fixture_with_prefix("native-unit", name);
}

/// What:
///  Move a path's modification time two minutes into the past without following a symbolic link.
/// Why:
///   An index entry whose file is as new as the index file proves nothing by its stat fields (Git hashes it again),
/// so a
///  control that needs a provably clean worktree ages its files before the index records them.
pub(crate) fn age(path: &Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    let past: libc::time_t = (std::time::SystemTime::now() - std::time::Duration::from_secs(120))
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("after the epoch")
        .as_secs() as libc::time_t;
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: past, tv_nsec: 0 }, libc::timespec { tv_sec: past, tv_nsec: 0 }];
    // SAFETY: `name` is a NUL-terminated string that outlives the call and `times` holds the two timestamps the call reads.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    assert_eq!(result, 0, "age {}", path.display());
}

/// Remove only the fixture directory a test created.
pub(crate) fn remove(root: &Path) {
    std::fs::remove_dir_all(root).expect("remove only the fixture");
}

/// Run real Git in a fixture with no system or global configuration and a fixed identity.
pub(crate) fn git_output<S: AsRef<OsStr>>(directory: &Path, arguments: &[S]) -> Output {
    return Command::new(REAL_GIT)
        .current_dir(directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(arguments)
        .output()
        .expect("native Git fixture command");
}

/// Run real Git in a fixture and require success.
pub(crate) fn git<S: AsRef<OsStr>>(directory: &Path, arguments: &[S]) -> Output {
    let output: Output = git_output(directory, arguments);
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        arguments.iter().map(AsRef::as_ref).collect::<Vec<&OsStr>>(),
        String::from_utf8_lossy(&output.stderr)
    );
    return output;
}

/// Create a repository with one empty commit on `main` inside a fixture directory.
pub(crate) fn repository(parent: &Path, name: &str) -> PathBuf {
    let root: PathBuf = parent.join(name);
    std::fs::create_dir(&root).expect("repository directory");
    git(
        root.as_path(),
        &["init", "--quiet", "--initial-branch=main"],
    );
    git(
        root.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message=initial"],
    );
    return root;
}

/// The one writer of executable fixture files,
/// shared with the binary-level controls.
#[cfg(unix)]
#[path = "fixture_executable.rs"]
mod fixture_executable;

#[cfg(unix)]
pub(crate) use fixture_executable::executable;

/// The writer of a fixture file under a new inode,
/// shared with the binary-level controls.
#[cfg(unix)]
#[path = "fixture_inode.rs"]
mod fixture_inode;

#[cfg(unix)]
pub(crate) use fixture_inode::renew_inode;

/// The sinks a control reads a notice from,
/// in a file of their own.
#[path = "fixture_notice_sinks.rs"]
mod fixture_notice_sinks;

pub(crate) use self::fixture_notice_sinks::{ActionSink, SharedSink};

/// The shell condition that holds while the test that started a fixture script is still there.
#[cfg(unix)]
#[path = "test_alive.rs"]
mod test_alive;

#[cfg(unix)]
pub(crate) use test_alive::{end_processes_working_in, while_test_lives};

/// What:
///  Variables that isolate a Git child started through the production runners from the
///       real system and global configuration,
///  with the fixed fixture identity.
/// Why:
///   Production runners add an overlay to the inherited environment instead of clearing
///       it,
///  so tests pass these pairs as that overlay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const overlay = [['GIT_CONFIG_NOSYSTEM', '1'], ['GIT_CONFIG_GLOBAL', '/nonexistent-global-config'], ...];
/// ```
pub(crate) fn isolated_overlay() -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    // `mut` allows collecting the pairs one at a time.
    let mut pairs: Vec<(std::ffi::OsString, std::ffi::OsString)> = Vec::new();
    for (name, value) in [
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ] {
        pairs.push((
            std::ffi::OsString::from(name),
            std::ffi::OsString::from(value),
        ));
    }
    return pairs;
}

/// What:
///  A transaction Git context running the real fixture Git with the isolating overlay.
/// Why:
///   Recovery and landing tests start Git through the production runner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const context = { gitPath: '/usr/bin/git', overlay: isolatedOverlay(), globalPrefix: [] };
/// ```
pub(crate) fn git_context() -> crate::transaction_git::GitContext {
    return crate::transaction_git::GitContext {
        real_git: PathBuf::from(REAL_GIT),
        overlay: isolated_overlay(),
        global_prefix: Vec::new(),
        // The fixture contexts name no repository, so the input files of their commands are made in the scratch
        // directory beside the fixtures, under the target directory and never in the system temporary directory.
        common_dir: Some(fixture_root()),
    };
}

/// What:
///  An environment whose home is a directory beside the fixtures,
///  where the per-user scratch directory of a child runner is made.
/// Why:
///   Runners that take an environment make their output files under its home,
///  so tests give them one under the target directory and never the real home or the system temporary directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const env = { HOME: join(fixtureRoot(), 'scratch-home') };
/// ```
pub(crate) fn scratch_environment() -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    return vec![(
        std::ffi::OsString::from("HOME"),
        fixture_root().join("scratch-home").into_os_string(),
    )];
}

/// What:
///  Real Git's standard output with one trailing newline removed,
///  as text.
/// Why:
///   Tests compare object IDs and ref values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(directory, args).stdout.toString().trimEnd()
/// ```
pub(crate) fn git_text<S: AsRef<OsStr>>(directory: &Path, arguments: &[S]) -> String {
    let output: Output = git(directory, arguments);
    let text: String = String::from_utf8(output.stdout).expect("UTF-8 Git output");
    return String::from(text.strip_suffix('\n').unwrap_or(text.as_str()));
}

/// The lines without their additive `hint` fields,
/// which the incumbent's shape does not have.
pub(crate) fn strip_hint_fields(text: &str) -> String {
    let marker: &str = ",\"hint\":\"";
    let mut rest: String = text.to_owned();
    while let Some(start) = rest.find(marker) {
        let bytes: &[u8] = rest.as_bytes();
        let mut end: usize = start + marker.len();
        while bytes[end] != b'"' {
            end += if bytes[end] == b'\\' { 2 } else { 1 };
        }
        rest = format!("{}{}", &rest[..start], &rest[end + 1..]);
    }
    return rest;
}

/// Variable that tells a child test the directory it works in.
pub(crate) const LIMITED_DIRECTORY_VARIABLE: &str = "CLI_GIT_LIMITED_CHILD_DIRECTORY";

/// What:
///  Run one `#[ignore]` child test in a process that may write no file past `limit` bytes,
/// and return how it ended.
/// Why:
///  A file-size limit is the privilege-free way to make a write fail half way:
/// the bytes below the limit are written
///  and the rest fails with `EFBIG` (`SIGXFSZ` is ignored so that the error reaches the caller instead of ending the
///  process).
/// The limit is a property of the whole process,
/// so the child runs alone and the parent's other tests
///  never see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync(process.execPath, [testBinary, '--exact', name, '--ignored'], { env: { DIRECTORY: directory }, limits: { fsize: limit } });
/// ```
#[cfg(unix)]
pub(crate) fn run_child_under_file_size_limit(test_name: &str, limit: u64, directory: &Path) -> std::process::ExitStatus {
    use std::os::unix::process::CommandExt;
    let mut command: Command = Command::new(this_test_binary().expect("test binary"));
    command
        .args(["--exact", test_name, "--ignored", "--nocapture"])
        .env(LIMITED_DIRECTORY_VARIABLE, directory)
        .stdin(std::process::Stdio::null());
    // SAFETY: between fork and exec only `signal` and `setrlimit`, both async-signal-safe, run.
    unsafe {
        command.pre_exec(move || {
            libc::signal(libc::SIGXFSZ, libc::SIG_IGN);
            let cap: libc::rlimit = libc::rlimit { rlim_cur: limit, rlim_max: limit };
            if libc::setrlimit(libc::RLIMIT_FSIZE, &cap) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            return Ok(());
        });
    }
    return command.status().expect("the child test starts");
}

/// What:
///  The number of descriptors this process holds now,
///  the directory handle of the listing itself not counted.
/// Why:
///   A limit is set from the count a control read,
///  so the control sweeps exactly the room it means to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const openDescriptors = () => readdirSync('/proc/self/fd').length - 1;
/// ```
pub(crate) fn open_descriptors() -> usize {
    let listed: usize = std::fs::read_dir("/proc/self/fd").expect("the descriptor listing").count();
    return listed - 1;
}

/// What:
///  Sets the soft descriptor limit and answers the one it replaced,
///  so the caller restores it.
/// Why:
///   A descriptor limit is the privilege-free way to make an open fail at `EMFILE`,
///  which is how a control makes the reads of `/proc` a look needs fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = getrlimit('nofile'); setrlimit('nofile', { soft }); return before;
/// ```
pub(crate) fn set_soft_limit(soft: u64) -> rustix::process::Rlimit {
    let before: rustix::process::Rlimit = rustix::process::getrlimit(rustix::process::Resource::Nofile);
    let lowered: rustix::process::Rlimit = rustix::process::Rlimit { current: Some(soft), maximum: before.maximum };
    rustix::process::setrlimit(rustix::process::Resource::Nofile, lowered).expect("a soft limit below the hard one can be set");
    return before;
}

/// Puts a soft descriptor limit back to the value `set_soft_limit` answered.
pub(crate) fn restore_limit(before: rustix::process::Rlimit) {
    rustix::process::setrlimit(rustix::process::Resource::Nofile, before).expect("the limit can be raised back to the hard one");
}

/// What:
///  Opens descriptors until `free` of them are left under the limit,
///  and answers the ones held.
/// Why:
///   The caller drops them when the step under test is done,
///  so exactly `free` opens work for as long as that step runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = openUntilFree(free); // dropped when the step ends
/// ```
pub(crate) fn leave_free(free: usize) -> Vec<std::fs::File> {
    let mut held: Vec<std::fs::File> = Vec::new();
    loop {
        match std::fs::File::open("/dev/null") {
            Ok(file) => held.push(file),
            Err(error) => {
                assert_eq!(error.raw_os_error(), Some(libc::EMFILE), "only the limit stops the opens: {error}");
                break;
            }
        }
    }
    // Give back `free` of them, so that exactly that many opens still work.
    for _ in 0..free {
        held.pop();
    }
    return held;
}

/// Controls for the fixture helpers above.
#[path = "test_support_tests.rs"]
mod tests;

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
pub(crate) fn dead_copy_owner() -> crate::worktree_copy_owner::CopyOwner {
    use std::process::Stdio;
    // Owned, so an assertion between the start and the kill does not leave the sleeper for a minute.
    let mut child: std::process::Child = spawn_owned(
        Command::new("sleep")
            .arg("60")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null()),
    )
    .expect("sleep starts");
    let pid: i64 = i64::from(child.id());
    let birth: String = crate::process_identity::process_birth_identity(pid)
        .expect("identity readable")
        .expect("the child runs");
    child.kill().expect("child is killed");
    child.wait().expect("child is reaped");
    return crate::worktree_copy_owner::CopyOwner {
        pid,
        birth_identity: birth,
        pid_namespace: crate::pid_namespace::current_pid_namespace(),
    };
}
