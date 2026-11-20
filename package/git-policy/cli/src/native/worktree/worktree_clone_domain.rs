//! What:
//!  The clone domain of a reflink:
//! whether two paths share data blocks,
//! and why they do not.
//! Why:
//!  A reflink shares data blocks only inside one clone domain,
//! and each kind of filesystem names that domain
//!  differently:
//! one btrfs filesystem has one mount source,
//! one ZFS dataset is named by its mount source and is the one
//!  device that name's mount has,
//! and every other filesystem shares blocks only with its own device.
//! The facts are read
//!  apart from the decision so the decision can be controlled on any filesystem,
//! and so proving a ZFS dataset needs
//!  only the mount table and the device number,
//! never a call into the ZFS binding.
//! The decision sits in a file of its own so `worktree_snapshot_source.rs` stays under the file-size
//!  limit and this file needs nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = cloneDomainRefusal(cloneSideOf(source), cloneSideOf(target)); // throws the reason
//! ```

use crate::fs_kind::may_clone;
use crate::fs_kind::{FilesystemKind, filesystem_kind};
use crate::fs_mount_options::read_mount_facts;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

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
pub(crate) fn clone_write_target((destination, parent): (&Path, &Path)) -> PathBuf {
    if std::fs::metadata(destination).is_ok() {
        return destination.to_path_buf();
    }
    return parent.to_path_buf();
}

/// What:
///  Why a filesystem cannot take or give a reflink,
/// naming the path that cannot,
/// or nothing when both sides can.
/// Why:
///   The answer asks the filesystem that would receive the clones first,
/// which is the destination when
///  something already occupies it,
/// so the refusal names the path whose filesystem was asked and not one whose
///  filesystem never mattered,
/// and a control can tell the destination from its parent by the name the refusal gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unclonable = ([source, target]) => !mayClone(target) ? `${target} cannot receive a clone` : !mayClone(source) ? `${source} cannot give one` : undefined;
/// ```
pub(crate) fn unclonable_filesystem((source, target): (&Path, &Path)) -> Option<String> {
    if !may_clone(target) {
        return Some(format!("{} would receive the clones and its filesystem does not share data blocks between files", target.display()));
    }
    if !may_clone(source) {
        return Some(format!("{} holds the files to clone and its filesystem does not share data blocks between files", source.display()));
    }
    return None;
}

/// What:
///  Why two paths are not one clone domain,
/// or nothing when they are,
/// read from each path.
/// Why:
///   The caller has paths and not the facts of a side,
/// so the reading and the decision are one call for it,
/// and `CloneSide` stays private to the file that owns the decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneDomainRefusalOf([source, target]: [string, string]): string | undefined;
/// ```
pub(crate) fn clone_domain_refusal_of((source, target): (&Path, &Path)) -> Option<String> {
    return clone_domain_refusal(&clone_side_of(source), &clone_side_of(target));
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

/// Clone-domain controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_domain_tests.rs"]
mod tests;
