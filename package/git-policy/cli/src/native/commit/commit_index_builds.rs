//! What:
//!  How each build of a commit's private index was made,
//! and whether it kept the real index's cached stat data,
//! kept until the commit's timing record takes the figures.
//! Why:
//!   An index built from the base or the tree alone holds no stat data,
//! so the refresh that follows it hashes every file of the work tree,
//! LFS-tracked files through the clean filter:
//! measured in this repository on 2026-11-07,
//! 233,618,538 bytes cleaned and 3.6 s against 0.1 s with the real index's stat data
//! (`doc/handover/cli-git-rust-implementation.md`,
//! "Slow rebuilds and the moved-branch bound").
//! The first build of a commit that took 171,383 ms on 2026-11-08 was never explained,
//! because the two lines that said which way a build went were written only under `MONOCHROMATIC_VERBOSE`,
//! which nothing sets in normal use.
//! The timing record is written on every commit,
//! so it carries the fact instead.
//! One entry per build,
//! in the order built:
//! the first entry is the first preparation's build,
//! and a later one is a rebuild after the branch moved.
//! The command's own process builds the index,
//! so every build of one command is in that command's record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const builds: { kind: 'first' | 'retry'; statDataKept: boolean }[] = [];
//! export const noteIndexBuild = (kind, statDataKept) => builds.push({ kind, statDataKept });
//! export const drainIndexBuilds = () => builds.splice(0);
//! ```

/// Escaping of a word inside a JSON string.
use crate::json_record::quote;
/// A lock around the shared list.
use std::sync::Mutex;

/// Which preparation built the private index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexBuildKind {
    /// The command's own preparation,
    /// which builds the index it commits from.
    First,
    /// A rebuild after the branch moved,
    /// made from the tree the replay produced.
    Retry,
}

impl IndexBuildKind {
    /// The word the timing record states this kind as.
    pub fn word(self) -> &'static str {
        return match self {
            Self::First => "first",
            Self::Retry => "retry",
        };
    }
}

/// One build of a commit's private index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexBuild {
    /// Which preparation built it.
    pub kind: IndexBuildKind,
    /// Whether the build started from a copy of the real index,
    /// so every entry the built tree shares with it keeps its cached stat data.
    /// A build from the base or the tree alone holds none,
    /// and the refresh that follows it hashes every file of the work tree.
    pub stat_data_kept: bool,
}

/// The builds of this process since a record last took them,
/// in the order built.
/// Lock kept:
///  one `Vec` the commit's own preparation pushes to and the timing record takes,
///  held for that
///  push or take and nothing else:
///  no Git process,
///  no file and no other lock runs under it,
///  and a build
///  can be noted from more than one thread of a command.
static BUILDS: Mutex<Vec<IndexBuild>> = Mutex::new(Vec::new());

/// What:
///  Keep one build of the private index.
/// Why:
///   Both ways of building the index report here,
/// so the record says of every build whether it kept the stat data,
/// and a commit that builds again after the branch moved is not confused with its first build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteIndexBuild('first', true);
/// ```
pub fn note_index_build(kind: IndexBuildKind, stat_data_kept: bool) {
    // A poisoned lock means another thread panicked while holding it; the list is still a list of builds.
    let mut builds = BUILDS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    builds.push(IndexBuild { kind, stat_data_kept });
}

/// What:
///  Take every build kept and forget them,
/// in the order built.
/// Why:
///   One record takes the builds of the command it describes;
/// the next command starts from nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const builds = drainIndexBuilds();
/// ```
pub fn drain_index_builds() -> Vec<IndexBuild> {
    let mut builds = BUILDS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    return std::mem::take(&mut *builds);
}

/// What:
///  One build as a JSON object:
/// `{"kind":"first","statDataKept":true}`.
/// Why:
///   `index_builds_json` joins them into the record's list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ kind, statDataKept })
/// ```
fn build_json(build: &IndexBuild) -> String {
    return format!("{{\"kind\":{},\"statDataKept\":{}}}", quote(build.kind.word()), build.stat_data_kept);
}

/// What:
///  The builds as a JSON array,
/// in the order given.
/// Why:
///   The client's timing record carries it under `indexBuilds`,
/// which is empty for a commit that never built an index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(builds)
/// ```
pub fn index_builds_json(builds: &[IndexBuild]) -> String {
    let members: Vec<String> = builds.iter().map(build_json).collect();
    return format!("[{}]", members.join(","));
}

/// The ledger,
/// its order and its escaping are kept as controls in the test module beside it.
#[cfg(test)]
#[path = "commit_index_builds_tests.rs"]
mod tests;
