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

/// Runs real Git in `directory` with a hermetic environment and returns its raw output.
pub fn git_output(directory: &Path, arguments: &[&str]) -> Output {
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
        .expect("start Git");
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
