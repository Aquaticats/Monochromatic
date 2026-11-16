//! What:
//!  The pass that prunes a repository's kept references:
//!  what one reading of them makes of the limits,
//!  the removal of each reference the limits allow,
//!  and the record and the log line that go with each removal.
//! Why:
//!  Nothing removed a kept reference before this,
//!  so the objects only they reach grew without end
//!  (the owner's answer of 2026-11-12,
//!  "Keep the reference,
//! auto prune by size and age").
//!  Three things decide a reference's fate,
//!  in this order:
//!  a commit a branch has since reached holds nothing,
//!  so its reference goes at once;
//!  a reference older than the age limit goes,
//!  because a person who was going to recover that commit has had the time;
//!  and while the objects only kept references reach weigh more than the size limit,
//!  the oldest references go first,
//!  never one younger than the floor,
//!  so a commit kept a moment ago is not the price of a burst.
//!  The pass runs where no command waits for it:
//!  in the commit service's after-commit upkeep
//!  (`service_work.rs`,
//!  `maintenance_awaiting`),
//!  and through `git monochromatic-git-policy-cli kept-references --prune`
//!  when a person asks.
//!  It is bounded three ways:
//!  every Git start it makes carries the step ceiling every other start of the wrapper carries
//!  (`transaction_git_step_bounds.rs`),
//!  it plans no more removals than `THE_MOST_A_PASS_PRUNES`,
//!  and it stops once `A_PASS_WITHIN` has passed,
//!  leaving the rest to the next pass,
//! which runs after the next commit.
//!  No lock is taken:
//!  two passes that run at once remove the same references conditionally,
//!  each with the value that reference had when that pass listed it
//!  (`git update-ref -d <ref> <old>`),
//!  so a reference another pass removed,
//! or one a keeper wrote after the listing,
//! is left alone by
//!  the pass that listed it,
//!  and two passes end where one of them alone would have.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pass = await runThePass(git, commonDir, { dryRun });
//! ```

use crate::kept_reference_limits::{KeptReferenceLimits, limits_from_the_environment};
use crate::kept_reference_log::log_one_pruning;
use crate::kept_reference_pass_answers::{
    ThePass, age_at, commits_of_survivors, kept_at, kept_one, now_ms, oldest_the_floor_allows, pruned_one, recorded_or_first_seen,
};
use crate::kept_reference_record::KeptReferenceRecord;
use crate::kept_references_reading::{OneKeptReference, bytes_only_these_reach, read_the_kept_references};
use crate::transaction_git::{GitContext, GitRequest};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Re-exported so the listing and the log read the pass's answers from the pass.
pub use crate::kept_reference_pass_answers::{Kept, Pruned, WhyNotPruned, WhyPruned};

/// How many kept references one pass may remove.
///  Sixteen:
///  a pass runs after every commit a repository lands,
/// so a burst larger than this is worked through
///  by the passes that follow it,
/// and no single pass holds the service's upkeep for long.
///  Measured:
/// this repository held five kept references in the 5.7 days before 2026-11-12,
///  so no pass has had to remove any yet,
///  and the cap costs nothing until a repository's kept references actually accumulate.
pub const THE_MOST_A_PASS_PRUNES: usize = 16;

/// How long a record whose reference is gone is kept before a pass removes it.
///  One hour:
///  a record names the worktree the commit was built in,
///  and a reference another command wrote after a pass listed the repository has one written a moment ago,
///  so the file is kept long past the moment anything could still be arriving for it.
///  The size it costs is nothing next to the reference it describes.
pub const A_RECORD_OUTLIVES_ITS_REFERENCE_FOR: Duration = Duration::from_secs(60 * 60);

/// How long one pass may spend planning and removing.
///  Twenty seconds:
///  the service's after-commit work starts housekeeping for thirty seconds a round
///  (`HOUSEKEEPING_OF_ONE_ROUND`),
/// and a pass that ends inside that slice leaves the round's own
///  bound intact.
///  Measured:
/// a pass over this repository's five kept references is four Git starts
///  (a `for-each-ref`,
/// a `rev-list`,
/// a `cat-file` and one `update-ref` for each removal),
///  each of them milliseconds on the host of 2026-11-12.
pub const A_PASS_WITHIN: Duration = Duration::from_secs(20);

