//! What:
//!  The three dataset names the design fixes,
//!  and the lookups that find the checkout,
//! the container and the probe dataset
//!  by them when the configuration leaves an entry out.
//! Why:
//!  With no configuration file the checkout is the one mounted ZFS dataset whose
//!  last name component is `monochromatic-git-checkout`,
//!  the container is the sibling
//!  `monochromatic-git-worktree-clone-datasets` the design places beside it,
//!  and the probe dataset is `nosnap/git-worktree-probe` in the pool that holds
//!  the checkout
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Dataset and path layout";
//!  `doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The source:
//! the probe dataset";
//!  and the project's answers of 2026-11-22 in
//!  `doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "With no configuration file").
//!  A name that matches none,
//!  or several,
//!  is a refusal that lists what the table held and names the file entry that
//!  settles it:
//!  this route never picks one of several,
//!  because a name the account can make is exactly how a root program would be
//!  steered to the wrong dataset.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const checkout = theOneMountedCheckout(mounts);      // { dataset, mountPoint, root }
//! const container = theDesignsContainerBeside(checkout.dataset);
//! ```

use super::mounted_dataset_instance::{MountHoldingAPath, MountedDatasetInstance, MountedDatasets};
use super::mounted_dataset_refusal::MountedDatasetRefusal;
use std::path::{Path, PathBuf};

/// The leaf name the design gives the checkout dataset.
pub const DESIGN_CHECKOUT_DATASET_NAME: &str = "monochromatic-git-checkout";

/// The leaf name the design gives the container of managed clones.
pub const DESIGN_CONTAINER_DATASET_NAME: &str = "monochromatic-git-worktree-clone-datasets";

/// The probe dataset's name inside the pool that holds the checkout,
/// which the start probe's design fixes.
pub const DESIGN_PROBE_DATASET_UNDER_THE_POOL: &str = "nosnap/git-worktree-probe";

/// The last `/`-separated component of a dataset name,
/// which is the leaf name the design fixes.
fn leaf_of(dataset: &str) -> &str {
    return dataset.rsplit_once('/').map_or(dataset, |(_parent, leaf)| return leaf);
}

/// What:
///  Whether a row's dataset is a managed clone of the container,
///  which is never the checkout dataset.
/// Why:
///  The design puts the checkout dataset and the container beside each other,
///  and gives the account `create` on the container alone
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Scoped delegation"),
///  so an account can create a dataset there whose last name component is the
///  checkout's leaf name;
///  a guest run of 2026-11-22 made exactly that clone.
///  The design's dataset is a sibling of the container and never a child of it,
///  so a row inside one is a clone the route itself attaches at a slot,
///  and it is no candidate for the checkout.
///  Without this rule an account's clone would add a second instance and make
///  the sweep refuse,
///  and with it the sweep finds the checkout it should.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clone = row.dataset.split('/').slice(0, -1).at(-1) === 'monochromatic-git-worktree-clone-datasets';
/// ```
pub fn is_a_managed_clone(row: &MountedDatasetInstance) -> bool {
    let Some((parent, _leaf)) = row.dataset.rsplit_once('/') else {
        return false;
    };
    return leaf_of(parent) == DESIGN_CONTAINER_DATASET_NAME;
}

/// What:
///  Why one row the table held under the leaf name is no candidate for the
///  checkout.
/// Why:
///  A person reading the refusal has to see which row was found and why nothing
///  was done with it,
///  because "no dataset" sends them to the setup while a row that is there and
///  ignored sends them to the mount.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const why = !row.ownRoot ? 'a bind tree' : 'a managed clone';
/// ```
fn why_ignored(row: &MountedDatasetInstance) -> String {
    let mut reasons: Vec<&'static str> = Vec::new();
    if !row.shows_the_datasets_own_root() {
        reasons.push("a bind mount of a subtree, which shows no dataset's own root");
    }
    if is_a_managed_clone(row) {
        reasons.push("a managed clone inside the clone container, which is never the checkout");
    }
    return reasons.join(" and ");
}

/// What:
///  The rows of the table that are instances of one dataset name.
/// Why:
///  An instance means the dataset's own root,
/// not a bind mount of a subtree:
///  a subtree mount does not say which dataset the checkout is on,
///  and counting it would turn one dataset into several.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const instances = rows.filter((row) => row.dataset === dataset && row.root === '/');
/// ```
pub fn instances_of(dataset: &str, rows: &[MountedDatasetInstance]) -> Vec<MountedDatasetInstance> {
    return rows.iter().filter(|row| return row.dataset == dataset && row.shows_the_datasets_own_root()).cloned().collect();
}

