//! What:
//!  Whether the creating worktree can be the source of a snapshot for a new worktree,
//! decided before real Git runs,
//! and
//!  everything the snapshot needs to know about it.
//! Why:
//!  A snapshot copies the whole subvolume,
//! so the source must be a subvolume root on the same filesystem as the
//!  destination,
//! clean,
//! and free of every feature whose result would differ from Git's checkout:
//! a sparse checkout,
//! a
//!  worktree-specific configuration,
//! a submodule,
//! and a nested linked worktree that is an ordinary directory (a
//!  snapshot carries its files,
//! and removing them one by one costs more than the checkout saves).
//! Each refusal returns
//!  its reason in words;
//! the caller logs it and falls to the next rung without a word to the person.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const source = await chooseSnapshotSource({ context, identity, plan }); // throws the reason it cannot be one
//! ```

use crate::add_timing::{begin_work, end_work};
use crate::diagnostic_log::debug;
pub use crate::worktree_snapshot_source_changed_paths::{ChangedPaths, changed_paths, plain};
use crate::config_snapshot::{ConfigEntry, ConfigSnapshot, read_config_snapshot, read_config_snapshot_awaiting};
use crate::fs_kind::{FilesystemKind, filesystem_kind, is_subvolume_root};
use crate::fs_mount_options::read_mount_facts;
use crate::lazy_runtime::run_on_runtime;
use crate::transaction_git::GitContext;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clean::{CleanRefusal, CleanRefusalKind, CleanSource, check_clean, check_clean_awaiting, dirty_source};
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_trash_ignore_rule::{trash_location_is_ignored, trash_location_is_ignored_awaiting};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeSnapshotSource";

/// What:
///  The source a snapshot is taken from and what the later steps need to know about it.
/// Why:
///   The checks ran once before Git;
/// the steps after Git use their results instead of asking again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SnapshotSource = { root: string; gitDir: string; commonDir: string; clean: CleanSource; nested: string[]; config: ConfigSnapshot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotSource {
    /// The subvolume root of the creating worktree.
    pub root: PathBuf,
    /// Its Git directory:
    /// its own for a linked worktree,
    /// the common one for a main checkout.
    pub git_dir: PathBuf,
    /// The common Git directory.
    pub common_dir: PathBuf,
    /// What its `HEAD` names,
    /// proven clean.
    pub clean: CleanSource,
    /// Registered linked worktrees whose roots lie inside the source;
    /// the snapshot holds an empty stub for each.
    pub nested: Vec<PathBuf>,
    /// The configuration the checks read,
    /// which names the abbreviation length of the line Git's checkout prints.
    pub config: ConfigSnapshot,
    /// Whether Git ignores the place a snapshot sets its left-out directories aside;
    /// false when it does not,
    ///  and then the leave-out removes in the foreground.
    pub trash_ignored: bool,
    /// The uncommitted changes of a source that is not clean,
    /// when it is snapshotted anyway and restored to its `HEAD`;
    ///  nothing for a clean source.
    pub dirty: Option<ChangedPaths>,
}

/// What:
///  The roots of the linked worktrees registered in a repository,
/// read from their `gitdir` files.
/// Why:
///   Git lists the same paths;
/// reading them here costs no Git process.
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

/// The configuration key that turns a sparse checkout on;
/// Git copies the patterns into a new worktree,
/// a snapshot cannot.
const SPARSE_KEY: &str = "core.sparsecheckout";

/// What:
///  The reason the configuration rules the source out,
/// or nothing.
/// Why:
///   Git copies the filtered worktree configuration and the sparse-checkout patterns into a new worktree,
/// and conversion
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
///   A snapshot and a clone cross subvolumes of one filesystem only;
/// the mount table names the device,
/// which is the
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
///  The registered worktrees inside the source that are not empty stubs,
/// if any.
/// Why:
///   A nested worktree that is a subvolume appears in a snapshot as an empty directory;
/// an ordinary directory would be
///  carried with all its files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedOrdinaryWorktree(nested: string[]): string | undefined;
/// ```
fn nested_ordinary_worktree(nested: &[PathBuf]) -> Option<PathBuf> {
    for path in nested {
        if path.exists() && !is_subvolume_root(path.as_path()).unwrap_or(false) {
            return Some(path.clone());
        }
    }
    return None;
}

