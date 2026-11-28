//! What:
//!  The caller's own mount table,
//!  read once into the rows the checkout's mount checks walk.
//! Why:
//!  The checkout's row proves the dataset is mounted where the layout says,
//!  its root field proves the row is the dataset's own mount rather than a bind
//!  of a subtree,
//!  its optional fields say whether the mount is private,
//!  and every other row under the checkout path says what a later mount may
//!  have hidden
//!  (`crate::doctor_zfs_route::mount_checks`;
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "One-time move and checkout boot protection").
//!  The kernel's line grammar is read by the crate's own parser
//!  (`crate::subvolume::fs_mount_options`),
//!  so this file only opens the file,
//!  keeps the read bounded and hands the bytes on.
//!  A table that could not be read answers the four checks that read a row an
//!  unknown naming the file,
//!  because an empty table would report the checkout as unmounted and the
//!  staging binds as private without proving either.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mounts = theMountFacts({ places, layout, unread }); // { checkoutPath, checkoutDataset, rows }
//! ```

/// The places the table is named by.
use super::the_route_places::TheRoutePlaces;
/// The unknown a read that failed answers.
use super::the_unknown_a_failed_read_answers::the_unknowns_of;
/// One check's stable name,
/// and the report line a failed read answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// The rows the mount checks read,
/// the checkout's path and the dataset that should be mounted there.
use crate::doctor_zfs_route::mount_checks::MountCheckFacts;
/// The read of one plain file,
/// with the bound a mount table is read under.
use crate::foundation::plain_file::{
    MOUNT_TABLE_LIMIT,
    read_plain_file,
};
/// The kernel's mountinfo grammar and one parsed row.
use crate::subvolume::fs_mount_options::{
    MountFacts,
    parse_mount_table,
};
/// The three names the route's layout holds.
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;

/// What:
///  The four checks that read a row of the caller's table.
/// Why:
///  The checkout's own row answers three checks and the rows under the cache
///  answer the staging bind's propagation,
///  so a table nobody could read leaves all four unproved,
///  and each answers its own unknown rather than a finding about a mount
///  nobody saw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_CHECKS_OF_THE_TABLE = ['checkout-mount', 'checkout-propagation', 'nested-reachability', 'staging-bind-propagation'];
/// ```
const THE_CHECKS_OF_THE_TABLE: &[Check] = &[
    Check::CheckoutMount,
    Check::CheckoutPropagation,
    Check::NestedReachability,
    Check::StagingBindPropagation,
];

/// What:
///  The checkout's path,
///  the dataset the design mounts there,
///  and every row of the caller's own table.
/// Why:
///  The three facts decide which row is the checkout's and whether a nested row
///  is reachable,
///  so one value carries both and a report cannot mix two tables.
///  A read that failed answers the four checks an unknown and leaves the rows
///  empty,
///  which no check reads because each of them answered already.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mounts = theMountFacts({ places, layout, unread });
/// ```
pub(crate) fn the_mount_facts(
    places: &TheRoutePlaces,
    layout: &SweepConfiguration,
    unread: &mut Vec<CheckReport>,
) -> MountCheckFacts {
    let path: &std::path::Path = places.mount_table.as_path();
    let read: Result<Vec<u8>, std::io::Error> = read_plain_file(path, MOUNT_TABLE_LIMIT);
    let rows: Vec<MountFacts> = match read {
        Ok(bytes) => parse_mount_table(bytes.as_slice()),
        Err(error) => {
            // What:     The reason names the file and the kernel's own answer, so a person knows which table the
            //           report could not read.
            // Why:      Every mount fact of the report stands on this one read.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // why = `the caller's mount table in ${path} could not be read (${error})`;
            // ```
            let why: String = format!(
                "the caller's mount table in {} could not be read ({error}), so no mount row of this report was proved",
                path.display()
            );
            unread.extend(the_unknowns_of(THE_CHECKS_OF_THE_TABLE, why.as_str()));
            Vec::new()
        }
    };
    return MountCheckFacts {
        checkout_path: layout.checkout.clone(),
        checkout_dataset: String::from(layout.checkout_dataset.as_str()),
        rows,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_mount_table_read_tests.rs"]
mod tests;
