//! What:
//!  Disposable experiments against the installed Git (`/usr/bin/git`, 2.56.0 on the gate) for the protocol's Git-facing claims, run on both
//!  reference backends (`files` and `reftable`): an `update-ref --stdin` transaction over a real FIFO built as the design's pipe bootstrap
//!  specifies, the hook phases of the `reference-transaction` hook and their absence under an empty hooks path, the diagnostics of a refused
//!  prepare, and the push exclusion and fixed-object pushes.
//! Why:
//!  The design states the FIFO bootstrap "as a specification to check by experiment" and lists "disposable Git experiments: actual FIFO,
//!  hook phases, diagnostics and backend behavior" and "push exclusion and frozen updates against actual Git" in its acceptance target.
//!  A model of the pipe says what the protocol needs from Git; these tests say what Git does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const backend of ['files', 'reftable']) test(`update-ref --stdin over a FIFO commits only on commit (${backend})`, ...);
//! ```

use crate::model_fixture::{REAL_GIT, git, git_output, git_text, scratch};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// The reference storage formats the wrapper's repositories may use.
const BACKENDS: [&str; 2] = ["files", "reftable"];

/// The longest a test waits for Git's answer on a pipe; a local Git answers at once.
const ANSWER_WAIT: Duration = Duration::from_secs(10);

/// How long a test waits to show that nothing happened (the transaction did not end by itself).
const QUIET_WAIT: Duration = Duration::from_millis(250);

/// The all-zero object name: "the ref must not exist".
const NO_OBJECT: &str = "0000000000000000000000000000000000000000";

/// The environment Git runs with in these fixtures, as `model_fixture::git_output` sets it, plus `LC_ALL=C` as the design specifies for
/// the transaction child.
fn hermetic(command: &mut Command) {
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid");
}

/// A repository on `backend` with `refs/heads/main` at `first` and a second commit `second` that no ref names.
struct Repository {
    path: PathBuf,
    first: String,
    second: String,
}

fn repository(label: &str, backend: &str) -> Repository {
    let path: PathBuf = scratch(&format!("git-{}-{}", label, backend));
    git(&path, &["init", "--quiet", &format!("--ref-format={}", backend), "--initial-branch=main"]);
    git(&path, &["commit", "--quiet", "--allow-empty", "--message", "first"]);
    let first: String = git_text(&path, &["rev-parse", "HEAD"]).trim().to_string();
    git(&path, &["commit", "--quiet", "--allow-empty", "--message", "second"]);
    let second: String = git_text(&path, &["rev-parse", "HEAD"]).trim().to_string();
    git(&path, &["update-ref", "refs/heads/main", &first]);
    return Repository { path, first, second };
}

impl Repository {
    /// The object a ref names, or `None`.
    fn resolve(&self, name: &str) -> Option<String> {
        let output = git_output(&self.path, &["rev-parse", "--verify", "--quiet", name]);
        return output.status.success().then(|| { return String::from_utf8_lossy(&output.stdout).trim().to_string(); });
    }

    /// The instructions that move `refs/heads/main` from the first commit to the second.
    fn move_main(&self) -> String {
        return format!("update refs/heads/main {} {}\n", self.second, self.first);
    }
}

/// One `update-ref --stdin` child reading a FIFO, started as the design's pipe bootstrap says: read end opened `O_NONBLOCK`, keeper write
/// end opened `O_NONBLOCK` (it succeeds because a reader exists), `O_NONBLOCK` cleared on the read end, the read end handed over as
/// standard input and the launcher's copy closed after the spawn.
struct Transaction {
    child: Child,
    keeper: Option<File>,
    fifo: PathBuf,
    out: PathBuf,
    err: PathBuf,
}

fn open_transaction(repository: &Repository, extra: &[&str]) -> Transaction {
    return open_transaction_with(repository, extra, &[]);
}

/// As `open_transaction`, with `options` given to `update-ref` itself (`--no-deref`, say).
fn open_transaction_with(repository: &Repository, extra: &[&str], options: &[&str]) -> Transaction {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::io::AsRawFd;
    let fifo: PathBuf = scratch("git-fifo").join("in");
    let name: CString = CString::new(fifo.as_os_str().as_bytes()).expect("a path without NUL");
    // SAFETY: `name` is a NUL-terminated path that lives through the call.
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0, "mkfifo");
    let reader: File = OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK).open(&fifo).expect("the read end opens with no writer");
    let keeper: File = OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(&fifo).expect("the keeper opens because a reader exists");
    // SAFETY: `fcntl` takes the descriptor of the live `reader` and plain integers.
    unsafe {
        let flags: libc::c_int = libc::fcntl(reader.as_raw_fd(), libc::F_GETFL);
        assert!(flags >= 0);
        assert_eq!(libc::fcntl(reader.as_raw_fd(), libc::F_SETFL, flags & !libc::O_NONBLOCK), 0);
    }
    let out: PathBuf = fifo.with_file_name("out");
    let err: PathBuf = fifo.with_file_name("err");
    let mut command: Command = Command::new(REAL_GIT);
    hermetic(&mut command);
    command
        .current_dir(&repository.path)
        .args(extra)
        .args(["update-ref"])
        .args(options)
        .args(["--stdin"])
        .stdin(Stdio::from(reader))
        .stdout(File::create(&out).expect("out"))
        .stderr(File::create(&err).expect("err"));
    let child: Child = command.spawn().expect("spawn update-ref");
    // Dropping the command closes the launcher's copy of the read end.
    drop(command);
    return Transaction { child, keeper: Some(keeper), fifo, out, err };
}