/// What:
///  The registered worktrees inside the source whose recorded path is not their real path,
/// if any.
/// Why:
///   A recorded path reached through a link names a directory the snapshot carries as a link,
/// not as a stub;
/// the
///  snapshot rung leaves such a source to the next rung.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedThroughLink(nested: string[]): string | undefined;
/// ```
fn nested_through_link(nested: &[PathBuf]) -> Option<PathBuf> {
    for path in nested {
        if path.exists() && std::fs::canonicalize(path.as_path()).ok().as_ref() != Some(path) {
            return Some(path.clone());
        }
    }
    return None;
}

/// What:
///  The root,
/// Git directory and common directory of the creating worktree,
/// or why there is none.
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
///  The cheap reasons the snapshot rung cannot serve a planned add:
/// the source is no subvolume root,
/// or it and the
///  destination are not on one btrfs filesystem.
/// Why:
///   These need two `statfs` calls and one inode number,
/// so they run before the dear checks.
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
///  The device number of what a path leads to,
/// or nothing when it cannot be examined.
/// Why:
///   Two paths share data blocks only on one device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// statSync(path).dev
/// ```
fn device_of_path(path: &Path) -> Option<u64> {
    return match std::fs::metadata(path) {
        Ok(found) => Some(found.dev()),
        Err(_unreadable) => None,
    };
}

/// What:
///  Whether a kind of filesystem may share data blocks between files at all:
/// btrfs,
/// overlayfs and ZFS may.
/// Why:
///   Those three are the kinds whose clone domain the ladder proves from the mount table;
/// XFS is the one
///  filesystem whose clone support the kind does not hold,
/// so its first clone answers for it and `may_clone` asks that
///  separately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kindMayClone = (kind: FilesystemKind) => kind === 'btrfs' || kind === 'overlayfs' || kind === 'zfs';
/// ```
fn kind_may_clone(kind: FilesystemKind) -> bool {
    return matches!(kind, FilesystemKind::Btrfs | FilesystemKind::Overlayfs | FilesystemKind::Zfs);
}

/// What:
///  Whether the filesystem of `path` may share data blocks between files:
/// btrfs,
/// XFS (reflink is a per-filesystem
///  feature that the first clone reveals),
/// overlayfs (a clone inside the overlay reaches the layers' filesystem) and ZFS
///  (whose block cloning works inside one dataset and nowhere else).
/// Why:
///   Other filesystems never clone,
/// so the clone rung is not tried there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mayClone(path: string): boolean;
/// ```
fn may_clone(path: &Path) -> bool {
    if kind_may_clone(filesystem_kind(path).unwrap_or(FilesystemKind::Other)) {
        return true;
    }
    return crate::fs_kind::is_xfs(path).unwrap_or(false);
}

/// What:
///  What the clone rung reads about one side of a reflink:
/// the path,
/// the kind of filesystem that holds it,
/// that
///  filesystem's device number and the mount source the kernel reports for it.
/// Why:
///   A reflink shares data blocks only inside one clone domain,
/// and each kind of filesystem names that domain
///  differently:
/// one btrfs filesystem has one mount source,
/// one ZFS dataset is named by its mount source and is the one
///  device that name's mount has,
/// and every other filesystem shares blocks only with its own device.
/// The facts are read
///  apart from the decision so the decision can be controlled on any filesystem,
/// and so proving a ZFS dataset needs
///  only the mount table and the device number,
/// never a call into the ZFS binding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneSide = { path: string; kind: FilesystemKind; device?: number; mountSource?: string };
/// ```
struct CloneSide {
    /// The path this side describes,
    /// for the refusal's words.
    path: PathBuf,
    /// The kind of filesystem that holds it;
    /// `Other` also when the kind cannot be read.
    kind: FilesystemKind,
    /// The device number of that filesystem's mount,
    /// or nothing when the path cannot be examined.
    device: Option<u64>,
    /// The mount source of that mount,
    /// or nothing when the table cannot name it;
    /// read only where the
    ///  decision uses it.
    mount_source: Option<String>,
}

