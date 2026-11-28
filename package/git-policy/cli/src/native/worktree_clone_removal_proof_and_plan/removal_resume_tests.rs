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
use crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::AStagingFolderAControlAnswers;
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
use crate::worktree_clone_removal_proof_and_plan::removal_resume::start_the_removal;
use crate::worktree_clone_removal_proof_and_plan::removal_start_inputs::TheStart;
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
    let _ = enter_the_removal(
        &at,
        &pool,
        &unmount,
        &ScriptedClock,
        &AStagingFolderAControlAnswers::making_nothing(),
    )
    .expect("the entry writes the plan");
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
    let started: TheStart = start_the_removal(
        &at,
        &pool,
        &unmount,
        &ScriptedClock,
        &GoneProbe,
        &AStagingFolderAControlAnswers::making_nothing(),
    )
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
    let started: Result<TheStart, RemovalStop> = start_the_removal(
        &at,
        &pool,
        &unmount,
        &ScriptedClock,
        &GoneProbe,
        &AStagingFolderAControlAnswers::making_nothing(),
    );
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
    let started: Result<TheStart, RemovalStop> = start_the_removal(
        &at,
        &pool,
        &unmount,
        &ScriptedClock,
        &probe,
        &AStagingFolderAControlAnswers::making_nothing(),
    );
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
    let started: TheStart = start_the_removal(
        &at,
        &pool,
        &unmount,
        &ScriptedClock,
        &GoneProbe,
        &AStagingFolderAControlAnswers::making_nothing(),
    )
    .expect("a repository with no plan enters");
    assert!(
        matches!(started, TheStart::Present { .. }),
        "the entry's own proof decides, and the plan it writes stands"
    );
}

/// The GUID another clone answers under the recorded name.
const ANOTHER_CLONES_GUID: u64 = 17_157_080_670_377_245_571;

/// What:
///  A probe that answers one GUID under any name,
///  or fails every read,
///  and lists no child of anything.
/// Why:
///  The resume path is chosen from the plan and the live clone alone,
///  so a probe that answers only that one question decides it.
struct AProbeThatAnswers {
    /// The GUID every name answers with,
    /// or nothing when the reads fail instead.
    clone_guid: Option<u64>,
    /// The read that fails in place of an answer.
    failure: Option<ReadFailure>,
}

