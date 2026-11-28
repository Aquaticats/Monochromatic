//! What:
//!  The head destroy's controls:
//!  the names settled by their GUIDs,
//!  the amended plan a new own snapshot takes,
//!  the `requested` line before the request,
//!  and the three words its proof ends on.
//! Why:
//!  A snapshot ends only when no live object carries its recorded GUID,
//!  and the head's word is decided by the absence proof and never by the request's answer,
//!  so each control plants one of those states and reads the plan's own file back.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const head = destroyTheHead({ plan, folder, probe, device });
//! expect(head.head).toBe('gone');
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
    ReadFailure,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_the_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    HeadWord,
    SnapshotOutcome,
};
use crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::AStagingFolderAControlAnswers;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::{
    ADoneUnmount,
    EpochProbe,
    ScriptedClock,
    ScriptedHead,
};
use crate::worktree_clone_removal_proof_and_plan::removal_entry::RemovalEntry;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE,
    CLONE_CREATETXG,
    CLONE_GUID,
    OWN_CREATETXG,
    OWN_GUID,
    OWN_SNAPSHOT,
    SANOID_CREATETXG,
    SANOID_GUID,
    SANOID_SNAPSHOT,
    ScriptedPool,
};
use crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::{
    CloneAbsence,
    HeadDestroyRequest,
    destroy_the_head,
    the_clone_absence,
};
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject;
use crate::worktree_clone_removal_proof_and_plan::removal_snapshot_settlement::{
    amend_the_plan,
    settle_the_recorded_names,
    the_new_own_snapshots,
};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_removal_deferred_origin_check::fixtures::Pool;
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use std::cell::{
    Cell,
    RefCell,
};
use std::rc::Rc;

/// The one live snapshot of the fixture,
/// as the absence rule reads it.
fn a_live(
    name: &str,
    guid: u64,
    createtxg: u64,
) -> LiveObject {
    return LiveObject {
        kind: EntryKind::Snapshot,
        name: String::from(name),
        guid: DatasetGuid::parse(guid.to_string().as_str()).expect("a number is a canonical GUID"),
        createtxg,
        owning_dataset: String::from(CLONE),
        owning_dataset_guid: DatasetGuid::parse(CLONE_GUID.to_string().as_str())
            .expect("the fixture's GUID is canonical"),
    };
}

/// What:
///  The plan one removal's entry wrote,
///  with its folder.
/// Why:
///  The head destroy rewrites the plan's own file,
///  so a control starts from the entry's first write rather than from a hand-built value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [folder, plan] = anEntered(fixture, pool);
/// ```
fn an_entered(
    fixture: &crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture,
    pool: &ScriptedPool,
) -> (PlanFolder, DestructionPlan) {
    let recorded = fixture.recorded();
    let unmount: ADoneUnmount = ADoneUnmount {
        clone_guid: recorded.clone_guid,
    };
    let at: RemovalEntry<'_> = RemovalEntry {
        common_dir: fixture.common(),
        slot: recorded.slot.as_str(),
    };
    let entered = crate::worktree_clone_removal_proof_and_plan::removal_entry::enter_the_removal(
        &at,
        pool,
        &unmount,
        &ScriptedClock,
        &AStagingFolderAControlAnswers::making_nothing(),
    )
    .expect("the entry runs over the healthy pool");
    let folder: PlanFolder =
        PlanFolder::open_or_create(fixture.common()).expect("the plan folder opens");
    return (folder, entered.plan);
}

#[test]
fn an_absent_guid_is_destroyed_and_a_live_one_is_intact() {
    let fixture =
        crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture::healthy(
            "head-settling",
        );
    let pool: ScriptedPool = fixture.healthy_pool();
    let (_folder, mut plan) = an_entered(&fixture, &pool);
    let live: Vec<LiveObject> = vec![a_live(OWN_SNAPSHOT, OWN_GUID, OWN_CREATETXG)];
    let settled: Vec<String> =
        settle_the_recorded_names(&mut plan, live.as_slice()).expect("the live set settles");
    assert_eq!(settled.len(), 1, "one name is proved gone");
    let own: String = format!("{CLONE}@{OWN_SNAPSHOT}");
    let sanoid: String = format!("{CLONE}@{SANOID_SNAPSHOT}");
    assert_eq!(
        plan.current_state(own.as_str())
            .map(|held| return held.outcome),
        Some(SnapshotOutcome::Intact),
        "a live GUID with its recorded group is intact"
    );
    assert_eq!(
        plan.current_state(sanoid.as_str())
            .map(|held| return held.outcome),
        Some(SnapshotOutcome::Destroyed),
        "an absent GUID is destroyed"
    );
    assert_eq!(settled, vec![sanoid]);
}