/// What:
///  Read the clone facts of one path.
/// Why:
///   The mount table is read only for the two kinds whose clone domain is a mount,
/// so an overlay or XFS
///  ask costs one `statfs` and one `stat`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneSideOf(path: string): CloneSide;
/// ```
fn clone_side_of(path: &Path) -> CloneSide {
    let kind: FilesystemKind = filesystem_kind(path).unwrap_or(FilesystemKind::Other);
    let device: Option<u64> = device_of_path(path);
    let mount_source: Option<String> = match kind {
        FilesystemKind::Btrfs | FilesystemKind::Zfs => read_mount_facts(path).ok().flatten().map(|facts| { return facts.source; }),
        FilesystemKind::Overlayfs | FilesystemKind::Other => None,
    };
    return CloneSide { path: path.to_path_buf(), kind, device, mount_source };
}

/// What:
///  The path whose filesystem receives the clones:
/// the destination when something is already there,
/// and its
///  parent otherwise.
/// Why:
///   Before Git runs there is nothing at the destination,
/// so the bytes land in its parent's filesystem;
/// when
///  the name is taken (a mounted clone at its slot is exactly that),
/// the bytes land in what is there instead,
/// and only a
///  reflink into that filesystem shares blocks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cloneWriteTarget = ([destination, parent]: [string, string]) => existsSync(destination) ? destination : parent;
/// ```
fn clone_write_target((destination, parent): (&Path, &Path)) -> PathBuf {
    if std::fs::metadata(destination).is_ok() {
        return destination.to_path_buf();
    }
    return parent.to_path_buf();
}

/// What:
///  Why the two sides are not one clone domain,
/// or nothing when they are.
/// Why:
///   The kind decides how the domain is named,
/// and a pair of different kinds is two domains whatever their
///  device numbers say;
/// the answer is a reason for the log rather than a word to the person,
/// because nothing the person
///  does on a filesystem that clones can make another filesystem clone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneDomainRefusal(source: CloneSide, target: CloneSide): string | undefined;
/// ```
fn clone_domain_refusal(source: &CloneSide, target: &CloneSide) -> Option<String> {
    if source.kind != target.kind {
        return Some(different_filesystems(source, target));
    }
    return match source.kind {
        FilesystemKind::Btrfs => one_mount_source(source, target),
        FilesystemKind::Zfs => one_zfs_dataset(source, target),
        FilesystemKind::Overlayfs | FilesystemKind::Other => one_device(source, target),
    };
}

/// The refusal of two paths that are not on one filesystem.
fn different_filesystems(source: &CloneSide, target: &CloneSide) -> String {
    return format!("{} and {} are on different filesystems", source.path.display(), target.path.display());
}

/// What:
///  The refusal of two btrfs paths whose mounts name different filesystems,
/// or nothing when they name one.
/// Why:
///   A btrfs clone crosses subvolumes of one filesystem,
/// whose every mount carries the same source in the mount table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oneMountSource = (source, target) => source.mountSource && source.mountSource === target.mountSource ? undefined : differentFilesystems(source, target);
/// ```
fn one_mount_source(source: &CloneSide, target: &CloneSide) -> Option<String> {
    if source.mount_source.is_some() && source.mount_source == target.mount_source {
        return None;
    }
    return Some(different_filesystems(source, target));
}

