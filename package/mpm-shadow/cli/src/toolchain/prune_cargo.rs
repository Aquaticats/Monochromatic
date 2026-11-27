//! What:
//! The cleanup step for `cargo-binstall` installs:
//! `mpm-shadow prune --toolchain` deletes the version directories under `<toolchain root>/cargo` that no known lock
//! names,
//! and for a crate this repository publishes keeps only the newest installed version;
//! the same holds for a release archive of this repository that a lock names (`published_archive.rs`).
//! Why:
//! Version-keyed installs let trees with different locks coexist,
//! so nothing removes a version when the last lock naming it moves on.
//! The known locks are this scope's own lock,
//! the lock of every worktree `git worktree list` reports for the repository,
//! and any file given with `--lock`;
//! the command refuses to delete anything when a listed worktree's lock cannot be read,
//! because an unreadable lock might name a version that is about to be deleted,
//! and a file given with `--lock` names versions or refuses the command:
//! it may not be absent,
//! since a file that said what to keep cannot be read as keeping nothing.
//! A crate directory the scan cannot read refuses the command too:
//! reading it as empty would remove the whole directory,
//! including a version a lock names.
//! The work is one file per step,
//! named for what it holds:
//! the rule that decides what goes (`the_rule_that_decides_what_goes`),
//! what the known locks name (`what_the_known_locks_name`),
//! the versions on disk (`the_versions_on_disk`),
//! what a failed run says it removed (`what_a_failed_run_says_it_removed`),
//! the prune itself (`the_prune_itself`),
//! and the command a person runs (`the_prune_command`).
//! A caller imports from this file and never from a part.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // prune-cargo.ts
//! export function planPrune(present: Map<string, Present[]>, named: Map<string, Set<string>>): Removal[]
//! export function namedVersions(lockText: string): Map<string, Set<string>>
//! export function pruneToolchain(scope: Scope, runner: Runner, extraLocks: string[], dryRun: boolean): string[]
//! export * from './prune_cargo/the_rule_that_decides_what_goes.ts';
//! export * from './prune_cargo/what_the_known_locks_name.ts';
//! export * from './prune_cargo/the_versions_on_disk.ts';
//! export * from './prune_cargo/what_a_failed_run_says_it_removed.ts';
//! export * from './prune_cargo/the_prune_itself.ts';
//! export * from './prune_cargo/the_prune_command.ts';
//! ```

/// The `prune --toolchain` command itself.
mod the_prune_command;
/// The deletion of the versions no lock names.
mod the_prune_itself;
/// The rule that decides which versions go,
/// and which version of one crate the link may keep.
mod the_rule_that_decides_what_goes;
/// The versions installed on disk now.
mod the_versions_on_disk;
/// What a failed run says it has already removed.
mod what_a_failed_run_says_it_removed;
/// The cargo versions and release-archive tools the known locks name.
mod what_the_known_locks_name;

/// Re-exports the command `cli.rs` hands the parsed arguments to.
pub use self::the_prune_command::run_prune_command;
/// Re-exports the cleanup of the version directories no lock names.
pub use self::the_prune_itself::prune_toolchain;
/// Re-exports the versions one crate has installed,
/// and the rule that decides which of them go.
pub use self::the_rule_that_decides_what_goes::{
    PresentVersion,
    plan_prune,
};
/// Re-exports the scan of the versions installed now,
/// which the prune runs again after its deletions.
#[cfg(test)]
pub(crate) use self::the_versions_on_disk::present_versions;
/// Re-exports the restatement of a failure that follows a removal.
#[cfg(test)]
pub(crate) use self::what_a_failed_run_says_it_removed::after_removing;
/// Re-exports what every known lock names,
/// which a locked worktree lost to another run is allowed to be missing from.
#[cfg(test)]
pub(crate) use self::what_the_known_locks_name::named_by_all;
/// Re-exports what one lock's text names.
pub use self::what_the_known_locks_name::named_versions;
