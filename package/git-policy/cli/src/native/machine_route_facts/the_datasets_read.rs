//! What:
//!  The datasets the route's checks judge:
//!  the checkout,
//!  the clone container,
//!  and every clone the container lists as its immediate child.
//! Why:
//!  Each dataset's facts are one kernel read,
//!  and a report names the dataset a property fails on,
//!  so the role,
//!  the name and the reading travel together
//!  (`crate::doctor_zfs_route::zfs_checks_dataset_roles`).
//!  A clone is found by listing the container rather than by reading the
//!  store,
//!  because a clone whose record a killed writer never filed is still a
//!  dataset the pool holds and a person must see it.
//!  The listing is the sweep's own walk,
//!  with its own bound,
//!  so a container somebody else filled is reported where it stopped instead
//!  of read for ever.
//!  A listing that failed answers the two checks that read the set of clones an
//!  unknown naming the container,
//!  because an empty list of clones is what a container with no clone holds
//!  and would pass both of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const datasets = theDatasetFacts({ layout, pool, unread }); // [checkout, container, ...clones]
//! ```

/// The pool the datasets are read from.
use super::the_machines_pool_answers::TheMachinesPool;
/// The unknown a read that failed answers.
use super::the_unknown_a_failed_read_answers::the_unknowns_of;
/// One check's stable name,
/// and the report line a failed read answers.
use crate::doctor_zfs_route::check_outcome::Check;
/// One report line.
use crate::doctor_zfs_route::check_outcome::CheckReport;
/// One dataset's role and its name and reading.
use crate::doctor_zfs_route::zfs_checks_dataset_roles::{
    DatasetFacts,
    DatasetRole,
};
/// The three names the route's layout holds.
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;

/// What:
///  The two checks a container's listing of its children feeds.
/// Why:
///  A clone's properties and a clone's request are both read per clone the
///  listing names,
///  and a listing that failed leaves both sets empty,
///  so both answer unknown rather than a pass over nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_CHECKS_OF_THE_CLONES = ['clone-properties', 'unresolved-requests'];
/// ```
const THE_CHECKS_OF_THE_CLONES: &[Check] = &[Check::CloneProperties, Check::UnresolvedRequests];

/// What:
///  The checkout,
///  the container and every clone below it,
///  each with what its own read answered.
/// Why:
///  One list is what every dataset check walks,
///  so a report cannot judge one dataset against another's facts,
///  and a listing that stopped answers the clones' two checks an unknown
///  instead of an empty set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = theDatasetFacts({ layout, pool, unread });
/// ```
pub(crate) fn the_dataset_facts(
    layout: &SweepConfiguration,
    pool: &dyn TheMachinesPool,
    unread: &mut Vec<CheckReport>,
) -> Vec<DatasetFacts> {
    let checkout_dataset: &str = layout.checkout_dataset.as_str();
    let container: &str = layout.parent_dataset.as_str();
    let mut datasets: Vec<DatasetFacts> = vec![
        DatasetFacts {
            role: DatasetRole::Checkout {
                mount_point: layout.checkout.clone(),
            },
            name: String::from(checkout_dataset),
            reading: pool.one_datasets_facts(checkout_dataset),
        },
        DatasetFacts {
            role: DatasetRole::Container,
            name: String::from(container),
            reading: pool.one_datasets_facts(container),
        },
    ];
    match pool.the_containers_children(container) {
        Ok(children) => {
            for child in children {
                datasets.push(DatasetFacts {
                    role: DatasetRole::Clone,
                    name: String::from(child.as_str()),
                    reading: pool.one_datasets_facts(child.as_str()),
                });
            }
        }
        Err(why) => {
            // What:     `extend` appends one unknown per check the failed listing feeds, so neither check passes
            //           over the empty set the failure left.
            // Why:      A container that could not be listed proves no clone's properties and no clone's request.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // unread.push(...theUnknownsOf(THE_CHECKS_OF_THE_CLONES, why));
            // ```
            unread.extend(the_unknowns_of(THE_CHECKS_OF_THE_CLONES, why.as_str()));
        }
    }
    return datasets;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_datasets_read_tests.rs"]
mod tests;
