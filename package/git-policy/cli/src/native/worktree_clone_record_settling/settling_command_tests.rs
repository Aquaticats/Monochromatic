//! Controls for the whole command:
//!  a proved absence removes every record and the registration,
//!  an unprovable absence does the same with its refusal printed,
//!  a clone that stands and a plan whose head is gone refuse and change nothing,
//!  a checkout leaves the registration and the slot,
//!  a plan's lost names are printed before it goes,
//!  a live claim refuses naming its process,
//!  and a missing clone GUID names the add again.

use crate::testing::mounted_dataset_table::ScriptedMountTable;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_record_settling::settling_command::{
    SettlingDone,
    SettlingReads,
    settle_the_records_of_a_slot,
};
use crate::worktree_clone_record_settling::settling_fixtures::{
    CLONE_GUID,
    ScriptedProbe,
    TheFixture,
    a_guid,
    an_identity,
    binding_path,
    publish_the_record,
    the_fixture,
    write_the_pointer,
};
use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_record_settling::settling_request::SettlingRequest;
use crate::worktree_clone_record_settling::settling_steps::SettlingStep;
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

/// The clone's dataset name the fixture record carries.
const CLONE: &str = "tank/monochromatic-git-worktree-clone-datasets/topic";

/// The design's own line,
/// as the person types it.
fn ask() -> SettlingRequest {
    return SettlingRequest {
        slot: String::from("topic"),
        confirmed_slot: String::from("topic"),
        digits: String::from("3102"),
    };
}

/// The two live reads one run takes.
struct TheReads {
    /// The scripted pool.
    probe: ScriptedProbe,
    /// The scripted mount table.
    table: ScriptedMountTable,
}

impl TheReads {
    /// The two sources one run is handed.
    fn borrowed(&self) -> SettlingReads<'_> {
        return SettlingReads {
            probe: &self.probe,
            mounts: &self.table,
        };
    }
}

/// A record published with this add's pointer,
/// and the two reads a control scripts.
fn ready(name: &str) -> (TheFixture, TheReads) {
    let made: TheFixture = the_fixture(name);
    publish_the_record(&made, Phase::Filled);
    write_the_pointer(&made);
    let reads: TheReads = TheReads {
        probe: ScriptedProbe::empty(),
        table: ScriptedMountTable::holding(&[]),
    };
    return (made, reads);
}

/// The plan's own facts for one fixture's record,
/// with one snapshot recorded as destroyed.
fn publish_the_plan(
    made: &TheFixture,
    head: crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord,
) -> String {
    use crate::worktree_clone_record_settling::settling_record::read_the_record;
    use crate::worktree_clone_removal_proof_and_plan::fixtures::{
        guid,
        nonce,
    };
    use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
    use crate::worktree_clone_removal_proof_and_plan::plan_record::{
        DestructionPlan,
        PlanFacts,
        PlanSnapshot,
    };
    use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
    let record = read_the_record(made.common.as_path(), "topic").expect("the record reads");
    let facts: PlanFacts = PlanFacts {
        run: nonce("00112233445566778899aabbccddeeff"),
        owner: String::from("48213:linux:9481733"),
        started: String::from("2026-11-25T09:12:44-05:00"),
        clone_dataset: record.recorded.clone_dataset.clone(),
        clone_guid: record.recorded.clone_guid,
        clone_createtxg: record.recorded.clone_createtxg,
        origin_dataset: record.recorded.origin_dataset.clone(),
        origin_dataset_guid: record.recorded.origin_dataset_guid,
        origin_snapshot: record.recorded.origin_snapshot.clone(),
        origin_snapshot_guid: record.recorded.origin_snapshot_guid,
        origin_snapshot_createtxg: record.recorded.origin_snapshot_createtxg,
        registration_id: record.recorded.registration_id.clone(),
        admin: String::from("2049:131081:1763750412:987654321"),
        target: record.recorded.slot_path.clone(),
        unmount_nonce: nonce("0a1b2c3d4e5f60718293a4b5c6d7e8f9"),
        unmount_mount_id: None,
    };
    let lost: &str = "tank/monochromatic-git-worktree-clone-datasets/topic@worktree-own";
    let mut plan: DestructionPlan = DestructionPlan::first_write(
        facts,
        &[PlanSnapshot {
            name: String::from(lost),
            guid: guid(17_157_080_670_377_245_572),
            createtxg: 18,
        }],
    );
    plan.attempts[0].outcome = SnapshotOutcome::Destroyed;
    plan.head = head;
    let folder: PlanFolder =
        PlanFolder::open_or_create(made.common.as_path()).expect("the plan folder");
    folder.publish(&plan).expect("the plan writes");
    return String::from(lost);
}

