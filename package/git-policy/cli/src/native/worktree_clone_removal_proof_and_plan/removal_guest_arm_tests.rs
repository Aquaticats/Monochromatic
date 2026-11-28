//! What:
//!  The guest arm of plan step 14's piece:
//!  the healthy removal entering against a real pool,
//!  the promote and reverse-promote controls of VM item 11,
//!  the live and stale bookmark controls of VM item 13,
//!  and the same-name replacement and grant controls of VM item 12.
//! Why:
//!  The proof reads `/dev/zfs`,
//!  so no control on a person's machine may run it against a pool it did not build,
//!  and item 11's loss is only visible on a real pool:
//!  a promoted clone's transferred snapshot is history the checkout still depends on
//!  (`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
//! items 11,
//! 12 and 13).
//!  The arm runs inside the throwaway `mvm` guest the package's e2e recipe drives,
//!  and a run on this machine prints one `SKIPPED` line,
//!  which the recipe reads as a failure when it meant the arm to run.
//!  The unmount is a scripted answer here:
//!  step 9's live path needs the root handler the guest fixture does not start,
//!  and this arm judges the proof and the plan,
//! never the mount work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the removal lineage guest arm', () => { /* zpool, checkout, clone, promote, bookmarks */ });
//! ```

use crate::worktree_clone_binding_record::binding_inventory::InventoryRecord;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::bookmark_rule::{
    explanations_of,
    refuse_the_unexplained_bookmarks,
};
use crate::worktree_clone_removal_proof_and_plan::lineage_proof::{
    ProvenRemoval,
    prove_the_lineage,
};
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_the_plan;
use crate::worktree_clone_removal_proof_and_plan::removal_clock::TheLiveClock;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::AStagingFolderAControlAnswers;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{
    EnteredRemoval,
    RemovalEntry,
    enter_the_removal,
};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_proof_and_plan::removal_live_fixtures::{
    ACCOUNT,
    LivePoolFixture,
    PoolVariables,
    SLOT,
};
use crate::worktree_clone_removal_proof_and_plan::removal_reads::LiveRemovalReads;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{
    RemovalUnmount,
    UnmountAnswer,
};
use crate::worktree_removal_deferred_origin_check::removal_history::RemovalHistory;
use std::path::PathBuf;

/// The variable naming the guest this arm may run in.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_LINEAGE_GUEST";
/// The variable naming the file-backed pool the arm builds.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_LINEAGE_POOL";
/// The variable naming the OpenZFS 2.4.4 prefix the guest carries.
const PREFIX_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_LINEAGE_PREFIX";
/// The variable naming the folder the pool's image and the fixture repository live under.
const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_LINEAGE_ROOT";

/// The guest directory the fixture lives in when the recipe names none.
const DEFAULT_ROOT: &str = "/home/zfs-step-14-fixture";
/// The pool the arm builds when the recipe names none.
const DEFAULT_POOL: &str = "zfsstep14";
/// The clone's own snapshot.
const OWN_SNAPSHOT: &str = "worktree-own";

/// The four variables and the two defaults this arm's fixture takes.
fn variables() -> PoolVariables {
    return PoolVariables {
        guest: GUEST_VARIABLE,
        pool: POOL_VARIABLE,
        prefix: PREFIX_VARIABLE,
        root: ROOT_VARIABLE,
        default_root: DEFAULT_ROOT,
        default_pool: DEFAULT_POOL,
    };
}

/// What:
///  The unmount answer the guest gives:
///  the intent's own nonce and the live clone's GUID,
///  so the arm judges the proof and the plan and never the mount work.
/// Why:
///  Step 9's live unmount needs the root handler the guest fixture does not start,
///  and a control that skipped the unmount would not exercise the entry's order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unmount = { requestTheUnmount: (intent) => ({ ...done, nonce: intent.nonce }) };
/// ```
struct ADoneUnmount {
    clone_guid: DatasetGuid,
}

