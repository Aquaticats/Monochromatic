//! What:
//! Whether a path under the repository is unreachable because the repository's link to the 860 drive dangles,
//! and the plain message that says so,
//! by asking the installed repository link check (`package/repository-link/check`) once, on a failure path only.
//! Why:
//! After the repository moves to the 860 drive its old path is a link to the copy there.
//! An unreadable lock file or a launcher folder that cannot be made then has one common cause the person can act on
//! (unlock the drive),
//! and the check already words it.
//! The launcher stays fast: nothing here runs unless a read or a write already failed or an install is about to write under the link,
//! and with no identity file (`~/.config/monochromatic-dev/repository-link.json`) nothing starts.
//! The check program is the one installed outside the repository,
//! so it never dangles itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // repository-link-cause.ts
//! export function repositoryLinkVerdict(home: string, path: string, deadlineMs: number): LinkVerdict
//! ```

/// Imports `Duration` and `Instant` from `std::time`.
use std::time::{Duration, Instant};
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// The identity file the cut-over writes,
/// below the home directory.
pub const IDENTITY_RELATIVE_PATH: &str = ".config/monochromatic-dev/repository-link.json";

/// The installed check program,
/// below the home directory:
/// outside the repository,
/// so it runs when the repository is away.
pub const CHECK_RELATIVE_PATH: &str = ".local/share/monochromatic-dev/repository-link/repository-link-check";

/// How long the check program may run before it is killed and the answer is that it did not answer.
/// The check waits five seconds for the drive itself (`package/repository-link/check/src/constants.ts`,
/// `DEFAULT_DEADLINE_SECONDS`),
/// and the owner's rule is "nothing should ever wait forever or indefinitely";
/// this bound is that wait plus the time to start Node.
pub const CHECK_DEADLINE: Duration = Duration::from_secs(10);

/// How often the wait for the check program looks at it.
const CHECK_POLL: Duration = Duration::from_millis(10);

/// What:
/// What asking the repository link check came to.
/// Why:
/// A caller prints the message of `NotReady`,
/// and goes on for every other answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkVerdict = { kind: 'not-configured' | 'ready' } | { kind: 'not-ready'; message: string } | { kind: 'check-unavailable'; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkVerdict {
    /// There is no identity file,
    /// or the path is not under the repository:
    /// nothing was asked.
    NotConfigured,
    /// The check says the repository is usable.
    Ready,
    /// The check says it is not,
    /// or did not answer in time;
    /// this is its plain message.
    NotReady(String),
    /// The check program could not be started;
    /// this says why.
    CheckUnavailable(String),
}

/// What:
/// Whether a path lies under the repository the identity file names:
/// under its link path,
/// under its approved canonical root,
/// or under either with the part that exists resolved.
/// Why:
/// A launcher folder or a lock file is under the repository only by one of its spellings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// underRepositoryLink(home, path)
/// ```
pub fn under_repository_link(home: &Path, path: &Path) -> bool {
    todo!("{} {}", home.display(), path.display())
}

/// What:
/// Asks the installed repository link check whether the repository is usable,
/// when `path` is under the repository the identity file names.
/// Why:
/// The check words every cause (drive not mounted,
/// folder missing,
/// a real folder in place of the link) once,
/// so no launcher words them again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryLinkVerdict(home, path, deadline)
/// ```
pub fn repository_link_verdict(home: &Path, path: &Path, deadline: Duration) -> LinkVerdict {
    todo!("{} {} {deadline:?} {:?}", home.display(), path.display(), (Instant::now(), PathBuf::new(), CHECK_POLL))
}

/// The unit tests of the repository link cause.
#[cfg(test)]
#[path = "repository_link_cause_tests.rs"]
mod tests;
