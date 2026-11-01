//! What:
//!  The two kinds of Git step a commit starts for itself, and the bound and the notice grace of each.
//! Why:
//!  One bound for every step had to be long enough for the slowest, so a stalled `rev-parse` held a commit for as long
//!  as a stalled `pack-objects`, and the first notice came after one second for steps that only needed a moment.
//!  The classes follow what a step touches, measured on the repository this crate lives in
//!  (`doc/planning/cli-git-commit-timeouts.md`):
//!  a query reads refs, commits and trees and ends in milliseconds whatever the worktree holds, while a step that
//!  writes objects, or that compares the index with the worktree, costs time in proportion to the bytes it reads
//!  (`reset`, `update-index --refresh` and `ls-files --modified` took 2.9 s on a stat-dirty 20 000-file index against
//!  0.03 s on a clean one, and a cold disk multiplies that).
//!  The user's own `git commit` with its hooks is not a class: its runtime belongs to the user and it is never bounded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const limit = QUERY_SUBCOMMANDS.includes(subcommandOf(args)) ? QUERY_LIMIT_MS : BULK_LIMIT_MS;
//! ```

/// Positive-seconds parsing shared with the lock wait limits.
use super::bounded_push::positive_seconds;
/// The name of the subcommand a request runs.
use super::transaction_git::subcommand_of;
/// Operating-system text of the arguments and of the environment value.
use std::ffi::{OsStr, OsString};
/// Time spans.
use std::time::Duration;

/// Environment variable that sets, in whole seconds, how long any Git step the wrapper starts for itself may run
///  before it is stopped, whatever its class.
pub const CHILD_WAIT_VARIABLE: &str = "CLI_GIT_CHILD_WAIT_SECONDS";

/// How long a query step may run:
///  a hundred times the slowest such step measured on a host at several times its core count in load.
pub const PLUMBING_CHILD_WAIT: Duration = Duration::from_secs(60);

/// How long a bulk step may run:
///  its time grows with the bytes it moves or compares, so the bound leaves room for a large or stat-dirty worktree
///  on a loaded host.
pub const OBJECT_CHILD_WAIT: Duration = Duration::from_secs(300);

/// How long a query step runs before the first notice.
pub const PLUMBING_NOTICE_GRACE: Duration = Duration::from_secs(5);

/// How long a bulk step runs before the first notice.
pub const OBJECT_NOTICE_GRACE: Duration = Duration::from_secs(10);

/// Subcommands that only read refs, commits and trees, or write one ref or one tree from the index as it is:
///  they end in milliseconds whatever the worktree holds.
///  Every other subcommand, named or not, is a bulk step and gets the longer bound instead of an early stop:
///  `pack-objects`, `index-pack`, `add`, `hash-object` and `maintenance` move bytes, and `reset`, `read-tree`,
///  `update-index`, `ls-files`, `diff`, `diff-index`, `show` and `ls-remote` refresh the index, compare it with the
///  worktree or talk to a remote.
pub const PLUMBING_SUBCOMMANDS: [&str; 13] = [
    "rev-parse",
    "rev-list",
    "symbolic-ref",
    "for-each-ref",
    "ls-tree",
    "cat-file",
    "diff-tree",
    "merge-base",
    "write-tree",
    "update-ref",
    "config",
    "remote",
    "version",
];

/// What:
///  The kind of a Git step, which decides its bound and when its first notice comes.
/// Why:
///   The two kinds differ by orders of magnitude in how long a healthy step takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildClass = 'plumbing' | 'object';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildClass {
    /// A query of refs, commits and trees.
    Plumbing,
    /// A step that writes objects, compares the index with the worktree, or is not known to be a query.
    Object,
}

impl ChildClass {
    /// How a notice or a timeout sentence names the kind of step.
    pub fn label(self) -> &'static str {
        return match self {
            Self::Plumbing => "a quick query step",
            Self::Object => "a bulk step",
        };
    }

    /// How long a step of this class may run when the variable is absent or unusable.
    pub fn default_limit(self) -> Duration {
        return match self {
            Self::Plumbing => PLUMBING_CHILD_WAIT,
            Self::Object => OBJECT_CHILD_WAIT,
        };
    }

    /// How long a step of this class runs before its first notice.
    pub fn notice_grace(self) -> Duration {
        return match self {
            Self::Plumbing => PLUMBING_NOTICE_GRACE,
            Self::Object => OBJECT_NOTICE_GRACE,
        };
    }

    /// The bound for a value of the variable: the person's whole seconds, or this class's default for a missing,
    ///  zero or malformed value, never "no limit".
    pub fn limit_from(self, value: Option<&OsStr>) -> Duration {
        let Some(text) = value else {
            return self.default_limit();
        };
        return positive_seconds(CHILD_WAIT_VARIABLE, text).unwrap_or_else(|| { return self.default_limit(); });
    }

    /// The bound this process runs this class under: the variable in its environment, or the class default.
    pub fn limit(self) -> Duration {
        return self.limit_from(std::env::var_os(CHILD_WAIT_VARIABLE).as_deref());
    }
}

/// What:
///  The class of the Git step a request runs.
/// Why:
///   A step is a query only when its subcommand is named in `PLUMBING_SUBCOMMANDS`; every other step, a request with
///  no subcommand included, gets the longer bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// QUERIES.includes(subcommandOf(args)) ? 'plumbing' : 'object'
/// ```
pub fn class_of(arguments: &[OsString]) -> ChildClass {
    let Some((_, name)) = subcommand_of(arguments) else {
        return ChildClass::Object;
    };
    if PLUMBING_SUBCOMMANDS.contains(&name.as_str()) {
        return ChildClass::Plumbing;
    }
    return ChildClass::Object;
}

/// Class controls stay out of the release executable.
#[cfg(test)]
#[path = "child_class_tests.rs"]
mod tests;
