//! What:
//!  Why a design name found no dataset,
//!  or found several,
//!  and the words a person reads for each.
//! Why:
//!  A person has to tell a dataset nobody mounted from two datasets that share a
//!  leaf name and from a table this process cannot read,
//!  and each refusal names what was found and the configuration entry that
//!  settles it instead
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 8,
//!  "Where the checkout and the datasets are found").
//!  This route never picks one of several,
//!  because a name the account can make is exactly how a root program would be
//!  steered to the wrong dataset.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type MountedDatasetRefusal = { kind: 'table-unreadable' | 'none-mounted' | ... };
//! ```

use super::the_designs_datasets_in_the_mount_table::{DESIGN_CHECKOUT_DATASET_NAME, DESIGN_CONTAINER_DATASET_NAME, DESIGN_PROBE_DATASET_UNDER_THE_POOL};
use std::path::{Path, PathBuf};

/// What:
///  Why a design name found no dataset,
/// or found several.
/// Why:
///  A person has to tell a dataset nobody mounted from two datasets that share a
///  leaf name and from a table this process cannot read,
///  and each refusal names what was found and the configuration entry that
///  settles it instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountedDatasetRefusal = { kind: 'table-unreadable' | 'none-mounted' | 'several-mounted' | 'dataset-not-mounted' | 'dataset-mounted-several-times' | 'no-parent' | 'no-pool' | 'no-dataset-at-the-path' | 'leaf-differs' | 'not-a-dataset'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MountedDatasetRefusal {
    /// This process's mount table could not be read.
    TableUnreadable {
        /// What the read answered.
        said: String,
    },
    /// No mounted instance of the checkout dataset exists,
    /// and here is what the table held whose source carried the leaf name.
    NoneMounted {
        /// The leaf name looked for.
        leaf: &'static str,
        /// The rows whose source carried it and that are no instance,
        /// each with the reason it is none.
        ignored: Vec<String>,
    },
    /// More than one mounted instance of the checkout dataset exists,
    /// and this route never picks one of several.
    SeveralMounted {
        /// The leaf name looked for.
        leaf: &'static str,
        /// Every instance,
        /// described.
        found: Vec<String>,
    },
    /// The dataset the configuration named has no mount,
    /// so no mount point says where the checkout path is.
    DatasetNotMounted {
        /// The dataset.
        dataset: String,
    },
    /// The dataset the configuration named is mounted more than once,
    /// so no single mount point says where the checkout path is.
    DatasetMountedSeveralTimes {
        /// The dataset.
        dataset: String,
        /// Every mount point of it.
        mount_points: Vec<String>,
    },
    /// The checkout dataset has no parent component,
    /// so the design's container cannot sit beside it.
    NoParentForTheContainer {
        /// The dataset whose parent was needed.
        dataset: String,
    },
    /// The checkout dataset's name has no pool component,
    /// so the design's probe dataset cannot be named in its pool.
    NoPoolForTheProbeDataset {
        /// The dataset whose pool was needed.
        dataset: String,
    },
    /// No mount stands at the checkout's own path,
    /// so the pool the probe dataset lives in cannot be told.
    NoDatasetAtThePath {
        /// The path that was asked about.
        path: PathBuf,
    },
    /// The dataset the checkout's path is on carries another leaf name,
    /// so it is another checkout's dataset.
    LeafNameDiffers {
        /// The path that was asked about.
        path: PathBuf,
        /// The dataset mounted there.
        found: String,
        /// The leaf name this route finds.
        wanted: &'static str,
    },
    /// The checkout's path is on a mount that is no ZFS dataset.
    NotADataset {
        /// The path that was asked about.
        path: PathBuf,
        /// The filesystem type it is on.
        found: String,
    },
}

