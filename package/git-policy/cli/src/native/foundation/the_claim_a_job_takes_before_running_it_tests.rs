//! The claim on one job's work:
//! whichever of the job and the reporter takes it first decides,
//! the other is refused,
//! and neither can take it twice.

use super::*;

/// A job that takes the claim first does its work, and the reporter may not promise it never started.
#[test]
fn the_job_that_takes_the_claim_first_runs_and_the_report_says_the_result_is_not_known() {
    let claim: Arc<TheClaimOnAJobsWork> = TheClaimOnAJobsWork::new();
    assert!(claim.the_job_takes_it(), "the job is first, so it may do its work");
    assert!(!claim.the_reporter_takes_it(), "the reporter cannot say the job never started once the job has taken the claim");
    assert!(!claim.the_job_takes_it(), "the claim is taken once, so a second take by the job is refused");
    assert!(!claim.the_reporter_takes_it(), "the claim is taken once, so a second take by the reporter is refused");
}

/// A reporter that takes the claim first can promise the work did not run, and the job does nothing.
#[test]
fn the_reporter_that_takes_the_claim_first_stops_the_job() {
    let claim: Arc<TheClaimOnAJobsWork> = TheClaimOnAJobsWork::new();
    assert!(claim.the_reporter_takes_it(), "the reporter is first, so it may say the job never started");
    assert!(!claim.the_job_takes_it(), "the job must do nothing once the report has said it never started");
    assert!(!claim.the_reporter_takes_it(), "the claim is taken once, so a second take by the reporter is refused");
}