/// What:
///  The refusal of two paths that are not on one device,
/// or nothing when they are.
/// Why:
///   An overlay or XFS path shares blocks only with a path on its own device,
/// and a device that cannot be read
///  proves nothing either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oneDevice = (source, target) => source.device && source.device === target.device ? undefined : differentFilesystems(source, target);
/// ```
fn one_device(source: &CloneSide, target: &CloneSide) -> Option<String> {
    let (Some(one), Some(other)) = (source.device, target.device) else {
        return Some(format!("{} and {} cannot both be examined, so a reflink between them cannot be proven", source.path.display(), target.path.display()));
    };
    return if one == other { None } else { Some(different_filesystems(source, target)) };
}

/// What:
///  The refusal of two ZFS paths that are not in one dataset,
/// or nothing when they are.
/// Why:
///   A reflink shares data blocks only inside one dataset,
/// and a dataset is named by its mount source and is the one device
///  that name's mount has,
/// so the pair is proven by both facts and refused when either is missing
/// or they
///  disagree;
/// a mounted clone is a dataset of its own,
/// which is what refuses it as a destination and as a source
///  for a slot in the checkout's dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oneZfsDataset = (source, target) => source.mountSource === target.mountSource && source.device === target.device ? undefined : unproven(source, target);
/// ```
fn one_zfs_dataset(source: &CloneSide, target: &CloneSide) -> Option<String> {
    let (Some(source_name), Some(target_name)) = (source.mount_source.as_deref(), target.mount_source.as_deref()) else {
        return Some(format!("the ZFS dataset holding {} or {} cannot be named from the mount table, so a reflink between them cannot be proven", source.path.display(), target.path.display()));
    };
    let (Some(source_device), Some(target_device)) = (source.device, target.device) else {
        return Some(format!("the device of {} or {} cannot be read, so a reflink between them cannot be proven", source.path.display(), target.path.display()));
    };
    if source_name == target_name && source_device == target_device {
        return None;
    }
    return Some(format!(
        "{} is in the ZFS dataset {source_name} (device {source_device}) and {} is in {target_name} (device {target_device}); a reflink shares data blocks only inside one dataset",
        source.path.display(),
        target.path.display()
    ));
}

/// What:
///  The cheap reasons the clone rung cannot serve a planned add:
/// either side is on a filesystem that never clones,
/// or the
///  two are not in one clone domain (one btrfs filesystem,
/// one ZFS dataset,
/// or one device elsewhere).
/// Why:
///   A clone crosses subvolumes of one btrfs filesystem,
/// shares blocks inside one ZFS dataset and nowhere else,
/// and
///  an overlay or XFS is one device;
/// the question is asked of the filesystem that would receive the clones,
/// which is
///  the destination when something already occupies it.
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
    let target: PathBuf = clone_write_target((plan.destination.as_path(), parent));
    if !may_clone(root.as_path()) || !may_clone(target.as_path()) {
        return Some(String::from("the filesystem does not share data blocks between files"));
    }
    return clone_domain_refusal(&clone_side_of(root.as_path()), &clone_side_of(target.as_path()));
}

/// Which kind of check refused a source,
/// so the person can be told the cause in words that fit it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusalKind {
    /// The source could not be proven clean,
    /// for the kind of reason the clean check gives.
    Unclean(CleanRefusalKind),
    /// A feature a copy of the files does not reproduce,
    /// or a state the checks cannot read.
    Unsupported,
    /// The stat data could not vouch for the files,
    /// and the Git run that settles it by content failed.
    StatusFailed,
}

/// What:
///  Why a source cannot be copied,
/// and which kind of check said so.
/// Why:
///   The reason is for the log and for the sentence the person reads;
/// the kind lets the sentence be chosen without reading
///  the reason's words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { reason: string; kind: 'unclean' | 'unsupported' | 'status-failed' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal {
    /// The reason,
    /// in words.
    pub reason: String,
    /// The kind of check that refused.
    pub kind: RefusalKind,
}

