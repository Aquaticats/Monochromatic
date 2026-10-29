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
use std::process::{Command, Output, Stdio};

/// The native executable Cargo built for this test run.
pub const WRAPPER: &str = env!("CARGO_BIN_EXE_cli-git-native");

/// The real Git 2.56.0 executable installed in the verification image.
pub const REAL_GIT: &str = "/usr/bin/git";

/// Seconds after which a wrapped command is killed;
///  real commands here finish in milliseconds.
/// The bound is short so a mutant that loops fails every affected control quickly.
pub const TIME_BOUND_SECONDS: &str = "5";

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

/// One disposable directory with a `bin/git` link to a private copy of the wrapper.
pub struct Fixture {
    /// Canonical fixture root.
    pub root: PathBuf,
    /// The private wrapper copy every link and hard link in this fixture points at.
    pub wrapper: PathBuf,
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
        settle_fixture(self.root.as_path(), std::env::var_os(KEEP_FIXTURES_VARIABLE).is_some());
    }
}

/// Create a fixture:
///  `<root>/wrapper/cli-git-native` (copy) and `<root>/bin/git` (symbolic link to it).
pub fn fixture(name: &str) -> Fixture {
    let created: PathBuf = fixture_root().join(format!("native-binary-{}-{name}", std::process::id()));
    if created.exists() {
        std::fs::remove_dir_all(&created).expect("remove stale fixture");
    }
    std::fs::create_dir(&created).expect("fresh fixture");
    let root: PathBuf = std::fs::canonicalize(&created).expect("canonical fixture");
    for directory in ["wrapper", "bin", "home"] {
        std::fs::create_dir(root.join(directory)).expect("fixture directory");
    }
    let wrapper: PathBuf = root.join("wrapper/cli-git-native");
    copy_executable(Path::new(WRAPPER), wrapper.as_path());
    std::os::unix::fs::symlink(&wrapper, root.join("bin/git")).expect("git-named link");
    return Fixture { root, wrapper };
}

/// Remove only the fixture directory a control created.
pub fn remove(fixture: &Fixture) {
    std::fs::remove_dir_all(&fixture.root).expect("remove only the fixture");
}

/// Apply the hermetic environment every wrapped and direct run shares.
fn hermetic(command: &mut Command, fixture: &Fixture, path: &OsStr) {
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
        .current_dir(&fixture.root);
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
    let mut command: Command = Command::new("/usr/bin/timeout");
    command.arg("--signal=KILL").arg(seconds).arg(program);
    hermetic(&mut command, fixture, path);
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

/// What a command run under the shared bound asks the wrapper to do, read the way the wrapper reads it.
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
///  The shared bound is for single Git commands. A landed commit starts 40 Git processes and makes 36
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

/// Run a command to completion with every byte of its output kept,
///  feeding `input` to its standard input.
/// # Panics
///  When the command is a commit transaction or a manual page under the shared bound; see `shared_bound_misuse`.
pub fn observe_raw(command: &mut Command, input: &[u8]) -> Observed {
    match shared_bound_misuse(command) {
        Some(SharedBoundMisuse::CommitTransaction) => panic!(
            "a commit under the shared {TIME_BOUND_SECONDS}-second bound: it starts 40 Git processes and makes 36 \
             sync calls, so run it with `run_commit` (the {}-second bound of binary_commit_tests.rs)",
            super::commit::COMMIT_BOUND_SECONDS
        ),
        Some(SharedBoundMisuse::ManualPage) => panic!(
            "a `--help` under the shared {TIME_BOUND_SECONDS}-second bound renders a manual page through man and groff \
             (9 process starts); put an executable `man` stub in the fixture's bin directory first (`stub_manual_viewer`)"
        ),
        None => {}
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("process starts");
    // A child that exits without reading its input closes the pipe; that is not a test failure.
    let _ = child.stdin.take().expect("piped stdin").write_all(input);
    let output: Output = child.wait_with_output().expect("process finishes");
    assert_ne!(
        output.status.signal(),
        Some(TIMED_OUT_SIGNAL),
        // Built only on failure: a bound hit under host load must name the command that stalled.
        "the command hit the {TIME_BOUND_SECONDS}-second bound: {:?} in {:?}",
        command.get_args().collect::<Vec<&OsStr>>(),
        command.get_current_dir()
    );
    return Observed {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    };
}

/// Define a `pre-commit` hook in the repository's configuration that runs `script`.
///  The script sits in the fixture root, outside the worktree, so it never shows up as a change.
///  Traditional hooks (scripts in the hooks directory) are unsupported, so controls of hook behavior use this.
pub fn configure_pre_commit_hook(fixture: &Fixture, repo: &Path, script: &[u8]) {
    let path: PathBuf = fixture.root.join("pre-commit-hook-script");
    executable(path.as_path(), script);
    git(fixture, repo, &["config", "hook.control.event", "pre-commit"]);
    git(fixture, repo, &["config", "hook.control.command", path.to_str().expect("UTF-8")]);
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
#[path = "fixture_executable.rs"]
mod fixture_executable;

pub use fixture_executable::executable;

/// What:
///  Copy an executable through a child `cp` process,
///  keeping its mode.
/// Why:
///   A copy made by this process would be open for writing here,
///  with the same
///       "Text file busy" window as `executable` describes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync('cp', ['--', from, to]);
/// ```
pub fn copy_executable(from: &Path, to: &Path) {
    let copied: bool = Command::new("cp")
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
///  Publish a transaction in a repository's registry,
///  owned by this test process with its
///       real birth identity (live) or with an identity no process has (dead).
/// Why:
///   The wrapper decides liveness from the record,
///  so this process stands in for an owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await publishTransactionDirectory({ root, transactionId, owner: await createTransactionOwnerRecord(...) });
/// ```
pub fn registered_transaction(repo: &Path, live: bool) -> PathBuf {
    let registry: PathBuf = repo.join(".git/cli-git-transactions");
    git_policy_cli::transaction_registry::ensure_transaction_root(registry.as_path())
        .expect("registry");
    let mut owner: git_policy_cli::transaction_owner::TransactionOwner =
        git_policy_cli::transaction_owner::current_transaction_owner(
            TRANSACTION_ID,
            "2026-10-06T00:00:00.000Z",
        )
        .expect("owner");
    if !live {
        owner.owner_identity = String::from("linux:1");
    }
    git_policy_cli::transaction_registry::publish_transaction_directory(
        registry.as_path(),
        TRANSACTION_ID,
        git_policy_cli::transaction_owner::encode_transaction_owner(&owner).as_bytes(),
    )
    .expect("published");
    return registry;
}
