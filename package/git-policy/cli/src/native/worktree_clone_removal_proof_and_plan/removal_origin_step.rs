//! What:
//!  The origin cleanup step of one removal:
//!  the record published before the first destroy,
//!  and the attempt that destroys the checkout's origin snapshot or records why it cannot.
//! Why:
//!  The project decided the cleanup runs under the account's descendants-only `mount,destroy` on the
//!  checkout,
//!  with a separate durable record,
//!  and that the registration is settled whatever the origin does
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Project decisions",
//!  "Origin cleanup authority" and "Git registration when origin cleanup is blocked").
//!  A removal whose record cannot be written destroys nothing,
//!  because that record is what carries the cleanup past the clone
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals",
//!  step 6).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = publishTheOriginRecord(input, plan);
//! const done = attemptTheOriginCleanup(input, plan);
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_attempt::{
    CleanupAttempt,
    CleanupOutcome,
    attempt_the_cleanup,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_lines::{
    line_of_a_block,
    line_of_a_destruction,
    the_reason_changed,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_publication::CleanupFolder;
use crate::worktree_clone_origin_cleanup::origin_cleanup_reading::{
    an_existing_cleanup_record,
    origin_cleanups_folder,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_origin_cleanup::origin_cleanup_words::BlockReason;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_inputs::RemovalCompletion;
use crate::worktree_clone_removal_proof_and_plan::removal_report::OriginCleanupDone;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::path::PathBuf;

/// What:
///  The origin cleanup record,
///  published once before the removal's first destroy,
///  and kept when a record for the same snapshot already stands with the same identity.
/// Why:
///  The record is what carries the cleanup past the clone,
///  so a removal whose write fails destroys nothing,
///  and a rerun that finds the record keeps it when the identities agree and refuses when they
///  differ.
///
/// # Throws
/// A record that refuses the write,
/// or one that names another identity for the same snapshot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = publishTheOriginRecord(input, plan);
/// ```
pub(crate) fn publish_the_origin_record(
    input: &RemovalCompletion<'_>,
    plan: &DestructionPlan,
) -> Result<CleanupFolder, RemovalStop> {
    let Some(own_name) = plan
        .facts
        .origin_snapshot
        .rsplit_once('@')
        .map(|pair| return pair.1)
    else {
        return Err(a_refused_record(
            origin_cleanups_folder(input.entry.common_dir)
                .join(plan.facts.origin_snapshot.as_str()),
            format!(
                "the plan's origin snapshot `{}` names no own name, so no file of that record can be named",
                plan.facts.origin_snapshot
            ),
        ));
    };
    let folder: CleanupFolder =
        CleanupFolder::open_or_create(input.entry.common_dir).map_err(|refusal| {
            return a_refused_record(
                origin_cleanups_folder(input.entry.common_dir),
                refusal_words(&refusal),
            );
        })?;
    let wanted: OriginCleanupRecord = OriginCleanupRecord::owed(
        plan.facts.origin_dataset.as_str(),
        plan.facts.origin_dataset_guid,
        own_name,
        plan.facts.origin_snapshot_guid,
        plan.facts.origin_snapshot_createtxg,
        plan.facts.clone_dataset.as_str(),
        plan.facts.clone_guid,
        plan.facts.target.as_path(),
        input.now,
    );
    match an_existing_cleanup_record(input.entry.common_dir, own_name) {
        Ok(Some(standing)) if same_identity(&standing, &wanted) => return Ok(folder),
        Ok(Some(standing)) => {
            return Err(a_refused_record(
                folder.path().join(own_name),
                format!(
                    "a record already names `{}` with another identity",
                    standing.full_snapshot_name()
                ),
            ));
        }
        Ok(None) => {}
        Err(refusal) => {
            return Err(a_refused_record(
                folder.path().join(own_name),
                refusal_words(&refusal),
            ));
        }
    }
    folder.publish(&wanted).map_err(|refusal| {
        return a_refused_record(folder.path().join(own_name), refusal_words(&refusal));
    })?;
    return Ok(folder);
}

/// What:
///  The stop one record's refused write leaves:
///  the file the record would stand at and the field the reader refused.
/// Why:
///  The stop's sentence promises a path and a person needs the field too,
///  so the path is the record's own file and the field travels in the record's
///  own words,
///  which is how the reader's message already spells a refusal
///  (cause,
///  field and the value it held).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new RemovalStop(`the record at ${path} could not be written (${cause} on \`${field}\`: ${said})`);
/// ```
fn a_refused_record(
    path: PathBuf,
    said: String,
) -> RemovalStop {
    return RemovalStop::TheOriginCleanupCouldNotBeRecorded {
        path,
        said,
    };
}

/// The record's own words about what refused,
/// which name the field the reader refused and the value it held there.
fn refusal_words(
    refusal: &crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::CleanupRefusal
) -> String {
    return format!(
        "{} on `{}`: {}",
        refusal.cause.name(),
        refusal.field,
        refusal.said
    );
}

/// Whether a standing record names the same objects as the one this run wants.
fn same_identity(
    standing: &OriginCleanupRecord,
    wanted: &OriginCleanupRecord,
) -> bool {
    return standing.origin_dataset == wanted.origin_dataset
        && standing.origin_dataset_guid == wanted.origin_dataset_guid
        && standing.origin_snapshot == wanted.origin_snapshot
        && standing.origin_snapshot_guid == wanted.origin_snapshot_guid
        && standing.origin_snapshot_createtxg == wanted.origin_snapshot_createtxg
        && standing.clone_dataset == wanted.clone_dataset
        && standing.clone_guid == wanted.clone_guid;
}

/// What:
///  The removal's own origin attempt:
///  the record read,
///  the seven steps run,
///  and the record removed on completion or rewritten on a block.
/// Why:
///  The removal attempts its cleanup straight after the head goes,
///  so the ordinary case removes the record before the settle and the registration is settled
///  whatever the origin did.
///
/// # Throws
/// A record that cannot be read or rewritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const origin = attemptTheOriginCleanup(input, folder, plan);
/// ```
pub(crate) fn attempt_the_origin_cleanup(
    input: &RemovalCompletion<'_>,
    plan: &DestructionPlan,
) -> Result<OriginCleanupDone, RemovalStop> {
    let Some(own_name) = plan
        .facts
        .origin_snapshot
        .rsplit_once('@')
        .map(|pair| return pair.1)
    else {
        return Ok(OriginCleanupDone::NoRecord {
            said: String::from("the plan's origin snapshot names no own name"),
        });
    };
    let record: OriginCleanupRecord =
        match an_existing_cleanup_record(input.entry.common_dir, own_name) {
            Ok(Some(read)) => read,
            Ok(None) => {
                return Ok(OriginCleanupDone::NoRecord {
                    said: String::from(
                        "no record stands, so a finished cleanup is the state the pool is read for",
                    ),
                });
            }
            Err(refusal) => {
                return Ok(OriginCleanupDone::Unreadable {
                    path: origin_cleanups_folder(input.entry.common_dir).join(own_name),
                    said: refusal.said,
                });
            }
        };
    let folder: CleanupFolder =
        CleanupFolder::open_or_create(input.entry.common_dir).map_err(|refusal| {
            return a_refused_record(
                origin_cleanups_folder(input.entry.common_dir),
                refusal_words(&refusal),
            );
        })?;
    let outcome: CleanupOutcome = attempt_the_cleanup(&CleanupAttempt {
        record: &record,
        reads: input.reads,
        device: input.snapshots,
        attempted_at: input.now,
    });
    return match outcome {
        CleanupOutcome::Finished {
            destroyed,
        } => {
            folder.remove(own_name).map_err(|refusal| {
                return a_refused_record(folder.path().join(own_name), refusal_words(&refusal));
            })?;
            let line: Option<String> = destroyed
                .as_ref()
                .map(|name| return line_of_a_destruction(name.as_str()));
            Ok(OriginCleanupDone::Finished {
                snapshot: destroyed,
                line,
            })
        }
        CleanupOutcome::Blocked {
            record: next,
            detail,
        } => {
            folder.publish(next.as_ref()).map_err(|refusal| {
                return a_refused_record(folder.path().join(own_name), refusal_words(&refusal));
            })?;
            let reason: BlockReason = next
                .block
                .map_or(BlockReason::Unproved, |held| return held.reason);
            let line: Option<String> = if the_reason_changed(Some(&record), &next) {
                Some(line_of_a_block(
                    &next,
                    folder.path().join(own_name).as_path(),
                    detail.as_str(),
                ))
            } else {
                None
            };
            Ok(OriginCleanupDone::Blocked {
                snapshot: next.full_snapshot_name(),
                reason,
                detail,
                line,
            })
        }
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_origin_step_tests.rs"]
mod tests;
