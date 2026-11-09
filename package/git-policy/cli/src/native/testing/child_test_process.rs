//! What:
//!  A test body run in a child of the test binary,
//! which is where state that belongs to one process is controlled.
//! Why:
//!  A descriptor,
//!  a signal mask and a record of one process belong to that process,
//! so a control of them runs its own body in a child and
//!  asserts what the parent sees.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! spawnSync(process.execPath, [binary, '--exact', name], { env: { [variable]: '1' } })
//! ```

/// The path of this test binary,
/// asked of the process that runs.
use super::this_test_binary;
/// `PathBuf` is an owned filesystem path of raw OS bytes.
use std::path::PathBuf;

/// What:
///  This test binary started on one test,
/// with `variable` set so the child knows it is the child.
/// Why:
///  A descriptor,
///  a signal mask and a record of one process belong to that process,
/// so each control of them runs its own body in a child and
///  asserts what the parent sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync(process.execPath, [binary, '--exact', name], { env: { [variable]: '1' } })
/// ```
pub(crate) fn run(variable: &str, name: &str) -> std::process::Output {
    let binary: PathBuf = this_test_binary().expect("the test binary");
    return std::process::Command::new(binary)
        .args(["--exact", name, "--nocapture", "--test-threads=1"])
        .env(variable, "1")
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the child test runs");
}
