//! What:
//!  The controls of the entry's one order:
//!  a healthy removal that writes its plan and prints its loss,
//!  each refusal that stops it before the unmount is asked,
//!  an answer that does not prove the unmount,
//!  and a plan that already stands.
//! Why:
//!  The design's order is the whole guarantee:
//!  the proof refuses before anything is written,
//!  the intent and the unmount come before the plan,
//!  and the plan is the last thing the entry writes
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//! section 3).
//!  A control that asserts only the refusal would not show which step it refused at,
//!  so each one also asserts that the unmount was never asked and that no plan exists.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entered = enterTheRemoval(entry, pool, unmount, clock);
//! expect(entered.plan.head).toBe('present');
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_the_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    HeadWord,
    SnapshotOutcome,
};
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{
    EnteredRemoval,
    RemovalClock,
    RemovalEntry,
    enter_the_removal,
};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE,
    CLONE_GUID,
    ORIGIN_CREATETXG,
    ORIGIN_SNAPSHOT_GUID,
    RemovalFixture,
    SLOT,
    ScriptedPool,
    an_object,
    guid,
};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{
    RemovalUnmount,
    UnmountAnswer,
};
use std::cell::RefCell;

/// The run nonce every control's plan carries.
const RUN_NONCE: &str = "00112233445566778899aabbccddeeff";

/// What:
///  The clock a control answers,
///  so the plan's three process facts are the control's and not the test runner's.
/// Why:
///  The owner and the moment are facts about the writing process,
///  and a control that read its own would make the assertion about the runner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clock = { run: () => nonce, owner: () => '48213:linux:9481733', started: () => '2026-11-22T18:40:12-05:00' };
/// ```
struct ScriptedClock;

impl RemovalClock for ScriptedClock {
    fn run(&self) -> Result<RequestNonce, RemovalStop> {
        return Ok(RequestNonce::parse(RUN_NONCE).expect("the control's nonce is canonical"));
    }

    fn owner(&self) -> Result<String, RemovalStop> {
        return Ok(String::from("48213:linux:9481733"));
    }

    fn started(&self) -> String {
        return String::from("2026-11-22T18:40:12-05:00");
    }
}

/// What:
///  What one scripted unmount does to the repository while it answers.
/// Why:
///  The design verifies the clone GUID,
/// the underlay identity and the Git registration again after the
///  unmount,
/// so a registration swapped during the request has to stop the plan rather than the destroy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Act = 'answer' | 'swap-the-registration';
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
enum Act {
    Answer,
    SwapTheRegistration,
}

/// What:
///  The unmount a control answers,
///  with the count of times it was asked and the operation it was asked for.
/// Why:
///  Every refusal must stop before the unmount is requested,
///  and that is a fact about this seam rather than about the plan folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ScriptedUnmount { asked = 0; answer = (intent) => done(intent); }
/// ```
struct ScriptedUnmount {
    asked: RefCell<Vec<MountOperation>>,
    refuse: Option<String>,
    act: Act,
    registration: Option<std::path::PathBuf>,
}

impl ScriptedUnmount {
    fn answering() -> ScriptedUnmount {
        return ScriptedUnmount {
            asked: RefCell::new(Vec::new()),
            refuse: None,
            act: Act::Answer,
            registration: None,
        };
    }

    fn refusing(step: &str) -> ScriptedUnmount {
        return ScriptedUnmount {
            asked: RefCell::new(Vec::new()),
            refuse: Some(String::from(step)),
            act: Act::Answer,
            registration: None,
        };
    }

    fn swapping_the_registration(registration: &std::path::Path) -> ScriptedUnmount {
        return ScriptedUnmount {
            asked: RefCell::new(Vec::new()),
            refuse: None,
            act: Act::SwapTheRegistration,
            registration: Some(registration.to_path_buf()),
        };
    }

    fn times_asked(&self) -> usize {
        return self.asked.borrow().len();
    }
}

impl RemovalUnmount for ScriptedUnmount {
    fn request_the_unmount(
        &self,
        intent: &crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent,
    ) -> Result<UnmountAnswer, RemovalStop> {
        self.asked.borrow_mut().push(intent.operation);
        if let Some(step) = &self.refuse {
            return Err(RemovalStop::UnmountFailed {
                step: step.clone(),
                said: String::from("the control refused this step"),
            });
        }
        if self.act == Act::SwapTheRegistration {
            let Some(registration) = &self.registration else {
                panic!("the control that swaps a registration names one");
            };
            std::fs::remove_dir_all(registration.as_path()).expect("the registration goes");
            std::fs::create_dir_all(registration.as_path())
                .expect("another registration takes its name");
        }
        return Ok(UnmountAnswer {
            dataset: guid(CLONE_GUID),
            operation: intent.operation,
            nonce: intent.nonce.clone(),
            outcome: ReceiptOutcome::Done,
            mount_id: None,
            failed_step: None,
            errno: None,
        });
    }
}

