//! What:
//!  The check that an explicit-path landing changed no staged entry outside the paths it committed.
//! Why:
//!  Staged work from other sessions once vanished and nothing said which step removed it. The landing computes its
//!  post-index from a copy of the live index and resets only the committed and guarded paths, so every other entry
//!  must read the same afterwards. The check compares the two indexes and warns, naming the paths, when it does
//!  not; it never changes the landing, whose outcome is already decided.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const moved = pathsChangedOutside(listEntries(pre), listEntries(post), allowed);
//! if (moved.length > 0) warn(`staged entries changed outside the commit: ${moved}`);
//! ```

use super::auto_push_dirty_paths::listed;
use super::diagnostic_log::{debug, warn};
use super::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
use std::collections::BTreeMap;
use std::path::Path;

/// Module tag for diagnostics.
const TAG: &str = "landingInvariant";

/// What:
///  The stage records of an `ls-files --stage -z` listing, grouped by path.
///  A `BTreeMap` keeps its keys sorted.
/// Why:
///   A path holds one record per merge stage, and two listings agree on a path only when all of its records agree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordsByPath(listing: Uint8Array): Map<string, Uint8Array[]>;
/// ```
fn records_by_path(listing: &[u8]) -> BTreeMap<Vec<u8>, Vec<Vec<u8>>> {
    let mut grouped: BTreeMap<Vec<u8>, Vec<Vec<u8>>> = BTreeMap::new();
    for record in listing.split(|byte| { return *byte == 0; }) {
        // A record is `<mode> <object> <stage>\t<path>`; a record with no tab is not one.
        let Some(tab) = record.iter().position(|byte| { return *byte == b'\t'; }) else {
            continue;
        };
        grouped.entry(record[tab + 1..].to_vec()).or_default().push(record.to_vec());
    }
    return grouped;
}

/// What:
///  The paths whose entries differ between two listings and are not allowed to differ, sorted.
/// Why:
///   A path in the allowed list is one the landing is meant to rewrite; any other difference is a loss or a leak.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathsChangedOutside(pre: Uint8Array, post: Uint8Array, allowed: Uint8Array[]): Uint8Array[];
/// ```
pub fn paths_changed_outside(pre: &[u8], post: &[u8], allowed: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let before = records_by_path(pre);
    let after = records_by_path(post);
    let mut moved: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for path in before.keys().chain(after.keys()) {
        if before.get(path) != after.get(path) && !allowed.contains(path) && !moved.contains(path) {
            moved.push(path.clone());
        }
    }
    moved.sort();
    return moved;
}

/// What:
///  Lists every staged entry of one index.
/// Why:
///   The whole index is compared, so an entry no one expected to move is still seen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await listEntries({ context, cwd, index });
/// ```
fn list_entries(context: &GitContext, cwd: &Path, index: &Path) -> Result<Vec<u8>, GitFailure> {
    let mut request = GitRequest::new(cwd, &["ls-files", "--stage", "-z", "--full-name"]);
    request.index = Some(index.to_path_buf());
    let output: GitOutput = run_git_checked(context, &request)?;
    return Ok(output.stdout);
}

/// What:
///  Warn when the post-index differs from the pre-landing index outside the allowed paths, and return the text
///  that was logged, or nothing when the indexes agree or one could not be read.
///  A failure to read either index is a debug line, not an error.
/// Why:
///   The check exists to attribute a loss; it must not turn a finished landing into a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text: string | undefined = await warnWhenEntriesMoved({ context, cwd, pre, post, allowed });
/// ```
pub fn warn_when_entries_moved(
    context: &GitContext,
    cwd: &Path,
    pre_landing_index: &Path,
    post_index: &Path,
    allowed: &[Vec<u8>],
) -> Option<String> {
    let listings = list_entries(context, cwd, pre_landing_index)
        .and_then(|pre| { return list_entries(context, cwd, post_index).map(|post| { return (pre, post); }); });
    let (pre, post) = match listings {
        Ok(found) => found,
        Err(failure) => {
            debug(TAG, &format!("the indexes could not be compared: {}", failure.0));
            return None;
        }
    };
    let moved: Vec<Vec<u8>> = paths_changed_outside(&pre, &post, allowed);
    if moved.is_empty() {
        return None;
    }
    let message: String = format!(
        "The landing changed staged entries outside the paths it committed: {}. Your staging should have been left as it was; \
         compare with git diff --cached, and report the commit that did this with the lines of cli-git-audit.jsonl in the common Git directory.",
        listed(&moved)
    );
    warn(TAG, &message);
    return Some(message);
}

/// Pure comparison and the warning over real indexes.
#[cfg(test)]
#[path = "landing_invariant_tests.rs"]
mod tests;
