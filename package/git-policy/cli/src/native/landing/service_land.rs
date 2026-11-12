//! What:
//!  The service's serialized step:
//! land a batch of prepared commits on a branch,
//! and finish the batches a killed
//!  service left open.
//! Why:
//!  Everything that must happen one commit at a time happens here,
//! in a fixed order,
//! with one durable write:
//! the objects
//!  of the whole batch are written by one `fast-import`,
//! the intent is journaled,
//! the branch moves by one compare-and-swap,
//!  the index entries of the named paths are updated,
//! and the intent is marked done
//!  (`doc/planning/cli-git-resident-service.md`,
//! "The serialized step,
//! group commit,
//! FIFO and backpressure").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcomes = await landBatch(context, seq, requests);
//! ```

#[cfg(test)]
use std::path::PathBuf;
pub use crate::service_land_context::LandContext;
use crate::service_land_outcomes::{IgnoreAnnouncements, any_landed};

/// What became of one request of a batch,
/// named here so every caller of a landing keeps one path.
pub use crate::service_land_outcomes::Outcome;

pub use crate::service_land_finish::{Handled, RecoveryReport, recover, recover_instance, recover_instance_without_drops};
#[cfg(test)]
use crate::service_land_finish::follow_up;
#[cfg(test)]
use crate::service_journal::{Recovery, list_intents};
#[cfg(test)]
use crate::service_index_lock::try_acquire;
use crate::service_built_checks::history_is_plain;
use crate::service_scratch_ref_marker_of_a_batch::end_scratch_ref_record;
#[cfg(test)]
use crate::service_journal::{Intent, OPEN_EXTENSION};
#[cfg(test)]
use crate::service_head::{follows_move, still_checked_out};
#[cfg(test)]
use crate::transaction_git::GitContext;
#[cfg(test)]
use std::path::Path;
use crate::service_land_batch_body::land_batch_body;
use crate::service_land_settings::Settings;

/// How many times a batch is built again when the branch moved under it before the failure is reported,
/// named here so the service client keeps one path.
pub use crate::service_land_settings::LANDING_ATTEMPTS;

use crate::service_index_lock::Announcer;
use crate::service_timing::LandTiming;
use crate::config_snapshot::{ConfigSnapshot, read_config_snapshot};
use crate::service_protocol::{CommitRequest, FailureCode};
use std::time::Instant;

/// What:
///  Lands a batch of requests that share a repository,
/// a common git directory and a branch.
/// Why:
///   Requests whose paths changed on the branch since their base are sent back to be prepared again;
/// the others become
///  one commit each,
/// in order,
/// on the branch,
/// with one journal write and one branch update for all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await landBatch(context, seq, requests) // one outcome per request, in order
/// ```
pub fn land_batch(context: &LandContext, seq: u64, requests: &[CommitRequest]) -> Result<Vec<Outcome>, FailureCode> {
    return land_batch_announcing(context, seq, requests, &IgnoreAnnouncements);
}

/// `land_batch`,
/// telling the clients of the batch what it waits for when another program holds the index.
pub fn land_batch_announcing(context: &LandContext, seq: u64, requests: &[CommitRequest], announce: &dyn Announcer) -> Result<Vec<Outcome>, FailureCode> {
    return land_batch_timed(context, seq, requests, announce, &mut LandTiming::default());
}

