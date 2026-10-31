//! What:
//!  Recognize the four top-level spellings after which the wrapper says it is not plain Git
//!  (`--version`, `-v`, `--help`, `-h`),
//!  and build the text it prints once Git has answered.
//! Why:
//!  A caller who types `git --version` should be able to tell that a policy wrapper sits in front of Git,
//!  without the wrapper changing a single byte of what Git itself printed.
//!  Every other query (`--exec-path`, `--html-path`, ...) stays an exact process replacement.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const query = announcedQuery(args, globalLayout(args));
//! // after the child exits: if (outcome === { exited: 0 }) process.stdout.write(announcement(query, version, realGit));
//! ```

/// What:
///  How a child ended,
///  which decides whether the wrapper adds text.
/// Why:
///   Only a child that ended with exit code 0 answered the question.
use super::forwarding::ChildOutcome;
/// The result of the global-option scan that says where the query token sits.
use super::global_arguments::{GlobalLayout, GlobalOutcome};
/// Native argument storage;
///  no lossy UTF-8 conversion occurs.
use std::ffi::OsString;
/// Borrowed filesystem path of raw OS bytes.
use std::path::Path;

/// What:
///  The crate version printed in the banner.
///       `env!` reads a Cargo-provided environment variable while compiling.
/// Why:
///   The banner must name the wrapper build that answered,
///  and the manifest is the one place that version lives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WRAPPER_VERSION = packageJson.version;
/// ```
pub const WRAPPER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Which announced query an invocation is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnnouncedQuery {
    /// `--version` or `-v`:
    ///  one line follows Git's version line.
    Version,
    /// `--help` or `-h`:
    ///  a short section follows Git's usage text.
    Help,
}

/// What:
///  Say whether the query token of an invocation is one of the four announced spellings.
///       `Option<AnnouncedQuery>` is "the query, or nothing".
/// Why:
///   The layout already told the caller this is a query and where it sits;
///  only the exact token separates `--version` from `--exec-path`.
///  Nothing else is read, so commands that are not queries pay one comparison.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function announcedQuery(args: string[], layout: GlobalLayout): 'version' | 'help' | undefined;
/// ```
pub fn announced_query(arguments: &[OsString], layout: &GlobalLayout) -> Option<AnnouncedQuery> {
    if layout.outcome != GlobalOutcome::Query {
        return None;
    }
    // `.as_encoded_bytes()` lends the raw bytes of the query token.
    match arguments[layout.prefix_len].as_encoded_bytes() {
        b"--version" | b"-v" => return Some(AnnouncedQuery::Version),
        b"--help" | b"-h" => return Some(AnnouncedQuery::Help),
        _ => return None,
    }
}

/// What:
///  Build the text printed after Git's own answer.
///       `&str` borrows the version;
///  `&Path` borrows the resolved real Git path;
///  the result is owned, line-terminated text.
/// Why:
///   The banner states which wrapper build and which Git answered;
///  the help section also says what the wrapper adds.
///  Building it from plain values keeps it testable without a process.
///  A path that is not UTF-8 prints with replacement characters, because the text is for people.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function announcement(query: AnnouncedQuery, version: string, realGit: string): string;
/// ```
pub fn announcement(query: AnnouncedQuery, version: &str, real_git: &Path) -> String {
    match query {
        AnnouncedQuery::Version => {
            return format!(
                "monochromatic-git-policy-cli {version}, shadowing git at {}\n",
                real_git.display()
            );
        }
        AnnouncedQuery::Help => {
            return format!(
                "\nmonochromatic-git-policy-cli {version} sits in front of this git\n\
                 It adds repository policies on commit, push and worktree commands,\n\
                 the `git cli-git` management command (`git cli-git --help`),\n\
                 and `git cli-git doctor`, which lists the traditional hooks a repository holds.\n\
                 real git: {}\n",
                real_git.display()
            );
        }
    }
}

/// What:
///  The part of the announcement to print for a finished child.
///       The result borrows from `text`.
/// Why:
///   Git that failed or was killed did not answer,
///  so the caller sees Git's status and Git's own diagnostics unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function printableAfter(outcome: ChildOutcome, text: string): string { return outcome.exited === 0 ? text : ''; }
/// ```
pub fn printable_after(outcome: ChildOutcome, text: &str) -> &str {
    if outcome == ChildOutcome::Exited(0) {
        return text;
    }
    return "";
}

/// Layout, text and status controls stay out of the release executable.
#[cfg(test)]
#[path = "announced_query_tests.rs"]
mod tests;
