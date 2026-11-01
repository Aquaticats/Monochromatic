//! What:
//!  The two kinds of Git step a commit starts for itself, and the bound and the notice grace of each.
//! Why:
//!  One bound for every step had to be long enough for the slowest, so a stalled `rev-parse` held a commit for as long
//!  as a stalled `pack-objects`, and the first notice came after one second for steps that only needed a moment.
//!  The classes follow what a step does and how its time grows, measured on the repository this crate lives in
//!  (`doc/planning/cli-git-commit-timeouts.md`):
//!  a query or plumbing step reads or edits refs and indexes and ends in milliseconds,
//!  while a step that writes objects costs time in proportion to the bytes it moves.
//!  The user's own `git commit` with its hooks is not a class: its runtime belongs to the user and it is never bounded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const limit = classOf(args) === 'plumbing' ? PLUMBING_LIMIT_MS : OBJECT_LIMIT_MS;
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

/// How long a query or plumbing step may run:
///  a hundred times the slowest such step measured on a host at several times its core count in load.
pub const PLUMBING_CHILD_WAIT: Duration = Duration::from_secs(60);

/// How long a step that writes objects may run:
///  its time grows with the bytes it moves, so the bound leaves room for a large file on a loaded host.
pub const OBJECT_CHILD_WAIT: Duration = Duration::from_secs(300);

/// How long a query or plumbing step runs before the first notice.
pub const PLUMBING_NOTICE_GRACE: Duration = Duration::from_secs(5);

/// How long a step that writes objects runs before the first notice.
pub const OBJECT_NOTICE_GRACE: Duration = Duration::from_secs(10);

/// Subcommands that write or move objects, whose time grows with the bytes they handle.
///  Any subcommand not named here and not a known plumbing one is treated the same way, so an unknown step gets the
///  longer bound instead of an early stop.
pub const OBJECT_SUBCOMMANDS: [&str; 7] =
    ["pack-objects", "index-pack", "unpack-objects", "add", "hash-object", "fast-import", "repack"];

/// Subcommands that read, or edit refs and indexes, and end in milliseconds.
pub const PLUMBING_SUBCOMMANDS: [&str; 23] = [
    "rev-parse",
    "rev-list",
    "symbolic-ref",
    "for-each-ref",
    "ls-files",
    "ls-tree",
    "ls-remote",
    "diff",
    "diff-index",
    "diff-tree",
    "cat-file",
    "show",
    "merge-base",
    "write-tree",
    "read-tree",
    "update-index",
    "update-ref",
    "reset",
    "init",
    "config",
    "remote",
    "maintenance",
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
    /// A query, or an edit of refs or an index.
    Plumbing,
    /// A step that writes or moves objects.
    Object,
}

impl ChildClass {
    /// How a notice or a timeout sentence names the kind of step.
    pub fn label(self) -> &'static str {
        return match self {
            Self::Plumbing => "a local plumbing step",
            Self::Object => "an object-writing step",
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
///   A step is classed by its subcommand; one that is not named in `PLUMBING_SUBCOMMANDS` gets the longer bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// PLUMBING.includes(subcommandOf(args)) ? 'plumbing' : 'object'
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
