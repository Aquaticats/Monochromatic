//! What:
//!  The ZFS datasets this process's own mount table shows mounted,
//!  and the design names the route finds the checkout,
//! its container and the
//!  probe dataset by.
//! Why:
//!  The project answered on 2026-11-22 that everything works by default with no
//!  configuration file:
//!  with no file,
//!  with an empty object,
//!  or with a file that leaves an entry out,
//!  the root handler finds the checkout as the one mounted ZFS dataset whose last
//!  name component is `monochromatic-git-checkout`,
//!  takes the container beside it,
//!  and the start probe finds its dataset by its own design name too
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "With no configuration file",
//!  and `doc/decision/cli-git-zfs-start-probe.md`,
//!  "Answer of 2026-11-22:
//! the probe needs no configuration file").
//!  The wrapper and the root modes both need that answer and neither may read the other's folder,
//!  so it sits here,
//! above both
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 8,
//!  "Where the checkout and the datasets are found").
//!  It reads and mounts nothing:
//!  every row comes from the kernel's table,
//! and a control hands in a table it
//!  wrote.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `mounted_dataset_instance.rs`:
//!   one row of the mount table that is a ZFS dataset's mount,
//!   what a caller reads from it,
//!   and the reader the release path and a control each supply.
//! - `mounted_dataset_refusal.rs`:
//!   why a design name found no dataset or several,
//!   and the words a person reads for each.
//! - `the_designs_datasets_in_the_mount_table.rs`:
//!   the three names the design gives,
//!   and the lookups that find them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mounts = new LiveMountedDatasets();
//! const checkout = theOneMountedCheckout(mounts); // { dataset, mountPoint, root }
//! const container = theDesignsContainerBeside(checkout.dataset);
//! ```

/// One ZFS dataset's mount,
/// as one row of the kernel's table.
pub mod mounted_dataset_instance;

/// Why a design name found no dataset or several,
/// and the words for each.
pub mod mounted_dataset_refusal;

/// The design's own dataset names,
/// and the lookups that find them.
pub mod the_designs_datasets_in_the_mount_table;