/// What:
///  Remove one kept reference only while it still names the commit the pass listed,
///  and answer Git's words when it did not.
/// Why:
///  A reference another command wrote between the listing and this removal names a commit this pass
///  never judged,
///  and that commit may be the only one its name holds,
///  so the old value is the condition of the removal
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = await git(['update-ref', '--no-deref', '-d', name, old]); // refused when the value moved
/// ```
async fn remove_only_while_it_names(git: &GitContext, common_dir: &Path, name: &str, commit: &str) -> Option<String> {
    let request: GitRequest = GitRequest::new(common_dir, &["update-ref", "--no-deref", "-d", name, commit]);
    return match run_git_awaiting(git, &request).await {
        Ok(output) if output.succeeded() => None,
        Ok(output) => Some(String::from_utf8_lossy(output.stderr.as_slice()).trim().to_owned()),
        Err(error) => Some(error.to_string()),
    };
}

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
async fn plan_the_pass(
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
    plan.surviving_bytes = bytes_only_these_reach(git, common_dir, commits_of_survivors(reading, plan.planned.as_slice()).as_slice()).await?;
    // Each removal of the size rule is followed by a fresh reading of what the survivors reach, so the limit is judged
    //  from what the repository would keep and not from what it started with.
    while plan.surviving_bytes > limits.max_bytes {
        if count >= THE_MOST_A_PASS_PRUNES || began.elapsed() >= A_PASS_WITHIN {
            plan.stopped_early = true;
            break;
        }
        // Every reference that is left being younger than the floor is no failure: no pass may remove one for its size.
        let Some(oldest) = oldest_the_floor_allows(reading, records, plan.planned.as_slice(), limits, now) else {
            break;
        };
        plan.planned[oldest] = true;
        plan.reasons[oldest] = Some(WhyPruned::TooMuchKept);
        count += 1;
        plan.surviving_bytes = bytes_only_these_reach(git, common_dir, commits_of_survivors(reading, plan.planned.as_slice()).as_slice()).await?;
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
struct ThePlan {
    /// Whether each reference goes.
    planned: Vec<bool>,
    /// Why each reference that goes does.
    reasons: Vec<Option<WhyPruned>>,
    /// Whether the pass's cap left each reference to a later pass.
    dropped: Vec<bool>,
    /// The bytes the references that stay reach.
    surviving_bytes: u64,
    /// Whether work the limits allowed was left to a later pass.
    stopped_early: bool,
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
fn why_it_stayed(planned: bool, dropped: bool, age_ms: u128, limits: KeptReferenceLimits) -> WhyNotPruned {
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

/// What:
///  Run one pass over a repository's kept references.
/// Why:
///  The service's upkeep and a person's command are the same judgement and the same removals,
///  and only the removal itself is left out when a person asks what the pass would do.
///  A failed Git question removes nothing and answers the reason,
///  so a repository under a lock or a full disk keeps every commit it holds.
///  A removal Git refuses leaves the reference where it is and says so,
///  which is what a reference another command moved or another pass removed comes to,
///  and the next pass judges it from what it names then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pass = await runThePass(git, commonDir, dryRun);
/// ```
///
/// # Errors
/// The words of the Git question that failed,
/// when the pass could not read the repository.
pub async fn run_the_pass(git: &GitContext, common_dir: &Path, dry_run: bool) -> Result<ThePass, String> {
    let began: Instant = Instant::now();
    let limits: KeptReferenceLimits = limits_from_the_environment();
    let now: u128 = now_ms();
    let reading = read_the_kept_references(git, common_dir).await?;
    let mut records: Vec<Option<KeptReferenceRecord>> = Vec::with_capacity(reading.kept.len());
    for one in &reading.kept {
        records.push(recorded_or_first_seen(common_dir, one, now));
    }
    let plan: ThePlan = plan_the_pass(git, common_dir, reading.kept.as_slice(), records.as_slice(), limits, now, began).await?;
    let mut pass: ThePass =
        ThePass { bytes_before: reading.bytes_only_they_reach, bytes_after: plan.surviving_bytes, dry_run, stopped_early: plan.stopped_early, ..ThePass::default() };
    // A control acts between the listing and the removals here: a reference written after this point is not one this
    //  pass listed, and one written at a listed name carries another value than this pass reads.
    if let Err(error) = crate::transaction_phase::reach_transaction_phase(crate::transaction_phase::Phase::KeptReferencesListed, None) {
        return Err(format!("the phase marker of a control stopped the pass: {error}"));
    }
    let mut refused: bool = false;
    for (at, one) in reading.kept.iter().enumerate() {
        let record: Option<&KeptReferenceRecord> = records[at].as_ref();
        let age_ms: u128 = age_at(records.as_slice(), at, now);
        let Some(why) = plan.reasons[at] else {
            pass.kept.push(kept_one(one, record, now, why_it_stayed(plan.planned[at], plan.dropped[at], age_ms, limits)));
            continue;
        };
        let pruned: Pruned = pruned_one(one, record, now, why);
        if dry_run {
            pass.pruned.push(pruned);
            continue;
        }
        // A pass that runs out of its time leaves the rest to the next one, which no command waits for either.
        if began.elapsed() >= A_PASS_WITHIN {
            pass.stopped_early = true;
            pass.kept.push(kept_one(one, record, now, WhyNotPruned::ThePassRanOutOfTime));
            continue;
        }
        // A branch can move off the commit between the listing and this removal, and the removal's own condition
        //  would not see it: the question is asked again, and a commit nothing reaches stays where it is.
        if why == WhyPruned::ReachedABranch
            && !crate::kept_references_reading::a_branch_reaches(git, common_dir, one.commit.as_str()).await?
        {
            pass.kept.push(kept_one(one, record, now, WhyNotPruned::TheBranchMovedOffIt));
            refused = true;
            continue;
        }
        // The line goes before the reference does, because the reference was the only handle on the commit:
        //  a crash between the two must not leave the repository without either.
        log_one_pruning(common_dir, &pruned, OUTCOME_PRUNED, now);
        if let Some(words) = remove_only_while_it_names(git, common_dir, one.reference.as_str(), one.commit.as_str()).await {
            crate::service_warn!(
                "the kept reference {} was not pruned ({words}), so its commit {} is left where it is",
                one.reference,
                one.commit
            );
            log_one_pruning(common_dir, &pruned, OUTCOME_REFUSED, now);
            pass.kept.push(kept_one(one, record, now, WhyNotPruned::TheRemovalWasRefused));
            refused = true;
            continue;
        }
        crate::kept_reference_record::remove_record(common_dir, one.reference.as_str());
        pass.pruned.push(pruned);
    }
    // A removal another pass or another command refused leaves a reference the plan expected to go, so the figure the
    //  report prints is read from the references that are still there.
    if refused {
        pass.bytes_after = bytes_only_these_reach(git, common_dir, commits_of_survivors(reading.kept.as_slice(), planned_by_name(&pass, reading.kept.as_slice()).as_slice()).as_slice()).await?;
    }
    // A record whose reference is gone names nothing, and the folder it sits in is bounded by the references the
    //  repository holds: a grace of the floor keeps a record another command wrote a moment ago.
    if !dry_run {
        crate::kept_reference_record::sweep_records_of_references_that_are_gone(common_dir, reading.kept.as_slice(), now, A_RECORD_OUTLIVES_ITS_REFERENCE_FOR);
    }
    return Ok(pass);
}

/// The word a log line carries when the pass is about to remove the reference it names.
pub const OUTCOME_PRUNED: &str = "pruned";

/// The word a log line carries when the removal the line before it announced was refused.
pub const OUTCOME_REFUSED: &str = "refused";

/// What:
///  Which references the pass removed,
///  as the plan the size question reads.
/// Why:
///  A removal another command refused leaves a reference the plan expected to go,
///  and the weight of what a repository still holds is asked of the references that are there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plannedByName = (pass, kept) => kept.map((one) => !pass.pruned.some((pruned) => pruned.reference === one.reference));
/// ```
fn planned_by_name(pass: &ThePass, kept: &[OneKeptReference]) -> Vec<bool> {
    let mut removed: Vec<bool> = Vec::with_capacity(kept.len());
    for one in kept {
        let mut gone: bool = false;
        for pruned in &pass.pruned {
            if pruned.reference == one.reference {
                gone = true;
                break;
            }
        }
        // A reference still there is a survivor, which the commits of the survivors name.
        removed.push(gone);
    }
    return removed;
}

/// What:
///  The common Git directory the kept references of a worktree's repository belong to.
/// Why:
///   A linked worktree's own Git directory holds neither the references nor the records,
///  and every question and every path the pass uses is the common directory's
///  (`service_work_common_directory.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commonDirectory = (gitDir) => commonDirectoryOf(gitDir) ?? gitDir;
/// ```
pub fn common_directory_of_this_repository(git_dir: &Path) -> PathBuf {
    return match crate::service_work_common_directory::common_directory_of(git_dir) {
        Some(found) => found,
        // A worktree whose Git directory names no other is the repository's common one.
        None => git_dir.to_path_buf(),
    };
}

/// What:
///  Prune the kept references of the repository a worktree's commit landed in,
///  for the commit service's after-commit upkeep,
///  and say in the service's log what the pass did.
/// Why:
///  This is the place the pruning runs by itself:
///  the person's commit has returned,
///  the upkeep is one record of the after-commit work,
///  and nothing a command waits for is behind it
///  (`service_work.rs`,
///  `maintenance_awaiting`).
///  Nothing here can fail the upkeep:
///  the maintenance step's own verdict is what the round answers,
///  and a pass that could not read the repository says so and removes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await with_kept_references_pruned(context, item.git_dir);
/// ```
pub async fn prune_after_a_commit(git: &GitContext, git_dir: &Path) {
    let common_dir: PathBuf = common_directory_of_this_repository(git_dir);
    // The questions of the pass carry their revisions on standard input, which Git is fed from a private file in the
    //  repository's own scratch directory: the service's context names none, so the pass names the one it is about.
    let about_the_repository: GitContext = GitContext { common_dir: Some(common_dir.clone()), ..git.clone() };
    return match run_the_pass(&about_the_repository, common_dir.as_path(), false).await {
        Ok(pass) => crate::service_log!(
            "kept references: {} pruned of {}, {} bytes held before and {} after",
            pass.pruned.len(),
            pass.pruned.len() + pass.kept.len(),
            pass.bytes_before,
            pass.bytes_after
        ),
        Err(reason) => crate::service_warn!("the kept references of {} were not pruned: {reason}", common_dir.display()),
    };
}
