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
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// What a commit stages from the worktree,
/// which decides whose recorded stat data still matters.
use crate::commit_index_stat_staleness::WorktreeStaging;
/// What the walk found about the index's stat data,
/// answered without starting Git,
/// and the paths a refresh has to look at.
use crate::commit_index_stat_staleness::{StaleStatData, stale_stat_data};
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

/// One stale path in this many entries of the index is the most a refresh names before it refreshes the whole index.
/// Citation:
/// measured, and the measurement is repeated here because the number is a cost, not a limit.
/// One fixture of 2,000 files, an index whose entries hold the stat data of their files, and every entry's
///  modification time moved but its content untouched, so the walk names exactly the paths that were moved. The
///  whole-index form (`update-index --refresh`) cost 4,031 `newfstatat` and 166 to 230 ms whatever was named. The
///  named form (`add --refresh` with the paths it is given) cost 31 questions plus three for each named path
/// (`doc/planning/cli-git-commit-latency-under-many-agents.md`, the section this constant was measured for):
///  34 for one path, 331 for a hundred, 1,531 for half the index, 6,031 for the whole of it. So the named form wins
///  while its paths are at most two thirds of the entries, and half of them is named here, with the margin the
///  measurement's shape leaves. The stale paths are the files the person changed since the landing that refreshed the
///  index, so a commit normally names one or two.
pub const REFRESH_NAMES_ONE_PATH_IN: usize = 2;

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
///  Refreshes the stat data `index` records for `paths`,
/// and answers whether Git kept the refresh in that file.
/// Why:
///  `git add --refresh` refreshes exactly the entries its pathspecs name, and one question per named path is what a
///  commit needs: the whole-index form asks about every entry of the index, which on a worktree of this repository is
///  tens of thousands of questions the commit waits on for no answer it uses.
///  `:(top,literal)` names each path as the index stores it, from the worktree's top level, whatever directory the
///  command runs in and whatever byte the path holds.
///  A path Git cannot match is a refresh that did not happen, which is the answer the whole-index form gives when it
///  cannot start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGit(['add', '--refresh', '--pathspec-from-file=-', '--pathspec-file-nul'], { indexFile: index, input: paths.map((p) => ':(top,literal)' + p).join('\\0') });
/// ```
pub fn refresh_named_stat_data(context: &GitContext, cwd: &Path, index: &Path, paths: &[Vec<u8>]) -> bool {
    let mut input: Vec<u8> = Vec::new();
    for path in paths {
        input.extend_from_slice(b":(top,literal)");
        input.extend_from_slice(path);
        input.push(0);
    }
    let arguments: [&str; 4] = ["add", "--refresh", "--pathspec-from-file=-", "--pathspec-file-nul"];
    let mut request: GitRequest = indexed_request(context, cwd, &arguments, Some(index), None);
    request.input = Some(input);
    let output: GitOutput = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return without_the_refresh(index, error.to_string().as_str()),
    };
    if output.code.is_some_and(keeps_the_refresh) {
        crate::diagnostic_log::debug("refreshStatData", format!("{} holds the stat data of {} paths now", index.display(), paths.len()).as_str());
        return true;
    }
    return without_the_refresh(index, String::from_utf8_lossy(output.stderr.as_slice()).trim());
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
    //  whose answer decides whether Git runs at all, and what it is asked to look at.
    // Why:
    //  A refresh Git ran would take the real index's lock even when it writes nothing,
    //  and the paths the walk names are the whole of what it has to look at.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const answer = staleStatData({ context, cwd, index, staging });
    // if (answer.kind === 'current') return 'not-needed';
    // if (answer.kind === 'stale' && answer.paths.length * REFRESH_NAMES_ONE_PATH_IN <= answer.entries) return await refreshNamedStatData(...);
    // ```
    let named: Option<Vec<Vec<u8>>> = match stale_stat_data(context, cwd, index, staging) {
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
        StaleStatData::Current => return StatRefresh::NotNeeded,
        // What:
        //  The paths Git is asked to refresh,
        //  and nothing when naming them would cost more than the whole index.
        // Why:
        //  Git matches every pathspec against the index, so a list as long as the index itself costs more than
        //  refreshing everything (`REFRESH_NAMES_ONE_PATH_IN`).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const named = answer.kind === 'stale' && answer.paths.length * REFRESH_NAMES_ONE_PATH_IN <= answer.entries ? answer.paths : undefined;
        // ```
        StaleStatData::Stale { paths, entries } => (paths.len() * REFRESH_NAMES_ONE_PATH_IN <= entries).then_some(paths),
        StaleStatData::Unknown => None,
    };
    // What:
    //  The refresh Git runs, over the paths the walk named or over the whole index.
    // Why:
    //  The copies made after it carry the worktree's stat data,
    //  and the real index keeps them for the next commit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const kept = named === undefined ? await refreshStatData({...}) : await refreshNamedStatData({..., paths: named});
    // ```
    let kept: bool = match named {
        Some(paths) => refresh_named_stat_data(context, cwd, index, paths.as_slice()),
        None => refresh_stat_data(context, cwd, index),
    };
    return if kept { StatRefresh::Kept } else { StatRefresh::NotKept };
}

/// Refreshing an index stays out of the release executable.
#[cfg(test)]
#[path = "commit_index_stat_refresh_tests.rs"]
mod tests;
