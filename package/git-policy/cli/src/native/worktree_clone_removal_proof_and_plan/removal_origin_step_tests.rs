//! What:
//!  The controls of the origin step:
//!  its own tests live with the cleanup folder's attempt,
//!  and this sidecar holds the rules the step adds.
//! Why:
//!  The record's moment before the first destroy and the attempt's seven steps are the
//!  cleanup folder's own,
//!  so this file names only what the removal adds to them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theRecordStands()).toBe(true);
//! ```

use crate::foundation::iso_time::now_iso;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_fixtures::{
    ClearingDevice,
    EpochProbe,
    LivePool,
    ORIGIN_OWN_NAME,
    ScriptedHead,
    ScriptedHolders,
    ScriptedSettle,
    TheCompletion,
    a_clear_scan,
    a_fixture,
    complete_with_the_moment,
    the_budget,
    the_checks_pool,
};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_removal_deferred_origin_check::fixtures::Pool;
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

/// What:
///  A removal whose record its own reader refuses stops with a sentence that names
///  the record's path and the field the reader refused.
/// Why:
///  The stop promised a path and carried the refused field's name where a path belongs,
///  so a person read "the origin cleanup record at startedAt could not be written",
///  which names neither the file a person has to look at nor the field inside it.
///  The moment is the producer's own millisecond spelling,
///  the form the command path read before the fix beside this control,
///  and the record's reader refuses it here.
#[test]
fn the_stop_names_the_records_path_and_the_field_its_reader_refused() {
    let fixture: RemovalFixture = a_fixture("the-stops-sentence");
    let pool: LivePool = LivePool::healthy(ORIGIN_OWN_NAME);
    let check: Pool = the_checks_pool();
    let hidden: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let probe: EpochProbe = EpochProbe {
        pool: check,
        hidden: Rc::clone(&hidden),
    };
    let device: ClearingDevice = ClearingDevice::over(&pool);
    let head: ScriptedHead = ScriptedHead::succeeding(
        fixture.plan_folder(),
        fixture.recorded().clone_guid,
        Rc::clone(&hidden),
    );
    let holders: ScriptedHolders = ScriptedHolders::always(a_clear_scan());
    let settle: ScriptedSettle = ScriptedSettle::succeeding();
    let refused: String = now_iso();
    let stop: RemovalStop = complete_with_the_moment(
        &TheCompletion {
            fixture: &fixture,
            pool: &pool,
            check: &probe,
            probe: &probe,
            device: &device,
            head: &head,
            holders: &holders,
            settle: &settle,
            cancelling: None,
            budget: the_budget(),
        },
        refused.as_str(),
    )
    .expect_err("a moment the record's reader refuses stops the removal");
    let (path, said): (PathBuf, String) = match &stop {
        RemovalStop::TheOriginCleanupCouldNotBeRecorded {
            path,
            said,
        } => (path.clone(), said.clone()),
        other => panic!("the record's write is what refused, not {other:?}"),
    };
    assert_eq!(
        path.file_name().and_then(|name| return name.to_str()),
        Some(ORIGIN_OWN_NAME),
        "the stop names the record's own file: {}",
        path.display()
    );
    assert!(
        said.contains("`startedAt`"),
        "the stop's words name the field the reader refused: {said}"
    );
    let sentence: String = stop.message();
    let shown: String = path.display().to_string();
    assert!(
        sentence.contains(shown.as_str()),
        "the sentence a person reads names the record's path: {sentence}"
    );
    assert!(
        sentence.contains("startedAt"),
        "the sentence a person reads names the field: {sentence}"
    );
    assert!(
        !sentence.contains("record at startedAt"),
        "the sentence never promises a path and prints a field's name there: {sentence}"
    );
}