impl Refusal {
    /// A refusal by a feature or a state the checks cannot read.
    fn unsupported(reason: impl Into<String>) -> Refusal {
        return Refusal { reason: reason.into(), kind: RefusalKind::Unsupported };
    }
}

/// What:
///  Check the creating worktree for everything a copy of its files needs,
/// from the cheapest check to the dearest,
/// and
///  collect what the later steps use;
/// the refusal is the error.
/// Why:
///   The dear checks (the clean check starts one Git run) happen once for both rungs,
/// and only after a cheap check let a
///  rung through.
/// The kind of the refusal lets the ladder tell the person the cause in fitting words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function sourceRefusal(context: GitContext, identity: WorktreeIdentity): Promise<SnapshotSource>; // throws a Refusal
/// ```
pub fn source_refusal(context: &GitContext, identity: &WorktreeIdentity) -> Result<SnapshotSource, Refusal> {
    return checked_source(context, identity, false);
}

/// What:
///  `source_refusal`,
/// except that a main checkout with only ordinary uncommitted changes is a source too,
/// carrying the list of
///  its changes;
/// every other refusal stays.
/// Why:
///   The ladder snapshots such a source and restores the copy to the new worktree's `HEAD`;
/// the clone rung and the other
///  callers of the clean answer never see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function sourceOrDirty(context: GitContext, identity: WorktreeIdentity): Promise<SnapshotSource>; // throws a Refusal
/// ```
pub fn source_or_dirty(context: &GitContext, identity: &WorktreeIdentity) -> Result<SnapshotSource, Refusal> {
    return checked_source(context, identity, true);
}

/// What:
///  The Git configuration of the source,
/// timed as the work `readConfig`.
/// Why:
///   The read writes nothing,
/// so it runs beside the cleanliness check;
/// the add timing record sums the work of the checks by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork(); const read = readConfigSnapshot(context, root); endWork('readConfig', began);
/// ```
fn read_configuration_timed((context, root): (&GitContext, &Path)) -> Option<ConfigSnapshot> {
    let began: std::time::Instant = begin_work();
    let read: Option<ConfigSnapshot> = read_config_snapshot(context, root);
    end_work("readConfig", began);
    return read;
}

/// What:
///  `read_configuration_timed` with the Git child awaited on the runtime.
/// Why:
///   The read is one of the three the source check makes on a task of the runtime,
/// which starts no child through the
///  synchronous face;
/// the run's bound and quiet window are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork(); const read = await readConfigSnapshotAwaiting(context, root); endWork('readConfig', began);
/// ```
async fn read_configuration_timed_awaiting((context, root): (&GitContext, &Path)) -> Option<ConfigSnapshot> {
    let began: std::time::Instant = begin_work();
    let read: Option<ConfigSnapshot> = read_config_snapshot_awaiting(context, root).await;
    end_work("readConfig", began);
    return read;
}

/// What:
///  Whether the place left-out directories are set aside is ignored in this source,
/// for every kind of source,
///  timed as the work `trashIgnored`.
/// Why:
///   The answer decides whether another command's stage container is renamed into a trash in one step
/// or removed in the
///  foreground,
/// and a linked worktree has the same ignore rules as the checkout it was made from.
/// The
///  repository's own exclude line is written before the question is asked,
/// so the answer does not depend on what the
///  target commit ignores:
/// an add whose target commit ignores nothing under `.claude/` still gets a trash it can set aside
///  at once,
/// and the standing trash is still unstageable (`worktree_trash_exclude_line.rs`).
/// A line that cannot be
///  written leaves the question to the ignore rules the repository already has,
/// and a `false` answer removes the directories
///  in the foreground,
/// which is the safe shape the person already had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork(); ensureTrashIsExcluded(commonDir); const ignored = trashLocationIsIgnored(context, root); endWork('trashIgnored', began);
/// ```
fn trash_location_ignored_timed((context, root, _git_dir, common_dir): (&GitContext, &Path, &Path, &Path)) -> bool {
    let began: std::time::Instant = begin_work();
    let ignored: bool = ignore_line_written(common_dir) && trash_location_is_ignored(context, root);
    end_work("trashIgnored", began);
    return ignored;
}

