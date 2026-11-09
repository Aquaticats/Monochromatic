//! What:
//!  The paths of one migration and the refusals that stop it before anything is created:
//! every refusal names its cause
//!  and every remedy that applies.
//! Why:
//!  Converting a checkout swaps its directory under every process and worktree,
//! so the migration starts only when the
//!  filesystem,
//! the checkout and the state left by an earlier run allow it.
//! The conditions are those of the proven
//!  script (`doc/handover/cli-git-subvolume-migration-script.md`,
//! "Refusals") with the rest of the owner's rules:
//!  nothing is spawned,
//! and a refusal is text a person can act on.
//! A condition checks only metadata (the dry run reads
//!  `.git` and `.git/worktrees/*/gitdir`,
//! and lists every directory of the tree once to count its entries for the free-space
//!  check,
//! without opening a file).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusals: { cause: string; remedy: string }[] = preflight({ platform, layout, journal });
//! ```

use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::fs_kind::FilesystemKind;
use crate::single_owner_claim::live_owner_of;
use crate::subvolume_entry_count::count_entries;
use crate::subvolume_journal::Journal;
use crate::subvolume_lock::{is_ours, token_of};
use crate::subvolume_platform::Platform;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The metadata the copy writes for one entry,
/// in bytes of the filesystem's own accounting:
/// the `Metadata, DUP` figure of
///  `btrfs filesystem df --block-size=1` read before and after reflink-copying a tree of 22,719 entries eight times
///  (181,752 entries) on this host,
/// three alternating rounds against a no-copy control of the same wall time:
///  957.9,
/// 979.0 and 984.8 bytes per entry,
/// while the control moved by at most 9.9 MB against 174 MB to 179 MB for a copy.
///  Rounded up to one kibibyte.
/// Recorded in `doc/planning/cli-git-shipped-limits-audit.md`,
/// "The free-space preflight".
/// This is
///  the coordinator's pick under the owner's delegation ("Pick options as if I had picked them"),
/// open to the owner's veto.
pub const METADATA_BYTES_PER_ENTRY: u64 = 1024;

/// How many physical copies of the metadata the filesystem keeps:
/// the `DUP` profile of this host writes each metadata byte twice
///  (`Metadata ratio: 2.00` in `btrfs filesystem usage`),
/// and the free space the check reads is physical.
/// A filesystem with a
///  single copy is over-estimated by this factor,
/// which only refuses earlier.
pub const METADATA_COPIES: u64 = 2;

/// What:
///  The free bytes the migration needs for the metadata of `entries` entries.
/// Why:
///   The copy shares data blocks and writes one metadata record per entry,
/// so the need grows with the entries,
/// not with a fixed
///  floor;
/// the measured cost of an entry and the copies of the metadata are the two named constants above.
/// A count so large
///  that the product overflows saturates at the largest `u64`,
/// which no filesystem has free,
/// so it refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const needed = Math.min(Number.MAX_SAFE_INTEGER, entries * METADATA_BYTES_PER_ENTRY * METADATA_COPIES);
/// ```
pub fn required_free_bytes(entries: u64) -> u64 {
    // What: `saturating_mul` multiplies two `u64` values and gives the largest `u64` instead of wrapping around when the product is too big.
    // Why: An absurd entry count must refuse, never wrap to a small number that passes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return Math.min(U64_MAX, entries * (METADATA_BYTES_PER_ENTRY * METADATA_COPIES));
    // ```
    return entries.saturating_mul(METADATA_BYTES_PER_ENTRY.saturating_mul(METADATA_COPIES));
}

/// The names whose presence in `.git` or in a linked worktree's administrative directory means Git is mid-operation.
pub const IN_PROGRESS_NAMES: [&str; 12] = [
    "index.lock",
    "MERGE_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "BISECT_LOG",
    "rebase-merge",
    "rebase-apply",
    "sequencer",
    "HEAD.lock",
    "config.lock",
    "packed-refs.lock",
    "shallow.lock",
];

/// What:
///  Every path of one migration,
/// derived from the checkout's path.
/// Why:
///   The new tree and the old tree sit beside the checkout (one parent,
/// so the exchange stays on one filesystem),
/// and
///  the journal and the unreadable-file list sit beside them,
/// outside the tree that is exchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Layout = { root: string; parent: string; newPath: string; oldPath: string; journalPath: string; unreadablePath: string; claimPrefix: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Layout {
    /// The main checkout.
    pub root: PathBuf,
    /// Its parent directory.
    pub parent: PathBuf,
    /// `<root>.subvol-new`:
    /// the subvolume being filled,
    /// and after the exchange the old tree until it is renamed.
    pub new_path: PathBuf,
    /// `<root>.before-subvolume`:
    /// the original tree,
    /// kept until removed on request.
    pub old_path: PathBuf,
    /// `<parent>/.cli-git-migrate-<name>.journal.json`.
    pub journal_path: PathBuf,
    /// `<parent>/.cli-git-migrate-<name>.unreadable.txt`.
    pub unreadable_path: PathBuf,
    /// `.cli-git-migrate-<name>.owner`:
    /// the name,
    /// in the parent directory,
    /// of the numbered claim files that give one command
    ///  at a time the migration of this checkout.
    pub claim_prefix: std::ffi::OsString,
}

