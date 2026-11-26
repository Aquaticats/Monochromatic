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
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
    Output,
};

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
/// and no process or task started (`run_progress.rs`).
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
///  The fixture lifecycle both this target and the binary controls run on,
///  in one file both include by `#[path]` so a fix lands in both.
/// Why:
///  The two targets each kept a copy of these helpers and the copies drifted apart
/// (`doc/debt-ledger-2026-11-25.md`, item 11),
/// so every helper below is written once in `fixture_support_shared_by_both_test_targets.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // shared/fixture-support.ts
/// ```
#[path = "fixture_support_shared_by_both_test_targets.rs"]
mod fixture_support_shared_by_both_test_targets;

/// The shared helpers, named here so every `crate::test_support::name` path stays the same.
pub(crate) use self::fixture_support_shared_by_both_test_targets::{
    FIXTURE_ROOT_VARIABLE,
    KEEP_FIXTURES_VARIABLE,
    fixture_root,
    forbid_core_dumps,
    made_fixture_folder,
    restore_owner_access,
    settle_fixture,
    system_temporary_directories,
    this_test_binary,
};
/// The fixtures one test thread created,
/// removed when the thread ends however the test ended.
/// Why:
///  The helper process groups are owned by `test_process_owner.rs`,
///  so this registry holds only the folders.
struct ThreadFixtures {
    /// Roots to remove.
    roots: std::cell::RefCell<Vec<PathBuf>>,
}

/// A panic unwinds the test thread,
/// which runs this.
impl Drop for ThreadFixtures {
    fn drop(&mut self) {
        // The helpers die first, so no process is left in a directory that is about to be removed.
        super::test_process_owner::end_owned_groups();
        let keep: bool = std::env::var_os(KEEP_FIXTURES_VARIABLE).is_some();
        for root in self.roots.borrow().iter() {
            end_processes_working_in(root.as_path());
            settle_fixture(root.as_path(), keep);
        }
    }
}

thread_local! {
    /// The fixtures of the running test thread.
    static THREAD_FIXTURES: ThreadFixtures = const { ThreadFixtures { roots: std::cell::RefCell::new(Vec::new()) } };
}

/// What:
///  The owner of every helper process a control starts,
///  named here so every `crate::test_support::name` path stays the same.
///  The owner is a file of its own because the binary controls' crate and the candidate consumer's crate include it by
///  path, and the library's test-only modules are invisible to them.
/// Why:
///  One shape starts such a helper, so one file decides when it dies.
///  `own_group` is not named here: the library's own controls all start their helper through `spawn_owned`, and a target
///  that starts one without it names the owner directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { spawnOwned } from './test_process_owner';
/// ```
pub(crate) use super::test_process_owner::spawn_owned;

/// What:
///  Start a helper process whose output nobody reads, with its three standard streams on the null device.
/// Why:
///  A stand-in that `exec`s a long sleep inherits this run's output pipe and holds it open,
///  so a caller that pipes the run's output waits for a process the run has already lost (issue 6463);
///  a program whose output nobody reads gives the pipe back by holding none.
/// Gotcha:
///  This overrides whatever streams the caller set,
///  so a control that reads its child's output starts it with `spawn_owned` and says which stream it pipes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = spawnOwned(command, { stdio: 'ignore' });
/// ```
pub(crate) fn spawn_owned_quiet(command: &mut Command) -> std::io::Result<Child> {
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    return spawn_owned(command);
}

/// What:
///  Make one empty fixture directory for this call of this test,
///  under a prefix that keeps the fixtures of different support modules apart.
/// Why:
///   Every call names a folder of its own,
/// so two tests that pass the same name never share one,
///  and neither removes the other's live folder:
/// the folder carries the process's number and the number this call took,
/// and the name the test writes.
/// A folder of a run that was killed may carry this process's number and this call's number,
///  so one that exists is removed first.
pub(crate) fn fixture_with_prefix(
    prefix: &str,
    name: &str,
) -> PathBuf {
    arm_watchdog();
    forbid_core_dumps();
    let root: PathBuf = made_fixture_folder(fixture_root().as_path(), prefix, name);
    THREAD_FIXTURES.with(|fixtures| {
        fixtures.roots.borrow_mut().push(root.clone());
    });
    return root;
}

/// Create one empty fixture directory for this call of this test.
pub(crate) fn fixture(name: &str) -> PathBuf {
    return fixture_with_prefix("native-unit", name);
}

/// What:
///  An empty stand-in process table for one fixture:
///  a folder under the fixture's root that holds no process,
///  which a removal reads in place of this machine's `/proc`.
/// Why:
///   A removal of a whole worktree reads every process of the account to see whether one works in it,
///  and a control that read the machine's own table would depend on every program every other session runs:
///  a program started after the fixture's worktree was made keeps that worktree,
///  and issue 6475's container gate refused a removal over another test's process.
///  A control about a process working inside the worktree writes the entry it needs here instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = anEmptyProcessTable(fixtureRoot); // a folder that holds no process
/// ```
pub(crate) fn an_empty_process_table(root: &Path) -> PathBuf {
    let table: PathBuf = root.join("proc");
    // A table that cannot be made is no table at all, so the control stops here rather than reading the machine's.
    std::fs::create_dir_all(table.as_path()).expect("an empty process table");
    return table;
}

