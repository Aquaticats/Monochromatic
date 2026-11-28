//! What:
//!  The controls of the dataset facts:
//!  the checkout,
//!  the container and every clone the container lists,
//!  a listing that failed,
//!  and a clone whose own read answered nothing.
//! Why:
//!  Every dataset check walks this one list,
//!  so the order,
//!  the roles and the readings are what keep a check from judging one
//!  dataset against another's facts,
//!  and a listing that failed must answer the clones' checks an unknown rather
//!  than the pass an empty list gets.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theDatasetFacts({ layout, pool, unread })).toHaveLength(3);
//! expect(unread.map((r) => r.check)).toEqual(['clone-properties', 'unresolved-requests']);
//! ```

/// The pool a control scripts.
use super::super::a_scripted_pool_for_each_control::AScriptedPool;
/// The read a control judges.
use super::the_dataset_facts;
/// The check a failed listing answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// The names the design's layout fixes.
use crate::doctor_zfs_route::route_test_facts::{
    CHECKOUT_DATASET,
    CLONE_DATASET,
    CONTAINER_DATASET,
};
/// One dataset's role and reading.
use crate::doctor_zfs_route::zfs_checks_dataset_roles::{
    DatasetFacts,
    DatasetRole,
};
/// What a dataset's read answered.
use crate::doctor_zfs_route::zfs_checks_reading::Reading;
/// The three names a layout holds.
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;
/// The owned and borrowed paths these controls use.
use std::path::{
    Path,
    PathBuf,
};

/// One layout naming the design's datasets at a fixture path.
fn the_layout(checkout: &Path) -> SweepConfiguration {
    return SweepConfiguration {
        parent_dataset: String::from(CONTAINER_DATASET),
        checkout: checkout.to_path_buf(),
        checkout_dataset: String::from(CHECKOUT_DATASET),
    };
}

/// What:
///  The checkout and the container come first,
///  each with its own role and name,
///  and every clone the container listed follows.
/// Why:
///  The report prints one line per dataset in this order,
///  so a control that found the checkout second would read another dataset's
///  answer under the checkout's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theDatasetFacts({ layout, pool, unread }).map((d) => d.name)).toEqual([CHECKOUT_DATASET, CONTAINER_DATASET, CLONE_DATASET]);
/// ```
#[test]
fn the_checkout_the_container_and_each_clone_follow_in_order() {
    let layout: SweepConfiguration = the_layout(Path::new("/fixture/checkout"));
    let pool: AScriptedPool = AScriptedPool::a_healthy_machine();
    let mut unread: Vec<CheckReport> = Vec::new();
    let datasets: Vec<DatasetFacts> = the_dataset_facts(&layout, &pool, &mut unread);
    assert_eq!(datasets.len(), 3);
    assert_eq!(datasets[0].name, CHECKOUT_DATASET);
    assert!(matches!(datasets[0].role, DatasetRole::Checkout { .. }));
    assert_eq!(datasets[1].name, CONTAINER_DATASET);
    assert_eq!(datasets[1].role, DatasetRole::Container);
    assert_eq!(datasets[2].name, CLONE_DATASET);
    assert_eq!(datasets[2].role, DatasetRole::Clone);
    assert!(unread.is_empty(), "{unread:?}");
    let DatasetRole::Checkout {
        mount_point,
    } = &datasets[0].role
    else {
        panic!("the checkout's role carries its path");
    };
    assert_eq!(mount_point, &PathBuf::from("/fixture/checkout"));
}

/// What:
///  A container whose children could not be listed answers the two checks that
///  read the clones an unknown naming the reason,
///  and still reports the checkout and the container.
/// Why:
///  An empty list of clones is what a container holding none gives,
///  so the two checks would pass over work nobody read,
///  and the checkout's own facts are readable and must not be lost with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const datasets = theDatasetFacts({ layout, pool: poolWithFailingChildren, unread });
/// expect(unread.map((r) => r.check)).toEqual(['clone-properties', 'unresolved-requests']);
/// ```
#[test]
fn a_listing_that_failed_answers_the_clones_two_checks_an_unknown() {
    let layout: SweepConfiguration = the_layout(Path::new("/fixture/checkout"));
    let mut pool: AScriptedPool = AScriptedPool::a_healthy_machine();
    pool.children = Err(String::from(
        "the container's children could not be listed (EIO)",
    ));
    let mut unread: Vec<CheckReport> = Vec::new();
    let datasets: Vec<DatasetFacts> = the_dataset_facts(&layout, &pool, &mut unread);
    assert_eq!(datasets.len(), 2);
    assert_eq!(
        unread
            .iter()
            .map(|report| return report.check)
            .collect::<Vec<Check>>(),
        vec![Check::CloneProperties, Check::UnresolvedRequests]
    );
    for report in &unread {
        assert!(report.unknown(), "{}", report.line());
        assert!(report.line().contains("EIO"), "{}", report.line());
    }
}

/// What:
///  A clone whose own read answered nothing keeps its name and its reading,
///  so the property check names it rather than losing it.
/// Why:
///  The listing proves the dataset exists,
///  and a read that failed proves only that this run could not judge it,
///  which is unknown rather than a dataset nobody listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(datasets[2].reading).toEqual({ kind: 'unreadable', why: 'EACCES' });
/// ```
#[test]
fn a_clone_whose_read_failed_keeps_its_name_and_its_reading() {
    let layout: SweepConfiguration = the_layout(Path::new("/fixture/checkout"));
    let pool: AScriptedPool = AScriptedPool::a_healthy_machine().answering(
        CLONE_DATASET,
        Reading::Unreadable {
            why: String::from("EACCES"),
        },
    );
    let mut unread: Vec<CheckReport> = Vec::new();
    let datasets: Vec<DatasetFacts> = the_dataset_facts(&layout, &pool, &mut unread);
    assert_eq!(datasets.len(), 3);
    assert_eq!(datasets[2].name, CLONE_DATASET);
    let Reading::Unreadable {
        why,
    } = &datasets[2].reading
    else {
        panic!("the clone's read failed: {:?}", datasets[2].reading);
    };
    assert_eq!(why, "EACCES");
    assert!(unread.is_empty(), "{unread:?}");
}