impl Transaction {
    fn send(&mut self, text: &str) {
        let keeper: &mut File = self.keeper.as_mut().expect("the keeper is open");
        keeper.write_all(text.as_bytes()).expect("write to the FIFO");
    }

    fn close_keeper(&mut self) {
        self.keeper = None;
    }

    /// Waits until standard output holds `text`.
    fn wait_for_answer(&mut self, text: &str) {
        let deadline: Instant = Instant::now() + ANSWER_WAIT;
        loop {
            let seen: String = fs::read_to_string(&self.out).unwrap_or_default();
            if seen.contains(text) {
                return;
            }
            if Instant::now() >= deadline {
                panic!("no `{}` within {:?}; stdout {:?}, stderr {:?}", text, ANSWER_WAIT, seen, fs::read_to_string(&self.err).unwrap_or_default());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Whether the child is still running now.
    fn running(&mut self) -> bool {
        return self.child.try_wait().expect("try_wait").is_none();
    }

    /// Waits for the child to end; a child that outlives the bound is killed and the test fails.
    fn finish(&mut self) -> ExitStatus {
        let deadline: Instant = Instant::now() + ANSWER_WAIT;
        loop {
            if let Some(status) = self.child.try_wait().expect("try_wait") {
                return status;
            }
            if Instant::now() >= deadline {
                let _ = self.child.kill();
                let _ = self.child.wait();
                panic!("update-ref did not end within {:?}", ANSWER_WAIT);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn stdout(&self) -> String {
        return fs::read_to_string(&self.out).unwrap_or_default();
    }

    fn stderr(&self) -> String {
        return fs::read_to_string(&self.err).unwrap_or_default();
    }
}

/// With the keeper open, Git reads no end of input between instructions: after `prepare` it answers `prepare: ok`, holds the ref at the old
/// value through a pause, and `commit` then moves it; closing the keeper ends the child with success. A second writer can open the FIFO
/// while the reader lives.
#[test]
fn a_fifo_transaction_prepares_holds_through_a_pause_and_commits_on_commit() {
    for backend in BACKENDS {
        let repository: Repository = repository("fifo-commit", backend);
        let mut transaction: Transaction = open_transaction(&repository, &[]);
        transaction.send("start\n");
        transaction.wait_for_answer("start: ok");
        transaction.send(&repository.move_main());
        transaction.send("prepare\n");
        transaction.wait_for_answer("prepare: ok");
        std::thread::sleep(QUIET_WAIT);
        assert!(transaction.running(), "{}: the child waits for the next instruction and reads no end of input", backend);
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.first.as_str()), "{}: nothing moved before commit", backend);
        let other: File = OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(&transaction.fifo).expect("a second writer opens while the reader lives");
        drop(other);
        transaction.send("commit\n");
        transaction.wait_for_answer("commit: ok");
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.second.as_str()), "{}", backend);
        transaction.close_keeper();
        let status: ExitStatus = transaction.finish();
        assert!(status.success(), "{}: {:?} {}", backend, status, transaction.stderr());
    }
}

/// `abort` after `prepare` answers `abort: ok` and leaves the ref where it was.
#[test]
fn a_fifo_abort_after_prepare_leaves_the_ref_alone() {
    for backend in BACKENDS {
        let repository: Repository = repository("fifo-abort", backend);
        let mut transaction: Transaction = open_transaction(&repository, &[]);
        transaction.send("start\n");
        transaction.send(&repository.move_main());
        transaction.send("prepare\n");
        transaction.wait_for_answer("prepare: ok");
        transaction.send("abort\n");
        transaction.wait_for_answer("abort: ok");
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.first.as_str()), "{}", backend);
        transaction.close_keeper();
        let status: ExitStatus = transaction.finish();
        assert!(status.success(), "{}: {:?} {}", backend, status, transaction.stderr());
    }
}

/// End of input after `prepare`, with no commit: the closing of the only writer ends the child and the prepared transaction does not
/// commit; the ref stays and no lock file remains (files backend).
#[test]
fn end_of_input_after_prepare_aborts_the_transaction() {
    for backend in BACKENDS {
        let repository: Repository = repository("fifo-eof", backend);
        let mut transaction: Transaction = open_transaction(&repository, &[]);
        transaction.send("start\n");
        transaction.send(&repository.move_main());
        transaction.send("prepare\n");
        transaction.wait_for_answer("prepare: ok");
        transaction.close_keeper();
        let _status: ExitStatus = transaction.finish();
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.first.as_str()), "{}: the prepared transaction did not commit", backend);
        assert!(!repository.path.join(".git/refs/heads/main.lock").exists(), "{}: no lock file remains", backend);
        assert!(!transaction.stdout().contains("commit: ok"), "{}: {}", backend, transaction.stdout());
    }
}