/// The entry one fixture makes.
fn entry(fixture: &RemovalFixture) -> RemovalEntry<'_> {
    return RemovalEntry {
        common_dir: fixture.common(),
        slot: SLOT,
    };
}

#[test]
fn a_healthy_removal_writes_the_plan_after_the_unmount_and_prints_its_loss() {
    let fixture: RemovalFixture = RemovalFixture::healthy("entry-healthy");
    let pool: ScriptedPool = fixture.healthy_pool();
    let unmount: ScriptedUnmount = ScriptedUnmount::answering();
    let entered: EnteredRemoval =
        enter_the_removal(&entry(&fixture), &pool, &unmount, &ScriptedClock)
            .expect("the healthy removal enters");
    assert_eq!(
        unmount.times_asked(),
        1,
        "the protection asked for the unmount exactly once"
    );
    assert_eq!(unmount.asked.borrow()[0], MountOperation::Unmount);
    assert_eq!(entered.plan.head, HeadWord::Present);
    assert_eq!(entered.plan.attempts.len(), 2);
    for line in &entered.plan.attempts {
        assert_eq!(line.attempt, 0);
        assert_eq!(line.outcome, SnapshotOutcome::Planned);
    }
    assert_eq!(entered.plan.facts.clone_dataset, CLONE);
    assert_eq!(
        entered.plan.facts.unmount_nonce.as_str(),
        unmount_nonce_of(&entered.plan.facts.unmount_nonce)
    );
    assert_eq!(entered.plan.facts.owner, "48213:linux:9481733");
    let read_back =
        read_the_plan(fixture.common(), guid(CLONE_GUID)).expect("the written plan reads back");
    assert_eq!(
        read_back, entered.plan,
        "the file holds the plan the entry returns"
    );
    let lost: String = entered.lost.text();
    assert!(
        lost.contains(&format!("{CLONE}@worktree-own")),
        "the loss names the own snapshot: {lost}"
    );
    assert!(
        lost.contains(&format!("{CLONE}@sanoid-hourly")),
        "the loss names the sanoid snapshot: {lost}"
    );
}

/// The nonce one plan carries,
/// which is the one the control's unmount answered.
fn unmount_nonce_of(nonce: &RequestNonce) -> String {
    return String::from(nonce.as_str());
}

#[test]
fn a_refused_lineage_stops_before_the_unmount_is_asked_and_writes_no_plan() {
    let fixture: RemovalFixture = RemovalFixture::healthy("entry-transferred");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.snapshots.push(an_object(
        EntryKind::Snapshot,
        "earliest",
        ORIGIN_SNAPSHOT_GUID,
        ORIGIN_CREATETXG,
    ));
    let unmount: ScriptedUnmount = ScriptedUnmount::answering();
    match enter_the_removal(&entry(&fixture), &pool, &unmount, &ScriptedClock)
        .expect_err("a transferred snapshot refuses the removal")
    {
        RemovalStop::TransferredHistory {
            ..
        } => {}
        other => panic!("a transferred snapshot refuses as inherited history, not {other:?}"),
    }
    assert_eq!(
        unmount.times_asked(),
        0,
        "no unmount was requested before the proof passed"
    );
    assert_eq!(fixture.the_plan_folder_holds(), 0, "no plan was written");
}

#[test]
fn a_refused_bookmark_stops_before_the_unmount_and_names_every_refused_one() {
    let fixture: RemovalFixture = RemovalFixture::healthy("entry-bookmark");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.bookmarks.push(an_object(
        EntryKind::Bookmark,
        "from-a-gone-snapshot",
        42,
        19,
    ));
    pool.bookmarks.push(an_object(
        EntryKind::Bookmark,
        "from-another-gone-snapshot",
        43,
        21,
    ));
    let unmount: ScriptedUnmount = ScriptedUnmount::answering();
    match enter_the_removal(&entry(&fixture), &pool, &unmount, &ScriptedClock)
        .expect_err("a refused bookmark refuses the removal")
    {
        RemovalStop::BookmarksRefused {
            refused,
            ..
        } => assert_eq!(refused.len(), 2, "every refused bookmark is named"),
        other => panic!("a refused bookmark refuses as one, not {other:?}"),
    }
    assert_eq!(unmount.times_asked(), 0);
    assert_eq!(fixture.the_plan_folder_holds(), 0);
}