/// `parent/<name><suffix>`:
/// the name of a file kept beside the root under a fixed suffix.
fn with_suffix(parent: &Path, name: &std::ffi::OsStr, suffix: &str) -> PathBuf {
    let mut text: std::ffi::OsString = name.to_os_string();
    text.push(suffix);
    return parent.join(text);
}

/// `parent/.cli-git-migrate-<name><suffix>`:
/// the name of a hidden file kept beside the root under a fixed suffix.
fn dotted(parent: &Path, name: &std::ffi::OsStr, suffix: &str) -> PathBuf {
    let mut text: std::ffi::OsString = std::ffi::OsString::from(".cli-git-migrate-");
    text.push(name);
    text.push(suffix);
    return parent.join(text);
}

/// What:
///  The layout for a checkout path.
///  A root with no parent or no name (`/`) has no layout.
/// Why:
///   The names are fixed by the root's name,
/// so a rerun finds the same files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function layoutOf(root: string): Layout | undefined;
/// ```
pub fn layout_of(root: &Path) -> Option<Layout> {
    let name: &std::ffi::OsStr = root.file_name()?;
    let parent: &Path = root.parent()?;

    return Some(Layout {
        root: root.to_path_buf(),
        parent: parent.to_path_buf(),
        new_path: with_suffix(parent, name, ".subvol-new"),
        old_path: with_suffix(parent, name, ".before-subvolume"),
        journal_path: dotted(parent, name, ".journal.json"),
        unreadable_path: dotted(parent, name, ".unreadable.txt"),
        claim_prefix: dotted(parent, name, ".owner").file_name().unwrap_or_default().to_os_string(),
    });
}

/// What:
///  One refusal.
/// Why:
///   The cause says what is wrong with which path;
/// the remedy lists every way forward,
/// so the person never has to guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { cause: string; remedy: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal {
    /// What is wrong.
    pub cause: String,
    /// What can be done about it.
    pub remedy: String,
}

