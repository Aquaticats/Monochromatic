//! What:
//!  The refresh of an index's stat data:
//! the private commit index's,
//! which the scratch commit reads,
//! and the copy's that a landing installs as the real index.
//! Why:
//!  Git's own `git commit` refreshes the stat data of the index it commits and writes that index back,
//! so a worktree whose files all have new inodes costs one read of every file and every later commit is fast.
//!  This wrapper commits a private copy of the index,
//! so the refresh Git makes lands in the copy and is thrown away,
//! and the copy of every later commit holds the same stale stat data again.
//!  With the ceiling of the scratch step
//! (`SCRATCH_COMMIT_STEP_CEILING`,
//! 16,332 ms,
//! and 3,006 ms at the time of that commit)
//! that stopped a commit on every attempt after the move to the 860 drive,
//! and the same command failed the same way
//! (`doc/handover/cli-git-rust-implementation.md`,
//! "A commit stopped at the scratch ceiling on every retry after the 860 move").
//!  Measured on 2026-11-08 in a throwaway repository whose clean filter sleeps 100 ms per file:
//! a `git commit` over a copy of a stale index paid the whole refresh (1,248 ms for 12 files) and left the real
//! index stale,
//! so a second copy paid it again;
//! the same commit over a refreshed index took 109 ms.
//!  So the refresh runs here,
//! before the scratch commit,
//! where the ceiling of the scratch step does not apply,
//! and it names a private index:
//! a preparation names the copy it commits,
//! which takes that file's own lock,
//! and a landing names the copy it installs,
//! while the landing holds the real index lock anyway
//! (`service_land_follow.rs`).
//!  The real index therefore keeps the refresh,
//! as native Git keeps it,
//! so the commit after this one does not pay it again,
//! and no preparation ever waits for another program's `index.lock`:
//! a batch of overlapping commits no longer serializes on the one file they all read
//! (`doc/decision/cli-git-concurrent-commits.md`).
//!  `update-index --refresh` is a form the cut experiment proved may be run twice
//! (`transaction_git_step_rerun_and_end_rules.rs`),
//! so the step is ended at its class's ceiling and a stop leaves the index as it was.
//!  Git takes `index.lock` on every one of these refreshes,
//!  even the refresh of an index that already describes the worktree,
//!  so the step runs only when an entry's recorded stat data differs from its file's
//! (`commit_index_stat_staleness.rs`):
//!  an up-to-date index costs no Git start and no lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = statRefreshIfStale({ context, cwd, index, staging });
//! ```

/// A Git request naming one index,
/// so the step is classed and bounded as the index writer it is.
use crate::private_index_request::indexed_request;
/// One Git start,
/// its output,
/// and the context it runs in.
use crate::transaction_git::{GitContext, GitOutput, run_git};
/// What a commit stages from the worktree,
/// which decides whose recorded stat data still matters.
use crate::commit_index_stat_staleness::WorktreeStaging;
/// Whether the index still records the worktree's stat data,
/// answered without starting Git.
use crate::commit_index_stat_staleness::index_stat_data_needs_refresh;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What happened to the real index's stat data before the private copies were made.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatRefresh {
    /// The index already described the worktree:
    /// no Git ran and the file was not touched.
    NotNeeded,
    /// Git wrote the worktree's stat data into the index,
    /// so the copies carry it.
    Kept,
    /// Git did not write it:
    /// another program held `index.lock`,
    /// the step was ended at its ceiling,
    /// or it could not start.
    NotKept,
}

/// The exit statuses of `git update-index --refresh` that leave the index holding the worktree's stat data:
/// zero,
/// and one,
/// which Git also returns when a path differs from the index,
/// as an unstaged change makes it.
/// That write carries the refreshed stat data of every other entry.
/// Any other status is a refresh Git did not write,
/// which it reports as fatal,
/// for instance when another program holds `index.lock`.
const STATUSES_THAT_KEEP_THE_REFRESH: [i32; 2] = [0, 1];