/// What:
///  Write the repository's exclude line for the trash place,
/// and say whether it is there.
/// Why:
///   The question that follows decides whether a trash is made at all,
/// so a repository whose exclude file cannot be written
///  keeps the foreground removal rather than a trash nothing keeps out of the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ignoreLineWritten = (commonDir) => { try { ensureTrashIsExcluded(commonDir); return true; } catch (reason) { debug(reason); return false; } };
/// ```
fn ignore_line_written(common_dir: &Path) -> bool {
    return match crate::worktree_trash_exclude_line::ensure_trash_is_excluded(common_dir) {
        Ok(_added) => true,
        Err(reason) => {
            debug(TAG, reason.as_str());
            false
        }
    };
}

/// What:
///  `trash_location_ignored_timed` with the Git child awaited on the runtime.
/// Why:
///   The question is one of the three the source check asks on a task of the runtime,
/// beside the configuration read and the
///  cleanliness check;
/// the run's bound and quiet window are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork(); const ignored = await trashLocationIsIgnoredAwaiting(context, root); endWork('trashIgnored', began);
/// ```
async fn trash_location_ignored_timed_awaiting((context, root, _git_dir, common_dir): (&GitContext, &Path, &Path, &Path)) -> bool {
    let began: std::time::Instant = begin_work();
    // The write is a few bytes of one file, small enough to do on this task rather than as a job of the pool, and the
    //  question below must not be asked before it.
    let ignored: bool = ignore_line_written(common_dir) && trash_location_is_ignored_awaiting(context, root).await;
    end_work("trashIgnored", began);
    return ignored;
}

/// The cleanliness check of the source,
/// which runs beside the two reads.
fn clean_check_of_source((context, root, git_dir, common_dir): (&GitContext, &Path, &Path, &Path)) -> Result<CleanSource, CleanRefusal> {
    return check_clean(context, root, git_dir, common_dir);
}

/// What the trash-location read and the cleanliness check found.
type TrashAndCleanVerdicts = (bool, Result<CleanSource, CleanRefusal>);

/// What:
///  `read whether the trash location is ignored` and `check the cleanliness of the source` on a task of the runtime:
/// the trash question awaited
///  through its child and the cleanliness check awaited through its own two waits.
/// Why:
///   The two read different things and write nothing;
/// the verdicts are read in the order the checks have always had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [ignored, checked] = await Promise.all([trashLocationIgnored(args), cleanCheckOfSource(args)]);
/// ```
async fn check_trash_and_cleanliness_awaiting((context, root, git_dir, common_dir): (&GitContext, &Path, &Path, &Path)) -> TrashAndCleanVerdicts {
    return tokio::join!(trash_location_ignored_timed_awaiting((context, root, git_dir, common_dir)), check_clean_awaiting(context, root, git_dir, common_dir));
}

/// What:
///  The three checks the source needs before it can be copied:
/// the configuration read,
/// the ignore rule of the trash
///  location and the cleanliness of the files,
/// each a wait of this task where Git is asked.
/// Why:
///   The three read different things and write nothing,
/// so they run beside each other;
/// the filesystem proof inside the
///  cleanliness check is a job of the blocking pool (`worktree_clean.rs`),
/// so no walk of the source holds the Git waits open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [config, [ignored, checked]] = await Promise.all([readConfiguration(args), checkTrashAndCleanliness(args)]);
/// ```
async fn the_checks_of_the_source(state: (&GitContext, &Path, &Path, &Path)) -> (Option<ConfigSnapshot>, TrashAndCleanVerdicts) {
    return tokio::join!(read_configuration_timed_awaiting((state.0, state.1)), check_trash_and_cleanliness_awaiting(state));
}

