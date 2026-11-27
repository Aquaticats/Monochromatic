//! What:
//!  What a failure hands a person to act on:
//!  the commands it tells them to run,
//!  the sentence it ends with when nothing in the repository changed,
//!  the terms that must never reach them,
//!  and the way a repository path is written in its text.
//! Why:
//!  Every rendering of a failure is built from these,
//!  so the command a person copies is written once,
//!  one wording of "nothing changed" is shared by all of them,
//!  and a test holds every term of the deny list against every rendering
//!  (`service_messages_tests.rs`).
//!  They live beside the rendering rather than inside it,
//!  because that file is at the line bound the repository linter enforces (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceMessagesWhatAFailureGivesAPerson.ts
//! export const LOG_COMMAND = "journalctl --user --unit 'monochromatic-git-policy-cli@*.service'";
//! export const nothingChanged = "Nothing in the repository changed.";
//! export const denylist: string[] = ["protocol", "journal", "request id"];
//! export const pathText = (path: string): string => path;
//! ```

use std::path::Path;

/// The command that shows why the service did not start (Linux with systemd).
pub const LOG_COMMAND: &str = "journalctl --user --unit 'monochromatic-git-policy-cli@*.service'";

/// The command that turns on user services for an account that is not logged in (Linux with systemd).
pub const LINGER_COMMAND: &str = "loginctl enable-linger";

/// The command that restarts the commit service,
/// so the manager starts the one the installed units name (Linux with systemd).
pub const RESTART_COMMAND: &str =
    "systemctl --user restart 'monochromatic-git-policy-cli@*.service'";

/// The command that stops the commit service,
/// so the manager starts the one the installed units name on the next connection (Linux with systemd).
/// Why:
///   A restart starts the program the manager's unit names,
/// which after a rollback is still the newer release's,
/// so stopping the
///  service is the one command that lets the older wrapper's own service answer.
pub const STOP_COMMAND: &str = "systemctl --user stop 'monochromatic-git-policy-cli@*.service'";

/// The commands that show whether the commit was made.
pub const CHECK_COMMANDS: &[&str] = &["git log -1", "git status"];

/// The command that shows what the commit service of a worktree is doing,
/// and names the unit that serves it.
///  Why:
///   The unit of one worktree is the one a restart may name,
/// and the wrapper's own command is the one that reads the state of that service without asking for anything:
/// a restart of every service of the account is what a commit must never ask for,
/// because it stops the
///  commits of every other worktree.
pub const STATUS_COMMAND: &str = "git monochromatic-git-policy-cli service status";

/// The command that restarts one unit of the service manager,
/// followed by the unit the status named.
pub const RESTART_ONE_UNIT_COMMAND: &str = "systemctl --user restart";

/// What a failure says about the staged work when nothing in the repository changed,
/// in the words these failures use.
///  Why:
///   A commit that stopped before it asked anything of the service changed nothing,
/// and the person is owed that sentence in the words of the failure they read.
pub const NOTHING_CHANGED_AND_THE_STAGED_WORK: &str =
    "Your staged changes and your working files are as they were.";

/// The sentence every failure ends with when nothing was changed.
pub const NOTHING_CHANGED: &str = "Nothing in the repository changed.";

/// The texts that must never reach a user:
/// implementation terms the system should have resolved itself.
///  Matched case-insensitively against whole words after the quoted commands are removed.
pub const DENYLIST: &[&str] = &[
    "protocol",
    "journal",
    "request id",
    "daemon",
    "socket",
    "errno",
    "epipe",
    "econnrefused",
    "eagain",
    "flock",
    "lock file",
    "peer",
    "frame",
    "handshake",
    "batch",
    "oid",
    "fast-import",
    "unix domain",
    "abstract",
    "pid",
    "uid",
    "os error",
];

/// The repository path a failure names,
/// as text.
pub fn path_text(path: &Path) -> String {
    return path.display().to_string();
}
