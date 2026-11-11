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

/// How long the run may show no sign of work before the watchdog ends it.
/// A run that is moving is left alone,
/// however long it takes:
/// the slowest test of this crate's families spends most of its time waiting for the disk
/// (measured 2026-11-08 on this host:
/// 43 of 47 s of one test alone,
/// and 64 of 69 s of another while its family ran,
/// in the btrfs transaction and log commit waits),
/// so a wall clock ended tests that were working.
/// The longest stretch any process of those families showed nothing was 8.0 s of a test and the `git` it ran together,
/// and 0.0 s for every test that the disk made slow,
/// so this window is 15 times the longest quiet stretch measured
/// (`doc/troubleshooting/git-policy-cli-unit-test-watchdog-judges-progress.md`).
const QUIET_WINDOW: std::time::Duration = std::time::Duration::from_secs(120);

/// Ends the test process once the run has shown no sign of work for `QUIET_WINDOW`.
/// A run that is stuck shows nothing:
/// no task able to run,
/// none waiting for the disk,
/// no processor time,
/// no bytes moved,
/// and no process or task started or ended (`run_progress.rs`).
/// Without it a mutant that leaves a lock held or ignores a deadline hangs the run until an outside timeout,
/// which the mutation tool records as a timeout instead of a failed test.
/// The first fixture of the run arms it;
/// later calls do nothing.
pub(crate) fn arm_watchdog() {
    static ARMED: std::sync::Once = std::sync::Once::new();
    ARMED.call_once(|| {
        // A process that is given no thread (a control of a site that starts threads) runs without the watchdog; its parent has its own.
        let _armed: Option<std::thread::JoinHandle<()>> = crate::run_progress::watch(QUIET_WINDOW);
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

/// What:
///  The directory this process's fixtures are created in:
///  `test-fixtures/process-<pid>` beside the build output of the running test executable
/// (`<target>/debug/deps/<test>` has the target directory three levels up),
/// never `/tmp`.
/// Why:
///   The build output is shared by every checkout that builds into the same target directory,
/// so `test-fixtures` itself is
///  shared by every checkout and every test process of this host.
/// The names under it that are not the fixtures' own (the scratch home the product keeps
///  its per-user state in,
/// the marker `refused-push-ran`,
/// the repositories `repository(root, name)` makes beside the fixtures)
///  were one fixed
///  name for all of them,
/// so two runs at once shared them:
/// two checkouts started at the same moment reported three subvolume controls failing in
///  ten to twenty seconds while the same three passed alone.
/// A directory of the process's own makes every fixed name below it the process's,
/// and nothing else changes:
///  fixtures are still removed by their own test and kept for inspection on request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = join(target, 'test-fixtures', `process-${process.pid}`);
/// ```
pub(crate) fn fixture_root() -> PathBuf {
    let base: PathBuf = std::env::var_os(FIXTURE_ROOT_VARIABLE).map_or_else(
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
    let root: PathBuf = base.join(format!("process-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("fixture root");
    return root;
}

/// What:
///  Give this user back the read,
/// write and search permission on every directory of a tree that has lost it,
///  without following a symbolic link.
/// Why:
///   A control that writes a directory its owner may not write (`0500`,
/// `0555`,
/// `0000`) proves what the product does with one,
/// and removing that directory afterwards needs read to list it and write and search to unlink what it holds,
/// so a fixture left in that state outlived the test that made it
/// (issue 6282:
/// fixtures named `rollback-read-only`,
/// `remove-old-read-only`,
/// `migrate-unreadable-dir` and `sync-unreadable-dir`),
/// and a build-tree sweeper could not delete the tree they sat in.
/// Only directories are changed,
/// only ones this user owns,
/// and only the owner's three bits:
/// a file with no permissions of its own is unlinked from a writable directory,
/// and a group or other-user bit a control set stays as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreOwnerAccess(root: string): void // lstat each entry, chmod OWNER_RWX on the directories this user owns
/// ```
#[cfg(unix)]
pub(crate) fn restore_owner_access(root: &Path) {
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    let own: u32 = unsafe { libc::geteuid() };
    let owner_bits: u32 = 0o700;
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(metadata) = std::fs::symlink_metadata(&directory) else {
            continue;
        };
        if !metadata.is_dir() {
            continue;
        }
        let mode: u32 = metadata.mode() & 0o7777;
        if metadata.uid() == own && mode & owner_bits != owner_bits {
            let _widened = std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(mode | owner_bits));
        }
        // The directory may have been closed to this user until a moment ago, so it is listed after the mode is restored.
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            let Ok(found) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if found.is_dir() {
                stack.push(path);
            }
        }
    }
}

/// What:
///  On a platform without Unix permissions there is nothing to give back.
/// Why:
///   The teardown calls this on every platform.
#[cfg(not(unix))]
pub(crate) fn restore_owner_access(_root: &Path) {}

/// What:
///  Remove a fixture tree unless the person asked to keep it,
/// trying again for up to five seconds.
/// Why:
///   A landed commit may leave a maintenance child finishing for a moment;
/// a tree already gone is the explicit
///  `remove` having run first,
/// not a failure.
/// A control that made a directory read-only inside its fixture is answered here,
/// so the fixture goes however the control ended,
/// panic included.
pub(crate) fn settle_fixture(root: &Path, keep: bool) {
    if keep {
        return;
    }
    restore_owner_access(root);
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
        // A fixture an earlier run was killed in the middle of may hold a directory it closed, so the permission is given back first.
        restore_owner_access(&root);
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

/// Remove only the fixture directory a test created,
/// giving this user back the permission on any directory of it that a test closed.
pub(crate) fn remove(root: &Path) {
    restore_owner_access(root);
    std::fs::remove_dir_all(root).expect("remove only the fixture");
}

/// What:
///  Shows each notice to a buffer and answers the words it wrote,
/// as a command's standard error would have received them
///  without the newline the show adds.
/// Why:
///   A record goes only once its words are written out,
/// so a control reads them through the step the person's terminal
///  receives them by,
/// and a record whose words could not be written stays as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// notices.map((notice) => { const out = []; notice.show(out); return out.join('').replace(/\n$/, '') })
/// ```
pub(crate) fn shown_notices(notices: Vec<crate::push_failure_notice::Notice>) -> Vec<String> {
    let mut texts: Vec<String> = Vec::new();
    for notice in notices {
        let mut written: Vec<u8> = Vec::new();
        notice.show(&mut written).expect("a buffer takes the words");
        let text: String = String::from_utf8(written).expect("the words are UTF-8");
        texts.push(text.strip_suffix('\n').map(str::to_owned).unwrap_or(text));
    }
    return texts;
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
#[path = "fixture_git_commands.rs"]
pub(crate) mod fixture_git_commands;

/// Run real Git in a fixture with no system or global configuration and a fixed identity.
pub(crate) use fixture_git_commands::{git, git_output};

#[path = "child_test_process.rs"]
pub(crate) mod child_test_process;

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

/// The bounds a control's wait and its timed step give up under,
/// in a file of their own.
#[path = "test_support_wait_bounds.rs"]
mod test_support_wait_bounds;

pub(crate) use test_support_wait_bounds::{STILL_WORKING_WINDOW, WAIT_LOOK_EVERY, WaitingFor, finished_within, finished_within_at, wait_until, wait_until_for};

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
    // What the step under test sees must be exactly that room:
    // an open one past it fails with `EMFILE`,
    // so a limit that was set from a stale count is reported here and not as a look that answered.
    let mut probes: Vec<std::fs::File> = Vec::new();
    for _ in 0..free {
        probes.push(std::fs::File::open("/dev/null").expect("the room that was given back is usable"));
    }
    let refused: std::io::Error = std::fs::File::open("/dev/null").expect_err("the limit refuses the open after the room it leaves");
    assert_eq!(refused.raw_os_error(), Some(libc::EMFILE), "only the limit stops the open after the room it leaves: {refused}");
    drop(probes);
    return held;
}

/// Controls for the fixture helpers above.
#[path = "test_support_tests.rs"]
mod tests;

/// The owner of a process that really started and was then killed,
/// a fixture a control builds instead of waiting for a real crash.
pub(crate) use crate::test_support_dead_copy_owner::dead_copy_owner;
