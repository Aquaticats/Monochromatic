//! What:
//!  The controls of the whole gather:
//!  a machine at the design's own state,
//!  one read that failed,
//!  a layout the gather cannot name,
//!  and the clone group the sibling round fills.
//! Why:
//!  The gather is one body that reads every fact in the design's order,
//!  so a control must show that every fact arrives,
//!  that one failed read costs its own checks and no other,
//!  and that a machine whose layout cannot be named answers the sentence
//!  instead of a report about names nobody read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gathered = the_machines_route_facts({ places, pool, store, readiness, receipts });
//! expect(gathered.unread).toEqual([]);
//! ```

/// The pool a control scripts.
use super::super::a_scripted_pool_for_each_control::AScriptedPool;
/// One fixture of the whole route.
use super::super::one_fixtures_places_and_readers::TheFixturesReaders;
/// The value the whole gather answers,
/// and its refusal.
use super::{
    MachineFactsRefusal,
    MachineRouteFacts,
    TheMachinesReads,
    the_machines_route_facts,
};
/// One check's stable name,
/// and its report line.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// The checks over the facts this gather reads.
use crate::doctor_zfs_route::route_checks::route_checks;
/// The names the design's layout fixes.
use crate::doctor_zfs_route::route_test_facts::{
    CHECKOUT_DATASET,
    CLONE_DATASET,
    CONTAINER_DATASET,
    LIVE_BOOT,
};
/// The store folder a control opened,
/// and the refusal of one that could not be.
use crate::worktree_clone_mount::{
    SlotIdentityStore,
    StoreRefusal,
};

/// One gather over one fixture and one scripted pool.
fn gathered(
    fixture: &TheFixturesReaders,
    pool: &AScriptedPool,
) -> Result<MachineRouteFacts, MachineFactsRefusal> {
    let store: Result<SlotIdentityStore, StoreRefusal> = fixture.store();
    let reads: TheMachinesReads<'_> = TheMachinesReads {
        places: &fixture.places,
        pool,
        store: store.as_ref(),
        readiness: &fixture.readiness,
        receipts: &fixture.receipts,
    };
    return the_machines_route_facts(reads);
}

/// What:
///  A machine at the design's state gives every machine fact from its own read,
///  and only the checks a fixture cannot prove are reported as unknown.
/// Why:
///  The report prints these facts under the checks' own names,
///  so one fact left at a placeholder would answer a check nobody could
///  explain.
///  The slot's walk opens the store under root control,
///  which a fixture's folder cannot answer,
///  so that one check answers unknown here and the guest arm proves its pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(gathered.unread).toEqual([]);
/// expect(gathered.route.pool.pool).toBe('zpcachyos');
/// ```
#[test]
fn a_machine_at_the_design_s_state_answers_every_fact_and_no_unknown() {
    let fixture: TheFixturesReaders = TheFixturesReaders::made("gather-healthy");
    let pool: AScriptedPool = AScriptedPool::a_healthy_machine();
    let facts: MachineRouteFacts =
        gathered(&fixture, &pool).expect("a machine at the design's state");
    assert!(facts.unread.is_empty(), "{:?}", facts.unread);
    assert_eq!(facts.route.device.path, fixture.places.device);
    assert_eq!(facts.route.pool.pool, "zpcachyos");
    assert_eq!(facts.route.datasets.len(), 3);
    assert_eq!(facts.route.datasets[2].name, CLONE_DATASET);
    assert_eq!(facts.route.mounts.rows.len(), 4);
    assert_eq!(facts.route.store.slots.len(), 1);
    assert_eq!(facts.route.slots.len(), 1);
    assert_eq!(facts.route.live_boot, LIVE_BOOT);
    assert_eq!(facts.route.requests.len(), 1);
    assert_eq!(facts.route.requests[0].clone, CLONE_DATASET);
    assert_eq!(facts.route.requests[0].intent_nonce, None);
    assert_eq!(facts.route.delegation.checkout_dataset, CHECKOUT_DATASET);
    assert_eq!(facts.route.delegation.container_dataset, CONTAINER_DATASET);
    let reports: Vec<CheckReport> = route_checks(&facts.route);
    let unknown: Vec<Check> = reports
        .iter()
        .filter(|report| return report.unknown())
        .map(|report| return report.check)
        .collect();
    // The three facts a later step owns answer unknown naming that step, the backup coverage's own line answers
    // unknown because its arm is not built, and the slot's walk refuses a store a fixture cannot put under root
    // control; the cancel command's line passes on the build's own grammar, and every other check answers from its own
    // fact.
    assert_eq!(
        unknown,
        vec![
            Check::SlotIdentity,
            Check::BootProtection,
            Check::ServiceUnit,
            Check::BackupCoverage,
            Check::Sanoid,
        ]
    );
}

