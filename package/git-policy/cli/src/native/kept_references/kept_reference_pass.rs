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
//!  The pass is split across siblings because this file is over the line
//!  limit:
//!  this one runs one pass and lists which references it removed,
//!  `kept_reference_pass_bounds.rs` holds the three bounds it is held to,
//!  `kept_reference_pass_plan.rs` makes the judgement one reading leads to,
//!  `kept_reference_pass_removal.rs` removes one reference conditionally,
//!  and `kept_reference_pass_after_a_commit.rs` is where the pass runs by
//!  itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pass = await runThePass(git, commonDir, { dryRun });
//! ```

/// The three bounds one pass is held to,
/// re-exported so a reader of the pass finds them here.
pub use super::kept_reference_pass_bounds::{
    A_PASS_WITHIN,
    A_RECORD_OUTLIVES_ITS_REFERENCE_FOR,
    THE_MOST_A_PASS_PRUNES,
};
/// The judgement one reading makes of the limits.
use super::kept_reference_pass_plan::{
    ThePlan,
    plan_the_pass,
    why_it_stayed,
};
/// The conditional removal of one reference.
use super::kept_reference_pass_removal::remove_only_while_it_names;
/// The limits one reading is judged by,
/// and the environment the limits are read from.
use crate::kept_reference_limits::{
    KeptReferenceLimits,
    limits_from_the_environment,
};
/// The line one pruned reference leaves in the repository's log,
/// which is written before the reference goes.
use crate::kept_reference_log::log_one_pruning;
/// The answers one pass gives about a reference,
/// re-exported so the listing and the log read them from the pass.
pub use crate::kept_reference_pass_answers::{
    Kept,
    Pruned,
    WhyNotPruned,
    WhyPruned,
};
use crate::kept_reference_pass_answers::{
    ThePass,
    age_at,
    commits_of_survivors,
    kept_one,
    now_ms,
    pruned_one,
    recorded_or_first_seen,
};
/// What was written beside one kept reference when it was written.
use crate::kept_reference_record::KeptReferenceRecord;
/// One reading of a repository's kept references,
/// and the bytes the references that stay reach.
use crate::kept_references_reading::{
    OneKeptReference,
    bytes_only_these_reach,
    read_the_kept_references,
};
/// The Git context the pass asks its questions in.
use crate::transaction_git::GitContext;
/// The common Git directory every question is asked of,
/// and the worktree directory it is found from.
use std::path::{
    Path,
    PathBuf,
};
/// The moment the pass began,
/// which its own time is measured from.
use std::time::Instant;

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
pub async fn run_the_pass(
    git: &GitContext,
    common_dir: &Path,
    dry_run: bool,
) -> Result<ThePass, String> {
    let began: Instant = Instant::now();
    let limits: KeptReferenceLimits = limits_from_the_environment();
    let now: u128 = now_ms();
    let reading = read_the_kept_references(git, common_dir).await?;
    let mut records: Vec<Option<KeptReferenceRecord>> = Vec::with_capacity(reading.kept.len());
    for one in &reading.kept {
        records.push(recorded_or_first_seen(common_dir, one, now));
    }
    let plan: ThePlan = plan_the_pass(
        git,
        common_dir,
        reading.kept.as_slice(),
        records.as_slice(),
        limits,
        now,
        began,
    )
    .await?;
    let mut pass: ThePass = ThePass {
        bytes_before: reading.bytes_only_they_reach,
        bytes_after: plan.surviving_bytes,
        dry_run,
        stopped_early: plan.stopped_early,
        ..ThePass::default()
    };
    // A control acts between the listing and the removals here: a reference written after this point is not one this
    //  pass listed, and one written at a listed name carries another value than this pass reads.
    if let Err(error) = crate::transaction_phase::reach_transaction_phase(
        crate::transaction_phase::Phase::KeptReferencesListed,
        None,
    ) {
        return Err(format!(
            "the phase marker of a control stopped the pass: {error}"
        ));
    }
    let mut refused: bool = false;
    for (at, one) in reading.kept.iter().enumerate() {
        let record: Option<&KeptReferenceRecord> = records[at].as_ref();
        let age_ms: u128 = age_at(records.as_slice(), at, now);
        let Some(why) = plan.reasons[at] else {
            pass.kept.push(kept_one(
                one,
                record,
                now,
                why_it_stayed(plan.planned[at], plan.dropped[at], age_ms, limits),
            ));
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
            pass.kept.push(kept_one(
                one,
                record,
                now,
                WhyNotPruned::ThePassRanOutOfTime,
            ));
            continue;
        }
        // A branch can move off the commit between the listing and this removal, and the removal's own condition
        //  would not see it: the question is asked again, and a commit nothing reaches stays where it is.
        if why == WhyPruned::ReachedABranch
            && !crate::kept_references_reading::a_branch_reaches(
                git,
                common_dir,
                one.commit.as_str(),
            )
            .await?
        {
            pass.kept.push(kept_one(
                one,
                record,
                now,
                WhyNotPruned::TheBranchMovedOffIt,
            ));
            refused = true;
            continue;
        }
        // The line goes before the reference does, because the reference was the only handle on the commit:
        //  a crash between the two must not leave the repository without either.
        log_one_pruning(common_dir, &pruned, OUTCOME_PRUNED, now);
        if let Some(words) =
            remove_only_while_it_names(git, common_dir, one.reference.as_str(), one.commit.as_str())
                .await
        {
            crate::service_warn!(
                "the kept reference {} was not pruned ({words}), so its commit {} is left where it is",
                one.reference,
                one.commit
            );
            log_one_pruning(common_dir, &pruned, OUTCOME_REFUSED, now);
            pass.kept.push(kept_one(
                one,
                record,
                now,
                WhyNotPruned::TheRemovalWasRefused,
            ));
            refused = true;
            continue;
        }
        crate::kept_reference_record::remove_record(common_dir, one.reference.as_str());
        pass.pruned.push(pruned);
    }
    // A removal another pass or another command refused leaves a reference the plan expected to go, so the figure the
    //  report prints is read from the references that are still there.
    if refused {
        pass.bytes_after = bytes_only_these_reach(
            git,
            common_dir,
            commits_of_survivors(
                reading.kept.as_slice(),
                planned_by_name(&pass, reading.kept.as_slice()).as_slice(),
            )
            .as_slice(),
        )
        .await?;
    }
    // A record whose reference is gone names nothing, and the folder it sits in is bounded by the references the
    //  repository holds: a grace of the floor keeps a record another command wrote a moment ago.
    if !dry_run {
        crate::kept_reference_record::sweep_records_of_references_that_are_gone(
            common_dir,
            reading.kept.as_slice(),
            now,
            A_RECORD_OUTLIVES_ITS_REFERENCE_FOR,
        );
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
fn planned_by_name(
    pass: &ThePass,
    kept: &[OneKeptReference],
) -> Vec<bool> {
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
pub async fn prune_after_a_commit(
    git: &GitContext,
    git_dir: &Path,
) {
    let common_dir: PathBuf = common_directory_of_this_repository(git_dir);
    // The questions of the pass carry their revisions on standard input, which Git is fed from a private file in the
    //  repository's own scratch directory: the service's context names none, so the pass names the one it is about.
    let about_the_repository: GitContext = GitContext {
        common_dir: Some(common_dir.clone()),
        ..git.clone()
    };
    return match run_the_pass(&about_the_repository, common_dir.as_path(), false).await {
        Ok(pass) => crate::service_log!(
            "kept references: {} pruned of {}, {} bytes held before and {} after",
            pass.pruned.len(),
            pass.pruned.len() + pass.kept.len(),
            pass.bytes_before,
            pass.bytes_after
        ),
        Err(reason) => crate::service_warn!(
            "the kept references of {} were not pruned: {reason}",
            common_dir.display()
        ),
    };
}
