//! What:
//!  The line that tells the person a snapshot of a checkout with changes left the changes out,
//! and the journal record of what the add did with a dirty checkout.
//! Why:
//!   The new worktree matches a clean `git worktree add`,
//! which the person may not expect from a copy of a dirty checkout,
//! and the line on standard error is gone when the command ends.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const note = leftOutNote(changed);
//! journal.info(note, { CLI_GIT_WORKTREE: destination, CLI_GIT_DIRTY_PATHS: count });
//! ```

use crate::worktree_snapshot_source::ChangedPaths;
use std::path::Path;

pub fn left_out_note(changed: &ChangedPaths) -> String {
    let paths: &str = if changed.count == 1 { "path" } else { "paths" };
    return format!(
        "monochromatic-git-policy-cli: the uncommitted changes of the main checkout ({} {paths}) were left out of the new worktree, which holds what Git's own checkout holds.\n",
        changed.count
    );
}

/// What:
///  Record in the journal,
/// at the ordinary level,
/// that a snapshot left the uncommitted changes of the main checkout out of the
///  new worktree,
/// with the destination and the number of paths as fields.
/// Why:
///   The line on standard error is gone when the command ends;
/// the journal keeps what the add did with a dirty checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// journal.info(note, { CLI_GIT_WORKTREE: destination, CLI_GIT_DIRTY_PATHS: count });
/// ```
pub fn record_left_out(
    destination: &Path,
    changed: &ChangedPaths,
) {
    let note: String = left_out_note(changed);
    let fields: [(&str, String); 3] = [
        (
            "CLI_GIT_WORKTREE",
            destination.to_string_lossy().into_owned(),
        ),
        ("CLI_GIT_DIRTY_PATHS", changed.count.to_string()),
        (
            "CLI_GIT_DIRTY_FIRST",
            changed.first.clone().unwrap_or_default(),
        ),
    ];
    crate::client_journal::record_entry(
        crate::journald_native::Priority::Info,
        note.trim_end(),
        &fields,
    );
}