/// What:
///  Whether one exit status means the index holds the worktree's stat data after the refresh.
/// Why:
///  `update-index --refresh` reports an unstaged change as a nonzero status that still wrote the refresh,
///  so the status has to be read before the refresh is judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keepsTheRefresh = (code: number) => code === 0 || code === 1;
/// ```
fn keeps_the_refresh(code: i32) -> bool {
    return STATUSES_THAT_KEEP_THE_REFRESH.contains(&code);
}

/// What:
///  Refreshes the stat data `index` records for the worktree,
/// and answers whether Git kept the refresh in that file.
/// Why:
///   A step of this wrapper may not fail a commit:
/// a refresh Git could not write leaves the index as it was,
/// which costs the commit the time of the refresh and never its result,
/// so the answer is a fact to log and to act on,
/// not an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGit(['update-index', '--refresh'], { indexFile: index, allowFailure: true });
/// ```
pub fn refresh_stat_data(context: &GitContext, cwd: &Path, index: &Path) -> bool {
    let arguments: [&str; 2] = ["update-index", "--refresh"];
    let request = indexed_request(context, cwd, arguments.as_slice(), Some(index), None);
    let output: GitOutput = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return without_the_refresh(index, error.to_string().as_str()),
    };
    if output.code.is_some_and(keeps_the_refresh) {
        crate::diagnostic_log::debug("refreshStatData", format!("{} holds the worktree's stat data now", index.display()).as_str());
        return true;
    }
    return without_the_refresh(index, String::from_utf8_lossy(output.stderr.as_slice()).trim());
}

/// What:
///  Records a refresh Git did not keep,
/// and answers that it did not.
/// Why:
///  Both ways of losing the refresh say the same thing,
/// and the record is the only trace of why the next commit took longer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutTheRefresh = (index: string, why: string): boolean => { log.debug(`...: ${why}`); return false; };
/// ```
fn without_the_refresh(index: &Path, why: &str) -> bool {
    crate::diagnostic_log::debug("refreshStatData", format!("the refresh of {} was not kept: {why}", index.display()).as_str());
    return false;
}

/// What:
///  Refreshes `index`'s stat data only when the worktree moved under it.
/// Why:
///  Git's `index.lock` is taken by every `update-index --refresh`,
/// even one that writes nothing,
/// so an index that describes the worktree must cost neither a Git start nor the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = statRefreshIfStale({ context, cwd, index, staging });
/// ```
pub fn refresh_stat_data_when_stale(context: &GitContext, cwd: &Path, index: &Path, staging: WorktreeStaging<'_>) -> StatRefresh {
    // What:
    //  The in-process question,
    //  whose answer decides whether Git runs at all.
    // Why:
    //  A refresh Git ran would take the real index's lock even when it writes nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!indexStatDataNeedsRefresh({ context, cwd, index, staging })) return 'not-needed';
    // ```
    if !index_stat_data_needs_refresh(context, cwd, index, staging) {
        // What:
        //  The answer for an index that already describes the worktree.
        // Why:
        //  The caller must not refresh the private index instead:
        //  there is nothing to refresh.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return 'not-needed';
        // ```
        return StatRefresh::NotNeeded;
    }
    // What:
    //  The refresh Git runs on the real index.
    // Why:
    //  The copies made after it carry the worktree's stat data,
    //  and the real index keeps them for the next commit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const kept: boolean = await refreshStatData({ gitPath, cwd, indexFile: index });
    // ```
    let kept: bool = refresh_stat_data(context, cwd, index);
    // What:
    //  The answer of the refresh Git ran.
    // Why:
    //  The caller decides whether the private index needs the refresh instead,
    //  and a refresh that was not kept is the only case that does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return kept ? 'kept' : 'not-kept';
    // ```
    return if kept { StatRefresh::Kept } else { StatRefresh::NotKept };
}

/// Refreshing an index stays out of the release executable.
#[cfg(test)]
#[path = "commit_index_stat_refresh_tests.rs"]
mod tests;
