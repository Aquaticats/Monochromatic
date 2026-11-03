//! What:
//!  The fsmonitor section of `git cli-git doctor`: whether Git's own file-system monitor (`core.fsmonitor`) is off, and
//!  where it is on, the file and scope that turn it on and the command that turns it off.
//! Why:
//!  The owner decided on 2026-11-03 to keep Git's fsmonitor off everywhere; doctor says so (stub for the red test).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = renderFsmonitor(parseFsmonitorEntries(await git(['config', '--list', '--show-origin', '--show-scope', '-z'])));
//! ```

use super::transaction_git::GitContext;
use std::path::Path;

/// The repository-relative path of the document that records why the owner turned Git's fsmonitor off.
pub const FSMONITOR_DOCUMENT: &str = "doc/troubleshooting/git-fsmonitor-daemon-start-wait.md";

/// The date of the owner's decision to keep Git's fsmonitor off.
pub const DECISION_DATE: &str = "2026-11-03";

/// The one-sentence reason, from measurements of 2026-11-03 on the owner's main checkout (261 daemons held 4,591,335
///  watches; one daemon watched 2,269,377 directories and spent 117.7 s of CPU and 734 MB to start; a running daemon
///  saved about 7 ms per `git status`) and from Git 2.56.0 (`compat/fsmonitor/fsm-listen-linux.c`,
///  `builtin/fsmonitor--daemon.c`); the full account is in the document named by `FSMONITOR_DOCUMENT`.
pub const FSMONITOR_REASON: &str = "Git's fsmonitor daemon registers a watch on every directory of the work tree, \
linked worktrees inside it included, before it answers, so a Git command that finds no running daemon starts one and \
waits for it for up to `fsmonitor.startTimeout` (60 s by default), while a running daemon saved about 7 ms per \
`git status` on the owner's main checkout.";

/// One `core.fsmonitor` entry of the configuration listing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FsmonitorEntry {
    /// `system`, `global`, `local`, `worktree` or `command`.
    pub scope: String,
    /// Where Git read the entry, as it prints it: `file:<path>` or `command line:`.
    pub origin: String,
    /// The value bytes, or `None` for a key written without `=`.
    pub value: Option<Vec<u8>>,
}

/// What:
///  The `core.fsmonitor` entries of a `git config --list --show-origin --show-scope -z` listing, in precedence order.
/// Why:
///   Stub.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseFsmonitorEntries(bytes);
/// ```
pub fn parse_fsmonitor_entries(_listing: &[u8]) -> Option<Vec<FsmonitorEntry>> {
    return None;
}

/// What:
///  The standard error text for the entries of `core.fsmonitor`.
/// Why:
///   Stub.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renderFsmonitor(entries);
/// ```
pub fn render_fsmonitor(_entries: &[FsmonitorEntry]) -> String {
    return String::new();
}

/// What:
///  Read the listing for `cwd` and render the fsmonitor text.
/// Why:
///   Stub.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await fsmonitorReport({ context, cwd });
/// ```
pub fn fsmonitor_report(_context: &GitContext, _cwd: &Path) -> String {
    return String::new();
}

/// Fsmonitor controls are in the test module.
#[cfg(test)]
#[path = "doctor_fsmonitor_tests.rs"]
mod tests;
