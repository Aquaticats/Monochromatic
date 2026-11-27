//! What:
//!  The refusals that stop a fresh or resumed migration before anything is created:
//!  the filesystem,
//!  the free space,
//!  the parent device,
//!  a mount point,
//!  what an earlier run or someone else left beside the checkout,
//!  and the stage name that is no longer the subvolume this migration recorded making.
//! Why:
//!  Converting a checkout swaps its directory under every process and worktree,
//!  so the migration starts only when the filesystem,
//!  the checkout and the state left by an earlier run allow it,
//!  and the journal is what says which leftovers are this migration's own.
//!  No condition spawns a process and none opens a file:
//!  the free-space check lists every directory of the tree once and counts its entries.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusals = migrateRefusals(platform, layout, journal); // empty means it may start
//! ```

use crate::fs_kind::FilesystemKind;
use crate::subvolume_entry_count::count_entries;
use crate::subvolume_journal::Journal;
use crate::subvolume_platform::Platform;
use crate::subvolume_preflight_git_state_refusals::git_state_refusals;
use crate::subvolume_preflight_paths::{
    Layout,
    METADATA_BYTES_PER_ENTRY,
    METADATA_COPIES,
    required_free_bytes,
};
use crate::subvolume_preflight_refusal::{
    Refusal,
    refuse,
};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// What:
///  The refusals of a fresh or resumed migration;
/// empty means it may start or continue.
/// Why:
///   See the module description.
/// The journal says which leftovers are this migration's own:
/// the new tree after its
///  `create` step began,
/// the old tree after the `exchange` step began,
/// `.git/index.lock` after the `lock` step began.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusals = migrateRefusals(platform, layout, journal);
/// ```
pub fn migrate_refusals(
    platform: &dyn Platform,
    layout: &Layout,
    journal: &Journal,
) -> Vec<Refusal> {
    let mut found: Vec<Refusal> = Vec::new();
    let root: &Path = layout.root.as_path();
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(root) {
        Ok(read) => read,
        Err(error) => {
            found.push(refuse(
                format!("{} cannot be examined: {error}", root.display()),
                "Run the command inside the repository's main checkout, or check the path.",
            ));
            return found;
        }
    };
    if !metadata.is_dir() {
        found.push(refuse(
            format!(
                "{} is not a directory (it is a symbolic link or a file)",
                root.display()
            ),
            "Run the command from the real path of the main checkout.",
        ));
        return found;
    }
    match platform.filesystem_facts(root) {
        Ok((FilesystemKind::Btrfs, _)) => {}
        Ok((kind, _)) => found.push(refuse(
            format!("{} is on {}, not on btrfs, and only btrfs has subvolumes", root.display(), if kind == FilesystemKind::Overlayfs { "overlayfs" } else { "a filesystem other than btrfs" }),
            "Nothing to convert here: on this filesystem `git worktree add` uses Git's checkout (on overlayfs the wrapper can reflink files inside the overlay). To get subvolume speed, move the checkout to a btrfs filesystem first.",
        )),
        Err(error) => found.push(refuse(format!("the filesystem of {} cannot be identified: {error}", root.display()), "Check that the path exists and that this user can read it.")),
    }
    // The need is the metadata of btrfs, so the tree is counted and the space compared only where the checkout is on btrfs.
    let on_btrfs: bool = matches!(
        platform.filesystem_facts(root),
        Ok((FilesystemKind::Btrfs, _))
    );
    if on_btrfs && let Ok((_, free)) = platform.filesystem_facts(layout.parent.as_path()) {
        // Counting reads directory listings only; it is the one walk of the tree this check makes.
        let entries: u64 = count_entries(root);
        let needed: u64 = required_free_bytes(entries);
        if free < needed {
            found.push(refuse(
                format!(
                    "{} has {free} bytes free, fewer than the {needed} bytes the migration requires for the metadata of the {entries} entries of {} ({METADATA_COPIES} copies of {METADATA_BYTES_PER_ENTRY} bytes each)",
                    layout.parent.display(),
                    root.display(),
                ),
                "Free space on that filesystem (the copy shares data blocks but writes one metadata record per entry), then run the command again.",
            ));
        }
    }
    if let Ok(parent_metadata) = std::fs::metadata(layout.parent.as_path())
        && parent_metadata.dev() != metadata.dev()
        && !platform.is_subvolume_root(root).unwrap_or(false)
    {
        found.push(refuse(
            format!(
                "{} and its parent {} are on different devices, so the two cannot be swapped",
                root.display(),
                layout.parent.display()
            ),
            "Run the command for a checkout that is not itself a mount point.",
        ));
    }
    if platform.is_mount_point(root) {
        found.push(refuse(
            format!("{} is a mount point, and a mount point cannot be swapped with another directory", root.display()),
            "Convert the directory that is mounted there instead, or mount a subvolume at that path yourself (`btrfs subvolume create` and `mount --options subvol=...`).",
        ));
    }
    found.extend(state_refusals(platform, layout, journal));
    found.extend(git_state_refusals(layout, journal));
    return found;
}