impl RemovalUnmount for ADoneUnmount {
    fn request_the_unmount(
        &self,
        intent: &MountIntent,
    ) -> Result<UnmountAnswer, RemovalStop> {
        return Ok(UnmountAnswer {
            dataset: self.clone_guid,
            operation: intent.operation,
            nonce: intent.nonce.clone(),
            outcome: ReceiptOutcome::Done,
            mount_id: None,
            failed_step: None,
            errno: None,
        });
    }
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_removal_lineage_guest_arms() {
    let Some(fixture) =
        LivePoolFixture::from_environment("the_removal_lineage_guest_arms", &variables())
    else {
        return;
    };
    println!("REMOVAL guest={} pool={}", fixture.guest, fixture.pool);
    fixture.build(&[OWN_SNAPSHOT]);
    let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");

    // Arm 1: the healthy removal enters, writes its plan and prints its one own snapshot.
    let recorded: RecordedBinding = fixture.recorded();
    fixture.write_the_adds_records(&recorded);
    let unmount: ADoneUnmount = ADoneUnmount {
        clone_guid: recorded.clone_guid,
    };
    let common: PathBuf = fixture.common();
    let entry: RemovalEntry<'_> = RemovalEntry {
        common_dir: common.as_path(),
        slot: SLOT,
    };
    let entered: EnteredRemoval = enter_the_removal(&entry, &reads, &unmount, &TheLiveClock, &AStagingFolderAControlAnswers::making_nothing())
        .expect("the healthy removal enters");
    assert_eq!(
        entered.plan.attempts.len(),
        1,
        "the clone's one own snapshot is the plan's one tuple"
    );
    assert_eq!(entered.plan.facts.clone_guid, recorded.clone_guid);
    let read_back =
        read_the_plan(common.as_path(), recorded.clone_guid).expect("the plan reads back");
    assert_eq!(read_back, entered.plan);
    assert!(
        entered
            .lost
            .text()
            .contains(&format!("{}@{OWN_SNAPSHOT}", fixture.clone()))
    );
    println!(
        "REMOVAL healthy=entered snapshots={} bookmarks={}",
        entered.plan.attempts.len(),
        entered.lost.bookmarks.len()
    );

    // Arm 2: promotion moves the checkout's older history under the clone, and the proof refuses.
    let before: Vec<String> = fixture.protected_guids();
    // The fixture makes three snapshots and one bookmark, and a bookmark shares its source snapshot's tuple:
    // the kind is what keeps the four objects four entries here.
    assert_eq!(
        before.len(),
        4,
        "the arm read the fixture's three snapshots and one bookmark: {before:?}"
    );
    fixture.run("zfs", &["promote", fixture.clone().as_str()]);
    let origin_inventory: InventoryRecord = fixture.the_origin_inventory(&recorded);
    let refused: RemovalStop = prove_the_lineage(
        &reads,
        &recorded,
        &origin_inventory,
        &RemovalHistory::NothingDestroyed,
    )
    .expect_err("a promoted clone's lineage is not the recorded one");
    let after: Vec<String> = fixture.protected_guids();
    assert_eq!(
        before, after,
        "promotion moved tuples between the datasets without losing one, and the refusal destroyed nothing"
    );
    println!(
        "REMOVAL promoted=refused cause={:?} protected=survived",
        refusal_name(&refused)
    );

    // Reverse promotion restores the map, and the recorded tuple proves again.
    fixture.run("zfs", &["promote", fixture.checkout().as_str()]);
    let proven: ProvenRemoval = prove_the_lineage(
        &reads,
        &recorded,
        &fixture.the_origin_inventory(&recorded),
        &RemovalHistory::NothingDestroyed,
    )
    .expect("the reverse promotion restores the recorded lineage");
    assert_eq!(
        proven.own_snapshots.len(),
        1,
        "the clone owns exactly its own snapshot again"
    );
    println!(
        "REMOVAL reverse-promotion=proved origin={}",
        fixture.origin()
    );

