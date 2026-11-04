//! What:
//!  Internal native cli-git implementation,
//!  not yet the production executable.
//! Why:
//!  The Rust rewrite preserves Git's own argument,
//!  repository and transaction semantics.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Internal modules used by the forthcoming native entry point and consumer verification.
//! ```

/// What one invocation does from arguments to Git:
///  the decision,
///  real Git resolution,
///  forwarding,
///  startup and management words.
pub mod invocation;

/// Re-exports every module of `invocation` at the crate root, as above.
pub use invocation::*;

/// Reading and validating `cli-git.config.jsonc` and the Git configuration one invocation is judged by.
pub mod config;

/// Re-exports every module of `config` at the crate root, as above.
pub use config::*;

/// Worktree commands:
///  add,
///  remove,
///  clean,
///  convert,
///  trash,
///  snapshots and the placement ladder.
pub mod worktree;

/// Re-exports every module of `worktree` at the crate root, as above.
pub use worktree::*;

/// The policy engine:
///  the registry,
///  triggers,
///  events,
///  the passes over candidates and direct fixes.
pub mod policy;

/// Re-exports every module of `policy` at the crate root, as above.
pub use policy::*;

/// In-process repository reads:
///  objects,
///  packs,
///  refs,
///  layout and configuration without starting Git.
pub mod repo_read;

/// Re-exports every module of `repo_read` at the crate root, as above.
pub use repo_read::*;

/// Locks and their owners:
///  the index lock,
///  stale reference locks,
///  and the claims that let one command work on a worktree.
pub mod lock;

/// Re-exports every module of `lock` at the crate root, as above.
pub use lock::*;

/// Pushes:
///  the automatic push after a commit,
///  manual push handling,
///  bounded Git queries,
///  retries and progress watching.
pub mod push;

/// Re-exports every module of `push` at the crate root, as above.
pub use push::*;

/// Test-only fixtures shared by many unit tests,
///  never part of the release executable.
#[cfg(test)]
pub mod testing;

/// Re-exports every module of `testing` at the crate root, as above.
#[cfg(test)]
pub(crate) use testing::*;

/// Per-command handling:
///  the option grammar of each Git command the wrapper understands,
///  and the rules that rewrite, refuse or hint at a command.
pub mod command;

/// What:
///  `pub use command::*;` also names every module declared in `command/mod.rs` directly under the crate root,
///  with the visibility each module was declared with.
/// Why:
///  The modules moved into folders but keep their paths (`crate::name`, `git_policy_cli::name`),
///  so no caller, fuzz target or test changes and the move stays mechanical.
/// Gotcha:
///  A glob re-export is shadowed by a same-named item written in this file,
///  so no folder may share a name with a module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './command';
/// ```
pub use command::*;

/// Pure core of the index-against-`HEAD` check.
pub mod rule_commit_index;

/// Pure decision of the commit-only transform.
pub mod rule_commit_only;

/// User-facing diagnostics of the commit-only transform.
pub mod rule_commit_only_message;

/// Pure core of the merge,
///  cherry-pick and revert conclusion check.
pub mod rule_commit_sequencer;

/// Conflict handling around commits and checkouts:
///  side flags,
///  saved copies of conflicted files and leftover markers.
pub mod conflict;

/// Re-exports every module of `conflict` at the crate root, as above.
pub use conflict::*;

/// Slice 6: writes one corrected file into the private commit index as a blob and an index entry.
pub mod private_index_stage;

/// Slice 6: the commit path's bounded policy convergence over the private index.
pub mod commit_convergence;

/// The commit transaction in the incumbent's order, up to the settled policies.
pub mod commit_transaction;

/// The conclusion of a settled commit transaction: preparation, landing and post-landing completion.
pub mod commit_conclusion;

/// The normalization-only transaction for a correction that removed every change.
pub mod commit_no_change;

/// The finding for a staged deletion that an explicit-path commit cannot record.
pub mod commit_staged_additions;
pub mod commit_staged_deletions;