/// Shorthand for building a refusal from two texts.
fn refuse(cause: String, remedy: &str) -> Refusal {
    return Refusal { cause, remedy: remedy.to_string() };
}

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
pub fn migrate_refusals(platform: &dyn Platform, layout: &Layout, journal: &Journal) -> Vec<Refusal> {
    let mut found: Vec<Refusal> = Vec::new();
    let root: &Path = layout.root.as_path();
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(root) {
        Ok(read) => read,
        Err(error) => {
            found.push(refuse(format!("{} cannot be examined: {error}", root.display()), "Run the command inside the repository's main checkout, or check the path."));
            return found;
        }
    };
    if !metadata.is_dir() {
        found.push(refuse(format!("{} is not a directory (it is a symbolic link or a file)", root.display()), "Run the command from the real path of the main checkout."));
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
    let on_btrfs: bool = matches!(platform.filesystem_facts(root), Ok((FilesystemKind::Btrfs, _)));
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
    if let Ok(parent_metadata) = std::fs::metadata(layout.parent.as_path()) && parent_metadata.dev() != metadata.dev() && !platform.is_subvolume_root(root).unwrap_or(false) {
        found.push(refuse(
            format!("{} and its parent {} are on different devices, so the two cannot be swapped", root.display(), layout.parent.display()),
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
fn state_refusals(platform: &dyn Platform, layout: &Layout, journal: &Journal) -> Vec<Refusal> {
    let mut found: Vec<Refusal> = Vec::new();
    let is_root_subvolume: bool = platform.is_subvolume_root(layout.root.as_path()).unwrap_or(false);
    if is_root_subvolume && !journal.has_begun("exchange") {
        found.push(refuse(
            format!("{} is already a subvolume (inode 256) and no migration of it is recorded", layout.root.display()),
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
    let Ok(mut entries) = std::fs::read_dir(path) else { return false };
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
fn stage_identity_refusal(platform: &dyn Platform, layout: &Layout, journal: &Journal) -> Option<Refusal> {
    let new_path: &Path = layout.new_path.as_path();
    let found: Option<crate::fs_beneath::ObjectId> = crate::worktree_convert_phases::identity_of_path(new_path);
    return match (journal.stage_identity(), found) {
        (Some(recorded), Some(named)) if crate::worktree_convert_journal::is_recorded_object(&recorded, &named) => None,
        (Some(_), Some(_)) => Some(refuse(
            format!("{} is not the subvolume this migration created (the journal records another identity), so nothing is copied into it or deleted from it", new_path.display()),
            "If the subvolume is yours, move it away and run the command again; if it is the migration's own stage, move it back to that name.",
        )),
        (Some(_), None) => Some(refuse(
            format!("{} was created by this migration and is gone, so the copy has nowhere to go", new_path.display()),
            "If it was moved, move it back to that path. Otherwise run `git monochromatic-git-policy-cli migrate-to-subvolume --rollback` to clear the journal, then run the migration again.",
        )),
        (None, None) => None,
        (None, Some(_)) if platform.is_subvolume_root(new_path).unwrap_or(false) && holds_no_entry(new_path) => None,
        (None, Some(_)) => Some(refuse(
            format!("{} holds something the journal records no identity for, so it cannot be shown to be the subvolume this migration made; nothing is copied into it or deleted from it", new_path.display()),
            "If it is yours, move it away and run the command again; if it is the migration's own stage, run `git monochromatic-git-policy-cli migrate-to-subvolume --rollback` to clear the journal, then run the migration again.",
        )),
    };
}

/// The refusal for a conversion of the linked worktree whose administrative directory this is,
/// when its command is running.
fn conversion_refusal(administrative: &Path) -> Option<Refusal> {
    return match live_owner_of(administrative, std::ffi::OsStr::new(crate::worktree_convert_journal::CLAIM_PREFIX)) {
        Ok(None) => None,
        Ok(Some(owner)) => Some(refuse(
            format!("process {} is converting the linked worktree registered in {} to a subvolume (it owns {})", owner.pid, administrative.display(), owner.path.display()),
            "Wait for `git monochromatic-git-policy-cli convert-worktree-to-subvolume` to finish, then run the command again.",
        )),
        Err(error) => Some(refuse(
            format!("whether the linked worktree registered in {} is being converted cannot be decided ({error})", administrative.display()),
            "Look at the claim file the message names; if no monochromatic-git-policy-cli command is running, remove it, then run the command again.",
        )),
    };
}

/// The refusals about Git's own state in the checkout.
fn git_state_refusals(layout: &Layout, journal: &Journal) -> Vec<Refusal> {
    let mut found: Vec<Refusal> = Vec::new();
    let git_directory: PathBuf = layout.root.join(".git");
    if !git_directory.is_dir() {
        found.push(refuse(
            format!("{} is not a directory, so this is not the main checkout of a repository", git_directory.display()),
            "Run the command from the main checkout (the directory that holds the repository's `.git` directory), not from a linked worktree.",
        ));
        return found;
    }
    let token: String = token_of(journal);
    let mut directories: Vec<PathBuf> = vec![git_directory.clone()];
    if let Ok(names) = std::fs::read_dir(git_directory.join("worktrees")) {
        for name in names.flatten() {
            directories.push(name.path());
        }
    }
    directories.sort();
    for directory in &directories {
        for name in IN_PROGRESS_NAMES {
            let own_lock: bool = name == "index.lock" && is_ours(directory.join(name).as_path(), token.as_str());
            if directory.join(name).exists() && !own_lock {
                found.push(refuse(
                    format!("Git is in the middle of an operation: {} exists", directory.join(name).display()),
                    "Finish or abort that operation (commit, merge, rebase, cherry-pick, bisect), or, if no Git command is running, remove the stale file after checking, then run the command again.",
                ));
            }
        }
        if let Some(refusal) = conversion_refusal(directory.as_path()) {
            found.push(refusal);
        }
        for queue in ["pending", "claimed"] {
            let path: PathBuf = path_for_reading(directory.as_path(), "monochromatic-git-policy-cli/work").join(queue);
            if let Ok(mut entries) = std::fs::read_dir(&path)
                && entries.next().is_some()
            {
                found.push(refuse(
                    format!("the commit service has unfinished work in {}", path.display()),
                    "Wait for the service to finish landing its commits (`git monochromatic-git-policy-cli service status`), then run the command again.",
                ));
            }
        }
    }
    return found;
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
pub fn rollback_refusals(layout: &Layout, journal: &Journal) -> Vec<Refusal> {
    if journal.is_persisted() {
        return Vec::new();
    }
    return vec![refuse(
        format!("there is no migration journal at {}, so there is nothing recorded to undo", layout.journal_path.display()),
        "If a migration was started by hand, undo it by hand: exchange the directories again with `mv --exchange` and delete the new subvolume.",
    )];
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
pub fn remove_old_refusals(layout: &Layout, journal: &Journal) -> Vec<Refusal> {
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
    return found;
}

/// Preflight controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_preflight_tests.rs"]
mod tests;
