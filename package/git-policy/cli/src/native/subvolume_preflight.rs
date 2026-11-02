//! What:
//!  The paths of one migration and the refusals that stop it before anything is created: every refusal names its cause
//!  and every remedy that applies.
//! Why:
//!  Converting a checkout swaps its directory under every process and worktree, so the migration starts only when the
//!  filesystem, the checkout and the state left by an earlier run allow it. The conditions are those of the proven
//!  script (`doc/handover/cli-git-subvolume-migration-script.md`, "Refusals") with the rest of the owner's rules:
//!  nothing is spawned, and a refusal is text a person can act on. A condition checks only metadata (the dry run reads
//!  `.git` and `.git/worktrees/*/gitdir` and never walks the tree).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusals: { cause: string; remedy: string }[] = preflight({ platform, layout, journal });
//! ```

use super::fs_kind::FilesystemKind;
use super::subvolume_journal::Journal;
use super::subvolume_platform::Platform;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The free space the parent filesystem must have: the copy shares data blocks but still writes one metadata record
///  per entry (the script's bound, 5 GiB; about 1 KiB per entry is an assumption, not a measurement).
pub const MIN_FREE_BYTES: u64 = 5 * 1024 * 1024 * 1024;

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
///  Every path of one migration, derived from the checkout's path.
/// Why:
///   The new tree and the old tree sit beside the checkout (one parent, so the exchange stays on one filesystem), and
///  the journal and the unreadable-file list sit beside them, outside the tree that is exchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Layout = { root: string; parent: string; newPath: string; oldPath: string; journalPath: string; unreadablePath: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Layout {
    /// The main checkout.
    pub root: PathBuf,
    /// Its parent directory.
    pub parent: PathBuf,
    /// `<root>.subvol-new`: the subvolume being filled, and after the exchange the old tree until it is renamed.
    pub new_path: PathBuf,
    /// `<root>.before-subvolume`: the original tree, kept until removed on request.
    pub old_path: PathBuf,
    /// `<parent>/.cli-git-migrate-<name>.journal.json`.
    pub journal_path: PathBuf,
    /// `<parent>/.cli-git-migrate-<name>.unreadable.txt`.
    pub unreadable_path: PathBuf,
}

/// What:
///  The layout for a checkout path.
///  A root with no parent or no name (`/`) has no layout.
/// Why:
///   The names are fixed by the root's name, so a rerun finds the same files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function layoutOf(root: string): Layout | undefined;
/// ```
pub fn layout_of(root: &Path) -> Option<Layout> {
    let name: &std::ffi::OsStr = root.file_name()?;
    let parent: &Path = root.parent()?;
    let with_suffix = |suffix: &str| -> PathBuf {
        let mut text: std::ffi::OsString = name.to_os_string();
        text.push(suffix);
        return parent.join(text);
    };
    let dotted = |suffix: &str| -> PathBuf {
        let mut text: std::ffi::OsString = std::ffi::OsString::from(".cli-git-migrate-");
        text.push(name);
        text.push(suffix);
        return parent.join(text);
    };
    return Some(Layout {
        root: root.to_path_buf(),
        parent: parent.to_path_buf(),
        new_path: with_suffix(".subvol-new"),
        old_path: with_suffix(".before-subvolume"),
        journal_path: dotted(".journal.json"),
        unreadable_path: dotted(".unreadable.txt"),
    });
}

/// What:
///  One refusal.
/// Why:
///   The cause says what is wrong with which path; the remedy lists every way forward, so the person never has to guess.
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
///  The refusals of a fresh or resumed migration; empty means it may start or continue.
/// Why:
///   See the module description. The journal says which leftovers are this migration's own: the new tree after its
///  `create` step began, the old tree after the `exchange` step began, `.git/index.lock` after the `lock` step began.
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
    if let Ok((_, free)) = platform.filesystem_facts(layout.parent.as_path()) && free < MIN_FREE_BYTES {
        found.push(refuse(
            format!("{} has {free} bytes free, fewer than the {MIN_FREE_BYTES} bytes the migration requires for its metadata", layout.parent.display()),
            "Free space on that filesystem (the copy shares data blocks but writes one metadata record per entry), then run the command again.",
        ));
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

/// The refusals about what an earlier run, or someone else, left beside the checkout.
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
            "Run `git cli-git migrate-to-subvolume --rollback` again to finish it; migrating again would work on the wrong directories.",
        ));
    }
    if layout.new_path.exists() && !journal.has_begun("create") {
        found.push(refuse(
            format!("{} exists and no migration journal ({}) says that this command created it", layout.new_path.display(), layout.journal_path.display()),
            "Look at what it is. If it is the leftover of a migration whose journal was deleted, remove it (`btrfs subvolume delete <path>`, or empty it and `rmdir` it) and run the command again. If it is something else, rename it.",
        ));
    }
    if layout.old_path.exists() && !journal.has_begun("exchange") {
        found.push(refuse(
            format!("{} exists already, and it is where the original tree would be moved", layout.old_path.display()),
            "If it is the old tree of an earlier migration you have checked, delete it (`git cli-git migrate-to-subvolume --remove-old` needs the journal; otherwise `rm --recursive` it yourself). Otherwise rename it.",
        ));
    }
    return found;
}

/// The refusals about Git's own state in the checkout.
fn git_state_refusals(layout: &Layout, journal: &Journal) -> Vec<Refusal> {
    let mut found: Vec<Refusal> = Vec::new();
    let git_directory: PathBuf = layout.root.join(".git");
    if !std::fs::metadata(&git_directory).is_ok_and(|read: std::fs::Metadata| { return read.is_dir(); }) {
        found.push(refuse(
            format!("{} is not a directory, so this is not the main checkout of a repository", git_directory.display()),
            "Run the command from the main checkout (the directory that holds the repository's `.git` directory), not from a linked worktree.",
        ));
        return found;
    }
    let mut directories: Vec<PathBuf> = vec![git_directory.clone()];
    if let Ok(names) = std::fs::read_dir(git_directory.join("worktrees")) {
        for name in names.flatten() {
            directories.push(name.path());
        }
    }
    directories.sort();
    for directory in &directories {
        for name in IN_PROGRESS_NAMES {
            let own_lock: bool = *directory == git_directory && name == "index.lock" && journal.has_begun("lock");
            if directory.join(name).exists() && !own_lock {
                found.push(refuse(
                    format!("Git is in the middle of an operation: {} exists", directory.join(name).display()),
                    "Finish or abort that operation (commit, merge, rebase, cherry-pick, bisect), or, if no Git command is running, remove the stale file after checking, then run the command again.",
                ));
            }
        }
        for queue in ["pending", "claimed"] {
            let path: PathBuf = directory.join("cli-git").join("work").join(queue);
            if std::fs::read_dir(&path).is_ok_and(|mut entries: std::fs::ReadDir| { return entries.next().is_some(); }) {
                found.push(refuse(
                    format!("the commit service has unfinished work in {}", path.display()),
                    "Wait for the service to finish landing its commits (`git cli-git service status`), then run the command again.",
                ));
            }
        }
    }
    return found;
}

/// What:
///  The refusals of a rollback: it needs a journal.
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
///  The refusals of deleting the old tree: the migration must have finished and the old tree must exist.
/// Why:
///   The old tree is the only copy of the files this user could not read, and of anything not yet carried over; it is
///  deleted only when the person asks, after a verified migration.
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
            "Run `git cli-git migrate-to-subvolume` to finish (or `--rollback` to undo) before deleting the old tree.",
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
