//! What:
//!  The controls of the caller's mount table:
//!  a table with the checkout's own row and the cache's,
//!  and a table nobody could read.
//! Why:
//!  Three checks read the checkout's row and one reads the rows under the
//!  cache,
//!  so a table that could not be read must answer those four an unknown rather
//!  than reporting the checkout unmounted and the binds private.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theMountFacts({ places, layout, unread }).rows).toHaveLength(2);
//! expect(unread.map((r) => r.check)).toEqual(['checkout-mount', 'checkout-propagation', 'nested-reachability', 'staging-bind-propagation']);
//! ```

/// One fixture of the whole route,
/// which writes this control's table.
use super::super::one_fixtures_places_and_readers::TheFixturesReaders;
/// The read a control judges.
use super::the_mount_facts;
/// The check a failed read answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// The rows and the two names the checks read.
use crate::doctor_zfs_route::mount_checks::MountCheckFacts;
/// The names the design's layout fixes.
use crate::doctor_zfs_route::route_test_facts::{
    CHECKOUT_DATASET,
    CONTAINER_DATASET,
};
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
///  The table's rows answer the checkout's path and dataset the checks read,
///  and the checkout's row is the one the design mounts there.
/// Why:
///  The three mount checks find the checkout's row by the path and the dataset
///  together,
///  so a table read under another path would report a mount no person has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theMountFacts({ places, layout, unread })).toMatchObject({ checkoutPath: places.mountTable && layout.checkout, checkoutDataset: CHECKOUT_DATASET });
/// ```
#[test]
fn a_table_answers_the_rows_and_the_two_names_the_checks_read() {
    let fixture: TheFixturesReaders = TheFixturesReaders::made("mount-table");
    let layout: SweepConfiguration = the_layout(fixture.checkout.as_path());
    let mut unread: Vec<CheckReport> = Vec::new();
    let facts: MountCheckFacts = the_mount_facts(&fixture.places, &layout, &mut unread);
    assert_eq!(facts.checkout_path, fixture.checkout);
    assert_eq!(facts.checkout_dataset, CHECKOUT_DATASET);
    assert_eq!(facts.rows.len(), 4);
    assert!(unread.is_empty(), "{unread:?}");
    let checkout_row = facts
        .rows
        .iter()
        .find(|row| return row.mount_point == fixture.checkout)
        .expect("the checkout's row");
    assert_eq!(checkout_row.source, CHECKOUT_DATASET);
    assert_eq!(checkout_row.root, PathBuf::from("/"));
}

/// What:
///  A table nobody could read answers the four checks that read a row an
///  unknown naming the file,
///  and leaves the rows empty.
/// Why:
///  An empty table would report the checkout as unmounted and the staging binds
///  as private,
///  which are two findings about a machine nobody looked at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(unread.map((r) => r.check)).toEqual(['checkout-mount', 'checkout-propagation', 'nested-reachability', 'staging-bind-propagation']);
/// ```
#[test]
fn a_table_nobody_could_read_answers_four_unknowns() {
    let fixture: TheFixturesReaders = TheFixturesReaders::made("mount-table-gone");
    std::fs::remove_file(fixture.places.mount_table.as_path()).expect("the table goes");
    let layout: SweepConfiguration = the_layout(fixture.checkout.as_path());
    let mut unread: Vec<CheckReport> = Vec::new();
    let facts: MountCheckFacts = the_mount_facts(&fixture.places, &layout, &mut unread);
    assert!(facts.rows.is_empty());
    assert_eq!(
        unread
            .iter()
            .map(|report| return report.check)
            .collect::<Vec<Check>>(),
        vec![
            Check::CheckoutMount,
            Check::CheckoutPropagation,
            Check::NestedReachability,
            Check::StagingBindPropagation,
        ]
    );
    let first: &CheckReport = unread.first().expect("one unknown");
    assert!(
        first
            .line()
            .contains(fixture.places.mount_table.display().to_string().as_str()),
        "{}",
        first.line()
    );
}
