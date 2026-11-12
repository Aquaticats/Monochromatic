//! Controls for the storage steps of a drain:
//! a step whose own job outlasted its ceiling says so,
//! and never that it did not run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const said = await aStorageStep(shortCeiling, 'the step', 'nothing was done', aJobHeldOpen); expect(said).toContain('gave no answer');
//! ```

use super::*;
use crate::parallel_test_support::PATIENCE;

/// How long the control gives a step's place and then its job,
/// short enough that the job cannot finish inside it while the control holds the job open.
const THE_CONTROLS_CEILING: Duration = Duration::from_millis(200);

/// What:
///  A storage step whose own job outlasted its ceiling is answered as a step that gave no answer,
///  never as a step that did not run.
/// Why:
///   The two answers decide what the drain may do next and they are not the same:
/// a step that did not run changed nothing,
/// so its records keep their lease for the next round,
/// while a step that outlasted its ceiling may still be finishing what it began,
/// and a caller told "nothing happened" could claim the same records beside it.
/// The job is held open by this control,
/// so the answer cannot come from the work ending inside the ceiling.
#[test]
fn a_step_whose_own_job_outlasted_its_ceiling_says_so() {
    let (started, has_started) = std::sync::mpsc::channel::<()>();
    let (let_go, waits) = std::sync::mpsc::channel::<()>();
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime for the control");
    let answered = runtime.block_on(a_storage_step_awaited_no_longer_than(THE_CONTROLS_CEILING, "the step", "nothing was done", move || -> u8 {
        let _told = started.send(());
        let _held = waits.recv();
        return 7;
    }));
    assert!(
        has_started.recv_timeout(PATIENCE).is_ok(),
        "the step's job started, so its ceiling is the only thing that can end this step's wait"
    );
    let said: String = answered.expect_err("a job the control held open outlasted the ceiling, so the step gave no answer");
    assert!(said.contains("the step gave no answer"), "the step says it gave no answer: {said}");
    assert!(said.contains("nothing was done"), "the step says what it did not do: {said}");
    assert!(
        said.contains("may still be being written"),
        "the step says what it began may still be finishing, which is what decides that nothing may claim its records: {said}"
    );
    assert!(!said.contains("did not run"), "a job that started did not fail to run: {said}");
    // The job is let go, and the runtime's own drop waits for it.
    let _let_go = let_go.send(());
    drop(runtime);
}