#[test]
fn a_live_guid_in_another_group_refuses_and_writes_nothing_for_that_name() {
    let fixture =
        crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture::healthy(
            "head-other-group",
        );
    let pool: ScriptedPool = fixture.healthy_pool();
    let (_folder, mut plan) = an_entered(&fixture, &pool);
    let live: Vec<LiveObject> = vec![
        a_live(OWN_SNAPSHOT, OWN_GUID, OWN_CREATETXG + 1),
        a_live(SANOID_SNAPSHOT, SANOID_GUID, SANOID_CREATETXG),
    ];
    let stop: RemovalStop = settle_the_recorded_names(&mut plan, live.as_slice())
        .expect_err("another group is another object");
    match &stop {
        RemovalStop::ASnapshotIsNotTheRecordedOne {
            recorded_txg,
            live_txg,
            ..
        } => {
            assert_eq!(*recorded_txg, OWN_CREATETXG);
            assert_eq!(*live_txg, OWN_CREATETXG + 1);
        }
        other => panic!("the group's own refusal, not {other:?}"),
    }
    assert!(
        stop.message().contains("no wrapper command settles this"),
        "the message leaves the decision to a person"
    );
}

#[test]
fn a_live_guid_under_another_name_refuses() {
    let fixture =
        crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture::healthy(
            "head-other-name",
        );
    let pool: ScriptedPool = fixture.healthy_pool();
    let (_folder, mut plan) = an_entered(&fixture, &pool);
    let live: Vec<LiveObject> = vec![a_live("renamed", OWN_GUID, OWN_CREATETXG)];
    let stop: RemovalStop = settle_the_recorded_names(&mut plan, live.as_slice())
        .expect_err("a renamed snapshot is not settled by its name");
    assert!(matches!(
        stop,
        RemovalStop::ASnapshotIsNotTheRecordedOne { .. }
    ));
}

#[test]
fn a_new_own_snapshot_is_added_and_history_at_or_below_the_clone_refuses() {
    let fixture =
        crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture::healthy(
            "head-amended",
        );
    let pool: ScriptedPool = fixture.healthy_pool();
    let (_folder, mut plan) = an_entered(&fixture, &pool);
    let new_one: LiveObject = a_live("sanoid-minute", SANOID_GUID + 1, CLONE_CREATETXG + 6);
    let added = the_new_own_snapshots(
        &plan,
        &[
            a_live(OWN_SNAPSHOT, OWN_GUID, OWN_CREATETXG),
            new_one.clone(),
        ],
    )
    .expect("a newer snapshot is the clone's own");
    assert_eq!(added.len(), 1, "only the unnamed snapshot is added");
    amend_the_plan(&mut plan, added.as_slice()).expect("the plan takes the new line");
    assert_eq!(
        plan.current_state(new_one.full_name().as_str())
            .map(|held| return held.outcome),
        Some(SnapshotOutcome::Planned),
        "the amendment records it as `planned` at attempt 0"
    );
    assert_eq!(plan.attempts.len(), 3, "every earlier line is kept");
    let transferred: LiveObject = a_live("inherited", SANOID_GUID + 2, CLONE_CREATETXG);
    let stop: RemovalStop = the_new_own_snapshots(&plan, &[transferred])
        .expect_err("history at the clone's own group is transferred");
    assert!(matches!(stop, RemovalStop::TransferredHistory { .. }));
}

