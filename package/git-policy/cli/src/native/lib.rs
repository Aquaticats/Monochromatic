//! What:
//!  Internal native monochromatic-git-policy-cli implementation,
//!  not yet the production executable.
//! Why:
//!  The Rust rewrite preserves Git's own argument,
//!  repository and transaction semantics.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Internal modules used by the forthcoming native entry point and consumer verification.
//! ```
//!
//! Layout:
//!  Every folder below is one subsystem,
//!  and its `mod.rs` declares the modules inside it.
//!  A glob re-export beside each folder keeps the old flat paths (`crate::module_name`) working.
//!  `README.md` ("Source layout") says where a new file belongs.

//region What one command and its policies do

/// What one invocation does from arguments to Git:
///  the decision,
///  real Git resolution,
///  forwarding,
///  startup and management words.
pub mod invocation;

/// What:
///  `pub use invocation::*;` also names every module declared in `invocation/mod.rs` directly under the crate root,
///  with the visibility each module was declared with.
/// Why:
///  The modules moved into folders but keep their paths,
///  `crate::name` and `git_policy_cli::name`,
///  so no caller,
///  fuzz target or test changes and the move stays mechanical.
/// Gotcha:
///  A glob re-export is shadowed by a same-named item written in this file,
///  so no folder may share a name with a module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './invocation';
/// ```
pub use invocation::*;

/// Reading and validating `monochromatic-git-policy-cli.config.jsonc` and the Git configuration one invocation is judged by.
pub mod config;

/// The same re-export for `config`.
pub use config::*;

/// Per-command handling:
///  the option grammar of each Git command the wrapper understands,
///  and the rules that rewrite,
///  refuse or hint at a command.
pub mod command;

/// The same re-export for `command`.
pub use command::*;

/// The policy engine:
///  the registry,
///  triggers,
///  events,
///  the passes over candidates and direct fixes.
pub mod policy;

/// The same re-export for `policy`.
pub use policy::*;

/// The `markdown/autofix` policy and its in-process linter.
pub mod markdown;

/// The same re-export for `markdown`.
pub use markdown::*;

/// The dependent-version policy:
///  manifest and import facts,
///  the release plan and the policy over them.
pub mod dependent_version;

/// The same re-export for `dependent_version`.
pub use dependent_version::*;

/// The standards-review shadow run:
///  its request,
///  the patch it covers,
///  its one HTTPS call and the result it appends.
pub mod standards_review;

/// The same re-export for `standards_review`.
pub use standards_review::*;

/// Candidate content:
///  what a commit or push would add,
///  read from the index or objects,
///  and the scanner run over it.
pub mod candidate;

/// The same re-export for `candidate`.
pub use candidate::*;

//endregion

//region The commit path and the resident commit service

/// The wrapper's commit path:
///  arguments,
///  preparation,
///  convergence,
///  conclusion and timing.
pub mod commit;

/// The same re-export for `commit`.
pub use commit::*;

/// The shadow repository and the replay of captured changes onto it.
pub mod shadow;

/// The same re-export for `shadow`.
pub use shadow::*;

/// Durable transaction state:
///  owner records,
///  journal,
///  phases and recovery.
pub mod transaction;

/// The same re-export for `transaction`.
pub use transaction::*;

/// What the commit service does to land a commit:
///  the serialized step,
///  the index lock hold,
///  preparation,
///  replay and after-commit work.
pub mod landing;

/// The same re-export for `landing`.
pub use landing::*;

/// The resident commit service:
///  protocol,
///  server,
///  client,
///  journal,
///  provisioning and logging.
pub mod service;

/// The same re-export for `service`.
pub use service::*;

//endregion

//region Pushes, merges and conflicts

/// Pushes:
///  the automatic push after a commit,
///  manual push handling,
///  bounded Git queries,
///  retries and progress watching.
pub mod push;

/// The same re-export for `push`.
pub use push::*;

/// Git LFS support:
///  pre-push planning,
///  pointer scanning and bounded uploads.
pub mod lfs;

/// The same re-export for `lfs`.
pub use lfs::*;

/// Merges:
///  the word-level second pass,
///  isolated merges and pull handling.
pub mod merge;

/// The same re-export for `merge`.
pub use merge::*;

/// Conflict handling around commits and checkouts:
///  side flags,
///  saved copies of conflicted files and leftover markers.
pub mod conflict;

/// The same re-export for `conflict`.
pub use conflict::*;

//endregion

//region Worktrees, filesystems and reports

/// Worktree commands:
///  add,
///  remove,
///  clean,
///  convert,
///  trash,
///  snapshots and the placement ladder.
pub mod worktree;

/// The same re-export for `worktree`.
pub use worktree::*;

/// Copying the ignored state of a linked worktree into a new one,
///  with staging,
///  journal and recovery.
pub mod worktree_copying;

/// The same re-export for `worktree_copying`.
pub use worktree_copying::*;

/// Btrfs subvolume worktrees and the filesystem primitives under them.
pub mod subvolume;

/// The same re-export for `subvolume`.
pub use subvolume::*;

/// Whether the repository path is a usable link to the 860 drive,
/// and the gate that asks it before a command touches the repository.
#[cfg(target_os = "linux")]
pub mod repository_link_check;

/// The same re-export for `repository_link_check`.
#[cfg(target_os = "linux")]
pub use repository_link_check::*;

/// Package-manager and toolchain repair after a worktree is created.
pub mod repair;

/// The same re-export for `repair`.
pub use repair::*;

/// The `doctor` report and the checks of hooks and configuration it runs.
pub mod health;

/// The same re-export for `health`.
pub use health::*;

//endregion

//region Reading and locking the repository, processes and shared building blocks

/// In-process repository reads:
///  objects,
///  packs,
///  refs,
///  layout and configuration without starting Git.
pub mod repo_read;

/// The same re-export for `repo_read`.
pub use repo_read::*;

/// In-process reading and writing of the Git index file and its extensions.
pub mod index;

/// The same re-export for `index`.
pub use index::*;

/// Locks and their owners:
///  the index lock,
///  stale reference locks,
///  and the claims that let one command work on a worktree.
pub mod lock;

/// The same re-export for `lock`.
pub use lock::*;

/// Processes:
///  identities,
///  trees,
///  sockets,
///  signals and quiet-time ledgers.
pub mod process;

/// The same re-export for `process`.
pub use process::*;

/// Small shared building blocks:
///  private files,
///  scratch locations,
///  JSON,
///  time,
///  random ids,
///  paths and bounded parallelism.
pub mod foundation;

/// The same re-export for `foundation`.
pub use foundation::*;

//endregion

//region Test-only code, never part of the release executable

/// Test-only fixtures shared by many unit tests,
///  never part of the release executable.
#[cfg(test)]
pub mod testing;

/// The same re-export for `testing`.
#[cfg(test)]
pub(crate) use testing::*;

/// Controls that read the crate's own source text.
#[cfg(test)]
pub mod source_guard;

/// The same re-export for `source_guard`.
#[cfg(test)]
pub(crate) use source_guard::*;

//endregion
