//! What:
//!  Whether the creating worktree can be the source of a snapshot for a new worktree, decided before real Git runs, and
//!  everything the snapshot needs to know about it.
//! Why:
//!  A snapshot copies the whole subvolume, so the source must be a subvolume root on the same filesystem as the
//!  destination, clean, and free of every feature whose result would differ from Git's checkout: a sparse checkout, a
//!  worktree-specific configuration, a submodule, and a nested linked worktree that is an ordinary directory (a
//!  snapshot carries its files, and removing them one by one costs more than the checkout saves). Each refusal returns
//!  its reason in words; the caller logs it and falls to the next rung without a word to the person.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const source = await chooseSnapshotSource({ context, identity, plan }); // throws the reason it cannot be one
//! ```

use super::config_snapshot::{ConfigEntry, ConfigSnapshot, read_config_snapshot};
use super::fs_kind::{FilesystemKind, filesystem_kind, is_subvolume_root};
use super::fs_mount_options::read_mount_facts;
use super::transaction_git::GitContext;
use super::worktree_add_plan::AddPlan;
use super::worktree_clean::{CleanSource, check_clean};
use super::worktree_identity::WorktreeIdentity;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// What:
///  The source a snapshot is taken from and what the later steps need to know about it.
/// Why:
///   The checks ran once before Git; the steps after Git use their results instead of asking again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SnapshotSource = { root: string; gitDir: string; commonDir: string; clean: CleanSource; nested: string[]; config: ConfigSnapshot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotSource {
    /// The subvolume root of the creating worktree.
    pub root: PathBuf,
    /// Its Git directory: its own for a linked worktree, the common one for a main checkout.
    pub git_dir: PathBuf,
    /// The common Git directory.
    pub common_dir: PathBuf,
    /// What its `HEAD` names, proven clean.
    pub clean: CleanSource,
    /// Registered linked worktrees whose roots lie inside the source; the snapshot holds an empty stub for each.
    pub nested: Vec<PathBuf>,
    /// The configuration the checks read, which names the abbreviation length of the line Git's checkout prints.
    pub config: ConfigSnapshot,
    /// Whether Git ignores the place a snapshot of the main checkout sets its left-out directories aside; false for a linked
    ///  worktree, which sets nothing aside.
    pub trash_ignored: bool,
}

/// What:
///  The roots of the linked worktrees registered in a repository, read from their `gitdir` files.
/// Why:
///   Git lists the same paths; reading them here costs no Git process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function registeredRoots(commonDir: string): Promise<string[]>;
/// ```
pub fn registered_roots(common_dir: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(common_dir.join("worktrees")) else { return Vec::new() };
    let mut roots: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let Ok(recorded) = std::fs::read(entry.path().join("gitdir")) else { continue };
        let line: &[u8] = recorded.strip_suffix(b"\n").unwrap_or(recorded.as_slice());
        let pointer: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(line));
        if let Some(root) = pointer.parent() {
            roots.push(root.to_path_buf());
        }
    }
    return roots;
}

/// The configuration key that turns a sparse checkout on; Git copies the patterns into a new worktree, a snapshot cannot.
const SPARSE_KEY: &str = "core.sparsecheckout";

/// What:
///  The reason the configuration rules the source out, or nothing.
/// Why:
///   Git copies the filtered worktree configuration and the sparse-checkout patterns into a new worktree, and conversion
///  settings that live only in a worktree would disagree between the two worktrees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configurationRefusal(entries: ConfigEntry[]): string | undefined;
/// ```
fn configuration_refusal(entries: &[ConfigEntry]) -> Option<String> {
    for entry in entries {
        if entry.key == SPARSE_KEY {
            return Some(String::from("core.sparseCheckout is set"));
        }
        if entry.scope == "worktree" {
            return Some(format!("the worktree-specific configuration sets {}", entry.key));
        }
    }
    return None;
}

/// What:
///  Whether the mounts that hold two paths are one btrfs filesystem.
/// Why:
///   A snapshot and a clone cross subvolumes of one filesystem only; the mount table names the device, which is the
///  same for every subvolume of a filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onOneBtrfs(first: string, second: string): boolean;
/// ```
fn on_one_btrfs(first: &Path, second: &Path) -> bool {
    if filesystem_kind(first) != Ok(FilesystemKind::Btrfs) || filesystem_kind(second) != Ok(FilesystemKind::Btrfs) {
        return false;
    }
    let (Ok(Some(one)), Ok(Some(other))) = (read_mount_facts(first), read_mount_facts(second)) else { return false };
    return one.source == other.source;
}