#[test]
fn the_head_request_is_written_before_the_request_and_its_proof_settles_every_name() {
    let fixture =
        crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture::healthy(
            "head-gone",
        );
    let pool: ScriptedPool = fixture.healthy_pool();
    let (folder, mut plan) = an_entered(&fixture, &pool);
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: Pool::new(Vec::new()),
        hidden: Rc::clone(&hidden),
    };
    let head: ScriptedHead = ScriptedHead::succeeding(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    let done = destroy_the_head(&mut HeadDestroyRequest {
        plan: &mut plan,
        folder: &folder,
        probe: &probe,
        device: &head,
    })
    .expect("the head destroy finishes");
    assert_eq!(
        head.sent.borrow().clone(),
        vec![String::from(CLONE)],
        "the clone's own name is the one request"
    );
    assert_eq!(
        head.words.borrow().first().map(|word| return word.as_str()),
        Some("head=requested"),
        "the plan said `requested` before the request arrived"
    );
    assert_eq!(done.head, HeadWord::Gone);
    assert_eq!(
        done.settled.len(),
        2,
        "the head's proof covers every snapshot it held"
    );
    let filed: DestructionPlan = read_the_plan(fixture.common(), fixture.recorded().clone_guid)
        .expect("the plan reads back");
    assert_eq!(filed.head, HeadWord::Gone, "the file says the head is gone");
    assert!(
        filed.named_snapshots().iter().all(|snapshot| return filed
            .current_state(snapshot.name.as_str())
            .map(|held| return held.outcome == SnapshotOutcome::Destroyed)
            .unwrap_or(false)),
        "every name is destroyed by that proof"
    );
}

#[test]
fn a_refused_head_leaves_the_word_refused_and_no_name_settled() {
    let fixture =
        crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture::healthy(
            "head-refused",
        );
    let pool: ScriptedPool = fixture.healthy_pool();
    let (folder, mut plan) = an_entered(&fixture, &pool);
    let probe: StubbornProbe = StubbornProbe {
        clone_guid: fixture.recorded().clone_guid,
    };
    let head: RefusingHead = RefusingHead {
        refuses: RefCell::new(0),
    };
    let done = destroy_the_head(&mut HeadDestroyRequest {
        plan: &mut plan,
        folder: &folder,
        probe: &probe,
        device: &head,
    })
    .expect("a refusal is an outcome, not a stop");
    assert_eq!(
        done.head,
        HeadWord::Refused,
        "a clone that still answers is a refused head"
    );
    assert_eq!(
        done.errno,
        u32::try_from(libc::EBUSY).ok(),
        "the kernel's errno travels with the word"
    );
    assert!(
        done.settled.is_empty(),
        "no name is settled by a refused head"
    );
    let filed: DestructionPlan = read_the_plan(fixture.common(), fixture.recorded().clone_guid)
        .expect("the plan reads back");
    assert_eq!(filed.head, HeadWord::Refused);
}

#[test]
fn an_unprovable_absence_leaves_the_word_unknown() {
    let fixture =
        crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture::healthy(
            "head-unknown",
        );
    let pool: ScriptedPool = fixture.healthy_pool();
    let (folder, mut plan) = an_entered(&fixture, &pool);
    let probe: UnreadableProbe = UnreadableProbe;
    let head: SucceedingHead = SucceedingHead;
    let stop: RemovalStop = destroy_the_head(&mut HeadDestroyRequest {
        plan: &mut plan,
        folder: &folder,
        probe: &probe,
        device: &head,
    })
    .expect_err("an unreadable pool proves nothing");
    assert!(
        matches!(stop, RemovalStop::DoesNotAnswer { .. }),
        "the read that failed is the stop"
    );
    let filed: DestructionPlan = read_the_plan(fixture.common(), fixture.recorded().clone_guid)
        .expect("the plan reads back");
    assert_eq!(
        filed.head,
        HeadWord::Unknown,
        "the word says the outcome cannot be attributed"
    );
}

#[test]
fn a_clone_is_absent_only_by_its_guid_and_no_child_of_its_parent() {
    let probe: ChildCarrier = ChildCarrier;
    let absent = the_clone_absence(
        &probe,
        CLONE,
        DatasetGuid::parse(CLONE_GUID.to_string().as_str()).expect("a GUID"),
    )
    .expect("the read answers");
    assert_eq!(
        absent,
        CloneAbsence::There {
            name: String::from("tank/wt/moved-name"),
            guid: DatasetGuid::parse(CLONE_GUID.to_string().as_str()).expect("a GUID")
        },
        "a child carrying the GUID is the clone"
    );
    let clean: EmptyParent = EmptyParent;
    let gone = the_clone_absence(
        &clean,
        CLONE,
        DatasetGuid::parse(CLONE_GUID.to_string().as_str()).expect("a GUID"),
    )
    .expect("the read answers");
    assert_eq!(gone, CloneAbsence::Gone);
}

/// A probe whose clone name answers another GUID and whose parent holds one child carrying the recorded one.
struct ChildCarrier;