/// What:
///  `land_batch_announcing`,
/// leaving in `timing` the time each phase of the landing took.
/// Why:
///   The service's log line for each request states the wait for another program's hold on the index,
/// the objects,
/// the journal
///  write,
/// the branch move and the index update;
/// `timing` is filled as the phases end,
/// so a landing that fails part way still
///  reports the phases it reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await landBatchTimed(context, seq, requests, announce, timing)
/// ```
///
/// # Errors
///
/// The failure code of the landing,
/// as `land_batch_announcing`.
pub fn land_batch_timed(
    context: &LandContext,
    seq: u64,
    requests: &[CommitRequest],
    announce: &dyn Announcer,
    timing: &mut LandTiming,
) -> Result<Vec<Outcome>, FailureCode> {
    // The batch names the worktree it belongs to, and every Git of the landing reads and writes that repository: a Git
    //  directory beside its work tree, or one an environment variable sent elsewhere, is found in this process only when the
    //  landing names it (`repository_selection::of_paths`).
    let named: LandContext = match requests.first() {
        Some(first) => context.for_the_repository(first.git_dir.as_path(), first.repository.as_path()),
        None => context.clone(),
    };
    let landing: &LandContext = &named;
    // The landing queues its own syncs and settles them when it ends: nothing is flushed while the person waits for a commit that
    //  is published, and what a landing that publishes nothing wrote is flushed before it returns (`service_durable.rs`). The
    //  index file follows `core.fsync` as native Git's index write does; a configuration that cannot be read is taken to name it.
    let config_began: Instant = Instant::now();
    let snapshot: Option<ConfigSnapshot> = match requests.first() {
        Some(first) => read_config_snapshot(&landing.git, first.repository.as_path()),
        None => None,
    };
    let hardens_index: bool = snapshot.as_ref().and_then(ConfigSnapshot::hardens_index).unwrap_or(true);
    // Native Git writes a commit's objects as loose objects and flushes none by default; the import of a small batch writes a
    //  temporary pack, flushes it and deletes it, so the flush is dropped unless the repository asks for flushed objects.
    let import_fsync: Option<String> = snapshot.as_ref().and_then(ConfigSnapshot::import_fsync_value);
    let deferral = crate::service_durable::begin(hardens_index);
    // The object store may answer the checks of a commit Git built only where Git shows the history the files say: the settings that
    //  change it (a shallow or graft file elsewhere, another replacement namespace, replacements switched off) are read here from
    //  the environment and configuration the landing's Git children run under, and a configuration that cannot be read counts as one.
    let plain_history: bool = match requests.first() {
        Some(first) => history_is_plain(first.common_dir.as_path(), &landing.git, snapshot.as_ref()),
        None => false,
    };
    timing.reads_split.config = config_began.elapsed();
    let settings: Settings<'_> = Settings { import_fsync: import_fsync.as_deref(), plain_history };
    let result: Result<Vec<Outcome>, FailureCode> = land_batch_body(landing, seq, requests, announce, timing, &settings);
    // A step ended at its ceiling anywhere else in the landing: the clients are told which, and the commit the batch built, when it
    //  built one, is kept on a side ref. No open intent finishes a batch whose branch has not moved, so nothing lands later.
    if result == Err(FailureCode::StepTooSlow) {
        crate::service_land_git_steps_and_import_stream::keep_what_an_ended_step_built(landing, announce, requests, seq);
    }
    // A batch whose own side reference did not keep the commit Git built is answered as not made, and the commit is on no branch: it is
    //  kept where a commit that did not land is kept, and the person is told where, because a commit no reference names is one
    //  `git gc --prune=now` takes (`keep_what_a_side_reference_write_left`).
    crate::service_land_side_reference_write::keep_what_a_side_reference_write_left(landing, announce, requests, seq);
    // A landing that ends before its intent is in the journal leaves its batch's objects on the scratch branch, and no restart
    //  can land them: only an intent lets a restart decide the batch from the branch, so a failed landing keeps the commit and
    //  removes the branch itself (`service_scratch_ref_of_a_dropped_batch.rs`). One that wrote its intent leaves it to the restart.
    // The record that names this batch's worktree (`service_scratch_ref_marker_of_a_batch.rs`) goes when the reference it names
    //  does, and stays beside a reference a failed landing had to leave where it is.
    end_scratch_ref_record(landing, requests, seq, result.as_ref().err());
    let published: bool = match result.as_ref() {
        Ok(outcomes) => any_landed(outcomes.as_slice()),
        Err(_failure) => false,
    };
    deferral.settle(published);
    // Every sync the landing made or handed to the background, with the time of each: the last thing the answer states about it.
    timing.syncs = crate::service_durable::take_syncs();
    return result;
}



/// Landing controls stay out of the release executable.
#[cfg(test)]
#[path = "service_land_tests.rs"]
mod tests;

/// Controls for the adoption of the batches an earlier release's service left open,
/// which land real batches with the fixtures above.
#[cfg(test)]
#[path = "earlier_service_journal_adopted_tests.rs"]
mod adoption_tests;

/// Controls for settling the batches an earlier release's service left open when the person asks.
#[cfg(test)]
#[path = "earlier_batches_settled_on_request_tests.rs"]
mod settle_request_tests;

/// Controls for a replay over the index the landing saw stay out of the release executable.
#[cfg(test)]
#[path = "service_land_witness_tests.rs"]
mod witness_tests;

/// Controls for a service whose branch write does not return stay out of the release executable.
#[cfg(test)]
#[path = "service_land_hung_write_tests.rs"]
mod hung_write_tests;

/// Controls for the worktree's head around a landing stay out of the release executable.
#[cfg(test)]
#[path = "service_land_head_tests.rs"]
mod head_tests;

/// Controls for the flushes of the Git children of a landing stay out of the release executable.
#[cfg(test)]
#[path = "service_land_fsync_tests.rs"]
mod fsync_tests;

/// Controls for the checks of a commit Git built stay out of the release executable.
#[cfg(test)]
#[path = "service_land_built_tests.rs"]
mod built_tests;

/// Controls for a write of one of the landing's own side references whose wait ends before the write does
/// stay out of the release executable.
#[cfg(test)]
#[path = "service_land_side_reference_write_tests.rs"]
mod side_reference_write_tests;

/// Controls for the push a landed commit leaves before its client is answered stay out of the release executable.
#[cfg(test)]
#[path = "service_land_owed_push_tests.rs"]
mod owed_push_tests;

/// Controls for the index update a landed batch owes stay out of the release executable.
#[cfg(test)]
#[path = "service_land_owed_index_tests.rs"]
mod owed_index_tests;

/// Controls for the restart decisions when a Git question about the branch fails stay out of the release executable.
#[cfg(test)]
#[path = "service_land_finish_git_failure_tests.rs"]
mod finish_git_failure_tests;

/// Controls for a restart that settles a held batch and then decides about it stay out of the release executable.
#[cfg(test)]
#[path = "service_land_finish_held_settlement_tests.rs"]
mod finish_held_settlement_tests;

/// Controls for the stat data of the index a landing installs stay out of the release executable.
#[cfg(test)]
#[path = "service_land_index_stat_tests.rs"]
mod index_stat_tests;

/// Controls for a commit whose client is gone before it lands stay out of the release executable.
#[cfg(test)]
#[path = "service_land_client_gone_tests.rs"]
mod client_gone_tests;
