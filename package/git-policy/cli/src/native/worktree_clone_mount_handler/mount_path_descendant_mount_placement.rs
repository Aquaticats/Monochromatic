//! What:
//!  The walk from the checkout's opened mount to the mountpoint and the mount
//!  placed on the folder that walk opened:
//!  one component's creation when it is missing,
//!  the names the walk takes,
//!  and the two lines a run answers.
//! Why:
//!  The mount lands on the descriptor the walk held,
//!  never on a path,
//!  so a folder an account renamed onto the path receives nothing,
//!  and a missing last component is created while a missing intermediate one
//!  refuses
//!  (`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`).
//!  The walk and the mount are their own file because what a run decides and
//!  what a run places are two readings
//!  (rule `MXL`):
//!  `mount_path_descendant_mode_plan.rs` proves the plan this file acts on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = placeTheMount({ dataset, checkout, mountpoint, attributes });
//! ```

use super::mount_path_descendant_mode_plan::MountPlan;
use super::mount_path_descendant_mode_refusals::MountPathDescendantFailure;
use crate::worktree_clone_mount::attach_the_clone::attach_with_attributes;
use std::path::Path;

/// What:
///  The walk from the checkout's opened mount to the mountpoint,
///  and the mount placed on the folder the walk opened.
/// Why:
///   The mount lands on the descriptor the walk held,
///  never on a path,
///  so a folder an account renamed onto the path receives nothing,
///  and a missing last component is created while a missing intermediate one
///  refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = placeTheMount({ dataset, checkout, mountpoint, attributes });
/// ```
///
/// # Errors
/// `MountpointUnusable` when the walk refuses,
/// and `MountFailed` when the primitive does.
pub(super) fn place_the_mount(plan: &MountPlan) -> Result<String, MountPathDescendantFailure> {
    let root: std::fs::File = crate::subvolume::fs_beneath::open_root(plan.checkout.as_path())
        .map_err(|error| {
            return MountPathDescendantFailure::MountpointUnusable {
                dataset: plan.dataset.clone(),
                mountpoint: plan.mountpoint.clone(),
                because: error.to_string(),
            };
        })?;
    let names = relative_names(plan.checkout.as_path(), plan.mountpoint.as_path());
    let Some((last, between)) = names.split_last() else {
        return Err(MountPathDescendantFailure::MountpointUnusable {
            dataset: plan.dataset.clone(),
            mountpoint: plan.mountpoint.clone(),
            because: String::from("the mountpoint is the checkout itself"),
        });
    };
    let held: Vec<std::fs::File> = crate::subvolume::fs_beneath_walk_chain::open_chain_beneath(
        &root, between,
    )
    .map_err(|refusal| {
        return MountPathDescendantFailure::MountpointUnusable {
            dataset: plan.dataset.clone(),
            mountpoint: plan.mountpoint.clone(),
            because: describe_chain_refusal(&refusal),
        };
    })?;
    let Some(parent) = held.last() else {
        return Err(MountPathDescendantFailure::MountpointUnusable {
            dataset: plan.dataset.clone(),
            mountpoint: plan.mountpoint.clone(),
            because: String::from("the walk held no descriptor"),
        });
    };
    let folder: std::fs::File =
        match crate::subvolume::fs_beneath_walk_chain::open_directory_at_a_mount(
            parent,
            last.as_os_str(),
        ) {
            Ok(opened) => opened,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                create_and_open(parent, last.as_os_str()).map_err(|why| {
                    return MountPathDescendantFailure::MountpointUnusable {
                        dataset: plan.dataset.clone(),
                        mountpoint: plan.mountpoint.clone(),
                        because: why,
                    };
                })?
            }
            Err(error) => {
                return Err(MountPathDescendantFailure::MountpointUnusable {
                    dataset: plan.dataset.clone(),
                    mountpoint: plan.mountpoint.clone(),
                    because: error.to_string(),
                });
            }
        };
    attach_with_attributes(plan.dataset.as_str(), &folder, plan.attributes).map_err(|refusal| {
        return MountPathDescendantFailure::MountFailed {
            dataset: plan.dataset.clone(),
            refusal,
        };
    })?;
    return Ok(mounted_line(
        plan.dataset.as_str(),
        plan.mountpoint.as_path(),
    ));
}

/// What:
///  One walk refusal in words a person can act on,
///  naming the component that stopped the walk.
/// Why:
///  The walk's own type carries the component and the system's error,
///  and a mountpoint refusal that said only "the walk refused" would send a
///  person to the whole path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = describeChainRefusal({ kind: 'link', step: 1, component: 'worktrees' });
/// ```
fn describe_chain_refusal(
    refusal: &crate::subvolume::fs_beneath_walk_chain::ChainRefusal
) -> String {
    use crate::subvolume::fs_beneath_walk_chain::ChainRefusal;
    return match refusal {
        ChainRefusal::Link {
            step,
            component,
        } => format!(
            "component {} (`{}`) is a link, and the walk follows none",
            step + 1,
            component.to_string_lossy()
        ),
        ChainRefusal::Boundary {
            step,
            component,
        } => format!(
            "component {} (`{}`) is a mount point or a folder on another device",
            step + 1,
            component.to_string_lossy()
        ),
        ChainRefusal::Unreadable {
            step,
            error,
        } => format!("component {} could not be opened: {error}", step + 1),
    };
}

/// What:
///  Create a missing last component as a folder only root may write and reopen
///  it without following a link.
/// Why:
///   The mode creates the mountpoint and never a tree above it,
///  so only the last component is ever made,
///  and the descriptor acts on the folder it opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = mkdirAndOpen(parent, 'output');
/// ```
fn create_and_open(
    parent: &std::fs::File,
    name: &std::ffi::OsStr,
) -> Result<std::fs::File, String> {
    rustix::fs::mkdirat(parent, name, rustix::fs::Mode::from_bits_truncate(0o755))
        .map_err(|error| return format!("the last component could not be created: {error}"))?;
    return rustix::fs::openat(
        parent,
        name,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map(std::fs::File::from)
    .map_err(|error| return format!("the folder could not be reopened: {error}"));
}

/// The components of a mountpoint below a checkout,
/// as names,
/// which is what the walk takes.
fn relative_names(
    checkout: &Path,
    mountpoint: &Path,
) -> Vec<std::ffi::OsString> {
    let relative: &Path = mountpoint.strip_prefix(checkout).unwrap_or(mountpoint);
    return relative
        .components()
        .map(|component| return component.as_os_str().to_os_string())
        .collect();
}

/// The line a run that placed a mount answers.
fn mounted_line(
    dataset: &str,
    mountpoint: &Path,
) -> String {
    return format!("mounted: {dataset} at {}", mountpoint.display());
}