/// What:
///  A mount table nobody could read answers the four checks that read a row,
///  and every other check still answers from its own facts.
/// Why:
///  One unreadable file must cost its own checks and no other,
///  which is what a partial read is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(gathered.unread.map((r) => r.check)).toEqual(['checkout-mount', 'checkout-propagation', 'nested-reachability', 'staging-bind-propagation']);
/// ```
#[test]
fn one_failed_read_answers_its_own_checks_and_no_other() {
    let fixture: TheFixturesReaders = TheFixturesReaders::made("gather-partial");
    std::fs::remove_file(fixture.places.mount_table.as_path()).expect("the table goes");
    let pool: AScriptedPool = AScriptedPool::a_healthy_machine();
    let facts: MachineRouteFacts =
        gathered(&fixture, &pool).expect("the rest of the machine still reads");
    assert_eq!(
        facts
            .unread
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
    assert_eq!(facts.route.datasets.len(), 3);
    assert_eq!(facts.route.store.slots.len(), 1);
}

/// What:
///  A layout the gather cannot name answers the refusal naming the file,
///  and no value at all.
/// Why:
///  Without the checkout's path and the two datasets no route fact can be
///  named,
///  and a report over invented names would send a person to the wrong datasets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theMachinesRouteFacts(reads)).toMatchObject({ why: expect.stringContaining('could not be named') });
/// ```
#[test]
fn a_layout_the_gather_cannot_name_answers_the_refusal() {
    let fixture: TheFixturesReaders = TheFixturesReaders::made("gather-no-layout");
    std::fs::remove_file(fixture.places.configuration.as_path()).expect("the configuration goes");
    let pool: AScriptedPool = AScriptedPool::a_healthy_machine();
    let refusal: MachineFactsRefusal = gathered(&fixture, &pool)
        .err()
        .expect("a layout nobody named is a refusal");
    assert!(
        refusal
            .why
            .contains(fixture.places.configuration.display().to_string().as_str()),
        "{}",
        refusal.why
    );
    assert!(
        refusal.why.contains("could not be named"),
        "{}",
        refusal.why
    );
}

/// What:
///  The gather leaves the clone group empty,
///  and the report therefore answers no clone's line from it.
/// Why:
///  Each managed clone's removal facts are the sibling round's,
///  so a value that filled the group with placeholders would print lines
///  nobody read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(gathered.route.clones).toEqual([]);
/// ```
#[test]
fn the_gather_leaves_the_clone_group_to_the_sibling_round() {
    let fixture: TheFixturesReaders = TheFixturesReaders::made("gather-clone-group");
    let pool: AScriptedPool = AScriptedPool::a_healthy_machine();
    let facts: MachineRouteFacts = gathered(&fixture, &pool).expect("the machine's facts");
    assert!(facts.route.clones.is_empty());
    let reports: Vec<CheckReport> = route_checks(&facts.route);
    assert!(
        !reports
            .iter()
            .any(|report| return report.check == Check::CloneState),
        "no clone's line is answered"
    );
}