/// What:
///  The one mounted instance of the design's checkout dataset.
/// Why:
///  With no configuration entry this row is both the checkout dataset and the
///  checkout path,
///  and a name that matches none or several is refused rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const checkout = theOneMountedCheckout(mounts);
/// ```
///
/// # Errors
/// The table could not be read,
/// no row is an instance,
/// or more than one row is.
pub fn the_one_mounted_checkout(mounts: &dyn MountedDatasets) -> Result<MountedDatasetInstance, MountedDatasetRefusal> {
    let rows: Vec<MountedDatasetInstance> = mounts.mounted_datasets().map_err(|said| return MountedDatasetRefusal::TableUnreadable { said })?;
    let carrying_the_leaf: Vec<MountedDatasetInstance> = rows.into_iter().filter(|row| return row.last_name_component() == DESIGN_CHECKOUT_DATASET_NAME).collect();
    let instances: Vec<MountedDatasetInstance> = carrying_the_leaf
        .iter()
        .filter(|row| return row.shows_the_datasets_own_root() && !is_a_managed_clone(row))
        .cloned()
        .collect();
    let ignored: Vec<String> = carrying_the_leaf
        .iter()
        .filter(|row| return !(row.shows_the_datasets_own_root() && !is_a_managed_clone(row)))
        .map(|row| return format!("{} ({})", row.described(), why_ignored(row)))
        .collect();
    return match instances.as_slice() {
        [] => Err(MountedDatasetRefusal::NoneMounted {
            leaf: DESIGN_CHECKOUT_DATASET_NAME,
            ignored,
        }),
        [one] => Ok(one.clone()),
        several => Err(MountedDatasetRefusal::SeveralMounted { leaf: DESIGN_CHECKOUT_DATASET_NAME, found: several.iter().map(|row| return row.described()).collect() }),
    };
}

/// What:
///  The one mount point of a dataset the configuration named.
/// Why:
///  A file that names the dataset and no path needs the path from the dataset's
///  own mount,
///  and a dataset mounted twice says nothing about which path the walk starts at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = theOneMountOf('pool/home/monochromatic-git-checkout', mounts);
/// ```
///
/// # Errors
/// The table could not be read,
/// the dataset is not mounted,
/// or it is mounted more than once.
pub fn the_one_mount_of(dataset: &str, mounts: &dyn MountedDatasets) -> Result<PathBuf, MountedDatasetRefusal> {
    let rows: Vec<MountedDatasetInstance> = mounts.mounted_datasets().map_err(|said| return MountedDatasetRefusal::TableUnreadable { said })?;
    return match instances_of(dataset, rows.as_slice()).as_slice() {
        [] => Err(MountedDatasetRefusal::DatasetNotMounted { dataset: String::from(dataset) }),
        [one] => Ok(one.mount_point.clone()),
        several => Err(MountedDatasetRefusal::DatasetMountedSeveralTimes {
            dataset: String::from(dataset),
            mount_points: several.iter().map(|row| return row.mount_point.display().to_string()).collect(),
        }),
    };
}

/// What:
///  The dataset the checkout's own path is on.
/// Why:
///  The sweep's origin is a snapshot of this dataset,
///  so a path on a mount that is no dataset,
///  or on a dataset of another leaf name,
///  is another checkout and is refused rather than walked;
///  the probe needs the same answer because its dataset's pool is part of that
///  name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dataset = theCheckoutDatasetAt('/home/user/Monochromatic', mounts);
/// ```
///
/// # Errors
/// The table could not be read,
/// no mount stands at the path,
/// the mount there is no ZFS dataset,
/// or its dataset carries another leaf name.
pub fn the_checkout_dataset_at(path: &Path, mounts: &dyn MountedDatasets) -> Result<String, MountedDatasetRefusal> {
    let holding: Option<MountHoldingAPath> = mounts.mount_holding(path).map_err(|said| return MountedDatasetRefusal::TableUnreadable { said })?;
    let Some(mount) = holding else {
        return Err(MountedDatasetRefusal::NoDatasetAtThePath { path: path.to_path_buf() });
    };
    let Some(dataset) = mount.dataset else {
        return Err(MountedDatasetRefusal::NotADataset { path: path.to_path_buf(), found: mount.filesystem_type });
    };
    if leaf_of(dataset.as_str()) != DESIGN_CHECKOUT_DATASET_NAME {
        return Err(MountedDatasetRefusal::LeafNameDiffers { path: path.to_path_buf(), found: dataset, wanted: DESIGN_CHECKOUT_DATASET_NAME });
    }
    return Ok(dataset);
}

/// What:
///  The container the design places beside the checkout dataset.
/// Why:
///  The two are siblings in the design's layout,
///  so a file holding no container name gets the one the layout fixes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const container = theDesignsContainerBeside('pool/ROOT/cos/home/monochromatic-git-checkout');
/// ```
///
/// # Errors
/// The checkout dataset has no parent component.
pub fn the_designs_container_beside(checkout_dataset: &str) -> Result<String, MountedDatasetRefusal> {
    let Some((parent, _leaf)) = checkout_dataset.rsplit_once('/') else {
        return Err(MountedDatasetRefusal::NoParentForTheContainer { dataset: String::from(checkout_dataset) });
    };
    return Ok(format!("{parent}/{DESIGN_CONTAINER_DATASET_NAME}"));
}

/// What:
///  The probe dataset the design names in the pool that holds the checkout.
/// Why:
///  The probe's design fixes its name under the pool and not under the checkout
///  dataset's own parent,
///  so the pool is the first component of the checkout dataset's name and
///  everything after it is the design's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const probe = theDesignsProbeDatasetUnder('zpcachyos/ROOT/cos/home/monochromatic-git-checkout');
/// ```
///
/// # Errors
/// The checkout dataset's name has no pool component.
pub fn the_designs_probe_dataset_under(checkout_dataset: &str) -> Result<String, MountedDatasetRefusal> {
    let Some((pool, _rest)) = checkout_dataset.split_once('/') else {
        return Err(MountedDatasetRefusal::NoPoolForTheProbeDataset { dataset: String::from(checkout_dataset) });
    };
    return Ok(format!("{pool}/{DESIGN_PROBE_DATASET_UNDER_THE_POOL}"));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_designs_datasets_in_the_mount_table_tests.rs"]
mod tests;
