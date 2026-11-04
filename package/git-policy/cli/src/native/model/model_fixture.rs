//! What:
//!  Disposable directories and a hermetic Git runner for the step-1 fixtures of the commit protocol.
//! Why:
//!  Fixtures must never touch the real systemd user directory, the session bus, the production service or a real
//!  `.git`: each test gets its own new directory under the build's `CARGO_TARGET_TMPDIR`, and Git runs with a cleared
//!  environment, no system or global configuration and a fixed identity.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const dir = mkdtempSync(join(targetTmp, 'cli-model-')); execFileSync('/usr/bin/git', args, { env: {...} });
//! ```

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The real Git installed in the verification image (2.56.0 on the gate).
pub const REAL_GIT: &str = "/usr/bin/git";

/// A counter so two fixtures of one test process never share a name.
static NEXT: AtomicUsize = AtomicUsize::new(0);

/// What:
///  A new empty directory `<target tmp>/cli-model/<label>-<pid>-<n>`.
///  `static NEXT: AtomicUsize` is a process-wide counter that threads may bump safely (TS has no threads to share one).
/// Why:
///  Every fixture is disposable and private; the test target's own temporary directory is in the build tree, not `/tmp`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scratch(label: string): string;
/// ```
pub fn scratch(label: &str) -> PathBuf {
    let number: usize = NEXT.fetch_add(1, Ordering::SeqCst);
    let directory: PathBuf =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cli-model").join(format!("{}-{}-{}", label, std::process::id(), number));
    std::fs::create_dir_all(&directory).expect("scratch directory");
    return directory;
}

/// The longest the runner waits to reap a leader it has signalled.
pub const TEARDOWN_LIMIT: std::time::Duration = std::time::Duration::from_secs(5);

/// The longest any process a fixture starts may run before it is killed ("nothing should ever wait forever or indefinitely").
pub const PROCESS_LIMIT: std::time::Duration = std::time::Duration::from_secs(120);

