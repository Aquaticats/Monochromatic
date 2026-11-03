//! What:
//!  The shell condition a fixture script tests to learn that the test that started it is still there.
//! Why:
//!  Product code starts these scripts (a fake Git, a hook, a lock holder), so the test cannot kill them with a guard of its own,
//!  and a script that waits for a release a failed test never writes would run on in a fixture that is already removed. The
//!  script asks the condition at each turn of its wait: it holds while the fixture directory exists and the test process lives,
//!  and the end of either, however the test ended, ends the wait. No time bound is guessed, so a slow host never cuts a wait short.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const condition = `[ -d '${root}' ] && kill -0 ${process.pid} 2>/dev/null`;
//! ```

use std::path::Path;

/// What:
///  The condition, for a shell `while` or `if`, that holds while the fixture at `root` exists and this test process lives.
/// Why:
///  A failed test removes its fixture as it unwinds, which is the first sign that nobody will release the script; the process
///  check covers a test that keeps its fixture on request (`CLI_GIT_KEEP_FIXTURES`) or dies without unwinding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whileTestLives(root) // "[ -d '/fixture' ] && kill -0 4242 2>/dev/null"
/// ```
pub fn while_test_lives(root: &Path) -> String {
    // A single quote in the path ends the quoted text, is written as itself, and starts it again.
    let quoted: String = root.display().to_string().replace('\'', "'\\''");
    return format!("[ -d '{quoted}' ] && kill -0 {} 2>/dev/null", std::process::id());
}
