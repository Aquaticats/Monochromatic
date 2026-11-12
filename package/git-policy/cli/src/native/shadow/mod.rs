//! What:
//!  The shadow repository and the replay of captured changes onto it.
//! Why:
//!  Commit preparation works on a shadow,
//!  so its helpers stay together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shadow/index.ts: one `export * from './file'` per module below.
//! ```

/// Git commands run against a transaction's shadow repository.
pub mod shadow_git;

/// Replay:
/// the unified-diff parser of subsumption (`replay_patch`).
pub mod replay_patch;

/// Replay:
/// the exact strict reverse check of subsumption (`replay_reverse_apply`).
pub mod replay_reverse_apply;

/// Replay:
/// the one-sided extension of subsumption over zero-context hunks (`replay_containment`).
pub mod replay_containment;

/// Replay:
/// capture order over the landed first-parent history and the per-path decision (`replay_capture_order`).
pub mod replay_capture_order;

/// Replay:
/// the shadow-repository Git runner that every replay step uses (`replay_shadow`).
pub mod replay_shadow;

/// Replay:
/// shared paths of the landed history and the prepared commit,
/// with their three entries (`replay_shared_paths`).
pub mod replay_shared_paths;

/// Replay:
/// the text half of subsumption over index-named blobs and hunks (`replay_subsumption_text`).
pub mod replay_subsumption_text;

/// Replay:
/// subsumption rules and the synthetic merge base (`replay_subsumption`).
pub mod replay_subsumption;

/// Replay:
/// the merge tree,
/// the first conflicting landed commit,
/// and the replayed commit rebuild (`replay_merge`).
pub mod replay_merge;

/// The empty tree's object name under each known object format.
pub mod empty_tree;

/// The shadow repository `config` file and the copied repository format.
pub mod shadow_config;

/// The shadow repository links to the real shared entries.
pub mod shadow_links;

/// The shadow ref store:
/// snapshot,
/// private target,
/// symbolic refs and `HEAD`.
pub mod shadow_refs;

/// Creation of the shadow repository of one transaction.
pub mod shadow_create;

/// The pool of finished shadows a commit can take instead of making one.
pub mod shadow_pool;

/// What a member of that pool must be for one invocation to take it.
pub mod shadow_pool_member;

/// The finished shadows the service draws out of the pool ahead of the commits that ask for one.
pub mod shadow_drawn;

/// The entries of an empty bare repository,
/// created in-process instead of by `git init`.
pub mod shadow_skeleton;