/// A `commit` written before the last writer closes is read before end of input: all four instructions and an immediate close commit.
#[test]
fn a_commit_buffered_before_the_writers_close_is_executed() {
    for backend in BACKENDS {
        let repository: Repository = repository("fifo-buffered", backend);
        let mut transaction: Transaction = open_transaction(&repository, &[]);
        transaction.send(&format!("start\n{}prepare\ncommit\n", repository.move_main()));
        transaction.close_keeper();
        let status: ExitStatus = transaction.finish();
        assert!(status.success(), "{}: {:?} {}", backend, status, transaction.stderr());
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.second.as_str()), "{}", backend);
    }
}

/// A writer that opens the FIFO `O_WRONLY | O_NONBLOCK` after the reader has gone gets `ENXIO` and does not block.
#[test]
fn a_writer_opening_after_the_reader_ended_gets_enxio() {
    for backend in BACKENDS {
        let repository: Repository = repository("fifo-enxio", backend);
        let mut transaction: Transaction = open_transaction(&repository, &[]);
        transaction.close_keeper();
        let _status: ExitStatus = transaction.finish();
        let error = OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(&transaction.fifo).expect_err("no reader remains");
        assert_eq!(error.raw_os_error(), Some(libc::ENXIO), "{}: {}", backend, error);
    }
}

/// Installs a `reference-transaction` hook that appends `<phase> <stdin lines>` to a log, and returns the log's path.
fn install_hook(repository: &Repository, verdict: &str) -> PathBuf {
    let log: PathBuf = repository.path.join("hook.log");
    let script: String = format!("#!/bin/sh\nprintf '%s ' \"$1\" >> '{log}'\ncat | tr '\\n' ';' >> '{log}'\nprintf '\\n' >> '{log}'\n{verdict}\n", log = log.display(), verdict = verdict);
    let hook: PathBuf = repository.path.join(".git/hooks/reference-transaction");
    fs::create_dir_all(hook.parent().expect("hooks directory")).expect("hooks directory");
    fs::write(&hook, script).expect("hook");
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).expect("executable hook");
    return log;
}

/// The phases the hook saw, in order.
fn phases(log: &Path) -> Vec<String> {
    return fs::read_to_string(log).unwrap_or_default().lines().map(|line| { return line.split(' ').next().unwrap_or_default().to_string(); }).collect();
}

/// The hook runs `preparing` and `prepared` at `prepare` and `committed` at `commit`, and `aborted` after those on abort, with the old value, new value and
/// ref name on its input; under an empty `core.hooksPath` it does not run at all (the design's hook-free internal transactions).
#[test]
fn the_reference_transaction_hook_phases_follow_prepare_commit_and_abort_and_an_empty_hooks_path_silences_them() {
    for backend in BACKENDS {
        let committed: Repository = repository("hook-commit", backend);
        let log: PathBuf = install_hook(&committed, "exit 0");
        let mut transaction: Transaction = open_transaction(&committed, &[]);
        transaction.send(&format!("start\n{}prepare\n", committed.move_main()));
        transaction.wait_for_answer("prepare: ok");
        assert_eq!(phases(&log), ["preparing", "prepared"], "{}: Git 2.56.0 runs a preparing phase before prepared", backend);
        transaction.send("commit\n");
        transaction.wait_for_answer("commit: ok");
        assert_eq!(phases(&log), ["preparing", "prepared", "committed"], "{}", backend);
        let text: String = fs::read_to_string(&log).expect("log");
        assert!(text.contains(&format!("{} {} refs/heads/main", committed.first, committed.second)), "{}: the hook's input names old, new and ref: {}", backend, text);
        transaction.close_keeper();
        assert!(transaction.finish().success(), "{}", backend);

        let aborted: Repository = repository("hook-abort", backend);
        let abort_log: PathBuf = install_hook(&aborted, "exit 0");
        let mut second: Transaction = open_transaction(&aborted, &[]);
        second.send(&format!("start\n{}prepare\n", aborted.move_main()));
        second.wait_for_answer("prepare: ok");
        second.send("abort\n");
        second.wait_for_answer("abort: ok");
        assert_eq!(phases(&abort_log), ["preparing", "prepared", "aborted"], "{}", backend);
        second.close_keeper();
        assert!(second.finish().success(), "{}", backend);

        let silent: Repository = repository("hook-off", backend);
        let silent_log: PathBuf = install_hook(&silent, "exit 0");
        let empty: PathBuf = scratch("empty-hooks");
        let mut third: Transaction = open_transaction(&silent, &["-c", &format!("core.hooksPath={}", empty.display())]);
        third.send(&format!("start\n{}prepare\ncommit\n", silent.move_main()));
        third.wait_for_answer("commit: ok");
        assert!(phases(&silent_log).is_empty(), "{}: no hook ran under an empty hooks path", backend);
        assert_eq!(silent.resolve("refs/heads/main").as_deref(), Some(silent.second.as_str()), "{}", backend);
        third.close_keeper();
        assert!(third.finish().success(), "{}", backend);
    }
}

