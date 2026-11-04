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

/// The wrapper's commit path:
///  arguments,
///  preparation,
///  convergence,
///  conclusion and timing.
pub mod commit;

/// Re-exports every module of `commit` at the crate root, as above.
pub use commit::*;

/// Conflict handling around commits and checkouts:
///  side flags,
///  saved copies of conflicted files and leftover markers.
pub mod conflict;

/// Re-exports every module of `conflict` at the crate root, as above.
pub use conflict::*;

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

/// Controls that read the crate's own source text.
#[cfg(test)]
pub mod source_guard;

/// Re-exports every module of `source_guard` at the crate root, as above.
#[cfg(test)]
pub(crate) use source_guard::*;

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

/// In-process reading and writing of the Git index file and its extensions.
pub mod index;

/// Re-exports every module of `index` at the crate root, as above.
pub use index::*;

/// What the commit service does to land a commit:
///  the serialized step,
///  the index lock hold,
///  preparation,
///  replay and after-commit work.
pub mod landing;

/// Re-exports every module of `landing` at the crate root, as above.
pub use landing::*;

/// Merges:
///  the word-level second pass,
///  isolated merges and pull handling.
pub mod merge;

/// Re-exports every module of `merge` at the crate root, as above.
pub use merge::*;

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

/// The resident commit service:
///  protocol,
///  server,
///  client,
///  journal,
///  provisioning and logging.
pub mod service;

/// Re-exports every module of `service` at the crate root, as above.
pub use service::*;

/// Btrfs subvolume worktrees and the filesystem primitives under them.
pub mod subvolume;

/// Re-exports every module of `subvolume` at the crate root, as above.
pub use subvolume::*;

/// Package-manager and toolchain repair after a worktree is created.
pub mod repair;

/// Re-exports every module of `repair` at the crate root, as above.
pub use repair::*;