/// What:
///  The registered worktrees inside the source that are not empty stubs, if any.
/// Why:
///   A nested worktree that is a subvolume appears in a snapshot as an empty directory; an ordinary directory would be
///  carried with all its files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedOrdinaryWorktree(nested: string[]): string | undefined;
/// ```
fn nested_ordinary_worktree(nested: &[PathBuf]) -> Option<PathBuf> {
    return nested
        .iter()
        .find(|path: &&PathBuf| { return path.exists() && !is_subvolume_root(path.as_path()).unwrap_or(false); })
        .cloned();
}

/// What:
///  The registered worktrees inside the source whose recorded path is not their real path, if any.
/// Why:
///   A recorded path reached through a link names a directory the snapshot carries as a link, not as a stub; the
///  snapshot rung leaves such a source to the next rung.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedThroughLink(nested: string[]): string | undefined;
/// ```
fn nested_through_link(nested: &[PathBuf]) -> Option<PathBuf> {
    return nested
        .iter()
        .find(|path: &&PathBuf| { return path.exists() && std::fs::canonicalize(path.as_path()).ok().as_ref() != Some(*path); })
        .cloned();
}

/// What:
///  The root, Git directory and common directory of the creating worktree, or why there is none.
/// Why:
///   A bare repository and a location outside any worktree have no files to copy from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeOf(identity: WorktreeIdentity): { root: string; gitDir: string; commonDir: string }; // throws a reason
/// ```
fn worktree_of(identity: &WorktreeIdentity) -> Result<(&PathBuf, &PathBuf, &PathBuf), String> {
    return match identity {
        WorktreeIdentity::MainWorktree { worktree_root, git_dir, common_dir } | WorktreeIdentity::LinkedWorktree { worktree_root, git_dir, common_dir } => Ok((worktree_root, git_dir, common_dir)),
        WorktreeIdentity::BareRepository { .. } | WorktreeIdentity::OutsideWorktree => Err(String::from("there is no worktree to copy from")),
    };
}

/// What:
///  The cheap reasons the snapshot rung cannot serve a planned add: the source is no subvolume root, or it and the
///  destination are not on one btrfs filesystem.
/// Why:
///   These need two `statfs` calls and one inode number, so they run before the dear checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function snapshotRefusal(identity: WorktreeIdentity, plan: AddPlan): string | undefined;
/// ```
pub fn snapshot_refusal(identity: &WorktreeIdentity, plan: &AddPlan) -> Option<String> {
    let (root, _git_dir, _common_dir): (&PathBuf, &PathBuf, &PathBuf) = match worktree_of(identity) {
        Ok(found) => found,
        Err(reason) => return Some(reason),
    };
    let Some(parent) = plan.destination.parent() else { return Some(String::from("the destination has no parent directory")) };
    if !is_subvolume_root(root.as_path()).unwrap_or(false) {
        return Some(format!("{} is not a btrfs subvolume", root.display()));
    }
    if !on_one_btrfs(root.as_path(), parent) {
        return Some(format!("{} and {} are not on one btrfs filesystem", root.display(), parent.display()));
    }
    return None;
}

/// What:
///  Whether the filesystem of `path` may share data blocks between files: btrfs, XFS (reflink is a per-filesystem
///  feature that the first clone reveals) and overlayfs (a clone inside the overlay reaches the layers' filesystem).
/// Why:
///   Other filesystems never clone, so the clone rung is not tried there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mayClone(path: string): boolean;
/// ```
fn may_clone(path: &Path) -> bool {
    if matches!(filesystem_kind(path), Ok(FilesystemKind::Btrfs | FilesystemKind::Overlayfs)) {
        return true;
    }
    return super::fs_kind::is_xfs(path).unwrap_or(false);
}