/// A hook that refuses at `prepared` makes `prepare` fail with a diagnostic and a failing exit; the ref stays and the `aborted` phase runs.
#[test]
fn a_hook_refusal_at_the_prepared_phase_fails_the_prepare_and_leaves_the_ref() {
    for backend in BACKENDS {
        let repository: Repository = repository("hook-refuse", backend);
        let log: PathBuf = install_hook(&repository, "[ \"$1\" = prepared ] && exit 1\nexit 0");
        let mut transaction: Transaction = open_transaction(&repository, &[]);
        transaction.send(&format!("start\n{}prepare\n", repository.move_main()));
        let status: ExitStatus = transaction.finish();
        assert!(!status.success(), "{}: a refused prepare is a failing exit", backend);
        assert!(!transaction.stdout().contains("prepare: ok"), "{}: {}", backend, transaction.stdout());
        assert!(transaction.stderr().contains("reference-transaction"), "{}: the diagnostic names the hook: {}", backend, transaction.stderr());
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.first.as_str()), "{}", backend);
        assert_eq!(phases(&log), ["preparing", "prepared", "aborted"], "{}: the refusal at prepared is followed by aborted", backend);
    }
}

/// A stale expected value is refused at `prepare` with a diagnostic that names the ref and both values, a failing exit, and no
/// `prepare: ok`; the ref is unchanged. This is the positive failure evidence the protocol reads.
#[test]
fn a_stale_expected_value_is_refused_at_prepare_with_a_diagnostic_naming_the_ref_and_both_values() {
    for backend in BACKENDS {
        let repository: Repository = repository("diagnostic-stale", backend);
        let mut transaction: Transaction = open_transaction(&repository, &[]);
        transaction.send(&format!("start\nupdate refs/heads/main {} {}\nprepare\n", repository.second, repository.second));
        let status: ExitStatus = transaction.finish();
        assert_eq!(status.code(), Some(128), "{}: {}", backend, transaction.stderr());
        let message: String = transaction.stderr();
        assert!(message.contains("prepare: cannot lock ref 'refs/heads/main'"), "{}: {}", backend, message);
        assert!(message.contains(&format!("is at {} but expected {}", repository.first, repository.second)), "{}: {}", backend, message);
        assert!(!transaction.stdout().contains("prepare: ok"), "{}: {}", backend, transaction.stdout());
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.first.as_str()), "{}", backend);
    }
}

/// A creation that expects the ref to be absent is refused when it exists, with a diagnostic naming the ref.
#[test]
fn creating_a_ref_that_exists_is_refused_with_a_diagnostic_naming_it() {
    for backend in BACKENDS {
        let repository: Repository = repository("diagnostic-exists", backend);
        let mut transaction: Transaction = open_transaction(&repository, &[]);
        transaction.send(&format!("start\nupdate refs/heads/main {} {}\nprepare\n", repository.second, NO_OBJECT));
        let status: ExitStatus = transaction.finish();
        assert_eq!(status.code(), Some(128), "{}: {}", backend, transaction.stderr());
        assert!(transaction.stderr().contains("refs/heads/main"), "{}: {}", backend, transaction.stderr());
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.first.as_str()), "{}", backend);
    }
}

/// Pushes with the given refspecs from `local` to the bare `remote` and requires success.
fn push(local: &Repository, remote: &Path, refspecs: &[&str]) {
    let mut arguments: Vec<&str> = vec!["push", "--quiet", remote.to_str().expect("a UTF-8 path")];
    arguments.extend(refspecs);
    git(&local.path, &arguments);
}

/// The push exclusion: a negative refspec `^refs/cli-git/*` keeps the internal namespace off the remote while the same positive refspecs
/// without it push it (the control); and a push that names an object identifier pushes that object though the local branch moved after the
/// command was composed (the fixed identifiers of a frozen update).
#[test]
fn a_negative_refspec_keeps_the_internal_namespace_off_the_remote_and_a_named_object_is_pushed_as_given() {
    for backend in BACKENDS {
        let local: Repository = repository("push", backend);
        git(&local.path, &["update-ref", "refs/cli-git/instance/withdrawn/id", &local.first]);
        let positive: [&str; 2] = ["refs/heads/*:refs/heads/*", "refs/cli-git/*:refs/cli-git/*"];

        let excluded: PathBuf = scratch(&format!("push-excluded-{}", backend));
        git(&excluded, &["init", "--quiet", "--bare", &format!("--ref-format={}", backend)]);
        push(&local, &excluded, &[positive[0], positive[1], "^refs/cli-git/*"]);
        assert_eq!(git_text(&excluded, &["for-each-ref", "--format=%(refname)", "refs/heads"]).trim(), "refs/heads/main", "{}", backend);
        assert!(git_text(&excluded, &["for-each-ref", "refs/cli-git"]).trim().is_empty(), "{}: the negative refspec kept the namespace off the remote", backend);

        let control: PathBuf = scratch(&format!("push-control-{}", backend));
        git(&control, &["init", "--quiet", "--bare", &format!("--ref-format={}", backend)]);
        push(&local, &control, &positive);
        assert_eq!(git_text(&control, &["for-each-ref", "--format=%(refname)", "refs/cli-git"]).trim(), "refs/cli-git/instance/withdrawn/id", "{}: without the exclusion it is pushed", backend);

        let frozen: PathBuf = scratch(&format!("push-frozen-{}", backend));
        git(&frozen, &["init", "--quiet", "--bare", &format!("--ref-format={}", backend)]);
        let composed: String = format!("{}:refs/heads/frozen", local.first);
        git(&local.path, &["update-ref", "refs/heads/main", &local.second]);
        push(&local, &frozen, &[&composed]);
        assert_eq!(git_text(&frozen, &["rev-parse", "refs/heads/frozen"]).trim(), local.first, "{}: the remote has the object that was named, not the moved branch", backend);
        assert!(git_text(&frozen, &["for-each-ref", "refs/heads/main"]).trim().is_empty(), "{}", backend);
    }
}

