//! What:
//!  The three names the route's facts are about,
//!  read from the handler's configuration file:
//!  the checkout's path,
//!  the checkout dataset,
//!  and the clone container below the same pool.
//! Why:
//!  Every other machine fact is named through these three:
//!  the pool is the checkout dataset's first component,
//!  the mount table's checkout row is found at the path,
//!  a clone is a child of the container,
//!  and the delegation is read at the two datasets.
//!  A file that names none of them is not a refusal here:
//!  the release finds the design's datasets from this process's own mount
//!  table
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "With no configuration file"),
//!  so a machine set up by the design's own names needs no file at all.
//!  A read that names none and finds none stops the whole gather:
//!  without a checkout path and two dataset names no route fact can be named,
//!  and the report says which file and which cause rather than inventing
//!  placeholders a check would print as the machine's own names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const layout = theRoutesLayout(places); // { parentDataset, checkout, checkoutDataset } | { why }
//! ```

/// The places the configuration file is named by.
use super::the_route_places::TheRoutePlaces;
/// The three names the sweep acts on,
/// read from the handler's configuration.
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;
/// The reader of that file,
/// whose refusals say which entry or which discovery failed.
use crate::worktree_clone_mount_handler::sweep_configuration::read_configuration;
/// The datasets this machine's mount table holds,
/// which is what a file naming no dataset is read against.
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;

/// What:
///  The checkout's path,
///  the checkout dataset and the clone container,
///  as the handler's configuration names them or this process's mount table
///  proves them.
/// Why:
///  A report that cannot name these three has no dataset to judge,
///  no pool to read and no path to look at,
///  so the answer is the refusal's own words and the gather stops there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = theRoutesLayout(places); // Ok({ checkout, checkoutDataset, parentDataset }) | Err('the file…')
/// ```
pub(crate) fn the_routes_layout(places: &TheRoutePlaces) -> Result<SweepConfiguration, String> {
    return read_configuration(places.configuration.as_path(), &LiveMountedDatasets).map_err(
        |refusal| {
            return format!(
                "the route's own layout could not be named from {}, so no pool, dataset, mount or delegation fact could be read: {}",
                places.configuration.display(),
                refusal.message()
            );
        },
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_route_layout_tests.rs"]
mod tests;
