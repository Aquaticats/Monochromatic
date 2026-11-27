//! What:
//!  The question whether an index still records the stat data its worktree has,
//! answered in process.
//! Why:
//!  The scratch `git commit` reads and hashes every file whose index entry records stat data the worktree no longer
//!  has,
//!  and that read is what the scratch step's ceiling ends
//!  (`commit_index_stat_refresh.rs`).
//!  A refresh of the real index pays those reads where the ceiling does not apply and keeps them,
//!  but every `git update-index --refresh` takes Git's `index.lock` on the real index,
//!  even the refresh of an index that already describes the worktree:
//!  measured on 2026-11-08 in throwaway repositories,
//!  `update-index --refresh` creates `index.lock` on every run and renames it over the index only when it writes.
//!  So the refresh is asked for only when an entry's recorded stat data differs from the file's.
//!  An index Git calls racily clean is not asked for:
//!  its entries record the stat data their files have,
//!  and three refreshes of an index written in the same second as its entry left the index's bytes identical (the same
//!  SHA-1),
//!  so such a refresh writes nothing the private copies could carry,
//!  and the scratch commit reads those entries just as native `git commit` does.
//!  The entries of paths the commit stages from the worktree are skipped as well,
//!  because `git add` writes their stat data into the private index afresh.
//!  Anything this reading does not fully understand answers "stale",
//!  and Git's own refresh decides.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stale: boolean = indexStatDataNeedsRefresh({ context, cwd, index, staging });
//! ```

/// The one reading of an index file the answer is computed from.
use crate::commit_index_stat_staleness_reading::read_in_process;
/// What the question is asked with,
///  and the answer it gives.
pub use crate::commit_index_stat_staleness_staging::WorktreeStaging;
pub use crate::commit_index_stat_staleness_walk::StaleStatData;
/// The context the question runs in.
use crate::transaction_git::GitContext;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  What `index` records about the stat data of the worktree,
/// answered in process for every entry `staging` does not write afresh.
/// Why:
///  A commit the worktree has not moved needs no refresh,
/// and asking Git would cost it a start and the real index's lock;
/// a commit whose worktree moved gets the paths a refresh has to look at,
///  which costs it a question per stale path instead of one per entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = staleStatData({ context, cwd, index, staging }); // unknown: ask Git
/// ```
#[cfg(unix)]
pub fn stale_stat_data(
    context: &GitContext,
    cwd: &Path,
    index: &Path,
    staging: WorktreeStaging<'_>,
) -> StaleStatData {
    // The reading is the preparation's own work: it parses the whole index and stats every entry the commit leaves alone,
    // so a record that names it is the only thing that can say whether a slow preparation was this question.
    let began: std::time::Instant = std::time::Instant::now();
    let answer: StaleStatData = read_in_process(context, cwd, index, staging);
    crate::prepare_work_spans::note_span(
        crate::prepare_work_spans::INDEX_STAT_CHECK,
        began.elapsed(),
    );
    return answer;
}

/// Where there is no `stat` this reading knows,
/// Git's own refresh always decides.
#[cfg(not(unix))]
pub fn stale_stat_data(
    _context: &GitContext,
    _cwd: &Path,
    _index: &Path,
    _staging: WorktreeStaging<'_>,
) -> StaleStatData {
    return StaleStatData::Unknown;
}

/// Controls for the question itself,
/// kept in a file of their own so the reading above stays readable.
#[cfg(test)]
#[path = "commit_index_stat_staleness_tests.rs"]
mod tests;
