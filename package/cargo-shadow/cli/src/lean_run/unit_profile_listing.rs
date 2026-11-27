//! What:
//! The profile directories mechanisms B and E read,
//! each with the cap that applies to it,
//! in the order a recorded position indexes.
//! Why:
//! Both mechanisms read the same directories with the same caps,
//! and two runs must walk them in the same order or a recorded position names another place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function unitProfiles(base: string | undefined, checkouts: string[]): [string, number][]
//! const byProfile = (left: [string, number], right: [string, number]): number => ...
//! const sameProfile = (left: [string, number], right: [string, number]): boolean => ...
//! ```
/// Imports the tree listing and the profile directories of one tree.
use crate::lean_build_trees::{
    list_trees,
    profile_dirs_of,
};
/// Imports the caps that apply inside a checkout and in the host build base.
use crate::lean_policy::{
    HOST_PROFILE_CAP_BYTES,
    IN_TREE_PROFILE_CAP_BYTES,
};
/// Imports the target directories of one checkout.
use crate::lean_run_discovery::target_directories;
/// Imports the borrowed and owned path types a profile directory is named by.
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `pub(super) fn unit_profiles(base: Option<&Path>, checkouts: &[PathBuf]) -> Vec<(PathBuf, u64)>` lists every profile directory the sweep and the size caps reach,
/// each with the cap that applies to it.
/// Why:
/// Mechanisms B and E read the same directories with the same caps,
/// in the checkout's own `target/` directories and in the host build base,
/// and the list is sorted so a recorded position names the same place on the next run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unitProfiles(base: string | undefined, checkouts: string[]): [string, number][]
/// ```
pub(super) fn unit_profiles(
    base: Option<&Path>,
    checkouts: &[PathBuf],
) -> Vec<(PathBuf, u64)> {
    let mut profiles: Vec<(PathBuf, u64)> = Vec::new();
    for checkout in checkouts {
        for target in target_directories(checkout) {
            for profile in profile_dirs_of(&target) {
                profiles.push((profile, IN_TREE_PROFILE_CAP_BYTES));
            }
        }
    }
    if let Some(found) = base {
        for tree in list_trees(found) {
            for profile in profile_dirs_of(&tree.path) {
                profiles.push((profile, HOST_PROFILE_CAP_BYTES));
            }
        }
    }
    profiles.sort_by(by_profile);
    profiles.dedup_by(same_profile);
    return profiles;
}

/// What:
/// `fn by_profile(left: &(PathBuf, u64), right: &(PathBuf, u64)) -> std::cmp::Ordering` is the order a recorded position
/// indexes.
/// Why:
/// The units pass continues after a key,
/// so two runs must walk the profile directories in the same order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byProfile = (left: [string, number], right: [string, number]): number => left[0] < right[0] ? -1 : left[0] > right[0] ? 1 : 0;
/// ```
fn by_profile(
    left: &(PathBuf, u64),
    right: &(PathBuf, u64),
) -> std::cmp::Ordering {
    return left.0.cmp(&right.0);
}

/// What:
/// `fn same_profile(left: &mut (PathBuf, u64), right: &mut (PathBuf, u64)) -> bool` says whether two entries name the same
/// directory.
/// Why:
/// A checkout and a build tree can name one profile directory,
/// and judging it twice would pay for it twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sameProfile = (left: [string, number], right: [string, number]): boolean => left[0] === right[0];
/// ```
fn same_profile(
    left: &mut (PathBuf, u64),
    right: &mut (PathBuf, u64),
) -> bool {
    return left.0 == right.0;
}
