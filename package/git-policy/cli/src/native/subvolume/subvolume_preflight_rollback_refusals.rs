//! What:
//!  The refusals of a rollback and of deleting the old tree,
//!  and the identity that tells the installed checkout from whatever stands at its path now.
//! Why:
//!  Without a journal the command cannot tell which directory is the original and which the copy,
//!  so a rollback needs one.
//!  The deletion of the old tree carries what is newer in it into the checkout first and then deletes it,
//!  and the rollback exchanges the two directories,
//!  so either one acts on the wrong place when the checkout was moved or replaced by hand after the migration.
//!  A journal written by an earlier release records no identity,
//!  and without a record the command does not refuse on a guess.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusals = rollbackRefusals(layout, journal);
//! const refusals = removeOldRefusals(layout, journal);
//! ```

use crate::subvolume_journal::Journal;
use crate::subvolume_preflight_paths::Layout;
use crate::subvolume_preflight_refusal::{
    Refusal,
    refuse,
};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// The journal value that records the identity of the subvolume the migration installed at the checkout,
/// as `device:inode`.
pub const INSTALLED_ROOT: &str = "installed-root";

/// What:
///  The identity of the directory at `path` as `device:inode`,
/// or nothing when it cannot be examined.
/// Why:
///   A name says nothing about which directory it holds now;
/// a device and an inode do,
/// and they survive a rename and an exchange.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = installedIdentity(root); // '0:256' | undefined
/// ```
pub fn installed_identity(path: &Path) -> Option<String> {
    let metadata: std::fs::Metadata = std::fs::metadata(path).ok()?;
    return Some(format!("{}:{}", metadata.dev(), metadata.ino()));
}

/// What:
///  The refusal for a checkout that is not the directory the journal records installing,
/// or nothing when the journal records none or the directory is that one.
/// Why:
///   The deletion of the old tree carries what is newer in the old tree into the checkout before it deletes it,
/// and the
///  rollback exchanges the two directories:
/// either acts on the wrong place when the checkout was moved or replaced by hand
///  after the migration,
/// which writes into a directory the migration never recorded and then deletes the old tree.
/// A
///  journal written by an earlier release records no identity,
/// and without a record the command cannot tell,
/// so it
///  does not refuse on a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = wrongRootRefusal(layout, journal); // undefined when the checkout is the installed one
/// ```
pub fn wrong_root_refusal(
    layout: &Layout,
    journal: &Journal,
) -> Option<Refusal> {
    let recorded: &str = journal.value(INSTALLED_ROOT)?;
    let now: String = installed_identity(layout.root.as_path())?;
    if now == recorded {
        return None;
    }
    return Some(refuse(
        format!(
            "{} is not the directory installed by the migration there: it now has the identity {now}, and {} records {recorded}",
            layout.root.display(),
            journal.path().display()
        ),
        "If the checkout was moved or replaced, move the directory the migration installed back to that path and run the command again. Nothing was changed.",
    ));
}

/// What:
///  The refusals of a rollback:
/// it needs a journal.
/// Why:
///   Without a record the command cannot tell which directory is the original and which the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusals = rollbackRefusals(layout, journal);
/// ```
pub fn rollback_refusals(
    layout: &Layout,
    journal: &Journal,
) -> Vec<Refusal> {
    if !journal.is_persisted() {
        return vec![refuse(
            format!(
                "there is no migration journal at {}, so there is nothing recorded to undo",
                layout.journal_path.display()
            ),
            "If a migration was started by hand, undo it by hand: exchange the directories again with `mv --exchange` and delete the new subvolume.",
        )];
    }
    // Once the rollback's own exchange ran, the checkout is the original directory again, and a rerun finishes from there.
    if journal.has_begun("rollback-swap") || journal.state("exchange") == Some("rolled-back") {
        return Vec::new();
    }
    return match wrong_root_refusal(layout, journal) {
        Some(found) => vec![found],
        None => Vec::new(),
    };
}

/// What:
///  The refusals of deleting the old tree:
/// the migration must have finished and the old tree must exist.
/// Why:
///   The old tree is the only copy of the files this user could not read,
/// and of anything not yet carried over;
/// it is
///  deleted only when the person asks,
/// after a verified migration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusals = removeOldRefusals(layout, journal);
/// ```
pub fn remove_old_refusals(
    layout: &Layout,
    journal: &Journal,
) -> Vec<Refusal> {
    let mut found: Vec<Refusal> = Vec::new();
    if !journal.is_done("verify") {
        found.push(refuse(
            format!("the migration of {} is not recorded as finished in {}", layout.root.display(), layout.journal_path.display()),
            "Run `git monochromatic-git-policy-cli migrate-to-subvolume` to finish (or `--rollback` to undo) before deleting the old tree.",
        ));
    }
    if !layout.old_path.exists() {
        found.push(refuse(
            format!("{} does not exist", layout.old_path.display()),
            "Nothing to delete: it was removed already.",
        ));
    }
    if let Some(wrong) = wrong_root_refusal(layout, journal) {
        found.push(wrong);
    }
    return found;
}
