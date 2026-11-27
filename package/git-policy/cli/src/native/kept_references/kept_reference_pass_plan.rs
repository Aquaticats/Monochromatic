//! What:
//!  The judgement one reading of a repository's kept references makes of the
//!  limits:
//!  which references go,
//!  why each goes,
//!  what the references that stay then reach,
//!  and whether the pass stopped with work still to do.
//! Why:
//!  The listing and the pass are the same judgement,
//!  so what a person reads is what the next pass does,
//!  and the order of the rules matters:
//!  a commit a branch reaches is no loss whatever its age,
//!  and its objects stop counting against the size limit as soon as it is
//!  planned.
//!  What the pass takes first is the oldest reference the rules allow,
//!  so the cap a pass has is spent on the references that waited longest.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = await planThePass(reading, records, limits, now, began);
//! type ThePlan = { planned: boolean[]; reasons: (WhyPruned | null)[];
//!   dropped: boolean[]; survivingBytes: number; stoppedEarly: boolean };
//! ```

/// The three bounds a pass is held to.
use super::kept_reference_pass_bounds::{
    A_PASS_WITHIN,
    THE_MOST_A_PASS_PRUNES,
};
/// The limits one reading is judged by.
use crate::kept_reference_limits::KeptReferenceLimits;
/// The answers one pass gives about a reference.
use crate::kept_reference_pass_answers::{
    WhyNotPruned,
    WhyPruned,
    age_at,
    commits_of_survivors,
    kept_at,
    oldest_the_floor_allows,
};
/// What was written beside a kept reference when it was written.
use crate::kept_reference_record::KeptReferenceRecord;
/// One reading of a repository's kept references,
/// and the bytes the references that stay reach.
use crate::kept_references_reading::{
    OneKeptReference,
    bytes_only_these_reach,
};
/// The Git context the survivors' commits are counted through.
use crate::transaction_git::GitContext;
/// The common Git directory every question is asked of.
use std::path::Path;
/// The moment the pass began,
/// which its own time is measured from.
use std::time::Instant;
/// What:
///  The judgement one reading makes of the limits:
///  which references go,
///  why each goes,
///  what the references that stay then reach,
///  and whether the pass stopped with work still to do.
/// Why:
///  The listing and the pass are the same judgement,
///  so what a person reads is what the next pass does.
///  The order of the rules matters:
///  a commit a branch reaches is no loss whatever its age,
///  and its objects stop counting against the size limit as soon as it is planned.
///  What the pass takes first is the oldest reference the rules allow,
///  so the cap a pass has is spent on the references that waited longest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = await planThePass(reading, records, limits, now);
/// ```
pub(super) async fn plan_the_pass(
    git: &GitContext,
    common_dir: &Path,
    reading: &[OneKeptReference],
    records: &[Option<KeptReferenceRecord>],
    limits: KeptReferenceLimits,
    now: u128,
    began: Instant,
) -> Result<ThePlan, String> {
    let mut plan: ThePlan = ThePlan {
        planned: vec![false; reading.len()],
        reasons: vec![None; reading.len()],
        dropped: vec![false; reading.len()],
        surviving_bytes: 0,
        stopped_early: false,
    };
    // The oldest first, so the references that waited longest are the ones a pass of the cap takes.
    let mut times: Vec<(u128, usize)> = Vec::with_capacity(reading.len());
    for at in 0..reading.len() {
        times.push((kept_at(records, at, now), at));
    }
    times.sort_by_key(the_time_of_a_pair);
    let mut order: Vec<usize> = Vec::with_capacity(times.len());
    for pair in &times {
        order.push(pair.1);
    }
    let mut count: usize = 0;
    for at in order {
        let why: Option<WhyPruned> = if reading[at].reached_a_branch {
            Some(WhyPruned::ReachedABranch)
        } else if age_at(records, at, now) > limits.max_age.as_millis() {
            Some(WhyPruned::TooOld)
        } else {
            None
        };
        let Some(reason) = why else {
            continue;
        };
        if count >= THE_MOST_A_PASS_PRUNES {
            // What the limits allow is left to the next pass, which runs after the next landed commit.
            plan.dropped[at] = true;
            plan.stopped_early = true;
            continue;
        }
        plan.planned[at] = true;
        plan.reasons[at] = Some(reason);
        count += 1;
    }
    plan.surviving_bytes = bytes_only_these_reach(
        git,
        common_dir,
        commits_of_survivors(reading, plan.planned.as_slice()).as_slice(),
    )
    .await?;
    // Each removal of the size rule is followed by a fresh reading of what the survivors reach, so the limit is judged
    //  from what the repository would keep and not from what it started with.
    while plan.surviving_bytes > limits.max_bytes {
        if count >= THE_MOST_A_PASS_PRUNES || began.elapsed() >= A_PASS_WITHIN {
            plan.stopped_early = true;
            break;
        }
        // Every reference that is left being younger than the floor is no failure: no pass may remove one for its size.
        let Some(oldest) =
            oldest_the_floor_allows(reading, records, plan.planned.as_slice(), limits, now)
        else {
            break;
        };
        plan.planned[oldest] = true;
        plan.reasons[oldest] = Some(WhyPruned::TooMuchKept);
        count += 1;
        plan.surviving_bytes = bytes_only_these_reach(
            git,
            common_dir,
            commits_of_survivors(reading, plan.planned.as_slice()).as_slice(),
        )
        .await?;
    }
    return Ok(plan);
}

/// What:
///  What one pass decided about every kept reference of a repository.
/// Why:
///  The removals and the listing are two readings of one decision,
///  so the decision is made once and carried as a value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ThePlan = { planned: boolean[]; reasons: (WhyPruned | null)[]; dropped: boolean[]; survivingBytes: number; stoppedEarly: boolean };
/// ```
pub(super) struct ThePlan {
    /// Whether each reference goes.
    pub(super) planned: Vec<bool>,
    /// Why each reference that goes does.
    pub(super) reasons: Vec<Option<WhyPruned>>,
    /// Whether the pass's cap left each reference to a later pass.
    pub(super) dropped: Vec<bool>,
    /// The bytes the references that stay reach.
    pub(super) surviving_bytes: u64,
    /// Whether work the limits allowed was left to a later pass.
    pub(super) stopped_early: bool,
}

/// What:
///  The time of one kept reference and its place in the reading,
/// as the sort reads it.
/// Why:
///  A pass takes the oldest references first,
///  and the place is what the rest of the pass asks about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theTimeOfAPair = ([keptAt]) => keptAt;
/// ```
fn the_time_of_a_pair(pair: &(u128, usize)) -> u128 {
    return pair.0;
}

/// What:
///  Why one reference the pass left is still there.
/// Why:
///  The listing names the rule that spared a reference,
///  and a reference the plan dropped for the cap is spared by the plan and not by a limit,
///  which a person has to be able to tell apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whyItStayed = (planned, dropped, age, limits) => dropped ? 'cap' : planned ? 'refused' : age <= limits.maxAge ? 'young' : 'floor';
/// ```
pub(super) fn why_it_stayed(
    planned: bool,
    dropped: bool,
    age_ms: u128,
    limits: KeptReferenceLimits,
) -> WhyNotPruned {
    if dropped {
        return WhyNotPruned::ThisPassReachedItsCap;
    }
    if planned {
        // The plan said this one goes and the removal was refused, which is where a reference another command moved ends.
        return WhyNotPruned::TheRemovalWasRefused;
    }
    if age_ms <= limits.max_age.as_millis() {
        return WhyNotPruned::YoungerThanTheLimit;
    }
    return WhyNotPruned::UnderTheSizeLimit;
}
