//! What:
//!  The one reading of an index file the stat-data question is answered from:
//!  its bytes read whole under a bound,
//!  its entries parsed in process,
//!  and the walk of those entries.
//! Why:
//!  Every entry and its stat data is in the file's bytes,
//!  so no Git is needed to ask the question;
//!  the reading and the walk are kept apart from it so the answer stays readable.
//!  Anything this reading does not fully understand answers "stale",
//!  and Git's own refresh decides.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const readInProcess = (context, cwd, index, staging) => { const bytes = readIndex(index); ... };
//! ```

use crate::commit_index_stat_staleness_left_to_git::restaged_paths;
use crate::commit_index_stat_staleness_staging::WorktreeStaging;
use crate::commit_index_stat_staleness_walk::{
    EntrySlice,
    StaleStatData,
    stale_paths_in_slice,
    walk_in_slices,
    workers_for,
};
/// The whole parsed index,
///  and the entries it holds.
use crate::index_full::{
    FullIndex,
    parse_full_index,
};
use crate::private_storage::{
    ReadRefusal,
    read_regular_file,
};
/// The worktree layout the walk reads,
///  and the repository's object hash length.
use crate::repo_read_layout::{
    RepoLayout,
    read_layout_in,
};
use crate::repo_read_object::name_length;
/// The context the reading runs in.
use crate::transaction_git::GitContext;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// The most bytes of an index file this reads:
/// the bound the in-process query layer already puts on the same file
/// (`repo_read_query_request_answer_and_context.rs`),
/// not a new measurement.
#[cfg(unix)]
const INDEX_LIMIT: u64 = 1 << 30;

/// What:
///  The reading itself:
/// every entry the commit leaves alone is compared with its file,
/// and every doubt answers "stale".
/// Why:
///  `index_stat_data_needs_refresh` is the one caller,
/// and separating the reading keeps that answer readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readInProcess = ({ context, cwd, index, staging }) => {
///   const bytes = readIndex(index);          // absent: nothing to refresh
///   const layout = readLayout(cwd, overlay(context));
///   const parsed = parseFullIndex(bytes, layout.objectFormat);
///   const restaged = restagedPaths(staging, layout.prefix);
///   const stale = walkInSlices(parsed.entries, layout.worktreeRoot, restaged, workersFor(parsed.entries.length));
///   return stale === undefined ? UNKNOWN : stale.length === 0 ? CURRENT : STALE(stale, parsed.entries.length);
/// };
/// ```
#[cfg(unix)]
pub(crate) fn read_in_process(
    context: &GitContext,
    cwd: &Path,
    index: &Path,
    staging: WorktreeStaging<'_>,
) -> StaleStatData {
    // What:
    //  The index file's bytes.
    // Why:
    //  Every entry and its stat data is in them,
    //  so no Git is needed to ask the question.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const bytes = readRegularFile(index, INDEX_LIMIT);
    // ```
    let bytes: Vec<u8> = match read_regular_file(index, INDEX_LIMIT) {
        Ok(read) => read,
        // A repository nothing was ever staged in has no index, and so no stat data to refresh.
        Err(ReadRefusal::Missing) => return StaleStatData::Current,
        // An index this reading cannot open is one Git answers for.
        Err(_) => return StaleStatData::Unknown,
    };
    // What:
    //  The environment the directory is read with,
    //  including the real Git a child would run.
    // Why:
    //  The in-process reader declines some configurations,
    //  and the answer then has to be "stale".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const overlay = withRealGit(context.overlay, context.realGit);
    // ```
    let overlay: Vec<(std::ffi::OsString, std::ffi::OsString)> =
        crate::git_system_config::with_real_git(
            context.overlay.as_slice(),
            context.real_git.as_path(),
        );
    // What:
    //  The worktree's top level, the start directory below it and the repository's object hash.
    // Why:
    //  Index paths are relative to the top level, which need not be the directory the command runs in,
    //  and the hash length says where the index's entries end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const layout = readLayoutIn(cwd, overlay);
    // ```
    let layout: RepoLayout = match read_layout_in(cwd, overlay.as_slice()) {
        Some(read) => read,
        None => return StaleStatData::Unknown,
    };
    // What:
    //  Every entry of the index with its stat data.
    // Why:
    //  A form this reader does not understand is one Git has to answer for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parsed = parseFullIndex(bytes, nameLength(layout.objectFormat));
    // ```
    let parsed: FullIndex =
        match parse_full_index(bytes.as_slice(), name_length(layout.object_format)) {
            Some(read) => read,
            None => return StaleStatData::Unknown,
        };
    // What:
    //  The top-relative paths this commit stages from the worktree.
    // Why:
    //  Their recorded stat data is replaced by `git add`,
    //  so a stale record of theirs costs the scratch commit nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const restaged = restagedPaths(staging, layout.prefix);
    // ```
    let restaged: Vec<Vec<u8>> = restaged_paths(&staging, layout.prefix.as_slice());
    // What:
    //  The paths whose entries record stat data their files no longer have,
    // walked in slices on several threads
    //  where the index is large enough for the walk to pay for them (`stale_paths_in_slice`).
    // Why:
    //  Git's own stat pass preloads the same work across threads (`core.preloadIndex`),
    // and one thread of this walk took 47 to
    //  58 ms on a checkout of 22,161 entries against Git's 10 ms;
    // over 22,200 files a walk of one thread took 160.12 ms and seven
    //  took 30.56 ms (`STAT_WALK_BREAK_EVEN_ENTRIES`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stale = walkInSlices(parsed.entries, layout.worktreeRoot, restaged, workersFor(parsed.entries.length));
    // ```
    let workers: usize = workers_for(parsed.entries.len());
    let slice: EntrySlice<'_> = EntrySlice {
        entries: parsed.entries.as_slice(),
        worktree_root: layout.worktree_root.as_path(),
        restaged: restaged.as_slice(),
    };
    let walked: Option<Vec<Vec<u8>>> = match workers {
        1 => Some(stale_paths_in_slice(slice)),
        _ => walk_in_slices(slice, workers),
    };
    let Some(stale) = walked else {
        return StaleStatData::Unknown;
    };
    if stale.is_empty() {
        return StaleStatData::Current;
    }
    return StaleStatData::Stale {
        paths: stale,
        entries: parsed.entries.len(),
    };
}