/// What:
///  The cheap reasons the clone rung cannot serve a planned add: either side is on a filesystem that never clones, or the
///  two are not in one clone domain (one btrfs filesystem, or one device elsewhere).
/// Why:
///   A clone crosses subvolumes of one btrfs filesystem and nothing else; an overlay or XFS is one device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneRefusal(identity: WorktreeIdentity, plan: AddPlan): string | undefined;
/// ```
pub fn clone_refusal(identity: &WorktreeIdentity, plan: &AddPlan) -> Option<String> {
    let (root, _git_dir, _common_dir): (&PathBuf, &PathBuf, &PathBuf) = match worktree_of(identity) {
        Ok(found) => found,
        Err(reason) => return Some(reason),
    };
    let Some(parent) = plan.destination.parent() else { return Some(String::from("the destination has no parent directory")) };
    if !may_clone(root.as_path()) || !may_clone(parent) {
        return Some(String::from("the filesystem does not share data blocks between files"));
    }
    let one_domain: bool = if filesystem_kind(root.as_path()) == Ok(FilesystemKind::Btrfs) {
        on_one_btrfs(root.as_path(), parent)
    } else {
        std::fs::metadata(root.as_path()).map(|found: std::fs::Metadata| { return found.dev(); }).ok() == std::fs::metadata(parent).map(|found: std::fs::Metadata| { return found.dev(); }).ok()
    };
    return if one_domain { None } else { Some(format!("{} and {} are on different filesystems", root.display(), parent.display())) };
}

/// What:
///  Check the creating worktree for everything a copy of its files needs, from the cheapest check to the dearest, and
///  collect what the later steps use; the reason is the error.
/// Why:
///   The dear checks (the clean check starts one Git run) happen once for both rungs, and only after a cheap check let a
///  rung through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commonSource(context: GitContext, identity: WorktreeIdentity): Promise<SnapshotSource>; // throws a reason
/// ```
pub fn common_source(context: &GitContext, identity: &WorktreeIdentity) -> Result<SnapshotSource, String> {
    let (root, git_dir, common_dir): (&PathBuf, &PathBuf, &PathBuf) = worktree_of(identity)?;
    if git_dir.join("info/sparse-checkout").exists() || common_dir.join("info/sparse-checkout").exists() {
        return Err(String::from("the source has a sparse checkout"));
    }
    let nested: Vec<PathBuf> = registered_roots(common_dir.as_path()).into_iter().filter(|path: &PathBuf| { return path != root && path.starts_with(root); }).collect();
    let snapshot: ConfigSnapshot = read_config_snapshot(context, root.as_path()).ok_or_else(|| { return String::from("the Git configuration could not be read"); })?;
    if let Some(reason) = configuration_refusal(snapshot.entries()) {
        return Err(reason);
    }
    let clean: CleanSource = check_clean(context, root.as_path(), git_dir.as_path(), common_dir.as_path())?;
    let trash_ignored: bool = git_dir == common_dir && super::worktree_trash::trash_location_is_ignored(context, root.as_path());
    return Ok(SnapshotSource { root: root.clone(), git_dir: git_dir.clone(), common_dir: common_dir.clone(), clean, nested, config: snapshot, trash_ignored });
}

/// What:
///  The registered worktree inside the source that would be carried into a snapshot with all its files, if any.
/// Why:
///   Only the snapshot rung cares: a nested worktree that is a subvolume is an empty stub in a snapshot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function snapshotBlocker(source: SnapshotSource): string | undefined;
/// ```
pub fn snapshot_blocker(source: &SnapshotSource) -> Option<String> {
    if let Some(path) = nested_through_link(source.nested.as_slice()) {
        return Some(format!("the worktree {} inside the source is reached through a symbolic link", path.display()));
    }
    // A snapshot of the main checkout sets the nested ordinary worktrees aside (`worktree_trash.rs`), so they do not block it.
    if source.git_dir == source.common_dir {
        return None;
    }
    return nested_ordinary_worktree(source.nested.as_slice()).map(|path: PathBuf| { return format!("the worktree {} inside the source is an ordinary directory; `git cli-git migrate-to-subvolume` makes it a subvolume", path.display()); });
}

/// What:
///  Decide whether the creating worktree of a planned add can be the snapshot source: the cheap refusals, the shared
///  checks, then what only a snapshot minds.
/// Why:
///   The ladder asks the three in turn so the clone rung can share the dear checks; this is the whole question for the
///  snapshot alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function chooseSnapshotSource(args: { context: GitContext; identity: WorktreeIdentity; plan: AddPlan }): Promise<SnapshotSource>; // throws a reason
/// ```
pub fn choose_snapshot_source(context: &GitContext, identity: &WorktreeIdentity, plan: &AddPlan) -> Result<SnapshotSource, String> {
    if let Some(reason) = snapshot_refusal(identity, plan) {
        return Err(reason);
    }
    let source: SnapshotSource = common_source(context, identity)?;
    return match snapshot_blocker(&source) {
        Some(reason) => Err(reason),
        None => Ok(source),
    };
}

/// Source selection controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_snapshot_source_tests.rs"]
mod tests;
