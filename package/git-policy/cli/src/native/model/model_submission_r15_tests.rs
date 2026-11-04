//! What:
//!  The controls of the submission window (the fifteenth review's controls 7 and 13).
//! Why:
//!  Red under revision 14 (a non-final record read as failure; a drain reading an empty FIFO), green over every schedule under revision 15.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a full commit write then client death is unknown, never failed', ...);
//! ```

use crate::model_explore::{Report, replay};
use crate::model_submission_r15::{Rules, check, initial, invariant, steps};

/// Control 7: the full `commit` write succeeds, the client dies before it records the submission, and the last record is the earlier
/// non-final `not-submitted`. Revision 14 reads it as positively failed and starts a second attempt beside the first (red); revision 15 reads
/// a non-final record as unknown and offers no retry.
#[test]
fn a_full_commit_write_then_client_death_is_unknown_and_not_failed() {
    let schedule = &["client: records not-submitted (non-final)", "client: writes commit", "environment: the client dies"];
    let fourteen = replay(initial(Rules::Fourteen), steps, schedule);
    let retried = replay(fourteen, steps, &["owner: E1 reads not-submitted as positively failed and starts a second attempt"]);
    assert_eq!(invariant(&retried), Some("a second attempt was started although the first attempt's commit was written".to_string()));
    let fifteen = replay(initial(Rules::Fifteen), steps, schedule);
    assert!(steps(&fifteen).iter().all(|step| { return !step.0.starts_with("owner: E1"); }), "no retry is offered for an unknown submission");
}

/// Control 13: Git read the line ahead into its buffer, so the FIFO looks empty; a drain that concludes "nothing was submitted" is contradicted
/// by the buffered `commit` that runs (red under 14). Revision 15's drain draws no conclusion from the FIFO.
#[test]
fn a_fifo_that_looks_empty_after_read_ahead_does_not_mean_nothing_was_submitted() {
    let reached = replay(
        initial(Rules::Fourteen),
        steps,
        &[
            "client: records not-submitted (non-final)",
            "client: writes commit",
            "git: reads the line ahead into its buffer; the FIFO looks empty",
            "owner: the drain finds the FIFO empty and concludes that nothing was submitted",
        ],
    );
    assert!(reached.buffered);
    assert_eq!(invariant(&reached), Some("the drain concluded that nothing was submitted and a commit was written (queued, buffered or run)".to_string()));
}

/// The search over every schedule: red under 14, green and exhausted under 15.
#[test]
fn the_submission_window_is_red_under_fourteen_and_green_under_fifteen() {
    assert!(check(Rules::Fourteen).violation.is_some());
    let fifteen: Report = check(Rules::Fifteen);
    assert!(fifteen.violation.is_none() && fifteen.exhausted, "{:?}", fifteen.violation.map(|found| { return found.trace; }));
}
