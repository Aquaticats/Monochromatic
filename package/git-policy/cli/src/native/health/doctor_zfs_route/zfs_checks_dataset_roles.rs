//! What:
//!  Which dataset of the route a fact belongs to,
//!  one dataset with its name and what a read of it answered,
//!  and the check that dataset's facts answer.
//! Why:
//!  The checkout is mounted at its path,
//!  the clone container is a legacy dataset nothing mounts automatically,
//!  and each clone is attached by descriptor under the container's properties,
//!  so the role decides the properties the dataset must carry,
//!  and the role and the reading travel together so a finding names both the
//!  dataset and the property it failed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type DatasetRole = { kind: 'checkout'; mountPoint } | { kind: 'container' } | { kind: 'clone' };
//! type DatasetFacts = { role; name; reading: Reading<CloneObjectFacts> };
//! ```

/// The check each dataset's facts answer.
use super::check_outcome::Check;
/// What a read of one dataset answered.
use super::zfs_checks_reading::Reading;
/// The facts one kernel read of a dataset answered,
/// which the property checks are read from.
use crate::worktree_clone_mount_handler::sweep_dataset_facts::CloneObjectFacts;
/// The path the checkout is mounted at.
use std::path::PathBuf;
//region The datasets and their properties

/// What:
///  Which dataset of the route a fact belongs to,
///  which decides the properties it must carry.
/// Why:
///  The checkout is mounted at its path,
///  the clone container is a legacy dataset nothing mounts automatically,
///  and each clone is attached by descriptor under the container's
///  properties,
///  so one property rule per role is what the checks compare against
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Dataset and path layout" and
///  "Scoped delegation").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DatasetRole = { kind: 'checkout'; mountPoint: string } | { kind: 'container' } | { kind: 'clone' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DatasetRole {
    /// The dataset the checkout sits on,
    /// mounted at the checkout path.
    Checkout {
        /// The path the checkout is mounted at.
        mount_point: PathBuf,
    },
    /// The unmounted container every managed clone is an immediate child of.
    Container,
    /// One managed clone below the container.
    Clone,
}

/// What:
///  One dataset of the route,
///  its role and what a read of it answered.
/// Why:
///  Every property check reads the same facts,
///  so the role and the reading travel together and a finding names both the
///  dataset and the property it failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DatasetFacts = { role: DatasetRole; name: string; reading: Reading<CloneObjectFacts> };
/// ```
#[derive(Clone, Debug)]
pub struct DatasetFacts {
    /// Which dataset of the route this is.
    pub role: DatasetRole,
    /// The dataset's full name.
    pub name: String,
    /// The facts the kernel answered,
    /// or why it answered none.
    pub reading: Reading<CloneObjectFacts>,
}

impl DatasetFacts {
    /// The check this dataset's facts answer.
    pub fn check(&self) -> Check {
        return match self.role {
            DatasetRole::Checkout {
                ..
            } => Check::CheckoutDataset,
            DatasetRole::Container => Check::ContainerDataset,
            DatasetRole::Clone => Check::CloneProperties,
        };
    }
}
//endregion
