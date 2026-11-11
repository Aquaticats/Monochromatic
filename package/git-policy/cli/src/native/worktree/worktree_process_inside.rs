//! What:
//!  Whether any process on this machine is working in a folder,
//!  named by its process id.
//! Why:
//!   A worktree whose branch has landed is removed only when nothing is working inside it:
//!  a process whose working folder is in the worktree is a person's shell,
//!  an agent's session,
//!  or a build that would fail the moment the folder went.
//!  This is a read of each process's working folder (`/proc/<pid>/cwd`),
//!  never of its environment.
//!  A process that ended while it was being read,
//!  or whose working folder this user may not read,
//!  is skipped:
//!  it cannot be shown to work there,
//!  and the removal then asks the other conditions again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pid = processWorkingInside(worktree, '/proc'); // undefined: nothing works there
//! ```

use std::path::Path;

/// What:
///  The first process id whose working folder is `root` or below it.
/// Why:
///   The removal needs only one reason to keep a worktree,
///  and the question is the same for every process,
///  so the walk stops at the first answer.
///  The caller's own process is skipped:
///  a command that runs in the worktree is excluded from the removal by its path,
///  and a command that does not could still have been started there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// processWorkingInside(root, procRoot) // number | undefined
/// ```
pub fn process_working_inside(root: &Path, proc_root: &Path) -> Option<u32> {
    let own: u32 = std::process::id();
    let listing = std::fs::read_dir(proc_root).ok()?;
    for entry in listing.flatten() {
        let Some(pid) = entry.file_name().to_str().and_then(|name| { return name.parse::<u32>().ok() }) else {
            continue;
        };
        if pid == own {
            continue;
        }
        let Ok(working) = std::fs::read_link(entry.path().join("cwd")) else {
            continue;
        };
        if working.starts_with(root) {
            return Some(pid);
        }
    }
    return None;
}

/// Process look controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_process_inside_tests.rs"]
mod tests;
