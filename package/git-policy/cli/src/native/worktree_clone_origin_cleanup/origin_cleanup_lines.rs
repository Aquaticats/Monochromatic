//! What:
//!  The lines the origin cleanup prints,
//!  and the rule that decides when a blocked record speaks.
//! Why:
//!  The project decided the look prints one line when it destroys a snapshot and one when a
//!  record's reason changes,
//!  and stays silent while nothing changed
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "How often a pending origin cleanup speaks").
//!  The reason is the code and the detail a person acts on:
//!  the two counts for a dependency,
//!  the mark for a deferral,
//!  the errno for a refusal,
//!  and the record's own words for an unproved identity.
//!  A record's bytes are never printed,
//!  because the record holds another slot's path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.error(lineOfADestruction(name));
//! if (reasonChanged(previous, next)) console.error(lineOfABlock(next, detail));
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_record::{
    CleanupBlock,
    OriginCleanupRecord,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::CleanupRefusal;
use std::path::Path;

/// What:
///  The line one destroyed snapshot prints.
/// Why:
///  It names the snapshot that is gone and what the checkout kept,
///  which is the whole report the project decided outlives the cleanup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lineOfADestruction('pool/checkout@git-worktree-clone-origin-4f0c…');
/// ```
pub fn line_of_a_destruction(snapshot: &str) -> String {
    return format!(
        "the origin cleanup destroyed the snapshot `{snapshot}`; the checkout keeps every other snapshot it holds, and this line is the whole report of that destruction."
    );
}

/// What:
///  The line one blocked record prints when its reason changed.
/// Why:
///  A person reads which snapshot is pinned,
///  why,
///  and where the record that holds the cleanup stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lineOfABlock(record, detail);
/// ```
pub fn line_of_a_block(
    record: &OriginCleanupRecord,
    path: &Path,
    detail: &str,
) -> String {
    let reason: &str = record
        .block
        .map_or("blocked", |held| return held.reason.clause());
    return format!(
        "the origin cleanup of the snapshot `{}` is not finished: {reason} ({detail}); the record at {} keeps the work, a later worktree command tries it again, and nothing expires it.",
        record.full_snapshot_name(),
        path.display()
    );
}

/// What:
///  The line one record this reader refuses prints.
/// Why:
///  The project decided a record that cannot be read is refused and named,
///  and nothing is changed or destroyed on its word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lineOfARefusal(name, refusal, path);
/// ```
pub fn line_of_a_refusal(
    name: &str,
    refusal: &CleanupRefusal,
    path: &Path,
) -> String {
    return format!(
        "the origin cleanup record `{name}` cannot be used, so nothing was destroyed: {}",
        refusal.message(path)
    );
}

/// What:
///  Whether one record's reason differs from the one before it,
///  which is when a blocked record prints.
/// Why:
///  The reason is the code and the detail together:
///  one clone becoming two prints,
///  the same count read again does not,
///  and a record that goes from blocked to destroyed prints the destruction's line instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const speaks = reasonChanged(previous, next);
/// ```
pub fn the_reason_changed(
    previous: Option<&OriginCleanupRecord>,
    next: &OriginCleanupRecord,
) -> bool {
    let before: Option<CleanupBlock> = previous.and_then(|held| return held.block);
    return before != next.block;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_cleanup_lines_tests.rs"]
mod tests;
