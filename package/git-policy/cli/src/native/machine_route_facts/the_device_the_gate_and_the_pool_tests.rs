//! What:
//!  The controls of the device's facts and the pool's:
//!  the pool a dataset's name begins with,
//!  and the three answers a pool read turns into the report's reading.
//! Why:
//!  A pool a configuration still names and the kernel no longer has is a
//!  finding naming the pool,
//!  one this account could not read is unknown,
//!  and both are read from the pool's own statistics,
//!  so the two arms must be shown apart without this machine's pool.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(thePoolsName('zpcachyos/ROOT/cos/home/x')).toBe('zpcachyos');
//! expect(thePoolFacts({ layout, pool })).toMatchObject({ pool: 'zpcachyos', reading: { kind: 'missing' } });
//! ```

/// The pool a control scripts.
use super::super::a_scripted_pool_for_each_control::AScriptedPool;
/// The places a fixture names.
use super::super::the_route_places::TheRoutePlaces;
/// The two reads a control judges.
use super::{
    the_device_facts,
    the_pool_facts,
    the_pools_name,
};
/// The names the design's layout fixes.
use crate::doctor_zfs_route::route_test_facts::CHECKOUT_DATASET;
/// The device's state and the pool's facts.
use crate::doctor_zfs_route::zfs_checks::{
    DeviceFacts,
    DeviceState,
    PoolFacts,
};
/// What a pool read answered.
use crate::doctor_zfs_route::zfs_checks_reading::Reading;
/// One fixture folder of this call's own.
use crate::testing::test_support::fixture;
/// The three names a layout holds.
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;
/// One dataset's facts,
/// which a scripted pool answers with.
use crate::worktree_clone_mount_handler::sweep_dataset_facts::CloneObjectFacts;
/// The owned and borrowed paths these controls use.
use std::path::{
    Path,
    PathBuf,
};

/// One layout naming the design's checkout dataset.
fn the_layout(checkout: &Path) -> SweepConfiguration {
    return SweepConfiguration {
        parent_dataset: String::from(
            "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets",
        ),
        checkout: checkout.to_path_buf(),
        checkout_dataset: String::from(CHECKOUT_DATASET),
    };
}

/// What:
///  A dataset's name begins with its pool's,
///  and a name with no separator is the pool itself.
/// Why:
///  The pool's own statistics are read by the name its datasets begin with,
///  so a wrong first component would ask the kernel about another pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(thePoolsName('zpcachyos/ROOT/cos/home/x')).toBe('zpcachyos');
/// expect(thePoolsName('zpcachyos')).toBe('zpcachyos');
/// ```
#[test]
fn a_datasets_name_answers_its_pool() {
    assert_eq!(
        the_pools_name("zpcachyos/ROOT/cos/home/monochromatic-git-checkout"),
        "zpcachyos"
    );
    assert_eq!(the_pools_name("zpcachyos"), "zpcachyos");
    assert_eq!(the_pools_name(""), "");
}

/// What:
///  The device's facts name the place's path and the state the pool answered,
///  and a device that is not there answers absent.
/// Why:
///  A finding names the device a person checks,
///  and the release path is the binding's own constant,
///  so a report and an install cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theDeviceFacts({ places, pool })).toEqual({ path: places.device, state: { kind: 'opens' } });
/// ```
#[test]
fn the_devices_facts_name_the_place_and_the_state() {
    let root: PathBuf = fixture("device-facts");
    let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    let pool: AScriptedPool = AScriptedPool::a_healthy_machine();
    let facts: DeviceFacts = the_device_facts(&places, &pool);
    assert_eq!(facts.path, places.device);
    assert_eq!(facts.state, DeviceState::Opens);
    let mut absent: AScriptedPool = AScriptedPool::a_healthy_machine();
    absent.device = DeviceState::Absent;
    assert_eq!(
        the_device_facts(&places, &absent).state,
        DeviceState::Absent
    );
}

/// What:
///  The pool's facts carry the name its dataset begins with and the reading
///  the pool answered:
///  its GUID when it answered,
///  and the missing or unreadable reason when it did not.
/// Why:
///  The pool check decides a finding from a missing pool and an unknown from
///  an unreadable one,
///  so the reading must arrive whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(thePoolFacts({ layout, pool })).toMatchObject({ pool: 'zpcachyos', reading: { kind: 'read', value: 7 } });
/// ```
#[test]
fn the_pools_facts_carry_its_name_and_its_reading() {
    let root: PathBuf = fixture("pool-facts");
    let layout: SweepConfiguration = the_layout(root.join("checkout").as_path());
    let answered: AScriptedPool = AScriptedPool::a_healthy_machine();
    let facts: PoolFacts = the_pool_facts(&layout, &answered);
    assert_eq!(facts.pool, "zpcachyos");
    let Reading::Read(guid) = facts.reading else {
        panic!("the pool answered: {:?}", facts.reading);
    };
    assert_eq!(guid, 7);
    let missing: AScriptedPool = AScriptedPool::a_healthy_machine().answering(
        "zpcachyos",
        Reading::Missing {
            why: String::from("no such pool"),
        },
    );
    let unanswered: PoolFacts = the_pool_facts(&layout, &missing);
    assert_eq!(
        unanswered.reading,
        Reading::Missing {
            why: String::from("no such pool"),
        }
    );
    let unreadable: AScriptedPool = AScriptedPool::a_healthy_machine().answering(
        "zpcachyos",
        Reading::Unreadable {
            why: String::from("EACCES"),
        },
    );
    assert_eq!(
        the_pool_facts(&layout, &unreadable).reading,
        Reading::Unreadable {
            why: String::from("EACCES"),
        }
    );
}

/// What:
///  A pool read that answered a dataset keeps the GUID the pool's own
///  statistics named,
///  and one the scripted pool never answered reads as missing.
/// Why:
///  The pool's identity is the one term the pool check reports,
///  and a name no read answered must not pass for a pool that is there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(thePoolFacts({ layout, pool: scripted }).reading.kind).toBe('missing');
/// ```
#[test]
fn a_pool_no_read_answered_is_missing() {
    let root: PathBuf = fixture("pool-unanswered");
    let layout: SweepConfiguration = the_layout(root.join("checkout").as_path());
    let empty: AScriptedPool = AScriptedPool {
        datasets: Vec::<(String, Reading<CloneObjectFacts>)>::new(),
        ..AScriptedPool::a_healthy_machine()
    };
    let facts: PoolFacts = the_pool_facts(&layout, &empty);
    assert!(
        matches!(facts.reading, Reading::Missing { .. }),
        "{:?}",
        facts.reading
    );
}