// region: experiments of round 22 for the supported scope: packed refs, linked worktrees, the side-ref transaction, creation, deletion and
// symbolic-reference updates, a torn instruction, and the order of the ref and the answer (publication evidence)

/// Sends `text` as a whole transaction (start, the instructions, prepare, commit) and returns the finished child's exit status.
fn run_transaction(repository: &Repository, extra: &[&str], instructions: &str) -> (ExitStatus, String, String) {
    return run_transaction_with(repository, extra, &[], instructions);
}

/// As `run_transaction`, with `options` given to `update-ref`.
fn run_transaction_with(repository: &Repository, extra: &[&str], options: &[&str], instructions: &str) -> (ExitStatus, String, String) {
    let mut transaction: Transaction = open_transaction_with(repository, extra, options);
    transaction.send(&format!("start\n{}prepare\ncommit\n", instructions));
    transaction.close_keeper();
    let status: ExitStatus = transaction.finish();
    return (status, transaction.stdout(), transaction.stderr());
}

/// Packed refs (files backend): after `pack-refs --all` every branch lives in `packed-refs`; one transaction updates a packed ref, deletes
/// another and creates a third, and the result is exactly that, with the deleted ref gone from `packed-refs` too. The reftable backend has no
/// `packed-refs` file at all.
#[test]
fn a_fifo_transaction_updates_creates_and_deletes_packed_refs() {
    let files: Repository = repository("packed", "files");
    git(&files.path, &["update-ref", "refs/heads/topic", &files.first]);
    git(&files.path, &["pack-refs", "--all"]);
    let packed: String = fs::read_to_string(files.path.join(".git/packed-refs")).expect("packed-refs");
    assert!(packed.contains("refs/heads/main") && packed.contains("refs/heads/topic"), "both branches are packed: {}", packed);
    assert!(!files.path.join(".git/refs/heads/main").exists(), "packed: no loose file for main");
    let instructions: String = format!(
        "update refs/heads/main {second} {first}\ndelete refs/heads/topic {first}\ncreate refs/heads/new {second}\n",
        second = files.second,
        first = files.first
    );
    let (status, _out, err) = run_transaction(&files, &[], &instructions);
    assert!(status.success(), "{:?} {}", status, err);
    assert_eq!(files.resolve("refs/heads/main").as_deref(), Some(files.second.as_str()));
    assert_eq!(files.resolve("refs/heads/topic"), None, "the packed ref is deleted");
    assert_eq!(files.resolve("refs/heads/new").as_deref(), Some(files.second.as_str()));
    let after: String = fs::read_to_string(files.path.join(".git/packed-refs")).expect("packed-refs");
    assert!(!after.contains("refs/heads/topic"), "the deleted ref is gone from packed-refs: {}", after);

    let reftable: Repository = repository("packed", "reftable");
    git(&reftable.path, &["pack-refs", "--all"]);
    assert!(!reftable.path.join(".git/packed-refs").exists(), "the reftable backend has no packed-refs file");
}

/// Creation, deletion and a refused deletion through the transaction, on both backends: a creation with the zero old value, a deletion with
/// the right old value, and a deletion with a wrong old value that is refused at `prepare` with a diagnostic that names the ref.
#[test]
fn creation_and_deletion_through_a_transaction_check_their_expected_values() {
    for backend in BACKENDS {
        let repository: Repository = repository("create-delete", backend);
        let (created, _o, e1) = run_transaction(&repository, &[], &format!("create refs/heads/made {}\n", repository.second));
        assert!(created.success(), "{}: {}", backend, e1);
        assert_eq!(repository.resolve("refs/heads/made").as_deref(), Some(repository.second.as_str()), "{}", backend);
        let mut refused: Transaction = open_transaction(&repository, &[]);
        refused.send(&format!("start\ndelete refs/heads/made {}\nprepare\n", repository.first));
        let status: ExitStatus = refused.finish();
        assert!(!status.success(), "{}: a deletion with a wrong old value is refused", backend);
        assert!(refused.stderr().contains("refs/heads/made"), "{}: the diagnostic names the ref: {}", backend, refused.stderr());
        assert!(!refused.stdout().contains("prepare: ok"), "{}", backend);
        assert_eq!(repository.resolve("refs/heads/made").as_deref(), Some(repository.second.as_str()), "{}: the ref stays", backend);
        let (deleted, _o2, e2) = run_transaction(&repository, &[], &format!("delete refs/heads/made {}\n", repository.second));
        assert!(deleted.success(), "{}: {}", backend, e2);
        assert_eq!(repository.resolve("refs/heads/made"), None, "{}", backend);
    }
}