/// The `git commit` command: the transaction, the settled events, the post-commit gate and the backup.
pub mod commit_command;

/// Rule facts from the real repository with candidates read from the private index.
pub mod commit_facts;

/// The worktree completion records of selected corrections and of hook changes.
pub mod commit_worktree_records;

/// The record of where one wrapped commit spent its time,
/// appended beside the discard audit.
pub mod commit_timing;

/// Processes:
///  identities,
///  trees,
///  sockets,
///  signals and quiet-time ledgers.
pub mod process;

/// Re-exports every module of `process` at the crate root, as above.
pub use process::*;

/// The dependent-version policy:
///  manifest and import facts,
///  the release plan and the policy over them.
pub mod dependent_version;

/// Re-exports every module of `dependent_version` at the crate root, as above.
pub use dependent_version::*;

/// The `markdown/autofix` policy and its in-process linter.
pub mod markdown;

/// Re-exports every module of `markdown` at the crate root, as above.
pub use markdown::*;

/// Candidate content:
///  what a commit or push would add,
///  read from the index or objects,
///  and the scanner run over it.
pub mod candidate;

/// Re-exports every module of `candidate` at the crate root, as above.
pub use candidate::*;

/// Small shared building blocks:
///  private files,
///  scratch locations,
///  JSON,
///  time,
///  random ids,
///  paths and bounded parallelism.
pub mod foundation;

/// Re-exports every module of `foundation` at the crate root, as above.
pub use foundation::*;

/// Durable transaction state:
///  owner records,
///  journal,
///  phases and recovery.
pub mod transaction;

/// Re-exports every module of `transaction` at the crate root, as above.
pub use transaction::*;

/// The ban on Git starting the wrapper for a hook: no entry writer, no start-name dispatch.
#[cfg(test)]
#[path = "hook_self_invocation_tests.rs"]
mod hook_self_invocation_tests;

/// The lock inventory of the service and its support modules: a lock stands only on the inventory, with its reason beside it.
#[cfg(test)]
#[path = "service_lock_inventory_tests.rs"]
mod service_lock_inventory_tests;

/// The `doctor` report and the checks of hooks and configuration it runs.
pub mod health;

/// Re-exports every module of `health` at the crate root, as above.
pub use health::*;

/// Git LFS support:
///  pre-push planning,
///  pointer scanning and bounded uploads.
pub mod lfs;

/// Re-exports every module of `lfs` at the crate root, as above.
pub use lfs::*;

/// The shadow repository and the replay of captured changes onto it.
pub mod shadow;

/// Re-exports every module of `shadow` at the crate root, as above.
pub use shadow::*;

/// Native conclusion cleanup reproduced in the owning worktree.
pub mod conclusion_cleanup;

/// The invocation capture:
///  repository layout, symbolic `HEAD`, conclusion kind, ref backend and base.
pub mod capture_invocation;

/// The private index and object store a commit's preparation selects, and the Git request that carries them.
pub mod private_index_request;

/// The shadow conclusion copies at preparation.
pub mod conclusion_copy;

/// Argument transforms of native preparation: normalisation, global split, `-a` removal and private arguments.
pub mod commit_args;

/// The private commit index: copies, selection modes, pathspec files and listings.
pub mod commit_index;

/// Native `git commit` in the shadow, the prepared commit read back and `prepared.json`.
pub mod commit_preparation;

/// In-process reading and writing of the Git index file and its extensions.
pub mod index;

/// Re-exports every module of `index` at the crate root, as above.
pub use index::*;

/// The landing location a forwarded index writer coordinates with:
///  the real index and the registry.
pub mod landing_location;

/// Merges:
///  the word-level second pass,
///  isolated merges and pull handling.
pub mod merge;

/// Re-exports every module of `merge` at the crate root, as above.
pub use merge::*;

/// Automatic maintenance decision: config record parsing and the enable and detach rules.
pub mod maintenance_decision;