    // Arm 3: a bookmark a live snapshot explains passes, and one whose snapshot went refuses.
    fixture.run(
        "zfs",
        &[
            "bookmark",
            format!("{}@{OWN_SNAPSHOT}", fixture.clone()).as_str(),
            format!("{}#from-own", fixture.clone()).as_str(),
        ],
    );
    let live: ProvenRemoval = prove_the_lineage(
        &reads,
        &recorded,
        &fixture.the_origin_inventory(&recorded),
        &RemovalHistory::NothingDestroyed,
    )
    .expect("a bookmark of an own snapshot keeps the lineage proved");
    let explanations = explanations_of(&live.inventory, None);
    let refused_live = refuse_the_unexplained_bookmarks(
        &explanations,
        live.inventory.bookmarks().as_slice(),
        &fixture.the_origin_inventory(&recorded),
    );
    assert!(
        refused_live.is_empty(),
        "a bookmark its own live snapshot explains passes"
    );
    println!("REMOVAL bookmark-live=passed");
    fixture.run(
        "zfs",
        &["snapshot", format!("{}@gone", fixture.clone()).as_str()],
    );
    fixture.run(
        "zfs",
        &[
            "bookmark",
            format!("{}@gone", fixture.clone()).as_str(),
            format!("{}#from-gone", fixture.clone()).as_str(),
        ],
    );
    fixture.run(
        "zfs",
        &["destroy", format!("{}@gone", fixture.clone()).as_str()],
    );
    assert!(
        fixture.exists(format!("{}#from-gone", fixture.clone()).as_str()),
        "the kernel keeps a bookmark whose snapshot it destroyed"
    );
    let after_the_destroy: ProvenRemoval = prove_the_lineage(
        &reads,
        &recorded,
        &fixture.the_origin_inventory(&recorded),
        &RemovalHistory::NothingDestroyed,
    )
    .expect("the clone's own history still proves");
    let stale = explanations_of(&after_the_destroy.inventory, None);
    let refused_stale = refuse_the_unexplained_bookmarks(
        &stale,
        after_the_destroy.inventory.bookmarks().as_slice(),
        &fixture.the_origin_inventory(&recorded),
    );
    assert_eq!(
        refused_stale.len(),
        1,
        "one bookmark's snapshot is gone and nothing explains it"
    );
    assert_eq!(
        refused_stale[0].name,
        format!("{}#from-gone", fixture.clone())
    );
    println!(
        "REMOVAL bookmark-stale=refused name={}",
        refused_stale[0].name
    );

    // Arm 4: the item-12 grant set carries no lineage-changing grant, and a same-name replacement refuses.
    fixture.run(
        "zfs",
        &[
            "allow",
            "-d",
            "-u",
            ACCOUNT,
            "mount,destroy,userprop",
            fixture.named("clones").as_str(),
        ],
    );
    fixture.run(
        "zfs",
        &[
            "allow",
            "-u",
            ACCOUNT,
            "create,mount",
            fixture.named("clones").as_str(),
        ],
    );
    fixture.run(
        "zfs",
        &[
            "allow",
            "-d",
            "-u",
            ACCOUNT,
            "clone",
            fixture.checkout().as_str(),
        ],
    );
    let grants: String = fixture.ask("zfs", &["allow", fixture.named("clones").as_str()]);
    for forbidden in ["promote", "rename", "receive"] {
        assert!(
            !grants.contains(forbidden),
            "the decided grant set carries no `{forbidden}`, and it reads: {grants}"
        );
    }
    println!("REMOVAL grants=lineage-changing-absent");
    fixture.run(
        "zfs",
        &[
            "destroy",
            format!("{}@{OWN_SNAPSHOT}", fixture.clone()).as_str(),
        ],
    );
    fixture.run("zfs", &["destroy", fixture.clone().as_str()]);
    fixture.run(
        "zfs",
        &["clone", fixture.origin().as_str(), fixture.clone().as_str()],
    );
    match prove_the_lineage(
        &reads,
        &recorded,
        &fixture.the_origin_inventory(&recorded),
        &RemovalHistory::NothingDestroyed,
    ) {
        Err(RemovalStop::IdentityDiffers {
            field,
            ..
        }) => {
            assert_eq!(
                field, "cloneGuid",
                "a replacement clone answers another GUID"
            );
            println!("REMOVAL replacement=refused field={field}");
        }
        Err(other) => panic!("a replacement clone refuses on its GUID, not {other:?}"),
        Ok(_) => panic!("a replacement clone must not prove the recorded lineage"),
    }

    fixture.teardown();
    println!("REMOVAL done");
}

/// The short name of one refusal,
/// for the log line the recipe greps.
fn refusal_name(stop: &RemovalStop) -> &'static str {
    return match stop {
        RemovalStop::TransferredHistory {
            ..
        } => "transferred-history",
        RemovalStop::OriginIsNotTheRecordedOne {
            ..
        } => "origin-is-not-the-recorded-one",
        RemovalStop::IdentityDiffers {
            ..
        } => "identity-differs",
        RemovalStop::OriginSideHit {
            ..
        } => "origin-side-hit",
        RemovalStop::OriginCheckRefused {
            ..
        } => "origin-marked",
        _ => "another-refusal",
    };
}