/// What:
///  Runs `command` with its output captured and kills it if it outlives `limit`; returns its output or a message that names the
///  timeout. The standard output and error are read on their own threads so a full pipe cannot stall the child.
/// Why:
///  A fixture that waits on a hung child waits for ever; every process a test starts goes through this function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runBounded(command: Command, limitMs: number): Output; // throws on timeout
/// ```
pub fn run_bounded(command: &mut Command, limit: std::time::Duration) -> Result<Output, String> {
    use std::io::Read;
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    use std::sync::mpsc;
    // The child leads its own process group so a timeout can end its descendants too, not only the leader.
    command.process_group(0);
    let mut child = command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|error| { return error.to_string(); })?;
    let group: i32 = i32::try_from(child.id()).map_err(|error| { return error.to_string(); })?;
    let mut stdout = child.stdout.take().ok_or("no stdout".to_string())?;
    let mut stderr = child.stderr.take().ok_or("no stderr".to_string())?;
    let (out_sender, out_receiver) = mpsc::channel::<Vec<u8>>();
    let (err_sender, err_receiver) = mpsc::channel::<Vec<u8>>();
    // The readers send their bytes when their pipe closes; the waiter below never joins them without a deadline.
    std::thread::spawn(move || { let mut bytes: Vec<u8> = Vec::new(); let _ = stdout.read_to_end(&mut bytes); let _ = out_sender.send(bytes); });
    std::thread::spawn(move || { let mut bytes: Vec<u8> = Vec::new(); let _ = stderr.read_to_end(&mut bytes); let _ = err_sender.send(bytes); });
    let deadline = std::time::Instant::now() + limit;
    let timed_out = |leader: &mut std::process::Child| -> String {
        // A negative pid addresses the whole process group the child leads.
        // SAFETY: `kill` takes plain integers and has no memory preconditions.
        unsafe { libc::kill(-group, libc::SIGKILL); }
        let _ = leader.kill();
        // The reaping is bounded too: a process in an uninterruptible wait may not die, and the runner then says so instead of waiting.
        let reap_deadline = std::time::Instant::now() + TEARDOWN_LIMIT;
        loop {
            match leader.try_wait() {
                Ok(Some(_)) => return format!("the process timed out after {:?}; its process group was signalled and the leader was reaped", limit),
                Ok(None) if std::time::Instant::now() < reap_deadline => std::thread::sleep(std::time::Duration::from_millis(5)),
                Ok(None) => return format!("the process timed out after {:?}; its process group was signalled and the leader was not reaped within {:?}", limit, TEARDOWN_LIMIT),
                Err(error) => return format!("the process timed out after {:?}; reaping it failed: {}", limit, error),
            }
        }
    };
    let status = loop {
        match child.try_wait().map_err(|error| { return error.to_string(); })? {
            Some(status) => break status,
            None => {
                if std::time::Instant::now() >= deadline {
                    return Err(timed_out(&mut child));
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
    };
    let remaining = |now: std::time::Instant| -> std::time::Duration { return deadline.saturating_duration_since(now); };
    let Ok(stdout_bytes) = out_receiver.recv_timeout(remaining(std::time::Instant::now())) else {
        return Err(timed_out(&mut child));
    };
    let Ok(stderr_bytes) = err_receiver.recv_timeout(remaining(std::time::Instant::now())) else {
        return Err(timed_out(&mut child));
    };
    return Ok(Output { status, stdout: stdout_bytes, stderr: stderr_bytes });
}

/// Runs real Git in `directory` with a hermetic environment and returns its raw output; a Git that outlives `PROCESS_LIMIT` is killed
/// and the test fails with the timeout.
pub fn git_output(directory: &Path, arguments: &[&str]) -> Output {
    let mut command: Command = Command::new(REAL_GIT);
    command
        .current_dir(directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(arguments);
    return run_bounded(&mut command, PROCESS_LIMIT).expect("bounded Git");
}

/// Runs Git, requires success and returns its standard output as bytes.
pub fn git(directory: &Path, arguments: &[&str]) -> Vec<u8> {
    let output: Output = git_output(directory, arguments);
    assert!(output.status.success(), "git {:?} failed: {}", arguments, String::from_utf8_lossy(&output.stderr));
    return output.stdout;
}

/// Runs Git, requires success and returns its standard output as trimmed text.
pub fn git_text(directory: &Path, arguments: &[&str]) -> String {
    return String::from_utf8(git(directory, arguments)).expect("utf-8 output").trim().to_string();
}

/// Appends one ledger line as the ledger's exclusive owner (acquired and released around the call): the standalone `append` is not
/// public, so a test goes through the ownership contract like the lane does.
pub fn owned_append(anchor: &Path, ledger: &Path, entry: &git_policy_cli::commit_ledger::LedgerEntry) -> Result<(), git_policy_cli::commit_ledger::LedgerError> {
    if let Some(parent) = ledger.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut owner = git_policy_cli::commit_ledger::LedgerOwner::acquire(ledger)?;
    return owner.append(anchor, ledger, entry);
}

/// Repairs a ledger's torn tail as its exclusive owner.
pub fn owned_repair(anchor: &Path, ledger: &Path) -> Result<git_policy_cli::commit_ledger::Repair, git_policy_cli::commit_ledger::LedgerError> {
    let mut owner = git_policy_cli::commit_ledger::LedgerOwner::acquire(ledger)?;
    return owner.repair_tail(anchor, ledger);
}

/// The longest a test waits for a process it forked to leave.
#[cfg(target_os = "linux")]
const FORKED_PROCESS_LIMIT: std::time::Duration = std::time::Duration::from_secs(30);

/// What:
///  Runs `body` in a forked copy of this test process, which has exactly one thread (a fork keeps only the forking thread), and returns
///  the exit code `body` gave (255 when `body` panicked); `None` when the copy was killed by a signal or did not leave within the limit (it is
///  killed then).
/// Why:
///  The test harness runs each test on a second thread beside its own main thread, so a launcher that refuses a threaded process can only
///  be exercised from a copy of the process that has one. The copy runs only the body and `_exit`s: nothing of the harness runs in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inSingleThreadedProcess(body: () => number): number | null; // fork, run body, exit with its code
/// ```
#[cfg(target_os = "linux")]
pub fn in_single_threaded_process<F: FnOnce() -> i32>(body: F) -> Option<i32> {
    // SAFETY: the child runs only `body` and `_exit`; the harness' other threads are not copied, and `body` takes no lock they hold.
    let child: libc::pid_t = unsafe { libc::fork() };
    assert!(child >= 0, "the test could not fork a single-threaded copy");
    if child == 0 {
        let code: i32 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)).unwrap_or(-1);
        // SAFETY: `_exit` takes an integer and never returns.
        unsafe { libc::_exit(if code < 0 { 255 } else { code }) };
    }
    let deadline: std::time::Instant = std::time::Instant::now() + FORKED_PROCESS_LIMIT;
    loop {
        let mut status: libc::c_int = 0;
        // SAFETY: `status` is a live local the call writes through.
        let reaped: libc::pid_t = unsafe { libc::waitpid(child, &raw mut status, libc::WNOHANG) };
        if reaped == child {
            return if libc::WIFEXITED(status) { Some(libc::WEXITSTATUS(status)) } else { None };
        }
        if std::time::Instant::now() >= deadline {
            // SAFETY: `kill` and `waitpid` take plain integers and a live local.
            unsafe {
                libc::kill(child, libc::SIGKILL);
                libc::waitpid(child, &raw mut status, 0);
            }
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}
