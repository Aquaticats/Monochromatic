//! What:
//!  The wrapper's commit path:
//!  arguments,
//!  preparation,
//!  convergence,
//!  conclusion and timing.
//! Why:
//!  A commit is the central transaction of the wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // commit/index.ts: one `export * from './file'` per module below.
//! ```

/// Pure core of the index-against-`HEAD` check.
pub mod rule_commit_index;

/// Pure decision of the commit-only transform.
pub mod rule_commit_only;

/// User-facing diagnostics of the commit-only transform.
pub mod rule_commit_only_message;

/// Pure core of the merge,
///  cherry-pick and revert conclusion check.
pub mod rule_commit_sequencer;

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

/// The post-commit gate over a landed commit: its events, its blocking and the backup it allows.
pub mod post_commit;

/// CRC-32, hexadecimal spellings, SHA-256 hex and identity checks of the commit protocol's durable records
/// (`doc/planning/cli-git-commits-never-fail.md`, step 1; nothing here is used by the live protocol yet).
#[cfg(target_os = "linux")]
pub mod commit_integrity;

/// The permanent append-only ledger of retired commit requests, with torn-tail repair and no deletion.
#[cfg(target_os = "linux")]
pub mod commit_ledger;

/// The shared durable-file helpers of the commit protocol's record components (temporary names, directory sync chains, the initialized
/// root, the test seams); the permanent identity binding it once held is removed.
#[cfg(target_os = "linux")]
pub mod commit_identity;

/// The immutable commit request record and its strict codec; publication is the inbox's `accept`.
#[cfg(target_os = "linux")]
pub mod commit_request;

/// The spool of a request's captured bytes, with a manifest and a verifier and no reclamation.
#[cfg(target_os = "linux")]
pub mod commit_spool;

/// Pure tree comparison and captured-delta replay over flattened trees.
#[cfg(target_os = "linux")]
pub mod commit_replay;

/// A request as one checksummed file in `inbox/`, the departure barrier and the content-hash identity (rule V, revision 12).
#[cfg(target_os = "linux")]
pub mod commit_inbox;

/// The state fields of the lane's real state document: marker, logical head and tail, request states and the consumption map.
#[cfg(target_os = "linux")]
pub mod commit_lane_state;

/// Revision 13's hash-chained append-only state log with checkpoint rotation (unused by the live protocol).
#[cfg(target_os = "linux")]
pub mod commit_state_log;

/// A detached helper launch (two forks, a new session, `close_range`) that is a function of this program and never a start of it.
#[cfg(target_os = "linux")]
pub mod commit_helper_launch;

/// The one-shot receipt flusher: the internal flush entry point under `lane/flusher.lock`.
#[cfg(target_os = "linux")]
pub mod commit_flusher;