#[test]
fn an_unmount_that_is_not_proved_stops_before_the_plan_is_written() {
    let fixture: RemovalFixture = RemovalFixture::healthy("entry-unmount");
    let pool: ScriptedPool = fixture.healthy_pool();
    let unmount: ScriptedUnmount = ScriptedUnmount::refusing("unmount");
    match enter_the_removal(&entry(&fixture), &pool, &unmount, &ScriptedClock)
        .expect_err("a refused unmount stops the removal")
    {
        RemovalStop::UnmountFailed {
            step,
            ..
        } => assert_eq!(step, "unmount"),
        other => panic!("a refused unmount stops as an unmount failure, not {other:?}"),
    }
    assert_eq!(unmount.times_asked(), 1);
    assert_eq!(
        fixture.the_plan_folder_holds(),
        0,
        "the plan is written only after the unmount is proved"
    );
}

#[test]
fn a_registration_swapped_during_the_unmount_stops_the_plan() {
    let fixture: RemovalFixture = RemovalFixture::healthy("entry-registration-swapped");
    let pool: ScriptedPool = fixture.healthy_pool();
    let unmount: ScriptedUnmount =
        ScriptedUnmount::swapping_the_registration(fixture.registration.as_path());
    match enter_the_removal(&entry(&fixture), &pool, &unmount, &ScriptedClock)
        .expect_err("a registration swapped during the unmount stops the removal")
    {
        RemovalStop::IdentityDiffers {
            field,
            ..
        } => assert_eq!(field, "registrationIdentity"),
        other => panic!("a swapped registration stops on its identity, not {other:?}"),
    }
    assert_eq!(
        unmount.times_asked(),
        1,
        "the swap happened inside the one unmount request"
    );
    assert_eq!(
        fixture.the_plan_folder_holds(),
        0,
        "no plan was written for a registration that is not the recorded one"
    );
}

#[test]
fn a_plan_already_standing_is_named_and_left() {
    let fixture: RemovalFixture = RemovalFixture::healthy("entry-standing-plan");
    let pool: ScriptedPool = fixture.healthy_pool();
    let first: ScriptedUnmount = ScriptedUnmount::answering();
    let entered: EnteredRemoval =
        enter_the_removal(&entry(&fixture), &pool, &first, &ScriptedClock)
            .expect("the first run writes its plan");
    let second: ScriptedUnmount = ScriptedUnmount::answering();
    match enter_the_removal(&entry(&fixture), &pool, &second, &ScriptedClock)
        .expect_err("a standing plan stops this entry")
    {
        RemovalStop::APlanStands {
            slot,
            path,
        } => {
            assert_eq!(slot, SLOT);
            assert_eq!(path, entered.plan_path);
        }
        other => panic!("a standing plan is named, not {other:?}"),
    }
    assert_eq!(
        second.times_asked(),
        0,
        "the resumed removal's path is later steps', and no second unmount was asked"
    );
}

#[test]
fn a_binding_that_is_not_there_stops_before_anything_is_read_from_the_pool() {
    let fixture: RemovalFixture = RemovalFixture::healthy("entry-no-binding");
    let pool: ScriptedPool = fixture.healthy_pool();
    let unmount: ScriptedUnmount = ScriptedUnmount::answering();
    match enter_the_removal(
        &RemovalEntry {
            common_dir: fixture.common(),
            slot: "no-such-slot",
        },
        &pool,
        &unmount,
        &ScriptedClock,
    )
    .expect_err("an absent binding stops the removal")
    {
        RemovalStop::NoBinding {
            slot,
            ..
        } => assert_eq!(slot, "no-such-slot"),
        other => panic!("an absent binding stops as absent, not {other:?}"),
    }
    assert_eq!(unmount.times_asked(), 0);
    assert_eq!(fixture.the_plan_folder_holds(), 0);
}

#[test]
fn a_registration_of_another_identity_stops_the_removal_before_the_pool() {
    let fixture: RemovalFixture = RemovalFixture::healthy("entry-registration");
    let pool: ScriptedPool = fixture.healthy_pool();
    let unmount: ScriptedUnmount = ScriptedUnmount::answering();
    std::fs::remove_dir_all(fixture.registration.as_path()).expect("the registration goes");
    std::fs::create_dir_all(fixture.registration.as_path())
        .expect("another registration takes its name");
    match enter_the_removal(&entry(&fixture), &pool, &unmount, &ScriptedClock)
        .expect_err("another worktree's registration stops the removal")
    {
        RemovalStop::IdentityDiffers {
            field,
            ..
        } => assert_eq!(field, "registrationIdentity"),
        other => panic!("another registration stops as an identity difference, not {other:?}"),
    }
    assert_eq!(unmount.times_asked(), 0);
    assert_eq!(fixture.the_plan_folder_holds(), 0, "no plan was written");
}