/// What:
///  Take a directory this test thread made under the thread's own cleanup,
/// so it is removed when the thread ends however the test ended.
/// Why:
///   A fixture root under the build directory is deep,
/// and a socket name cannot be longer than a socket address holds,
/// so a control of the wrapper's sockets keeps its runtime directory somewhere short and asks for this cleanup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// trackForThisThread(runtimeDirectory) // removed with the thread's fixtures
/// ```
pub(crate) fn track_for_this_thread(root: &Path) {
    THREAD_FIXTURES.with(|fixtures| {
        fixtures.roots.borrow_mut().push(root.to_path_buf());
    });
}

/// The folder chooser and the runtime directory built to a length inside it,
/// named here so every `crate::test_support::name` path stays the same.
pub(crate) use crate::runtime_directory_folders::*;

/// What:
///  Move a path's modification time two minutes into the past without following a symbolic link.
/// Why:
///   An index entry whose file is as new as the index file proves nothing by its stat fields (Git hashes it again),
/// so a
///  control that needs a provably clean worktree ages its files before the index records them,
/// and one that needs an entry the index cannot vouch for ages the index file instead.
pub(crate) fn age(path: &Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString =
        std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    let past: libc::time_t = (std::time::SystemTime::now() - std::time::Duration::from_secs(120))
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("after the epoch")
        .as_secs() as libc::time_t;
    let times: [libc::timespec; 2] = [
        libc::timespec {
            tv_sec: past,
            tv_nsec: 0,
        },
        libc::timespec {
            tv_sec: past,
            tv_nsec: 0,
        },
    ];
    // SAFETY: `name` is a NUL-terminated string that outlives the call and `times` holds the two timestamps the call reads.
    let result: libc::c_int = unsafe {
        libc::utimensat(
            libc::AT_FDCWD,
            name.as_ptr(),
            times.as_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
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
pub(crate) fn repository(
    parent: &Path,
    name: &str,
) -> PathBuf {
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
pub(crate) use fixture_git_commands::{
    FIXTURE_GIT_BOUND,
    QueryAnswer,
    QueryEnding,
    git,
    git_output,
    real_git_within,
};

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

pub(crate) use self::fixture_notice_sinks::{
    ActionSink,
    SharedSink,
};

/// The shell condition that holds while the test that started a fixture script is still there.
#[cfg(unix)]
#[path = "test_alive.rs"]
mod test_alive;

#[cfg(unix)]
pub(crate) use test_alive::{
    end_processes_working_in,
    while_test_lives,
};

/// The bounds a control's wait and its timed step give up under,
/// in a file of their own.
#[path = "test_support_wait_bounds.rs"]
mod test_support_wait_bounds;

pub(crate) use test_support_wait_bounds::{
    STILL_WORKING_WINDOW,
    WAIT_LOOK_EVERY,
    WaitingFor,
    finished_within,
    finished_within_at,
    wait_until,
    wait_until_for,
};

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
pub(crate) fn git_text<S: AsRef<OsStr>>(
    directory: &Path,
    arguments: &[S],
) -> String {
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
    let listed: usize = std::fs::read_dir("/proc/self/fd")
        .expect("the descriptor listing")
        .count();
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
    let before: rustix::process::Rlimit =
        rustix::process::getrlimit(rustix::process::Resource::Nofile);
    let lowered: rustix::process::Rlimit = rustix::process::Rlimit {
        current: Some(soft),
        maximum: before.maximum,
    };
    rustix::process::setrlimit(rustix::process::Resource::Nofile, lowered)
        .expect("a soft limit below the hard one can be set");
    return before;
}

/// Puts a soft descriptor limit back to the value `set_soft_limit` answered.
pub(crate) fn restore_limit(before: rustix::process::Rlimit) {
    rustix::process::setrlimit(rustix::process::Resource::Nofile, before)
        .expect("the limit can be raised back to the hard one");
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
                assert_eq!(
                    error.raw_os_error(),
                    Some(libc::EMFILE),
                    "only the limit stops the opens: {error}"
                );
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
        probes.push(
            std::fs::File::open("/dev/null").expect("the room that was given back is usable"),
        );
    }
    let refused: std::io::Error = std::fs::File::open("/dev/null")
        .expect_err("the limit refuses the open after the room it leaves");
    assert_eq!(
        refused.raw_os_error(),
        Some(libc::EMFILE),
        "only the limit stops the open after the room it leaves: {refused}"
    );
    drop(probes);
    return held;
}

/// Controls for the fixture helpers above.
#[path = "test_support_tests.rs"]
mod tests;

/// The owner of a process that really started and was then killed,
/// a fixture a control builds instead of waiting for a real crash.
pub(crate) use crate::test_support_dead_copy_owner::dead_copy_owner;