impl DatasetReader for AProbeThatAnswers {
    fn identity(
        &self,
        _dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        if let Some(failure) = &self.failure {
            return Err(failure.clone());
        }
        let value: u64 = self.clone_guid.unwrap_or(CLONE_GUID);
        return Ok(DatasetIdentity {
            stats: the_stats(value, false, CLONE_CREATETXG, 0, ORIGIN_SNAPSHOT),
            properties: NvList::new(),
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

/// The start one written plan with the head word a control names takes.
fn the_start_of(
    fixture: &RemovalFixture,
    pool: &ScriptedPool,
    head: HeadWord,
    probe: &dyn DatasetReader,
) -> Result<TheStart, RemovalStop> {
    set_the_head(fixture, head);
    let recorded = fixture.recorded();
    let unmount: ADoneUnmount = ADoneUnmount {
        clone_guid: recorded.clone_guid,
    };
    let at: RemovalEntry<'_> = RemovalEntry {
        common_dir: fixture.common(),
        slot: SLOT,
    };
    return start_the_removal(
        &at,
        pool,
        &unmount,
        &ScriptedClock,
        probe,
        &AStagingFolderAControlAnswers::making_nothing(),
    );
}

/// A plan whose head is `gone` and whose clone is gone takes the head-gone path,
/// which is the ordinary kill inside the head's destroy.
#[test]
fn a_plan_whose_head_is_gone_and_whose_clone_is_gone_takes_the_head_gone_path() {
    let (fixture, pool) = a_written_plan("resume-head-gone");
    let started: TheStart = the_start_of(&fixture, &pool, HeadWord::Gone, &GoneProbe)
        .expect("the absence proves the path");
    match started {
        TheStart::Gone {
            plan,
            ..
        } => assert_eq!(plan.head, HeadWord::Gone, "the plan is kept whole"),
        TheStart::Present {
            ..
        } => panic!("a clone proved gone is never the head-present path"),
    }
}

/// A recorded name that answers another clone's GUID is a reused name and the
/// removal refuses it.
#[test]
fn a_clone_name_that_answers_another_guid_is_refused_as_reused() {
    let (fixture, pool) = a_written_plan("resume-reused-name");
    let reuse: AProbeThatAnswers = AProbeThatAnswers {
        clone_guid: Some(ANOTHER_CLONES_GUID),
        failure: None,
    };
    let stop: RemovalStop = match the_start_of(&fixture, &pool, HeadWord::Requested, &reuse) {
        Ok(_) => panic!("a recorded name carrying another GUID is a reuse"),
        Err(stop) => stop,
    };
    match &stop {
        RemovalStop::TheCloneNameWasReused {
            name,
            recorded_guid,
            live_guid,
        } => {
            assert_eq!(name, CLONE);
            assert_eq!(recorded_guid, &CLONE_GUID.to_string());
            assert_eq!(live_guid, &ANOTHER_CLONES_GUID.to_string());
        }
        other => panic!("the reuse's own refusal, not {other:?}"),
    }
}

/// What:
///  A pool that cannot prove the clone's absence refuses the resume,
///  names the plan and the failed read,
///  and names the settling command for every head word but `gone`.
/// Why:
///  The design's cell for an unreadable or partial listing commits to exactly
///  those three things
///  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
///  "Which resume path a removal takes").
#[test]
fn a_pool_that_cannot_prove_the_absence_names_the_plan_and_the_settling_command() {
    let (fixture, pool) = a_written_plan("resume-unproved-present");
    let unreadable: AProbeThatAnswers = AProbeThatAnswers {
        clone_guid: None,
        failure: Some(ReadFailure {
            said: String::from("the pool answered EIO"),
            errno: Some(libc::EIO),
        }),
    };
    let stop: RemovalStop = match the_start_of(&fixture, &pool, HeadWord::Present, &unreadable) {
        Ok(_) => panic!("a read that stopped proves no path"),
        Err(stop) => stop,
    };
    let plan: std::path::PathBuf =
        crate::worktree_clone_binding_record::registration_home::plans_folder(fixture.common())
            .join(fixture.recorded().clone_guid.file_name());
    match &stop {
        RemovalStop::TheClonesAbsenceCouldNotBeProved {
            head,
            path,
            ..
        } => {
            assert_eq!(head, "present");
            assert_eq!(path, &plan);
        }
        other => panic!("the unproved absence's own refusal, not {other:?}"),
    }
    let said: String = stop.message();
    assert!(said.contains(plan.display().to_string().as_str()), "{said}");
    assert!(said.contains("the pool answered EIO"), "{said}");
    assert!(
        said.contains("settle-worktree-clone-record"),
        "a head that is not `gone` names the settling command: {said}"
    );
    assert!(said.contains("changes nothing"), "{said}");
}

/// The same refusal for a plan whose head is `gone` names the finish pass and
/// not the settling command,
/// which refuses such a plan.
#[test]
fn a_pool_that_cannot_prove_the_absence_of_a_gone_head_names_the_finish_pass() {
    let (fixture, pool) = a_written_plan("resume-unproved-gone");
    let unreadable: AProbeThatAnswers = AProbeThatAnswers {
        clone_guid: None,
        failure: Some(ReadFailure {
            said: String::from("the pool answered EIO"),
            errno: Some(libc::EIO),
        }),
    };
    let stop: RemovalStop = match the_start_of(&fixture, &pool, HeadWord::Gone, &unreadable) {
        Ok(_) => panic!("a read that stopped proves no path"),
        Err(stop) => stop,
    };
    let said: String = stop.message();
    assert!(said.contains("the finish pass"), "{said}");
    assert!(
        !said.contains("settle-worktree-clone-record"),
        "the settling command refuses a `gone` plan and is not named: {said}"
    );
}