/// What:
///  `the_checks_of_the_source` one check after another on the calling thread,
/// for the state where no runtime can be built.
/// Why:
///   The owner's rule is "Commits should never fail.",
/// so an add whose checks cannot share the runtime still gets the same three
///  answers and the same refusals;
/// nothing runs beside anything,
/// which is the only difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const config = readConfiguration(args); const ignored = trashLocationIgnored(args); const checked = checkClean(args);
/// ```
fn the_checks_of_the_source_in_turn((context, root, git_dir, common_dir): (&GitContext, &Path, &Path, &Path)) -> (Option<ConfigSnapshot>, TrashAndCleanVerdicts) {
    let read_config: Option<ConfigSnapshot> = read_configuration_timed((context, root));
    let trash_ignored: bool = trash_location_ignored_timed((context, root, git_dir, common_dir));
    let checked: Result<CleanSource, CleanRefusal> = clean_check_of_source((context, root, git_dir, common_dir));
    return (read_config, (trash_ignored, checked));
}

/// The checks of `source_refusal`,
/// and of `source_or_dirty` when `allow_dirty` is set.
fn checked_source(context: &GitContext, identity: &WorktreeIdentity, allow_dirty: bool) -> Result<SnapshotSource, Refusal> {
    let (root, git_dir, common_dir): (&PathBuf, &PathBuf, &PathBuf) = worktree_of(identity).map_err(Refusal::unsupported)?;
    if git_dir.join("info/sparse-checkout").exists() || common_dir.join("info/sparse-checkout").exists() {
        return Err(Refusal::unsupported("the source has a sparse checkout"));
    }
    let mut nested: Vec<PathBuf> = Vec::new();
    for path in registered_roots(common_dir.as_path()) {
        // A record that is inside the source by its text is kept as written, so a link inside the source is still caught.
        if path.starts_with(root) {
            if path != *root {
                nested.push(path);
            }
            continue;
        }
        // A record written before the repository moved behind a link spells the old name and names a worktree inside the source.
        let real: PathBuf = std::fs::canonicalize(path.as_path()).unwrap_or(path);
        if real != *root && real.starts_with(root) {
            nested.push(real);
        }
    }
    // The configuration, the ignore rule of the place left-out directories are set aside at and the cleanliness of the files
    //  read different things and write nothing, so they run side by side on the process's runtime, or one after another on this
    //  thread where no runtime can be built; the verdicts are read in the order the checks have always had.
    let state: (&GitContext, &Path, &Path, &Path) = (context, root.as_path(), git_dir.as_path(), common_dir.as_path());
    let (read_config, (trash_ignored, checked)): (Option<ConfigSnapshot>, TrashAndCleanVerdicts) = match run_on_runtime(the_checks_of_the_source(state)) {
        Ok(verdicts) => verdicts,
        Err(refusal) => {
            crate::diagnostic_log::debug("checkedSource", format!("the checks of the source run one after another ({refusal})").as_str());
            the_checks_of_the_source_in_turn(state)
        }
    };
    let Some(snapshot) = read_config else {
        return Err(Refusal::unsupported("the Git configuration could not be read"));
    };
    if let Some(reason) = configuration_refusal(snapshot.entries()) {
        return Err(Refusal::unsupported(reason));
    }
    let (clean, dirty): (CleanSource, Option<ChangedPaths>) = match checked {
        Ok(found) => (found, None),
        Err(why) if allow_dirty && restorable_here(identity) && why.kind.restorable() => {
            let found = dirty_source(git_dir.as_path(), common_dir.as_path(), why).map_err(unclean)?;
            // A source whose files Git finds equal to the commit is snapshotted too: the stat data was all that kept it from being clean.
            let changed: ChangedPaths = changed_paths(context, root.as_path()).map_err(status_failed)?;
            if changed.attributes_changed {
                return Err(Refusal::unsupported("a .gitattributes file is among the uncommitted changes, and its rules could change how the restore writes other files"));
            }
            crate::diagnostic_log::debug("sourceOrDirty", format!("{} is snapshotted with {} uncommitted paths, the first {:?}; the restore follows ({})", root.display(), changed.count, changed.first, found.why).as_str());
            (found.source, Some(changed))
        }
        Err(why) => return Err(unclean(why)),
    };
    return Ok(SnapshotSource { root: root.clone(), git_dir: git_dir.clone(), common_dir: common_dir.clone(), clean, nested, config: snapshot, trash_ignored, dirty });
}