/// Symbolic-reference updates through the transaction (Git 2.56.0 reads `symref-create`, `symref-update` and `symref-delete`), and an
/// `update HEAD` that follows the symbolic `HEAD` to the branch it names, as the design's publication target says.
#[test]
fn symbolic_references_are_created_updated_and_deleted_through_a_transaction_and_head_is_followed() {
    for backend in BACKENDS {
        let repository: Repository = repository("symref", backend);
        let target = |name: &str| -> String { return git_text(&repository.path, &["symbolic-ref", name]).trim().to_string(); };
        let (created, _o, e) = run_transaction(&repository, &[], "symref-create refs/heads/alias refs/heads/main\n");
        assert!(created.success(), "{}: {}", backend, e);
        assert_eq!(target("refs/heads/alias"), "refs/heads/main", "{}", backend);
        git(&repository.path, &["update-ref", "refs/heads/other", &repository.first]);
        let (updated, _o2, e2) = run_transaction(&repository, &[], "symref-update refs/heads/alias refs/heads/other\n");
        assert!(updated.success(), "{}: {}", backend, e2);
        assert_eq!(target("refs/heads/alias"), "refs/heads/other", "{}", backend);
        let (deleted, _o3, e3) = run_transaction_with(&repository, &[], &["--no-deref"], "symref-delete refs/heads/alias\n");
        assert!(deleted.success(), "{}: {}", backend, e3);
        assert!(!git_output(&repository.path, &["symbolic-ref", "refs/heads/alias"]).status.success(), "{}: the symbolic ref is gone", backend);
        let before_head: String = target("HEAD");
        let (moved, _o4, e4) = run_transaction(&repository, &[], &format!("update HEAD {} {}\n", repository.second, repository.first));
        assert!(moved.success(), "{}: {}", backend, e4);
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.second.as_str()), "{}: HEAD was followed to the branch", backend);
        assert_eq!(target("HEAD"), before_head, "{}: HEAD stays symbolic", backend);
    }
}

/// The side-ref transaction: an unreferenced commit is pruned by an immediate-expiry prune (the control), and kept once a ref under
/// `refs/cli-git/` names it; the ref is created by a transaction with an empty hooks path, so the person's `reference-transaction` hook
/// does not run for it (hook-free internal transactions), while the same transaction without the empty path does run the hook.
#[test]
fn a_side_ref_is_created_hook_free_and_makes_its_commit_reachable_for_a_prune() {
    for backend in BACKENDS {
        let repository: Repository = repository("side-ref", backend);
        let tree: String = git_text(&repository.path, &["rev-parse", "HEAD^{tree}"]).trim().to_string();
        let orphan: String = git_text(&repository.path, &["commit-tree", &tree, "-m", "orphan"]).trim().to_string();
        let dry = |candidate: &Repository| -> String { return git_text(&candidate.path, &["prune", "--dry-run", "--expire=now"]); };
        assert!(dry(&repository).contains(&orphan), "{}: the control: an unreferenced commit is a prune candidate", backend);
        let log: PathBuf = install_hook(&repository, "exit 0");
        let empty: PathBuf = scratch("side-empty-hooks");
        let (hook_free, _o, e) = run_transaction(&repository, &["-c", &format!("core.hooksPath={}", empty.display())], &format!("create refs/cli-git/instance/pending/id {}\n", orphan));
        assert!(hook_free.success(), "{}: {}", backend, e);
        assert!(phases(&log).is_empty(), "{}: no hook ran for the side ref: {:?}", backend, phases(&log));
        assert_eq!(repository.resolve("refs/cli-git/instance/pending/id").as_deref(), Some(orphan.as_str()), "{}", backend);
        assert!(!dry(&repository).contains(&orphan), "{}: a ref under refs/cli-git/ keeps its commit out of the prune view", backend);
        let (hooked, _o2, e2) = run_transaction(&repository, &[], &format!("create refs/cli-git/instance/pending/second {}\n", orphan));
        assert!(hooked.success(), "{}: {}", backend, e2);
        assert_eq!(phases(&log), ["preparing", "prepared", "committed"], "{}: without the empty path the hook runs", backend);
    }
}

/// A foreign program holds the lock of the branch. Files backend: the lock is the branch's own `main.lock`, so the update of the branch
/// fails at `prepare` with a diagnostic that names the lock while the side ref, a different ref, is created. Reftable: the lock is the
/// stack's `tables.list.lock`, one for every ref, so both fail; "save on a side ref" cannot rely on a different lock there.
#[test]
fn a_foreign_branch_lock_blocks_the_branch_and_not_the_side_ref_on_the_files_backend_but_blocks_both_on_reftable() {
    for backend in BACKENDS {
        let repository: Repository = repository("foreign-lock", backend);
        let lock: PathBuf = if backend == "files" { repository.path.join(".git/refs/heads/main.lock") } else { repository.path.join(".git/reftable/tables.list.lock") };
        fs::write(&lock, b"").expect("a foreign lock");
        let (branch, _o, branch_error) = run_transaction(&repository, &[], &repository.move_main());
        assert!(!branch.success(), "{}: the branch update is refused while the lock exists", backend);
        assert!(branch_error.contains("lock"), "{}: the diagnostic names a lock: {}", backend, branch_error);
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.first.as_str()), "{}", backend);
        let (side, _o2, side_error) = run_transaction(&repository, &[], &format!("create refs/cli-git/instance/pending/id {}\n", repository.second));
        if backend == "files" {
            assert!(side.success(), "files: the side ref has its own lock: {}", side_error);
            assert_eq!(repository.resolve("refs/cli-git/instance/pending/id").as_deref(), Some(repository.second.as_str()));
        } else {
            assert!(!side.success(), "reftable: one lock covers every ref: {}", side_error);
        }
    }
}

