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
    use std::process::Stdio;
    let mut child = command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|error| { return error.to_string(); })?;
    let mut stdout = child.stdout.take().ok_or("no stdout".to_string())?;
    let mut stderr = child.stderr.take().ok_or("no stderr".to_string())?;
    let out_reader = std::thread::spawn(move || { let mut bytes: Vec<u8> = Vec::new(); let _ = stdout.read_to_end(&mut bytes); return bytes; });
    let err_reader = std::thread::spawn(move || { let mut bytes: Vec<u8> = Vec::new(); let _ = stderr.read_to_end(&mut bytes); return bytes; });
    let deadline = std::time::Instant::now() + limit;
    let status = loop {
        match child.try_wait().map_err(|error| { return error.to_string(); })? {
            Some(status) => break status,
            None => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("the process timed out after {:?} and was killed", limit));
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
    };
    let stdout_bytes: Vec<u8> = out_reader.join().map_err(|_| { return "stdout reader".to_string(); })?;
    let stderr_bytes: Vec<u8> = err_reader.join().map_err(|_| { return "stderr reader".to_string(); })?;
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
    let owner = git_policy_cli::commit_ledger::LedgerOwner::acquire(ledger)?;
    return owner.append(anchor, ledger, entry);
}

/// Repairs a ledger's torn tail as its exclusive owner.
pub fn owned_repair(anchor: &Path, ledger: &Path) -> Result<git_policy_cli::commit_ledger::Repair, git_policy_cli::commit_ledger::LedgerError> {
    let owner = git_policy_cli::commit_ledger::LedgerOwner::acquire(ledger)?;
    return owner.repair_tail(anchor, ledger);
}