impl DatasetReader for ChildCarrier {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        if dataset == CLONE {
            return Ok(DatasetIdentity {
                stats: crate::worktree_clone_removal_proof_and_plan::removal_fixtures::the_stats(
                    CLONE_GUID + 1,
                    false,
                    CLONE_CREATETXG,
                    0,
                    "",
                ),
                properties: monochromatic_zfs_request_to_openzfs_kernel_device::NvList::new(),
            });
        }
        if dataset == "tank/wt/moved-name" {
            return Ok(DatasetIdentity {
                stats: crate::worktree_clone_removal_proof_and_plan::removal_fixtures::the_stats(
                    CLONE_GUID,
                    false,
                    CLONE_CREATETXG,
                    0,
                    "",
                ),
                properties: monochromatic_zfs_request_to_openzfs_kernel_device::NvList::new(),
            });
        }
        return Ok(DatasetIdentity {
            stats: crate::worktree_clone_removal_proof_and_plan::removal_fixtures::the_stats(
                99, false, 1, 0, "",
            ),
            properties: monochromatic_zfs_request_to_openzfs_kernel_device::NvList::new(),
        });
    }

    fn next_child(
        &self,
        parent: &str,
        cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        if parent != "tank/wt" {
            return Ok(None);
        }
        return match cursor {
            0 => Ok(Some(ListedChild {
                name: String::from("tank/wt/fix-flaky-test"),
                cursor: 1,
            })),
            1 => Ok(Some(ListedChild {
                name: String::from("tank/wt/moved-name"),
                cursor: 2,
            })),
            _ => Ok(None),
        };
    }
}

/// A probe whose clone name and parent's children hold no recorded GUID.
struct EmptyParent;

impl DatasetReader for EmptyParent {
    fn identity(
        &self,
        _dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        return Err(ReadFailure {
            said: String::from("no such dataset"),
            errno: Some(libc::ENOENT),
        });
    }

    fn next_child(
        &self,
        _parent: &str,
        _cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        return Ok(None);
    }
}

/// A probe that still answers the clone it was built over.
struct StubbornProbe {
    clone_guid: DatasetGuid,
}

impl DatasetReader for StubbornProbe {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        if dataset != CLONE {
            return Err(ReadFailure {
                said: String::from("no such dataset"),
                errno: Some(libc::ENOENT),
            });
        }
        let value: u64 = self.clone_guid.file_name().parse().unwrap_or(0);
        return Ok(DatasetIdentity {
            stats: crate::worktree_clone_removal_proof_and_plan::removal_fixtures::the_stats(
                value,
                false,
                CLONE_CREATETXG,
                0,
                "",
            ),
            properties: monochromatic_zfs_request_to_openzfs_kernel_device::NvList::new(),
        });
    }

    fn next_child(
        &self,
        _parent: &str,
        _cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        return Ok(None);
    }
}

/// A probe that refuses every read.
struct UnreadableProbe;

impl DatasetReader for UnreadableProbe {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        return Err(ReadFailure {
            said: format!("`{dataset}` cannot be read"),
            errno: Some(libc::EACCES),
        });
    }

    fn next_child(
        &self,
        parent: &str,
        _cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        return Err(format!("`{parent}` cannot be listed"));
    }
}

/// A head device that answers success.
struct SucceedingHead;

impl crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::DatasetDestroyDevice
    for SucceedingHead
{
    fn destroy_the_dataset(
        &self,
        _name: &str,
    ) -> Result<
        Option<monochromatic_zfs_request_to_openzfs_kernel_device::NvList>,
        monochromatic_zfs_request_to_openzfs_kernel_device::DeviceError,
    > {
        return Ok(None);
    }
}

/// A head device that refuses its one request with `EBUSY`.
struct RefusingHead {
    refuses: RefCell<usize>,
}

impl crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::DatasetDestroyDevice
    for RefusingHead
{
    fn destroy_the_dataset(
        &self,
        name: &str,
    ) -> Result<
        Option<monochromatic_zfs_request_to_openzfs_kernel_device::NvList>,
        monochromatic_zfs_request_to_openzfs_kernel_device::DeviceError,
    > {
        *self.refuses.borrow_mut() += 1;
        return Err(
            monochromatic_zfs_request_to_openzfs_kernel_device::DeviceError::Request {
                source: monochromatic_zfs_request_to_openzfs_kernel_device::Error::Call {
                    operation:
                        monochromatic_zfs_request_to_openzfs_kernel_device::Operation::Destroy,
                    target: String::from(name),
                    errno: libc::EBUSY,
                },
            },
        );
    }
}