impl MountedDatasetRefusal {
    /// The word that says why the checkout could not be told,
    /// as a record that holds closed words carries it.
    pub fn word(&self) -> &'static str {
        return match self {
            MountedDatasetRefusal::TableUnreadable { .. } => "mount-table-unreadable",
            MountedDatasetRefusal::SeveralMounted { .. } | MountedDatasetRefusal::DatasetMountedSeveralTimes { .. } => "several-mounted-checkouts",
            MountedDatasetRefusal::NoneMounted { .. }
            | MountedDatasetRefusal::DatasetNotMounted { .. }
            | MountedDatasetRefusal::NoDatasetAtThePath { .. }
            | MountedDatasetRefusal::LeafNameDiffers { .. }
            | MountedDatasetRefusal::NotADataset { .. } => "no-mounted-checkout",
            MountedDatasetRefusal::NoParentForTheContainer { .. } | MountedDatasetRefusal::NoPoolForTheProbeDataset { .. } => "no-source-name",
        };
    }

    /// The refusal in words,
    /// naming the names looked for,
    /// what was found,
    /// and the configuration entry or setup command that settles it.
    pub fn message(&self, configuration: &Path) -> String {
        let cause: String = match self {
            MountedDatasetRefusal::TableUnreadable { said } => format!(
                "this process's own mount table could not be read ({said}), and no configuration names the checkout or the checkout dataset, so nothing here says which dataset the route acts on"
            ),
            MountedDatasetRefusal::NoneMounted { leaf, ignored } => format!(
                "no mount in this process's own mount table is an instance of the checkout dataset, whose last name component is `{leaf}`, and the configuration names neither `cloneMounts.checkout` nor `cloneMounts.checkoutDataset`. A row counts only when its root within the dataset is `/`, and a dataset whose parent's last component is `{DESIGN_CONTAINER_DATASET_NAME}` is a managed clone the route attaches at a slot, never the checkout. The table's ZFS rows whose source carried that leaf name are {}",
                listed_or_nothing(ignored.as_slice())
            ),
            MountedDatasetRefusal::SeveralMounted { leaf, found } => format!(
                "two or more mounts in this process's own mount table are instances of the dataset whose last name component is `{leaf}`, so this run will not choose between them; two pools that hold that leaf name, one dataset mounted at two paths, and a bind mount of the dataset's own root outside it all look like this. The instances are {}",
                found.join(", ")
            ),
            MountedDatasetRefusal::DatasetNotMounted { dataset } => format!(
                "`cloneMounts.checkoutDataset` names `{dataset}` and this process's own mount table holds no ZFS mount of it at its own root, so no mount point says where the checkout path is"
            ),
            MountedDatasetRefusal::DatasetMountedSeveralTimes { dataset, mount_points } => format!(
                "`cloneMounts.checkoutDataset` names `{dataset}` and this process's own mount table mounts it {} times, so no single mount point says where the checkout path is: {}",
                mount_points.len(),
                mount_points.join(", ")
            ),
            MountedDatasetRefusal::NoParentForTheContainer { dataset } => format!(
                "`{dataset}` has no parent component, so `{DESIGN_CONTAINER_DATASET_NAME}` cannot be named beside it"
            ),
            MountedDatasetRefusal::NoPoolForTheProbeDataset { dataset } => format!(
                "`{dataset}` is a single name component, so no pool name can be put before `{DESIGN_PROBE_DATASET_UNDER_THE_POOL}` for the probe dataset"
            ),
            MountedDatasetRefusal::NoDatasetAtThePath { path } => format!("no mount stands at {}, so no dataset there says which pool the probe dataset lives in", path.display()),
            MountedDatasetRefusal::LeafNameDiffers { path, found, wanted } => format!(
                "the checkout {} is on the dataset `{found}`, and the dataset this route finds by its own name is the one whose last component is `{wanted}`; a checkout of another dataset is another checkout's, which one probe's pass says nothing about",
                path.display()
            ),
            MountedDatasetRefusal::NotADataset { path, found } => format!(
                "the checkout {} is on a `{found}` mount rather than on a ZFS dataset, and the design's own name only finds a dataset",
                path.display()
            ),
        };
        return format!(
            "{cause}. Nothing was enumerated, attached or recorded, and every clone's mount intent stands untouched. {remedy}",
            remedy = self.remedy(configuration)
        );
    }

    /// What a person does about this refusal,
    /// naming the setup command or the file entry.
    pub fn remedy(&self, configuration: &Path) -> String {
        let file = configuration.display();
        return match self {
            MountedDatasetRefusal::TableUnreadable { .. } => format!(
                "Ways forward: run the route from a process that can read `/proc/self/mountinfo`, or write `cloneMounts.checkout` and `cloneMounts.checkoutDataset` in {file}."
            ),
            MountedDatasetRefusal::NoneMounted { .. } | MountedDatasetRefusal::DatasetNotMounted { .. } => format!(
                "Ways forward: run plan step 19's setup, which creates the checkout dataset `{DESIGN_CHECKOUT_DATASET_NAME}` and mounts it at the checkout path, then run this again; or write `cloneMounts.checkout` (the path) or `cloneMounts.checkoutDataset` (the dataset) in {file}. `mount-git-worktree-clone-datasets doctor` reports which entry is missing."
            ),
            MountedDatasetRefusal::SeveralMounted { .. } | MountedDatasetRefusal::DatasetMountedSeveralTimes { .. } => format!(
                "Ways forward: leave one mounted instance of the checkout dataset and run this again, or name the dataset with `cloneMounts.checkoutDataset` and its path with `cloneMounts.checkout` in {file}."
            ),
            MountedDatasetRefusal::NoParentForTheContainer { .. } => format!(
                "Ways forward: write `cloneMounts.parent` in {file}, naming the dataset whose immediate children are the managed clones."
            ),
            MountedDatasetRefusal::NoPoolForTheProbeDataset { .. } | MountedDatasetRefusal::NoDatasetAtThePath { .. } => {
                format!("Ways forward: write `startProbe.dataset` in {file}, naming the probe dataset in full.")
            }
            MountedDatasetRefusal::LeafNameDiffers { found, .. } => format!(
                "Ways forward: mount the design's checkout dataset at that path, which plan step 19's setup does, or write `cloneMounts.checkoutDataset` in {file} naming `{found}`."
            ),
            MountedDatasetRefusal::NotADataset { .. } => format!(
                "Ways forward: mount the design's checkout dataset at that path, which plan step 19's setup does, or write `cloneMounts.checkoutDataset` in {file} naming the dataset."
            ),
        };
    }
}

/// One list in words,
/// or a phrase saying it held nothing.
fn listed_or_nothing(items: &[String]) -> String {
    return if items.is_empty() { String::from("none, so no row's source carried it") } else { items.join("; ") };
}