/// A torn final instruction: the writer sent the whole prepared transaction and the first letters of `commit`, then every writer closed.
/// Git does not commit a partial instruction; the ref stays.
#[test]
fn a_torn_commit_instruction_at_end_of_input_never_commits() {
    for backend in BACKENDS {
        let repository: Repository = repository("torn", backend);
        let mut transaction: Transaction = open_transaction(&repository, &[]);
        transaction.send(&format!("start\n{}prepare\ncomm", repository.move_main()));
        transaction.close_keeper();
        let status: ExitStatus = transaction.finish();
        assert!(!status.success(), "{}: a torn instruction is an error exit: {}", backend, transaction.stderr());
        assert!(transaction.stderr().contains("comm"), "{}: the diagnostic names the torn command: {}", backend, transaction.stderr());
        assert!(!transaction.stdout().contains("commit: ok"), "{}: {}", backend, transaction.stdout());
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.first.as_str()), "{}: a torn commit does not move the ref", backend);
    }
}

/// Publication evidence precedence: the ref moves before Git answers `commit: ok`. A `reference-transaction` hook that blocks at the
/// `committed` phase holds the answer back while the ref already names the new commit, so the ref is the evidence and the answer line is
/// only a report of it.
#[test]
fn the_ref_moves_before_the_commit_answer_is_written() {
    for backend in BACKENDS {
        let repository: Repository = repository("evidence", backend);
        let entered: PathBuf = repository.path.join("entered");
        let release: PathBuf = repository.path.join("release");
        let script: String = format!("#!/bin/sh\nif [ \"$1\" = committed ]; then : > '{}'; while [ ! -e '{}' ]; do sleep 0.02; done; fi\ncat >/dev/null\nexit 0\n", entered.display(), release.display());
        let hook: PathBuf = repository.path.join(".git/hooks/reference-transaction");
        fs::create_dir_all(hook.parent().expect("hooks directory")).expect("hooks directory");
        fs::write(&hook, script).expect("hook");
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).expect("executable hook");
        let mut transaction: Transaction = open_transaction(&repository, &[]);
        transaction.send(&format!("start\n{}prepare\n", repository.move_main()));
        transaction.wait_for_answer("prepare: ok");
        transaction.send("commit\n");
        let deadline: Instant = Instant::now() + ANSWER_WAIT;
        while !entered.exists() {
            assert!(Instant::now() < deadline, "{}: the hook never reached the committed phase", backend);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.second.as_str()), "{}: the ref moved before the answer", backend);
        assert!(!transaction.stdout().contains("commit: ok"), "{}: the answer is not written yet: {}", backend, transaction.stdout());
        fs::write(&release, b"").expect("release the hook");
        transaction.wait_for_answer("commit: ok");
        transaction.close_keeper();
        assert!(transaction.finish().success(), "{}", backend);
    }
}

/// Linked worktrees: shared refs and per-worktree refs. A transaction run from a linked worktree updates a shared branch (even one that the
/// main worktree has checked out) and, through `HEAD`, the linked worktree's own branch; a ref under `refs/worktree/` is private to the
/// worktree that created it, which is where the decision record's scratch ref `refs/worktree/cli-git-service/<seq>` lives.
#[test]
fn a_linked_worktree_shares_branches_and_keeps_refs_worktree_private() {
    for backend in BACKENDS {
        let main: Repository = repository("worktree", backend);
        let linked_path: PathBuf = scratch(&format!("worktree-linked-{}", backend)).join("linked");
        git(&main.path, &["worktree", "add", "--quiet", "-b", "linked-branch", linked_path.to_str().expect("a path"), &main.first]);
        let linked: Repository = Repository { path: linked_path.clone(), first: main.first.clone(), second: main.second.clone() };
        let (shared, _o, e) = run_transaction(&linked, &[], &linked.move_main());
        assert!(shared.success(), "{}: {}", backend, e);
        assert_eq!(main.resolve("refs/heads/main").as_deref(), Some(main.second.as_str()), "{}: a shared branch moved from the linked worktree", backend);
        let (own, _o2, e2) = run_transaction(&linked, &[], &format!("update HEAD {} {}\n", main.second, main.first));
        assert!(own.success(), "{}: {}", backend, e2);
        assert_eq!(main.resolve("refs/heads/linked-branch").as_deref(), Some(main.second.as_str()), "{}: HEAD in the linked worktree names its own branch", backend);
        let (private, _o3, e3) = run_transaction(&linked, &[], &format!("create refs/worktree/cli-git-service/1 {}\n", main.second));
        assert!(private.success(), "{}: {}", backend, e3);
        assert_eq!(linked.resolve("refs/worktree/cli-git-service/1").as_deref(), Some(main.second.as_str()), "{}: visible in its worktree", backend);
        assert_eq!(main.resolve("refs/worktree/cli-git-service/1"), None, "{}: not visible from the main worktree", backend);
    }
}

