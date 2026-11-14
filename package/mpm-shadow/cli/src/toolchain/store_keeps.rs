//! What:
//! The versions an install must not remove:
//! one a lock it can see names,
//! and one a running process of this user uses.
//! Why:
//! Two kinds of removal run during an install:
//! the size- and age-bounded cleanup (`store_trim.rs`),
//! and the newest-only removal of this repository's own tools,
//! which takes an older version away the moment a newer one is installed.
//! The second one is as old as the version-keyed layout,
//! and it asked no lock and no process table,
//! so it took away a version a worktree's lock pinned and a version a running service was executing.
//! Both now ask this one question,
//! and its answer is the rule `doc/toolchain-version-cleanup.md` states:
//! a lock that cannot be read,
//! and a process table that cannot be read,
//! keep every version.
//!
//! The name a process the kernel hides calls itself is deliberately not used here,
//! though the size-bounded pass uses it:
//! that pass judges only while the stores are over their bound,
//! while this rule runs whenever a newer version of a tool is installed,
//! so a hidden process whose name happened to match a tool's directory name would keep every version of that tool for
//! good.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // store-keeps.ts
//! export function toolDirectoryName(tool: LockedTool): string | undefined
//! export function keepsOf(ctx: Context): InstallKeeps
//! export function keepReason(keeps: InstallKeeps, toolDirectory: string, version: string, directory: string): string | undefined
//! ```

/// Imports `Path` and `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `Instant` from `std::time`.
use std::time::Instant;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `LockedTool` and `Route` from `super::model`.
use super::model::{LockedTool, Route};
/// Imports `package_name` from `super::observe`,
/// which names the crate a `cargo-binstall` entry installs.
use super::observe::package_name;
/// Imports what this module uses from `super::store_path_holders`.
use super::store_path_holders::{StoreHolders, held_reason, holders_of};
/// Imports what this module uses from `super::store_trim`,
/// so a removal reads the same locks and the same process table the cleanup does.
use super::store_trim::{NamedVersions, TrimBounds, read_named_versions, roots_to_trim, visible_locks};
/// Imports `install_directory_name` from `super::store_versions`,
/// which says which directory below a store root holds a tool's versions.
use super::store_versions::install_directory_name;

/// What:
/// The directory below a store root that holds the versions of `tool`,
/// relative to the root (`node`,
/// `cargo/apple-codesign`),
/// or `None` for a route that keeps its payload outside the store's version directories.
/// Why:
/// A removal has to name the tool the way the locks name it,
/// and that name is what a lock row becomes once it is read,
/// so both the lock reader and the removal ask here rather than each spelling the rule out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function toolDirectoryName(tool: LockedTool): string | undefined
/// ```
pub fn tool_directory_name(tool: &LockedTool) -> Option<String> {
    // What: a crate `cargo binstall` installs is named by the crate, and every other route by the tool id.
    // Why: its versions live under `<root>/cargo/<crate>`, while every other route's directory is named after the id.
    let name_source: String = match tool.declaration.route {
        Route::CargoBinstall => package_name(tool),
        _ => tool.declaration.id.clone(),
    };
    return install_directory_name(tool.declaration.route, &name_source);
}

/// What:
/// Everything a removal has to ask before it takes a version away:
/// the versions the visible locks name,
/// and the store paths a running process of this user holds.
/// Why:
/// One read answers both,
/// and a removal that read them twice could judge two versions by two different process tables.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallKeeps = { named: NamedVersions; holders: StoreHolders; unreadable?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InstallKeeps {
    /// The versions this scope's lock,
    /// every worktree's lock and the machine scope's lock name.
    named: NamedVersions,
    /// What the process table said.
    holders: StoreHolders,
    /// Why the locks or the process table could not be read,
    /// when they could not.
    /// A removal that cannot ask keeps every version,
    /// because the lock it could not read might name the version and the process it could not see might run it.
    unreadable: Option<String>,
}

/// The methods of `InstallKeeps`.
impl InstallKeeps {
    /// What:
    /// Reads the keeps of the store roots this scope can judge,
    /// now.
    /// Why:
    /// An install-time removal runs inside the install,
    /// so it reads the keeps itself rather than wait for the cleanup at the end:
    /// the version has to be kept before the newer one is installed, not after.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// export function keepsOf(ctx: Context): InstallKeeps
    /// ```
    pub fn of(ctx: &Context) -> InstallKeeps {
        // What: the roots and the locks are the cleanup's own, so both removals judge the same store.
        // Why: a removal that read fewer locks would take away a version of another worktree's tree that the cleanup keeps.
        let roots: Vec<PathBuf> = roots_to_trim(ctx);
        let bounds: TrimBounds = TrimBounds::from_env(ctx.env);
        let (lock_files, lock_problem) = visible_locks(ctx);
        let named: NamedVersions = match &lock_problem {
            // A lock nothing could list might name any version, and reading the others proves nothing.
            Some(_) => NamedVersions::default(),
            None => read_named_versions(&lock_files),
        };
        // What: the process scan gets the same work limit the cleanup gives its own scan.
        // Why: it runs inside an install, which must not wait on a process table of a machine under load.
        let holders: StoreHolders = holders_of(&roots, Instant::now() + bounds.work_limit);
        let mut unreadable: Option<String> = lock_problem;
        if unreadable.is_none() {
            unreadable = named.unreadable.clone();
        }
        return InstallKeeps { named, holders, unreadable };
    }

    /// What:
    /// Why `version` of the tool in `tool_directory` must not be removed,
    /// or `None` when nothing keeps it.
    /// Why:
    /// The reason is one sentence a person reads in a report or a warning,
    /// naming the lock or the process rather than the rule.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// export function keepReason(keeps: InstallKeeps, toolDirectory: string, version: string, directory: string): string | undefined
    /// ```
    pub fn keep_reason(&self, tool_directory: &str, version: &str, directory: &Path) -> Option<String> {
        if let Some(lock) = self.named.naming_lock(tool_directory, version) {
            // What: `naming_lock` answers with the file that pins this version, which is what a person checks.
            // Why: the file is the one place the pin is visible, so the sentence names it.
            return Some(format!("the lock {lock} names it"));
        }
        return self.in_use_reason(directory);
    }

    /// What:
    /// Why the version at `directory` must not be removed because a running process of this user uses it,
    /// and why every version is kept when the keeps could not be read,
    /// or `None` when no process uses it.
    /// Why:
    /// The newest-only rule across both toolchain roots and `prune --toolchain` ask this alone:
    /// a version a lock names is still removed there,
    /// because the owner's rule is that this repository's tools are one set across both roots and a tree whose lock
    /// pins an older version runs the newer one with a warning
    /// (`doc/decision/file-enforcer-rust-rewrite.md`).
    /// A version a process runs is kept by everything.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// export function inUseReason(keeps: InstallKeeps, directory: string): string | undefined
    /// ```
    pub fn in_use_reason(&self, directory: &Path) -> Option<String> {
        if let Some(reason) = &self.unreadable {
            // What: a read that failed keeps every version, and the line says which read it was.
            // Why: a removal that guessed would take away a version a lock it never read pins.
            return Some(format!("the locks and the process table could not be read: {reason}"));
        }
        if let Some(reason) = held_reason(&self.holders, directory) {
            // What: one sentence from the process scan, naming the process and what it holds.
            // Why: a version a service runs is not removed from under it; the line says which service.
            return Some(format!("it is in use: {reason}"));
        }
        return None;
    }
}