/// Running automatic maintenance after a landing: the config read, then `git maintenance run --auto`.
pub mod maintenance_run;

/// The post-commit gate over a landed commit: its events, its blocking and the backup it allows.
pub mod post_commit;

/// Copying the ignored state of a linked worktree into a new one,
///  with staging,
///  journal and recovery.
pub mod worktree_copying;

/// Re-exports every module of `worktree_copying` at the crate root, as above.
pub use worktree_copying::*;

/// The standards-review shadow run:
///  its request,
///  the patch it covers,
///  its one HTTPS call and the result it appends.
pub mod standards_review;

/// Re-exports every module of `standards_review` at the crate root, as above.
pub use standards_review::*;

/// The durable work-item record for asynchronous work (review, push, maintenance) left for a resident consumer.
pub mod work_item;
pub mod work_item_recovery;

/// The clocks a claim's lease is judged by.
pub mod claim_time;

/// The repositories the service looks at again when a claim another instance holds may have run out.
pub mod service_retry;

/// Messages between the wrapper and the resident commit service.
pub mod service_protocol;

/// What the user is told when the commit service cannot land a commit.
pub mod service_messages;

/// The longest the wrapper waits for each thing it waits for, with the measurement beside each number.
pub mod wait_ceilings;

/// The connection of one waiting client, answered by the landing thread and kept alive by a beat from the reading thread.
pub mod service_reply;

/// The commit service's journal and its restart decisions.
pub mod service_journal;

/// Test-only fault points of the commit service.
pub mod service_fault;

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

/// The native journal protocol and the standard error fallback the service's entries go through.
pub mod journald_native;

/// The lines the commit service writes about its life and about the time each request took.
pub mod service_timing;

/// The line the commit service writes when a signal ends it.
pub mod service_signal;

/// The commit service's loop.
pub mod service_server;

/// The wrapper's side of a commit through the commit service.
pub mod service_client;

/// Which commit command lines the commit service covers, and the message they stand for.
pub mod service_commit_args;

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

/// The wrapper's own provisioning of the commit service's socket activation.
pub mod service_provision;

/// A minimal D-Bus client for the user session bus, for the provisioning's two calls.
pub mod systemd_dbus;

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

/// The drain of several repositories of one round, side by side and without a lock.
pub mod service_round;

/// The wrapper's side of after-commit work: records and the wake.
pub mod service_handoff;

/// Which commit service instance belongs to a worktree: its name, socket and unit instances.
pub mod service_instance;

/// Whether a worktree's head still names the branch a commit was prepared on.
pub mod service_head;

/// How a commit headed for the service notices that its prepared content was removed before it landed.
pub mod service_pruned;

/// One entry from the wrapper a person ran to the journal, sent without waiting.
#[cfg(target_os = "linux")]
pub mod client_journal;

/// The mechanical ban: no release code starts the wrapper, passes it management words or writes shell text.
#[cfg(test)]
#[path = "no_self_invocation_tests.rs"]
mod no_self_invocation_tests;

/// The structural check that the review's network client is reachable only from the service run; never in the release build.
#[cfg(test)]
mod standards_review_boundary_tests;

/// The shared base of the checks that read the crate's own source text: a strict walk and the module tree, so a check
/// is right for nested module directories; never in the release build.
#[cfg(test)]
mod source_scan_tests;

/// The service's handler for a review work item, bounded by a call deadline and a total budget.
pub mod service_review;

/// The standards-review credential: redacted when printed, read from the wrapper's environment, sent in the work message.
pub mod service_review_key;

/// Btrfs subvolume worktrees and the filesystem primitives under them.
pub mod subvolume;

/// Re-exports every module of `subvolume` at the crate root, as above.
pub use subvolume::*;

/// Package-manager and toolchain repair after a worktree is created.
pub mod repair;

/// Re-exports every module of `repair` at the crate root, as above.
pub use repair::*;