/// What:
///  A refusal for a source that is not clean,
/// carrying the reason and kind of the clean check's refusal.
/// Why:
///   The ladder reports a refusal of its own kind,
/// and an unclean source is one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Refusal(why.reason, { unclean: why.kind })
/// ```
fn unclean(why: CleanRefusal) -> Refusal {
    return Refusal { reason: why.reason, kind: RefusalKind::Unclean(why.kind) };
}

/// What:
///  A refusal for a source whose changed paths Git could not list.
/// Why:
///   A source with changes is snapshotted only when its changes can be listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Refusal(reason, 'statusFailed')
/// ```
fn status_failed(reason: String) -> Refusal {
    return Refusal { reason, kind: RefusalKind::StatusFailed };
}

/// What:
///  Whether a source that is not clean can be copied and restored:
/// the main checkout,
/// whose index lies inside the snapshot.
/// Why:
///   The restore starts from the index the snapshot carries,
/// which is one instant with the files;
/// a linked worktree's index
///  lies outside its directory and is not in the snapshot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restorableHere(identity: WorktreeIdentity): boolean;
/// ```
fn restorable_here(identity: &WorktreeIdentity) -> bool {
    return matches!(identity, WorktreeIdentity::MainWorktree { worktree_root, git_dir, .. } if *git_dir == worktree_root.join(".git"));
}

/// What:
///  Check the creating worktree for everything a copy of its files needs;
/// the reason is the error.
/// Why:
///   Callers that only log or compare the reason ask for the words;
/// the ladder asks `source_refusal` for the kind too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commonSource(context: GitContext, identity: WorktreeIdentity): Promise<SnapshotSource>; // throws a reason
/// ```
pub fn common_source(context: &GitContext, identity: &WorktreeIdentity) -> Result<SnapshotSource, String> {
    return match source_refusal(context, identity) {
        Ok(source) => Ok(source),
        Err(refusal) => Err(refusal.reason),
    };
}

/// What:
///  The reason a nested worktree that is an ordinary directory blocks the snapshot.
/// Why:
///   The person is told which worktree to convert and with what.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the worktree ${path} inside the source is an ordinary directory; ...`
/// ```
fn ordinary_worktree_reason(path: PathBuf) -> String {
    return format!("the worktree {} inside the source is an ordinary directory; `git monochromatic-git-policy-cli migrate-to-subvolume` makes it a subvolume", path.display());
}

/// What:
///  The registered worktree inside the source that would be carried into a snapshot with all its files,
/// if any.
/// Why:
///   Only the snapshot rung cares:
/// a nested worktree that is a subvolume is an empty stub in a snapshot.
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
    return nested_ordinary_worktree(source.nested.as_slice()).map(ordinary_worktree_reason);
}

/// What:
///  Decide whether the creating worktree of a planned add can be the snapshot source:
/// the cheap refusals,
/// the shared
///  checks,
/// then what only a snapshot minds.
/// Why:
///   The ladder asks the three in turn so the clone rung can share the dear checks;
/// this is the whole question for the
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
