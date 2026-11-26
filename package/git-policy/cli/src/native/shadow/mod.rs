//! What:
//!  The shadow repository and the replay of captured changes onto it.
//! Why:
//!  Commit preparation works on a shadow,
//!  so its helpers stay together.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `empty_tree.rs`:
//!   the empty tree's object name under each known object format.
//! - `replay_capture_order.rs`:
//!   replay:
//!   capture order over the landed first-parent history and the per-path decision (`replay_capture_order`).
//! - `replay_patch.rs`:
//!   replay:
//!   the unified-diff parser of subsumption (`replay_patch`).
//! - `replay_containment.rs`:
//!   replay:
//!   the one-sided extension of subsumption over zero-context hunks (`replay_containment`).
//! - `replay_reverse_apply.rs`:
//!   replay:
//!   the exact strict reverse check of subsumption (`replay_reverse_apply`).
//! - `shadow_config.rs`:
//!   the shadow repository `config` file and the copied repository format.
//! - `shadow_create_private_storage.rs`:
//!   the shadow's own directories and files,
//!   made private to the account.
//! - `shadow_create_template.rs`:
//!   the facts a shadow repository is made from,
//!   and the bytes of its `config` and alternates.
//! - `shadow_git.rs`:
//!   git commands run against a transaction's shadow repository.
//! - `replay_merge.rs`:
//!   replay:
//!   the merge tree,
//!   the first conflicting landed commit,
//!   and the replayed commit rebuild (`replay_merge`).
//! - `replay_shadow.rs`:
//!   replay:
//!   the shadow-repository Git runner that every replay step uses (`replay_shadow`).
//! - `replay_shared_paths.rs`:
//!   replay:
//!   shared paths of the landed history and the prepared commit,
//!   with their three entries (`replay_shared_paths`).
//! - `replay_subsumption_text.rs`:
//!   replay:
//!   the text half of subsumption over index-named blobs and hunks (`replay_subsumption_text`).
//! - `replay_subsumption.rs`:
//!   replay:
//!   subsumption rules and the synthetic merge base (`replay_subsumption`).
//! - `shadow_init_attempts.rs`:
//!   how many times one thread's shadow creations started `init`,
//!   counted for the controls.
//! - `shadow_create_init.rs`:
//!   the `git init` that makes a reftable shadow,
//!   and the starts made again after one was cut or refused.
//! - `shadow_links.rs`:
//!   the shadow repository links to the real shared entries.
//! - `shadow_drawn.rs`:
//!   the finished shadows the service draws out of the pool ahead of the commits that ask for one.
//! - `shadow_pool.rs`:
//!   the pool of finished shadows a commit can take instead of making one.
//! - `shadow_refs.rs`:
//!   the real refs the shadow's ref store is written from:
//!   the ref files when they can be read,
//!   and one `git for-each-ref` otherwise.
//! - `shadow_ref_write.rs`:
//!   the shadow ref store:
//!   snapshot,
//!   private target,
//!   symbolic refs and `HEAD`.
//! - `shadow_skeleton.rs`:
//!   the entries of an empty bare repository,
//!   created in-process instead of by `git init`.
//! - `shadow_create.rs`:
//!   creation of the shadow repository of one transaction.
//! - `shadow_pool_member.rs`:
//!   what a member of that pool must be for one invocation to take it.
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

/// The real refs the shadow's ref store is written from:
/// the ref files when they can be read,
/// and one `git for-each-ref` otherwise.
pub mod shadow_refs;

/// The shadow ref store:
/// snapshot,
/// private target,
/// symbolic refs and `HEAD`.
pub mod shadow_ref_write;

/// The shadow's own directories and files,
/// made private to the account.
pub mod shadow_create_private_storage;

/// The facts a shadow repository is made from,
/// and the bytes of its `config` and alternates.
pub mod shadow_create_template;

/// The `git init` that makes a reftable shadow,
/// and the starts made again after one was cut or refused.
pub mod shadow_create_init;

/// Creation of the shadow repository of one transaction.
pub mod shadow_create;

/// How many times one thread's shadow creations started `init`,
/// counted for the controls.
pub mod shadow_init_attempts;

/// The pool of finished shadows a commit can take instead of making one.
pub mod shadow_pool;

/// What a member of that pool must be for one invocation to take it.
pub mod shadow_pool_member;

/// The finished shadows the service draws out of the pool ahead of the commits that ask for one.
pub mod shadow_drawn;

/// The entries of an empty bare repository,
/// created in-process instead of by `git init`.
pub mod shadow_skeleton;