/// The client-lock handoff with real Git: the client holds its `flock` on the lane's client file; the transaction child started as the
/// bootstrap says inherits no descriptor of it, so the lock dies with the client and not with a Git child that may outlive it.
#[test]
fn the_transaction_child_inherits_no_client_lock_descriptor() {
    for backend in BACKENDS {
        let repository: Repository = repository("client-lock", backend);
        let lock_path: PathBuf = scratch("client-lock-file").join("client");
        let client_lock: File = OpenOptions::new().create(true).write(true).truncate(false).open(&lock_path).expect("the client lock file");
        client_lock.try_lock().expect("the client holds its lock");
        let mut transaction: Transaction = open_transaction(&repository, &[]);
        transaction.send(&format!("start\n{}prepare\n", repository.move_main()));
        transaction.wait_for_answer("prepare: ok");
        let listing: Vec<PathBuf> = fs::read_dir(format!("/proc/{}/fd", transaction.child.id())).expect("the child's descriptors").flatten().filter_map(|entry| { return fs::read_link(entry.path()).ok(); }).collect();
        assert!(!listing.iter().any(|target| { return target == &lock_path; }), "{}: the transaction child holds the client lock: {:?}", backend, listing);
        drop(client_lock);
        let other: File = OpenOptions::new().write(true).open(&lock_path).expect("the lock file again");
        assert!(other.try_lock().is_ok(), "{}: the lock is free once the client let go, although the child lives", backend);
        assert!(transaction.running(), "{}", backend);
        transaction.send("abort\n");
        transaction.close_keeper();
        assert!(transaction.finish().success(), "{}", backend);
    }
}

/// The submission handoff of the write end: a second writer opens the FIFO while the first (the keeper) lives, the first closes, and the
/// transaction stays alive because a writer remains; the second then sends `commit`. Only when the last writer closes does Git read the
/// end of input.
#[test]
fn a_second_writer_keeps_the_transaction_alive_when_the_first_closes() {
    for backend in BACKENDS {
        let repository: Repository = repository("handoff", backend);
        let mut transaction: Transaction = open_transaction(&repository, &[]);
        transaction.send(&format!("start\n{}prepare\n", repository.move_main()));
        transaction.wait_for_answer("prepare: ok");
        let mut second: File = OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(&transaction.fifo).expect("a second writer opens while the reader lives");
        transaction.close_keeper();
        std::thread::sleep(QUIET_WAIT);
        assert!(transaction.running(), "{}: a writer remains, so there is no end of input", backend);
        second.write_all(b"commit\n").expect("the second writer commits");
        transaction.wait_for_answer("commit: ok");
        assert_eq!(repository.resolve("refs/heads/main").as_deref(), Some(repository.second.as_str()), "{}", backend);
        drop(second);
        assert!(transaction.finish().success(), "{}", backend);
    }
}

/// Interruption at the persistence boundaries by a kill of `update-ref`: after `prepare: ok` the ref is unchanged and the process leaves
/// the lock it held (files: the branch's own `main.lock`; reftable: the stack's `tables.list.lock`), which a later transaction reports;
/// after `commit: ok` the new value is there and no lock remains. The stale lock is what the lane's reclaim decision (the owner's decision of
/// 2026-11-04) is for, and the experiment shows that Git does not clear it itself.
#[test]
fn killing_update_ref_after_prepare_leaves_the_old_value_and_a_stale_lock_and_after_commit_the_new_value() {
    for backend in BACKENDS {
        let lock_name: &str = if backend == "files" { ".git/refs/heads/main.lock" } else { ".git/reftable/tables.list.lock" };
        let prepared: Repository = repository("kill-prepared", backend);
        let mut transaction: Transaction = open_transaction(&prepared, &[]);
        transaction.send(&format!("start\n{}prepare\n", prepared.move_main()));
        transaction.wait_for_answer("prepare: ok");
        assert!(prepared.path.join(lock_name).exists(), "{}: the prepared transaction holds {}", backend, lock_name);
        transaction.child.kill().expect("kill");
        let _ = transaction.child.wait();
        assert_eq!(prepared.resolve("refs/heads/main").as_deref(), Some(prepared.first.as_str()), "{}: the old value", backend);
        assert!(prepared.path.join(lock_name).exists(), "{}: Git does not clear the lock of a killed process", backend);
        let (later, _o, error) = run_transaction(&prepared, &[], &prepared.move_main());
        assert!(!later.success() && error.contains("lock"), "{}: a later transaction reports the stale lock: {}", backend, error);

        let committed: Repository = repository("kill-committed", backend);
        let mut second: Transaction = open_transaction(&committed, &[]);
        second.send(&format!("start\n{}prepare\ncommit\n", committed.move_main()));
        second.wait_for_answer("commit: ok");
        second.child.kill().expect("kill");
        let _ = second.child.wait();
        assert_eq!(committed.resolve("refs/heads/main").as_deref(), Some(committed.second.as_str()), "{}: the new value", backend);
        assert!(!committed.path.join(lock_name).exists(), "{}: no lock remains after a commit", backend);
    }
}

// endregion
