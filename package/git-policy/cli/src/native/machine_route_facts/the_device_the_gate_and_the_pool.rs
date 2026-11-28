//! What:
//!  The three facts a report reads before it reads any dataset:
//!  this account's look at and open of `/dev/zfs`,
//!  the loaded module's layout against the request contract,
//!  and the pool the checkout dataset names.
//! Why:
//!  Every other pool read stands on the device,
//!  a layout that does not match disables every raw request,
//!  reads included,
//!  and a pool's name in a configuration can outlive the pool,
//!  so each of the three answers its own check before anything below it
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  `doc/decision/cli-git-zfs-runtime-layout-checker.md`).
//!  The device's path travels with its state and the pool's name with its
//!  reading,
//!  so a finding names the file and the pool a person opens.
//!  The layout gate's verdict is the process's own answer,
//!  taken as it is:
//!  a refusal at the device step proves nothing about the layout,
//!  so that one is unknown rather than a second finding about a device the
//!  device check already names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const device = theDeviceFacts({ places, pool }); // { path: '/dev/zfs', state: { kind: 'opens' } }
//! const poolFacts = thePoolFacts({ layout, pool }); // { pool: 'zpcachyos', reading: { kind: 'read', value: 9 } }
//! ```

/// The pool the kernel answers beneath the device.
use super::the_machines_pool_answers::TheMachinesPool;
/// The places the device is named by.
use super::the_route_places::TheRoutePlaces;
/// The device's facts,
/// with the path a finding names.
use crate::doctor_zfs_route::zfs_checks::DeviceFacts;
/// The pool's name and what a read of the pool itself answered.
use crate::doctor_zfs_route::zfs_checks::PoolFacts;
/// What one dataset's read answered,
/// which the pool's own identity is read through.
use crate::doctor_zfs_route::zfs_checks_reading::Reading;
/// The three names the route's layout holds.
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;

/// The separator between a pool's name and the datasets below it.
const DATASET_NAME_SEPARATOR: char = '/';

/// What:
///  This account's look at the device and its open,
///  with the path a finding names.
/// Why:
///  The route needs the account's own access rather than root's,
///  so the open is made as this process,
///  and a look that was refused proves nothing either way and stays its own
///  state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = theDeviceFacts({ places, pool }); // { path, state }
/// ```
pub(crate) fn the_device_facts(
    places: &TheRoutePlaces,
    pool: &dyn TheMachinesPool,
) -> DeviceFacts {
    return DeviceFacts {
        path: places.device.clone(),
        state: pool.the_devices_state(),
    };
}

/// What:
///  The pool the checkout dataset names,
///  read from the first component of the checkout dataset's own name.
/// Why:
///  A dataset's name begins with its pool's,
///  and the pool's own statistics are what prove the pool is there,
///  so the name comes from the layout rather than a second entry a file could
///  disagree with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = thePoolsName('zpcachyos/home/monochromatic-git-checkout'); // 'zpcachyos'
/// ```
pub(crate) fn the_pools_name(checkout_dataset: &str) -> String {
    // What:     `split` walks the name's components and `next` takes the first, which is the pool.
    // Why:      A name with no separator is the pool itself and answers whole.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return checkoutDataset.split('/')[0] ?? '';
    // ```
    let first: Option<&str> = checkout_dataset.split(DATASET_NAME_SEPARATOR).next();
    return String::from(first.unwrap_or(""));
}

/// What:
///  The pool's facts:
///  its name and what a read of the pool itself answered.
/// Why:
///  A pool a configuration still names and the kernel no longer has is a
///  finding naming the pool,
///  and one this account could not read is unknown,
///  so the reading travels with the name rather than a bare boolean.
///  The pool's own GUID is what the kernel answers for it,
///  and the dataset read that carries a GUID is the one read of it there is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = thePoolFacts({ layout, pool }); // { pool: 'zpcachyos', reading: { kind: 'read', value: 9 } }
/// ```
pub(crate) fn the_pool_facts(
    layout: &SweepConfiguration,
    pool: &dyn TheMachinesPool,
) -> PoolFacts {
    let name: String = the_pools_name(layout.checkout_dataset.as_str());
    let identity: Reading<u64> = match pool.one_datasets_facts(name.as_str()) {
        Reading::Read(facts) => Reading::Read(facts.guid),
        Reading::Missing {
            why,
        } => Reading::Missing {
            why,
        },
        Reading::Unreadable {
            why,
        } => Reading::Unreadable {
            why,
        },
    };
    return PoolFacts {
        pool: name,
        reading: identity,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_device_the_gate_and_the_pool_tests.rs"]
mod tests;
