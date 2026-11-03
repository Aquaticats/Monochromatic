//! What:
//!  The shell condition a fixture script tests to learn that the test that started it is still there.
//! Why:
//!  Product code starts these scripts (a fake Git, a hook, a lock holder), so the test cannot kill them with a guard of its own,
//!  and a script that waits for a release a failed test never writes would run on in a fixture that is already removed. The
//!  script asks the condition at each turn of its wait: it holds while the fixture directory exists and the test process lives,
//!  and the end of either, however the test ended, ends the wait. No time bound is guessed, so a slow host never cuts a wait short.
//!  A script that must not start processes while it waits (a hook the progress meter judges by what its tree does) cannot ask
//!  the condition, so the fixture ends whatever still works in its directory when the test ends.
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

/// What:
///  End every process, other than this one, whose working directory is inside `root`.
/// Why:
///  A hook or a fake program that product code started in the fixture and that waits with one `exec sleep` cannot watch its test
///  without starting processes of its own, which a progress meter counts as work. Nothing else may work in a fixture once its
///  test is over, so the fixture ends it as it goes, before the directory is removed. A process that is gone, or one this user may
///  not signal, is skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const process of await listProcesses()) if (process.cwd.startsWith(root) && process.pid !== self) process.kill('SIGKILL');
/// ```
#[cfg(target_os = "linux")]
pub fn end_processes_working_in(root: &Path) {
    let own: u32 = std::process::id();
    let Ok(listing) = std::fs::read_dir("/proc") else {
        return;
    };
    for entry in listing.flatten() {
        // Only the numbered entries are processes.
        let Some(pid) = entry.file_name().to_str().and_then(|name| { return name.parse::<u32>().ok(); }) else {
            continue;
        };
        if pid == own {
            continue;
        }
        let Ok(directory) = std::fs::read_link(entry.path().join("cwd")) else {
            continue;
        };
        if directory.starts_with(root) {
            let Ok(target) = libc::pid_t::try_from(pid) else {
                continue;
            };
            // SAFETY: `kill` takes a pid and a signal and touches no memory; a process that ended meanwhile is `ESRCH`.
            unsafe { libc::kill(target, libc::SIGKILL) };
        }
    }
}

/// Where `/proc` does not list working directories nothing is ended; the scripts that can watch their test still do.
#[cfg(not(target_os = "linux"))]
pub fn end_processes_working_in(_root: &Path) {}
