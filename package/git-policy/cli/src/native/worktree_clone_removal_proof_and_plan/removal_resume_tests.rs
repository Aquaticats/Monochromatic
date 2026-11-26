//! What:
//!  The controls of where a removal starts:
//!  a fresh entry,
//!  the head-gone path for a clone that is proved gone,
//!  the settling command's case for a head that was never requested,
//!  and the live intent an adopted plan needs.
//! Why:
//!  The plan's head word and the clone's own absence decide the path,
//!  and a plan with no live intent while the head stands is refused
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "Which resume path a removal takes" and "The removal intent").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(startTheRemoval(...).kind).toBe('gone');
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
    ReadFailure,
};
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_the_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::{
    ADoneUnmount,
    ScriptedClock,
};
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{
    RemovalEntry,
    enter_the_removal,
};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::the_stats;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE,
    CLONE_CREATETXG,
    CLONE_GUID,
    ORIGIN_DATASET,
    ORIGIN_SNAPSHOT,
    RemovalFixture,
    ScriptedPool,
};
use crate::worktree_clone_removal_proof_and_plan::removal_resume::{
    TheStart,
    start_the_removal,
};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_removal_deferred_origin_check::fixtures::{
    Answer,
    Pool,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

/// The slot every control removes.
const SLOT: &str = "fix-flaky-test";

/// One fixture whose plan the entry wrote.
fn a_written_plan(name: &str) -> (RemovalFixture, ScriptedPool) {
    let fixture: RemovalFixture = RemovalFixture::healthy(name);
    let pool: ScriptedPool = fixture.healthy_pool();
    let recorded = fixture.recorded();
    let unmount: ADoneUnmount = ADoneUnmount {
        clone_guid: recorded.clone_guid,
    };
    let at: RemovalEntry<'_> = RemovalEntry {
        common_dir: fixture.common(),
        slot: SLOT,
    };
    let _ =
        enter_the_removal(&at, &pool, &unmount, &ScriptedClock).expect("the entry writes the plan");
    return (fixture, pool);
}

/// Rewrite the standing plan's head word,
/// as a run that sent the head destroy and was killed would leave it.
fn set_the_head(
    fixture: &RemovalFixture,
    head: HeadWord,
) {
    let mut plan: DestructionPlan =
        read_the_plan(fixture.common(), fixture.recorded().clone_guid).expect("the plan reads");
    plan.head = head;
    let folder =
        crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder::open_or_create(
            fixture.common(),
        )
        .expect("the folder opens");
    folder.publish(&plan).expect("the plan is rewritten");
}

/// A probe whose clone name and parent's children answer nothing.
struct GoneProbe;

impl DatasetReader for GoneProbe {
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

#[test]
fn a_plan_whose_head_was_requested_and_whose_clone_is_gone_takes_the_head_gone_path() {
    let (fixture, pool) = a_written_plan("resume-gone");
    set_the_head(&fixture, HeadWord::Requested);
    let recorded = fixture.recorded();
    let unmount: ADoneUnmount = ADoneUnmount {
        clone_guid: recorded.clone_guid,
    };
    let at: RemovalEntry<'_> = RemovalEntry {
        common_dir: fixture.common(),
        slot: SLOT,
    };
    let started: TheStart = start_the_removal(&at, &pool, &unmount, &ScriptedClock, &GoneProbe)
        .expect("the absence proves the path");
    match started {
        TheStart::Gone {
            plan,
            ..
        } => assert_eq!(
            plan.head,
            HeadWord::Requested,
            "the plan the run took is kept whole"
        ),
        TheStart::Present {
            ..
        } => panic!("a plan whose clone is gone is never the head-present path"),
    }
}

#[test]
fn a_plan_whose_head_is_present_and_whose_clone_is_gone_names_the_settling_command() {
    let (fixture, pool) = a_written_plan("resume-settling");
    let recorded = fixture.recorded();
    let unmount: ADoneUnmount = ADoneUnmount {
        clone_guid: recorded.clone_guid,
    };
    let at: RemovalEntry<'_> = RemovalEntry {
        common_dir: fixture.common(),
        slot: SLOT,
    };
    let started: Result<TheStart, RemovalStop> =
        start_the_removal(&at, &pool, &unmount, &ScriptedClock, &GoneProbe);
    let stop: RemovalStop = match started {
        Ok(_) => panic!("a head never requested cannot have been destroyed by this run"),
        Err(refusal) => refusal,
    };
    match &stop {
        RemovalStop::TheCloneIsTheSettlingCommands {
            head,
            ..
        } => assert_eq!(head, "present"),
        other => panic!("the settling command's case, not {other:?}"),
    }
    assert!(
        stop.message().contains("settle-worktree-clone-record"),
        "the way forward is named"
    );
}

#[test]
fn an_adopted_plan_needs_a_live_intent() {
    let (fixture, pool) = a_written_plan("resume-intent");
    let recorded = fixture.recorded();
    let unmount: ADoneUnmount = ADoneUnmount {
        clone_guid: recorded.clone_guid,
    };
    let at: RemovalEntry<'_> = RemovalEntry {
        common_dir: fixture.common(),
        slot: SLOT,
    };
    let probe: Pool = Pool::new(vec![
        (
            CLONE,
            Answer::of(
                the_stats(CLONE_GUID, false, CLONE_CREATETXG, 0, ORIGIN_SNAPSHOT),
                NvList::new(),
            ),
        ),
        (
            ORIGIN_DATASET,
            Answer::of(
                the_stats(4_815_162_342_109_876_543, false, 2, 1, ""),
                NvList::new(),
            ),
        ),
    ]);
    let started: Result<TheStart, RemovalStop> =
        start_the_removal(&at, &pool, &unmount, &ScriptedClock, &probe);
    let stop: RemovalStop = match started {
        Ok(_) => panic!("a plan with no live intent proves no unmount"),
        Err(refusal) => refusal,
    };
    assert!(matches!(stop, RemovalStop::NoLiveIntent { .. }), "{stop:?}");
}

#[test]
fn a_fresh_repository_with_no_plan_enters_afresh() {
    let fixture: RemovalFixture = RemovalFixture::healthy("resume-fresh");
    let pool: ScriptedPool = fixture.healthy_pool();
    let recorded = fixture.recorded();
    let unmount: ADoneUnmount = ADoneUnmount {
        clone_guid: recorded.clone_guid,
    };
    let at: RemovalEntry<'_> = RemovalEntry {
        common_dir: fixture.common(),
        slot: SLOT,
    };
    let started: TheStart = start_the_removal(&at, &pool, &unmount, &ScriptedClock, &GoneProbe)
        .expect("a repository with no plan enters");
    assert!(
        matches!(started, TheStart::Present { .. }),
        "the entry's own proof decides, and the plan it writes stands"
    );
}
