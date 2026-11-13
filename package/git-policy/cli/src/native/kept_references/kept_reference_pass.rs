//! What:
//!  The pass that prunes a repository's kept references:
//!  what one reading of them makes of the limits,
//!  the removal of each reference the limits allow,
//!  and the record and the log line that go with each removal.
//! Why:
//!  Nothing removed a kept reference before this,
//!  so the objects only they reach grew without end
//!  (the owner's answer of 2026-11-12,
//!  "Keep the reference, auto prune by size and age").
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
//!  leaving the rest to the next pass, which runs after the next commit.
//!  No lock is taken:
//!  two passes that run at once remove the same references conditionally,
//!  each with the value that reference had when that pass listed it
//!  (`git update-ref -d <ref> <old>`),
//!  so a reference another pass removed, or one a keeper wrote after the listing, is left alone by
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
    ThePass, commits_of_survivors, kept_one, now_ms, oldest_the_floor_allows, pruned_one, recorded_or_first_seen,
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
///  a pass runs after every commit a repository lands, so a burst larger than this is worked through
///  by the passes that follow it, and no single pass holds the service's upkeep for long.
///  Measured: this repository held five kept references in the 5.7 days before 2026-11-12,
///  so no pass has had to remove any yet,
///  and the cap costs nothing until a repository's kept references actually accumulate.
pub const THE_MOST_A_PASS_PRUNES: usize = 16;

/// How long one pass may spend planning and removing.
///  Twenty seconds:
///  the service's after-commit work starts housekeeping for thirty seconds a round
///  (`HOUSEKEEPING_OF_ONE_ROUND`), and a pass that ends inside that slice leaves the round's own
///  bound intact.
///  Measured: a pass over this repository's five kept references is four Git starts
///  (a `for-each-ref`, a `rev-list`, a `cat-file` and one `update-ref` for each removal),
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = async (reading, records, limits, now) => ({ planned, reasons, surviving, stoppedEarly });
/// ```
async fn plan_the_pass(
    git: &GitContext,
    common_dir: &Path,
    reading: &[OneKeptReference],
    records: &[Option<KeptReferenceRecord>],
    limits: KeptReferenceLimits,
    now: u128,
    began: Instant,
) -> Result<(Vec<bool>, Vec<Option<WhyPruned>>, u64, bool), String> {
    let mut planned: Vec<bool> = vec![false; reading.len()];
    let mut reasons: Vec<Option<WhyPruned>> = vec![None; reading.len()];
    let mut count: usize = 0;
    for at in 0..reading.len() {
        let age: u128 = crate::kept_reference_pass_answers::age_at(records, at, now);
        if reading[at].reached_a_branch {
            planned[at] = true;
            reasons[at] = Some(WhyPruned::ReachedABranch);
            count += 1;
        } else if age > limits.max_age.as_millis() {
            planned[at] = true;
            reasons[at] = Some(WhyPruned::TooOld);
            count += 1;
        }
    }
    // What the references that stay still reach, now that the two rules above have had their say.
    let mut surviving: u64 = bytes_only_these_reach(git, common_dir, commits_of_survivors(reading, planned.as_slice()).as_slice()).await?;
    let mut stopped_early: bool = false;
    while surviving > limits.max_bytes {
        if count >= THE_MOST_A_PASS_PRUNES || began.elapsed() >= A_PASS_WITHIN {
            stopped_early = true;
            break;
        }
        // Every reference that is left being younger than the floor is no failure: no pass may remove one for its size.
        let Some(oldest) = oldest_the_floor_allows(reading, records, planned.as_slice(), limits, now) else {
            break;
        };
        planned[oldest] = true;
        reasons[oldest] = Some(WhyPruned::TooMuchKept);
        count += 1;
        surviving = bytes_only_these_reach(git, common_dir, commits_of_survivors(reading, planned.as_slice()).as_slice()).await?;
    }
    return Ok((planned, reasons, surviving, stopped_early));
}

/// What:
///  Run one pass over a repository's kept references.
/// Why:
///  The service's upkeep and a person's command are the same judgement and the same removals,
///  and only the removal itself is left out when a person asks what the pass would do.
///  A failed Git question removes nothing and answers the reason,
///  so a repository under a lock or a full disk keeps every commit it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pass = await runThePass(git, commonDir, dryRun);
/// ```
///
/// # Errors
/// The words of the Git question that failed, when the pass could not read the repository.
pub async fn run_the_pass(git: &GitContext, common_dir: &Path, dry_run: bool) -> Result<ThePass, String> {
    let began: Instant = Instant::now();
    let limits: KeptReferenceLimits = limits_from_the_environment();
    let now: u128 = now_ms();
    let reading = read_the_kept_references(git, common_dir).await?;
    let mut records: Vec<Option<KeptReferenceRecord>> = Vec::with_capacity(reading.kept.len());
    for one in &reading.kept {
        records.push(recorded_or_first_seen(common_dir, one, now));
    }
    let (planned, reasons, bytes_after, stopped_early) =
        plan_the_pass(git, common_dir, reading.kept.as_slice(), records.as_slice(), limits, now, began).await?;
    let mut pass: ThePass = ThePass { bytes_before: reading.bytes_only_they_reach, bytes_after, dry_run, stopped_early, ..ThePass::default() };
    // A control acts between the listing and the removals here: a reference written after this point is not one this
    //  pass listed, and one written at a listed name carries another value than this pass reads.
    if let Err(error) = crate::transaction_phase::reach_transaction_phase(crate::transaction_phase::Phase::KeptReferencesListed, None) {
        return Err(format!("the phase marker of a control stopped the pass: {error}"));
    }
    for (at, one) in reading.kept.iter().enumerate() {
        let record: Option<&KeptReferenceRecord> = records[at].as_ref();
        let Some(why) = reasons[at] else {
            pass.kept.push(kept_one(one, record, now, limits, planned[at], stopped_early));
            continue;
        };
        let pruned: Pruned = pruned_one(one, record, now, why);
        if dry_run {
            pass.pruned.push(pruned);
            continue;
        }
        if let Some(refused) = remove_only_while_it_names(git, common_dir, one.reference.as_str(), one.commit.as_str()).await {
            crate::service_warn!(
                "the kept reference {} was not pruned ({refused}), so its commit {} is left where it is",
                one.reference,
                one.commit
            );
            pass.kept.push(kept_one(one, record, now, limits, planned[at], true));
            continue;
        }
        crate::kept_reference_record::remove_record(common_dir, one.reference.as_str());
        log_one_pruning(common_dir, &pruned, now);
        pass.pruned.push(pruned);
    }
    return Ok(pass);
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
    return crate::service_work_common_directory::common_directory_of(git_dir).unwrap_or_else(|| { return git_dir.to_path_buf(); });
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
    return match run_the_pass(git, common_dir.as_path(), false).await {
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
