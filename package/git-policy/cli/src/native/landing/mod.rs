//! What:
//!  What the commit service does to land a commit:
//!  the serialized step,
//!  the index lock hold,
//!  preparation,
//!  replay and after-commit work.
//! Why:
//!  The landing is the service's core and changes together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/index.ts: one `export * from './file'` per module below.
//! ```

/// The landing location a forwarded index writer coordinates with:
///  the real index and the registry.
pub mod landing_location;

/// Automatic maintenance decision: config record parsing and the enable and detach rules.
pub mod maintenance_decision;

/// Running automatic maintenance after a landing: the config read, then `git maintenance run --auto`.
pub mod maintenance_run;

/// The durable work-item record for asynchronous work (review, push, maintenance) left for a resident consumer.
pub mod work_item;

/// The state folders of the work queue, the names of claim files, and the listing of a folder oldest first.
pub mod work_item_state_directories_and_claim_names;

pub mod work_item_recovery;

/// The clocks a claim's lease is judged by.
pub mod claim_time;

/// The repositories the service looks at again when a claim another instance holds may have run out.
pub mod service_retry;

/// The Git directories the service looks at by itself when it starts.
pub mod service_claim_watch;

/// What the user is told when the commit service cannot land a commit.
pub mod service_messages;

/// The commit service's serialized step.
pub mod service_land;

/// What the commit service's landing does once the branch has moved: index entries and conclusion state.
pub mod service_land_copy;

pub mod service_land_follow;

pub mod service_land_start;

/// The commit service's hold on Git's index lock across a landing.
pub mod service_index_lock;

/// The attempts and the wait of an acquisition of the index lock.
pub mod service_index_attempts;

/// The marker the service writes into the index lock and the questions asked of a marker.
pub mod service_index_marker;

/// The marker of the next batch's index lock, made and flushed while the service is idle.
pub mod service_index_prestage;

/// What an acquisition of the index lock found when it had to wait, for the service's log.
pub mod service_index_found;

/// Who a marker's holder is: the identity a process writes, and the answers a reader gives (gone, running, unknown).
pub mod service_index_identity;

/// The service's hold on a placed index lock: the check that it is still the holder's, the release and the removal.
pub mod service_index_hold;

/// Whether the index lock is still the file the holder placed, and the record of a lock that could not be settled.
pub mod service_index_ownership;

/// The staging files of the index lock.
pub mod service_index_staging;

/// How the service's marked file becomes the index lock, and the cleanup of staging files.
pub mod service_index_placement;

/// The inotify watch the wait for the index lock blocks on.
pub mod service_index_watch;

/// The preparation of a commit that the service lands, for every command line.
pub mod service_prepare;

/// Which commit command lines the commit service covers, and the message they stand for.
pub mod service_commit_args;

/// Whether a `commit` command line opens an editor.
pub mod service_commit_args_editor_opening;

/// The one summary line a landed commit prints, with the abbreviated name Git would print.
pub mod landed_summary;

/// The wait of a wrapped index writer for the commit service's own hold on the index.
pub mod service_index_wait;

/// A forwarded index writer started again when the commit service took the index before its Git did.
pub mod service_hold_retry;

/// The commit command's path through the commit service.
pub mod service_commit;

/// The service's memory of when the bytes of the commits it landed were captured, per worktree.
pub mod service_captures;

/// The retry of a commit the service sent back: the captured change merged onto the branch, never prepared again from the worktree.
pub mod service_replay;

/// Which way a commit goes: through the service, set up first, or the earlier path.
pub mod service_route;

/// The message and identities of a service commit, made as native `git commit` makes them.
pub mod service_message;

/// What an amend takes from the commit it replaces: parents, message and author.
pub mod service_amend;

/// The commits real Git builds for the service: an editor, a signature, and what the service does not embed.
pub mod service_native;

/// The renewal that keeps the claims of a shared work directory from looking abandoned while their holder works.
pub mod service_claim_renewal;

/// The service's consumer of after-commit push and upkeep work.
pub mod service_work;

/// The kinds, handlers, acceptance rules, settling and replanning of the service's work.
pub mod service_work_handlers_replanning_and_acceptance;

/// Drains the pending items of one work directory.
pub mod service_work_drain_pending_claims;

/// Runs one round of the service's work over several directories.
pub mod service_work_run_round;

/// The drain of several repositories of one round, side by side and without a lock.
pub mod service_round;

/// The wrapper's side of after-commit work: records and the wake.
pub mod service_handoff;

/// Whether a worktree's head still names the branch a commit was prepared on.
pub mod service_head;

/// What a commit Git built adds to the branch, answered from the object store without starting Git.
pub mod service_built_checks;

/// How a commit headed for the service notices that its prepared content was removed before it landed.
pub mod service_pruned;

/// Where the landing's own syncs wait: queued for a background thread when the landing publishes, made before return otherwise.
pub mod service_durable;

pub mod service_land_finish;

/// The removal of a stale index lock, by a rename and a second look.
pub mod service_index_reclaim;

/// The claim chain that keeps two reclaimers apart where the file system gives no `flock`.
pub mod service_index_claim;

/// Whether a new index file holds the live index's bytes.
pub mod service_index_same;