/// The refusals about what an earlier run,
/// or someone else,
/// left beside the checkout.
fn state_refusals(
    platform: &dyn Platform,
    layout: &Layout,
    journal: &Journal,
) -> Vec<Refusal> {
    let mut found: Vec<Refusal> = Vec::new();
    let is_root_subvolume: bool = platform
        .is_subvolume_root(layout.root.as_path())
        .unwrap_or(false);
    if is_root_subvolume && !journal.has_begun("exchange") {
        found.push(refuse(
            format!(
                "{} is already a subvolume (inode 256) and no migration of it is recorded",
                layout.root.display()
            ),
            "Nothing to do.",
        ));
    }
    if journal.has_begun("rollback-swap") {
        found.push(refuse(
            format!("a rollback of {} swapped the directories and did not finish (it is recorded in {})", layout.root.display(), layout.journal_path.display()),
            "Run `git monochromatic-git-policy-cli migrate-to-subvolume --rollback` again to finish it; migrating again would work on the wrong directories.",
        ));
    }
    if layout.new_path.exists() && !journal.has_begun("create") {
        found.push(refuse(
            format!("{} exists and no migration journal ({}) says that this command created it", layout.new_path.display(), layout.journal_path.display()),
            "Look at what it is. If it is the leftover of a migration whose journal was deleted, remove it (`btrfs subvolume delete <path>`, or empty it and `rmdir` it) and run the command again. If it is something else, rename it.",
        ));
    }
    // The stage is what the copy fills and the deleting delta acts on; before the exchange it is the directory at the
    // stage's name, and it must be the subvolume this migration recorded making.
    if journal.has_begun("create")
        && !journal.has_begun("exchange")
        && let Some(refusal) = stage_identity_refusal(platform, layout, journal)
    {
        found.push(refusal);
    }
    if layout.old_path.exists() && !journal.has_begun("exchange") {
        found.push(refuse(
            format!("{} exists already, and it is where the original tree would be moved", layout.old_path.display()),
            "If it is the old tree of an earlier migration you have checked, delete it (`git monochromatic-git-policy-cli migrate-to-subvolume --remove-old` needs the journal; otherwise `rm --recursive` it yourself). Otherwise rename it.",
        ));
    }
    return found;
}

/// Whether the directory at `path` holds no entry;
/// a listing that fails answers `false`,
/// so nothing is adopted that cannot be shown empty.
fn holds_no_entry(path: &Path) -> bool {
    let Ok(mut entries) = std::fs::read_dir(path) else {
        return false;
    };
    return entries.next().is_none();
}

/// What:
///  The refusal for the directory at the stage's name when it is not the subvolume the journal recorded creating.
/// Why:
///   A resume skips the creation step when the journal records it as done,
/// and without this check the copy runs
///  into a subvolume another process made at that name and the deleting delta removes the files it holds.
/// The identity the
///  journal records before anything is copied in is what tells the two apart,
/// and it survives a rename and the exchange.
/// A journal that records the creation but not the identity was written by an earlier release or by a run killed between
///  the two records;
/// an empty subvolume at the name is adopted then,
/// because a copy into an empty subvolume loses
///  nothing,
/// and anything else is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = stageIdentityRefusal(platform, layout, journal);
/// ```
fn stage_identity_refusal(
    platform: &dyn Platform,
    layout: &Layout,
    journal: &Journal,
) -> Option<Refusal> {
    let new_path: &Path = layout.new_path.as_path();
    let found: Option<crate::fs_beneath::ObjectId> =
        crate::worktree_convert_phases::identity_of_path(new_path);
    return match (journal.stage_identity(), found) {
        (Some(recorded), Some(named))
            if crate::worktree_convert_journal::is_recorded_object(&recorded, &named) =>
        {
            None
        }
        (Some(_), Some(_)) => Some(refuse(
            format!(
                "{} is not the subvolume this migration created (the journal records another identity), so nothing is copied into it or deleted from it",
                new_path.display()
            ),
            "If the subvolume is yours, move it away and run the command again; if it is the migration's own stage, move it back to that name.",
        )),
        (Some(_), None) => Some(refuse(
            format!(
                "{} was created by this migration and is gone, so the copy has nowhere to go",
                new_path.display()
            ),
            "If it was moved, move it back to that path. Otherwise run `git monochromatic-git-policy-cli migrate-to-subvolume --rollback` to clear the journal, then run the migration again.",
        )),
        (None, None) => None,
        (None, Some(_))
            if platform.is_subvolume_root(new_path).unwrap_or(false)
                && holds_no_entry(new_path) =>
        {
            None
        }
        (None, Some(_)) => Some(refuse(
            format!(
                "{} holds something the journal records no identity for, so it cannot be shown to be the subvolume this migration made; nothing is copied into it or deleted from it",
                new_path.display()
            ),
            "If it is yours, move it away and run the command again; if it is the migration's own stage, run `git monochromatic-git-policy-cli migrate-to-subvolume --rollback` to clear the journal, then run the migration again.",
        )),
    };
}