/// A proved absence removes the registration,
/// its lock and the three records,
/// and the claim is released.
#[test]
fn a_proved_absence_settles_the_slot() {
    let (made, reads) = ready("settling-command-proved");
    let done: SettlingDone =
        settle_the_records_of_a_slot(made.common.as_path(), &ask(), &reads.borrowed())
            .expect("the slot settles");
    assert_eq!(
        done.steps,
        vec![
            SettlingStep::TheLostLock,
            SettlingStep::TheRegistrationRecipe,
            SettlingStep::TheOriginInventory,
            SettlingStep::TheCloneInventory,
            SettlingStep::TheBinding,
            SettlingStep::TheClaim
        ]
    );
    assert!(
        done.lock_written,
        "the fixture registration had no lock, so the sentence was written again"
    );
    assert!(!done.checkout_begun);
    assert!(!binding_path(&made).exists(), "the binding went last");
    assert!(!made.registration.exists(), "the registration went");
    assert!(!made.slot_path.join(".git").exists(), "the pointer went");
    let last: &String = done.lines.last().expect("a last line");
    assert!(last.contains("can no longer prove which clone"), "{last}");
    assert!(
        done.lines
            .iter()
            .any(|line| return line.contains("the clone's absence is proved")),
        "{:?}",
        done.lines
    );
    let claims: Vec<String> = std::fs::read_dir(
        made.common
            .join("monochromatic-git-policy-cli")
            .join("worktree-clones")
            .join("v1"),
    )
    .expect("the home lists")
    .map(|entry| {
        return entry
            .expect("an entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
    })
    .collect();
    assert!(
        claims
            .iter()
            .any(|name| return name.starts_with("topic.claim.")),
        "the released claim stands: {claims:?}"
    );
}

/// A refused listing is the state the command exists for:
/// the records go and the refusal is printed as evidence.
#[test]
fn an_unprovable_absence_settles_and_prints_the_refusal() {
    let (made, _unused) = ready("settling-command-unprovable");
    let reads: TheReads = TheReads {
        probe: ScriptedProbe::empty().refusing_the_listing("the pool answered EIO"),
        table: ScriptedMountTable::holding(&[]),
    };
    let done: SettlingDone =
        settle_the_records_of_a_slot(made.common.as_path(), &ask(), &reads.borrowed())
            .expect("the slot settles");
    assert!(!binding_path(&made).exists());
    assert!(
        done.lines.iter().any(|line| return line
            .contains("could not prove the recorded clone's absence")
            && line.contains("the pool answered EIO")),
        "{:?}",
        done.lines
    );
}

/// A clone that stands refuses,
/// and every record and the registration stay.
#[test]
fn a_clone_that_stands_changes_nothing() {
    let (made, _unused) = ready("settling-command-stands");
    let probe: ScriptedProbe =
        ScriptedProbe::answering(vec![(CLONE, an_identity(CLONE_GUID, 121_294))]);
    let reads: TheReads = TheReads {
        probe,
        table: ScriptedMountTable::holding(&[]),
    };
    let refused = settle_the_records_of_a_slot(made.common.as_path(), &ask(), &reads.borrowed())
        .expect_err("a standing clone refuses");
    assert!(
        matches!(refused, SettlingStop::TheCloneStillStands { .. }),
        "{refused:?}"
    );
    assert!(binding_path(&made).exists(), "the binding stays");
    assert!(made.registration.exists(), "the registration stays");
    assert!(made.slot_path.join(".git").exists(), "the pointer stays");
}

/// A checkout begun leaves the registration and the pointer,
/// and removes only the records.
#[test]
fn a_checkout_leaves_the_registration_and_the_slot() {
    let (made, reads) = ready("settling-command-checkout");
    std::fs::write(made.slot_path.join("README.md"), b"# the person's file\n")
        .expect("the file writes");
    let done: SettlingDone =
        settle_the_records_of_a_slot(made.common.as_path(), &ask(), &reads.borrowed())
            .expect("the slot settles");
    assert!(done.checkout_begun);
    assert_eq!(
        done.steps,
        vec![
            SettlingStep::TheOriginInventory,
            SettlingStep::TheCloneInventory,
            SettlingStep::TheBinding,
            SettlingStep::TheClaim
        ]
    );
    assert!(!binding_path(&made).exists(), "the records go");
    assert!(made.registration.exists(), "the registration stays");
    assert!(made.slot_path.join(".git").exists(), "the pointer stays");
    assert!(
        done.lines
            .iter()
            .any(|line| return line.contains("git worktree remove topic")),
        "{:?}",
        done.lines
    );
}

/// A standing plan's lost names are printed and the plan goes with the records.
#[test]
fn a_standing_plans_lost_names_are_printed_before_it_goes() {
    use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
    let (made, reads) = ready("settling-command-plan");
    let lost: String = publish_the_plan(&made, HeadWord::Present);
    let done: SettlingDone =
        settle_the_records_of_a_slot(made.common.as_path(), &ask(), &reads.borrowed())
            .expect("the slot settles");
    assert_eq!(
        done.files.first().map(String::as_str),
        Some(a_guid(CLONE_GUID).file_name().as_str()),
        "the plan goes first: {:?}",
        done.files
    );
    assert!(
        done.lines
            .iter()
            .any(|line| return line.contains(lost.as_str())),
        "{:?}",
        done.lines
    );
    assert!(
        !crate::worktree_clone_binding_record::registration_home::plans_folder(
            made.common.as_path()
        )
        .exists(),
        "the plan folder goes with its last plan"
    );
}

/// A plan whose head is gone refuses,
/// and nothing of the slot is changed.
#[test]
fn a_plan_whose_head_is_gone_changes_nothing() {
    use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
    let (made, reads) = ready("settling-command-plan-gone");
    publish_the_plan(&made, HeadWord::Gone);
    let refused = settle_the_records_of_a_slot(made.common.as_path(), &ask(), &reads.borrowed())
        .expect_err("a gone head refuses");
    assert!(
        matches!(refused, SettlingStop::ThePlanIsTheFinishPasss { .. }),
        "{refused:?}"
    );
    assert!(binding_path(&made).exists());
    assert!(made.registration.exists());
}

/// A claim another live command holds refuses,
/// naming its process.
#[test]
fn a_live_claim_refuses_naming_its_process() {
    let (made, reads) = ready("settling-command-claim");
    let home: std::path::PathBuf =
        crate::worktree_clone_binding_record::registration_home::home_folder(made.common.as_path());
    let _held = crate::lock::single_owner_claim::take_claim(
        home.as_path(),
        crate::worktree_clone_binding_record::registration_home::claim_prefix("topic").as_os_str(),
    )
    .expect("the claim is taken");
    let refused = settle_the_records_of_a_slot(made.common.as_path(), &ask(), &reads.borrowed())
        .expect_err("a live claim refuses");
    let SettlingStop::TheClaimIsHeld {
        slot,
        pid,
    } = &refused
    else {
        panic!("the live claim is its own arm, and this answered {refused:?}")
    };
    assert_eq!(slot, "topic");
    assert_eq!(
        *pid,
        i64::from(std::process::id()),
        "the claim names this process"
    );
    assert!(binding_path(&made).exists(), "nothing was removed");
}

/// A pointer naming another registration refuses,
/// and every record stays.
#[test]
fn a_pointer_naming_another_registration_changes_nothing() {
    let (made, reads) = ready("settling-command-foreign-pointer");
    std::fs::write(
        made.slot_path.join(".git"),
        b"gitdir: /elsewhere/.git/worktrees/other\n",
    )
    .expect("the pointer writes");
    let refused = settle_the_records_of_a_slot(made.common.as_path(), &ask(), &reads.borrowed())
        .expect_err("a foreign pointer refuses");
    assert!(
        matches!(refused, SettlingStop::ThePointerNamesAnother { .. }),
        "{refused:?}"
    );
    assert!(binding_path(&made).exists(), "nothing was removed");
    assert!(made.registration.exists(), "the registration stands");
}

/// A registration of another identity refuses,
/// and every record stays.
#[test]
fn a_registration_of_another_identity_changes_nothing() {
    let made: TheFixture = the_fixture("settling-command-foreign-registration");
    let mut record =
        crate::worktree_clone_record_settling::settling_fixtures::the_record(&made, Phase::Filled);
    record.registration_identity = Some(
        crate::worktree_clone_binding_record::binding_encoding::FolderIdentity::decode(
            "131074:1763750400:123456789",
        )
        .expect("a decodable identity"),
    );
    let home: std::fs::File =
        crate::worktree_clone_binding_record::registration_home::open_or_create_home(
            made.common.as_path(),
        )
        .expect("the home opens");
    crate::worktree_clone_binding_record::record_publication::write_binding(
        &home, "topic", &record,
    )
    .expect("the record writes");
    write_the_pointer(&made);
    let reads: TheReads = TheReads {
        probe: ScriptedProbe::empty(),
        table: ScriptedMountTable::holding(&[]),
    };
    let refused = settle_the_records_of_a_slot(made.common.as_path(), &ask(), &reads.borrowed())
        .expect_err("another identity refuses");
    assert!(
        matches!(refused, SettlingStop::TheRegistrationIsAnother { .. }),
        "{refused:?}"
    );
    assert!(binding_path(&made).exists(), "nothing was removed");
    assert!(made.registration.exists(), "the registration stands");
}

/// A record that holds no clone GUID names the add again and removes nothing.
#[test]
fn a_record_without_a_clone_names_the_add_again() {
    let made: TheFixture = the_fixture("settling-command-no-clone");
    let mut record = crate::worktree_clone_record_settling::settling_fixtures::the_record(
        &made,
        Phase::Snapshotted,
    );
    record.clone_guid = None;
    record.clone_createtxg = None;
    let home: std::fs::File =
        crate::worktree_clone_binding_record::registration_home::open_or_create_home(
            made.common.as_path(),
        )
        .expect("the home opens");
    crate::worktree_clone_binding_record::record_publication::write_binding(
        &home, "topic", &record,
    )
    .expect("the record writes");
    let reads: TheReads = TheReads {
        probe: ScriptedProbe::empty(),
        table: ScriptedMountTable::holding(&[]),
    };
    let refused = settle_the_records_of_a_slot(made.common.as_path(), &ask(), &reads.borrowed())
        .expect_err("no clone refuses");
    assert!(
        matches!(refused, SettlingStop::NoCloneRecorded { .. }),
        "{refused:?}"
    );
    assert!(binding_path(&made).exists());
}

/// The pool is asked about the clone and the parent only through the scripted probe,
/// so no control opens this machine's device.
#[test]
fn the_pool_is_asked_about_the_clone_and_the_parent() {
    let (made, _unused) = ready("settling-command-pool-reads");
    let probe: ScriptedProbe = ScriptedProbe::empty().listing_children(&[]);
    let reads: TheReads = TheReads {
        probe,
        table: ScriptedMountTable::holding(&[]),
    };
    settle_the_records_of_a_slot(made.common.as_path(), &ask(), &reads.borrowed())
        .expect("the slot settles");
    let asked: Vec<String> = reads.probe.reads.borrow().clone();
    assert_eq!(asked.first().map(String::as_str), Some(CLONE));
    let _ = NvList::new();
}
